use bitflags::bitflags;
use embassy_rp::{
    gpio::{Flex, Input, Level, Output, Pull},
    i2c::{Blocking, I2c},
    peripherals::{I2C1, PIN_9, PIN_10, PIN_11, PIN_27, PIN_28},
};
use embassy_time::{Duration, Timer, block_for};

use crate::{
    climatecontrol::{ClimateControlBacker, ClimateControlMode},
    map_u8,
};

bitflags! {
    //      Statically driven side of display(through driver IC over I2C)
    pub struct SegDisplayBits: u32{
        // ──     General indicators     ───────────────────────────────────────
        const FRESH_AIR = 0x0000_0001;
        const FACE = 0x0000_0002;
        const FAN = 0x0000_0004;
        const DEFROST = 0x0000_0008;
        const FEET = 0x0000_0010;
        const ACGAS = 0x0000_0020;
        const CELCIUS = 0x0000_0080; // Celcius temp indicator
        const BACKGROUND = 0x0000_0800; // filler backer
        const FARENHEIT = 0x0000_1000; // farenheit temp indicator
        const AC = 0x0000_2000;
        const HEAT = 0x0000_4000; // heat icon when watercock is active
        const RECIRC = 0x0000_8000;
        // ──   Set “second” digit (ones)  ──────────────────────────────────────
        const SET2_TL = 0x0010_0000;
        const SET2_T = 0x0020_0000;
        const SET2_TR = 0x0040_0000;
        const SET2_M = 0x0080_0000;
        const SET2_BR = 0x0000_0100;
        const SET2_B = 0x0000_0200;
        const SET2_BL = 0x0000_0400;

        const EMPTY = 0x0000_0000;
    }

    //      Serially controlled side of display
    pub struct SerialDisplayBits: u64 {
        // ── Ambient “second” digit (ones) ──────────────────────────────────────
        const AMB2_TL = 0x0000_0000_0000_0001; // top-left
        const AMB2_T  = 0x0000_0000_0000_0002; // top
        const AMB2_TR = 0x0000_0000_0000_0004; // top-right
        const AMB2_M  = 0x0000_0000_0000_0008; // middle
        const AMB2_BR = 0x0000_0000_0000_0010; // bottom-right
        const AMB2_B  = 0x0000_0000_0000_0020; // bottom
        const AMB2_BL = 0x0000_0000_0000_0040; // bottom-left

        // ── Ambient “first” digit (tens) ───────────────────────────────────────
        const AMB1_BR = 0x0000_0000_0000_0100;
        const AMB1_B  = 0x0000_0000_0000_0200;
        const AMB1_BL = 0x0000_0000_0000_0400;
        const AMB1_TL = 0x0000_0000_0000_1000;
        const AMB1_T  = 0x0000_0000_0000_2000;
        const AMB1_TR = 0x0000_0000_0000_4000;
        const AMB1_M  = 0x0000_0000_0000_8000;

        // ── Ambient “1” & “–” indicators ───────────────────────────────────────
        const AMB_ONE = 0x0000_0000_0001_0000;
        const AMB_NEG = 0x0000_0000_0002_0000;

        // ──       Temp gauge bars       ────────────────────────────────────────
        const TG_NEG1 = 0x0000_0000_0004_0000;
        const TG_NEG2 = 0x0000_0000_0008_0000;
        const TG_NEG3 = 0x0000_0000_0010_0000;
        const TG_NEG4 = 0x0000_0000_0020_0000;
        const TG_NEG5 = 0x0000_0000_0040_0000;
        const TG_PLUS5 = 0x0000_0000_0100_0000;
        const TG_PLUS4 = 0x0000_0000_0200_0000;
        const TG_PLUS3 = 0x0000_0000_0400_0000;
        const TG_PLUS2 = 0x0000_0000_0800_0000;
        const TG_ZERO  = 0x0000_0000_4000_0000;
        const TG_PLUS1 = 0x0000_0000_8000_0000;

        // ──  “Set” “1” & “–” indicators  ───────────────────────────────────────
        const SET_ONE = 0x0000_0000_1000_0000;
        const SET_NEG = 0x0000_0000_2000_0000;

        // ── “Set” digit (first/tens) segments ──────────────────────────────────
        const SET1_BR = 0x0000_0001_0000_0000;
        const SET1_B  = 0x0000_0002_0000_0000;
        const SET1_BL = 0x0000_0004_0000_0000;
        const SET1_TL = 0x0000_0010_0000_0000;
        const SET1_T  = 0x0000_0020_0000_0000;
        const SET1_TR = 0x0000_0040_0000_0000;
        const SET1_M  = 0x0000_0080_0000_0000;

        const EMPTY = 0x0000_0000_0000_0000;
    }
}
#[allow(unused)]
impl SerialDisplayBits {
    pub fn get_serialout(input: SerialDisplayBits) -> u128 {
        input.bits().into()
    }
    /// Pattern for ambient “second” (ones) digit 0x0–0xF
    fn amb_second(n: u8) -> SerialDisplayBits {
        match n {
            0 => {
                SerialDisplayBits::AMB2_T
                    | SerialDisplayBits::AMB2_TR
                    | SerialDisplayBits::AMB2_BR
                    | SerialDisplayBits::AMB2_B
                    | SerialDisplayBits::AMB2_BL
                    | SerialDisplayBits::AMB2_TL
            }
            1 => SerialDisplayBits::AMB2_TR | SerialDisplayBits::AMB2_BR,
            2 => {
                SerialDisplayBits::AMB2_T
                    | SerialDisplayBits::AMB2_TR
                    | SerialDisplayBits::AMB2_M
                    | SerialDisplayBits::AMB2_BL
                    | SerialDisplayBits::AMB2_B
            }
            3 => {
                SerialDisplayBits::AMB2_T
                    | SerialDisplayBits::AMB2_TR
                    | SerialDisplayBits::AMB2_M
                    | SerialDisplayBits::AMB2_BR
                    | SerialDisplayBits::AMB2_B
            }
            4 => {
                SerialDisplayBits::AMB2_TL
                    | SerialDisplayBits::AMB2_M
                    | SerialDisplayBits::AMB2_TR
                    | SerialDisplayBits::AMB2_BR
            }
            5 => {
                SerialDisplayBits::AMB2_T
                    | SerialDisplayBits::AMB2_TL
                    | SerialDisplayBits::AMB2_M
                    | SerialDisplayBits::AMB2_BR
                    | SerialDisplayBits::AMB2_B
            }
            6 => {
                SerialDisplayBits::AMB2_T
                    | SerialDisplayBits::AMB2_TL
                    | SerialDisplayBits::AMB2_M
                    | SerialDisplayBits::AMB2_BR
                    | SerialDisplayBits::AMB2_B
                    | SerialDisplayBits::AMB2_BL
            }
            7 => {
                SerialDisplayBits::AMB2_T | SerialDisplayBits::AMB2_TR | SerialDisplayBits::AMB2_BR
            }
            8 => {
                SerialDisplayBits::AMB2_T
                    | SerialDisplayBits::AMB2_TR
                    | SerialDisplayBits::AMB2_BR
                    | SerialDisplayBits::AMB2_B
                    | SerialDisplayBits::AMB2_BL
                    | SerialDisplayBits::AMB2_TL
                    | SerialDisplayBits::AMB2_M
            }
            9 => {
                SerialDisplayBits::AMB2_T
                    | SerialDisplayBits::AMB2_TR
                    | SerialDisplayBits::AMB2_TL
                    | SerialDisplayBits::AMB2_M
                    | SerialDisplayBits::AMB2_BR
                    | SerialDisplayBits::AMB2_B
            }
            0xA => {
                SerialDisplayBits::AMB2_T
                    | SerialDisplayBits::AMB2_TR
                    | SerialDisplayBits::AMB2_TL
                    | SerialDisplayBits::AMB2_M
                    | SerialDisplayBits::AMB2_BL
                    | SerialDisplayBits::AMB2_BR
            }
            0xB => {
                SerialDisplayBits::AMB2_M
                    | SerialDisplayBits::AMB2_BL
                    | SerialDisplayBits::AMB2_B
                    | SerialDisplayBits::AMB2_BR
                    | SerialDisplayBits::AMB2_TL
            }
            0xC => {
                SerialDisplayBits::AMB2_T
                    | SerialDisplayBits::AMB2_TL
                    | SerialDisplayBits::AMB2_BL
                    | SerialDisplayBits::AMB2_B
            }
            0xD => {
                SerialDisplayBits::AMB2_M
                    | SerialDisplayBits::AMB2_TR
                    | SerialDisplayBits::AMB2_BR
                    | SerialDisplayBits::AMB2_BL
                    | SerialDisplayBits::AMB2_B
            }
            0xE => {
                SerialDisplayBits::AMB2_T
                    | SerialDisplayBits::AMB2_TL
                    | SerialDisplayBits::AMB2_M
                    | SerialDisplayBits::AMB2_BL
                    | SerialDisplayBits::AMB2_B
            }
            0xF => {
                SerialDisplayBits::AMB2_T
                    | SerialDisplayBits::AMB2_TL
                    | SerialDisplayBits::AMB2_M
                    | SerialDisplayBits::AMB2_BL
            }
            _ => SerialDisplayBits::empty(),
        }
    }

