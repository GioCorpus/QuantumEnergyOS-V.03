use super::frame::EnergyTelemetryFrame;
use super::spsc::{SpScRingBuffer, RingBufferError};

pub const TELEMETRY_IRQ: u8 = 42;
pub const DMA_BUFFER_SIZE: usize = 1024;

pub struct TelemetryDriver<const N: usize> {
    ring_buffer: SpScRingBuffer<N>,
    sequence: u32,
}

impl<const N: usize> TelemetryDriver<N> {
    pub const fn new() -> Self {
        Self {
            ring_buffer: SpScRingBuffer::new(),
            sequence: 0,
        }
    }

    #[inline]
    pub unsafe fn handle_irq(&mut self, sensor_id: u32, power_w: f32, voltage_v: f32) -> Result<(), RingBufferError> {
        let frame = EnergyTelemetryFrame::new(sensor_id)
            .with_measurements(power_w, voltage_v, 0.0);
        
        self.ring_buffer.push(frame)
    }

    pub fn pop_frame(&mut self) -> Option<EnergyTelemetryFrame> {
        unsafe { self.ring_buffer.pop() }
    }

    pub fn len(&self) -> usize {
        self.ring_buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ring_buffer.is_empty()
    }
}

pub struct DmaBuffer {
    pub ptr: *mut u8,
    pub len: usize,
    pub aligned: bool,
}

impl DmaBuffer {
    pub fn new(len: usize) -> Self {
        let aligned_len = (len + 63) & !63;
        
        Self {
            ptr: core::ptr::null_mut(),
            len: aligned_len,
            aligned: true,
        }
    }

    pub fn is_dma_compatible(&self) -> bool {
        self.aligned && !self.ptr.is_null()
    }
}