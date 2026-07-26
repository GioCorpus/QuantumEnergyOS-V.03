//! qeos-telemetry-bench — Performance benchmarks for the telemetry pipeline
//!
//! Measures throughput, latency, and cache behavior of the SPSC ring buffer,
//! DMA buffer, backpressure manager, and end-to-end pipeline.
//!
//! Run with: `cargo run --bin qeos-telemetry-bench`
//!
//! [Production Kernel Component]

use qeos_telemetry::{
    backpressure::{BackpressureManager, BackpressureMode, FrameDisposition},
    config::TelemetryConfig,
    dma::DmaBuffer,
    frame::{EnergyTelemetryFrame, flags},
    pipeline::TelemetryPipeline,
    ring_buffer::{FillPolicy, RingBuffer},
};
use std::time::{Duration, Instant};

const BENCH_FRAME_COUNT: usize = 100_000;
const BENCH_BATCH_SIZES: &[usize] = &[1, 8, 16, 32, 64, 128, 256];
const BENCH_RING_SIZES: &[usize] = &[64, 256, 1024, 4096, 16384, 65536];

fn generate_test_frames(count: usize) -> Vec<EnergyTelemetryFrame> {
    (0..count)
        .map(|i| {
            EnergyTelemetryFrame::from_parts(
                i as u64 * 1_000_000,
                (i % 256) as u32,
                230_000,
                10_000,
                5000,
                flags::FLAG_CHECKSUM_OK | flags::FLAG_DMA_VALID,
            )
        })
        .collect()
}

/// Benchmark: SPSC ring buffer single-producer push throughput
fn bench_ring_buffer_push(capacity: usize) -> (usize, Duration) {
    let rb = RingBuffer::with_policy(capacity, FillPolicy::OverwriteOldest).unwrap();
    let frames = generate_test_frames(BENCH_FRAME_COUNT);
    let start = Instant::now();
    let mut written = 0usize;
    for f in &frames {
        match rb.push(*f) {
            qeos_telemetry::ring_buffer::PushResult::Ok => written += 1,
            _ => {}
        }
    }
    let elapsed = start.elapsed();
    (written, elapsed)
}

/// Benchmark: SPSC ring buffer single-consumer pop throughput
fn bench_ring_buffer_pop(capacity: usize) -> (usize, Duration) {
    let rb = RingBuffer::with_policy(capacity, FillPolicy::OverwriteOldest).unwrap();
    let frames = generate_test_frames(BENCH_FRAME_COUNT);
    for f in &frames {
        let _ = rb.push(*f);
    }
    let start = Instant::now();
    let mut popped = 0usize;
    while rb.pop().is_some() {
        popped += 1;
    }
    let elapsed = start.elapsed();
    (popped, elapsed)
}

/// Benchmark: SPSC ring buffer batch push throughput
fn bench_ring_buffer_batch_push(capacity: usize, batch_size: usize) -> (usize, Duration) {
    let rb = RingBuffer::with_policy(capacity, FillPolicy::OverwriteOldest).unwrap();
    let frames = generate_test_frames(BENCH_FRAME_COUNT);
    let batches: Vec<_> = frames.chunks(batch_size).collect();
    let start = Instant::now();
    let mut written = 0usize;
    for batch in batches {
        let (w, _) = rb.push_batch(batch);
        written += w;
    }
    let elapsed = start.elapsed();
    (written, elapsed)
}

/// Benchmark: SPSC ring buffer batch pop throughput
fn bench_ring_buffer_batch_pop(capacity: usize, batch_size: usize) -> (usize, Duration) {
    let rb = RingBuffer::with_policy(capacity, FillPolicy::OverwriteOldest).unwrap();
    let frames = generate_test_frames(BENCH_FRAME_COUNT);
    for f in &frames {
        let _ = rb.push(*f);
    }
    let mut out = vec![EnergyTelemetryFrame::new(); batch_size];
    let start = Instant::now();
    let mut popped = 0usize;
    loop {
        let n = rb.pop_batch(&mut out);
        if n == 0 {
            break;
        }
        popped += n;
    }
    let elapsed = start.elapsed();
    (popped, elapsed)
}

