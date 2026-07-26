#![warn(missing_docs)]

//! QuantumEnergyOS Kernel Virtual Memory Manager
//!
//! 1:1 identity-mapped VMM placeholder. Future work: page tables, TLB shootdowns.
//!
//! # Classification
//!
//! [Research Prototype]

use std::collections::BTreeMap;

/// Virtual memory region descriptor.
#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    /// Start virtual address.
    pub virt_start: usize,
    /// Region size in bytes.
    pub size: usize,
    /// Start physical address.
    pub phys_start: usize,
}

impl MemoryRegion {
    /// End virtual address (exclusive).
    pub const fn virt_end(&self) -> usize {
        self.virt_start + self.size
    }
}

/// Simple 1:1 virtual memory manager.
///
/// Tracks identity-mapped regions. Does not handle page faults, swapping,
/// or protection bits.
pub struct VirtualMemoryManager {
    regions: BTreeMap<usize, MemoryRegion>,
    page_size: usize,
}

impl VirtualMemoryManager {
    /// Creates a new VMM with the given page size.
    pub fn new(page_size: usize) -> Self {
        Self {
            regions: BTreeMap::new(),
            page_size,
        }
    }

    /// Maps a contiguous physical region into virtual address space.
    ///
    /// Returns `Some(())` on success or `None` if the virtual range overlaps
    /// an existing mapping.
    pub fn map(&mut self, region: MemoryRegion) -> Option<()> {
        if self.regions.contains_key(&region.virt_start) {
            return None;
        }

        // Check for overlap with existing regions.
        for (_key, r) in self.regions.range(..=region.virt_start).rev() {
            if region.virt_start < r.virt_end() {
                return None;
            }
            break;
        }

        self.regions.insert(region.virt_start, region);
        Some(())
    }

    /// Translates a virtual address to its physical counterpart.
    ///
    /// Returns `Some(phys)` if `virt` falls inside a mapped region, else `None`.
    pub fn translate(&self, virt: usize) -> Option<usize> {
        for (_start, region) in self.regions.range(..=virt).rev() {
            if virt < region.virt_end() {
                let offset = virt - region.virt_start;
                return Some(region.phys_start + offset);
            }
        }
        None
    }

    /// Unmaps a region by virtual start address.
    ///
    /// Returns the removed region if it existed.
    pub fn unmap(&mut self, virt_start: usize) -> Option<MemoryRegion> {
        self.regions.remove(&virt_start)
    }

    /// Returns the number of mapped regions.
    pub fn region_count(&self) -> usize {
        self.regions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_and_translate() {
        let mut vmm = VirtualMemoryManager::new(4096);
        let region = MemoryRegion {
            virt_start: 0x1000_0000,
            size: 0x2000,
            phys_start: 0x4000_0000,
        };
        vmm.map(region).unwrap();
        assert_eq!(vmm.translate(0x1000_0000), Some(0x4000_0000));
        assert_eq!(vmm.translate(0x1000_1000), Some(0x4000_1000));
    }

    #[test]
    fn overlapping_map_fails() {
        let mut vmm = VirtualMemoryManager::new(4096);
        let r1 = MemoryRegion { virt_start: 0x1000, size: 0x2000, phys_start: 0 };
        let r2 = MemoryRegion { virt_start: 0x1800, size: 0x1000, phys_start: 0x1000 };
        vmm.map(r1).unwrap();
        assert!(vmm.map(r2).is_none());
    }

    #[test]
    fn translate_missing_returns_none() {
        let vmm = VirtualMemoryManager::new(4096);
        assert!(vmm.translate(0xDEADBEEF).is_none());
    }
}
