use super::QuantumDaemonConfig;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct QuantumBackend {
    pub config: Arc<QuantumDaemonConfig>,
}

impl QuantumBackend {
    pub fn new(config: Arc<QuantumDaemonConfig>) -> Self {
        Self { config }
    }

    pub async fn initialize(&self) -> Result<()> {
        Ok(())
    }

    pub async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }

    pub async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}
