//! Graphics Support for EPDs

use crate::buffer_len;
use crate::color::Color;
use embedded_graphics_core::prelude::*;

/// Displayrotation
#[derive(Clone, Copy, Default)]
pub enum DisplayRotation {
    /// No rotation
    #[default]
    Rotate0,
    /// Rotate by 90 degrees clockwise
    Rotate90,
    /// Rotate by 180 degrees clockwise
    Rotate180,
    /// Rotate 270 degrees clockwise
    Rotate270,
}

/// Display specific pixel output configuration
///
/// Different chromatic displays differently treat the bits in chromatic color planes.
/// Some of them ([crate::epd2in13bc]) will render a color pixel if bit is set for that pixel,
/// which is a [DisplayColorRendering::Positive] mode.
///
/// Other displays, like [crate::epd5in83b_v2] in opposite, will draw color pixel if bit is
/// cleared for that pixel, which is a [DisplayColorRendering::Negative] mode.
///
#[derive(Clone, Copy)]
pub enum DisplayColorRendering {
    /// Positive: chromatic doesn't override white, white bit cleared for black, white bit set for white, both bits set for chromatic
    Positive,
    /// Negative: chromatic does override white, both bits cleared for black, white bit set for white, red bit set for black
    Negative,
}

/// Necessary traits for all displays to implement for drawing
///
/// Adds support for:
/// - Drawing (With the help of DrawTarget/Embedded Graphics)
/// - Rotations
/// - Clearing
pub trait Display: DrawTarget<Color = Color> {
    /// Clears the buffer of the display with the chosen background color
    fn clear_buffer(&mut self, background_color: Color) {
        for elem in self.get_mut_buffer().iter_mut() {
            *elem = background_color.get_byte_value();
        }
    }

    /// Returns the buffer
    fn buffer(&self) -> &[u8];

    /// Returns a mutable buffer
    fn get_mut_buffer(&mut self) -> &mut [u8];

    /// Sets the rotation of the display
    fn set_rotation(&mut self, rotation: DisplayRotation);

    /// Get the current rotation of the display
    fn rotation(&self) -> DisplayRotation;

    /// Helperfunction for the Embedded Graphics draw trait
    ///
    /// Becomes uneccesary when const_generics become stablised
    fn draw_helper(
        &mut self,
        width: u32,
        height: u32,
        pixel: Pixel<Color>,
    ) -> Result<(), Self::Error> {
        let rotation = self.rotation();
        let buffer = self.get_mut_buffer();

        let Pixel(point, color) = pixel;
        if outside_display(point, width, height, rotation) {
            return Ok(());
        }

        // Give us index inside the buffer and the bit-position in that u8 which needs to be changed
        let (index, bit) = find_position(point.x as u32, point.y as u32, width, height, rotation);
        let index = index as usize;

        // "Draw" the Pixel on that bit
        match color {
            // Black
            Color::Dark => {
                buffer[index] &= !bit;
            }
            // White
            Color::Green => {
                buffer[index] |= bit;
            }
        }
        Ok(())
    }
}

/// A variable Display without a predefined buffer
///
/// The buffer can be created as following:
/// buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value(); WIDTH / 8 * HEIGHT]
/// If WIDTH is not a multiple of 8, don't forget to round it up (ie. (WIDTH + 7) / 8)
pub struct VarDisplay<'a> {
    width: u32,
    height: u32,
    rotation: DisplayRotation,
    buffer: &'a mut [u8], //buffer: Box<u8>//[u8; 15000]
}

impl<'a> VarDisplay<'a> {
    /// Create a new variable sized display.
    ///
    /// Buffersize must be at least (width + 7) / 8 * height bytes.
    pub fn new(width: u32, height: u32, buffer: &'a mut [u8]) -> VarDisplay<'a> {
        let len = buffer.len() as u32;
        assert!(buffer_len(width as usize, height as usize) >= len as usize);
        VarDisplay {
            width,
            height,
            rotation: DisplayRotation::default(),
            buffer,
        }
    }
}

impl<'a> DrawTarget for VarDisplay<'a> {
    type Color = Color;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            self.draw_helper(self.width, self.height, pixel)?;
        }
        Ok(())
    }
}

impl<'a> OriginDimensions for VarDisplay<'a> {
    fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }
}

impl<'a> Display for VarDisplay<'a> {
    fn buffer(&self) -> &[u8] {
        self.buffer
    }

    fn get_mut_buffer(&mut self) -> &mut [u8] {
        self.buffer
    }

    fn set_rotation(&mut self, rotation: DisplayRotation) {
        self.rotation = rotation;
    }

    fn rotation(&self) -> DisplayRotation {
        self.rotation
    }
}

// Checks if a pos is outside the defined display
fn outside_display(p: Point, width: u32, height: u32, rotation: DisplayRotation) -> bool {
    if p.x < 0 || p.y < 0 {
        return true;
    }
    let (x, y) = (p.x as u32, p.y as u32);
    match rotation {
        DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => {
            if x >= width || y >= height {
                return true;
            }
        }
        DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => {
            if y >= width || x >= height {
                return true;
            }
        }
    }
    false
}

fn find_rotation(x: u32, y: u32, width: u32, height: u32, rotation: DisplayRotation) -> (u32, u32) {
    let nx;
    let ny;
    match rotation {
        DisplayRotation::Rotate0 => {
            nx = x;
            ny = y;
        }
        DisplayRotation::Rotate90 => {
            nx = width - 1 - y;
            ny = x;
        }
        DisplayRotation::Rotate180 => {
            nx = width - 1 - x;
            ny = height - 1 - y;
        }
        DisplayRotation::Rotate270 => {
            nx = y;
            ny = height - 1 - x;
        }
    }
    (nx, ny)
}

#[rustfmt::skip]
//returns index position in the u8-slice and the bit-position inside that u8
fn find_position(x: u32, y: u32, width: u32, height: u32, rotation: DisplayRotation) -> (u32, u8) {
    let (nx, ny) = find_rotation(x, y, width, height, rotation);
    (
        nx / 8 + ((width + 7) / 8) * ny,
        0x80 >> (nx % 8),
    )
}