    /// Pattern for ambient “first” (tens) digit 0x0–0xF
    fn amb_first(n: u8) -> SerialDisplayBits {
        match n {
            0 => {
                SerialDisplayBits::AMB1_T
                    | SerialDisplayBits::AMB1_TR
                    | SerialDisplayBits::AMB1_BR
                    | SerialDisplayBits::AMB1_B
                    | SerialDisplayBits::AMB1_BL
                    | SerialDisplayBits::AMB1_TL
            }
            1 => SerialDisplayBits::AMB1_TR | SerialDisplayBits::AMB1_BR,
            2 => {
                SerialDisplayBits::AMB1_T
                    | SerialDisplayBits::AMB1_TR
                    | SerialDisplayBits::AMB1_M
                    | SerialDisplayBits::AMB1_BL
                    | SerialDisplayBits::AMB1_B
            }
            3 => {
                SerialDisplayBits::AMB1_T
                    | SerialDisplayBits::AMB1_TR
                    | SerialDisplayBits::AMB1_M
                    | SerialDisplayBits::AMB1_BR
                    | SerialDisplayBits::AMB1_B
            }
            4 => {
                SerialDisplayBits::AMB1_TL
                    | SerialDisplayBits::AMB1_M
                    | SerialDisplayBits::AMB1_TR
                    | SerialDisplayBits::AMB1_BR
            }
            5 => {
                SerialDisplayBits::AMB1_T
                    | SerialDisplayBits::AMB1_TL
                    | SerialDisplayBits::AMB1_M
                    | SerialDisplayBits::AMB1_BR
                    | SerialDisplayBits::AMB1_B
            }
            6 => {
                SerialDisplayBits::AMB1_T
                    | SerialDisplayBits::AMB1_TL
                    | SerialDisplayBits::AMB1_M
                    | SerialDisplayBits::AMB1_BR
                    | SerialDisplayBits::AMB1_B
                    | SerialDisplayBits::AMB1_BL
            }
            7 => {
                SerialDisplayBits::AMB1_T | SerialDisplayBits::AMB1_TR | SerialDisplayBits::AMB1_BR
            }
            8 => {
                SerialDisplayBits::AMB1_T
                    | SerialDisplayBits::AMB1_TR
                    | SerialDisplayBits::AMB1_BR
                    | SerialDisplayBits::AMB1_B
                    | SerialDisplayBits::AMB1_BL
                    | SerialDisplayBits::AMB1_TL
                    | SerialDisplayBits::AMB1_M
            }
            9 => {
                SerialDisplayBits::AMB1_T
                    | SerialDisplayBits::AMB1_TR
                    | SerialDisplayBits::AMB1_TL
                    | SerialDisplayBits::AMB1_M
                    | SerialDisplayBits::AMB1_BR
                    | SerialDisplayBits::AMB1_B
            }
            0xA => {
                SerialDisplayBits::AMB1_T
                    | SerialDisplayBits::AMB1_TR
                    | SerialDisplayBits::AMB1_TL
                    | SerialDisplayBits::AMB1_M
                    | SerialDisplayBits::AMB1_BL
                    | SerialDisplayBits::AMB1_BR
            }
            0xB => {
                SerialDisplayBits::AMB1_M
                    | SerialDisplayBits::AMB1_BL
                    | SerialDisplayBits::AMB1_B
                    | SerialDisplayBits::AMB1_BR
                    | SerialDisplayBits::AMB1_TL
            }
            0xC => {
                SerialDisplayBits::AMB1_T
                    | SerialDisplayBits::AMB1_TL
                    | SerialDisplayBits::AMB1_BL
                    | SerialDisplayBits::AMB1_B
            }
            0xD => {
                SerialDisplayBits::AMB1_M
                    | SerialDisplayBits::AMB1_TR
                    | SerialDisplayBits::AMB1_BR
                    | SerialDisplayBits::AMB1_BL
                    | SerialDisplayBits::AMB1_B
            }
            0xE => {
                SerialDisplayBits::AMB1_T
                    | SerialDisplayBits::AMB1_TL
                    | SerialDisplayBits::AMB1_M
                    | SerialDisplayBits::AMB1_BL
                    | SerialDisplayBits::AMB1_B
            }
            0xF => {
                SerialDisplayBits::AMB1_T
                    | SerialDisplayBits::AMB1_TL
                    | SerialDisplayBits::AMB1_M
                    | SerialDisplayBits::AMB1_BL
            }
            _ => SerialDisplayBits::empty(),
        }
    }

