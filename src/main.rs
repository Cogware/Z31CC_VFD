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
use graphics::Display;
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
    /*let Pio {
        mut common, sm0, ..
    } = Pio::new(p.PIO0, Irqs);*/



    let sclk = p.PIN_22;
    let mosi = p.PIN_23;
    let miso = p.PIN_20;
    let cs = Output::new(p.PIN_25, Level::High);
    let rst = Output::new(p.PIN_24, Level::High);

    let mut config = spi::Config::default();
    config.frequency = 4_000_000;

    let mut config1 = spi::Config::default();
    config1.frequency = 4_000_000;

    let spi = Spi::new_blocking(p.SPI0, sclk, mosi, miso, config);

    let spibus: Mutex<NoopRawMutex, RefCell<_>> = Mutex::new(RefCell::new(spi));

    let vfd_spi = SpiDeviceWithConfig::new(&spibus, cs, config1);

    let mut vfd = Display::new(vfd_spi, rst);
    

    //vfd.set_brightness(128).unwrap();
    let mut ticker = Ticker::every(Duration::from_secs(1));

    /*const NUM_LEDS: usize = 1;
    let mut data = [RGB8::default(); NUM_LEDS];
    let program = PioWs2812Program::new(&mut common);
    let mut ws2812 = PioWs2812::new(&mut common, sm0, p.DMA_CH1, p.PIN_21, &program);*/

    // Wrap flash as block device
    let mut ticker = Ticker::every(Duration::from_millis(25));

    vfd.displaybootimage().await;

    loop {
        vfd.testdisplay();
        vfd.updatedisplay();

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


