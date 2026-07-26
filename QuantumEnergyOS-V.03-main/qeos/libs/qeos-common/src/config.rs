use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use validator::Validate;

use super::error::{QeosError, Result};

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("File not found: {path}")]
    NotFound { path: PathBuf },

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Missing field: {field} in {source}")]
    MissingField { field: String, source: String },
}

pub trait ConfigProvider: Send + Sync {
    fn load(&self) -> Result<DaemonConfig>;
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct QuantumConfig {
    pub enabled: bool,
    pub simulator_backend: SimulatorBackend,
    pub max_qubits: u32,
    pub default_shots: u32,
    pub noise_model: Option<String>,
    pub transpiler_optimization_level: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SimulatorBackend {
    QiskitLocal,
    Qoqo,
    AzureQuantum,
    IBMQuantum,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct EnergyConfig {
    pub enabled: bool,
    pub grid_simulation: bool,
    pub prediction_horizon_hours: u32,
    pub data_source: DataSource,
    pub consumption_model: ModelType,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataSource {
    Simulated,
    LiveAPI,
    HistoricalCSV { path: String },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelType {
    LinearRegression,
    RandomForest,
    NeuralNetwork,
    Transformer,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct SecurityConfig {
    pub tls_enabled: bool,
    pub cert_path: Option<PathBuf>,
    pub key_path: Option<PathBuf>,
    pub api_keys: Vec<String>,
    pub rate_limiting: RateLimitConfig,
    pub audit_logging: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    pub max_requests_per_minute: u32,
    pub burst_size: u32,
}
