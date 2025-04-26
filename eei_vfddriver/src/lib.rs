//! A simple Driver for the EEI Tech VFD Displays via SPI
//!
//! - Built using [`embedded-hal`] traits.
//! - Graphics support is added through [`embedded-graphics`]
//!
//! [`embedded-graphics`]: https://docs.rs/embedded-graphics/
//! [`embedded-hal`]: https://docs.rs/embedded-hal

#![no_std]
#![deny(missing_docs)]

#[cfg(feature = "graphics")]
pub mod graphics;

mod traits;

pub mod color;

/// Interface for the physical connection between display and the controlling device
mod interface;

pub mod gp1287bi;

/// Includes everything important besides the chosen Display
pub mod prelude {
    pub use crate::color::Color;
    pub use crate::traits::EEIDisplay;

    pub use crate::SPI_MODE;

    #[cfg(feature = "graphics")]
    pub use crate::graphics::{Display, DisplayRotation};
}

/// Computes the needed buffer length. Takes care of rounding up in case width
/// is not divisible by 8.
///
///  unused
///  bits        width
/// <----><------------------------>
/// \[XXXXX210\]\[76543210\]...\[76543210\] ^
/// \[XXXXX210\]\[76543210\]...\[76543210\] | height
/// \[XXXXX210\]\[76543210\]...\[76543210\] v
pub const fn buffer_len(width: usize, height: usize) -> usize {
    (width + 7) / 8 * height
}

use embedded_hal::spi::{Mode, Phase, Polarity};

/// SPI mode -
/// For more infos see [Requirements: SPI](index.html#spi)
pub const SPI_MODE: Mode = Mode {
    phase: Phase::CaptureOnFirstTransition,
    polarity: Polarity::IdleLow,
};
