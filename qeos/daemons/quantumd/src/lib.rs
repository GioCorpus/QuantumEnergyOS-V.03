mod config;
mod metrics;
mod service;
mod traits;

pub use config::QuantumDaemonConfig;
pub use service::QuantumDaemon;
pub use traits::QuantumBackend;

pub fn config() -> Result<QuantumDaemonConfig> {
    Ok(QuantumDaemonConfig::default())
}