    fn amb_neg(b: bool) -> SerialDisplayBits {
        if b == true {
            return SerialDisplayBits::AMB_NEG;
        }
        SerialDisplayBits::EMPTY
    }

    fn amb_hund(b: bool) -> SerialDisplayBits {
        if b == true {
            return SerialDisplayBits::AMB_ONE;
        }
        SerialDisplayBits::EMPTY
    }

    /// Pattern for the “set” digit (tens) 0x0–0xF
    pub fn set_first(n: u8) -> SerialDisplayBits {
        match n {
            0 => {
                SerialDisplayBits::SET1_T
                    | SerialDisplayBits::SET1_TR
                    | SerialDisplayBits::SET1_BR
                    | SerialDisplayBits::SET1_B
                    | SerialDisplayBits::SET1_BL
                    | SerialDisplayBits::SET1_TL
            }
            1 => SerialDisplayBits::SET1_TR | SerialDisplayBits::SET1_BR,
            2 => {
                SerialDisplayBits::SET1_T
                    | SerialDisplayBits::SET1_TR
                    | SerialDisplayBits::SET1_M
                    | SerialDisplayBits::SET1_BL
                    | SerialDisplayBits::SET1_B
            }
            3 => {
                SerialDisplayBits::SET1_T
                    | SerialDisplayBits::SET1_TR
                    | SerialDisplayBits::SET1_M
                    | SerialDisplayBits::SET1_BR
                    | SerialDisplayBits::SET1_B
            }
            4 => {
                SerialDisplayBits::SET1_TL
                    | SerialDisplayBits::SET1_M
                    | SerialDisplayBits::SET1_TR
                    | SerialDisplayBits::SET1_BR
            }
            5 => {
                SerialDisplayBits::SET1_T
                    | SerialDisplayBits::SET1_TL
                    | SerialDisplayBits::SET1_M
                    | SerialDisplayBits::SET1_BR
                    | SerialDisplayBits::SET1_B
            }
            6 => {
                SerialDisplayBits::SET1_T
                    | SerialDisplayBits::SET1_TL
                    | SerialDisplayBits::SET1_M
                    | SerialDisplayBits::SET1_BR
                    | SerialDisplayBits::SET1_B
                    | SerialDisplayBits::SET1_BL
            }
            7 => {
                SerialDisplayBits::SET1_T | SerialDisplayBits::SET1_TR | SerialDisplayBits::SET1_BR
            }
            8 => {
                SerialDisplayBits::SET1_T
                    | SerialDisplayBits::SET1_TR
                    | SerialDisplayBits::SET1_BR
                    | SerialDisplayBits::SET1_B
                    | SerialDisplayBits::SET1_BL
                    | SerialDisplayBits::SET1_TL
                    | SerialDisplayBits::SET1_M
            }
            9 => {
                SerialDisplayBits::SET1_T
                    | SerialDisplayBits::SET1_TR
                    | SerialDisplayBits::SET1_TL
                    | SerialDisplayBits::SET1_M
                    | SerialDisplayBits::SET1_BR
                    | SerialDisplayBits::SET1_B
            }
            0xA => {
                SerialDisplayBits::SET1_T
                    | SerialDisplayBits::SET1_TR
                    | SerialDisplayBits::SET1_TL
                    | SerialDisplayBits::SET1_M
                    | SerialDisplayBits::SET1_BL
                    | SerialDisplayBits::SET1_BR
            }
            0xB => {
                SerialDisplayBits::SET1_M
                    | SerialDisplayBits::SET1_BL
                    | SerialDisplayBits::SET1_B
                    | SerialDisplayBits::SET1_BR
                    | SerialDisplayBits::SET1_TL
            }
            0xC => {
                SerialDisplayBits::SET1_T
                    | SerialDisplayBits::SET1_TL
                    | SerialDisplayBits::SET1_BL
                    | SerialDisplayBits::SET1_B
            }
            0xD => {
                SerialDisplayBits::SET1_M
                    | SerialDisplayBits::SET1_TR
                    | SerialDisplayBits::SET1_BR
                    | SerialDisplayBits::SET1_BL
                    | SerialDisplayBits::SET1_B
            }
            0xE => {
                SerialDisplayBits::SET1_T
                    | SerialDisplayBits::SET1_TL
                    | SerialDisplayBits::SET1_M
                    | SerialDisplayBits::SET1_BL
                    | SerialDisplayBits::SET1_B
            }
            0xF => {
                SerialDisplayBits::SET1_T
                    | SerialDisplayBits::SET1_TL
                    | SerialDisplayBits::SET1_M
                    | SerialDisplayBits::SET1_BL
            }
            _ => SerialDisplayBits::empty(),
        }
    }

