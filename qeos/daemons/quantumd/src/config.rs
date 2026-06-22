use qeos_common::{DaemonConfig, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumDaemonConfig {
    pub base: DaemonConfig,
    pub max_qubits: u32,
    pub default_shots: u32,
    pub simulator_backend: SimulatorBackend,
    pub circuit_cache_size: usize,
    pub transpiler_optimization_level: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SimulatorBackend {
    Local,
    QiskitLocal,
    Qoqo,
    AzureQuantum { resource_id: String },
    IBMQuantum { token: String, hub: String, group: String, project: String },
}

impl Default for QuantumDaemonConfig {
    fn default() -> Self {
        Self {
            base: DaemonConfig {
                name: "quantumd".to_string(),
                log_level: "info".to_string(),
                metrics_port: 9091,
                ..Default::default()
            },
            max_qubits: 32,
            default_shots: 1024,
            simulator_backend: SimulatorBackend::Local,
            circuit_cache_size: 1024,
            transpiler_optimization_level: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let cfg = QuantumDaemonConfig::default();
        assert_eq!(cfg.max_qubits, 32);
        assert_eq!(cfg.default_shots, 1024);
    }

    #[test]
    fn config_serialization_roundtrip() {
        let cfg = QuantumDaemonConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let decoded: QuantumDaemonConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg.max_qubits, decoded.max_qubits);
    }
}
