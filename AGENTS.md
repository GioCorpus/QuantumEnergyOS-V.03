# QuantumEnergyOS — Agent Instructions

## Project Overview
QuantumEnergyOS is a Rust-based scientific operating system for energy optimization, climate monitoring, quantum simulation, and AI/ML research. Uses Arch Linux with systemd, Rust daemons, Python scientific stack, and Node.js dashboard.

## Key Crates
- `qeos-telemetry` — Real-time energy telemetry ingestion pipeline (SPSC ring buffer, DMA zero-copy, lock-free concurrency)
- `qeos-kernel` — Bare-metal microkernel (x86_64, no_std)
- `quantum-browser-daemon` — Browser automation daemon
- `photonic-core` — Photonic signal processing
- `photonic-bridge` — Bridge between photonic and quantum subsystems
- `universe-simulator` — Cosmos simulation engine

## Architecture Layers (Telemetry Subsystem)
1. Hardware Layer — I2C/SPI/PCIe/PMIC/FPGA sensors via DMA/MSI-X
2. Kernel Driver Layer — Top-half (IRQ) + Bottom-half (threaded IRQ)
3. Lock-Free Ring Buffer — SPSC, cache-line aligned, O(1) ops
4. DMA Zero-Copy Model — Direct HW→kernel→userspace via mmap
5. Userspace Daemon — Orchestration, batching, backpressure
6. AI/Simulation Core — Forecasting, anomaly detection, optimization

## Backpressure Modes
- Realtime (overwrite-oldest) — Live grid monitoring
- Scientific (preserve-full-history) — Research/ML training
- Emergency (polling) — Grid instability/fault conditions

## Performance Targets
- IRQ latency: < 5 µs
- Ring buffer write: O(1)
- Memory copies: 0 in hot path
- No runtime allocation in kernel path

## Code Quality Standards
- Idiomatic Rust: ownership, borrowing, zero-copy, Result<T,E>, Option<T>
- No unwrap() in production code
- No expect() in production code  
- No panic in IRQ/hot path
- Traits and generics for abstraction
- async with Tokio where appropriate
- rayon for parallelism
- Doc comments on all public items

## Testing
- Unit tests in each module
- Integration tests in `tests/integration_test.rs`
- Benchmark tests (criterion or manual)
- Property tests where applicable

## Security
- No unsafe in public API surface
- FFI safety at kernel/userspace boundary
- Input validation on all external data
- No secret exposure in logs

## Quartz5D Architecture (5 software layers, NOT physical dimensions)
- Layer 1: Physical — hardware abstraction
- Layer 2: Topological — connection/graph modeling
- Layer 3: Holographic Storage — encoding/compression
- Layer 4: Temporal Prediction — forecasting
- Layer 5: Cognitive Intelligence — AI/ML reasoning

## Refactoring History
- Fixed HardwareRegisters to use UnsafeCell for MMIO simulation (no Copy/Clone derives)
- Fixed EnergyTelemetryFrame from #[repr(C, packed)] to #[repr(C)] (packed caused unaligned access)
- Fixed anomaly detector frequency deviation bug (.max() → .min())
- Fixed PushResult import path (crate::ring_buffer::PushResult)
- Removed dummy_ring() from userspace.rs, rewrote with proper arch
- Added pipeline.rs for end-to-end orchestration
- Added hw/ module with SensorBackend trait, SimulatedSensor, SysfsPowerSensor
- Implemented bench.rs and telemetryd.rs binaries