use qeos_common::DaemonConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyDaemonConfig {
    pub base: DaemonConfig,
    pub grid_simulation_enabled: bool,
    pub prediction_horizon_hours: u32,
    pub historical_data_path: PathBuf,
    pub model_path: PathBuf,
    pub prediction_model: PredictionModelType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PredictionModelType {
    Linear,
    ARIMA,
    LSTM,
    Transformer,
}

impl Default for EnergyDaemonConfig {
    fn default() -> Self {
        Self {
            base: DaemonConfig {
                name: "energyd".to_string(),
                log_level: "info".to_string(),
                metrics_port: 9092,
                ..Default::default()
            },
            grid_simulation_enabled: true,
            prediction_horizon_hours: 24,
            historical_data_path: PathBuf::from("/var/lib/qeos/energy/data"),
            model_path: PathBuf::from("/usr/share/qeos/models/energy"),
            prediction_model: PredictionModelType::Linear,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_valid_ports() {
        let cfg = EnergyDaemonConfig::default();
        assert!(cfg.base.metrics_port > 0);
    }

    #[test]
    fn prediction_model_serializes() {
        let model = PredictionModelType::Linear;
        let json = serde_json::to_string(&model).unwrap();
        let decoded: PredictionModelType = serde_json::from_str(&json).unwrap();
        matches!(decoded, PredictionModelType::Linear);
    }
}
