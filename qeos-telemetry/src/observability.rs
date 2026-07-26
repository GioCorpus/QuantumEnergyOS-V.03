// ═══════════════════════════════════════════════════════════════════════════════
//  observability — Atomic counters, performance metrics, and diagnostics
// ═══════════════════════════════════════════════════════════════════════════════
//!
//! [Production Kernel Component]
//!
//! Observability is ONLY allowed in the safe path (bottom half, userspace).
//! NO logging in the IRQ path (top half).
//!
//! This module provides:
//!   - Atomic performance counters (no locking)
//!   - Ring buffer diagnostics
//!   - Lightweight tracing
//!   - Metrics export for monitoring systems
// ═══════════════════════════════════════════════════════════════════════════════

use crate::{
    ring_buffer::RingBuffer,
};
use core::sync::atomic::{AtomicU64, Ordering};

/// Telemetry performance counters
/// All counters are atomic for lock-free access from any context
#[repr(C, align(64))]
pub struct TelemetryCounters {
    /// Total IRQs received
    pub irq_count: AtomicU64,
    /// Total DMA completions (frames)
    pub dma_complete_count: AtomicU64,
    /// Total frames written to ring buffer
    pub frames_written: AtomicU64,
    /// Total frames read from ring buffer
    pub frames_read: AtomicU64,
    /// Total frames dropped (DropNewest policy)
    pub dropped_count: AtomicU64,
    /// Total frames overwritten (OverwriteOldest policy)
    pub overwritten_count: AtomicU64,
    /// Total checksums verified OK
    pub checksum_ok_count: AtomicU64,
    /// Total checksum failures
    pub checksum_fail_count: AtomicU64,
    /// Total anomalies detected
    pub anomaly_count: AtomicU64,
    /// Total bottom-half invocations
    pub bh_invoke_count: AtomicU64,
    /// Total frames processed by bottom half
    pub bh_frame_count: AtomicU64,
    /// Total bottom-half schedules
    pub bh_schedule_count: AtomicU64,
    /// Total errors
    pub error_count: AtomicU64,
    /// Total calibration applications
    pub calibration_count: AtomicU64,
    /// Last update timestamp (nanoseconds)
    pub last_update_ns: AtomicU64,
    /// Padding to separate hot/cold counters
    _pad: [u8; 64],
}

impl TelemetryCounters {
    /// Create a new set of counters
    #[inline]
    pub const fn new() -> Self {
        Self {
            irq_count: AtomicU64::new(0),
            dma_complete_count: AtomicU64::new(0),
            frames_written: AtomicU64::new(0),
            frames_read: AtomicU64::new(0),
            dropped_count: AtomicU64::new(0),
            overwritten_count: AtomicU64::new(0),
            checksum_ok_count: AtomicU64::new(0),
            checksum_fail_count: AtomicU64::new(0),
            anomaly_count: AtomicU64::new(0),
            bh_invoke_count: AtomicU64::new(0),
            bh_frame_count: AtomicU64::new(0),
            bh_schedule_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            calibration_count: AtomicU64::new(0),
            last_update_ns: AtomicU64::new(0),
            _pad: [0u8; 64],
        }
    }

    /// Snapshot all counters into a plain struct (no atomics)
    #[inline]
    pub fn snapshot(&self) -> CounterSnapshot {
        CounterSnapshot {
            irq_count: self.irq_count.load(Ordering::Relaxed),
            dma_complete_count: self.dma_complete_count.load(Ordering::Relaxed),
            frames_written: self.frames_written.load(Ordering::Relaxed),
            frames_read: self.frames_read.load(Ordering::Relaxed),
            dropped_count: self.dropped_count.load(Ordering::Relaxed),
            overwritten_count: self.overwritten_count.load(Ordering::Relaxed),
            checksum_ok_count: self.checksum_ok_count.load(Ordering::Relaxed),
            checksum_fail_count: self.checksum_fail_count.load(Ordering::Relaxed),
            anomaly_count: self.anomaly_count.load(Ordering::Relaxed),
            bh_invoke_count: self.bh_invoke_count.load(Ordering::Relaxed),
            bh_frame_count: self.bh_frame_count.load(Ordering::Relaxed),
            bh_schedule_count: self.bh_schedule_count.load(Ordering::Relaxed),
            error_count: self.error_count.load(Ordering::Relaxed),
            calibration_count: self.calibration_count.load(Ordering::Relaxed),
        }
    }

