#![no_std]
#![no_main]

extern crate alloc;

use embassy_rp::pio_programs::ws2812::{PioWs2812, PioWs2812Program};
use smart_leds::RGB8;
use z31_hvac::climatecontrol::ClimateControlBacker;
use z31_hvac::digidisplay::DigiDisplay;
use z31_hvac::temp::Thermistor;
use z31_hvac::*;

use core::cell::RefCell;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_executor::{Spawner, task};
use embassy_rp::adc::{Adc, Channel, Config};
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler as PIOInt, Pio};
use embassy_rp::spi;
use embassy_rp::spi::Spi;
use embassy_rp::{bind_interrupts, block, i2c};
use embassy_time::{Duration, Ticker, Timer, block_for};

use embedded_alloc::Heap;
use vfddisplay::Display;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct PIOIrqs {
    PIO0_IRQ_0 => PIOInt<PIO0>;
});

#[global_allocator]
static HEAP: Heap = Heap::empty();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(&raw mut HEAP_MEM as usize, HEAP_SIZE) }
    }

    let p = embassy_rp::init(Default::default());

    // Setup pio state machine for i2s output
    let Pio {
        mut common, sm0, ..
    } = Pio::new(p.PIO0, PIOIrqs);

    let sda = p.PIN_2;
    let scl = p.PIN_3;
    let serialclock = Output::new(p.PIN_6, Level::Low);
    let serialdata = Output::new(p.PIN_5, Level::Low);
    let i2c = i2c::I2c::new_blocking(p.I2C1, scl, sda, embassy_rp::i2c::Config::default());

    spawner.spawn(digidisplay::serialsyncer()).unwrap();
    let backend = ClimateControlBacker::default();
    let mut digidisp = DigiDisplay::new(i2c, serialclock, serialdata);

    const NUM_LEDS: usize = 1;
    let mut data = [RGB8::default(); NUM_LEDS];
    let program = PioWs2812Program::new(&mut common);
    let mut ws2812 = PioWs2812::new(&mut common, sm0, p.DMA_CH1, p.PIN_21, &program);


    

    /*let adc = Adc::new_blocking(p.ADC, Config::default());
    let ambtemp = Channel::new_pin(p.PIN_26, Pull::None);
    let onboardts = Channel::new_temp_sensor(p.ADC_TEMP_SENSOR);
    let mut therm = Thermistor::new(ambtemp, onboardts, adc);*/
    //let vals = therm.measure_temp();
    //vfd.ambient_temp = vals[0] as i8;
    //vfd.internal_temp = vals[1] as i8;

    loop {
        for j in 0..(256 * 5) {
            for i in 0..NUM_LEDS {
                data[i] = wheel((((i * 256) as u16 / NUM_LEDS as u16 + j as u16) & 255) as u8);
            }
            ws2812.write(&data).await;
            digidisp.update_display(&backend).await
        }
    }
}


//-----------------------------------------VFD stuff for later to be in a seperate fn------------------------------------------
    /*let sclk = p.PIN_22;
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

    vfd.set_brightness(128).unwrap();
    vfd.draw_boot_image().await;
    vfd.update_display();*/