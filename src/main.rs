#![no_std]
#![no_main]

extern crate alloc;

use z31_hvac::temp::Thermistor;
use z31_hvac::*;

use core::cell::RefCell;
use core::u128;
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
use embassy_time::{block_for, Duration, Ticker};
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
    let onboardts = Channel::new_temp_sensor(p.ADC_TEMP_SENSOR);

    let mut therm = Thermistor::new(ambtemp, onboardts, adc);

    let sclk = p.PIN_22;
    let mosi = p.PIN_23;
    let miso = p.PIN_20;
    
    let cs = Output::new(p.PIN_25, Level::High);
    let rst = Output::new(p.PIN_24, Level::High);

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

    /*const NUM_LEDS: usize = 1;
    let mut data = [RGB8::default(); NUM_LEDS];
    let program = PioWs2812Program::new(&mut common);
    let mut ws2812 = PioWs2812::new(&mut common, sm0, p.DMA_CH1, p.PIN_21, &program);*/

    // Wrap flash as block device
    let mut ticker = Ticker::every(Duration::from_millis(1000));
    let mut offdelay = Ticker::every(Duration::from_micros(8));
    let mut ontime = Ticker::every(Duration::from_micros(2));

    //vfd.draw_boot_image().await;
    let mut clock = Output::new(p.PIN_6, Level::Low);
    let mut data = Output::new(p.PIN_5, Level::Low);

    loop {
        let bitfield: u128 = 0x1FFFFFFFFFF;

        for i in 0..128 {
        if (bitfield >> i) & 1 == 1 {
            let single_bit: u128 = 1u128 << i;

            for i in (0..128).rev(){
            clock.set_low();
            block_for(Duration::from_micros(8));
            clock.set_high();
            block_for(Duration::from_micros(2));
            let gpio_level = (single_bit >> i) & 1 !=0;
            data.set_level(gpio_level.into());
        }
        block_for(Duration::from_millis(1000));
    }
    }

        //let vals = therm.measure_temp();
        //vfd.ambient_temp = vals[0] as i8;
        //vfd.internal_temp = vals[1] as i8;

        //vfd.update_display();

        //ticker.next().await;
        /*for j in 0..(256 * 5) {
            for i in 0..NUM_LEDS {
                data[i] = wheel((((i * 256) as u16 / NUM_LEDS as u16 + j as u16) & 255) as u8);
            }
            ws2812.write(&data).await;

            ticker.next().await;
        }*/
    }
}