    /// Compute derived metrics from snapshot
    #[inline]
    pub fn compute_metrics(snap: &CounterSnapshot) -> PerformanceMetrics {
        let total = snap.frames_written.max(1);
        PerformanceMetrics {
            throughput_fps: snap.frames_written, // Approximate
            drop_rate: snap.dropped_count as f64 / total as f64,
            overwrite_rate: snap.overwritten_count as f64 / total as f64,
            checksum_failure_rate: snap.checksum_fail_count as f64
                / (snap.checksum_ok_count + snap.checksum_fail_count).max(1) as f64,
            anomaly_rate: snap.anomaly_count as f64 / total as f64,
            bh_efficiency: snap.bh_frame_count as f64 / snap.bh_schedule_count.max(1) as f64,
            irq_rate: snap.irq_count as f64,
        }
    }

    /// Reset all counters (for testing/benchmarking)
    #[inline]
    pub fn reset(&self) {
        self.irq_count.store(0, Ordering::Relaxed);
        self.dma_complete_count.store(0, Ordering::Relaxed);
        self.frames_written.store(0, Ordering::Relaxed);
        self.frames_read.store(0, Ordering::Relaxed);
        self.dropped_count.store(0, Ordering::Relaxed);
        self.overwritten_count.store(0, Ordering::Relaxed);
        self.checksum_ok_count.store(0, Ordering::Relaxed);
        self.checksum_fail_count.store(0, Ordering::Relaxed);
        self.anomaly_count.store(0, Ordering::Relaxed);
        self.bh_invoke_count.store(0, Ordering::Relaxed);
        self.bh_frame_count.store(0, Ordering::Relaxed);
        self.bh_schedule_count.store(0, Ordering::Relaxed);
        self.error_count.store(0, Ordering::Relaxed);
        self.calibration_count.store(0, Ordering::Relaxed);
    }
}

/// Point-in-time snapshot of all counters
#[derive(Debug, Clone, Copy, Default)]
pub struct CounterSnapshot {
    pub irq_count: u64,
    pub dma_complete_count: u64,
    pub frames_written: u64,
    pub frames_read: u64,
    pub dropped_count: u64,
    pub overwritten_count: u64,
    pub checksum_ok_count: u64,
    pub checksum_fail_count: u64,
    pub anomaly_count: u64,
    pub bh_invoke_count: u64,
    pub bh_frame_count: u64,
    pub bh_schedule_count: u64,
    pub error_count: u64,
    pub calibration_count: u64,
}

/// Derived performance metrics
#[derive(Debug, Clone, Copy, Default)]
pub struct PerformanceMetrics {
    pub throughput_fps: u64,
    pub drop_rate: f64,
    pub overwrite_rate: f64,
    pub checksum_failure_rate: f64,
    pub anomaly_rate: f64,
    pub bh_efficiency: f64,
    pub irq_rate: f64,
}

impl core::fmt::Display for PerformanceMetrics {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "PerformanceMetrics {{ throughput={}fps drop={:.2}% overwrite={:.2}% checksum_fail={:.2}% anomaly={:.2}% bh_eff={:.2} irq_rate={:.1} }}",
            self.throughput_fps,
            self.drop_rate * 100.0,
            self.overwrite_rate * 100.0,
            self.checksum_failure_rate * 100.0,
            self.anomaly_rate * 100.0,
            self.bh_efficiency,
            self.irq_rate,
        )
    }
}

/// Ring buffer diagnostics
pub struct RingBufferDiagnostics {
    /// Ring buffer reference
    ring: *const RingBuffer,
}

impl RingBufferDiagnostics {
    /// Create diagnostics for a ring buffer
    #[inline]
    pub const fn new(ring: *const RingBuffer) -> Self {
        Self { ring }
    }

