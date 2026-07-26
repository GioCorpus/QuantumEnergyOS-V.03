// ═══════════════════════════════════════════════════════════════════════════════
//  userspace — Userspace orchestration: mmap reader, batcher, backpressure-aware consumer
// ═══════════════════════════════════════════════════════════════════════════════
//!
//! [Production Kernel Component]
//!
//! This module provides the userspace daemon components:
//!   - MmapTelemetryReader: zero-copy mmap access to DMA buffer
//!   - BatchConsumer: batch processing with backpressure awareness
//!   - TelemetryDaemon: main orchestration loop
//!
//! In production, the mmap region would be backed by a character device
//! (/dev/qeos-telemetry0) with vm_ops for mmap.
// ═══════════════════════════════════════════════════════════════════════════════

use crate::{
    backpressure::{BackpressureManager, BackpressureMode, BackpressureState, FrameDisposition},
    dma::{DmaBuffer, MmapRegion},
    frame::{EnergyTelemetryFrame, FRAME_SIZE},
    observability::TelemetryCounters,
    ring_buffer::RingBuffer,
};
use core::sync::atomic::{AtomicU64, Ordering};

/// Zero-copy reader for mmap'd DMA buffer
/// Reads frames directly from shared memory without copying
pub struct MmapTelemetryReader {
    mmap: MmapRegion,
    /// Current read position (frame index)
    read_pos: usize,
    /// Total frames consumed
    total_consumed: AtomicU64,
}

impl MmapTelemetryReader {
    /// Create a new mmap reader from a DMA buffer
    #[inline]
    pub fn new(dma: &DmaBuffer) -> Option<Self> {
        let mmap = dma.mmap()?;
        Some(Self {
            mmap,
            read_pos: 0,
            total_consumed: AtomicU64::new(0),
        })
    }

    /// Read a single frame from the mmap region (zero-copy)
    #[inline]
    pub fn read_frame(&mut self) -> Option<&EnergyTelemetryFrame> {
        unsafe { self.mmap.frame_at(self.read_pos) }.map(|f| {
            self.read_pos += 1;
            self.total_consumed.fetch_add(1, Ordering::Relaxed);
            f
        })
    }

    /// Read a batch of frames into a slice
    #[inline]
    pub fn read_batch(&mut self, out: &mut [EnergyTelemetryFrame]) -> usize {
        let count = out.len().min(
            (self.mmap.len() / FRAME_SIZE).saturating_sub(self.read_pos),
        );

        for i in 0..count {
            if let Some(frame) = unsafe { self.mmap.frame_at(self.read_pos + i) } {
                out[i] = *frame;
            } else {
                return i;
            }
        }

        self.read_pos += count;
        self.total_consumed.fetch_add(count as u64, Ordering::Relaxed);
        count
    }

    /// Reset read position
    #[inline]
    pub fn reset(&mut self) {
        self.read_pos = 0;
    }

    /// Get total frames consumed
    #[inline]
    pub fn total_consumed(&self) -> u64 {
        self.total_consumed.load(Ordering::Relaxed)
    }

    /// Check if we've reached the end of the buffer
    #[inline]
    pub fn is_eof(&self) -> bool {
        self.read_pos >= self.mmap.len() / FRAME_SIZE
    }
}

/// Batch consumer configuration
#[derive(Debug, Clone)]
pub struct BatchConsumerConfig {
    /// Target batch size
    pub batch_size: usize,
    /// Batch timeout in milliseconds
    pub batch_timeout_ms: u64,
    /// Backpressure mode
    pub backpressure_mode: BackpressureMode,
    /// Enable frame validation
    pub validate_frames: bool,
    /// Drop frames with invalid checksums
    pub drop_invalid: bool,
}

impl Default for BatchConsumerConfig {
    fn default() -> Self {
        Self {
            batch_size: 512,
            batch_timeout_ms: 10,
            backpressure_mode: BackpressureMode::Realtime,
            validate_frames: true,
            drop_invalid: true,
        }
    }
}

/// Batch consumer for telemetry frames
/// Accumulates frames into batches for efficient downstream processing
pub struct BatchConsumer {
    config: BatchConsumerConfig,
    queue: Vec<EnergyTelemetryFrame>,
    backpressure: BackpressureManager,
    counters: TelemetryCounters,
    /// Backpressure ring buffer (for overflow handling)
    _bp_ring: RingBuffer,
    /// Total batches processed
    batches_processed: AtomicU64,
}

impl BatchConsumer {
    /// Create a new batch consumer
    #[inline]
    pub fn new(config: BatchConsumerConfig) -> Self {
        let batch_size = config.batch_size;
        let bp_capacity = config.batch_size * 4;
        let backpressure = BackpressureManager::new(config.backpressure_mode);
        let bp_ring = RingBuffer::new(bp_capacity).unwrap();

        Self {
            config,
            queue: Vec::with_capacity(batch_size),
            backpressure,
            counters: TelemetryCounters::new(),
            _bp_ring: bp_ring,
            batches_processed: AtomicU64::new(0),
        }
    }

