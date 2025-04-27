use alloc::format;
use eei_vfd::{gp1287bi::VFD256x50, prelude::EEIDisplay};
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_rp::{
    gpio::Output,
    peripherals::SPI0,
    spi::{Blocking, Spi},
};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::{Delay, Duration, Ticker};
use embedded_graphics::{image::Image, mono_font::{ascii::{FONT_7X13, FONT_8X13_BOLD}, MonoTextStyle}, text::Text, Drawable};
use embedded_graphics::{
    framebuffer::{Framebuffer, buffer_size},
    pixelcolor::{
        BinaryColor,
        raw::{LittleEndian, RawU1},
    },
    prelude::{Point, Primitive},
    primitives::{PrimitiveStyle, Triangle},
};
use embedded_graphics_transform::Transpose;
use smart_leds::RGB8;
use tinybmp::Bmp;

const FAIRLADYBMP: &'static [u8] = include_bytes!("../assets/fairlady.bmp");
const CCBACKGROUND: &'static [u8] = include_bytes!("../assets/ClimateControlBackground.bmp");
const CCFACE: &'static [u8] = include_bytes!("../assets/Face.bmp");
const CCFEET: &'static [u8] = include_bytes!("../assets/Feet.bmp");
const CCFACEFEET: &'static [u8] = include_bytes!("../assets/FaceandFeet.bmp");
const CCDEF: &'static [u8] = include_bytes!("../assets/Def.bmp");


pub type InternalFrameBuffer = Transpose<Framebuffer<BinaryColor,RawU1,LittleEndian,128,256,{ buffer_size::<BinaryColor>(128, 256) },>,>;

pub enum Mode{
    Face,
    Feet,
    FaceFeet,
    FeetDef,
    Def
}

pub struct Display<'a> {
    vfd: VFD256x50<
        SpiDeviceWithConfig<'a, NoopRawMutex, Spi<'a, SPI0, Blocking>, Output<'a>>,
        Output<'a>,
        Delay,
    >,
    actoggle: bool,
    recirctoggle: bool,
    framebuffer: InternalFrameBuffer,
    bootimage: Bmp<'a, BinaryColor>,
    background: Bmp<'a, BinaryColor>,
    ccface: Bmp<'a, BinaryColor>,
    ccfeet: Bmp<'a, BinaryColor>,
    ccfacefeet: Bmp<'a, BinaryColor>,
    ccdef: Bmp<'a, BinaryColor>,
    tempfont: MonoTextStyle<'a, BinaryColor>,
    offfont: MonoTextStyle<'a, BinaryColor>,
    fill: PrimitiveStyle<BinaryColor>,
    mode: Mode,
    internaltemp: i8,
    ambtemp: i8,
    tempguage: u8,
    fanguage: u8,
}

impl<'a> Display<'a> {
    pub fn new(
        spibus: SpiDeviceWithConfig<'a, NoopRawMutex, Spi<'a, SPI0, Blocking>, Output<'a>>,
        reset: Output<'a>,
    ) -> Self {
        let mut vfd: VFD256x50<_, _, _> = EEIDisplay::new(spibus, reset, Delay).unwrap();

        vfd.clear_frame().unwrap();

        let fb = Framebuffer::<
            BinaryColor,
            _,
            LittleEndian,
            128,
            256,
            { buffer_size::<BinaryColor>(128, 256) },
        >::new();

        let vfdfb = Transpose::new(fb);

        let bootimage = Bmp::from_slice(FAIRLADYBMP).unwrap();
        let background = Bmp::from_slice(CCBACKGROUND).unwrap();
        let ccface = Bmp::from_slice(CCFACE).unwrap();
        let ccfeet = Bmp::from_slice(CCFEET).unwrap();
        let ccfacefeet = Bmp::from_slice(CCFACEFEET).unwrap();
        let ccdef = Bmp::from_slice(CCDEF).unwrap();

        let tempfont = MonoTextStyle::new(&FONT_8X13_BOLD, BinaryColor::On);
        let offfont = MonoTextStyle::new(&FONT_7X13, BinaryColor::On);
        let fill = PrimitiveStyle::with_fill(BinaryColor::On);
        let mode = Mode::FeetDef;
        let internaltemp: i8 = 60;
        let ambtemp: i8 = -60;
        let tempguage: u8 = 0;
        let fanguage: u8 = 0;

        let d = Display {
            vfd,
            actoggle: false,
            recirctoggle: false,
            framebuffer: vfdfb,
            bootimage,
            background,
            ccface,
            ccfeet,
            ccfacefeet,
            ccdef,
            tempfont,
            offfont,
            fill,
            mode,
            internaltemp,
            ambtemp,
            tempguage,
            fanguage,
        };
        d
    }

    pub async fn displaybootimage(&mut self) {
        let mut ticker = Ticker::every(Duration::from_secs(1));
        self.vfd.clear_frame().unwrap();

        Image::new(&self.bootimage, Point::new(5, 14)).draw(&mut self.framebuffer).unwrap();

        self.vfd.update_frame(self.framebuffer.data()).unwrap();
        self.vfd.set_brightness(128).unwrap();
        ticker.next().await;
        self.vfd.set_brightness(255).unwrap();
        ticker.next().await;

    }

    // return mutable refrence to framebuffer to use outside this struct
    pub fn useframebuffer( &mut self) -> &mut InternalFrameBuffer {
        &mut self.framebuffer
    }

