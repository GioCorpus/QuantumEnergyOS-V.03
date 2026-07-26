use core::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TelemetrySample {
    pub timestamp_ns: u64,
    pub source_id: u64,
    pub value: f64,
    pub flags: u16,
}

#[derive(Debug, Default)]
pub struct TelemetryBus {
    pub samples: Vec<TelemetrySample>,
    pub dropped: AtomicU64,
    pub sample_rate_hz: u32,
}

#[derive(Debug)]
pub struct QuartzTelemetry {
    pub bus: TelemetryBus,
}

impl QuartzTelemetry {
    pub const fn new(sample_rate_hz: u32) -> Self {
        Self {
            bus: TelemetryBus {
                samples: Vec::new(),
                dropped: AtomicU64::new(0),
                sample_rate_hz,
            },
        }
    }

    pub fn push(&mut self, sample: TelemetrySample) -> Result<(), ()> {
        self.bus.samples.push(sample);
        Ok(())
    }

    pub fn drop_count(&self) -> u64 {
        self.bus.dropped.load(Ordering::Relaxed)
    }

    pub fn sample_count(&self) -> usize {
        self.bus.samples.len()
    }
}

impl Clone for QuartzTelemetry {
    fn clone(&self) -> Self {
        Self {
            bus: TelemetryBus {
                samples: self.bus.samples.clone(),
                dropped: AtomicU64::new(0),
                sample_rate_hz: self.bus.sample_rate_hz,
            }
        }
    }
}
