use bitflags::bitflags;
use embassy_rp::{gpio::{Input, Level, Output, Pull}, i2c::{Blocking, I2c}, peripherals::{I2C1, PIN_10, PIN_9}};
use embassy_time::{block_for, Duration, Timer};

use crate::climatecontrol::{ClimateControlBacker, ClimateControlMode};

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

    pub fn set_neg(b: bool) -> SerialDisplayBits{
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

    pub fn setup_set(input: i8) -> (SerialDisplayBits, i8){
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
        base = base
            | SerialDisplayBits::amb_first(tens.try_into().unwrap());
        return (base, ones);
    }

}

#[allow(unused)]
impl SegDisplayBits {

    pub fn get_bitsout(input: SegDisplayBits) -> u32{
        input.bits()
    }

    pub fn set_second(n: i8) -> SegDisplayBits{
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
            7 => {
                SegDisplayBits::SET2_T | SegDisplayBits::SET2_TR | SegDisplayBits::SET2_BR
            }
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

    pub fn recirc(b: bool) -> SegDisplayBits{
        if b == true {
            return SegDisplayBits::RECIRC
        }
        SegDisplayBits::FRESH_AIR
    }

    pub fn mode(input: &ClimateControlMode) -> SegDisplayBits{
        match input{
            ClimateControlMode::Face => SegDisplayBits::FACE | SegDisplayBits::BACKGROUND | SegDisplayBits::FAN,
            ClimateControlMode::Feet => SegDisplayBits::FEET | SegDisplayBits::BACKGROUND | SegDisplayBits::FAN,
            ClimateControlMode::FaceFeet => SegDisplayBits::FACE | SegDisplayBits::FEET | SegDisplayBits::BACKGROUND | SegDisplayBits::FAN,
            ClimateControlMode::FeetDef => SegDisplayBits::FEET | SegDisplayBits::DEFROST | SegDisplayBits::BACKGROUND | SegDisplayBits::FAN,
            ClimateControlMode::Def => SegDisplayBits::DEFROST | SegDisplayBits::BACKGROUND | SegDisplayBits::FAN,
        }
    }

    pub fn ac_toggle(b: bool) -> SegDisplayBits{
        if b == true{
            return SegDisplayBits::AC
        }
        SegDisplayBits::EMPTY
    }

    pub fn c_or_f (b: bool) -> SegDisplayBits{
        if b == true{
            return SegDisplayBits::CELCIUS
        }
        SegDisplayBits::FARENHEIT
    }

    pub fn heat_watercock(b: bool) -> SegDisplayBits{
        if b == true{
            return SegDisplayBits::HEAT
        }
        SegDisplayBits::EMPTY
    }


}

pub struct DigiDisplay<'a>{
    i2c: I2c<'a, I2C1, Blocking>,
    serialclock: Output<'a>,
    serialdata: Output<'a>,
    chipaddr: u8,

}

impl<'a> DigiDisplay<'a>{
    pub fn new(mut i2c: I2c<'a, I2C1, Blocking>, serialclock: Output<'a>, serialdata: Output<'a>) -> Self{
        let chipaddr = 0x38;
        block_for(Duration::from_millis(2));
        i2c.blocking_write(chipaddr, &[0x49]).unwrap();

        DigiDisplay { i2c, serialclock, serialdata, chipaddr }

    }

    async fn write_serial(&mut self, input: u128){
        for i in (0..128).rev() {
            self.serialclock.set_low();
            Timer::after(Duration::from_micros(8)).await;
            self.serialclock.set_high();
            Timer::after(Duration::from_micros(2)).await;
            let gpio_level = (input >> i) & 1 != 0;
            self.serialdata.set_level(gpio_level.into());
        }
    }

    fn write_ic(&mut self, input: u32){
        let dispvalue = input.to_le_bytes();
        self.i2c.blocking_write(self.chipaddr, &[0x00, dispvalue[0], dispvalue[1], dispvalue[2]])
            .unwrap();
    }

    pub async fn update_display(&mut self, settings: &ClimateControlBacker){
        let mut serialdata = SerialDisplayBits::setup_amb(settings.ambient_temp());
        let mut segdata = SegDisplayBits::mode(settings.mode()) | SegDisplayBits::recirc(settings.recirc_toggle()) | SegDisplayBits::ac_toggle(settings.ac_toggle()) | SegDisplayBits::c_or_f(settings.displaymode());
        let (serialset, segset) = SerialDisplayBits::setup_set(settings.set_temp());
        serialdata = serialdata | serialset;
        segdata = segdata | SegDisplayBits::set_second(segset);

        self.write_serial(serialdata.bits().into()).await;
        self.write_ic(segdata.bits());
    }


}

#[embassy_executor::task]
// Syncronizer between Seg side and Serial side for Statically controlled LCD
pub async fn serialsyncer() -> ! {
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