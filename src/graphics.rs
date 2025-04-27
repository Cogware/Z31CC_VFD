use alloc::format;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::{FONT_7X13, FONT_8X13_BOLD};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Triangle};
use embedded_graphics::text::Text;
use tinybmp::Bmp;

const FAIRLADYBMP: &'static [u8] = include_bytes!("../assets/fairlady.bmp");
const CC_BACKGROUND: &'static [u8] = include_bytes!("../assets/ClimateControlBackground.bmp");
const CC_FACE: &'static [u8] = include_bytes!("../assets/Face.bmp");
const CC_FEET: &'static [u8] = include_bytes!("../assets/Feet.bmp");
const CC_FACE_FEET: &'static [u8] = include_bytes!("../assets/FaceandFeet.bmp");
const CC_DEF: &'static [u8] = include_bytes!("../assets/Def.bmp");

pub trait BinaryTarget: DrawTarget<Color = BinaryColor> {}
impl<T> BinaryTarget for T where T: DrawTarget<Color = BinaryColor> {}

pub enum ClimateControlMode {
    Face,
    Feet,
    FaceFeet,
    FeetDef,
    Def,
}

pub struct Image {
    data: Bmp<'static, BinaryColor>,
    offset: Point,
}

impl Drawable for Image {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        self.data.draw(&mut target.translated(self.offset))
    }
}

pub struct Graphics {
    boot: Image,
    background: Image,
    cc_face: Image,
    cc_feet: Image,
    cc_face_feet: Image,
    cc_def: Image,
    fill: PrimitiveStyle<BinaryColor>,
    def_text: Text<'static, MonoTextStyle<'static, BinaryColor>>,
    temp_font: MonoTextStyle<'static, BinaryColor>,
    off_font: MonoTextStyle<'static, BinaryColor>,
}

impl Graphics {
    pub fn load() -> Self {
        let boot = Image {
            data: Bmp::from_slice(FAIRLADYBMP).unwrap(),
            offset: Point::new(5, 14),
        };
        let background = Image {
            data: Bmp::from_slice(CC_BACKGROUND).unwrap(),
            offset: Point::new(0, 0),
        };
        let cc_face = Image {
            data: Bmp::from_slice(CC_FACE).unwrap(),
            offset: Point::new(135, 2),
        };
        let cc_feet = Image {
            data: Bmp::from_slice(CC_FEET).unwrap(),
            offset: Point::new(135, 2),
        };
        let cc_face_feet = Image {
            data: Bmp::from_slice(CC_FACE_FEET).unwrap(),
            offset: Point::new(135, 2),
        };
        let cc_def = Image {
            data: Bmp::from_slice(CC_DEF).unwrap(),
            offset: Point::new(135, 2),
        };

        let temp_font = MonoTextStyle::new(&FONT_8X13_BOLD, BinaryColor::On);
        let off_font = MonoTextStyle::new(&FONT_7X13, BinaryColor::On);

        let fill = PrimitiveStyle::with_fill(BinaryColor::On);
        let def_text = Text::new("DEF", Point::new(130, 12), temp_font);

        Self {
            boot,
            background,
            cc_face,
            cc_feet,
            cc_face_feet,
            cc_def,
            def_text,
            temp_font,
            off_font,
            fill,
        }
    }

    pub fn draw_boot_image<D: BinaryTarget>(&self, display: &mut D) {
        _ = self.boot.draw(display)
    }

    pub fn draw_background<D: BinaryTarget>(&self, display: &mut D) {
        _ = self.background.draw(display)
    }

    pub fn draw_climate_control_mode<D: BinaryTarget>(
        &self,
        mode: &ClimateControlMode,
        display: &mut D,
    ) {
        match mode {
            ClimateControlMode::Face => _ = self.cc_face.draw(display),
            ClimateControlMode::Feet => _ = self.cc_feet.draw(display),
            ClimateControlMode::FaceFeet => _ = self.cc_face_feet.draw(display),
            ClimateControlMode::FeetDef => {
                _ = self.cc_feet.draw(display);
                _ = self.def_text.draw(display);
            }
            ClimateControlMode::Def => {
                _ = self.cc_def.draw(display);
                _ = self.def_text.draw(display);
            }
        }
    }

    pub fn draw_ac_toggle<D: BinaryTarget>(&self, toggle_on: bool, display: &mut D) {
        match toggle_on {
            true => _ = Text::new("ON", Point::new(107, 19), self.temp_font).draw(display),
            false => _ = Text::new("OFF", Point::new(105, 19), self.off_font).draw(display),
        }
    }

    pub fn draw_recirc_toggle<D: BinaryTarget>(&self, toggle_on: bool, display: &mut D) {
        match toggle_on {
            true => _ = Text::new("ON", Point::new(107, 44), self.temp_font).draw(display),
            false => _ = Text::new("OFF", Point::new(105, 44), self.off_font).draw(display),
        }
    }

    pub fn draw_internal_temp<D: BinaryTarget>(&self, temp: i8, display: &mut D) {
        // TODO: is this what you want? you offset the ambient temp so I did it here too
        let point = {
            if temp < 0 {
                Point::new(35, 12)
            } else {
                Point::new(43, 12)
            }
        };

        _ = Text::new(&format!("{temp}"), point, self.temp_font).draw(display);
    }

    pub fn draw_ambient_temp<D: BinaryTarget>(&self, temp: i8, display: &mut D) {
        let point = {
            if temp < 0 {
                Point::new(35, 37)
            } else {
                Point::new(43, 37)
            }
        };

        _ = Text::new(&format!("{temp}"), point, self.temp_font).draw(display);
    }

    pub fn draw_fan_gauge<D: BinaryTarget>(&self, pos: i32, display: &mut D) {
        _ = Triangle::new(
            Point::new(92, pos),
            Point::new(94, pos - 2),
            Point::new(94, pos + 2),
        )
        .into_styled(self.fill)
        .draw(display);
    }

    pub fn draw_temp_guage<D: BinaryTarget>(&self, pos: i32, display: &mut D) {
        _ = Triangle::new(
            Point::new(21, pos),
            Point::new(23, pos - 2),
            Point::new(23, pos + 2),
        )
        .into_styled(self.fill)
        .draw(display);
    }
}