    pub fn set_neg(b: bool) -> SerialDisplayBits {
        if b == true {
            return SerialDisplayBits::SET_NEG;
        }
        SerialDisplayBits::EMPTY
    }

    pub fn set_hund(b: bool) -> SerialDisplayBits {
        if b == true {
            return SerialDisplayBits::SET_ONE;
        }
        SerialDisplayBits::EMPTY
    }

    /// Build the thermometer‐style gauge for levels –5…+5.
    fn gauge(level: u8) -> SerialDisplayBits {
        match level {
            0 => SerialDisplayBits::TG_NEG5,
            1 => SerialDisplayBits::TG_NEG4,
            2 => SerialDisplayBits::TG_NEG3,
            3 => SerialDisplayBits::TG_NEG2,
            4 => SerialDisplayBits::TG_NEG1,
            5 => SerialDisplayBits::TG_ZERO,
            6 => SerialDisplayBits::TG_PLUS1,
            7 => SerialDisplayBits::TG_PLUS2,
            8 => SerialDisplayBits::TG_PLUS3,
            9 => SerialDisplayBits::TG_PLUS4,
            10 => SerialDisplayBits::TG_PLUS5,
            _ => SerialDisplayBits::empty(),
        }
    }

    pub fn setup_amb(input: i8) -> SerialDisplayBits {
        let mut base = SerialDisplayBits::EMPTY;
        let mut n = input;
        if n < 0 {
            n = n * -1;
            base = base | SerialDisplayBits::amb_neg(true);
        }
        if n >= 100 {
            n = n - 100;
            base = base | SerialDisplayBits::amb_hund(true);
        }
        let tens = n / 10;
        let ones = n % 10;
        base = base
            | SerialDisplayBits::amb_first(tens.try_into().unwrap())
            | SerialDisplayBits::amb_second(ones.try_into().unwrap());
        return base;
    }

