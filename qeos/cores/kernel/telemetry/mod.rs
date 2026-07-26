#![warn(missing_docs)]

//! QuantumEnergyOS Kernel Telemetry
//!
//! High-frequency energy telemetry with lock-free SPSC ring buffer,
//! DMA-capable frame layout, and configurable backpressure policies.
//!
//! # Modules
//!
//! - [`frame`] — 64-byte binary frame layout
//! - [`spsc`] — lock-free single-producer / single-consumer ring buffer
//! - [`driver`] — telemetry driver with IRQ interface
//! - [`backpressure`] — overflow policies (RealTime, Scientific, Emergency)
//!
//! # Classification
//!
//! [Research Prototype] — real-time safety requires hardware validation.

pub mod frame;
pub mod spsc;
pub mod driver;
pub mod backpressure;

pub use frame::{EnergyTelemetryFrame, FrameStatus, HeaderFlags, TelemetryHeader};
pub use spsc::{RingBufferError, SpScRingBuffer, CACHE_LINE_SIZE};
pub use driver::{DmaBuffer, TelemetryDriver, DMA_BUFFER_SIZE, TELEMETRY_IRQ_VECTOR};
pub use backpressure::{BackpressureMode, BackpressurePolicy};
