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
    displaymode: bool,
}

#[allow(unused)]
impl ClimateControlBacker {
    pub fn new() -> Self {
        let mut mode = ClimateControlMode::Def;
        let mut ac_toggle = false;
        let mut recirc_toggle = false;
        let mut fan_speed = 0;
        let mut ambient_temp: i8 = 50;
        let mut set_temp: i8 = 50;
        let mut displaymode: bool = false;
        let cc = ClimateControlBacker {
            mode,
            ac_toggle,
            recirc_toggle,
            fan_speed,
            ambient_temp,
            set_temp,
            displaymode,
        };
        cc
    }

    pub fn mode(&self) -> &ClimateControlMode {
        &self.mode
    }

    pub fn next_mode(&mut self) {
        match self.mode{
            ClimateControlMode::Face => self.set_mode(ClimateControlMode::Feet),
            ClimateControlMode::Feet => self.set_mode(ClimateControlMode::FaceFeet),
            ClimateControlMode::FaceFeet => self.set_mode(ClimateControlMode::FeetDef),
            ClimateControlMode::FeetDef => self.set_mode(ClimateControlMode::Def),
            ClimateControlMode::Def => self.set_mode(ClimateControlMode::Face),
        }
    }

    pub fn set_mode(&mut self, mode: ClimateControlMode) {
        self.mode = mode;
    }

    pub fn ac_toggle(&self) -> bool {
        self.ac_toggle
    }

    pub fn set_ac_toggle(&mut self) {
        self.ac_toggle = !self.ac_toggle;
    }

    pub fn recirc_toggle(&self) -> bool {
        self.recirc_toggle
    }

    pub fn set_recirc_toggle(&mut self) {
        self.recirc_toggle = !self.recirc_toggle;
    }

    pub fn fan_speed(&mut self) -> u8 {
        self.fan_speed
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
        if set_temp >= 90{
            self.set_temp = 90
        }else if set_temp <= 60 {
            self.set_temp = 60
        }else{
        self.set_temp = set_temp;}
    }

    pub fn displaymode(&self) -> bool {
        self.displaymode
    }

    pub fn set_displaymode(&mut self, displaymode: bool) {
        self.displaymode = displaymode;
    }
}
