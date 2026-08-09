// ═══════════════════════════════════════════════════════════════════════════════
//  qeos-telemetry — Real-time Energy Telemetry Subsystem
// ═══════════════════════════════════════════════════════════════════════════════
//!
//! Production-grade telemetry ingestion pipeline for QuantumEnergyOS.
//!
//! Architecture layers:
//!   1. Hardware Layer       — I2C, SPI, PCIe, PMIC, FPGA sensors via DMA/MSI-X
//!   2. Kernel Driver Layer  — Top-half (IRQ) + Bottom-half (threaded IRQ)
//!   3. Lock-Free Ring Buffer — SPSC, cache-line aligned, O(1) ops
//!   4. DMA Zero-Copy Model  — Direct HW→kernel→userspace via mmap
//!   5. Userspace Daemon     — Orchestration, batching, backpressure
//!   6. AI/Simulation Core   — Forecasting, anomaly detection, optimization
//!
//! Performance targets:
//!   - IRQ latency:           < 5 µs
//!   - Ring buffer write:     O(1)
//!   - Memory copies:         0 in hot path
//!   - CPU usage under load:   minimized via batching
//!   - No runtime alloc in kernel path
//!
//! Author: QuantumEnergyOS Team — Mexicali, B.C.
//! License: MIT
// ═══════════════════════════════════════════════════════════════════════════════

#![warn(missing_docs)]
#![warn(clippy::all)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(feature = "std")]
pub mod userspace;
#[cfg(feature = "std")]
pub mod ai_simulation;

pub mod backpressure;
pub mod config;
pub mod dma;
pub mod driver;
pub mod frame;
pub mod observability;
pub mod pipeline;
pub mod platform;
pub mod ring_buffer;

pub use backpressure::*;
pub use config::*;
pub use dma::*;
pub use driver::*;
pub use frame::*;
pub use pipeline::*;
pub use platform::*;
pub use ring_buffer::*;

#[cfg(feature = "std")]
pub use ai_simulation::*;
