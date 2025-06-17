#![no_std]
#![no_main]

extern crate alloc;

use embassy_rp::pio_programs::ws2812::{PioWs2812, PioWs2812Program};
use smart_leds::RGB8;
use z31_hvac::temp::Thermistor;
use z31_hvac::*;

use core::cell::RefCell;
use core::u128;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_executor::{Spawner, task};
use embassy_rp::adc::{Adc, Channel, Config};
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{PIN_9, PIN_10, PIO0};
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

    let sda = p.PIN_2;
    let scl = p.PIN_3;
    let mut i2c = i2c::I2c::new_blocking(p.I2C1, scl, sda, embassy_rp::i2c::Config::default());
    let chipaddr: u8 = 0x38;
    block_for(Duration::from_millis(10));

    i2c.blocking_write(chipaddr, &[0x49]).unwrap();
    i2c.blocking_write(chipaddr, &[0x00, 0xFF, 0xFF, 0xFF, 0xFF])
        .unwrap();

    // let acset = false;
    // let recircset = false;
    // let defset = false;

    // Setup pio state machine for i2s output
    let Pio {
        mut common, sm0, ..
    } = Pio::new(p.PIO0, PIOIrqs);

    /*let adc = Adc::new_blocking(p.ADC, Config::default());

    let ambtemp = Channel::new_pin(p.PIN_26, Pull::None);
    let onboardts = Channel::new_temp_sensor(p.ADC_TEMP_SENSOR);

    let mut therm = Thermistor::new(ambtemp, onboardts, adc);

    let sclk = p.PIN_22;
    let mosi = p.PIN_23;
    let miso = p.PIN_20;

    let cs = Output::new(p.PIN_25, Level::High);
    let rst = Output::new(p.PIN_24, Level::High);*/

    //let mut config = spi::Config::default();
    //config.frequency = 4_000_000;

    //let mut config1 = spi::Config::default();
    //config1.frequency = 4_000_000;

    //let spi = Spi::new_blocking(p.SPI0, sclk, mosi, miso, config);

    //let spibus: Mutex<CriticalSectionRawMutex, RefCell<_>> = Mutex::new(RefCell::new(spi));

    //let vfd_spi = SpiDeviceWithConfig::new(&spibus, cs, config1);

    //let mut vfd = Display::new(vfd_spi, rst);

    //vfd.set_brightness(128).unwrap();
    // let mut ticker = Ticker::every(Duration::from_secs(1));

    const NUM_LEDS: usize = 1;
    let mut data = [RGB8::default(); NUM_LEDS];
    let program = PioWs2812Program::new(&mut common);
    let mut ws2812 = PioWs2812::new(&mut common, sm0, p.DMA_CH1, p.PIN_21, &program);

    // Wrap flash as block device
    let mut ticker = Ticker::every(Duration::from_millis(100));

    //vfd.draw_boot_image().await;
    let mut clock = Output::new(p.PIN_6, Level::Low);
    let mut dataserial = Output::new(p.PIN_5, Level::Low);

    //let vals = therm.measure_temp();
    //vfd.ambient_temp = vals[0] as i8;
    //vfd.internal_temp = vals[1] as i8;

    //vfd.update_display();
    spawner.spawn(serialsyncer()).unwrap();

    loop {
        for j in 0..(256 * 5) {
            let bitfield: u128 = 0x1FFFFFFFFFF;
            //    for i in 0..128 {
            //        if (bitfield >> i) & 1 == 1 {
            //            let single_bit: u128 = 1u128 << i;

            for i in (0..128).rev() {
                clock.set_low();
                block_for(Duration::from_micros(8));
                clock.set_high();
                block_for(Duration::from_micros(2));
                let gpio_level = (bitfield >> i) & 1 != 0;
                dataserial.set_level(gpio_level.into());
            }

            //for bit in 0..32{
            let value: u32 = 0x0080_0000; //1 << bit;
            let dispvalue = value.to_le_bytes();
            i2c.blocking_write(chipaddr, &[0x00, dispvalue[0], dispvalue[1], dispvalue[2]])
                .unwrap();
            Timer::after(Duration::from_millis(1000)).await;
            //}

            for i in 0..NUM_LEDS {
                data[i] = wheel((((i * 256) as u16 / NUM_LEDS as u16 + j as u16) & 255) as u8);
            }
            ws2812.write(&data).await;
        }
    }
}

#[embassy_executor::task]
async fn serialsyncer() -> ! {
    let inputpin = unsafe { PIN_9::steal() };
    let outputpin = unsafe { PIN_10::steal() };
    let mut input = Input::new(inputpin, Pull::None);
    let mut output = Output::new(outputpin, Level::Low);
    loop {
        input.wait_for_rising_edge().await;
        output.set_high();
        Timer::after(Duration::from_micros(3)).await;
        output.set_low();
        Timer::after(Duration::from_millis(12)).await;
    }
}