    /// Get current utilization
    #[inline]
    pub fn utilization(&self) -> f32 {
        if self.ring.is_null() {
            return 0.0;
        }
        let ring = unsafe { &*self.ring };
        ring.len() as f32 / ring.capacity() as f32
    }

    /// Get fill level
    #[inline]
    pub fn fill_level(&self) -> usize {
        if self.ring.is_null() {
            return 0;
        }
        unsafe { (*self.ring).len() }
    }

    /// Get capacity
    #[inline]
    pub fn capacity(&self) -> usize {
        if self.ring.is_null() {
            return 0;
        }
        unsafe { (*self.ring).capacity() }
    }

    /// Check if buffer is approaching capacity (>80%)
    #[inline]
    pub fn is_near_capacity(&self) -> bool {
        self.utilization() > 0.8
    }

    /// Check if buffer is critical (>95%)
    #[inline]
    pub fn is_critical(&self) -> bool {
        self.utilization() > 0.95
    }
}

/// Lightweight trace event
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TraceEvent {
    /// Event timestamp (nanoseconds)
    pub timestamp_ns: u64,
    /// Event type
    pub event_type: TraceEventType,
    /// Associated data (sensor_id, frame_index, etc.)
    pub data: u32,
    /// CPU core that generated the event
    pub cpu: u32,
}

/// Trace event types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum TraceEventType {
    IRQEntry = 1,
    IRQExit = 2,
    BHStart = 3,
    BHEnd = 4,
    FrameWritten = 5,
    FrameDropped = 6,
    FrameOverwritten = 7,
    AnomalyDetected = 8,
    EmergencyEntry = 9,
    EmergencyExit = 10,
    DmaComplete = 11,
}

/// Circular trace buffer (lock-free, single producer)
pub struct TraceBuffer {
    /// Trace events buffer
    events: Vec<TraceEvent>,
    /// Write index (producer)
    head: AtomicU64,
    /// Capacity (power of two)
    _capacity: usize,
    /// Mask for fast indexing
    mask: u64,
}

impl TraceBuffer {
    /// Create a new trace buffer
    #[inline]
    pub fn new(capacity: usize) -> Self {
        let cap = capacity.next_power_of_two();
        Self {
            events: Vec::with_capacity(cap),
            head: AtomicU64::new(0),
            _capacity: cap,
            mask: (cap - 1) as u64,
        }
    }

    /// Write a trace event (lock-free, single producer)
    #[inline]
    pub fn write(&self, event_type: TraceEventType, data: u32) {
        let head = self.head.fetch_add(1, Ordering::Relaxed);
        let idx = (head & self.mask) as usize;

        #[cfg(target_arch = "x86_64")]
        let cpu = unsafe { core::arch::x86_64::_rdtsc() as u32 };
        #[cfg(not(target_arch = "x86_64"))]
        let cpu = 0;

        let event = TraceEvent {
            timestamp_ns: crate::platform::timestamp_ns(),
            event_type,
            data,
            cpu,
        };

        if idx < self.events.len() {
            unsafe {
                core::ptr::write(self.events.as_ptr().add(idx) as *mut TraceEvent, event);
            }
        }
    }

    /// Get the number of events written
    #[inline]
    pub fn len(&self) -> u64 {
        self.head.load(Ordering::Relaxed)
    }

    /// Check if empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counters() {
        let counters = TelemetryCounters::new();
        counters.irq_count.fetch_add(10, Ordering::Relaxed);
        let snap = counters.snapshot();
        assert_eq!(snap.irq_count, 10);
    }

    #[test]
    fn test_metrics_computation() {
        let snap = CounterSnapshot {
            frames_written: 1000,
            dropped_count: 50,
            overwritten_count: 30,
            checksum_ok_count: 900,
            checksum_fail_count: 100,
            anomaly_count: 20,
            bh_frame_count: 950,
            bh_schedule_count: 10,
            ..Default::default()
        };
        let metrics = TelemetryCounters::compute_metrics(&snap);
        assert!((metrics.drop_rate - 0.05).abs() < 0.001);
        assert!((metrics.bh_efficiency - 95.0).abs() < 0.1);
    }
}