/// Benchmark: DMA buffer push/pop round-trip latency
fn bench_dma_roundtrip(capacity: usize) -> (usize, Duration) {
    let buf = DmaBuffer::new(capacity).unwrap();
    let frames = generate_test_frames(BENCH_FRAME_COUNT);
    let start = Instant::now();
    let mut written = 0usize;
    for f in &frames {
        if buf.push_frame(*f) == qeos_telemetry::ring_buffer::PushResult::Ok {
            written += 1;
        }
    }
    let mut read = 0usize;
    while buf.pop_frame().is_some() {
        read += 1;
    }
    let elapsed = start.elapsed();
    (read, elapsed)
}

/// Benchmark: End-to-end pipeline ingest throughput
fn bench_pipeline_ingest(mode: BackpressureMode) -> (usize, Duration) {
    let config = match mode {
        BackpressureMode::Realtime => TelemetryConfig::realtime(),
        BackpressureMode::Scientific => TelemetryConfig::scientific(),
        BackpressureMode::Emergency => TelemetryConfig::emergency(),
    };
    let mut pipeline = TelemetryPipeline::new(config).unwrap();
    let frames = generate_test_frames(BENCH_FRAME_COUNT);
    let start = Instant::now();
    let mut accepted = 0usize;
    for f in &frames {
        let disp = pipeline.ingest(*f);
        if matches!(disp, FrameDisposition::Accepted | FrameDisposition::Overwritten) {
            accepted += 1;
        }
    }
    let elapsed = start.elapsed();
    (accepted, elapsed)
}

/// Benchmark: Backpressure manager handle_frame latency
fn bench_backpressure_handle(mode: BackpressureMode) -> (usize, Duration) {
    let mut mgr = match mode {
        BackpressureMode::Realtime => BackpressureManager::realtime(),
        BackpressureMode::Scientific => BackpressureManager::scientific(),
        BackpressureMode::Emergency => BackpressureManager::emergency(),
    };
    let ring = RingBuffer::new(1024).unwrap();
    let frames = generate_test_frames(BENCH_FRAME_COUNT);
    let start = Instant::now();
    let mut accepted = 0usize;
    for f in &frames {
        let disp = mgr.handle_frame(*f, &ring);
        if matches!(disp, FrameDisposition::Accepted | FrameDisposition::Overwritten) {
            accepted += 1;
        }
    }
    let elapsed = start.elapsed();
    (accepted, elapsed)
}

