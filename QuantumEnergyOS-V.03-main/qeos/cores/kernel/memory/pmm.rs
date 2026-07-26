use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

/// Fixed-size frame allocator for physical memory.
/// [Research Prototype]
pub struct PhysicalMemoryManager {
    frames: Vec<AtomicU64>,
    total: usize,
    used: AtomicU64,
}

impl PhysicalMemoryManager {
    pub fn new(total_frames: usize) -> Self {
        let mut frames = Vec::with_capacity(total_frames);
        for _ in 0..total_frames {
            frames.push(AtomicU64::new(0));
        }
        Self {
            frames,
            total: total_frames,
            used: AtomicU64::new(0),
        }
    }

    pub fn allocate(&self) -> Option<usize> {
        for (i, frame) in self.frames.iter().enumerate() {
            if frame.compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed).is_ok() {
                self.used.fetch_add(1, Ordering::Relaxed);
                return Some(i);
            }
        }
        None
    }

    pub fn deallocate(&self, frame: usize) {
        if frame < self.total {
            self.frames[frame].store(0, Ordering::Release);
            self.used.fetch_sub(1, Ordering::Relaxed);
        }
    }

    pub fn usage(&self) -> (usize, usize) {
        (self.used.load(Ordering::Relaxed) as usize, self.total)
    }
}