    pub fn setup_set(input: i8) -> (SerialDisplayBits, i8) {
        let mut base = SerialDisplayBits::EMPTY;
        let mut n = input;
        if n < 0 {
            n = n * -1;
            base = base | SerialDisplayBits::set_neg(true);
        }
        if n >= 100 {
            n = n - 100;
            base = base | SerialDisplayBits::set_hund(true);
        }
        let tens = n / 10;
        let ones = n % 10;
        base = base | SerialDisplayBits::amb_first(tens.try_into().unwrap());
        return (base, ones);
    }
}

#[allow(unused)]
impl SegDisplayBits {
    pub fn get_bitsout(input: SegDisplayBits) -> u32 {
        input.bits()
    }

    pub fn set_second(n: i8) -> SegDisplayBits {
        match n {
            0 => {
                SegDisplayBits::SET2_T
                    | SegDisplayBits::SET2_TR
                    | SegDisplayBits::SET2_BR
                    | SegDisplayBits::SET2_B
                    | SegDisplayBits::SET2_BL
                    | SegDisplayBits::SET2_TL
            }
            1 => SegDisplayBits::SET2_TR | SegDisplayBits::SET2_BR,
            2 => {
                SegDisplayBits::SET2_T
                    | SegDisplayBits::SET2_TR
                    | SegDisplayBits::SET2_M
                    | SegDisplayBits::SET2_BL
                    | SegDisplayBits::SET2_B
            }
            3 => {
                SegDisplayBits::SET2_T
                    | SegDisplayBits::SET2_TR
                    | SegDisplayBits::SET2_M
                    | SegDisplayBits::SET2_BR
                    | SegDisplayBits::SET2_B
            }
            4 => {
                SegDisplayBits::SET2_TL
                    | SegDisplayBits::SET2_M
                    | SegDisplayBits::SET2_TR
                    | SegDisplayBits::SET2_BR
            }
            5 => {
                SegDisplayBits::SET2_T
                    | SegDisplayBits::SET2_TL
                    | SegDisplayBits::SET2_M
                    | SegDisplayBits::SET2_BR
                    | SegDisplayBits::SET2_B
            }
            6 => {
                SegDisplayBits::SET2_T
                    | SegDisplayBits::SET2_TL
                    | SegDisplayBits::SET2_M
                    | SegDisplayBits::SET2_BR
                    | SegDisplayBits::SET2_B
                    | SegDisplayBits::SET2_BL
            }
            7 => SegDisplayBits::SET2_T | SegDisplayBits::SET2_TR | SegDisplayBits::SET2_BR,
            8 => {
                SegDisplayBits::SET2_T
                    | SegDisplayBits::SET2_TR
                    | SegDisplayBits::SET2_BR
                    | SegDisplayBits::SET2_B
                    | SegDisplayBits::SET2_BL
                    | SegDisplayBits::SET2_TL
                    | SegDisplayBits::SET2_M
            }
            9 => {
                SegDisplayBits::SET2_T
                    | SegDisplayBits::SET2_TR
                    | SegDisplayBits::SET2_TL
                    | SegDisplayBits::SET2_M
                    | SegDisplayBits::SET2_BR
                    | SegDisplayBits::SET2_B
            }
            0xA => {
                SegDisplayBits::SET2_T
                    | SegDisplayBits::SET2_TR
                    | SegDisplayBits::SET2_TL
                    | SegDisplayBits::SET2_M
                    | SegDisplayBits::SET2_BL
                    | SegDisplayBits::SET2_BR
            }
            0xB => {
                SegDisplayBits::SET2_M
                    | SegDisplayBits::SET2_BL
                    | SegDisplayBits::SET2_B
                    | SegDisplayBits::SET2_BR
                    | SegDisplayBits::SET2_TL
            }
            0xC => {
                SegDisplayBits::SET2_T
                    | SegDisplayBits::SET2_TL
                    | SegDisplayBits::SET2_BL
                    | SegDisplayBits::SET2_B
            }
            0xD => {
                SegDisplayBits::SET2_M
                    | SegDisplayBits::SET2_TR
                    | SegDisplayBits::SET2_BR
                    | SegDisplayBits::SET2_BL
                    | SegDisplayBits::SET2_B
            }
            0xE => {
                SegDisplayBits::SET2_T
                    | SegDisplayBits::SET2_TL
                    | SegDisplayBits::SET2_M
                    | SegDisplayBits::SET2_BL
                    | SegDisplayBits::SET2_B
            }
            0xF => {
                SegDisplayBits::SET2_T
                    | SegDisplayBits::SET2_TL
                    | SegDisplayBits::SET2_M
                    | SegDisplayBits::SET2_BL
            }
            _ => SegDisplayBits::empty(),
        }
    }

