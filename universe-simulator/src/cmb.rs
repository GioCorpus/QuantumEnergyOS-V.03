// cmb.rs
use rand::random;

pub struct CosmicMicrowaveBackground {
    temp_now: f64,              // 2.725 K hoy
    redshift: f64,              // z ~1100 al decoupling
    fluctuation: f64,           // δT/T ~10^{-5}
}

impl CosmicMicrowaveBackground {
    pub fn new() -> Self {
        CosmicMicrowaveBackground {
            temp_now: 2.725,
            redshift: 1100.0,
            fluctuation: 1e-5,
        }
    }

    pub fn get_temp_at_decoupling(&self) -> f64 {
        self.temp_now * (self.redshift + 1.0)
    }

    pub fn map_fluctuations(&self) -> f64 {
        // Simula anisotropías: semillas de inflación
        self.fluctuation * (random::<f64>() - 0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::CosmicMicrowaveBackground;

    #[test]
    fn decoupling_temperature_is_positive() {
        let cmb = CosmicMicrowaveBackground::new();
        assert!(cmb.get_temp_at_decoupling() > 0.0);
    }
}
