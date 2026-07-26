use qeos_common::DaemonConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClimateDaemonConfig {
    pub base: DaemonConfig,
    pub weather_providers: Vec<WeatherProvider>,
    pub prediction_model: ClimatePredictionModel,
    pub data_retention_days: u32,
    pub alert_thresholds: AlertThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum WeatherProvider {
    OpenMeteo { endpoint: String },
    NOAA { api_key: String },
    Simulated { seed: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClimatePredictionModel {
    Statistical,
    RandomForest,
    Transformer { horizon_hours: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub temperature_high_c: f64,
    pub temperature_low_c: f64,
    pub wind_speed_kmh: f64,
    pub precipitation_mm: f64,
    pub air_quality_index_alert: u32,
}

impl Default for ClimateDaemonConfig {
    fn default() -> Self {
        Self {
            base: DaemonConfig {
                name: "climated".to_string(),
                log_level: "info".to_string(),
                metrics_port: 9093,
                ..Default::default()
            },
            weather_providers: vec![WeatherProvider::Simulated { seed: 42 }],
            prediction_model: ClimatePredictionModel::Statistical,
            data_retention_days: 365,
            alert_thresholds: AlertThresholds {
                temperature_high_c: 42.0,
                temperature_low_c: -10.0,
                wind_speed_kmh: 100.0,
                precipitation_mm: 50.0,
                air_quality_index_alert: 150,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let cfg = ClimateDaemonConfig::default();
        assert_eq!(cfg.base.metrics_port, 9093);
    }

    #[test]
    fn alert_thresholds_coherent() {
        let t = AlertThresholds {
            temperature_high_c: 42.0,
            temperature_low_c: -10.0,
            wind_speed_kmh: 100.0,
            precipitation_mm: 50.0,
            air_quality_index_alert: 150,
        };
        assert!(t.temperature_high_c > t.temperature_low_c);
    }
}