    fn drawbackground(&mut self){
        Image::new(&self.background, Point::new(0, 0)).draw(&mut self.framebuffer).unwrap();
    }

    fn draw_mode(&mut self){
        match self.mode {
            Mode::Feet => Image::new(&self.ccfeet, Point::new(135, 2)).draw(&mut self.framebuffer).unwrap(),
            Mode::Face => Image::new(&self.ccface, Point::new(135, 2)).draw(&mut self.framebuffer).unwrap(),
            Mode::FaceFeet => Image::new(&self.ccfacefeet, Point::new(135, 2)).draw(&mut self.framebuffer).unwrap(),
            Mode::FeetDef => {Image::new(&self.ccfeet, Point::new(135, 2)).draw(&mut self.framebuffer).unwrap();
                              Text::new("DEF", Point::new(130,12), self.tempfont).draw(&mut self.framebuffer).unwrap();   
                             },
            Mode::Def => {Image::new(&self.ccdef, Point::new(135, 2)).draw(&mut self.framebuffer).unwrap();
                Text::new("DEF", Point::new(130,12), self.tempfont).draw(&mut self.framebuffer).unwrap();   
                         },  
                    }
        if self.actoggle == true{
            Text::new("ON", Point::new(107,19), self.tempfont).draw(&mut self.framebuffer).unwrap();
        } else {
            Text::new("OFF", Point::new(105,19), self.offfont).draw(&mut self.framebuffer).unwrap();
        }
        if self.recirctoggle == true{
            Text::new("ON", Point::new(107,44), self.tempfont).draw(&mut self.framebuffer).unwrap();
        } else {
            Text::new("OFF", Point::new(105,44), self.offfont).draw(&mut self.framebuffer).unwrap();
        }
    }

    fn draw_temps(&mut self){
        Text::new(&format!("{:?}", self.internaltemp), Point::new(43,12), self.tempfont).draw(&mut self.framebuffer).unwrap();

        if self.ambtemp <= -1{
            Text::new(&format!("{:?}", self.ambtemp), Point::new(35,37), self.tempfont).draw(&mut self.framebuffer).unwrap();
        } else {
        Text::new(&format!("{:?}", self.ambtemp), Point::new(43,37), self.tempfont).draw(&mut self.framebuffer).unwrap();
        }
    }

    fn draw_fanguage(&mut self) { //5 HI 37 LO
        if self.fanguage > 32 {
            return ();
        }
        let pos = map_int(self.fanguage.into(), 0, 32, 37, 5);
        Triangle::new(
            Point::new(92, pos),
            Point::new(94, pos - 2),
            Point::new(94, pos + 2),
        )
        .into_styled(self.fill)
        .draw(&mut self.framebuffer)
        .unwrap();
    }

    fn draw_tempguage(&mut self) {// 42 HOT 5 COLD
        if self.tempguage > 36 {
            return ();
        }
        let pos = map_int(self.tempguage.into(), 0, 36, 42, 5);
        Triangle::new(
            Point::new(21, pos),
            Point::new(23, pos - 2),
            Point::new(23, pos + 2),
        )
        .into_styled(self.fill)
        .draw(&mut self.framebuffer)
        .unwrap();
    }

    pub fn updatedisplay(&mut self){
        self.vfd.clear_frame().unwrap();
        self.drawbackground();
        self.draw_mode();
        self.draw_fanguage();
        self.draw_tempguage();
        self.draw_temps();
        self.vfd.update_frame(self.framebuffer.data()).unwrap();
    }

    pub fn testdisplay(&mut self){

        for i in 0..=36{
            self.tempguage = i;
            self.updatedisplay();
        }

        self.actoggle = true;
        self.updatedisplay();
        self.mode = Mode::Face;
        self.updatedisplay();
        
        for i in 0..=36{
            let range = map_int(i.into(), 0, 36, 36, 0);
            self.tempguage = range.try_into().unwrap();
            self.updatedisplay();
        }

        self.actoggle = false;
        self.updatedisplay();
        self.mode = Mode::Feet;
        self.updatedisplay();

        for i in 60..=99{
            self.internaltemp = i;
            self.updatedisplay();
        }

        self.recirctoggle = true;
        self.updatedisplay();
        self.mode = Mode::FaceFeet;
        self.updatedisplay();

        for i in 60..=99{
            let range = map_int(i.into(), 60, 99, 99, 60);
            self.internaltemp = range.try_into().unwrap();
            self.updatedisplay();
        }

        self.recirctoggle = false;
        self.updatedisplay();
        self.mode = Mode::FeetDef;
        self.updatedisplay();

        for i in -60..=99{
            self.ambtemp = i;
            self.updatedisplay();
        }

        self.mode = Mode::Def;
        self.updatedisplay();

        for i in -60..=99{
            let range = map_int(i.into(), -60, 99, 99, -60);
            self.ambtemp = range.try_into().unwrap();
            self.updatedisplay();
        }

        for i in 0..=32{
            self.fanguage = i;
            self.updatedisplay();
        }

        for i in 32..=0{
            let range = map_int(i.into(), 0, 32, 32, 0);
            self.fanguage = range.try_into().unwrap();
            self.updatedisplay();
        }
    }
}




pub fn map_int(x: i32, in_min: i32, in_max: i32, out_min: i32, out_max: i32) -> i32 {
    (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
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
