use ndarray::Array1;
use num_complex::Complex;
use std::f64::consts::PI;

/// Photonic signal processing engine
/// 
/// [Research Prototype] - Optical simulation, hardware support future.
#[derive(Debug, Clone)]
pub struct PhotonicEngine {
    pub wavelength_nm: f64,
    pub sample_rate_hz: u64,
    pub fft_size: usize,
}

impl PhotonicEngine {
    pub fn new(wavelength_nm: f64, sample_rate_hz: u64) -> Self {
        Self {
            wavelength_nm,
            sample_rate_hz,
            fft_size: 1024,
        }
    }

    pub fn wavelength_m(&self) -> f64 {
        self.wavelength_nm / 1e9
    }

    pub fn frequency_hz(&self) -> f64 {
        299_792_458.0 / self.wavelength_m()
    }

    pub fn generate_optical_carrier(&self, duration_sec: f64) -> Vec<Complex<f64>> {
        let samples = (self.sample_rate_hz as f64 * duration_sec) as usize;
        let freq = self.frequency_hz();
        (0..samples)
            .map(|i| {
                let t = i as f64 / self.sample_rate_hz as f64;
                let phase = 2.0 * PI * freq * t;
                Complex::new(phase.cos(), phase.sin())
            })
            .collect()
    }

    pub fn apply_modulation(&self, signal: &[Complex<f64>], data: &[u8]) -> Vec<Complex<f64>> {
        signal
            .iter()
            .zip(data.iter().cycle())
            .map(|(carrier, &bit)| {
                let amplitude = if bit == 1 { 1.0 } else { -1.0 };
                carrier * amplitude
            })
            .collect()
    }

    pub fn simulate_fiber_propagation(&self, signal: &[Complex<f64>], distance_km: f64) -> Vec<Complex<f64>> {
        let attenuation_db_per_km = 0.2;
        let total_attenuation = attenuation_db_per_km * distance_km;
        let attenuation_linear = 10.0_f64.powf(-total_attenuation / 10.0);

        signal.iter().map(|s| s * attenuation_linear).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wavelength_conversion() {
        let eng = PhotonicEngine::new(1550.0, 1_000_000);
        assert!(eng.wavelength_m() > 1.0e-6 && eng.wavelength_m() < 2.0e-6);
    }

    #[test]
    fn frequency_in_optical_range() {
        let eng = PhotonicEngine::new(1550.0, 1_000_000);
        let f = eng.frequency_hz();
        assert!(f > 1.0e14 && f < 3.0e14);
    }
}
