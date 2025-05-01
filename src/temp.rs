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

    sensor: Channel<'a>,

    adc: Adc<'a, Blocking>,
}

impl <'a>Thermistor<'a> {
    pub fn new(sensor: Channel<'a>, adc: Adc<'a, Blocking>) -> Self {
        let pullup = 47_000.0;
        let r0 = 100_000.0;
        let t0_k = celsius_to_kelvin(25.0); // celcius to kelvin
        let beta = 3950.0;
        let adc_max = 4095.0;

        Thermistor {
            pullup,
            r0,
            t0_k,
            beta,
            adc_max,
            sensor,
            adc,
        }
    }

    pub fn measure_adc(&mut self) -> f64{
        let mut samples: [u16; 128] = [0u16; 128];
        for sample in &mut samples{
            let _ = self.adc.blocking_read(& mut self.sensor);
            *sample = self.adc.blocking_read(& mut self.sensor).unwrap();
        }
        let average: f64 = average(&samples).into();
        average
    }

    pub fn measure_temp(&mut self) -> f64{
        let adc = self.measure_adc();
        let res = self.adc_to_resistance(adc);
        let temp = self.calculate_temperature(res);
        let tempc = kelvin_to_celsius(temp);
        //let tempf = celcius_to_fahrenheit(tempc);
        tempc
    }

    /// Convert a raw 16-bit ADC reading → thermistor resistance (Ω)
    fn adc_to_resistance(&self, adc: f64) -> f64 {
        let x: f64 = self.pullup * ((self.adc_max/adc as f64) - 1.0);
        x
    }

    fn calculate_temperature(&mut self, current_res: f64) -> f64 {
        let ln_value = log(current_res / self.r0); 
        let inv_t = (1.0 / self.t0_k) + ((1.0 / self.beta) * ln_value);
        1.0 / inv_t
    }
}