use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use qeos_common::{QeosError, Result};
use serde::Serialize;

/// Energy calculation and analysis engine
#[derive(Debug, Clone)]
pub struct EnergyEngine {
    pub baseline_kw: f64,
    pub peak_kw: f64,
    pub efficiency_factor: f64,
}

impl EnergyEngine {
    pub fn new(baseline_kw: f64, peak_kw: f64, efficiency_factor: f64) -> Self {
        Self {
            baseline_kw,
            peak_kw,
            efficiency_factor,
        }
    }

    pub fn current_load(&self, hour_of_day: u32) -> f64 {
        let peak_hour = 18;
        let distance = (hour_of_day as i32 - peak_hour as i32).abs();
        let factor = 1.0 - (distance as f64 / 12.0) * self.efficiency_factor;
        self.peak_kw * factor.max(self.baseline_kw / self.peak_kw)
    }

    pub fn optimize_load_profile(&self, hours: &[f64]) -> Vec<f64> {
        hours.iter().map(|&h| self.current_load(h as u32)).collect()
    }

    pub fn calculate_co2_saved(&self, optimized_kwh: f64, baseline_kwh: f64) -> f64 {
        let grid_emission_factor_g_per_kwh = 450.0;
        let saved_kwh = baseline_kwh - optimized_kwh;
        saved_kwh * grid_emission_factor_g_per_kwh / 1000.0
    }
}

#[derive(Debug, Clone)]
pub struct EnergyReading {
    pub timestamp: u64,
    pub source: String,
    pub power_kw: f64,
    pub energy_kwh: f64,
    pub voltage_v: f64,
    pub current_a: f64,
    pub frequency_hz: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct OptimizationResult {
    pub original_cost_usd: f64,
    pub optimized_cost_usd: f64,
    pub savings_percent: f64,
    pub co2_saved_kg: f64,
    pub recommendations: Vec<OptimizationRecommendation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OptimizationRecommendation {
    pub action: String,
    pub expected_savings: f64,
    pub priority: Priority,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

/// Energy daemon service wrapper
pub struct EnergyDaemon {
    pub engine: EnergyEngine,
}

impl EnergyDaemon {
    pub fn new(engine: EnergyEngine) -> Self {
        Self { engine }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peak_hour_has_higher_load() {
        let eng = EnergyEngine::new(10.0, 100.0, 0.5);
        assert!(eng.current_load(18) > eng.current_load(3));
    }

    #[test]
    fn co2_saved_is_positive() {
        let eng = EnergyEngine::new(10.0, 100.0, 0.5);
        let saved = eng.calculate_co2_saved(40.0, 50.0);
        assert!(saved > 0.0);
    }
}
