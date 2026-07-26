//! x86_64 MMU module

pub mod paging;
pub mod table;

pub use paging::PageTable;
pub use table::PageTableEntry;
