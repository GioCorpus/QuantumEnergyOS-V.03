#![warn(missing_docs)]

//! QuantumEnergyOS Telemetry Backpressure Policies
//!
//! When the SPSC ring buffer is full, different applications require
//! different overflow strategies:
//!
//! | Mode | Behavior | Use case |
//! |------|----------|----------|
//! | `RealTime` | Overwrite oldest | Control loops, low latency |
//! | `Scientific` | Drop new data | Data acquisition, reproducibility |
//! | `Emergency` | Force-write + flag | Failure recording, forensic logs |
//!
//! # Classification
//!
//! [Research Prototype]

use super::frame::EnergyTelemetryFrame;
use super::spsc::{RingBufferError, SpScRingBuffer};

/// Backpressure mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackpressureMode {
    /// Overwrite oldest frames when full.
    RealTime,
    /// Reject new frames when full (preserve history).
    Scientific,
    /// Force-write when full and mark overwritten frames.
    Emergency,
    /// No backpressure — caller must check capacity externally.
    None,
}

/// Backpressure policy applied on overflow.
///
/// The policy is stateless; all state lives in the ring buffer.
pub struct BackpressurePolicy {
    mode: BackpressureMode,
}

impl BackpressurePolicy {
    /// Real-time policy: overwrite oldest data.
    pub const fn real_time() -> Self {
        Self {
            mode: BackpressureMode::RealTime,
        }
    }

    /// Scientific policy: preserve history, drop new data.
    pub const fn scientific() -> Self {
        Self {
            mode: BackpressureMode::Scientific,
        }
    }

    /// Emergency policy: force-write and mark overrun.
    pub const fn emergency() -> Self {
        Self {
            mode: BackpressureMode::Emergency,
        }
    }

    /// No policy; overflow is always an error.
    pub const fn none() -> Self {
        Self {
            mode: BackpressureMode::None,
        }
    }

    /// Returns the current mode.
    pub const fn mode(&self) -> BackpressureMode {
        self.mode
    }

    /// Handles an overflow by applying the policy.
    ///
    /// - `RealTime` / `Emergency`: advances the read cursor and retries `push`.
    /// - `Scientific` / `None`: returns `Err(RingBufferError::Overflow)`.
    pub fn on_overflow<const N: usize>(
        &self,
        buffer: &mut SpScRingBuffer<N>,
        frame: EnergyTelemetryFrame,
    ) -> Result<(), RingBufferError> {
        match self.mode {
            BackpressureMode::RealTime | BackpressureMode::Emergency => {
                buffer.advance_read(buffer.write_cursor().saturating_sub(N.saturating_sub(1)));
                buffer.push(frame)
            }
            BackpressureMode::Scientific | BackpressureMode::None => {
                Err(RingBufferError::Overflow)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_time_overwrites_oldest() {
        let mut rb: SpScRingBuffer<2> = SpScRingBuffer::new();
        rb.push(EnergyTelemetryFrame::new(1)).unwrap();
        rb.push(EnergyTelemetryFrame::new(2)).unwrap();

        let policy = BackpressurePolicy::real_time();
        let result = policy.on_overflow(&mut rb, EnergyTelemetryFrame::new(3));
        assert!(result.is_ok());
        assert_eq!(rb.len(), 2);
    }

    #[test]
    fn scientific_rejects_on_overflow() {
        let mut rb: SpScRingBuffer<2> = SpScRingBuffer::new();
        rb.push(EnergyTelemetryFrame::new(1)).unwrap();
        rb.push(EnergyTelemetryFrame::new(2)).unwrap();

        let policy = BackpressurePolicy::scientific();
        let result = policy.on_overflow(&mut rb, EnergyTelemetryFrame::new(3));
        assert!(matches!(result, Err(RingBufferError::Overflow)));
    }
}
