//! B/W Color for EPDs

use embedded_graphics_core::pixelcolor::raw::{RawData, RawU8};
#[cfg(feature = "graphics")]
use embedded_graphics_core::pixelcolor::BinaryColor;
use embedded_graphics_core::prelude::PixelColor;

#[cfg(feature = "graphics")]
pub use BinaryColor::Off as White;
#[cfg(feature = "graphics")]
pub use BinaryColor::On as Black;

/// When trying to parse u8 to one of the color types
#[derive(Debug, PartialEq, Eq)]
pub struct OutOfColorRangeParseError(u8);
impl core::fmt::Display for OutOfColorRangeParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Outside of possible Color Range: {}", self.0)
    }
}

impl OutOfColorRangeParseError {
    fn _new(size: u8) -> OutOfColorRangeParseError {
        OutOfColorRangeParseError(size)
    }
}

/// Only for the Black/White-Displays
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Color {
    /// Black color
    Dark,
    /// White color
    Green,
}

impl PixelColor for Color {
    type Raw = RawU8;
}

impl Color {
    /// Get the color encoding of the color for one bit
    pub fn get_bit_value(self) -> u8 {
        match self {
            Color::Green => 1u8,
            Color::Dark => 0u8,
        }
    }

    /// Gets a full byte of black or white pixels
    pub fn get_byte_value(self) -> u8 {
        match self {
            Color::Green => 0xff,
            Color::Dark => 0x00,
        }
    }

    /// Parses from u8 to Color
    fn from_u8(val: u8) -> Self {
        match val {
            0 => Color::Dark,
            _ => Color::Green,
        }
    }

    /// Returns the inverse of the given color.
    ///
    /// Black returns White and White returns Black
    pub fn inverse(self) -> Color {
        match self {
            Color::Green => Color::Dark,
            Color::Dark => Color::Green,
        }
    }
}

impl From<u8> for Color {
    fn from(value: u8) -> Self {
        Color::from_u8(value)
    }
}

impl From<RawU8> for Color {
    fn from(val: RawU8) -> Self {
        Self::from_u8(val.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_u8() {
        assert_eq!(Color::Dark, Color::from(0u8));
        assert_eq!(Color::Green, Color::from(1u8));
    }

    // test all values aside from 0 and 1 which all should panic
    #[test]
    fn from_u8_panic() {
        for val in 2..=u8::max_value() {
            extern crate std;
            let result = std::panic::catch_unwind(|| Color::from(val));
            assert!(result.is_err());
        }
    }

    #[test]
    fn u8_conversion_black() {
        assert_eq!(Color::from(Color::Dark.get_bit_value()), Color::Dark);
        assert_eq!(Color::from(0u8).get_bit_value(), 0u8);
    }

    #[test]
    fn u8_conversion_white() {
        assert_eq!(Color::from(Color::Green.get_bit_value()), Color::Green);
        assert_eq!(Color::from(1u8).get_bit_value(), 1u8);
    }
}
