use crate::gp1287bi::{DEFAULT_BACKGROUND_COLOR, HEIGHT, NUM_DISPLAY_BITS, WIDTH};
use crate::graphics::{Display, DisplayRotation};
use crate::prelude::Color;
use embedded_graphics_core::prelude::*;

/// Full size buffer for use with the 1in54c EPD
///
/// Can also be manually constructed and be used together with VarDisplay
pub struct Display256x50 {
    buffer: [u8; NUM_DISPLAY_BITS as usize],
    rotation: DisplayRotation,
}

impl Default for Display256x50 {
    fn default() -> Self {
        Display256x50 {
            buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value(); NUM_DISPLAY_BITS as usize],
            rotation: DisplayRotation::default(),
        }
    }
}

impl DrawTarget for Display256x50 {
    type Color = Color;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            self.draw_helper(WIDTH, HEIGHT, pixel)?;
        }
        Ok(())
    }
}

impl OriginDimensions for Display256x50 {
    fn size(&self) -> Size {
        Size::new(WIDTH, HEIGHT)
    }
}

impl Display for Display256x50 {
    fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    fn get_mut_buffer(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    fn set_rotation(&mut self, rotation: DisplayRotation) {
        self.rotation = rotation;
    }

    fn rotation(&self) -> DisplayRotation {
        self.rotation
    }
}
