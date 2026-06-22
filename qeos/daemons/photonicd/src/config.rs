use qeos_common::DaemonConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotonicDaemonConfig {
    pub base: DaemonConfig,
    pub wavelength_nm: f64,
    pub modulation: PhotonicModulation,
    pub channel_count: u32,
    pub enable_hardware_acceleration: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PhotonicModulation {
    OOK,
    PAM4,
    QPSK,
    QAM16,
    QAM64,
}

impl Default for PhotonicDaemonConfig {
    fn default() -> Self {
        Self {
            base: DaemonConfig {
                name: "photonicd".to_string(),
                log_level: "info".to_string(),
                metrics_port: 9094,
                ..Default::default()
            },
            wavelength_nm: 1550.0,
            modulation: PhotonicModulation::QPSK,
            channel_count: 128,
            enable_hardware_acceleration: false,
        }
    }
}
