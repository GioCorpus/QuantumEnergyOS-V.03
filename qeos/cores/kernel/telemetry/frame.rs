#[repr(C, align(64))]
#[derive(Debug, Clone, Copy)]
pub struct EnergyTelemetryFrame {
    pub timestamp_ns: u64,
    pub sensor_id: u32,
    pub sequence: u32,
    pub power_w: f32,
    pub voltage_v: f32,
    pub current_a: f32,
    pub frequency_hz: f32,
    pub energy_wh: f32,
    pub status: FrameStatus,
    pub padding: [u8; 28],
}

const _: () = assert!(core::mem::size_of::<EnergyTelemetryFrame>() == 64);

impl EnergyTelemetryFrame {
    pub fn new(sensor_id: u32) -> Self {
        Self {
            timestamp_ns: 0,
            sensor_id,
            sequence: 0,
            power_w: 0.0,
            voltage_v: 0.0,
            current_a: 0.0,
            frequency_hz: 0.0,
            energy_wh: 0.0,
            status: FrameStatus::default(),
            padding: [0; 28],
        }
    }

    pub fn valid(&self) -> bool {
        self.status == FrameStatus::VALID
    }

    pub fn with_timestamp(mut self, ts: u64) -> Self {
        self.timestamp_ns = ts;
        self
    }

    pub fn with_measurements(mut self, power: f32, voltage: f32, current: f32) -> Self {
        self.power_w = power;
        self.voltage_v = voltage;
        self.current_a = current;
        self
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameStatus {
    VALID = 0,
    OVERRUN = 1,
    UNDERFLOW = 2,
    CORRUPT = 3,
    TIMESTAMP_ERROR = 4,
}

impl Default for FrameStatus {
    fn default() -> Self {
        FrameStatus::VALID
    }
}

#[derive(Debug)]
pub struct TelemetryHeader {
    pub magic: [u8; 4],
    pub version: u16,
    pub frame_count: u16,
    pub sample_rate_hz: u32,
    pub flags: HeaderFlags,
    pub reserved: [u8; 42],
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct HeaderFlags(u16);

impl HeaderFlags {
    pub const DMA_CAPABLE: u16 = 1 << 0;
    pub const SHARED_MEM: u16 = 1 << 1;
    pub const REALTIME: u16 = 1 << 2;

    pub fn new(flags: u16) -> Self {
        Self(flags)
    }

    pub fn is_dma_capable(&self) -> bool {
        (self.0 & Self::DMA_CAPABLE) != 0
    }
}

#[repr(C)]
pub struct SharedRingBuffer {
    pub header: TelemetryHeader,
    pub read_index: core::sync::atomic::AtomicU64,
    pub write_index: core::sync::atomic::AtomicU64,
    pub buffer: [EnergyTelemetryFrame; 0],
}

impl SharedRingBuffer {
    pub fn buffer_ptr(&self) -> *const EnergyTelemetryFrame {
        self.buffer.as_ptr()
    }

    pub fn buffer_ptr_mut(&mut self) -> *mut EnergyTelemetryFrame {
        self.buffer.as_mut_ptr()
    }
}
