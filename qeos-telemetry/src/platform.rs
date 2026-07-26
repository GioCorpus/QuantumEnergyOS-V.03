// ═══════════════════════════════════════════════════════════════════════════════
//  platform — Platform abstraction for cache lines, atomics, memory ordering
// ═══════════════════════════════════════════════════════════════════════════════

/// Cache line size in bytes for x86_64
pub const CACHE_LINE_SIZE: usize = 64;

/// Minimum alignment for cache-line aligned structures
pub const CACHE_LINE_ALIGN: usize = 64;

/// Memory ordering re-exports
pub use core::sync::atomic::Ordering;

#[cfg(not(feature = "std"))]
pub use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize};
#[cfg(feature = "std")]
pub use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize};

#[cfg(not(feature = "std"))]
pub use core::sync::atomic::compiler_fence;
#[cfg(feature = "std")]
pub use std::sync::atomic::compiler_fence;

/// Monotonic timestamp in nanoseconds
#[cfg(feature = "std")]
#[inline(always)]
pub fn timestamp_ns() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

#[cfg(not(feature = "std"))]
#[inline(always)]
pub fn timestamp_ns() -> u64 {
    0
}

/// Prefetch hint for cache optimization
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub fn prefetch_read<T>(ptr: *const T) {
    unsafe {
        core::arch::x86_64::_mm_prefetch(ptr as *const i8, 3);
    }
}

#[cfg(not(target_arch = "x86_64"))]
#[inline(always)]
pub fn prefetch_read<T>(_ptr: *const T) {}

/// Likely branch hint (compiler hint via cold/hot attributes)
#[inline(always)]
pub fn likely(b: bool) -> bool {
    b
}

/// Unlikely branch hint
#[inline(always)]
pub fn unlikely(b: bool) -> bool {
    b
}
