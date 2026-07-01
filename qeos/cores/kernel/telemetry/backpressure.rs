use super::frame::EnergyTelemetryFrame;
use super::spsc::{RingBufferError, SpScRingBuffer};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackpressureMode {
    RealTime = 0,
    Scientific = 1,
    Emergency = 2,
}

pub enum BackpressurePolicy {
    RealTime { overwrite: bool },
    Scientific { preserve_history: bool },
    Emergency { polling_enabled: bool },
    Disabled,
}

impl BackpressurePolicy {
    pub const fn default_mode() -> BackpressureMode {
        BackpressureMode::RealTime
    }

    pub fn on_overflow<const N: usize>(
        &self,
        buffer: &mut SpScRingBufferWrapper<N>,
        frame: EnergyTelemetryFrame,
    ) -> Result<(), RingBufferError> {
        match self {
            Self::RealTime { overwrite: true } => unsafe { buffer.force_push(frame) },
            Self::Scientific { .. } => Err(RingBufferError::Overflow),
            Self::Emergency { polling_enabled } => {
                if *polling_enabled {
                    unsafe { buffer.force_push(frame) }
                } else {
                    Err(RingBufferError::Overflow)
                }
            }
            Self::Disabled => Err(RingBufferError::Overflow),
        }
    }

    pub fn real_time() -> Self {
        Self::RealTime { overwrite: true }
    }

    pub fn scientific() -> Self {
        Self::Scientific {
            preserve_history: true,
        }
    }

    pub fn emergency() -> Self {
        Self::Emergency {
            polling_enabled: true,
        }
    }
}

pub struct SpScRingBufferWrapper<const N: usize> {
    inner: SpScRingBuffer<N>,
}

impl<const N: usize> SpScRingBufferWrapper<N> {
    pub fn new() -> Self {
        Self {
            inner: SpScRingBuffer::new(),
        }
    }

    pub unsafe fn push(&mut self, frame: EnergyTelemetryFrame) -> Result<(), RingBufferError> {
        self.inner.push(frame)
    }

    #[inline]
    pub fn pop(&mut self) -> Option<EnergyTelemetryFrame> {
        unsafe { self.inner.pop() }
    }

    pub unsafe fn force_push(
        &mut self,
        frame: EnergyTelemetryFrame,
    ) -> Result<(), RingBufferError> {
        let read_idx = self.inner.read_cursor();
        let write_idx = self.inner.write_cursor();

        if write_idx.saturating_sub(read_idx) >= N {
            self.inner
                .advance_read(write_idx.saturating_sub(N.saturating_sub(1)));
        }

        self.inner.push(frame)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn available_capacity(&self) -> usize {
        self.inner.available_capacity()
    }
}
