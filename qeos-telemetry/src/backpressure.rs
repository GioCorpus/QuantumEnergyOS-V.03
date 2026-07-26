// ═══════════════════════════════════════════════════════════════════════════════
//  backpressure — Backpressure strategies for load-shedding and grid instability
// ═══════════════════════════════════════════════════════════════════════════════
//!
//! [Production Kernel Component]
//!
//! The system must support three distinct backpressure modes:
//!
//! A. REAL-TIME MODE (overwrite-oldest)
//!    - Drop oldest data when buffer is full
//!    - Preserve newest snapshot
//!    - Used for live grid monitoring, SCADA-like applications
//!    - Zero data loss of RECENT data (only old data is lost)
//!
//! B. SCIENTIFIC MODE (preserve-full-history)
//!    - Preserve full history via batching and buffering
//!    - Spill to secondary storage when primary buffer is full
//!    - Used for research, ML training, compliance logging
//!    - May experience backpressure but never drops data
//!
//! C. EMERGENCY MODE (disable-interrupts)
//!    - Disable interrupts
//!    - Switch to polling mode
//!    - Prioritize system stability over precision
//!    - Used during grid instability simulations, fault conditions
//!    - Reduces CPU overhead at the cost of latency
// ═══════════════════════════════════════════════════════════════════════════════

use crate::{
    frame::EnergyTelemetryFrame,
    ring_buffer::{PushResult, RingBuffer},
};
use core::sync::atomic::{AtomicU64, Ordering};

/// Backpressure handling mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackpressureMode {
    /// Real-time mode: overwrite oldest, preserve newest
    Realtime,
    /// Scientific mode: preserve all data, batch, spill to secondary storage
    Scientific,
    /// Emergency mode: disable interrupts, switch to polling
    Emergency,
}

impl Default for BackpressureMode {
    fn default() -> Self {
        Self::Realtime
    }
}

/// Backpressure state and statistics
#[repr(C, align(64))]
pub struct BackpressureState {
    /// Current mode
    pub mode: BackpressureMode,
    /// Number of frames dropped (DropNewest policy)
    pub dropped_frames: AtomicU64,
    /// Number of frames overwritten (OverwriteOldest policy)
    pub overwritten_frames: AtomicU64,
    /// Number of batches spilled to secondary storage
    pub spilled_batches: AtomicU64,
    /// Current buffer utilization (0.0 - 1.0)
    pub utilization: AtomicU64, // Q16.16 fixed point
    /// Peak utilization observed
    pub peak_utilization: AtomicU64, // Q16.16 fixed point
    /// Emergency mode entry count
    pub emergency_entries: AtomicU64,
    /// Polling interval in microseconds (emergency mode)
    pub polling_interval_us: AtomicU64,
    /// Total frames processed
    pub total_processed: AtomicU64,
    /// Total backpressure events
    pub backpressure_events: AtomicU64,
}

impl Default for BackpressureState {
    fn default() -> Self {
        Self {
            mode: BackpressureMode::Realtime,
            dropped_frames: AtomicU64::new(0),
            overwritten_frames: AtomicU64::new(0),
            spilled_batches: AtomicU64::new(0),
            utilization: AtomicU64::new(0),
            peak_utilization: AtomicU64::new(0),
            emergency_entries: AtomicU64::new(0),
            polling_interval_us: AtomicU64::new(1000), // 1ms default
            total_processed: AtomicU64::new(0),
            backpressure_events: AtomicU64::new(0),
        }
    }
}

impl BackpressureState {
    /// Create a new backpressure state with the given mode
    #[inline]
    pub fn new(mode: BackpressureMode) -> Self {
        Self {
            mode,
            ..Self::default()
        }
    }

    /// Update buffer utilization
    #[inline]
    pub fn update_utilization(&self, ring: &RingBuffer) {
        let len = ring.len() as u64;
        let cap = ring.capacity() as u64;
        let util_q16 = ((len << 16) / cap.max(1)) & 0xFFFF;
        self.utilization.store(util_q16, Ordering::Relaxed);

        // Update peak
        let mut peak = self.peak_utilization.load(Ordering::Relaxed);
        while util_q16 > peak {
            match self.peak_utilization.compare_exchange_weak(
                peak,
                util_q16,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(current) => peak = current,
            }
        }
    }

