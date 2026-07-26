use super::frame::EnergyTelemetryFrame;
use core::sync::atomic::{AtomicUsize, Ordering};

/// Cache-line size used for cache-line isolation of ring-buffer cursors.
pub const CACHE_LINE_SIZE: usize = 64;

/// Per-cursor state, padded to a full cache line to prevent false sharing.
///
/// # Safety
///
/// The layout is `#[repr(C, align(64))]` and the reserved padding byte count
/// is computed from `size_of::<AtomicUsize>()`. Do not reorder fields.
#[repr(C, align(64))]
pub struct ProducerState {
    /// Monotonically increasing write cursor.
    pub cursor: AtomicUsize,
    /// Padding to occupy one full cache line.
    pub reserved: [u8; CACHE_LINE_SIZE - core::mem::size_of::<AtomicUsize>()],
}

#[repr(C, align(64))]
pub struct ConsumerState {
    /// Monotonically increasing read cursor.
    pub cursor: AtomicUsize,
    /// Padding to occupy one full cache line.
    pub reserved: [u8; CACHE_LINE_SIZE - core::mem::size_of::<AtomicUsize>()],
}

unsafe impl Send for ProducerState {}
unsafe impl Sync for ProducerState {}
unsafe impl Send for ConsumerState {}
unsafe impl Sync for ConsumerState {}

/// Lock-free single-producer / single-consumer ring buffer.
///
/// # Template parameter
///
/// `N` — buffer capacity. Must be a power of two.
///
/// # Safety
///
/// The SPSC pattern is safe only if:
/// - Exactly one thread calls `push` / `write_cursor` / `available_capacity`.
/// - Exactly one (possibly different) thread calls `pop` / `read_cursor`.
/// - The producer never observes `read_cursor` via `Relaxed` and treats it as
///   authoritative without an `Acquire` fence.
///
/// The public API mirrors these preconditions: `push` and `pop` require `&mut self`,
/// which statically enforces exclusive access when used in a single-threaded context.
/// For multi-threaded use, split ownership with `alloc::rc::Rc<RefCell<…>>` or place
/// the producer/consumer halves behind `AtomicPtr`. This crate does **not** provide
/// a split-ownership wrapper yet.
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

    /// Creates a new ring buffer.
    ///
    /// # Panics
    ///
    /// Panics if `N` is not a power of two.
    #[allow(clippy::assertions_on_constants)]
    pub fn new() -> Self {
        assert!(
            Self::is_power_of_two(N),
            "Ring buffer capacity must be power of two"
        );

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

    /// Returns the capacity of the ring buffer.
    #[inline]
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Pushes a frame into the ring buffer.
    ///
    /// Returns `Err(RingBufferError::Overflow)` if no slot is available.
    ///
    /// # Safety
    ///
    /// This method is safe because `&mut self` guarantees exclusive access
    /// to the producer side. The atomic operations are used purely to publish
    /// the cursor across address spaces (e.g., shared memory), not for interior
    /// mutability within a single owner.
    #[inline]
    pub fn push(&mut self, frame: EnergyTelemetryFrame) -> Result<(), RingBufferError> {
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

    /// Pops a frame from the ring buffer.
    ///
    /// Returns `None` if the buffer is empty.
    #[inline]
    pub fn pop(&mut self) -> Option<EnergyTelemetryFrame> {
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

    /// Returns the number of frames currently in the buffer.
    #[inline]
    pub fn len(&self) -> usize {
        let read_idx = self.consumer.cursor.load(Ordering::Relaxed);
        let write_idx = self.producer.cursor.load(Ordering::Relaxed);
        write_idx.saturating_sub(read_idx) as usize
    }

    /// Returns true if the buffer contains no frames.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of slots available before overflow.
    #[inline]
    pub fn available_capacity(&self) -> usize {
        N - self.len()
    }

    /// Returns the current write cursor (monotonic).
    #[inline]
    pub fn write_cursor(&self) -> usize {
        self.producer.cursor.load(Ordering::Relaxed)
    }

    /// Returns the current read cursor (monotonic).
    #[inline]
    pub fn read_cursor(&self) -> usize {
        self.consumer.cursor.load(Ordering::Acquire)
    }

    /// Advances the read cursor to `idx` (monotonic).
    ///
    /// Used by backpressure policies that overwrite oldest data.
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

impl core::fmt::Display for RingBufferError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Overflow => write!(f, "ring buffer overflow"),
            Self::Underflow => write!(f, "ring buffer underflow"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop_single_frame() {
        let mut rb: SpScRingBuffer<256> = SpScRingBuffer::new();
        let frame = EnergyTelemetryFrame::new(1);

        rb.push(frame).unwrap();
        let popped = rb.pop().unwrap();
        assert_eq!(popped.sensor_id, 1);
    }

    #[test]
    fn overflow_returns_error() {
        let mut rb: SpScRingBuffer<2> = SpScRingBuffer::new();
        let f1 = EnergyTelemetryFrame::new(1);
        let f2 = EnergyTelemetryFrame::new(2);
        let f3 = EnergyTelemetryFrame::new(3);

        rb.push(f1).unwrap();
        rb.push(f2).unwrap();
        assert!(rb.push(f3).is_err());
    }

    #[test]
    fn pop_empty_returns_none() {
        let mut rb: SpScRingBuffer<4> = SpScRingBuffer::new();
        assert!(rb.pop().is_none());
    }

    #[test]
    fn capacity_is_power_of_two() {
        let rb: SpScRingBuffer<256> = SpScRingBuffer::new();
        assert_eq!(rb.capacity(), 256);
        assert_eq!(rb.capacity() & (rb.capacity() - 1), 0);
    }
}