fn format_throughput(count: usize, elapsed: Duration) -> String {
    let secs = elapsed.as_secs_f64();
    if secs > 0.0 {
        let rate = count as f64 / secs;
        if rate >= 1_000_000.0 {
            format!("{:.2}M ops/s", rate / 1_000_000.0)
        } else if rate >= 1_000.0 {
            format!("{:.2}K ops/s", rate / 1_000.0)
        } else {
            format!("{:.2} ops/s", rate)
        }
    } else {
        format!("{} ops (instant)", count)
    }
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║       qeos-telemetry-bench — Performance Benchmarks               ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();

    println!("═══ Ring Buffer Push Throughput (single-producer, overwrite policy) ═══");
    println!("{:<12} {:>16} {:>14} {:>20}", "Capacity", "Frames", "Time", "Throughput");
    println!("{:-<60}", "");
    for &cap in BENCH_RING_SIZES {
        let (n, elapsed) = bench_ring_buffer_push(cap);
        println!("{:<12} {:>16} {:>14.3}ms {:>20}", cap, n, elapsed.as_secs_f64() * 1000.0, format_throughput(n, elapsed));
    }
    println!();

    println!("═══ Ring Buffer Pop Throughput ═══");
    println!("{:<12} {:>16} {:>14} {:>20}", "Capacity", "Frames", "Time", "Throughput");
    println!("{:-<60}", "");
    for &cap in BENCH_RING_SIZES {
        let (n, elapsed) = bench_ring_buffer_pop(cap);
        println!("{:<12} {:>16} {:>14.3}ms {:>20}", cap, n, elapsed.as_secs_f64() * 1000.0, format_throughput(n, elapsed));
    }
    println!();

    println!("═══ Ring Buffer Batch Push Throughput ═══");
    println!("{:<12} {:>10} {:>16} {:>14} {:>20}", "Capacity", "Batch", "Frames", "Time", "Throughput");
    println!("{:-<76}", "");
    for &cap in &[64, 256, 1024] {
        for &bs in BENCH_BATCH_SIZES {
            let (n, elapsed) = bench_ring_buffer_batch_push(cap, bs);
            println!("{:<12} {:>10} {:>16} {:>14.3}ms {:>20}", cap, bs, n, elapsed.as_secs_f64() * 1000.0, format_throughput(n, elapsed));
        }
    }
    println!();

    println!("═══ Ring Buffer Batch Pop Throughput ═══");
    println!("{:<12} {:>10} {:>16} {:>14} {:>20}", "Capacity", "Batch", "Frames", "Time", "Throughput");
    println!("{:-<76}", "");
    for &cap in &[64, 256, 1024] {
        for &bs in BENCH_BATCH_SIZES {
            let (n, elapsed) = bench_ring_buffer_batch_pop(cap, bs);
            println!("{:<12} {:>10} {:>16} {:>14.3}ms {:>20}", cap, bs, n, elapsed.as_secs_f64() * 1000.0, format_throughput(n, elapsed));
        }
    }
    println!();

    println!("═══ DMA Buffer Round-Trip ═══");
    println!("{:<12} {:>16} {:>14} {:>20}", "Capacity", "Frames", "Time", "Throughput");
    println!("{:-<60}", "");
    for &cap in &[64, 256, 1024, 4096, 16384] {
        let (n, elapsed) = bench_dma_roundtrip(cap);
        println!("{:<12} {:>16} {:>14.3}ms {:>20}", cap, n, elapsed.as_secs_f64() * 1000.0, format_throughput(n, elapsed));
    }
    println!();

    println!("═══ Backpressure Manager Throughput ═══");
    for mode in &[BackpressureMode::Realtime, BackpressureMode::Scientific, BackpressureMode::Emergency] {
        let mode_str = format!("{:?}", mode);
        println!("─── Mode: {} ───", mode_str);
        println!("{:<16} {:>16} {:>14} {:>20}", "Frames", "Accepted", "Time", "Throughput");
        println!("{:-<68}", "");
        let (n, elapsed) = bench_backpressure_handle(*mode);
        println!("{:<16} {:>16} {:>14.3}ms {:>20}", BENCH_FRAME_COUNT, n, elapsed.as_secs_f64() * 1000.0, format_throughput(n, elapsed));
    }
    println!();

    println!("═══ End-to-End Pipeline Throughput ═══");
    for mode in &[BackpressureMode::Realtime, BackpressureMode::Scientific, BackpressureMode::Emergency] {
        let mode_str = format!("{:?}", mode);
        println!("─── Mode: {} ───", mode_str);
        println!("{:<16} {:>16} {:>14} {:>20}", "Frames", "Accepted", "Time", "Throughput");
        println!("{:-<68}", "");
        let (n, elapsed) = bench_pipeline_ingest(*mode);
        println!("{:<16} {:>16} {:>14.3}ms {:>20}", BENCH_FRAME_COUNT, n, elapsed.as_secs_f64() * 1000.0, format_throughput(n, elapsed));
    }
    println!();

    println!("═══ Latency Benchmarks (nanoseconds per operation) ═══");
    let rb = RingBuffer::with_policy(1024, FillPolicy::OverwriteOldest).unwrap();
    let frame = EnergyTelemetryFrame::from_parts(1000, 1, 230000, 10000, 5000, flags::FLAG_CHECKSUM_OK);

    let iters = 1_000_000;
    let start = Instant::now();
    for _ in 0..iters {
        let _ = rb.push(frame);
        let _ = rb.pop();
    }
    let elapsed = start.elapsed();
    let total_ops = iters as f64 * 2.0;
    let ns_per_op = elapsed.as_nanos() as f64 / total_ops;
    println!("Push+Pop round-trip: {:.2} ns/op ({} iterations)", ns_per_op, iters);

    let start = Instant::now();
    for _ in 0..iters {
        let _ = rb.push(frame);
    }
    let elapsed = start.elapsed();
    let ns_per_push = elapsed.as_nanos() as f64 / iters as f64;
    println!("Single push: {:.2} ns/op ({} iterations)", ns_per_push, iters);

    println!();
    println!("✅ Benchmarks complete.");
}