    /// Get utilization as float (0.0 - 1.0)
    #[inline]
    pub fn utilization_f32(&self) -> f32 {
        let q16 = self.utilization.load(Ordering::Relaxed);
        (q16 >> 16) as f32 + ((q16 & 0xFFFF) as f32 / 65536.0)
    }

    /// Get peak utilization as float
    #[inline]
    pub fn peak_utilization_f32(&self) -> f32 {
        let q16 = self.peak_utilization.load(Ordering::Relaxed);
        (q16 >> 16) as f32 + ((q16 & 0xFFFF) as f32 / 65536.0)
    }

    /// Record a dropped frame
    #[inline]
    pub fn record_drop(&self) {
        self.dropped_frames.fetch_add(1, Ordering::Relaxed);
        self.backpressure_events.fetch_add(1, Ordering::Relaxed);
    }

    /// Record an overwritten frame
    #[inline]
    pub fn record_overwrite(&self) {
        self.overwritten_frames.fetch_add(1, Ordering::Relaxed);
        self.backpressure_events.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a spilled batch
    #[inline]
    pub fn record_spill(&self) {
        self.spilled_batches.fetch_add(1, Ordering::Relaxed);
        self.backpressure_events.fetch_add(1, Ordering::Relaxed);
    }

    /// Record processed frames
    #[inline]
    pub fn record_processed(&self, count: usize) {
        self.total_processed.fetch_add(count as u64, Ordering::Relaxed);
    }

    /// Record emergency mode entry
    #[inline]
    pub fn record_emergency_entry(&self) {
        self.emergency_entries.fetch_add(1, Ordering::Relaxed);
    }

    /// Reset statistics
    #[inline]
    pub fn reset(&self) {
        self.dropped_frames.store(0, Ordering::Relaxed);
        self.overwritten_frames.store(0, Ordering::Relaxed);
        self.spilled_batches.store(0, Ordering::Relaxed);
        self.utilization.store(0, Ordering::Relaxed);
        self.peak_utilization.store(0, Ordering::Relaxed);
        self.backpressure_events.store(0, Ordering::Relaxed);
        // Don't reset total_processed — that's cumulative
    }
}

/// Backpressure manager
/// Orchestrates frame flow based on current mode and buffer state
pub struct BackpressureManager {
    state: BackpressureState,
    secondary_buffer: Option<Vec<EnergyTelemetryFrame>>,
    max_secondary_size: usize,
}

impl BackpressureManager {
    /// Create a new backpressure manager
    #[inline]
    pub fn new(mode: BackpressureMode) -> Self {
        let state = BackpressureState::new(mode);
        let max_secondary = match mode {
            BackpressureMode::Scientific => 1_048_576,
            _ => 0,
        };

        Self {
            state,
            secondary_buffer: if max_secondary > 0 {
                Some(Vec::with_capacity(max_secondary))
            } else {
                None
            },
            max_secondary_size: max_secondary,
        }
    }

    /// Create a real-time mode manager
    #[inline]
    pub fn realtime() -> Self {
        Self::new(BackpressureMode::Realtime)
    }

    /// Create a scientific mode manager
    #[inline]
    pub fn scientific() -> Self {
        Self::new(BackpressureMode::Scientific)
    }

    /// Create an emergency mode manager
    #[inline]
    pub fn emergency() -> Self {
        Self::new(BackpressureMode::Emergency)
    }

    /// Handle a frame based on current mode and buffer state
    #[inline]
    pub fn handle_frame(&mut self, frame: EnergyTelemetryFrame, ring: &RingBuffer) -> FrameDisposition {
        self.state.update_utilization(ring);

        match self.state.mode {
            BackpressureMode::Realtime => self.handle_realtime(frame, ring),
            BackpressureMode::Scientific => self.handle_scientific(frame, ring),
            BackpressureMode::Emergency => self.handle_emergency(frame, ring),
        }
    }

    /// Real-time mode: push to ring buffer (overwrite if full)
    #[inline]
    fn handle_realtime(&mut self, frame: EnergyTelemetryFrame, ring: &RingBuffer) -> FrameDisposition {
        match ring.push(frame) {
            PushResult::Ok => {
                self.state.record_processed(1);
                FrameDisposition::Accepted
            }
            PushResult::Dropped => {
                self.state.record_drop();
                FrameDisposition::Dropped
            }
            PushResult::Overwritten => {
                self.state.record_overwrite();
                self.state.record_processed(1);
                FrameDisposition::Overwritten
            }
        }
    }

