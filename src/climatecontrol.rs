#[derive(Default)]
pub enum ClimateControlMode {
    Face,
    Feet,
    FaceFeet,
    FeetDef,
    #[default]
    Def,
}
#[derive(Default)]
pub struct ClimateControlBacker {
    mode: ClimateControlMode,
    ac_toggle: bool,
    recirc_toggle: bool,
    fan_speed: u8,
    ambient_temp: i8,
    set_temp: i8,
}

#[allow(unused)]
impl ClimateControlBacker {
    pub fn new() -> Self {
        let mut mode = ClimateControlMode::Def;
        let mut ac_toggle = false;
        let mut recirc_toggle = false;
        let mut fan_speed = 0;
        let mut ambient_temp: i8 = 0;
        let mut set_temp: i8 = 0;
        let cc = ClimateControlBacker {
            mode,
            ac_toggle,
            recirc_toggle,
            fan_speed,
            ambient_temp,
            set_temp,
        };
        cc
    }

    pub fn mode(&self) -> &ClimateControlMode {
        &self.mode
    }

    pub fn set_mode(&mut self, mode: ClimateControlMode) {
        self.mode = mode;
    }

    pub fn ac_toggle(&self) -> bool {
        self.ac_toggle
    }

    pub fn set_ac_toggle(&mut self, ac_toggle: bool) {
        self.ac_toggle = ac_toggle;
    }

    pub fn recirc_toggle(&self) -> bool {
        self.recirc_toggle
    }

    pub fn set_fan_speed(&mut self, fan_speed: u8) {
        self.fan_speed = fan_speed;
    }

    pub fn ambient_temp(&self) -> i8 {
        self.ambient_temp
    }

    pub fn set_ambient_temp(&mut self, interal_temp: i8) {
        self.ambient_temp = interal_temp;
    }

    pub fn set_temp(&self) -> i8 {
        self.set_temp
    }

    pub fn set_set_temp(&mut self, set_temp: i8) {
        self.set_temp = set_temp;
    }
}
