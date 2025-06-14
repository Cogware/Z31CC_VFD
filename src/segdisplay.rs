use bitflags::bitflags;

bitflags! {
    /*pub struct Segbits: u32{
        0x0000_0001, // freshair
        0x0000_0002, // face
        0x0000_0004, // fan
        0x0000_0008, // defrost
        0x0000_0010, // feet
        0x0000_0020, // ACGas
        0x0000_0040, // Unused
        0x0000_0080, // Celcius
        0x0000_0100, // Set second bottem right
        0x0000_0200, // Set second bottem
        0x0000_0400, // set second bottem left
        0x0000_0800, // filler backer
        0x0000_1000, // farenheit
        0x0000_2000, // A/C
        0x0000_4000, // heat icon?
        0x0000_8000, // recirc
        0x0001_0000, // bit 16
        0x0002_0000, // bit 17
        0x0004_0000, // bit 18
        0x0008_0000, // bit 19
        0x0010_0000, // set second top left 
        0x0020_0000, // set second top
        0x0040_0000, // set second topright
        0x0080_0000, // set second middle
    }*/

    /// Every single segment/control bit in your display:
    pub struct DisplayBits: u64 {
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

impl DisplayBits {
    /// Pattern for ambient “second” (ones) digit 0x0–0xF
    pub fn amb_second(n: u8) -> DisplayBits {
        match n {
            0  => DisplayBits::AMB2_T  | DisplayBits::AMB2_TR | DisplayBits::AMB2_BR | DisplayBits::AMB2_B  | DisplayBits::AMB2_BL | DisplayBits::AMB2_TL,
            1  => DisplayBits::AMB2_TR | DisplayBits::AMB2_BR,
            2  => DisplayBits::AMB2_T  | DisplayBits::AMB2_TR | DisplayBits::AMB2_M | DisplayBits::AMB2_BL | DisplayBits::AMB2_B,
            3  => DisplayBits::AMB2_T  | DisplayBits::AMB2_TR | DisplayBits::AMB2_M | DisplayBits::AMB2_BR | DisplayBits::AMB2_B,
            4  => DisplayBits::AMB2_TL | DisplayBits::AMB2_M  | DisplayBits::AMB2_TR | DisplayBits::AMB2_BR,
            5  => DisplayBits::AMB2_T  | DisplayBits::AMB2_TL | DisplayBits::AMB2_M | DisplayBits::AMB2_BR | DisplayBits::AMB2_B,
            6  => DisplayBits::AMB2_T  | DisplayBits::AMB2_TL | DisplayBits::AMB2_M | DisplayBits::AMB2_BR | DisplayBits::AMB2_B  | DisplayBits::AMB2_BL,
            7  => DisplayBits::AMB2_T  | DisplayBits::AMB2_TR | DisplayBits::AMB2_BR,
            8  => DisplayBits::AMB2_T  | DisplayBits::AMB2_TR | DisplayBits::AMB2_BR | DisplayBits::AMB2_B  | DisplayBits::AMB2_BL | DisplayBits::AMB2_TL | DisplayBits::AMB2_M,
            9  => DisplayBits::AMB2_T  | DisplayBits::AMB2_TR | DisplayBits::AMB2_TL | DisplayBits::AMB2_M  | DisplayBits::AMB2_BR | DisplayBits::AMB2_B,
            0xA=> DisplayBits::AMB2_T  | DisplayBits::AMB2_TR | DisplayBits::AMB2_TL | DisplayBits::AMB2_M  | DisplayBits::AMB2_BL | DisplayBits::AMB2_BR,
            0xB=> DisplayBits::AMB2_M  | DisplayBits::AMB2_BL | DisplayBits::AMB2_B | DisplayBits::AMB2_BR | DisplayBits::AMB2_TL,
            0xC=> DisplayBits::AMB2_T  | DisplayBits::AMB2_TL | DisplayBits::AMB2_BL | DisplayBits::AMB2_B,
            0xD=> DisplayBits::AMB2_M  | DisplayBits::AMB2_TR | DisplayBits::AMB2_BR | DisplayBits::AMB2_BL | DisplayBits::AMB2_B,
            0xE=> DisplayBits::AMB2_T  | DisplayBits::AMB2_TL | DisplayBits::AMB2_M | DisplayBits::AMB2_BL | DisplayBits::AMB2_B,
            0xF=> DisplayBits::AMB2_T  | DisplayBits::AMB2_TL | DisplayBits::AMB2_M | DisplayBits::AMB2_BL,
            _  => DisplayBits::empty(),
        }
    }

    /// Pattern for ambient “first” (tens) digit 0x0–0xF
    pub fn amb_first(n: u8) -> DisplayBits {
        match n {
            0  => DisplayBits::AMB1_T  | DisplayBits::AMB1_TR | DisplayBits::AMB1_BR | DisplayBits::AMB1_B  | DisplayBits::AMB1_BL | DisplayBits::AMB1_TL,
            1  => DisplayBits::AMB1_TR | DisplayBits::AMB1_BR,
            2  => DisplayBits::AMB1_T  | DisplayBits::AMB1_TR | DisplayBits::AMB1_M | DisplayBits::AMB1_BL | DisplayBits::AMB1_B,
            3  => DisplayBits::AMB1_T  | DisplayBits::AMB1_TR | DisplayBits::AMB1_M | DisplayBits::AMB1_BR | DisplayBits::AMB1_B,
            4  => DisplayBits::AMB1_TL | DisplayBits::AMB1_M  | DisplayBits::AMB1_TR | DisplayBits::AMB1_BR,
            5  => DisplayBits::AMB1_T  | DisplayBits::AMB1_TL | DisplayBits::AMB1_M | DisplayBits::AMB1_BR | DisplayBits::AMB1_B,
            6  => DisplayBits::AMB1_T  | DisplayBits::AMB1_TL | DisplayBits::AMB1_M | DisplayBits::AMB1_BR | DisplayBits::AMB1_B  | DisplayBits::AMB1_BL,
            7  => DisplayBits::AMB1_T  | DisplayBits::AMB1_TR | DisplayBits::AMB1_BR,
            8  => DisplayBits::AMB1_T  | DisplayBits::AMB1_TR | DisplayBits::AMB1_BR | DisplayBits::AMB1_B  | DisplayBits::AMB1_BL | DisplayBits::AMB1_TL | DisplayBits::AMB1_M,
            9  => DisplayBits::AMB1_T  | DisplayBits::AMB1_TR | DisplayBits::AMB1_TL | DisplayBits::AMB1_M  | DisplayBits::AMB1_BR | DisplayBits::AMB1_B,
            0xA=> DisplayBits::AMB1_T  | DisplayBits::AMB1_TR | DisplayBits::AMB1_TL | DisplayBits::AMB1_M  | DisplayBits::AMB1_BL | DisplayBits::AMB1_BR,
            0xB=> DisplayBits::AMB1_M  | DisplayBits::AMB1_BL | DisplayBits::AMB1_B | DisplayBits::AMB1_BR | DisplayBits::AMB1_TL,
            0xC=> DisplayBits::AMB1_T  | DisplayBits::AMB1_TL | DisplayBits::AMB1_BL | DisplayBits::AMB1_B,
            0xD=> DisplayBits::AMB1_M  | DisplayBits::AMB1_TR | DisplayBits::AMB1_BR | DisplayBits::AMB1_BL | DisplayBits::AMB1_B,
            0xE=> DisplayBits::AMB1_T  | DisplayBits::AMB1_TL | DisplayBits::AMB1_M | DisplayBits::AMB1_BL | DisplayBits::AMB1_B,
            0xF=> DisplayBits::AMB1_T  | DisplayBits::AMB1_TL | DisplayBits::AMB1_M | DisplayBits::AMB1_BL,
            _  => DisplayBits::empty(),
        }
    }
    pub fn amb_neg(b: bool) -> DisplayBits{
        if b == true
        {
            return DisplayBits::AMB_NEG
        }
        DisplayBits::EMPTY
    }

    pub fn amb_hund(b: bool) -> DisplayBits{
        if b == true
        {
            return DisplayBits::AMB_ONE
        }
        DisplayBits::EMPTY
    }

    /// Pattern for the “set” digit (tens) 0x0–0xF
    pub fn set_first(n: u8) -> DisplayBits {
        match n {
            0  => DisplayBits::SET1_T  | DisplayBits::SET1_TR | DisplayBits::SET1_BR | DisplayBits::SET1_B  | DisplayBits::SET1_BL | DisplayBits::SET1_TL,
            1  => DisplayBits::SET1_TR | DisplayBits::SET1_BR,
            2  => DisplayBits::SET1_T  | DisplayBits::SET1_TR | DisplayBits::SET1_M | DisplayBits::SET1_BL | DisplayBits::SET1_B,
            3  => DisplayBits::SET1_T  | DisplayBits::SET1_TR | DisplayBits::SET1_M | DisplayBits::SET1_BR | DisplayBits::SET1_B,
            4  => DisplayBits::SET1_TL | DisplayBits::SET1_M  | DisplayBits::SET1_TR | DisplayBits::SET1_BR,
            5  => DisplayBits::SET1_T  | DisplayBits::SET1_TL | DisplayBits::SET1_M | DisplayBits::SET1_BR | DisplayBits::SET1_B,
            6  => DisplayBits::SET1_T  | DisplayBits::SET1_TL | DisplayBits::SET1_M | DisplayBits::SET1_BR | DisplayBits::SET1_B  | DisplayBits::SET1_BL,
            7  => DisplayBits::SET1_T  | DisplayBits::SET1_TR | DisplayBits::SET1_BR,
            8  => DisplayBits::SET1_T  | DisplayBits::SET1_TR | DisplayBits::SET1_BR | DisplayBits::SET1_B  | DisplayBits::SET1_BL | DisplayBits::SET1_TL | DisplayBits::SET1_M,
            9  => DisplayBits::SET1_T  | DisplayBits::SET1_TR | DisplayBits::SET1_TL | DisplayBits::SET1_M  | DisplayBits::SET1_BR | DisplayBits::SET1_B,
            0xA=> DisplayBits::SET1_T  | DisplayBits::SET1_TR | DisplayBits::SET1_TL | DisplayBits::SET1_M  | DisplayBits::SET1_BL | DisplayBits::SET1_BR,
            0xB=> DisplayBits::SET1_M  | DisplayBits::SET1_BL | DisplayBits::SET1_B | DisplayBits::SET1_BR | DisplayBits::SET1_TL,
            0xC=> DisplayBits::SET1_T  | DisplayBits::SET1_TL | DisplayBits::SET1_BL | DisplayBits::SET1_B,
            0xD=> DisplayBits::SET1_M  | DisplayBits::SET1_TR | DisplayBits::SET1_BR | DisplayBits::SET1_BL | DisplayBits::SET1_B,
            0xE=> DisplayBits::SET1_T  | DisplayBits::SET1_TL | DisplayBits::SET1_M | DisplayBits::SET1_BL | DisplayBits::SET1_B,
            0xF=> DisplayBits::SET1_T  | DisplayBits::SET1_TL | DisplayBits::SET1_M | DisplayBits::SET1_BL,
            _  => DisplayBits::empty(),
        }
    }

    /// Build the thermometer‐style gauge for levels –5…+5.
    pub fn gauge(level: u8) -> DisplayBits {
        match level {
             0 => DisplayBits::TG_NEG5,
             1 => DisplayBits::TG_NEG4,
             2 => DisplayBits::TG_NEG3,
             3 => DisplayBits::TG_NEG2,
             4 => DisplayBits::TG_NEG1,
             5 => DisplayBits::TG_ZERO,
             6 => DisplayBits::TG_PLUS1,
             7 => DisplayBits::TG_PLUS2,
             8 => DisplayBits::TG_PLUS3,
             9 => DisplayBits::TG_PLUS4,
             10 => DisplayBits::TG_PLUS5,
            _ => DisplayBits::empty(),
        }
    }

    pub fn set_amb(input: i8) -> DisplayBits {
        let mut base = DisplayBits::EMPTY;
        let mut n = input;
        if n < 0{
            n = n * -1;
            base = base | DisplayBits::amb_neg(true);
        }
        if n >=100{
            n = n - 100;
            base = base | DisplayBits::amb_hund(true);
        }
        let tens = n / 10;
        let ones = n % 10;
        base = base | DisplayBits::amb_first(tens.try_into().unwrap()) | DisplayBits::amb_second(ones.try_into().unwrap());
        return base;
    }
}