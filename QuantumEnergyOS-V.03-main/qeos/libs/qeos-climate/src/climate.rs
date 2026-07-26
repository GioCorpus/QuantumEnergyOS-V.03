use ndarray::Array1;
use std::collections::VecDeque;

/// Climate prediction and analysis engine
/// 
/// [Research Prototype]
pub struct ClimateEngine {
    history: VecDeque<ClimateReading>,
    window_size: usize,
}

#[derive(Clone)]
pub struct ClimateReading {
    pub timestamp: u64,
    pub temperature_c: f64,
    pub humidity_percent: f64,
    pub pressure_hpa: f64,
    pub wind_speed_kmh: f64,
    pub precipitation_mm: f64,
}

impl ClimateEngine {
    pub fn new(window: usize) -> Self {
        Self { history: VecDeque::with_capacity(window), window_size: window }
    }

    pub fn add_reading(&mut self, reading: ClimateReading) {
        if self.history.len() >= self.window_size {
            self.history.pop_front();
        }
        self.history.push_back(reading);
    }

    pub fn moving_average(&self, hours: usize) -> Vec<f64> {
        if self.history.is_empty() {
            return vec![];
        }
        let vals: Vec<f64> = self.history.iter().map(|r| r.temperature_c).collect();
        let mut result = Vec::with_capacity(vals.len());
        for i in 0..vals.len() {
            let start = i.saturating_sub(hours - 1);
            let window = vals.get(start..=i).unwrap_or(&vals);
            result.push(window.iter().sum::<f64>() / window.len() as f64);
        }
        result
    }

    pub fn detect_trend(&self) -> TrendDirection {
        if self.history.len() < 2 {
            return TrendDirection::Stable;
        }
        let recent: Vec<f64> = self.history.iter().rev().take(24).map(|r| r.temperature_c).collect();
        let sum_first: f64 = recent[..recent.len()/2].iter().sum();
        let sum_second: f64 = recent[recent.len()/2..].iter().sum();
        if sum_second > sum_first * 1.05 {
            TrendDirection::Warming
        } else if sum_second < sum_first * 0.95 {
            TrendDirection::Cooling
        } else {
            TrendDirection::Stable
        }
    }
}

use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum TrendDirection {
    Warming,
    Cooling,
    Stable,
}

impl fmt::Display for TrendDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Warming => write!(f, "warming"),
            Self::Cooling => write!(f, "cooling"),
            Self::Stable => write!(f, "stable"),
        }
    }
}
