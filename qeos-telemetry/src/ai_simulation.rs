// ═══════════════════════════════════════════════════════════════════════════════
//  ai_simulation — AI/Simulation core layer: consumers, forecasting, anomaly detection
// ═══════════════════════════════════════════════════════════════════════════════
//!
//! [Research Prototype]
//!
//! This module provides the AI/Simulation core layer:
//!   - TelemetryConsumer: trait for consuming telemetry frames
//!   - ForecastEngine: energy forecasting hooks
//!   - AnomalyDetector: ML-based anomaly detection (prototype)
//!   - GridSimulator: grid instability simulation
//!
//! Note: These are research prototypes intended for simulation and model
//! training. The kernel-path components are production-grade, but the
//! AI/Simulation layer is experimental.
// ═══════════════════════════════════════════════════════════════════════════════

use crate::{
    frame::{EnergyTelemetryFrame, flags},
};
use core::sync::atomic::{AtomicU64, Ordering};

/// Trait for telemetry frame consumers
pub trait TelemetryConsumer {
    fn on_frame(&mut self, frame: &EnergyTelemetryFrame);
    fn on_batch(&mut self, frames: &[EnergyTelemetryFrame]);
    fn on_anomaly(&mut self, frame: &EnergyTelemetryFrame);
    fn stats(&self) -> ConsumerStats;
    fn reset(&mut self);
}

/// Consumer statistics
#[derive(Debug, Clone, Copy, Default)]
pub struct ConsumerStats {
    pub frames_consumed: u64,
    pub batches_consumed: u64,
    pub anomalies_detected: u64,
    pub errors: u64,
    pub last_consume_ns: u64,
}

/// Forecasting engine for energy prediction
pub struct ForecastEngine {
    voltage_history: Vec<f32>,
    current_history: Vec<f32>,
    power_history: Vec<f32>,
    max_history: usize,
    last_forecast_ns: AtomicU64,
    forecast_count: AtomicU64,
}

impl ForecastEngine {
    #[inline]
    pub fn new(max_history: usize) -> Self {
        Self {
            voltage_history: Vec::with_capacity(max_history),
            current_history: Vec::with_capacity(max_history),
            power_history: Vec::with_capacity(max_history),
            max_history,
            last_forecast_ns: AtomicU64::new(0),
            forecast_count: AtomicU64::new(0),
        }
    }

    #[inline]
    pub fn default() -> Self {
        Self::new(1024)
    }

    #[inline]
    pub fn ingest(&mut self, frame: &EnergyTelemetryFrame) {
        let v = frame.voltage_v();
        let i = frame.current_a();
        let p = v * i;
        self.voltage_history.push(v);
        self.current_history.push(i);
        self.power_history.push(p);
        if self.voltage_history.len() > self.max_history {
            self.voltage_history.remove(0);
            self.current_history.remove(0);
            self.power_history.remove(0);
        }
    }

    #[inline]
    pub fn forecast(&self, horizon_seconds: f32) -> ForecastResult {
        let n = self.power_history.len();
        if n < 10 {
            return ForecastResult::insufficient_data();
        }
        let (slope, intercept) = self.linear_regression(&self.power_history);
        let current_power = self.power_history.last().copied().unwrap_or(0.0);
        let predicted_power = current_power + slope * horizon_seconds;
        let variance = self.variance(&self.power_history);
        let confidence = (1.0 - (variance / current_power.max(0.01))).clamp(0.0, 1.0);
        self.last_forecast_ns.store(crate::platform::timestamp_ns(), Ordering::Relaxed);
        self.forecast_count.fetch_add(1, Ordering::Relaxed);
        ForecastResult {
            predicted_power_w: predicted_power.max(0.0),
            confidence,
            horizon_seconds,
            model: ForecastModel::LinearRegression,
        }
    }

    #[inline]
    fn linear_regression(&self, data: &[f32]) -> (f32, f32) {
        let n = data.len() as f32;
        let sum_x: f32 = (0..data.len()).map(|i| i as f32).sum();
        let sum_y: f32 = data.iter().sum();
        let sum_xy: f32 = data.iter().enumerate().map(|(i, &y)| i as f32 * y).sum();
        let sum_x2: f32 = (0..data.len()).map(|i| (i as f32).powi(2)).sum();
        let denom = n * sum_x2 - sum_x * sum_x;
        if denom.abs() < 1e-6 {
            return (0.0, sum_y / n);
        }
        let slope = (n * sum_xy - sum_x * sum_y) / denom;
        let intercept = (sum_y - slope * sum_x) / n;
        (slope, intercept)
    }

    #[inline]
    fn variance(&self, data: &[f32]) -> f32 {
        if data.len() < 2 { return 0.0; }
        let mean = data.iter().sum::<f32>() / data.len() as f32;
        data.iter().map(|&x| (x - mean).powi(2)).sum::<f32>() / data.len() as f32
    }

    #[inline]
    pub fn forecast_count(&self) -> u64 { self.forecast_count.load(Ordering::Relaxed) }
    #[inline]
    pub fn history_size(&self) -> usize { self.voltage_history.len() }
}

