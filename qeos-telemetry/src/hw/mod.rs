// ═══════════════════════════════════════════════════════════════════════════════
//  hw — Hardware abstraction layer for sensor backends
// ═══════════════════════════════════════════════════════════════════════════════
//!
//! [Production Kernel Component] — trait and registry
//! [Research Prototype] — simulated and sysfs backends for development
//!
//! Supports I2C, SPI, PCIe, PMIC, and FPGA sensor classes via a unified
//! ingest interface. Production deployments wire real kernel drivers; this
//! module provides userspace simulation and Linux sysfs fallbacks.

#[cfg(feature = "std")]
pub mod simulated;
#[cfg(feature = "std")]
pub mod sysfs;

use crate::{
    config::SensorType,
    frame::{EnergyTelemetryFrame, flags},
    platform::timestamp_ns,
};

/// Result of a hardware ingest operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IngestResult {
    /// Frame produced successfully
    Ok,
    /// No data available (non-blocking poll)
    WouldBlock,
    /// Hardware error
    Error,
}

/// Hardware sensor backend trait
pub trait SensorBackend: Send {
    /// Unique sensor identifier (matches frame.sensor_id encoding)
    fn sensor_id(&self) -> u32;

    /// Sensor transport class
    fn sensor_type(&self) -> SensorType;

    /// Poll for a single frame (non-blocking)
    fn poll_frame(&mut self) -> IngestResult;

    /// Read the last polled frame (zero-copy within backend buffer)
    fn last_frame(&self) -> Option<&EnergyTelemetryFrame>;

    /// Estimated sampling rate in Hz
    fn sample_rate_hz(&self) -> u32 {
        self.sensor_type().typical_rate_hz()
    }

    /// Enter emergency polling mode (disable interrupt-driven path)
    fn set_emergency_mode(&mut self, enabled: bool);
}

/// Registry of active sensor backends
#[cfg(feature = "std")]
pub struct SensorRegistry {
    backends: Vec<Box<dyn SensorBackend>>,
}

#[cfg(feature = "std")]
impl SensorRegistry {
    pub fn new() -> Self {
        Self {
            backends: Vec::new(),
        }
    }

    pub fn register(&mut self, backend: Box<dyn SensorBackend>) {
        self.backends.push(backend);
    }

    pub fn poll_all(&mut self) -> Vec<EnergyTelemetryFrame> {
        let mut frames = Vec::new();
        for backend in self.backends.iter_mut() {
            if backend.poll_frame() == IngestResult::Ok {
                if let Some(frame) = backend.last_frame() {
                    frames.push(*frame);
                }
            }
        }
        frames
    }

    pub fn set_emergency_mode(&mut self, enabled: bool) {
        for backend in self.backends.iter_mut() {
            backend.set_emergency_mode(enabled);
        }
    }

    pub fn len(&self) -> usize {
        self.backends.len()
    }

    pub fn is_empty(&self) -> bool {
        self.backends.is_empty()
    }
}

#[cfg(feature = "std")]
impl Default for SensorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Build a default registry with simulated grid sensors
#[cfg(feature = "std")]
pub fn default_simulated_registry(node_count: usize) -> SensorRegistry {
    let mut reg = SensorRegistry::new();
    for i in 0..node_count {
        let sensor_id = 0x02000000 | (i as u32 + 1);
        reg.register(Box::new(simulated::SimulatedSensor::new(sensor_id, SensorType::SPI)));
    }
    reg
}

/// Encode sensor type into sensor_id high byte
#[inline]
pub const fn encode_sensor_id(sensor_type: SensorType, local_id: u32) -> u32 {
    let prefix = match sensor_type {
        SensorType::I2C => 0x01,
        SensorType::SPI => 0x02,
        SensorType::PCIe => 0x03,
        SensorType::PMIC => 0x04,
        SensorType::FPGA => 0x05,
        SensorType::Unknown => 0x00,
    };
    (prefix << 24) | (local_id & 0x00FFFFFF)
}

/// Prepare a frame with standard metadata flags
#[inline]
pub fn finalize_frame(mut frame: EnergyTelemetryFrame) -> EnergyTelemetryFrame {
    if frame.timestamp_ns == 0 {
        frame.timestamp_ns = timestamp_ns();
    }
    frame.set_flag(flags::FLAG_DMA_VALID);
    frame.set_flag(flags::FLAG_CHECKSUM_OK);
    frame
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_sensor_id() {
        assert_eq!(encode_sensor_id(SensorType::I2C, 1), 0x01000001);
        assert_eq!(encode_sensor_id(SensorType::PCIe, 42), 0x0300002A);
    }
}
