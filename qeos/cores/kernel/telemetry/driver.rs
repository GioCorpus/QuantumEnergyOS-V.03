#![warn(missing_docs)]

//! QuantumEnergyOS Telemetry Driver
//!
//! Interfaces between the hardware interrupt path and the SPSC ring buffer.
//! In production, the IRQ handler would be registered with the kernel's
//! interrupt controller; here it is exposed as a safe method on `TelemetryDriver`.
//!
//! # Classification
//!
//! [Research Prototype] — no actual DMA or interrupt wiring.

use super::frame::{EnergyTelemetryFrame, HeaderFlags, TelemetryHeader};
use super::spsc::{RingBufferError, SpScRingBuffer};

/// Interrupt line number assigned to the telemetry ADC / sensor hub.
///
/// # Note
///
/// This is a placeholder. Real IRQ numbers are platform-specific and must be
/// provided by the ACPI / Device Tree layer.
pub const TELEMETRY_IRQ_VECTOR: u8 = 42;

/// Suggested DMA buffer size (must be multiple of cache line and frame size).
///
/// `1024 frames * 64 bytes/frame = 64 KiB`.
pub const DMA_BUFFER_SIZE: usize = 1024;

/// Telemetry driver context.
///
/// Owns the ring buffer and sequence counter. In a real kernel this would
/// also contain interrupt enable/disable state and DMA descriptor pointers.
pub struct TelemetryDriver<const N: usize> {
    ring_buffer: SpScRingBuffer<N>,
    sequence: u32,
    header: TelemetryHeader,
}

impl<const N: usize> TelemetryDriver<N> {
    /// Creates a new telemetry driver with default header.
    pub fn new() -> Self {
        Self {
            ring_buffer: SpScRingBuffer::new(),
            sequence: 0,
            header: TelemetryHeader::new(0, HeaderFlags::new(0)),
        }
    }

    /// Creates a new driver with a specific sample rate and flags.
    pub fn with_config(sample_rate_hz: u32, flags: HeaderFlags) -> Self {
        Self {
            ring_buffer: SpScRingBuffer::new(),
            sequence: 0,
            header: TelemetryHeader::new(sample_rate_hz, flags),
        }
    }

    /// Records a new telemetry frame.
    ///
    /// This method is intended to be called from the hard-IRQ context or from
    /// the DMA completion callback. It is **safe** because `&mut self` guarantees
    /// exclusive access to the producer side of the ring buffer.
    ///
    /// # Returns
    ///
    /// - `Ok(())` on success.
    /// - `Err(RingBufferError::Overflow)` if the ring buffer is full.
    pub fn handle_irq(
        &mut self,
        sensor_id: u32,
        power_w: f32,
        voltage_v: f32,
        current_a: f32,
    ) -> Result<(), RingBufferError> {
        let mut frame = EnergyTelemetryFrame::new(sensor_id)
            .with_timestamp(0)
            .with_measurements(power_w, voltage_v, current_a);

        frame.sequence = self.sequence;
        self.sequence = self.sequence.wrapping_add(1);

        self.ring_buffer.push(frame)
    }

    /// Pops a frame for userspace consumption.
    ///
    /// Returns `None` if no frames are available.
    pub fn pop_frame(&mut self) -> Option<EnergyTelemetryFrame> {
        self.ring_buffer.pop()
    }

    /// Returns the number of frames currently in the ring buffer.
    pub fn len(&self) -> usize {
        self.ring_buffer.len()
    }

    /// Returns true if the ring buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.ring_buffer.is_empty()
    }

    /// Returns the capacity of the ring buffer.
    pub const fn capacity(&self) -> usize {
        self.ring_buffer.capacity()
    }

    /// Borrows the telemetry header.
    pub fn header(&self) -> &TelemetryHeader {
        &self.header
    }
}

/// DMA buffer descriptor for zero-copy shared memory.
///
/// `DmaBuffer` describes a physically contiguous, cache-line aligned memory
/// region suitable for DMA. The actual allocation is platform-specific and
/// must be performed by the driver before the buffer is passed to hardware.
///
/// # Safety
///
/// The caller must ensure:
/// - `ptr` points to a valid, aligned allocation for the lifetime of `self`.
/// - The memory region is not accessed concurrently while DMA is active.
/// - On drop, the memory is freed via the platform allocator (not currently automated).
#[derive(Debug, Clone, Copy)]
pub struct DmaBuffer {
    /// Raw pointer to the DMA-capable memory region.
    ///
    /// # Invariant
    ///
    /// If `is_dma_compatible()` is true, `ptr` is non-null and 64-byte aligned.
    pub ptr: *mut u8,
    /// Length of the buffer in bytes (always 64-byte aligned).
    pub len: usize,
    /// Alignment flag — true when `ptr` is 64-byte aligned.
    pub aligned: bool,
}

impl DmaBuffer {
    /// Creates a new `DmaBuffer` descriptor with aligned length.
    ///
    /// The pointer is initialized to null. Call `set_ptr` before use.
    pub fn new(len: usize) -> Self {
        let aligned_len = (len + 63) & !63;

        Self {
            ptr: core::ptr::null_mut(),
            len: aligned_len,
            aligned: true,
        }
    }

    /// Sets the backing pointer.
    ///
    /// # Safety
    ///
    /// The caller must guarantee `ptr` is valid for `len` bytes and 64-byte aligned.
    pub unsafe fn set_ptr(&mut self, ptr: *mut u8) {
        self.ptr = ptr;
        self.aligned = !ptr.is_null();
    }

    /// Returns true if the buffer is DMA-compatible (non-null + aligned).
    pub fn is_dma_compatible(&self) -> bool {
        self.aligned && !self.ptr.is_null()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn driver_roundtrip() {
        let mut driver: TelemetryDriver<16> = TelemetryDriver::new();
        driver.handle_irq(1, 100.0, 220.0, 0.5).unwrap();
        assert_eq!(driver.len(), 1);

        let frame = driver.pop_frame().unwrap();
        assert_eq!(frame.sensor_id, 1);
        assert_eq!(frame.power_w, 100.0);
    }

    #[test]
    fn dma_buffer_alignment() {
        let buf = DmaBuffer::new(100);
        assert_eq!(buf.len % 64, 0);
        assert!(!buf.is_dma_compatible());

        let mut buf = buf;
        let mut mem = [0u8; 128];
        unsafe {
            buf.set_ptr(mem.as_mut_ptr());
        }
        assert!(buf.is_dma_compatible());
    }
}
