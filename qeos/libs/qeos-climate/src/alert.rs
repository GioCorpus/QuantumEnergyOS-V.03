pub use super::climate::ClimateEngine;

/// Extreme event detection system
/// 
/// [Research Prototype]
#[derive(Debug, Clone)]
pub struct ExtremeEventDetector {
    thresholds: AlertThresholds,
}

#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub heat_wave_c: f64,
    pub cold_snap_c: f64,
    pub heavy_rain_mm: f64,
    pub drought_days: u32,
    pub wind_gust_kmh: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            heat_wave_c: 40.0,
            cold_snap_c: -10.0,
            heavy_rain_mm: 50.0,
            drought_days: 14,
            wind_gust_kmh: 100.0,
        }
    }
}

impl ExtremeEventDetector {
    pub fn new(thresholds: AlertThresholds) -> Self {
        Self { thresholds }
    }

    pub fn detect(&self, reading: &ClimateReading) -> Option<ExtremeEventType> {
        if reading.temperature_c >= self.thresholds.heat_wave_c {
            return Some(ExtremeEventType::HeatWave);
        }
        if reading.temperature_c <= self.thresholds.cold_snap_c {
            return Some(ExtremeEventType::ColdSnap);
        }
        if reading.precipitation_mm >= self.thresholds.heavy_rain_mm {
            return Some(ExtremeEventType::HeavyRain);
        }
        if reading.wind_speed_kmh >= self.thresholds.wind_gust_kmh {
            return Some(ExtremeEventType::SevereWind);
        }
        None
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ExtremeEventType {
    HeatWave,
    ColdSnap,
    HeavyRain,
    SevereWind,
    Drought,
    Wildfire,
    Flood,
    Hurricane,
}

impl std::fmt::Display for ExtremeEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HeatWave => write!(f, "heat_wave"),
            Self::ColdSnap => write!(f, "cold_snap"),
            Self::HeavyRain => write!(f, "heavy_rain"),
            Self::SevereWind => write!(f, "severe_wind"),
            Self::Drought => write!(f, "drought"),
            Self::Wildfire => write!(f, "wildfire"),
            Self::Flood => write!(f, "flood"),
            Self::Hurricane => write!(f, "hurricane"),
        }
    }
}
