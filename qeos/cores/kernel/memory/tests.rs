#![cfg(any(test, feature = "std"))]
use crate::memory::vmm::VirtualMemoryManager;

/// Memory manager smoke tests.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vmm_allocates_frames() {
        let mut vmm = VirtualMemoryManager::new(4096);
        let addr = vmm.allocate(4096).unwrap();
        assert!(addr > 0);
    }
}
