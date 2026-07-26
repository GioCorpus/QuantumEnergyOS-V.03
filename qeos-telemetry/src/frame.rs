// ═══════════════════════════════════════════════════════════════════════════════
//  frame — Fixed-width telemetry frame format (C-compatible, cache-line friendly)
// ═══════════════════════════════════════════════════════════════════════════════
//!
//! All frames are fixed-size, C-compatible, 64-bit aligned.
//! Total size: 24 bytes (cache-line friendly for batch processing).
//!
//! Memory layout:
//!   [0..8]   timestamp_ns: u64      — Monotonic nanosecond timestamp
//!   [8..12]  sensor_id: u32         — Unique sensor identifier
//!   [12..16] voltage_mv: u32        — Voltage in millivolts
//!   [16..20] current_ma: u32        — Current in milliamps
//!   [20..22] frequency_hz_x100: u16 — Frequency × 100 (e.g., 5995 = 59.95 Hz)
//!   [22..24] flags: u16             — Status/error flags
// ═══════════════════════════════════════════════════════════════════════════════



/// Size of EnergyTelemetryFrame in bytes
pub const FRAME_SIZE: usize = 24;

/// Frame flags (bitmask)
pub mod flags {
    pub const FLAG_OVERVOLTAGE:    u16 = 1 << 0;   // Voltage exceeds safe threshold
    pub const FLAG_OVERCURRENT:   u16 = 1 << 1;   // Current exceeds safe threshold
    pub const FLAG_FREQ_ANOMALY:  u16 = 1 << 2;   // Frequency deviation detected
    pub const FLAG_CALIBRATED:    u16 = 1 << 3;   // Frame has been calibrated
    pub const FLAG_IRQ_DRIVEN:    u16 = 1 << 4;   // Delivered via interrupt path
    pub const FLAG_DMA_VALID:     u16 = 1 << 5;   // DMA transfer completed successfully
    pub const FLAG_CHECKSUM_OK:   u16 = 1 << 6;   // Frame checksum verified
    pub const FLAG_ANOMALY:       u16 = 1 << 7;   // Lightweight anomaly detected
    pub const FLAG_GRID_INSTABLE: u16 = 1 << 8;   // Grid instability indicator
    pub const FLAG_PRIORITY_HIGH: u16 = 1 << 9;   // High-priority frame (emergency)
}

/// Fixed-width energy telemetry frame
/// C-compatible layout, 64-bit aligned, cache-line friendly
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EnergyTelemetryFrame {
    /// Monotonic nanosecond timestamp (from kernel clock)
    pub timestamp_ns: u64,
    /// Unique sensor identifier
    pub sensor_id: u32,
    /// Voltage in millivolts
    pub voltage_mv: u32,
    /// Current in milliamps
    pub current_ma: u32,
    /// Frequency × 100 (e.g., 5995 = 59.95 Hz)
    pub frequency_hz_x100: u16,
    /// Status flags (bitmask)
    pub flags: u16,
}

