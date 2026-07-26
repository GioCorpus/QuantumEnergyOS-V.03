// ═══════════════════════════════════════════════════════════════════════════════
//  hw/sysfs — Linux sysfs power supply sensor backend
// ═══════════════════════════════════════════════════════════════════════════════
//!
//! [Research Prototype]
//!
//! Reads battery/grid metrics from /sys/class/power_supply/* when available.
//! Suitable for development on laptops and embedded Linux targets.

use super::{IngestResult, SensorBackend};
use crate::{
    config::SensorType,
    frame::{EnergyTelemetryFrame, flags},
    hw::{encode_sensor_id, finalize_frame},
    platform::timestamp_ns,
};
use std::fs;
use std::path::{Path, PathBuf};

/// Sysfs-backed power supply sensor
pub struct SysfsPowerSensor {
    sensor_id: u32,
    supply_path: PathBuf,
    last: EnergyTelemetryFrame,
    emergency: bool,
}

impl SysfsPowerSensor {
    /// Probe the first available power supply under /sys/class/power_supply
    pub fn probe_first() -> Option<Self> {
        let base = Path::new("/sys/class/power_supply");
        let entry = fs::read_dir(base).ok()?.flatten().next()?;
        let path = entry.path();
        Some(Self {
            sensor_id: encode_sensor_id(SensorType::PMIC, 1),
            supply_path: path,
            last: EnergyTelemetryFrame::new(),
            emergency: false,
        })
    }

    pub fn from_path(path: PathBuf, local_id: u32) -> Self {
        Self {
            sensor_id: encode_sensor_id(SensorType::PMIC, local_id),
            supply_path: path,
            last: EnergyTelemetryFrame::new(),
            emergency: false,
        }
    }

    fn read_u32(&self, file: &str) -> Option<u32> {
        let content = fs::read_to_string(self.supply_path.join(file)).ok()?;
        content.trim().parse().ok()
    }
}

impl SensorBackend for SysfsPowerSensor {
    fn sensor_id(&self) -> u32 {
        self.sensor_id
    }

    fn sensor_type(&self) -> SensorType {
        SensorType::PMIC
    }

    fn poll_frame(&mut self) -> IngestResult {
        let voltage_uv = self.read_u32("voltage_now").unwrap_or(0);
        let current_ua = self.read_u32("current_now").unwrap_or(0);

        if voltage_uv == 0 && current_ua == 0 {
            return IngestResult::WouldBlock;
        }

        let mut frame = EnergyTelemetryFrame::from_parts(
            timestamp_ns(),
            self.sensor_id,
            (voltage_uv / 1000) as u32,
            (current_ua / 1000).unsigned_abs() as u32,
            5000,
            flags::FLAG_IRQ_DRIVEN,
        );
        frame = finalize_frame(frame);
        self.last = frame;
        IngestResult::Ok
    }

    fn last_frame(&self) -> Option<&EnergyTelemetryFrame> {
        Some(&self.last)
    }

    fn set_emergency_mode(&mut self, _enabled: bool) {
        self.emergency = _enabled;
    }
}
