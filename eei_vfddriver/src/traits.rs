use core::marker::Sized;
use embedded_hal::{delay::DelayNs, digital::*, spi::SpiDevice};

/// All commands need to have this trait which gives the address of the command
/// which needs to be send via SPI with activated CommandsPin (Data/Command Pin in CommandMode)
pub(crate) trait Command {
    fn address(self) -> u8;
}

pub(crate) trait EEIInit<SPI, RST, DELAY>
where
    SPI: SpiDevice,
    RST: OutputPin,
    DELAY: DelayNs,
{
    /// This initialises the display and powers it up
    ///
    /// This function is already called from
    ///  - [new()](EEIDisplay::new())
    ///  - [`wake_up`]
    ///
    ///
    /// This function calls [reset](EEIDisplay::reset),
    /// so you don't need to call reset your self when trying to wake your device up
    /// after setting it to sleep.
    fn init(&mut self) -> Result<(), SPI::Error>;
}

/// All the functions to interact with the EEI VFDs
pub trait EEIDisplay<SPI, RST, DELAY>
where
    SPI: SpiDevice,
    RST: OutputPin,
    DELAY: DelayNs,
{
    /// The Color Type used by the Display
    type DisplayColor;
    /// Creates a new driver from a SPI peripheral, CS Pin, Busy InputPin, DC
    ///
    /// This already initialises the device.
    fn new(spi: SPI, rst: RST, delay: DELAY) -> Result<Self, SPI::Error>
    where
        Self: Sized;

    /// Let the device enter deep-sleep mode to save power.
    ///
    /// The deep sleep mode returns to standby with a hardware reset.
    fn sleep(&mut self) -> Result<(), SPI::Error>;

    /// Wakes the device up from sleep
    ///
    /// Also reintialises the device if necessary.
    fn wake_up(&mut self) -> Result<(), SPI::Error>;

    /// Get the width of the display
    fn width(&self) -> u32;

    /// Get the height of the display
    fn height(&self) -> u32;

    /// set brightness of screen
    fn set_brightness(&mut self, val: u32) -> Result<(), SPI::Error>;

    /// Transmit a full frame to the SRAM of the EPD
    fn update_frame(&mut self, buffer: &[u8]) -> Result<(), SPI::Error>;

    /// Transmits partial data to the SRAM of the EPD
    ///
    /// (x,y) is the top left corner
    ///
    /// BUFFER needs to be of size: width / 8 * height !
    fn update_partial_frame(
        &mut self,
        buffer: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), SPI::Error>;

    /// Clears the frame buffer on the VFD with the declared background color
    ///
    fn clear_frame(&mut self) -> Result<(), SPI::Error>;
}
