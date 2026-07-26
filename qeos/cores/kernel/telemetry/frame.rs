#![warn(missing_docs)]

//! QuantumEnergyOS High-Frequency Telemetry
//!
//! Provides the binary frame layout and shared-memory header for the
//! telemetry fast path (DMA / ISR context).
//!
//! # Classification
//!
//! [Research Prototype] — real-time safety requires hardware validation.

/// Binary frame status codes written by the producer (ISR / DMA).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameStatus {
    /// Frame is valid and complete.
    Valid = 0,
    /// Producer overwrote an unread frame.
    Overrun = 1,
    /// Consumer read beyond available frames.
    Underflow = 2,
    /// CRC or content validation failed.
    Corrupt = 3,
    /// Timestamp could not be synchronized.
    TimestampError = 4,
}

impl Default for FrameStatus {
    fn default() -> Self {
        Self::Valid
    }
}

impl core::fmt::Display for FrameStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Valid => write!(f, "valid"),
            Self::Overrun => write!(f, "overrun"),
            Self::Underflow => write!(f, "underflow"),
            Self::Corrupt => write!(f, "corrupt"),
            Self::TimestampError => write!(f, "timestamp_error"),
        }
    }
}

/// 64-byte aligned energy telemetry frame.
///
/// Layout (little-endian):
///
/// | Offset | Size | Field |
/// |--------|------|-------|
/// | 0 | 8 | `timestamp_ns` |
/// | 8 | 4 | `sensor_id` |
/// | 12 | 4 | `sequence` |
/// | 16 | 4 | `power_w` |
/// | 20 | 4 | `voltage_v` |
/// | 24 | 4 | `current_a` |
/// | 28 | 4 | `frequency_hz` |
/// | 32 | 4 | `energy_wh` |
/// | 36 | 1 | `status` |
/// | 37 | 27 | `padding` |
///
/// # Size invariant
///
/// The struct is `#[repr(C, align(64))]` and the compile-time assertion below
/// guarantees `size_of == 64`. Never modify field ordering without updating this table.
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy)]
pub struct EnergyTelemetryFrame {
    /// Timestamp in nanoseconds since boot (or Unix epoch — see driver).
    pub timestamp_ns: u64,
    /// Source sensor / device identifier.
    pub sensor_id: u32,
    /// Monotonically increasing sequence number.
    pub sequence: u32,
    /// Instantaneous active power [W].
    pub power_w: f32,
    /// RMS voltage [V].
    pub voltage_v: f32,
    /// RMS current [A].
    pub current_a: f32,
    /// Grid frequency [Hz].
    pub frequency_hz: f32,
    /// Accumulated energy [Wh].
    pub energy_wh: f32,
    /// Frame status from [`FrameStatus`].
    pub status: FrameStatus,
    /// Reserved / padding to reach 64 bytes.
    pub padding: [u8; 27],
}

const _: () = assert!(core::mem::size_of::<EnergyTelemetryFrame>() == 64);

impl EnergyTelemetryFrame {
    /// Creates a new empty frame for `sensor_id`.
    pub const fn new(sensor_id: u32) -> Self {
        Self {
            timestamp_ns: 0,
            sensor_id,
            sequence: 0,
            power_w: 0.0,
            voltage_v: 0.0,
            current_a: 0.0,
            frequency_hz: 0.0,
            energy_wh: 0.0,
            status: FrameStatus::Valid,
            padding: [0; 27],
        }
    }

    /// Returns true if the frame is marked valid.
    pub const fn valid(&self) -> bool {
        matches!(self.status, FrameStatus::Valid)
    }

    /// Sets the timestamp (builder pattern).
    pub const fn with_timestamp(mut self, ts: u64) -> Self {
        self.timestamp_ns = ts;
        self
    }

    /// Sets power, voltage, and current (builder pattern).
    pub const fn with_measurements(mut self, power: f32, voltage: f32, current: f32) -> Self {
        self.power_w = power;
        self.voltage_v = voltage;
        self.current_a = current;
        self
    }

    /// Sets frequency and accumulated energy.
    pub const fn with_energy(mut self, frequency_hz: f32, energy_wh: f32) -> Self {
        self.frequency_hz = frequency_hz;
        self.energy_wh = energy_wh;
        self
    }

    /// Returns the frame size in bytes.
    pub const fn frame_size() -> usize {
        core::mem::size_of::<Self>()
    }
}

/// Telemetry stream header placed at the start of a shared memory region.
///
/// Use this to sanity-check a mapped buffer before reading frames.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TelemetryHeader {
    /// Magic bytes (`b'Q','T','E','L'`) to identify the stream.
    pub magic: [u8; 4],
    /// Header / format version.
    pub version: u16,
    /// Total frames written since stream start.
    pub frame_count: u16,
    /// Sample rate [Hz].
    pub sample_rate_hz: u32,
    /// Feature flags from [`HeaderFlags`].
    pub flags: HeaderFlags,
    /// Reserved for future use.
    pub reserved: [u8; 42],
}

impl TelemetryHeader {
    /// Creates a new header.
    pub const fn new(sample_rate_hz: u32, flags: HeaderFlags) -> Self {
        Self {
            magic: *b"QTEL",
            version: 1,
            frame_count: 0,
            sample_rate_hz,
            flags,
            reserved: [0; 42],
        }
    }

    /// Returns true if the magic bytes are correct.
    pub fn is_valid(&self) -> bool {
        self.magic == *b"QTEL"
    }
}

/// Bit flags for [`TelemetryHeader`].
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct HeaderFlags(u16);

impl HeaderFlags {
    /// Buffer supports DMA zero-copy.
    pub const DMA_CAPABLE: u16 = 1 << 0;
    /// Buffer is shared memory (cross-process).
    pub const SHARED_MEM: u16 = 1 << 1;
    /// Stream is real-time (strict latency requirements).
    pub const REALTIME: u16 = 1 << 2;

    /// Creates flags from a raw word.
    pub const fn new(flags: u16) -> Self {
        Self(flags)
    }

    /// Returns true if DMA zero-copy is advertised.
    pub const fn is_dma_capable(&self) -> bool {
        (self.0 & Self::DMA_CAPABLE) != 0
    }

    /// Returns true if the stream uses shared memory.
    pub const fn is_shared_mem(&self) -> bool {
        (self.0 & Self::SHARED_MEM) != 0
    }

    /// Returns true if the stream is real-time.
    pub const fn is_realtime(&self) -> bool {
        (self.0 & Self::REALTIME) != 0
    }

    /// Returns the raw flag word.
    pub const fn bits(&self) -> u16 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_size_is_64() {
        assert_eq!(EnergyTelemetryFrame::frame_size(), 64);
    }

    #[test]
    fn header_magic() {
        let h = TelemetryHeader::new(1000, HeaderFlags::new(0));
        assert!(h.is_valid());
    }

    #[test]
    fn builder_pattern() {
        let frame = EnergyTelemetryFrame::new(42)
            .with_timestamp(1234)
            .with_measurements(100.0, 220.0, 0.5)
            .with_energy(60.0, 1.2);

        assert_eq!(frame.sensor_id, 42);
        assert_eq!(frame.timestamp_ns, 1234);
        assert_eq!(frame.power_w, 100.0);
        assert!(frame.valid());
    }
}
