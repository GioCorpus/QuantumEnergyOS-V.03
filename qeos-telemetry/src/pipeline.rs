// ═══════════════════════════════════════════════════════════════════════════════
//  pipeline — End-to-end telemetry ingestion pipeline orchestrator
// ═══════════════════════════════════════════════════════════════════════════════
//!
//! [Production Kernel Component]
//!
//! Wires hardware backends → driver IRQ path → DMA ring → userspace consumer.
//! Supports all three backpressure modes with deterministic batching.

use crate::{
    backpressure::{BackpressureManager, BackpressureMode, FrameDisposition},
    config::{SensorType, TelemetryConfig},
    driver::DriverContext,
    frame::EnergyTelemetryFrame,
    observability::{CounterSnapshot, PerformanceMetrics, TelemetryCounters},
    ring_buffer::{FillPolicy, RingBuffer},
};
use core::sync::atomic::Ordering;

/// Pipeline operating mode mapped to ring buffer fill policy
#[inline]
pub fn fill_policy_for_mode(mode: BackpressureMode) -> FillPolicy {
    match mode {
        BackpressureMode::Realtime => FillPolicy::OverwriteOldest,
        BackpressureMode::Scientific => FillPolicy::DropNewest,
        BackpressureMode::Emergency => FillPolicy::BackpressureAware,
    }
}

/// Complete telemetry pipeline (userspace simulation of kernel+daemon path)
pub struct TelemetryPipeline {
    config: TelemetryConfig,
    driver: DriverContext,
    backpressure: BackpressureManager,
    batch: Vec<EnergyTelemetryFrame>,
}

impl TelemetryPipeline {
    /// Create pipeline from configuration
    pub fn new(config: TelemetryConfig) -> Option<Self> {
        let cap = config.ring_buffer_capacity;
        let sensor_type = SensorType::SPI;
        let mut driver = DriverContext::new(cap, config.irq_cpu as u32, sensor_type)?;
        driver.anomaly_detection = config.enable_anomaly_detection;

        let policy = fill_policy_for_mode(config.backpressure_mode);
        let output_ring = RingBuffer::with_policy(cap, policy)?;
        driver.output_ring = output_ring;

        let backpressure = BackpressureManager::new(config.backpressure_mode);

        Some(Self {
            config,
            driver,
            backpressure,
            batch: Vec::with_capacity(256),
        })
    }

    /// Ingest a raw frame through the full IRQ → BH → ring path
    pub fn ingest(&mut self, frame: EnergyTelemetryFrame) -> FrameDisposition {
        let _ = self.driver.simulate_irq(frame);
        if let Some(f) = self.driver.output_ring.pop() {
            self.backpressure.handle_frame(f, &self.driver.output_ring)
        } else {
            FrameDisposition::Dropped
        }
    }

    /// Drain available frames into internal batch (zero-copy within batch vec)
    pub fn drain_batch(&mut self, max: usize) -> &[EnergyTelemetryFrame] {
        self.batch.clear();
        while self.batch.len() < max {
            match self.driver.output_ring.pop() {
                Some(f) => self.batch.push(f),
                None => break,
            }
        }
        &self.batch
    }

    /// Switch backpressure mode at runtime
    pub fn set_mode(&mut self, mode: BackpressureMode) {
        self.config.backpressure_mode = mode;
        self.backpressure = BackpressureManager::new(mode);
        if let Some(ring) = RingBuffer::with_policy(
            self.config.ring_buffer_capacity,
            fill_policy_for_mode(mode),
        ) {
            self.driver.output_ring = ring;
        }
    }

    /// Performance counter snapshot
    pub fn counters(&self) -> CounterSnapshot {
        self.driver.counters.snapshot()
    }

    /// Derived performance metrics
    pub fn metrics(&self) -> PerformanceMetrics {
        TelemetryCounters::compute_metrics(&self.counters())
    }

    /// Ring buffer utilization (0.0–1.0)
    pub fn utilization(&self) -> f32 {
        let len = self.driver.output_ring.len();
        let cap = self.driver.output_ring.capacity();
        len as f32 / cap.max(1) as f32
    }

    /// Reference to output ring (for mmap-style readers)
    pub fn output_ring(&self) -> &RingBuffer {
        &self.driver.output_ring
    }

    /// Reference to driver counters
    pub fn driver_counters(&self) -> &TelemetryCounters {
        &self.driver.counters
    }

    /// Total IRQ count processed
    pub fn irq_count(&self) -> u64 {
        self.driver.counters.irq_count.load(Ordering::Relaxed)
    }

    /// Configuration reference
    pub fn config(&self) -> &TelemetryConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::flags;

    #[test]
    fn test_pipeline_ingest() {
        let config = TelemetryConfig::realtime();
        let mut pipe = TelemetryPipeline::new(config).unwrap();
        let frame = EnergyTelemetryFrame::from_parts(
            1000, 1, 230000, 10000, 5000, flags::FLAG_CHECKSUM_OK,
        );
        let disp = pipe.ingest(frame);
        assert!(matches!(disp, FrameDisposition::Accepted | FrameDisposition::Overwritten));
    }

    #[test]
    fn test_pipeline_mode_switch() {
        let mut pipe = TelemetryPipeline::new(TelemetryConfig::default()).unwrap();
        pipe.set_mode(BackpressureMode::Emergency);
        assert_eq!(pipe.config().backpressure_mode, BackpressureMode::Emergency);
    }
}
