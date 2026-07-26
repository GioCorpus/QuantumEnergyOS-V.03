// ═══════════════════════════════════════════════════════════════════════════════
//  driver — Kernel driver layer: top-half (IRQ) + bottom-half (threaded IRQ)
// ═══════════════════════════════════════════════════════════════════════════════
//!
//! [Production Kernel Component]
//!
//! Top Half (Hard IRQ Handler):
//!   - MUST execute in < 1-5 microseconds
//!   - NO allocations
//!   - NO logging
//!   - NO computation
//!   - ONLY: acknowledge interrupt, trigger DMA, push metadata pointer,
//!           signal bottom half
//!
//! Bottom Half (Threaded IRQ / Deferred Processing):
//!   - Runs in high-priority kernel thread context
//!   - Parse raw telemetry frames
//!   - Validate checksum / integrity
//!   - Normalize sensor values
//!   - Apply calibration constants
//!   - Detect anomalies (lightweight checks)
//!   - NO blocking calls
//!
//! This is a userspace simulation of the kernel driver logic.
//! In production, the top half would be written in C with kernel headers,
//! and the bottom half would be a kernel thread (request_threaded_irq).
// ═══════════════════════════════════════════════════════════════════════════════

use crate::{
    config::{CalibrationConstant, SensorType, MAX_CALIBRATION_ENTRIES},
    dma::DmaBuffer,
    frame::{EnergyTelemetryFrame, flags, FRAME_SIZE},
    observability::TelemetryCounters,
    platform::*,
    ring_buffer::{PushResult, RingBuffer},
};
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicU32, Ordering};

/// Simulated MMIO hardware registers
/// In production: these are memory-mapped at a fixed physical address
#[repr(C)]
#[derive(Debug)]
pub struct HardwareRegisters {
    pub isr: UnsafeCell<u32>,
    pub imr: UnsafeCell<u32>,
    pub dma_sr: UnsafeCell<u32>,
    pub dma_cr: UnsafeCell<u32>,
    pub dma_src: u64,
    pub dma_dst: u64,
    pub dma_size: UnsafeCell<u32>,
    pub sensor_id: u32,
    pub timestamp_ns: UnsafeCell<u64>,
    pub frame_count: UnsafeCell<u32>,
    pub error_sr: UnsafeCell<u32>,
}

impl Clone for HardwareRegisters {
    fn clone(&self) -> Self {
        Self {
            isr: UnsafeCell::new(unsafe { *self.isr.get() }),
            imr: UnsafeCell::new(unsafe { *self.imr.get() }),
            dma_sr: UnsafeCell::new(unsafe { *self.dma_sr.get() }),
            dma_cr: UnsafeCell::new(unsafe { *self.dma_cr.get() }),
            dma_src: self.dma_src,
            dma_dst: self.dma_dst,
            dma_size: UnsafeCell::new(unsafe { *self.dma_size.get() }),
            sensor_id: self.sensor_id,
            timestamp_ns: UnsafeCell::new(unsafe { *self.timestamp_ns.get() }),
            frame_count: UnsafeCell::new(unsafe { *self.frame_count.get() }),
            error_sr: UnsafeCell::new(unsafe { *self.error_sr.get() }),
        }
    }
}

impl Default for HardwareRegisters {
    fn default() -> Self {
        Self {
            isr: UnsafeCell::new(0),
            imr: UnsafeCell::new(0),
            dma_sr: UnsafeCell::new(0),
            dma_cr: UnsafeCell::new(0),
            dma_src: 0,
            dma_dst: 0,
            dma_size: UnsafeCell::new(0),
            sensor_id: 0,
            timestamp_ns: UnsafeCell::new(0),
            frame_count: UnsafeCell::new(0),
            error_sr: UnsafeCell::new(0),
        }
    }
}

impl HardwareRegisters {
    /// Check if data-ready interrupt is pending
    #[inline(always)]
    pub fn data_ready(&self) -> bool {
        unsafe { (*self.isr.get() & 0x1) != 0 }
    }

    /// Check if DMA complete interrupt is pending
    #[inline(always)]
    pub fn dma_complete(&self) -> bool {
        unsafe { (*self.isr.get() & 0x2) != 0 }
    }

    /// Check if error interrupt is pending
    #[inline(always)]
    pub fn error(&self) -> bool {
        unsafe { (*self.isr.get() & 0x4) != 0 }
    }

    /// Acknowledge all interrupts
    #[inline(always)]
    pub fn ack_all(&self) {
        unsafe { *self.isr.get() = 0; }
    }

