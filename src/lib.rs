#![no_std]

extern crate alloc;

pub mod display;
pub mod graphics;

#[allow(unused)]
pub(crate) fn map_u8(x: u8, in_min: u8, in_max: u8, out_min: u8, out_max: u8) -> u8 {
    (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
}

#[allow(unused)]
pub(crate) fn map_i8(x: i8, in_min: i8, in_max: i8, out_min: i8, out_max: i8) -> i8 {
    (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
}

#[allow(unused)]
pub(crate) fn map_i32(x: i32, in_min: i32, in_max: i32, out_min: i32, out_max: i32) -> i32 {
    (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
}

#[allow(unused)]
pub fn wheel(mut wheel_pos: u8) -> smart_leds::RGB8 {
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
