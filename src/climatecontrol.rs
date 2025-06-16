pub enum ClimateControlMode {
    Face,
    Feet,
    FaceFeet,
    FeetDef,
    Def,
}

pub struct ClimateControlBacker {
    mode: ClimateControlMode,
    ac_toggle: bool,
    recirc_toggle: bool,
    fan_speed: u8,
    interal_temp: i8,
    set_temp: i8,
}
impl ClimateControlBacker {
    pub fn new() -> Self {
        let mut mode = ClimateControlMode::Def;
        let mut ac_toggle = false;
        let mut recirc_toggle = false;
        let mut fan_speed = 0;
        let mut interal_temp: i8 = 0;
        let mut set_temp: i8 = 0;
        let cc = ClimateControlBacker {
            mode,
            ac_toggle,
            recirc_toggle,
            fan_speed,
            interal_temp,
            set_temp,
        };
        cc
    }

    pub fn get_mode(&self) -> ClimateControlMode {
        let mode = self.mode;
        mode
    }

    pub fn set_mode(&self, newmode: ClimateControlMode) {
        self.mode = newmode;
    }
}