    pub fn recirc(b: bool) -> SegDisplayBits {
        if b == true {
            return SegDisplayBits::RECIRC;
        }
        SegDisplayBits::FRESH_AIR
    }

    pub fn mode(input: &ClimateControlMode) -> SegDisplayBits {
        match input {
            ClimateControlMode::Face => {
                SegDisplayBits::FACE | SegDisplayBits::BACKGROUND | SegDisplayBits::FAN
            }
            ClimateControlMode::Feet => {
                SegDisplayBits::FEET | SegDisplayBits::BACKGROUND | SegDisplayBits::FAN
            }
            ClimateControlMode::FaceFeet => {
                SegDisplayBits::FACE
                    | SegDisplayBits::FEET
                    | SegDisplayBits::BACKGROUND
                    | SegDisplayBits::FAN
            }
            ClimateControlMode::FeetDef => {
                SegDisplayBits::FEET
                    | SegDisplayBits::DEFROST
                    | SegDisplayBits::BACKGROUND
                    | SegDisplayBits::FAN
            }
            ClimateControlMode::Def => {
                SegDisplayBits::DEFROST | SegDisplayBits::BACKGROUND | SegDisplayBits::FAN
            }
        }
    }

    pub fn ac_toggle(b: bool) -> SegDisplayBits {
        if b == true {
            return SegDisplayBits::AC;
        }
        SegDisplayBits::EMPTY
    }

    pub fn c_or_f(b: bool) -> SegDisplayBits {
        if b == true {
            return SegDisplayBits::CELCIUS;
        }
        SegDisplayBits::FARENHEIT
    }

    pub fn heat_watercock(b: bool) -> SegDisplayBits {
        if b == true {
            return SegDisplayBits::HEAT;
        }
        SegDisplayBits::EMPTY
    }
}

#[derive(Copy, Clone)]
pub enum Button {
    Auto,
    Demist,
    TempUp,
    Off,
    FanLo,
    FanHigh,
    Recirc,
    TempDown,
}

pub struct Buttons<'a> {
    pin1: Flex<'a>,
    pin2: Flex<'a>,
    pin3: Flex<'a>,
    pin4: Flex<'a>,
    pin5: Flex<'a>,
    pin6: Flex<'a>,
}

impl<'a> Buttons<'a> {
    pub fn new(
        pin1: Flex<'a>,
        pin2: Flex<'a>,
        pin3: Flex<'a>,
        pin4: Flex<'a>,
        pin5: Flex<'a>,
        pin6: Flex<'a>,
    ) -> Self {
        Buttons {
            pin1,
            pin2,
            pin3,
            pin4,
            pin5,
            pin6,
        }
    }

