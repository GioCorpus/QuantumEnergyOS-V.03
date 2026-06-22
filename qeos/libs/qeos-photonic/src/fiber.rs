use num_complex::Complex;
use std::f64::consts::{E, PI};

/// Fiber channel simulation
/// 
/// [Research Prototype]
#[derive(Debug, Clone)]
pub struct FiberChannel {
    pub length_km: f64,
    pub wavelength_nm: f64,
    pub attenuation_db_per_km: f64,
    pub dispersion_ps_nm_km: f64,
    pub nonlinear_coefficient: f64,
}

impl FiberChannel {
    pub fn new(length_km: f64, wavelength_nm: f64) -> Self {
        Self {
            length_km,
            wavelength_nm,
            attenuation_db_per_km: if wavelength_nm > 1300.0 && wavelength_nm < 1600.0 {
                0.2
            } else {
                1.0
            },
            dispersion_ps_nm_km: 17.0,
            nonlinear_coefficient: 1.2,
        }
    }

    /// Simulate signal propagation through fiber
    pub fn propagate(&self, input: &OpticalSignal) -> OpticalSignal {
        let total_attenuation = self.attenuation_db_per_km * self.length_km;
        let attenuation_linear = 10.0_f64.powf(-total_attenuation / 10.0);
        let output: Vec<Complex<f64>> = input.samples.iter().map(|s| s * attenuation_linear).collect();
        OpticalSignal::new(output, input.sample_rate_hz, input.wavelength_nm)
    }

    /// Calculate dispersion-induced pulse broadening
    pub fn dispersion_broadening(&self, pulse_width_ps: f64) -> f64 {
        (pulse_width_ps.powi(2) + (self.dispersion_ps_nm_km * self.length_km * 0.1).powi(2)).sqrt()
    }

    /// Calculate nonlinear phase shift
    pub fn nonlinear_phase(&self, power_mw: f64) -> f64 {
        self.nonlinear_coefficient * power_mw * self.length_km
    }
}
