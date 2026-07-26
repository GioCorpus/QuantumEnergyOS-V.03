#![warn(missing_docs)]

//! QuantumEnergyOS Kernel Memory subsystem
//!
//! # Modules
//!
//! - [`pmm`] — physical frame allocator
//! - [`vmm`] — virtual memory manager (identity mapping placeholder)
//!
//! # Classification
//!
//! [Research Prototype]

pub mod pmm;
pub mod vmm;

pub use pmm::PhysicalMemoryManager;
pub use vmm::{MemoryRegion, VirtualMemoryManager};