    /// Only borrow &mut self for this call; returns references valid
    /// for the duration of the borrow, not `’a`.
    pub fn get(&mut self, button: Button) -> (&mut Flex<'a>, &Flex<'a>) {
        match button {
            Button::Auto => (&mut self.pin1, &self.pin4),
            Button::Demist => (&mut self.pin1, &self.pin2),
            Button::TempUp => (&mut self.pin4, &self.pin5),
            Button::Off => (&mut self.pin3, &self.pin4),
            Button::FanLo => (&mut self.pin6, &self.pin2),
            Button::FanHigh => (&mut self.pin6, &self.pin4),
            Button::Recirc => (&mut self.pin2, &self.pin3),
            Button::TempDown => (&mut self.pin2, &self.pin5),
        }
    }
}

pub struct ButtonIter {
    next: Option<Button>,
}

impl ButtonIter {
    pub fn new() -> Self {
        ButtonIter {
            next: Some(Button::Auto),
        }
    }
}

impl Iterator for ButtonIter {
    type Item = Button;

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.next.take()?;
        self.next = match curr {
            Button::Auto => Some(Button::Demist),
            Button::Demist => Some(Button::TempUp),
            Button::TempUp => Some(Button::Off),
            Button::Off => Some(Button::FanLo),
            Button::FanLo => Some(Button::FanHigh),
            Button::FanHigh => Some(Button::Recirc),
            Button::Recirc => Some(Button::TempDown),
            Button::TempDown => None,
        };
        Some(curr)
    }
}

pub struct DigiDisplay<'a> {
    //i2c: I2c<'a, I2C1, Blocking>,
    serialclock: Output<'a>,
    serialdata: Output<'a>,
    chipaddr: u8,
    demist_led: Output<'a>,
    ac_led: Output<'a>,
    econ_led: Output<'a>,
    defrost_led: Output<'a>,
    fanhigh_led: Output<'a>,
    fanlow_led: Output<'a>,
    recirc_led: Output<'a>,
    buttons: Buttons<'a>,
    backend: ClimateControlBacker,
}

impl<'a> DigiDisplay<'a> {
    pub fn new(
        //mut i2c: I2c<'a, I2C1, Blocking>,
        serialclock: Output<'a>,
        serialdata: Output<'a>,
        demist_led: Output<'a>,
        ac_led: Output<'a>,
        econ_led: Output<'a>,
        defrost_led: Output<'a>,
        fanhigh_led: Output<'a>,
        fanlow_led: Output<'a>,
        recirc_led: Output<'a>,
        pin1: Flex<'a>,
        pin2: Flex<'a>,
        pin3: Flex<'a>,
        pin4: Flex<'a>,
        pin5: Flex<'a>,
        pin6: Flex<'a>,
        backend: ClimateControlBacker,
    ) -> Self {
        let chipaddr = 0x38;
        embassy_time::block_for(Duration::from_millis(10));
        //i2c.blocking_write(chipaddr, &[0x49]).unwrap();

        let buttons = Buttons::new(pin1, pin2, pin3, pin4, pin5, pin6);

        DigiDisplay {
            //i2c,
            serialclock,
            serialdata,
            chipaddr,
            demist_led,
            ac_led,
            econ_led,
            defrost_led,
            fanhigh_led,
            fanlow_led,
            recirc_led,
            buttons,
            backend,
        }
    }