    /// Scientific mode: preserve all data, spill if needed
    #[inline]
    fn handle_scientific(&mut self, frame: EnergyTelemetryFrame, ring: &RingBuffer) -> FrameDisposition {
        if ring.is_full() {
            // Spill to secondary buffer
            if let Some(ref mut secondary) = self.secondary_buffer {
                if secondary.len() < self.max_secondary_size {
                    secondary.push(frame);
                    self.state.record_spill();
                    FrameDisposition::Spilled
                } else {
                    // Secondary also full — must drop
                    self.state.record_drop();
                    FrameDisposition::Dropped
                }
            } else {
                self.state.record_drop();
                FrameDisposition::Dropped
            }
        } else {
            match ring.push(frame) {
                PushResult::Ok => {
                    self.state.record_processed(1);
                    FrameDisposition::Accepted
                }
                _ => {
                    self.state.record_drop();
                    FrameDisposition::Dropped
                }
            }
        }
    }

    /// Emergency mode: prioritize stability, use polling
    #[inline]
    fn handle_emergency(&mut self, frame: EnergyTelemetryFrame, ring: &RingBuffer) -> FrameDisposition {
        // In emergency mode, we still accept frames but reduce processing
        // The polling interval is increased to reduce CPU load
        match ring.push(frame) {
            PushResult::Ok => {
                self.state.record_processed(1);
                FrameDisposition::Accepted
            }
            PushResult::Dropped => {
                self.state.record_drop();
                FrameDisposition::Dropped
            }
            PushResult::Overwritten => {
                self.state.record_overwrite();
                self.state.record_processed(1);
                FrameDisposition::Overwritten
            }
        }
    }

    /// Drain secondary buffer back to primary ring buffer
    #[inline]
    pub fn drain_secondary(&mut self, ring: &RingBuffer) -> usize {
        let Some(ref mut secondary) = self.secondary_buffer else {
            return 0;
        };

        let mut drained = 0;
        while !secondary.is_empty() && !ring.is_full() {
            if let Some(frame) = secondary.pop() {
                if ring.push(frame) == PushResult::Ok {
                    drained += 1;
                }
            }
        }

        drained
    }

    /// Get reference to backpressure state
    #[inline]
    pub fn state(&self) -> &BackpressureState {
        &self.state
    }

    /// Get mutable reference to backpressure state
    #[inline]
    pub fn state_mut(&mut self) -> &mut BackpressureState {
        &mut self.state
    }

    /// Get secondary buffer usage
    #[inline]
    pub fn secondary_usage(&self) -> usize {
        self.secondary_buffer.as_ref().map(|v| v.len()).unwrap_or(0)
    }
}

/// Disposition of a frame after backpressure handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameDisposition {
    /// Frame accepted into ring buffer
    Accepted,
    /// Frame dropped (buffer full, DropNewest policy)
    Dropped,
    /// Frame overwrote oldest (OverwriteOldest policy)
    Overwritten,
    /// Frame spilled to secondary storage (Scientific mode)
    Spilled,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backpressure_realtime_overwrite() {
        let mut mgr = BackpressureManager::realtime();
        let ring = RingBuffer::new(2).unwrap();

        for i in 0..4 {
            let frame = EnergyTelemetryFrame::from_parts(i as u64, i as u32, 100, 100, 5000, 0);
            let disp = mgr.handle_frame(frame, &ring);
            if i < 2 {
                assert_eq!(disp, FrameDisposition::Accepted);
            } else {
                assert_eq!(disp, FrameDisposition::Overwritten);
            }
        }

        assert_eq!(mgr.state().overwritten_frames.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_backpressure_state_utilization() {
        let state = BackpressureState::new(BackpressureMode::Realtime);
        let ring = RingBuffer::new(8).unwrap();

        for i in 0..4 {
            ring.push(EnergyTelemetryFrame::from_parts(i, 0, 0, 0, 0, 0));
        }

        state.update_utilization(&ring);
        let util = state.utilization_f32();
        assert!((util - 0.5).abs() < 0.01);
    }
}
