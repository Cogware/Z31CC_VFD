use embedded_graphics::{framebuffer::{buffer_size, Framebuffer}, pixelcolor::{raw::{LittleEndian, RawU1}, BinaryColor}, prelude::{Point, Primitive}, primitives::{PrimitiveStyle, Triangle}};
use embedded_graphics_transform::Transpose;
use embedded_graphics::Drawable;
use smart_leds::RGB8;

pub fn set_tempguage( // 42 HOT 5 COLD
    fb: &mut Transpose<
        Framebuffer<
            BinaryColor,
            RawU1,
            LittleEndian,
            128,
            256,
            { buffer_size::<BinaryColor>(128, 256) }
        >
    >, val: u8
) {
    if val > 36{
        return();
    }
    let fill = PrimitiveStyle::with_fill(BinaryColor::On);
    let pos = map_int(val.into(), 0,36,42, 5);
    Triangle::new(Point::new(21,pos), Point::new(23, pos - 2 ), Point::new(23, pos + 2)).into_styled(fill).draw(fb).unwrap();

}
pub fn set_fanguage( //5 HI 37 LO
    fb: &mut Transpose<
        Framebuffer<
            BinaryColor,
            RawU1,
            LittleEndian,
            128,
            256,
            { buffer_size::<BinaryColor>(128, 256) }
        >
    >, val: u8
) {
    if val > 32{
        return();
    }
    let fill = PrimitiveStyle::with_fill(BinaryColor::On);
    let pos = map_int(val.into(), 0,32,37, 5);
    Triangle::new(Point::new(92,pos), Point::new(94, pos - 2 ), Point::new(94, pos + 2)).into_styled(fill).draw(fb).unwrap();

}

pub fn map_int(x: i32, in_min: i32, in_max: i32, out_min: i32, out_max: i32) -> i32 {
    (x - in_min)
        * (out_max - out_min)
        / (in_max - in_min)
        + out_min
}

pub fn wheel(mut wheel_pos: u8) -> RGB8 {
    wheel_pos = 255 - wheel_pos;
    if wheel_pos < 85 {
        return (255 - wheel_pos * 3, 0, wheel_pos * 3).into();
    }
    if wheel_pos < 170 {
        wheel_pos -= 85;
        return (0, wheel_pos * 3, 255 - wheel_pos * 3).into();
    }
    wheel_pos -= 170;
    (wheel_pos * 3, 255 - wheel_pos * 3, 0).into()
}