// ═══════════════════════════════════════════════════════════════════════════════
//  hw/simulated — Simulated sensor backend for grid instability testing
// ═══════════════════════════════════════════════════════════════════════════════
//!
//! [Research Prototype]

use super::{IngestResult, SensorBackend};
use crate::{
    ai_simulation::GridSimulator,
    config::SensorType,
    frame::EnergyTelemetryFrame,
    hw::finalize_frame,
};

/// Simulated energy sensor with grid dynamics
pub struct SimulatedSensor {
    sensor_id: u32,
    sensor_type: SensorType,
    grid: GridSimulator,
    last: EnergyTelemetryFrame,
    emergency: bool,
    tick: u64,
}

impl SimulatedSensor {
    pub fn new(sensor_id: u32, sensor_type: SensorType) -> Self {
        Self {
            sensor_id,
            sensor_type,
            grid: GridSimulator::new(),
            last: EnergyTelemetryFrame::new(),
            emergency: false,
            tick: 0,
        }
    }

    pub fn with_grid_instability(mut self, sag_pct: f32, duration: f32) -> Self {
        self.grid.voltage_sag(sag_pct, duration);
        self
    }
}

impl SensorBackend for SimulatedSensor {
    fn sensor_id(&self) -> u32 {
        self.sensor_id
    }

    fn sensor_type(&self) -> SensorType {
        self.sensor_type
    }

    fn poll_frame(&mut self) -> IngestResult {
        self.tick += 1;
        let frames = self.grid.step();
        let mut frame = frames.first().copied().unwrap_or_default();
        frame.sensor_id = self.sensor_id;
        frame = finalize_frame(frame);
        self.last = frame;
        IngestResult::Ok
    }

    fn last_frame(&self) -> Option<&EnergyTelemetryFrame> {
        Some(&self.last)
    }

    fn set_emergency_mode(&mut self, enabled: bool) {
        self.emergency = enabled;
    }
}
