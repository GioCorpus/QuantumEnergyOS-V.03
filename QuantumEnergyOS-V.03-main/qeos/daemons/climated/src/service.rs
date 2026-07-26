use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Climate prediction and analysis engine
/// 
/// [Research Prototype] - Uses statistical baseline models, ML models planned.
#[derive(Debug, Clone)]
pub struct ClimateEngine {
    pub history: VecDeque<ClimateSnapshot>,
    pub window_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClimateSnapshot {
    pub timestamp: u64,
    pub temperature_c: f64,
    pub humidity_percent: f64,
    pub pressure_hpa: f64,
    pub wind_speed_kmh: f64,
    pub precipitation_mm: f64,
    pub location: (f64, f64), // lat, lon
}

#[derive(Debug, Clone, Serialize)]
pub struct ClimateForecast {
    pub generated_at: u64,
    pub location: (f64, f64),
    pub horizon_hours: u32,
    pub predictions: Vec<ClimateSnapshot>,
    pub confidence_interval_low: Vec<f64>,
    pub confidence_interval_high: Vec<f64>,
    pub model: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExtremeEventAlert {
    pub id: uuid::Uuid,
    pub severity: AlertSeverity,
    pub event_type: ExtremeEventType,
    pub location: (f64, f64),
    pub probability: f64,
    pub description: String,
    pub generated_at: u64,
    pub valid_until: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Info,
    Warning,
    Watch,
    Advisory,
    WarningCritical,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ExtremeEventType {
    HeatWave,
    ColdSnap,
    HeavyRain,
    Drought,
    Hurricane,
    Tornado,
    Flood,
    Wildfire,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_construction() {
        let snap = ClimateSnapshot {
            timestamp: 0,
            temperature_c: 25.0,
            humidity_percent: 50.0,
            pressure_hpa: 1013.0,
            wind_speed_kmh: 10.0,
            precipitation_mm: 0.0,
            location: (0.0, 0.0),
        };
        assert_eq!(snap.temperature_c, 25.0);
    }
}
