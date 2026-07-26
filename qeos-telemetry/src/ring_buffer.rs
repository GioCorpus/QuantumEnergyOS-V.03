// ═══════════════════════════════════════════════════════════════════════════════
//  ring_buffer — Lock-free SPSC ring buffer with cache-line alignment
// ═══════════════════════════════════════════════════════════════════════════════
//!
//! Design:
//!   - Single Producer / Single Consumer (SPSC)
//!   - Atomic memory model (Acquire/Release semantics)
//!   - Cache-line aligned head/tail (64-byte) to prevent false sharing
//!   - Pre-allocated, zero runtime allocation in hot path
//!   - O(1) insert and O(1) read
//!   - Zero mutex, zero spinlocks in fast path
//!
//! Memory layout:
//!   [Cache line 0] head (AtomicUsize) — written by producer
//!   [Cache line 1] tail (AtomicUsize) — written by consumer
//!   [Cache line 2+] data[] (Frame array)
//!
//! Policies:
//!   - OverwriteOldest: real-time mode, drop oldest on full
//!   - DropNewest: safe mode, reject write on full
// ═══════════════════════════════════════════════════════════════════════════════

use crate::{
    frame::EnergyTelemetryFrame,
    platform::*,
};
use core::{
    mem,
    ptr,
    sync::atomic::{AtomicUsize, Ordering},
};

/// Result of a push operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushResult {
    Ok,
    Dropped,
    Overwritten,
}

/// Ring buffer fill policy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillPolicy {
    /// Reject write when full (safe mode)
    DropNewest,
    /// Overwrite oldest entry (real-time mode)
    OverwriteOldest,
    /// Adaptive: drop until >95% utilization, then overwrite oldest
    BackpressureAware,
}

/// Lock-free Single-Producer Single-Consumer ring buffer
///
/// # Safety
///
/// Caller must ensure:
///   - Only one producer calls push() concurrently
///   - Only one consumer calls pop() concurrently
///   - Capacity is a power of 2
#[repr(C, align(64))]
pub struct RingBuffer {
    head: AtomicUsize,
    _pad0: [u8; CACHE_LINE_SIZE - mem::size_of::<AtomicUsize>()],
    tail: AtomicUsize,
    _pad1: [u8; CACHE_LINE_SIZE - mem::size_of::<AtomicUsize>()],
    capacity: usize,
    mask: usize,
    policy: FillPolicy,
    buf: *mut EnergyTelemetryFrame,
}

unsafe impl Send for RingBuffer {}
unsafe impl Sync for RingBuffer {}

impl RingBuffer {
    /// Create a new ring buffer with power-of-2 capacity
    #[inline]
    pub fn new(capacity: usize) -> Option<Self> {
        if capacity == 0 || !capacity.is_power_of_two() {
            return None;
        }
        let layout = core::alloc::Layout::from_size_align(
            capacity * mem::size_of::<EnergyTelemetryFrame>(),
            64,
        ).ok()?;

        #[cfg(feature = "std")]
        let raw = unsafe { std::alloc::alloc(layout) };
        #[cfg(not(feature = "std"))]
        let raw = unsafe { alloc::alloc::alloc(layout) };

        if raw.is_null() {
            return None;
        }

        Some(Self {
            head: AtomicUsize::new(0),
            _pad0: [0u8; CACHE_LINE_SIZE - mem::size_of::<AtomicUsize>()],
            tail: AtomicUsize::new(0),
            _pad1: [0u8; CACHE_LINE_SIZE - mem::size_of::<AtomicUsize>()],
            capacity,
            mask: capacity - 1,
            policy: FillPolicy::OverwriteOldest,
            buf: raw as *mut EnergyTelemetryFrame,
        })
    }

    /// Create with specific fill policy
    #[inline]
    pub fn with_policy(capacity: usize, policy: FillPolicy) -> Option<Self> {
        let mut rb = Self::new(capacity)?;
        rb.policy = policy;
        Some(rb)
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize { self.capacity }

    #[inline(always)]
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        head.wrapping_sub(tail)
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool { self.len() == 0 }

    #[inline(always)]
    pub fn is_full(&self) -> bool { self.len() >= self.capacity }

    /// Push a frame (HOT PATH)
    #[inline(always)]
    pub fn push(&self, frame: EnergyTelemetryFrame) -> PushResult {
        let head = self.head.fetch_add(1, Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        let count = head.wrapping_sub(tail);

        if count >= self.capacity {
            match self.policy {
                FillPolicy::DropNewest => {
                    self.head.fetch_sub(1, Ordering::Release);
                    return PushResult::Dropped;
                }
                FillPolicy::BackpressureAware => {
                    let util_pct = (count * 100) / self.capacity;
                    if util_pct >= 95 {
                        self.tail.store(tail.wrapping_add(1), Ordering::Release);
                        compiler_fence(Ordering::Release);
                    } else {
                        self.head.fetch_sub(1, Ordering::Release);
                        return PushResult::Dropped;
                    }
                }
                FillPolicy::OverwriteOldest => {
                    self.tail.store(tail.wrapping_add(1), Ordering::Release);
                    compiler_fence(Ordering::Release);
                }
            }
        }

        let idx = head & self.mask;
        unsafe {
            ptr::write(self.buf.add(idx), frame);
        }
        compiler_fence(Ordering::Release);

        if count >= self.capacity
            && matches!(
                self.policy,
                FillPolicy::OverwriteOldest | FillPolicy::BackpressureAware
            )
        {
            PushResult::Overwritten
        } else {
            PushResult::Ok
        }
    }

    /// Pop a frame (HOT PATH)
    #[inline(always)]
    pub fn pop(&self) -> Option<EnergyTelemetryFrame> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);
        if tail == head {
            return None;
        }
        let idx = tail & self.mask;
        let frame = unsafe { ptr::read(self.buf.add(idx)) };
        self.tail.store(tail.wrapping_add(1), Ordering::Release);
        Some(frame)
    }

