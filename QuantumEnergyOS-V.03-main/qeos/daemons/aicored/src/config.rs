use super::AICoreDaemonConfig;
use qeos_common::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AICoreDaemonConfig {
    pub base: DaemonConfig,
    pub model_path: PathBuf,
    pub max_batch_size: usize,
}

impl Default for AICoreDaemonConfig {
    fn default() -> Self {
        Self {
            base: DaemonConfig {
                name: "aicored".to_string(),
                log_level: "info".to_string(),
                metrics_port: 9095,
                ..Default::default()
            },
            model_path: PathBuf::from("/usr/share/qeos/models/ai"),
            max_batch_size: 64,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AICoreDaemon {
    pub config: AICoreDaemonConfig,
}

impl AICoreDaemon {
    pub async fn new(config: AICoreDaemonConfig) -> Result<Self> {
        Ok(Self { config })
    }

    pub async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
}
