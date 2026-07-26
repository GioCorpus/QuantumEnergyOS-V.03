#![warn(missing_docs)]

//! QuantumEnergyOS Kernel Physical Memory Manager
//!
//! Fixed-size frame allocator backed by an array of atomic frame descriptors.
//! Each frame transitions between **free (0)** and **used (1)** atomically.
//!
//! # Classification
//!
//! [Research Prototype] — linear scan on allocate; no buddy/slab.

use std::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

/// Physical frame allocator for the kernel.
///
/// Frame indices correspond to physical page numbers.
/// The allocator does **not** track contiguous runs; use for single-page allocations.
pub struct PhysicalMemoryManager {
    free_list: Vec<usize>,
    total: usize,
    used: AtomicU64,
}

impl PhysicalMemoryManager {
    /// Creates a new PMM managing `total_frames` frames, initially all free.
    pub fn new(total_frames: usize) -> Self {
        let mut free_list = Vec::with_capacity(total_frames);
        for i in 0..total_frames {
            free_list.push(i);
        }

        Self {
            free_list,
            total: total_frames,
            used: AtomicU64::new(0),
        }
    }

    /// Allocates a single physical frame.
    ///
    /// # Returns
    ///
    /// `Some(frame_index)` on success, or `None` if no frames are available.
    ///
    /// # Complexity
    ///
    /// O(1) amortized (vector pop), but not cache-friendly for large frame counts.
    pub fn allocate(&self) -> Option<usize> {
        let frame = self.free_list.pop()?;
        self.used.fetch_add(1, Ordering::Relaxed);
        Some(frame)
    }

    /// Deallocates a single physical frame previously returned by `allocate`.
    ///
    /// # Safety
    ///
    /// The caller must ensure `frame` was allocated and not yet freed.
    pub fn deallocate(&self, frame: usize) {
        if frame < self.total {
            self.free_list.push(frame);
            self.used.fetch_sub(1, Ordering::Relaxed);
        }
    }

    /// Returns `(used, total)` frame counts.
    pub fn usage(&self) -> (usize, usize) {
        (
            self.used.load(Ordering::Relaxed) as usize,
            self.total,
        )
    }

    /// Returns the total frame count.
    pub const fn total_frames(&self) -> usize {
        self.total
    }

    /// Returns true if no frames are available.
    pub fn is_exhausted(&self) -> bool {
        self.free_list.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_and_free() {
        let pmm = PhysicalMemoryManager::new(16);
        let f = pmm.allocate().unwrap();
        assert!(f < 16);
        let (used, total) = pmm.usage();
        assert_eq!(used, 1);
        assert_eq!(total, 16);

        pmm.deallocate(f);
        let (used, _) = pmm.usage();
        assert_eq!(used, 0);
    }

    #[test]
    fn exhaustion() {
        let pmm = PhysicalMemoryManager::new(2);
        assert!(pmm.allocate().is_some());
        assert!(pmm.allocate().is_some());
        assert!(pmm.allocate().is_none());
    }

    #[test]
    fn out_of_range_dealloc_is_noop() {
        let pmm = PhysicalMemoryManager::new(4);
        pmm.deallocate(999);
        assert_eq!(pmm.usage().0, 0);
    }
}