    /// Trigger DMA transfer
    #[inline(always)]
    pub fn trigger_dma(&self, size: u32) {
        unsafe {
            *self.dma_cr.get() |= 0x1; // Set DMA enable bit
            *self.dma_size.get() = size;
            *self.dma_sr.get() &= !0x1; // Clear DMA complete
        }
    }

    /// Check if DMA is complete
    #[inline(always)]
    pub fn dma_done(&self) -> bool {
        unsafe { (*self.dma_sr.get() & 0x1) != 0 }
    }
}

/// Interrupt context for top-half handler
/// Must not allocate, must not block
pub struct IrqContext<'a> {
    /// Hardware registers (read-only in IRQ)
    regs: &'a HardwareRegisters,
    /// DMA buffer
    dma: &'a DmaBuffer,
    /// Performance counters
    counters: &'a TelemetryCounters,
    /// Bottom-half scheduler flag
    bh_scheduled: &'a AtomicU32,
}

impl<'a> IrqContext<'a> {
    /// Create a new IRQ context
    #[inline]
    pub fn new(
        regs: &'a HardwareRegisters,
        dma: &'a DmaBuffer,
        counters: &'a TelemetryCounters,
        bh_scheduled: &'a AtomicU32,
    ) -> Self {
        Self {
            regs,
            dma,
            counters,
            bh_scheduled,
        }
    }

    /// Top-half IRQ handler — MUST be < 5 µs
    ///
    /// This is the fastest code path in the subsystem.
    /// No allocations, no logging, no syscalls.
    ///
    /// Returns true if bottom half was scheduled.
    #[inline(always)]
    pub fn handle_irq(&self) -> bool {
        // 1. Acknowledge interrupt (write to hardware register)
        // In real hardware: writel(1, regs + ISR_OFFSET)
        // Here we just count
        self.counters.irq_count.fetch_add(1, Ordering::Relaxed);

        // 2. Check interrupt source
        if unlikely(self.regs.error()) {
            // Error interrupt — log and acknowledge
            self.counters.error_count.fetch_add(1, Ordering::Relaxed);
            return false; // Don't schedule bottom half on error
        }

        if self.regs.data_ready() {
            // 3. Trigger DMA if not already running
            if !self.regs.dma_done() {
                self.trigger_dma_transfer();
            }

            // 4. Schedule bottom half (threaded IRQ)
            // In Linux: irq_wake_thread(irq, dev_id)
            // Here: set atomic flag
            self.schedule_bottom_half();
        }

        true
    }

    /// Trigger DMA transfer from sensor to memory
    #[inline(always)]
    fn trigger_dma_transfer(&self) {
        let dma_size = unsafe { *self.regs.dma_size.get() };
        let frame_count = (dma_size / FRAME_SIZE as u32).max(1) as u64;
        self.dma.dma_complete(frame_count as usize);
        self.counters
            .dma_complete_count
            .fetch_add(frame_count, Ordering::Relaxed);
    }

