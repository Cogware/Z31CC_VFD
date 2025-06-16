use eei_vfd::{gp1287bi::VFD256x50, prelude::EEIDisplay};
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_rp::{
    gpio::Output,
    peripherals::SPI0,
    spi::{Blocking, Spi},
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Delay, Duration, Ticker};
use embedded_graphics::{
    framebuffer::{Framebuffer, buffer_size},
    pixelcolor::{
        BinaryColor,
        raw::{LittleEndian, RawU1},
    },
};
use embedded_graphics_transform::Transpose;

use crate::climatecontrol::ClimateControlMode;
use crate::{map_i32, vfdgraphics::Graphics};

pub type InternalFrameBuffer = Framebuffer<
    BinaryColor,
    RawU1,
    LittleEndian,
    128,
    256,
    { buffer_size::<BinaryColor>(128, 256) },
>;

pub type VFD<'a> = VFD256x50<
    SpiDeviceWithConfig<'a, CriticalSectionRawMutex, Spi<'a, SPI0, Blocking>, Output<'a>>,
    Output<'a>,
    Delay,
>;

pub struct Display<'a> {
    vfd: VFD<'a>,
    framebuffer: Transpose<InternalFrameBuffer>,
    graphics: Graphics,
    temp_gauge: u8,
    fan_gauge: u8,
}

impl<'a> Display<'a> {
    pub fn new(
        spi_bus: SpiDeviceWithConfig<
            'a,
            CriticalSectionRawMutex,
            Spi<'a, SPI0, Blocking>,
            Output<'a>,
        >,
        reset: Output<'a>,
    ) -> Self {
        let mut vfd: VFD = EEIDisplay::new(spi_bus, reset, Delay).unwrap();
        vfd.clear_frame().unwrap();

        let fb = InternalFrameBuffer::new();
        let framebuffer = Transpose::new(fb);
        let graphics = Graphics::load();

        let temp_gauge: u8 = 0;
        let fan_gauge: u8 = 0;

        let d = Display {
            vfd,
            framebuffer,
            graphics,
            temp_gauge,
            fan_gauge,
        };
        d
    }

    pub async fn draw_boot_image(&mut self) {
        let mut ticker = Ticker::every(Duration::from_secs(1));

        self.vfd.clear_frame().unwrap();
        self.graphics.draw_boot_image(&mut self.framebuffer);

        self.vfd.update_frame(self.framebuffer.data()).unwrap();
        self.vfd.set_brightness(128).unwrap();
        ticker.next().await;
        self.vfd.set_brightness(255).unwrap();
        ticker.next().await;
    }

    // return mutable refrence to framebuffer to use outside this struct
    pub fn use_frame_buffer(&mut self) -> &mut InternalFrameBuffer {
        &mut self.framebuffer
    }

    fn draw_background(&mut self) {
        self.graphics.draw_background(&mut self.framebuffer);
    }

    fn draw_mode(&mut self) {
        self.graphics
            .draw_climate_control_mode(&self.mode, &mut self.framebuffer);
        self.graphics
            .draw_ac_toggle(self.ac_toggle, &mut self.framebuffer);
        self.graphics
            .draw_recirc_toggle(self.recirc_toggle, &mut self.framebuffer);
    }

    fn draw_temps(&mut self) {
        self.graphics
            .draw_internal_temp(self.internal_temp, &mut self.framebuffer);
        self.graphics
            .draw_ambient_temp(self.ambient_temp, &mut self.framebuffer);
    }

    fn draw_fan_gauge(&mut self) {
        //5 HI 37 LO
        if self.fan_gauge > 32 {
            return;
        }
        let pos = map_i32(self.fan_gauge.into(), 0, 32, 37, 5) as i32;
        self.graphics.draw_fan_gauge(pos, &mut self.framebuffer);
    }

    fn draw_temp_gauge(&mut self) {
        // 42 HOT 5 COLD
        if self.temp_gauge > 36 {
            return ();
        }
        let pos = map_i32(self.temp_gauge.into(), 0, 36, 42, 5) as i32;
        self.graphics.draw_temp_guage(pos, &mut self.framebuffer);
    }

    pub fn update_display(&mut self) {
        self.vfd.clear_frame().unwrap();
        self.draw_background();
        self.draw_mode();
        self.draw_fan_gauge();
        self.draw_temp_gauge();
        self.draw_temps();
        self.vfd.update_frame(self.framebuffer.data()).unwrap();
    }
}
