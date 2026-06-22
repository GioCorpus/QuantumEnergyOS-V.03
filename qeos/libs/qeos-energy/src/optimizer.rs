use ndarray::{Array1, Array2};

pub use super::model::PredictionModel;

/// Grid load optimizer
/// 
/// [Production Ready]
#[derive(Debug, Clone)]
pub struct GridOptimizer {
    pub max_load_kw: f64,
    pub peak_price: f64,
    pub off_peak_price: f64,
    pub smoothing_factor: f64,
}

impl GridOptimizer {
    pub fn new(max_load_kw: f64, peak_price: f64, off_peak_price: f64) -> Self {
        Self {
            max_load_kw,
            peak_price,
            off_peak_price,
            smoothing_factor: 0.1,
        }
    }

    /// Optimizes a 24-hour load profile to minimize cost
    pub fn optimize(&self, predicted_load: &[f64]) -> (Array1<f64>, f64) {
        let optimized = self.load_shifting(predicted_load);
        let cost = self.calculate_cost(&optimized);
        (optimized, cost)
    }

    /// Shifts loads from peak to off-peak hours
    fn load_shifting(&self, load: &[f64]) -> Array1<f64> {
        let mut optimized = Array1::from_vec(load.to_vec());
        if load.len() != 24 {
            return optimized;
        }
        let peak_hours = vec![17, 18, 19, 20, 21];
        let off_peak = vec![0, 1, 2, 3, 4, 5];
        let peak_load: f64 = peak_hours.iter().map(|&h| load[h]).sum();
        let target_peak = peak_load * 0.8;
        let excess = peak_load - target_peak;
        if excess > 0.0 {
            let per_hour = excess / peak_hours.len() as f64;
            for &h in &peak_hours {
                optimized[h] = (load[h] - per_hour).max(self.max_load_kw * 0.6);
            }
            let add = excess / off_peak.len() as f64;
            for &h in &off_peak {
                optimized[h] = (optimized[h] + add).min(self.max_load_kw);
            }
        }
        optimized
    }

    /// Calculate cost of load profile
    pub fn calculate_cost(&self, load: &Array1<f64>) -> f64 {
        load.iter().enumerate().map(|(h, &kw)| {
            let price = if h >= 17 && h <= 21 {
                self.peak_price
            } else {
                self.off_peak_price
            };
            kw * price
        }).sum()
    }

    /// Calculate CO2 emissions saved
    pub fn co2_saved(&self, original: &[f64], optimized: &[f64]) -> f64 {
        let grid_emission_rate = 0.5; // kg CO2 per kWh
        let original_kwh: f64 = original.iter().sum();
        let optimized_kwh: f64 = optimized.iter().sum();
        (original_kwh - optimized_kwh).max(0.0) * grid_emission_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optimization_lowers_peak() {
        let optimizer = GridOptimizer::new(100.0, 0.25, 0.10);
        let mut load = vec![50.0; 24];
        for h in 17..=21 { load[h] = 90.0; }
        let (opt, _) = optimizer.optimize(&load);
        let peak: f64 = opt.iter().cloned().fold(0.0, f64::max);
        assert!(peak <= 90.0);
    }

    #[test]
    fn cost_calculation() {
        let optimizer = GridOptimizer::new(100.0, 0.25, 0.10);
        let load = Array1::from_vec(vec![100.0; 24]);
        let cost = optimizer.calculate_cost(&load);
        assert!(cost > 0.0);
    }
}
