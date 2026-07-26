use num_complex::Complex64;

/// Optical signal definition
/// 
/// [Research Prototype]
#[derive(Debug, Clone)]
pub struct OpticalSignal {
    pub samples: Vec<Complex64>,
    pub sample_rate_hz: u64,
    pub wavelength_nm: f64,
    pub power_dbm: f64,
}

impl OpticalSignal {
    pub fn new(samples: Vec<Complex64>, sample_rate_hz: u64, wavelength_nm: f64) -> Self {
        Self { samples, sample_rate_hz, wavelength_nm, power_dbm: 0.0 }
    }

    pub fn power_mw(&self) -> f64 {
        10.0_f64.powf(self.power_dbm / 10.0)
    }

    pub fn set_power_dbm(&mut self, dbm: f64) {
        self.power_dbm = dbm;
    }

    pub fn duration_sec(&self) -> f64 {
        self.samples.len() as f64 / self.sample_rate_hz as f64
    }
}