    /// Schedule the bottom half for processing
    #[inline(always)]
    fn schedule_bottom_half(&self) {
        let prev = self.bh_scheduled.swap(1, Ordering::AcqRel);
        if prev == 0 {
            // Wasn't already scheduled — wake up bottom half thread
            // In real kernel: irq_wake_thread() or tasklet_schedule()
            self.counters
                .bh_schedule_count
                .fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// Calibration table for sensors
/// Pre-loaded at probe time, read-only in hot path
pub struct CalibrationTable {
    entries: [Option<CalibrationConstant>; MAX_CALIBRATION_ENTRIES],
    count: AtomicU32,
}

impl Default for CalibrationTable {
    fn default() -> Self {
        Self {
            entries: [None; MAX_CALIBRATION_ENTRIES],
            count: AtomicU32::new(0),
        }
    }
}

impl CalibrationTable {
    /// Create a new empty calibration table
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Load calibration for a sensor (called at probe/init time)
    #[inline]
    pub fn load(&mut self, sensor_id: u32, calib: CalibrationConstant) -> bool {
        let idx = (sensor_id as usize) % MAX_CALIBRATION_ENTRIES;
        self.entries[idx] = Some(calib);
        self.count.fetch_add(1, Ordering::Relaxed);
        true
    }

    /// Get calibration for a sensor (read-only, no lock needed if pre-loaded)
    #[inline]
    pub fn get(&self, sensor_id: u32) -> Option<CalibrationConstant> {
        let idx = (sensor_id as usize) % MAX_CALIBRATION_ENTRIES;
        self.entries[idx]
    }

    /// Get loaded calibration count
    #[inline]
    pub fn len(&self) -> u32 {
        self.count.load(Ordering::Relaxed)
    }

    /// Check if table is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Bottom-half context (threaded IRQ)
/// Runs in high-priority kernel thread context
pub struct BottomHalfContext<'a> {
    /// DMA buffer containing raw frames
    dma: &'a DmaBuffer,
    /// Output ring buffer (to userspace)
    output_ring: &'a RingBuffer,
    /// Calibration table
    calibration: &'a CalibrationTable,
    /// Performance counters
    counters: &'a TelemetryCounters,
    /// Bottom-half scheduled flag
    bh_scheduled: &'a AtomicU32,
    /// Anomaly detection enabled
    anomaly_detection: bool,
    /// Running flag (for graceful shutdown)
    running: &'a AtomicU32,
}

impl<'a> BottomHalfContext<'a> {
    /// Create a new bottom-half context
    #[inline]
    pub fn new(
        dma: &'a DmaBuffer,
        output_ring: &'a RingBuffer,
        calibration: &'a CalibrationTable,
        counters: &'a TelemetryCounters,
        bh_scheduled: &'a AtomicU32,
        anomaly_detection: bool,
        running: &'a AtomicU32,
    ) -> Self {
        Self {
            dma,
            output_ring,
            calibration,
            counters,
            bh_scheduled,
            anomaly_detection,
            running,
        }
    }

    /// Process pending frames from DMA buffer
    ///
    /// This is the main work loop for the bottom-half thread.
    /// Processes as many frames as available, then sleeps.
    #[inline]
    pub fn run_once(&self) -> usize {
        if self.running.load(Ordering::Relaxed) == 0 {
            return 0;
        }

        // Clear scheduled flag
        self.bh_scheduled.store(0, Ordering::Release);

        let mut processed = 0;
        let max_batch = 256; // Process up to 256 frames per invocation

        for _ in 0..max_batch {
            // Pop raw frame from DMA buffer
            let mut frame = match self.dma.pop_frame() {
                Some(f) => f,
                None => break,
            };

            // Process frame
            self.process_frame(&mut frame);

            // Push to output ring buffer
            match self.output_ring.push(frame) {
                PushResult::Ok => {}
                PushResult::Dropped => {
                    self.counters
                        .dropped_count
                        .fetch_add(1, Ordering::Relaxed);
                }
                PushResult::Overwritten => {
                    self.counters
                        .overwritten_count
                        .fetch_add(1, Ordering::Relaxed);
                }
            }

            processed += 1;
        }

        self.counters
            .bh_frame_count
            .fetch_add(processed as u64, Ordering::Relaxed);

        processed as usize
    }

    /// Process a single frame (parse, validate, calibrate, detect anomalies)
    #[inline]
    fn process_frame(&self, frame: &mut EnergyTelemetryFrame) {
        // 1. Validate checksum
        if frame.has_flag(flags::FLAG_CHECKSUM_OK) {
            self.counters
                .checksum_ok_count
                .fetch_add(1, Ordering::Relaxed);
        } else {
            self.counters
                .checksum_fail_count
                .fetch_add(1, Ordering::Relaxed);
            // Invalid frame — skip calibration
            return;
        }

        // 2. Apply calibration
        if frame.has_flag(flags::FLAG_CALIBRATED) {
            // Already calibrated (shouldn't happen in bottom half, but handle gracefully)
        } else if let Some(calib) = self.calibration.get(frame.sensor_id) {
            calib.apply(frame);
        }

        // 3. Validate normalized values
        if frame.voltage_mv == 0 || frame.current_ma == 0 {
            frame.set_flag(flags::FLAG_ANOMALY);
            self.counters
                .anomaly_count
                .fetch_add(1, Ordering::Relaxed);
        }

        // 4. Lightweight anomaly detection
        if self.anomaly_detection {
            self.detect_anomalies(frame);
        }
    }

    /// Lightweight anomaly detection (no allocations, no blocking)
    #[inline]
    fn detect_anomalies(&self, frame: &mut EnergyTelemetryFrame) {
        let voltage_v = frame.voltage_v();
        let current_a = frame.current_a();
        let freq_hz = frame.frequency_hz();

        // Overvoltage detection (> 260V for 230V grid, > 150V for 120V grid)
        if voltage_v > 260.0 || voltage_v < 80.0 {
            frame.set_flag(flags::FLAG_OVERVOLTAGE);
            self.counters
                .anomaly_count
                .fetch_add(1, Ordering::Relaxed);
        }

        // Overcurrent detection (> 100A for typical residential, > 1000A industrial)
        if current_a > 1000.0 {
            frame.set_flag(flags::FLAG_OVERCURRENT);
            self.counters
                .anomaly_count
                .fetch_add(1, Ordering::Relaxed);
        }

        // Frequency anomaly (> ±2 Hz from nominal 50/60 Hz)
        let freq_dev = (freq_hz - 50.0).abs().min((freq_hz - 60.0).abs());
        if freq_dev > 2.0 {
            frame.set_flag(flags::FLAG_FREQ_ANOMALY);
            frame.set_flag(flags::FLAG_GRID_INSTABLE);
            self.counters
                .anomaly_count
                .fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// Complete driver context
/// Holds all state for the telemetry driver
pub struct DriverContext {
    /// Simulated hardware registers
    pub regs: HardwareRegisters,
    /// DMA buffer
    pub dma: DmaBuffer,
    /// Output ring buffer (to userspace)
    pub output_ring: RingBuffer,
    /// Calibration table
    pub calibration: CalibrationTable,
    /// Performance counters
    pub counters: TelemetryCounters,
    /// Bottom-half scheduled flag
    pub bh_scheduled: AtomicU32,
    /// Bottom-half running flag
    pub bh_running: AtomicU32,
    /// IRQ number (simulated)
    pub irq_number: u32,
    /// Sensor type
    pub sensor_type: SensorType,
    /// Anomaly detection enabled
    pub anomaly_detection: bool,
}

impl DriverContext {
    /// Create a new driver context
    #[inline]
    pub fn new(
        frame_capacity: usize,
        irq_number: u32,
        sensor_type: SensorType,
    ) -> Option<Self> {
        let dma = DmaBuffer::new(frame_capacity)?;
        let output_ring = RingBuffer::new(frame_capacity)?;

        Some(Self {
            regs: HardwareRegisters::default(),
            dma,
            output_ring,
            calibration: CalibrationTable::new(),
            counters: TelemetryCounters::new(),
            bh_scheduled: AtomicU32::new(0),
            bh_running: AtomicU32::new(0),
            irq_number,
            sensor_type,
            anomaly_detection: true,
        })
    }

    /// Start the bottom-half processing thread
    #[cfg(feature = "std")]
    #[inline]
    pub fn start_bottom_half(&self) {
        self.bh_running.store(1, Ordering::Release);
        // In real kernel: this would be a kernel thread
        // Here we just set the running flag
    }

    /// Stop the bottom-half processing thread
    #[inline]
    pub fn stop_bottom_half(&self) {
        self.bh_running.store(0, Ordering::Release);
        self.bh_scheduled.store(1, Ordering::Release); // Wake up one last time
    }

    /// Simulate an interrupt (for testing/benchmarking)
    #[inline]
    pub fn simulate_irq(&self, frame: EnergyTelemetryFrame) -> bool {
        unsafe {
            *self.regs.isr.get() = 0x1;
        }

        let irq_ctx = IrqContext::new(
            &self.regs,
            &self.dma,
            &self.counters,
            &self.bh_scheduled,
        );
        irq_ctx.handle_irq();

        self.dma.push_frame(frame);

        unsafe {
            *self.regs.dma_sr.get() = 0x1;
            *self.regs.isr.get() = 0x2;
        }

        let bh_ctx = BottomHalfContext::new(
            &self.dma,
            &self.output_ring,
            &self.calibration,
            &self.counters,
            &self.bh_scheduled,
            self.anomaly_detection,
            &self.bh_running,
        );
        let processed = bh_ctx.run_once();

        unsafe {
            *self.regs.isr.get() = 0;
        }

        processed > 0
    }

    /// Load calibration for a sensor
    #[inline]
    pub fn load_calibration(&mut self, sensor_id: u32, calib: CalibrationConstant) {
        self.calibration.load(sensor_id, calib);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_registers() {
        let regs = HardwareRegisters::default();
        unsafe { *regs.isr.get() = 0x1; }
        assert!(regs.data_ready());
        regs.ack_all();
        assert!(!regs.data_ready());
    }

    #[test]
    fn test_calibration_table() {
        let mut table = CalibrationTable::new();
        assert!(table.is_empty());

        let calib = CalibrationConstant::identity();
        assert!(table.load(42, calib));
        assert_eq!(table.len(), 1);

        let loaded = table.get(42);
        assert!(loaded.is_some());
    }

    #[test]
    fn test_driver_context() {
        let driver = DriverContext::new(1024, 16, SensorType::SPI).unwrap();
        driver.start_bottom_half();
        assert_eq!(driver.bh_running.load(Ordering::Relaxed), 1);

        let frame = EnergyTelemetryFrame::from_parts(1000, 1, 5000, 1000, 5995, flags::FLAG_CHECKSUM_OK);
        let result = driver.simulate_irq(frame);
        assert!(result);

        driver.stop_bottom_half();
    }
}
