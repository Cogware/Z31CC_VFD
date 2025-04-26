#![no_std]
#![no_main]

mod graphics;

extern crate alloc;

use core::cell::RefCell;
use eei_vfd::gp1287bi::VFD256x50;
use eei_vfd::prelude::EEIDisplay;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::pio_programs::ws2812::{PioWs2812, PioWs2812Program};
use embassy_rp::spi;
use embassy_rp::spi::Spi;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::{Delay, Duration, Instant, Ticker, Timer};
use embedded_graphics::framebuffer::{Framebuffer, buffer_size};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::{FONT_10X20, FONT_4X6, FONT_6X12, FONT_6X9, FONT_7X13, FONT_8X13_BOLD};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::pixelcolor::raw::LittleEndian;
use embedded_graphics::primitives::PrimitiveStyle;

use embedded_graphics::{
    framebuffer,
    image::Image,
    mono_font::{MonoTextStyleBuilder, ascii::FONT_6X10},
    prelude::*,
    text::Text,
};
use embedded_graphics_transform::Transpose;
use graphics::{set_fanguage, set_tempguage};
use smart_leds::RGB8;
use tinybmp::Bmp;
use core::alloc::Layout;
use embedded_alloc::Heap;
use alloc::string::String;
use alloc::format;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

const FAIRLADYBMP: &'static [u8] = include_bytes!("../assets/fairlady.bmp");
const CCBACKGROUND: &'static [u8] = include_bytes!("../assets/ClimateControlBackground.bmp");
const CCFACE: &'static [u8] = include_bytes!("../assets/Face.bmp");
const CCFEET: &'static [u8] = include_bytes!("../assets/Feet.bmp");
const CCFACEFEET: &'static [u8] = include_bytes!("../assets/FaceandFeet.bmp");


#[global_allocator]
static HEAP: Heap = Heap::empty();


#[embassy_executor::main]
async fn main(_spawner: Spawner) {

    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(&raw mut HEAP_MEM as usize, HEAP_SIZE) }
    }

    let p = embassy_rp::init(Default::default());
    

    let acset = false;
    let recircset = false;
    let defset = false;


    // Setup pio state machine for i2s output
    let Pio {
        mut common, sm0, ..
    } = Pio::new(p.PIO0, Irqs);

    let bootimage = Bmp::from_slice(FAIRLADYBMP).unwrap();
    let background = Bmp::from_slice(CCBACKGROUND).unwrap();
    //let face = Bmp::from_slice(CCFACE).unwrap();
    //let feet = Bmp::from_slice(CCFEET).unwrap();
    let facefeet = Bmp::from_slice(CCFACEFEET).unwrap();

    let sclk = p.PIN_22;
    let mosi = p.PIN_23;
    let miso = p.PIN_20;
    let cs = Output::new(p.PIN_25, Level::High);
    let rst = Output::new(p.PIN_24, Level::High);

    let mut config = spi::Config::default();
    config.frequency = 4_000_000;

    let spi = Spi::new_blocking(p.SPI0, sclk, mosi, miso, config);

    let bus: Mutex<NoopRawMutex, RefCell<_>> = Mutex::new(RefCell::new(spi));

    let mut config1 = spi::Config::default();
    config1.frequency = 4_000_000;

    let mut vfd_spi = SpiDeviceWithConfig::new(&bus, cs, config1);

    let mut delay = Delay;

    let mut vfd: VFD256x50<_, _, _> = EEIDisplay::new(&mut vfd_spi, rst, &mut delay).unwrap();

    vfd.clear_frame().unwrap();
    //vfd.set_brightness(128).unwrap();
    let mut ticker = Ticker::every(Duration::from_secs(1));

    let fb = Framebuffer::<
        BinaryColor,
        _,
        LittleEndian,
        128,
        256,
        { buffer_size::<BinaryColor>(128, 256) },
    >::new();

    let mut vfdfb = Transpose::new(fb);


    Image::new(&bootimage, Point::new(5, 14)).draw(&mut vfdfb).unwrap();

    vfd.update_frame(vfdfb.data()).unwrap();
    vfd.set_brightness(128).unwrap();
    ticker.next().await;
    vfd.set_brightness(255).unwrap();
    ticker.next().await;

    let tempfont = MonoTextStyle::new(&FONT_8X13_BOLD, BinaryColor::On);
    let offfont = MonoTextStyle::new(&FONT_7X13, BinaryColor::On);
    let fill = PrimitiveStyle::with_fill(BinaryColor::On);

    /*const NUM_LEDS: usize = 1;
    let mut data = [RGB8::default(); NUM_LEDS];
    let program = PioWs2812Program::new(&mut common);
    let mut ws2812 = PioWs2812::new(&mut common, sm0, p.DMA_CH1, p.PIN_21, &program);*/

    // Wrap flash as block device
    let mut ticker = Ticker::every(Duration::from_millis(25));
    vfd.set_brightness(128).unwrap();

    let mut fanval = 0; 
    let mut tempval = 0;
    let mut internaltemp = 60;
    let mut ambtemp = 60;

    loop {
        if fanval >= 32{
            fanval = 0;
        } else {
            fanval = fanval + 1;
        }
        if tempval >= 36{
            tempval = 0;
        } else {
            tempval = tempval + 1;
        }
        if internaltemp >= 99{
            internaltemp = 60;
        } else {
            internaltemp = internaltemp + 1;
        }
        if ambtemp >= 99{
            ambtemp = -60
        } else {
            ambtemp = ambtemp + 1;
        }

    
        vfd.clear_frame().unwrap();
        vfdfb.clear(BinaryColor::Off).unwrap();

        Image::new(&background, Point::new(0, 0)).draw(&mut vfdfb).unwrap();
        Image::new(&facefeet, Point::new(135, 2)).draw(&mut vfdfb).unwrap();
        Text::new(&format!("{:?}", internaltemp), Point::new(43,12), tempfont).draw(&mut vfdfb).unwrap();

        let negshift = 8;
        if ambtemp <= -1{
            Text::new(&format!("{:?}", ambtemp), Point::new(43 - negshift,37), tempfont).draw(&mut vfdfb).unwrap();
        } else{
        Text::new(&format!("{:?}", ambtemp), Point::new(43,37), tempfont).draw(&mut vfdfb).unwrap();
        }

        Text::new("OFF", Point::new(105,19), offfont).draw(&mut vfdfb).unwrap();
        Text::new("ON", Point::new(107,44), tempfont).draw(&mut vfdfb).unwrap();
        
        set_tempguage(&mut vfdfb, tempval);
        set_fanguage(&mut vfdfb, fanval);

        vfd.update_frame(vfdfb.data()).unwrap();

        ticker.next().await;
        /*for j in 0..(256 * 5) {
            for i in 0..NUM_LEDS {
                data[i] = wheel((((i * 256) as u16 / NUM_LEDS as u16 + j as u16) & 255) as u8);
            }
            ws2812.write(&data).await;

            ticker.next().await;
        }*/
    }
}


