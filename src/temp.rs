use libm::log;

use embassy_rp::adc::{Adc, Blocking, Channel};

pub fn average(numbers: &[u16]) -> u16 {
    let len = numbers.len();
    if len == 0 {
        return 0;
    }
    let sum: u32 = numbers.iter().map(|&x| x as u32).sum();
    let count = len as u32;

    let avg = (sum + (count / 2)) / count;

    avg.min(u16::MAX as u32) as u16
}

pub fn kelvin_to_celsius(kelvin: f64) -> f64 {
    kelvin -  273.15
}

pub fn celsius_to_kelvin(celsius: f64) -> f64 {
    celsius + 273.15
}

pub fn celcius_to_fahrenheit(celsius: f64) -> f64 {
    celsius * 1.8 + 32.0
}

fn onboard_temp_calculate(adcin: f64) -> f64{
        // According to chapter 12.4.6 Temperature Sensor in RP235x datasheet
        let temp = 27.0 - (adcin * 3.3 / 4096.0 - 0.706) / 0.001721;
        let sign = if temp < 0.0 { -1.0 } else { 1.0 };
        let rounded_temp_x10 = (temp * 10.0) + 0.5 * sign;
        (rounded_temp_x10) / 10.0
}

pub struct Thermistor<'a>{
    /// pull-up resistor in the divider (Ω)
    pullup: f64,
    /// nominal R₀ @ T₀ (Ω)
    r0: f64,
    /// T₀ in K (25 °C → 298.15 K)
    t0_k: f64,
    /// β constant (K)
    beta: f64,

    adc_max: f64,

    sensor1: Channel<'a>,

    sensor2: Channel<'a>,

    adc: Adc<'a, Blocking>,
}

impl <'a>Thermistor<'a> {
    pub fn new(sensor1: Channel<'a>, sensor2: Channel<'a>, adc: Adc<'a, Blocking>) -> Self {
        let pullup = 10_000.0;
        let r0 = 10_000.0;
        let t0_k = celsius_to_kelvin(25.0); // celcius to kelvin
        let beta = 3950.0;
        let adc_max = 4095.0;

        Thermistor {
            pullup,
            r0,
            t0_k,
            beta,
            adc_max,
            sensor1,
            sensor2,
            adc,
        }
    }

    pub fn measure_adcs(&mut self) -> [f64; 2]{
        let mut samples1: [u16; 128] = [0u16; 128];
        for sample in &mut samples1{
            *sample = self.adc.blocking_read(& mut self.sensor1).unwrap();
        }
        let average1: f64 = average(&samples1).into();
        let mut samples2: [u16; 128] = [0u16; 128];
        for sample in &mut samples2{
            *sample = self.adc.blocking_read(& mut self.sensor2).unwrap();
        }
        let average2: f64 = average(&samples2).into();
        [average1,average2]
    }

    pub fn measure_temp(&mut self) -> [f64; 2]{
        let adc = self.measure_adcs();
        let res1 = self.adc_to_resistance(adc[0]);
        let temp1 = self.calculate_temperature(res1);
        let tempc1 = kelvin_to_celsius(temp1);
        let tempf1 = celcius_to_fahrenheit(tempc1);
        let res2 = onboard_temp_calculate(adc[1]);
        let tempf2 = celcius_to_fahrenheit(res2);
        [tempf1, tempf2]
    }



    /// Convert a raw 16-bit ADC reading → thermistor resistance (Ω)
    fn adc_to_resistance(&self, adc: f64) -> f64 {
        let x: f64 = self.pullup / ((self.adc_max/adc as f64) - 1.0);
        x
    }

    fn calculate_temperature(&mut self, current_res: f64) -> f64 {
        let ln_value = log(current_res / self.r0); 
        let inv_t = (1.0 / self.t0_k) + ((1.0 / self.beta) * ln_value);
        1.0 / inv_t
    }
}