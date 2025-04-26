//! SPI Commands for the EEI GB1287BI VFD
use crate::traits;

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub(crate) enum Command {
    // software reset
    Reset = 0b1010_1010,
    // clear screen
    ClearGRAM = 0b0101_0101,

    VFDModeSetting = 0b1100_1100,
    DisplayAreaSetting = 0b1110_0000,
    InternalSpeedSetting = 0b1011_0001,

    BrightnessSetting = 0b1010_0000,
    WriteGRAM = 0b1111_0000,
    DisplayPosition1Offset = 0b1100_0000,
    DisplayPosition2Offset = 0b1101_0000,
    DisplayModeSetting = 0b1000_0000,
    FrameSyncSetting = 0b0000_1000,
    OscillationSetting = 0b0111_1000,
    UnknownInit = 0x90,
    WakeUp = 0b0110_1101,
    Sleep = 0b0110_0001,
}

impl traits::Command for Command {
    /// Returns the address of the command
    fn address(self) -> u8 {
        self as u8
    }
}