    /// Peek at next frame without removing
    #[inline(always)]
    pub fn peek(&self) -> Option<&EnergyTelemetryFrame> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);
        if tail == head {
            return None;
        }
        let idx = tail & self.mask;
        unsafe { Some(&*self.buf.add(idx)) }
    }

    /// Pop multiple frames (batch read)
    #[inline]
    pub fn pop_batch(&self, out: &mut [EnergyTelemetryFrame]) -> usize {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);
        let available = head.wrapping_sub(tail);
        let count = available.min(out.len()).min(self.capacity);

        for i in 0..count {
            let idx = tail.wrapping_add(i) & self.mask;
            unsafe {
                out[i] = ptr::read(self.buf.add(idx));
            }
        }

        if count > 0 {
            self.tail.store(tail.wrapping_add(count), Ordering::Release);
        }
        count
    }

    /// Push multiple frames (batch write)
    #[inline]
    pub fn push_batch(&self, frames: &[EnergyTelemetryFrame]) -> (usize, usize) {
        let mut written = 0;
        let mut overwritten = 0;
        for frame in frames.iter() {
            match self.push(*frame) {
                PushResult::Ok => written += 1,
                PushResult::Dropped => {}
                PushResult::Overwritten => overwritten += 1,
            }
        }
        (written, overwritten)
    }

    /// Reset buffer
    #[inline]
    pub fn reset(&self) {
        let head = self.head.load(Ordering::Relaxed);
        self.tail.store(head, Ordering::Release);
    }

    /// Get buffer pointer
    #[inline]
    pub fn buffer_ptr(&self) -> *const EnergyTelemetryFrame {
        self.buf
    }

    /// Get mutable buffer pointer
    #[inline]
    pub fn buffer_ptr_mut(&self) -> *mut EnergyTelemetryFrame {
        self.buf
    }
}

impl Drop for RingBuffer {
    fn drop(&mut self) {
        if !self.buf.is_null() {
            let layout = core::alloc::Layout::from_size_align(
                self.capacity * mem::size_of::<EnergyTelemetryFrame>(),
                64,
            ).unwrap();
            #[cfg(feature = "std")]
            unsafe { std::alloc::dealloc(self.buf as *mut u8, layout); }
            #[cfg(not(feature = "std"))]
            unsafe { alloc::alloc::dealloc(self.buf as *mut u8, layout); }
            self.buf = ptr::null_mut();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation() {
        let rb = RingBuffer::new(64).unwrap();
        assert_eq!(rb.capacity(), 64);
        assert!(rb.is_empty());
    }

    #[test]
    fn test_push_pop() {
        let rb = RingBuffer::new(4).unwrap();
        let f = EnergyTelemetryFrame::from_parts(100, 1, 5000, 1000, 5995, 0);
        assert_eq!(rb.push(f), PushResult::Ok);
        assert_eq!(rb.len(), 1);
        let p = rb.pop().unwrap();
        assert_eq!(p.timestamp_ns, 100);
    }

    #[test]
    fn test_overwrite() {
        let rb = RingBuffer::with_policy(2, FillPolicy::OverwriteOldest).unwrap();
        for i in 0..4u64 {
            rb.push(EnergyTelemetryFrame::from_parts(i, i as u32, 100, 100, 5000, 0));
        }
        assert_eq!(rb.len(), 2);
        let p1 = rb.pop().unwrap();
        let p2 = rb.pop().unwrap();
        assert_eq!(p1.timestamp_ns, 2);
        assert_eq!(p2.timestamp_ns, 3);
    }

    #[test]
    fn test_drop_newest() {
        let rb = RingBuffer::with_policy(2, FillPolicy::DropNewest).unwrap();
        for i in 0..4u64 {
            let r = rb.push(EnergyTelemetryFrame::from_parts(i, 0, 100, 100, 5000, 0));
            if i < 2 { assert_eq!(r, PushResult::Ok); }
            else { assert_eq!(r, PushResult::Dropped); }
        }
        assert_eq!(rb.len(), 2);
    }

    #[test]
    fn test_batch() {
        let rb = RingBuffer::new(16).unwrap();
        let frames: Vec<_> = (0..8).map(|i| {
            EnergyTelemetryFrame::from_parts(i as u64, i as u32, 100, 100, 5000, 0)
        }).collect();
        let (w, _) = rb.push_batch(&frames);
        assert_eq!(w, 8);
        let mut out = vec![EnergyTelemetryFrame::new(); 8];
        assert_eq!(rb.pop_batch(&mut out), 8);
    }

    #[test]
    fn test_invalid_capacity() {
        assert!(RingBuffer::new(0).is_none());
        assert!(RingBuffer::new(3).is_none());
        assert!(RingBuffer::new(1).is_some());
    }
}
