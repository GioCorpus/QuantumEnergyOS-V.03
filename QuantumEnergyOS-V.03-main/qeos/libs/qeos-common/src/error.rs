use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum QeosError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Quantum computation error: {0}")]
    Quantum(String),

    #[error("Energy system error: {0}")]
    Energy(String),

    #[error("Climate system error: {0}")]
    Climate(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, QeosError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    pub name: String,
    pub version: String,
    pub log_level: String,
    pub data_dir: PathBuf,
    pub bind_address: String,
    pub metrics_port: u16,
    pub database_url: Option<String>,
    pub redis_url: Option<String>,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            name: "unknown".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            log_level: "info".to_string(),
            data_dir: PathBuf::from("/var/lib/qeos"),
            bind_address: "127.0.0.1".to_string(),
            metrics_port: 9090,
            database_url: None,
            redis_url: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub component: String,
    pub details: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub hostname: String,
    pub os: String,
    pub kernel_version: String,
    pub cpu_count: usize,
    pub memory_total: u64,
    pub cpu_info: CpuInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub model: String,
    pub cores: usize,
    pub threads: usize,
    pub frequency_mhz: u64,
    pub supports_avx: bool,
    pub supports_sse: bool,
}
