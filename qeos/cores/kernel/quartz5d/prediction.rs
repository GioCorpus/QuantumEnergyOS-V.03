use core::fmt;

#[derive(Debug, Clone, Copy)]
pub struct ConfidenceInterval {
    pub lower: f64,
    pub upper: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct Forecast {
    pub predicted_value: f64,
    pub horizon_ns: u64,
    pub confidence: ConfidenceInterval,
    pub model_id: u64,
}

#[derive(Debug, Clone)]
pub struct QuartzPredictor {
    pub model_id: u64,
    pub history: Vec<(u64, f64)>,
}

impl QuartzPredictor {
    pub const fn new(model_id: u64) -> Self {
        Self {
            model_id,
            history: Vec::new(),
        }
    }

    pub fn observe(&mut self, timestamp_ns: u64, value: f64) {
        self.history.push((timestamp_ns, value));
    }

    pub fn predict(&self, horizon_ns: u64) -> Forecast {
        let predicted_value = if self.history.is_empty() {
            0.0
        } else {
            let sum: f64 = self.history.iter().map(|(_, v)| *v).sum();
            sum / self.history.len() as f64
        };
        Forecast {
            predicted_value,
            horizon_ns,
            confidence: ConfidenceInterval {
                lower: predicted_value * 0.95,
                upper: predicted_value * 1.05,
                confidence: 0.95,
            },
            model_id: self.model_id,
        }
    }
}

impl fmt::Display for Forecast {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Forecast(value={:.4}, horizon={}ns, confidence={:.2})",
            self.predicted_value, self.horizon_ns, self.confidence.confidence
        )
    }
}