/// Forecast result
#[derive(Debug, Clone, Copy)]
pub struct ForecastResult {
    pub predicted_power_w: f32,
    pub confidence: f32,
    pub horizon_seconds: f32,
    pub model: ForecastModel,
}

impl ForecastResult {
    #[inline]
    pub fn insufficient_data() -> Self {
        Self {
            predicted_power_w: 0.0,
            confidence: 0.0,
            horizon_seconds: 0.0,
            model: ForecastModel::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForecastModel {
    None,
    LinearRegression,
    MovingAverage,
    ExponentialSmoothing,
    ARIMA,
}

/// Anomaly detector
pub struct AnomalyDetector {
    voltage_mean: f32,
    voltage_std: f32,
    current_mean: f32,
    current_std: f32,
    z_threshold: f32,
    total_frames: AtomicU64,
    anomalies_detected: AtomicU64,
}

impl AnomalyDetector {
    #[inline]
    pub fn new() -> Self {
        Self {
            voltage_mean: 120.0,
            voltage_std: 10.0,
            current_mean: 10.0,
            current_std: 5.0,
            z_threshold: 3.0,
            total_frames: AtomicU64::new(0),
            anomalies_detected: AtomicU64::new(0),
        }
    }

    #[inline]
    pub fn with_params(v_mean: f32, v_std: f32, i_mean: f32, i_std: f32, z_thresh: f32) -> Self {
        Self {
            voltage_mean: v_mean, voltage_std: v_std,
            current_mean: i_mean, current_std: i_std,
            z_threshold: z_thresh,
            total_frames: AtomicU64::new(0),
            anomalies_detected: AtomicU64::new(0),
        }
    }

    #[inline]
    pub fn detect(&self, frame: &EnergyTelemetryFrame) -> AnomalyResult {
        self.total_frames.fetch_add(1, Ordering::Relaxed);
        let v = frame.voltage_v();
        let i = frame.current_a();
        let v_z = (v - self.voltage_mean).abs() / self.voltage_std.max(0.01);
        let i_z = (i - self.current_mean).abs() / self.current_std.max(0.01);
        let mut anomalies = Vec::new();
        if v_z > self.z_threshold { anomalies.push(AnomalyType::VoltageOutlier { z_score: v_z }); }
        if i_z > self.z_threshold { anomalies.push(AnomalyType::CurrentOutlier { z_score: i_z }); }
        let freq_dev = (frame.frequency_hz() - 50.0).abs().min((frame.frequency_hz() - 60.0).abs());
        if freq_dev > 2.0 { anomalies.push(AnomalyType::FrequencyDeviation { deviation_hz: freq_dev }); }
        let power = v * i;
        let expected_power = self.voltage_mean * self.current_mean;
        if power > expected_power * 3.0 { anomalies.push(AnomalyType::PowerSpike { power_w: power }); }
        if !anomalies.is_empty() { self.anomalies_detected.fetch_add(1, Ordering::Relaxed); }
        AnomalyResult {
            is_anomalous: !anomalies.is_empty(),
            anomalies,
            confidence: 1.0 - (v_z.max(i_z) / (self.z_threshold * 2.0)).clamp(0.0, 1.0),
        }
    }

    #[inline] pub fn total_frames(&self) -> u64 { self.total_frames.load(Ordering::Relaxed) }
    #[inline] pub fn anomalies_detected(&self) -> u64 { self.anomalies_detected.load(Ordering::Relaxed) }
}

#[derive(Debug, Clone)]
pub struct AnomalyResult {
    pub is_anomalous: bool,
    pub anomalies: Vec<AnomalyType>,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
pub enum AnomalyType {
    VoltageOutlier { z_score: f32 },
    CurrentOutlier { z_score: f32 },
    FrequencyDeviation { deviation_hz: f32 },
    PowerSpike { power_w: f32 },
}

/// Grid instability simulator
pub struct GridSimulator {
    base_voltage_v: f32,
    base_current_a: f32,
    base_frequency_hz: f32,
    dt_seconds: f32,
    sim_time: f32,
    active_events: Vec<GridEvent>,
    next_sensor_id: u32,
}

impl GridSimulator {
    #[inline]
    pub fn new() -> Self {
        Self {
            base_voltage_v: 230.0,
            base_current_a: 10.0,
            base_frequency_hz: 50.0,
            dt_seconds: 0.001,
            sim_time: 0.0,
            active_events: Vec::new(),
            next_sensor_id: 0x01000001,
        }
    }

    #[inline]
    pub fn north_american() -> Self {
        Self {
            base_voltage_v: 120.0,
            base_current_a: 15.0,
            base_frequency_hz: 60.0,
            dt_seconds: 0.001,
            sim_time: 0.0,
            active_events: Vec::new(),
            next_sensor_id: 0x01000001,
        }
    }

    #[inline]
    pub fn step(&mut self) -> Vec<EnergyTelemetryFrame> {
        self.sim_time += self.dt_seconds;
        let mut frames = Vec::new();
        self.active_events.retain_mut(|event| {
            event.elapsed += self.dt_seconds;
            event.elapsed < event.duration_seconds
        });
        let mut voltage_offset = 0.0f32;
        let mut current_offset = 0.0f32;
        let mut freq_offset = 0.0f32;
        for event in &self.active_events {
            match event.event_type {
                GridEventType::VoltageSag { depth_pct } => { voltage_offset -= (depth_pct / 100.0) * self.base_voltage_v; }
                GridEventType::VoltageSwell { swell_pct } => { voltage_offset += (swell_pct / 100.0) * self.base_voltage_v; }
                GridEventType::FrequencyDeviation { dev_hz } => { freq_offset += dev_hz; }
                GridEventType::LoadStep { delta_a } => { current_offset += delta_a; }
                GridEventType::Transient { peak_v } => {
                    let t = event.elapsed / event.duration_seconds;
                    voltage_offset += peak_v * (-((t - 0.1) / 0.05).powi(2)).exp();
                }
            }
        }
        let voltage = (self.base_voltage_v + voltage_offset + self.gaussian_noise(1.0)).max(0.0);
        let current = (self.base_current_a + current_offset + self.gaussian_noise(0.5)).max(0.0);
        let freq = self.base_frequency_hz + freq_offset + self.gaussian_noise(0.1);
        let sensor_id = self.next_sensor_id;
        self.next_sensor_id = self.next_sensor_id.wrapping_add(1);
        let mut fflags = flags::FLAG_DMA_VALID | flags::FLAG_CHECKSUM_OK;
        if voltage < 80.0 || voltage > 260.0 { fflags |= flags::FLAG_OVERVOLTAGE; }
        if current > 100.0 { fflags |= flags::FLAG_OVERCURRENT; }
        if (freq - 50.0).abs() > 2.0 { fflags |= flags::FLAG_FREQ_ANOMALY | flags::FLAG_GRID_INSTABLE; }
        let frame = EnergyTelemetryFrame::from_parts(
            (self.sim_time * 1e9) as u64,
            sensor_id,
            (voltage * 1000.0) as u32,
            (current * 1000.0) as u32,
            (freq * 100.0) as u16,
            fflags,
        );
        frames.push(frame);
        frames
    }

    #[inline]
    pub fn inject_event(&mut self, event: GridEvent) { self.active_events.push(event); }

    #[inline]
    pub fn voltage_sag(&mut self, depth_pct: f32, duration_s: f32) {
        self.inject_event(GridEvent { event_type: GridEventType::VoltageSag { depth_pct }, duration_seconds: duration_s, elapsed: 0.0 });
    }

    #[inline]
    pub fn frequency_deviation(&mut self, dev_hz: f32, duration_s: f32) {
        self.inject_event(GridEvent { event_type: GridEventType::FrequencyDeviation { dev_hz }, duration_seconds: duration_s, elapsed: 0.0 });
    }

    #[inline]
    pub fn sim_time(&self) -> f32 { self.sim_time }

    #[inline]
    fn gaussian_noise(&self, std: f32) -> f32 {
        let seed = (self.sim_time * 1e6) as u64;
        let a = ((seed.wrapping_mul(6_364_136_223_846_793_005u64).wrapping_add(1_442_695_040_888_963_407u64)) as f64 / u64::MAX as f64).max(1e-300);
        let b = ((seed.wrapping_mul(2_862_933_555_777_941_757u64).wrapping_add(3_037_000_493u64)) as f64 / u64::MAX as f64).max(1e-300);
        let noise_f64 = (std as f64) * (-2.0 * a.ln()).sqrt() * (2.0 * core::f64::consts::PI * b).cos();
        noise_f64 as f32
    }
}

/// Grid event type
#[derive(Debug, Clone, Copy)]
pub enum GridEventType {
    VoltageSag { depth_pct: f32 },
    VoltageSwell { swell_pct: f32 },
    FrequencyDeviation { dev_hz: f32 },
    LoadStep { delta_a: f32 },
    Transient { peak_v: f32 },
}

/// Grid event
#[derive(Debug, Clone, Copy)]
pub struct GridEvent {
    pub event_type: GridEventType,
    pub duration_seconds: f32,
    pub elapsed: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forecast_engine() {
        let mut engine = ForecastEngine::new(100);
        for i in 0..20u32 {
            engine.ingest(&EnergyTelemetryFrame::from_parts(i as u64, 0, 230000, 10000, 5000, flags::FLAG_CHECKSUM_OK));
        }
        let forecast = engine.forecast(1.0);
        assert!(forecast.predicted_power_w > 0.0);
    }

    #[test]
    fn test_anomaly_detector() {
        let detector = AnomalyDetector::new();
        let normal = EnergyTelemetryFrame::from_parts(1000, 1, 120000, 10000, 5000, 0);
        let result = detector.detect(&normal);
        assert!(!result.is_anomalous);

        let spike = EnergyTelemetryFrame::from_parts(1000, 1, 500000, 50000, 5000, 0);
        let result2 = detector.detect(&spike);
        assert!(result2.is_anomalous);
    }

    #[test]
    fn test_grid_simulator() {
        let mut sim = GridSimulator::new();
        let frames = sim.step();
        assert_eq!(frames.len(), 1);
        assert!(frames[0].voltage_mv > 0);
    }
}
