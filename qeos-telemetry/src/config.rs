// ═══════════════════════════════════════════════════════════════════════════════
//  config — Telemetry subsystem configuration
// ═══════════════════════════════════════════════════════════════════════════════

use crate::{
    backpressure::BackpressureMode,
    frame::EnergyTelemetryFrame,
};

/// Default ring buffer capacity (power of 2 for efficient masking)
pub const DEFAULT_RING_BUFFER_CAPACITY: usize = 65536;

/// Default DMA buffer size (must be multiple of page size)
pub const DEFAULT_DMA_BUFFER_SIZE: usize = 4 * 1024 * 1024; // 4 MB

/// Default IRQ affinity CPU (for threaded IRQ)
pub const DEFAULT_IRQ_CPU: u32 = 0;

/// Maximum sensor ID count
pub const MAX_SENSORS: usize = 1024;

/// Calibration table entry count
pub const MAX_CALIBRATION_ENTRIES: usize = 4096;

/// Telemetry subsystem configuration
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    /// Ring buffer capacity (number of frames)
    pub ring_buffer_capacity: usize,
    /// DMA buffer size in bytes
    pub dma_buffer_size: usize,
    /// Backpressure handling mode
    pub backpressure_mode: BackpressureMode,
    /// CPU affinity for threaded IRQ (-1 = no affinity)
    pub irq_cpu: i32,
    /// Enable lightweight anomaly detection in bottom half
    pub enable_anomaly_detection: bool,
    /// Enable performance counters
    pub enable_counters: bool,
    /// Maximum batch size for userspace consumption
    pub max_batch_size: usize,
    /// Consumer timeout in milliseconds
    pub consumer_timeout_ms: u64,
    /// Sensor calibration file path (if any)
    pub calibration_path: Option<&'static str>,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            ring_buffer_capacity: DEFAULT_RING_BUFFER_CAPACITY,
            dma_buffer_size: DEFAULT_DMA_BUFFER_SIZE,
            backpressure_mode: BackpressureMode::Realtime,
            irq_cpu: DEFAULT_IRQ_CPU as i32,
            enable_anomaly_detection: true,
            enable_counters: true,
            max_batch_size: 1024,
            consumer_timeout_ms: 1000,
            calibration_path: None,
        }
    }
}

impl TelemetryConfig {
    /// Create a real-time mode configuration (overwrite oldest on full)
    #[inline]
    pub fn realtime() -> Self {
        Self {
            backpressure_mode: BackpressureMode::Realtime,
            ..Self::default()
        }
    }

    /// Create a scientific mode configuration (preserve all data, batch)
    #[inline]
    pub fn scientific() -> Self {
        Self {
            backpressure_mode: BackpressureMode::Scientific,
            ring_buffer_capacity: 262144, // 256K frames
            ..Self::default()
        }
    }

    /// Create an emergency mode configuration (disable IRQ, poll)
    #[inline]
    pub fn emergency() -> Self {
        Self {
            backpressure_mode: BackpressureMode::Emergency,
            enable_anomaly_detection: false,
            ..Self::default()
        }
    }
}

/// Calibration constant for a sensor
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CalibrationConstant {
    /// Voltage offset (mV)
    pub voltage_offset_mv: i32,
    /// Voltage scale factor (fixed-point Q16.16)
    pub voltage_scale: i32,
    /// Current offset (mA)
    pub current_offset_ma: i32,
    /// Current scale factor (fixed-point Q16.16)
    pub current_scale: i32,
    /// Frequency offset (Hz × 100)
    pub freq_offset_hz_x100: i16,
    /// Timestamp of last calibration (nanoseconds)
    pub calib_timestamp_ns: u64,
}

impl CalibrationConstant {
    /// Identity calibration (no adjustment)
    pub const fn identity() -> Self {
        Self {
            voltage_offset_mv: 0,
            voltage_scale: 0x00010000, // 1.0 in Q16.16
            current_offset_ma: 0,
            current_scale: 0x00010000,
            freq_offset_hz_x100: 0,
            calib_timestamp_ns: 0,
        }
    }

    /// Apply calibration to a frame in-place
    #[inline]
    pub fn apply(&self, frame: &mut EnergyTelemetryFrame) {
        // Voltage: (raw + offset) * scale
        let v_raw = frame.voltage_mv as i32;
        let v_cal = ((v_raw + self.voltage_offset_mv) as i64
            * (self.voltage_scale as i64 >> 16)) as i32;
        frame.voltage_mv = v_cal.max(0) as u32;

        // Current: (raw + offset) * scale
        let i_raw = frame.current_ma as i32;
        let i_cal = ((i_raw + self.current_offset_ma) as i64
            * (self.current_scale as i64 >> 16)) as i32;
        frame.current_ma = i_cal.max(0) as u32;

        // Frequency: raw + offset
        let f_raw = frame.frequency_hz_x100 as i16;
        frame.frequency_hz_x100 = (f_raw + self.freq_offset_hz_x100) as u16;

        frame.set_flag(crate::frame::flags::FLAG_CALIBRATED);
    }
}

/// Sensor type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorType {
    /// I2C temperature/voltage sensor
    I2C,
    /// SPI telemetry chip
    SPI,
    /// PCIe energy monitoring device
    PCIe,
    /// PMIC voltage/current sensor
    PMIC,
    /// Custom FPGA telemetry unit
    FPGA,
    /// Unknown/proprietary
    Unknown,
}

impl SensorType {
    /// Determine sensor type from sensor_id prefix
    #[inline]
    pub fn from_id(sensor_id: u32) -> Self {
        match (sensor_id >> 24) & 0xFF {
            0x01 => Self::I2C,
            0x02 => Self::SPI,
            0x03 => Self::PCIe,
            0x04 => Self::PMIC,
            0x05 => Self::FPGA,
            _ => Self::Unknown,
        }
    }

    /// Estimated sampling rate for sensor type (Hz)
    #[inline]
    pub fn typical_rate_hz(&self) -> u32 {
        match self {
            Self::I2C => 100,
            Self::SPI => 10_000,
            Self::PCIe => 1_000_000,
            Self::PMIC => 10_000,
            Self::FPGA => 10_000_000,
            Self::Unknown => 1_000,
        }
    }
}
