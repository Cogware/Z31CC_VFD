use crate::traits::Command;
use embedded_hal::{delay::DelayNs, digital::*, spi::Operation, spi::SpiDevice};

/// The Connection Interface of all (?) EEI VFD
///
pub(crate) struct DisplayInterface<SPI, RST, DELAY> {
    /// SPI
    spi: SPI,
    /// DELAY
    pub(crate) delay: DELAY,
    /// Pin for Resetting
    rst: RST,
}

impl<SPI, RST, DELAY> DisplayInterface<SPI, RST, DELAY>
where
    SPI: SpiDevice,
    RST: OutputPin,
    DELAY: DelayNs,
{
    pub fn new(spi: SPI, rst: RST, delay: DELAY) -> Self {
        DisplayInterface {
            spi,
            delay,
            rst,
        }
    }

    /// Basic function for sending [Commands](Command) and the data belonging to it.
    ///
    /// TODO: directly use ::write? cs wouldn't needed to be changed twice than
    pub(crate) fn cmd_with_data<T: Command>(
        &mut self,
        command: T,
        args: &[u8],
        data: &[u8],
    ) -> Result<(), SPI::Error> {
        self.spi.write(&[])?;
        self.spi.transaction(&mut [
            Operation::Write(&[command.address().reverse_bits()]),
            Operation::Write(args),
            Operation::Write(data),
        ])
    }

    /// Basic function for sending [Commands](Command) and the data belonging to it.
    ///
    /// TODO: directly use ::write? cs wouldn't needed to be changed twice than
    pub(crate) fn cmd_with_arg<T: Command>(
        &mut self,
        command: T,
        args: &[u8],
    ) -> Result<(), SPI::Error> {
        // this is nessessary for shifting out the previous frame when communicating
        // with high frequency
        self.spi.write(&[])?;
        self.spi.transaction(&mut [
            Operation::Write(&[command.address().reverse_bits()]),
            Operation::Write(args),
        ])
    }

    /// Basic function for sending the same byte of data (one u8) multiple times over spi
    ///
    /// Enables direct interaction with the device with the help of [command()](ConnectionInterface::command())
    #[allow(unused)]
    pub(crate) fn data_x_times<const R: usize>(&mut self, val: u8) -> Result<(), SPI::Error> {
        // Transfer data (u8) over spi
        self.spi.write(&[val; R])
    }

    /// Resets the device.
    ///
    /// Often used to awake the module from deep sleep. See [VFD256x50::sleep()](Epd4in2::sleep())
    ///
    /// The timing of keeping the reset pin low seems to be important and different per device.
    /// Most displays seem to require keeping it low for 10ms, but the 7in5_v2 only seems to reset
    /// properly with 2ms
    pub(crate) fn reset(&mut self, duration: u32) {
        let _ = self.rst.set_low();
        self.delay.delay_ms(duration);
        let _ = self.rst.set_high();
        self.delay.delay_ms(1)
    }
}