    /// Create a real-time batch consumer
    #[inline]
    pub fn realtime() -> Self {
        Self::new(BatchConsumerConfig {
            backpressure_mode: BackpressureMode::Realtime,
            ..Default::default()
        })
    }

    /// Create a scientific batch consumer
    #[inline]
    pub fn scientific() -> Self {
        Self::new(BatchConsumerConfig {
            backpressure_mode: BackpressureMode::Scientific,
            batch_size: 2048,
            ..Default::default()
        })
    }

    /// Submit a frame to the batch consumer
    #[inline]
    pub fn submit(&mut self, frame: EnergyTelemetryFrame) -> FrameDisposition {
        // Validate frame if configured
        if self.config.validate_frames && !frame.is_valid() {
            if self.config.drop_invalid {
                self.counters
                    .checksum_fail_count
                    .fetch_add(1, Ordering::Relaxed);
                return FrameDisposition::Dropped;
            }
        }

        // Check if batch is ready
        if self.queue.len() >= self.config.batch_size {
            self.flush_batch();
        }

        // Add to local queue
        self.queue.push(frame);
        FrameDisposition::Accepted
    }

    /// Flush current batch
    #[inline]
    pub fn flush_batch(&mut self) {
        if self.queue.is_empty() {
            return;
        }
        let count = self.queue.len();
        self.counters
            .frames_read
            .fetch_add(count as u64, Ordering::Relaxed);
        self.queue.clear();
        self.batches_processed.fetch_add(1, Ordering::Relaxed);
    }

    /// Check if a batch is ready
    #[inline]
    pub fn batch_ready(&self) -> bool {
        self.queue.len() >= self.config.batch_size
    }

    /// Get current queue depth
    #[inline]
    pub fn queue_depth(&self) -> usize {
        self.queue.len()
    }

    /// Get backpressure state
    #[inline]
    pub fn backpressure_state(&self) -> &BackpressureState {
        self.backpressure.state()
    }

    /// Get performance counters
    #[inline]
    pub fn counters(&self) -> &TelemetryCounters {
        &self.counters
    }

    /// Get total batches processed
    #[inline]
    pub fn batches_processed(&self) -> u64 {
        self.batches_processed.load(Ordering::Relaxed)
    }
}

/// Main telemetry daemon orchestration loop
#[cfg(feature = "std")]
pub struct TelemetryDaemon {
    dma: DmaBuffer,
    output_ring: RingBuffer,
    _consumer: BatchConsumer,
    running: bool,
    thread_handle: Option<std::thread::JoinHandle<()>>,
}

#[cfg(feature = "std")]
impl TelemetryDaemon {
    /// Create a new telemetry daemon
    #[inline]
    pub fn new(frame_capacity: usize, config: BatchConsumerConfig) -> Option<Self> {
        let dma = DmaBuffer::new(frame_capacity)?;
        let output_ring = RingBuffer::new(frame_capacity)?;
        let consumer = BatchConsumer::new(config);

        Some(Self {
            dma,
            output_ring,
            _consumer: consumer,
            running: false,
            thread_handle: None,
        })
    }

    /// Start the daemon (spawns background thread)
    #[inline]
    pub fn start(&mut self) {
        if self.running {
            return;
        }
        self.running = true;

        let _dma_ptr = self.dma.as_ptr();
        let _dma_len = self.dma.size();
        let _output_ring_ptr = self.output_ring.buffer_ptr();

        let handle = std::thread::spawn(move || {
            // Daemon loop
            loop {
                // In production: poll DMA buffer, process frames, push to output
                std::thread::sleep(std::time::Duration::from_micros(100));
            }
        });

        self.thread_handle = Some(handle);
    }

    /// Stop the daemon
    #[inline]
    pub fn stop(&mut self) {
        self.running = false;
        if let Some(handle) = self.thread_handle.take() {
            handle.join().unwrap();
        }
    }

    /// Check if daemon is running
    #[inline]
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Get reference to DMA buffer
    #[inline]
    pub fn dma(&self) -> &DmaBuffer {
        &self.dma
    }

    /// Get reference to output ring buffer
    #[inline]
    pub fn output_ring(&self) -> &RingBuffer {
        &self.output_ring
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::flags;

    #[test]
    fn test_batch_consumer_creation() {
        let consumer = BatchConsumer::realtime();
        assert_eq!(consumer.queue_depth(), 0);
    }

    #[test]
    fn test_batch_consumer_config() {
        let config = BatchConsumerConfig {
            batch_size: 256,
            backpressure_mode: BackpressureMode::Scientific,
            ..Default::default()
        };
        let consumer = BatchConsumer::new(config);
        assert_eq!(consumer.backpressure_state().mode, BackpressureMode::Scientific);
    }

    #[test]
    fn test_batch_consumer_submit() {
        let mut consumer = BatchConsumer::realtime();
        let frame = EnergyTelemetryFrame::from_parts(1000, 1, 5000, 1000, 5995, flags::FLAG_CHECKSUM_OK);
        let disp = consumer.submit(frame);
        assert_eq!(disp, FrameDisposition::Accepted);
        assert_eq!(consumer.queue_depth(), 1);
    }
}
