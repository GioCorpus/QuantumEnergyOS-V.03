use super::pmm::PhysicalMemoryManager;
use alloc::collections::BTreeMap;
use core::ops::Range;

/// Simple 1:1 virtual memory manager (identity mapping placeholder).
/// [Research Prototype]
pub struct VirtualMemoryManager {
    regions: BTreeMap<usize, Range<usize>>,
    page_size: usize,
}

impl VirtualMemoryManager {
    pub fn new(page_size: usize) -> Self {
        Self {
            regions: BTreeMap::new(),
            page_size,
        }
    }

    pub fn map(&mut self, phys: usize, virt: usize, count: usize) -> Option<()> {
        if self.regions.contains_key(&virt) {
            return None;
        }
        self.regions.insert(virt, virt..virt + count * self.page_size);
        Some(())
    }

    pub fn translate(&self, virt: usize) -> Option<usize> {
        for (start, range) in self.regions.range(..=virt).rev() {
            if virt < range.end {
                return Some(virt - start + range.start);
            }
        }
        None
    }
}
