//! A simple Driver for the gp1287bi display for SPI

use embedded_hal::{delay::DelayNs, digital::*, spi::SpiDevice};

use crate::interface::DisplayInterface;
use crate::traits::{EEIDisplay, EEIInit};

/// Width of gp1287bi in pixels
pub const WIDTH: u32 = 56;
/// Height of gp1287bi in pixels
pub const HEIGHT: u32 = 256;
/// Default Background Color (white)
pub const DEFAULT_BACKGROUND_COLOR: Color = Color::Dark;
const NUM_DISPLAY_BITS: u32 = WIDTH * HEIGHT / 8;

use crate::color::Color;

pub(crate) mod command;

#[cfg(feature = "graphics")]
mod graphics;

use self::command::Command;
#[cfg(feature = "graphics")]
pub use self::graphics::Display256x50;

/// vfd1in02 driver
pub struct VFD256x50<SPI, RST, DELAY> {
    interface: DisplayInterface<SPI, RST, DELAY>,
}

impl<SPI, RST, DELAY> EEIInit<SPI, RST, DELAY> for VFD256x50<SPI, RST, DELAY>
where
    SPI: SpiDevice,
    RST: OutputPin,
    DELAY: DelayNs,
{
    fn init(&mut self) -> Result<(), SPI::Error> {
        // Based on the spec (not public accessible)
        self.interface.reset(1);

        // software reset
        self.command(Command::Reset)?;
        self.interface.delay.delay_ms(1);

        self.cmd_with_args(Command::OscillationSetting, &[0x08])?;

        // set VFD mode
        self.cmd_with_args(Command::VFDModeSetting, &[0x02, 0x00])?;

        // set display area
        self.cmd_with_args(
            Command::DisplayAreaSetting,
            &[0xFF, 0x31, 0x00, 0x20, 0x00, 0x00, 0x80],
        )?;

        // set internal speed
        self.cmd_with_args(Command::InternalSpeedSetting, &[0x20, 0x3F, 0x00, 0x01])?;

        // set brightness
        self.set_brightness(0x30)?;

        // clear gram
        self.command(Command::ClearGRAM)?;
        self.interface.delay.delay_ms(15);

        // offset: no offset
        self.cmd_with_args(Command::DisplayPosition1Offset, &[0x00, 0x00])?;
        self.cmd_with_args(Command::DisplayPosition2Offset, &[0x00, 0x00])?;

        // unknown
        self.cmd_with_args(Command::UnknownInit, &[0x00])?;

        // set frame sync
        self.cmd_with_args(Command::FrameSyncSetting, &[0x00])?;

        // set display mode
        self.cmd_with_args(Command::DisplayModeSetting, &[0x00])

    }
}

impl<SPI, RST, DELAY> EEIDisplay<SPI, RST, DELAY> for VFD256x50<SPI, RST, DELAY>
where
    SPI: SpiDevice,
    RST: OutputPin,
    DELAY: DelayNs,
{
    type DisplayColor = Color;
    fn new(spi: SPI, rst: RST, delay: DELAY) -> Result<Self, SPI::Error> {
        let interface = DisplayInterface::new(spi, rst, delay);

        let mut vfd = VFD256x50 { interface };

        vfd.init()?;

        Ok(vfd)
    }

    fn set_brightness(&mut self, val: u32) -> Result<(), SPI::Error> {
        self.cmd_with_args(
            Command::BrightnessSetting,
            &[((val >> 8) as u8) & 0b11, val as u8],
        )
    }

    fn sleep(&mut self) -> Result<(), SPI::Error> {
        self.command(Command::Sleep)
    }

    fn wake_up(&mut self) -> Result<(), SPI::Error> {
        self.command(Command::WakeUp)
    }

    fn width(&self) -> u32 {
        WIDTH
    }

    fn height(&self) -> u32 {
        HEIGHT
    }

    fn update_frame(&mut self, buffer: &[u8]) -> Result<(), SPI::Error> {
        self.cmd_with_data(Command::WriteGRAM, &[0x00, 0x00, 0x7F], buffer)
    }

    #[allow(unused)]
    fn update_partial_frame(
        &mut self,
        buffer: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), SPI::Error> {
        unimplemented!()
    }

    fn clear_frame(&mut self) -> Result<(), SPI::Error> {
        // Clear the black
        self.command(Command::ClearGRAM)?;
        self.interface.delay.delay_ms(10);
        Ok(())
    }
}

impl<SPI, RST, DELAY> VFD256x50<SPI, RST, DELAY>
where
    SPI: SpiDevice,
    RST: OutputPin,
    DELAY: DelayNs,
{
    fn command(&mut self, command: Command) -> Result<(), SPI::Error> {
        self.cmd_with_args(command, &[])
    }

    fn cmd_with_args(&mut self, command: Command, args: &[u8]) -> Result<(), SPI::Error> {
        self.interface.cmd_with_arg(command, args)
    }

    fn cmd_with_data(
        &mut self,
        command: Command,
        args: &[u8],
        data: &[u8],
    ) -> Result<(), SPI::Error> {
        self.interface.cmd_with_data(command, args, data)
    }
}
