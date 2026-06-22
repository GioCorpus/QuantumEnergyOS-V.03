use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryState {
    pub level_pct:     f64,
    pub charging:      bool,
    pub voltage_v:     Option<f64>,
    pub temperature_c: Option<f64>,
    pub health:        String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbientData {
    pub temperature_c: Option<f64>,
    pub humidity_pct:  Option<f64>,
    pub pressure_hpa:  Option<f64>,
    pub light_lux:     Option<f64>,
    pub acceleration:  Option<[f32; 3]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolarForecast {
    pub location:         String,
    pub risk_level:       String,
    pub kp_index:         f64,
    pub alert_message:    String,
    pub grid_impact_pct:  f64,
    pub recommendation:   String,
    pub next_event_hours: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridBalanceResult {
    pub n_nodes:         usize,
    pub energy_saved_kw: f64,
    pub optimal_config:  Vec<f64>,
    pub quartz_hash:     String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridHistoryEntry {
    pub id:           i64,
    pub timestamp:    String,
    pub n_nodes:      usize,
    pub energy_saved: f64,
    pub config:       String,
}