impl EnergyTelemetryFrame {
    /// Create a new empty/default frame
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            timestamp_ns: 0,
            sensor_id: 0,
            voltage_mv: 0,
            current_ma: 0,
            frequency_hz_x100: 0,
            flags: 0,
        }
    }

    /// Create a frame with all fields specified
    #[inline(always)]
    pub const fn from_parts(
        timestamp_ns: u64,
        sensor_id: u32,
        voltage_mv: u32,
        current_ma: u32,
        frequency_hz_x100: u16,
        flags: u16,
    ) -> Self {
        Self {
            timestamp_ns,
            sensor_id,
            voltage_mv,
            current_ma,
            frequency_hz_x100,
            flags,
        }
    }

    /// Check if a specific flag is set
    #[inline(always)]
    pub fn has_flag(&self, flag: u16) -> bool {
        (self.flags & flag) != 0
    }

    /// Set a flag
    #[inline(always)]
    pub fn set_flag(&mut self, flag: u16) {
        self.flags |= flag;
    }

    /// Clear a flag
    #[inline(always)]
    pub fn clear_flag(&mut self, flag: u16) {
        self.flags &= !flag;
    }

    /// Voltage in volts (float)
    #[inline(always)]
    pub fn voltage_v(&self) -> f32 {
        self.voltage_mv as f32 / 1000.0
    }

    /// Current in amps (float)
    #[inline(always)]
    pub fn current_a(&self) -> f32 {
        self.current_ma as f32 / 1000.0
    }

    /// Frequency in Hz (float)
    #[inline(always)]
    pub fn frequency_hz(&self) -> f32 {
        self.frequency_hz_x100 as f32 / 100.0
    }

    /// Power in watts (P = V × I)
    #[inline(always)]
    pub fn power_w(&self) -> f32 {
        self.voltage_v() * self.current_a()
    }

    /// Apparent power in VA (for AC)
    #[inline(always)]
    pub fn apparent_power_va(&self) -> f32 {
        self.voltage_v() * self.current_a()
    }

    /// Reactive power estimate (simplified)
    #[inline(always)]
    pub fn reactive_power_var(&self) -> f32 {
        let freq_deviation = (self.frequency_hz() - 50.0).abs();
        self.voltage_v() * self.current_a() * (freq_deviation / 10.0)
    }

    /// Compute simple checksum over frame fields
    #[inline(always)]
    pub fn checksum(&self) -> u16 {
        let data: &[u8; FRAME_SIZE] = unsafe { core::mem::transmute(self) };
        let mut sum: u16 = 0;
        for &byte in data.iter() {
            sum = sum.wrapping_add(byte as u16);
        }
        !sum  // One's complement
    }

    /// Validate frame integrity
    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        self.timestamp_ns > 0
            && self.sensor_id > 0
            && self.voltage_mv > 0
            && self.current_ma > 0
            && self.has_flag(flags::FLAG_CHECKSUM_OK)
    }

    /// Zero-copy read from raw pointer (unsafe)
    #[inline(always)]
    pub unsafe fn from_ptr(ptr: *const u8) -> &'static Self {
        &*(ptr as *const Self)
    }

    /// Zero-copy write to raw pointer (unsafe)
    #[inline(always)]
    pub unsafe fn write_to_ptr(&self, ptr: *mut u8) {
        let dst = ptr as *mut Self;
        *dst = *self;
    }
}

impl core::fmt::Display for EnergyTelemetryFrame {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let ts = self.timestamp_ns;
        let sid = self.sensor_id;
        let v = self.voltage_mv;
        let c = self.current_ma;
        let freq = self.frequency_hz_x100;
        let fl = self.flags;
        write!(
            f,
            "EnergyTelemetryFrame {{ ts={}ns sensor={} V={}mV I={}mA f={}Hz flags={:04x} }}",
            ts, sid, v, c, freq as f32 / 100.0, fl
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_size() {
        assert_eq!(core::mem::size_of::<EnergyTelemetryFrame>(), FRAME_SIZE);
    }

    #[test]
    fn test_frame_default() {
        let f = EnergyTelemetryFrame::new();
        assert_eq!(f.timestamp_ns, 0);
        assert_eq!(f.sensor_id, 0);
    }

    #[test]
    fn test_frame_flags() {
        let mut f = EnergyTelemetryFrame::new();
        assert!(!f.has_flag(flags::FLAG_OVERVOLTAGE));
        f.set_flag(flags::FLAG_OVERVOLTAGE);
        assert!(f.has_flag(flags::FLAG_OVERVOLTAGE));
        f.clear_flag(flags::FLAG_OVERVOLTAGE);
        assert!(!f.has_flag(flags::FLAG_OVERVOLTAGE));
    }

    #[test]
    fn test_frame_calculations() {
        let mut f = EnergyTelemetryFrame::new();
        f.voltage_mv = 120_000; // 120V
        f.current_ma = 10_000;  // 10A
        f.frequency_hz_x100 = 5995; // 59.95 Hz
        assert!((f.voltage_v() - 120.0).abs() < 0.01);
        assert!((f.current_a() - 10.0).abs() < 0.01);
        assert!((f.power_w() - 1200.0).abs() < 0.01);
    }

    #[test]
    fn test_frame_from_parts() {
        let f = EnergyTelemetryFrame::from_parts(1000, 42, 5000, 1000, 5995, flags::FLAG_CALIBRATED);
        assert_eq!(f.timestamp_ns, 1000);
        assert_eq!(f.sensor_id, 42);
        assert!(f.has_flag(flags::FLAG_CALIBRATED));
    }
}
