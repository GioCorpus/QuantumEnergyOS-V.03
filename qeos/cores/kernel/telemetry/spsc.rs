use core::sync::atomic::{AtomicUsize, Ordering};
use super::frame::EnergyTelemetryFrame;

pub const CACHE_LINE_SIZE: usize = 64;

#[repr(C, align(64))]
pub struct ProducerState {
    pub cursor: AtomicUsize,
    pub reserved: [u8; CACHE_LINE_SIZE - core::mem::size_of::<AtomicUsize>()],
}

#[repr(C, align(64))]
pub struct ConsumerState {
    pub cursor: AtomicUsize,
    pub reserved: [u8; CACHE_LINE_SIZE - core::mem::size_of::<AtomicUsize>()],
}

pub struct SpScRingBuffer<const N: usize> {
    buffer: [EnergyTelemetryFrame; N],
    producer: ProducerState,
    consumer: ConsumerState,
    capacity_mask: usize,
}

impl<const N: usize> SpScRingBuffer<N> {
    const fn is_power_of_two(x: usize) -> bool {
        x != 0 && (x & (x - 1)) == 0
    }

    pub fn new() -> Self {
        assert!(Self::is_power_of_two(N), "Ring buffer capacity must be power of two");
        
        Self {
            buffer: core::array::from_fn(|_| EnergyTelemetryFrame::new(0)),
            producer: ProducerState {
                cursor: AtomicUsize::new(0),
                reserved: [0; CACHE_LINE_SIZE - core::mem::size_of::<AtomicUsize>()],
            },
            consumer: ConsumerState {
                cursor: AtomicUsize::new(0),
                reserved: [0; CACHE_LINE_SIZE - core::mem::size_of::<AtomicUsize>()],
            },
            capacity_mask: N - 1,
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        N
    }

    #[inline]
    pub unsafe fn push(&mut self, frame: EnergyTelemetryFrame) -> Result<(), RingBufferError> {
        let write_idx = self.producer.cursor.load(Ordering::Relaxed);
        let read_idx = self.consumer.cursor.load(Ordering::Acquire);
        
        let available = N - (write_idx - read_idx);
        
        if available == 0 {
            return Err(RingBufferError::Overflow);
        }
        
        self.buffer[write_idx & self.capacity_mask] = frame;
        self.producer.cursor.store(write_idx + 1, Ordering::Release);
        
        Ok(())
    }

    #[inline]
    pub unsafe fn pop(&mut self) -> Option<EnergyTelemetryFrame> {
        let read_idx = self.consumer.cursor.load(Ordering::Relaxed);
        let write_idx = self.producer.cursor.load(Ordering::Acquire);
        
        if read_idx == write_idx {
            return None;
        }
        
        let frame = self.buffer[read_idx & self.capacity_mask];
        core::sync::atomic::fence(Ordering::AcqRel);
        self.consumer.cursor.store(read_idx + 1, Ordering::Release);
        
        Some(frame)
    }

    #[inline]
    pub fn len(&self) -> usize {
        let read_idx = self.consumer.cursor.load(Ordering::Relaxed);
        let write_idx = self.producer.cursor.load(Ordering::Relaxed);
        write_idx.saturating_sub(read_idx) as usize
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn available_capacity(&self) -> usize {
        N - self.len()
    }

    #[inline]
    pub fn write_cursor(&self) -> usize {
        self.producer.cursor.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn read_cursor(&self) -> usize {
        self.consumer.cursor.load(Ordering::Acquire)
    }

    #[inline]
    pub fn advance_read(&mut self, idx: usize) {
        self.consumer.cursor.store(idx, Ordering::Release);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RingBufferError {
    Overflow,
    Underflow,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop_single_frame() {
        let mut rb: SpScRingBuffer<256> = SpScRingBuffer::new();
        let frame = EnergyTelemetryFrame::new(1);
        
        unsafe {
            rb.push(frame).unwrap();
            let popped = rb.pop().unwrap();
            assert_eq!(popped.sensor_id, 1);
        }
    }
}