    pub async fn buttonreader(&mut self) {
        let mut iter = ButtonIter::new();

        while let Some(button) = iter.next() {
            {
                let (setpin, _) = self.buttons.get(button);
                setpin.set_as_output();
                setpin.set_low();
            }
            Timer::after(Duration::from_millis(1)).await;

            let pressed = {
                let (_, checkpin) = self.buttons.get(button);
                !checkpin.is_high()
            };

            if pressed {
                // short debounce delay
                Timer::after(Duration::from_millis(10)).await;

                // second sample
                let still_pressed = {
                    let (_, checkpin) = self.buttons.get(button);
                    !checkpin.is_high()
                };
                match button {
                    Button::Auto => self.backend.set_ac_toggle(),
                    Button::Demist => self.backend.next_mode(),
                    Button::TempUp => self.backend.set_set_temp(self.backend.set_temp() + 1),
                    Button::Off => self.backend.set_fan_speed(0),
                    Button::FanLo => self.backend.set_fan_speed(50),
                    Button::FanHigh => self.backend.set_fan_speed(100),
                    Button::Recirc => self.backend.set_recirc_toggle(),
                    Button::TempDown => self.backend.set_set_temp(self.backend.set_temp() - 1),
                }
                if still_pressed {
                    Timer::after(Duration::from_millis(100)).await;

                    let is_held = {
                        let (_, checkpin) = self.buttons.get(button);
                        !checkpin.is_high()
                    };

                    if is_held {
                        loop {
                            Timer::after(Duration::from_millis(100)).await;

                            let still_pressed = {
                                let (_, checkpin) = self.buttons.get(button);
                                !checkpin.is_high()
                            };
                            if !still_pressed {
                                break;
                            }
                            match button {
                                Button::Auto => self.backend.set_ac_toggle(),
                                Button::Demist => self.backend.next_mode(),
                                Button::TempUp => {
                                    self.backend.set_set_temp(self.backend.set_temp() + 1)
                                }
                                Button::Off => self.backend.set_fan_speed(0),
                                Button::FanLo => self.backend.set_fan_speed(50),
                                Button::FanHigh => self.backend.set_fan_speed(100),
                                Button::Recirc => self.backend.set_recirc_toggle(),
                                Button::TempDown => {
                                    self.backend.set_set_temp(self.backend.set_temp() - 1)
                                }
                            }
                            self.update_display().await
                        }
                    }
                }
            }

            // 4) restore pin to input + pull-up
            {
                let (setpin, _) = self.buttons.get(button);
                setpin.set_as_input();
                setpin.set_pull(Pull::Up);
            }
        }
    }

    fn led_writer(&mut self) {
        match self.backend.mode() {
            ClimateControlMode::Face => {
                self.defrost_led.set_high();
                self.demist_led.set_low();
            }
            ClimateControlMode::Feet => {
                self.defrost_led.set_high();
                self.demist_led.set_high();
            }
            ClimateControlMode::FaceFeet => {
                self.defrost_led.set_high();
                self.demist_led.set_high();
            }
            ClimateControlMode::FeetDef => {
                self.defrost_led.set_high();
                self.demist_led.set_high();
            }
            ClimateControlMode::Def => {
                self.defrost_led.set_low();
                self.demist_led.set_high();
            }
        }

        if self.backend.ac_toggle() == true {
            self.ac_led.set_low();
            self.econ_led.set_high();
        } else if self.backend.ac_toggle() == false {
            self.ac_led.set_high();
            self.econ_led.set_low();
        }

        if self.backend.recirc_toggle() == true {
            self.recirc_led.set_low();
        } else if self.backend.recirc_toggle() == false {
            self.recirc_led.set_high();
        }

        match self.backend.fan_speed() {
            0 => {
                self.fanhigh_led.set_high();
                self.fanlow_led.set_high();
            }
            50 => {
                self.fanhigh_led.set_high();
                self.fanlow_led.set_low();
            }
            100 => {
                self.fanhigh_led.set_low();
                self.fanlow_led.set_high();
            }
            _ => (),
        }
    }

    async fn write_serial(&mut self, input: u128) {
        for i in (0..128).rev() {
            self.serialclock.set_low();
            Timer::after(Duration::from_micros(8)).await;
            self.serialclock.set_high();
            Timer::after(Duration::from_micros(2)).await;
            let gpio_level = (input >> i) & 1 != 0;
            self.serialdata.set_level(gpio_level.into());
        }
    }

    fn write_ic(&mut self, input: u32) {
        let dispvalue = input.to_le_bytes();
        /*self.i2c
        .blocking_write(
            self.chipaddr,
            &[0x00, dispvalue[0], dispvalue[1], dispvalue[2]],
        )
        .unwrap();*/
    }

    pub async fn update_display(&mut self) {
        let mut serialdata = SerialDisplayBits::setup_amb(self.backend.ambient_temp());
        let mut segdata = SegDisplayBits::mode(self.backend.mode())
            | SegDisplayBits::recirc(self.backend.recirc_toggle())
            | SegDisplayBits::ac_toggle(self.backend.ac_toggle())
            | SegDisplayBits::c_or_f(self.backend.displaymode());
        let (serialset, segset) = SerialDisplayBits::setup_set(self.backend.set_temp());
        let tempguage = map_u8(self.backend.set_temp().try_into().unwrap(), 60, 90, 0, 10);
        serialdata = serialdata | serialset | SerialDisplayBits::gauge(tempguage);
        segdata = segdata | SegDisplayBits::set_second(segset);

        self.write_serial(serialdata.bits().into()).await;
        self.write_ic(segdata.bits());
        self.led_writer();
    }
}

#[embassy_executor::task]
// Syncronizer between Seg side and Serial side for Statically controlled LCD
pub async fn serialsyncer() -> ! {
    let inputpin = unsafe { PIN_28::steal() };
    let outputpin = unsafe { PIN_27::steal() };
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
