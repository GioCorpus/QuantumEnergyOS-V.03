// ═══════════════════════════════════════════════════════════════════════
//  photonic_pqa.rs — Analizador Cuántico-Fotónico de Calidad de Energía
//  QuantumEnergyOS V.02 — Módulo de Sensor de Bajo Nivel
// ═══════════════════════════════════════════════════════════════════════

use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

/// Estructura de métricas de calidad de energía
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerQualityData {
    pub timestamp: u64,
    pub voltage_v: f64,
    pub current_a: f64,
    pub frequency_hz: f64,
    pub thd_pct: f64, // Total Harmonic Distortion
    pub power_factor: f64,
    pub status: String,
}

/// Trait común para sensores de energía
pub trait PowerQualitySensor {
    fn read(&mut self) -> Result<PowerQualityData, String>;
    fn calibrate(&mut self, factor: f64);
    fn get_status(&self) -> String;
}

pub struct PqaDriver {
    simulation_mode: bool,
    last_read: PowerQualityData,
    calibration_factor: f64,
    // En hardware real, aquí iría el bus GPIO o SPI
    // #[cfg(target_arch = "arm")]
    // gpio_pin: Option<rppal::gpio::InputPin>,
}

impl PqaDriver {
    pub fn new(simulation: bool) -> Self {
        Self {
            simulation_mode: simulation,
            last_read: PowerQualityData::default(),
            calibration_factor: 1.0,
        }
    }

    /// Genera lecturas realistas para la red de Mexicali (60Hz + calor extremo)
    fn simulate_mexicali_grid(&self) -> PowerQualityData {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        // Simulación de fluctuaciones térmicas (Mexicali > 45°C)
        // El calor aumenta la resistencia y el ruido en la red
        let thermal_noise = (now % 3600) as f64 / 3600.0 * 0.5;
        
        // 60Hz nominal con jitter cuántico simulado
        let freq = 60.0 + (rand_pseudo(now) * 0.1) - 0.05;
        let volt = 127.0 + (rand_pseudo(now + 1) * 5.0) - 2.5;
        let thd = 1.2 + thermal_noise + (rand_pseudo(now + 2) * 2.0);

        PowerQualityData {
            timestamp: now,
            voltage_v: volt * self.calibration_factor,
            current_a: 15.4 + (rand_pseudo(now + 3) * 2.0),
            frequency_hz: freq,
            thd_pct: thd,
            power_factor: 0.95 - (thermal_noise * 0.1),
            status: if thd > 5.0 { "WARNING: High THD".into() } else { "OK".into() },
        }
    }
}

impl PowerQualitySensor for PqaDriver {
    fn read(&mut self) -> Result<PowerQualityData, String> {
        if self.simulation_mode {
            self.last_read = self.simulate_mexicali_grid();
            Ok(self.last_read.clone())
        } else {
            // Lógica para hardware GPIO real (Raspberry Pi / Industrial PC)
            // Aquí se usaría rppal::gpio para contar pulsos de cruce por cero
            // o leer de un ADC vía SPI.
            #[cfg(feature = "hardware_gpio")]
            {
                // Implementación simplificada de lectura GPIO
                // let level = self.gpio_pin.as_ref().unwrap().read();
                Err("Hardware GPIO not fully implemented in this demo".into())
            }
            #[cfg(not(feature = "hardware_gpio"))]
            {
                Err("Hardware support not compiled. Use simulation_mode=true".into())
            }
        }
    }

    fn calibrate(&mut self, factor: f64) {
        self.calibration_factor = factor;
        println!("[PQA] Calibración ajustada a: {}", factor);
    }

    fn get_status(&self) -> String {
        format!("Mode: {}, Last THD: {:.2}%", 
            if self.simulation_mode { "SIM" } else { "HW" },
            self.last_read.thd_pct
        )
    }
}

impl Default for PowerQualityData {
    fn default() -> Self {
        Self {
            timestamp: 0,
            voltage_v: 0.0,
            current_a: 0.0,
            frequency_hz: 60.0,
            thd_pct: 0.0,
            power_factor: 1.0,
            status: "Initializing".into(),
        }
    }
}

// Helper para evitar dependencias externas en la lógica de simulación
fn rand_pseudo(seed: u64) -> f64 {
    let x = (seed as f64).sin() * 10000.0;
    x - x.floor()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_values() {
        let mut driver = PqaDriver::new(true);
        let data = driver.read().unwrap();
        assert!(data.voltage_v > 100.0 && data.voltage_v < 150.0);
        assert!(data.frequency_hz > 59.0 && data.frequency_hz < 61.0);
    }

    #[test]
    fn test_calibration() {
        let mut driver = PqaDriver::new(true);
        driver.calibrate(2.0);
        let data = driver.read().unwrap();
        // El valor base simulado es ~127, con factor 2.0 debe ser > 200
        assert!(data.voltage_v > 200.0);
    }
}
