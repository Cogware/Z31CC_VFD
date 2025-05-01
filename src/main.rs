#![no_std]
#![no_main]

extern crate alloc;

use z31_hvac::temp::Thermistor;
use z31_hvac::*;

use core::cell::RefCell;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output, Pull};
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::InterruptHandler as PIOInt;
use embassy_rp::spi;
use embassy_rp::spi::Spi;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Duration, Ticker};
use embassy_rp::adc::{Adc, Channel, Config};

use display::Display;
use embedded_alloc::Heap;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct PIOIrqs {
    PIO0_IRQ_0 => PIOInt<PIO0>;
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

    // let acset = false;
    // let recircset = false;
    // let defset = false;

    // Setup pio state machine for i2s output
    /*let Pio {
        mut common, sm0, ..
    } = Pio::new(p.PIO0, Irqs);*/

    let adc = Adc::new_blocking(p.ADC, Config::default());

    let ambtemp = Channel::new_pin(p.PIN_26, Pull::None);

    let mut therm = Thermistor::new(ambtemp, adc);

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

    let spibus: Mutex<CriticalSectionRawMutex, RefCell<_>> = Mutex::new(RefCell::new(spi));

    let vfd_spi = SpiDeviceWithConfig::new(&spibus, cs, config1);

    let mut vfd = Display::new(vfd_spi, rst);

    //vfd.set_brightness(128).unwrap();
    // let mut ticker = Ticker::every(Duration::from_secs(1));

    /*const NUM_LEDS: usize = 1;
    let mut data = [RGB8::default(); NUM_LEDS];
    let program = PioWs2812Program::new(&mut common);
    let mut ws2812 = PioWs2812::new(&mut common, sm0, p.DMA_CH1, p.PIN_21, &program);*/

    // Wrap flash as block device
    let mut ticker = Ticker::every(Duration::from_millis(25));

    vfd.draw_boot_image().await;

    loop {



        vfd.ambient_temp = therm.measure_temp() as i8;

        vfd.update_display();

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
