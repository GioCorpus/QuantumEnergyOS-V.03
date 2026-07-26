// ═══════════════════════════════════════════════════════════════════════════════
//  qeos-telemetry integration tests
// ═══════════════════════════════════════════════════════════════════════════════

use qeos_telemetry::*;
use qeos_telemetry::frame::flags;

mod ring_buffer_tests {
    use super::*;

    #[test]
    fn test_spsc_basic() {
        let rb = RingBuffer::new(64).unwrap();
        let frame = EnergyTelemetryFrame::from_parts(100, 1, 5000, 1000, 5995, flags::FLAG_CHECKSUM_OK);
        assert_eq!(rb.push(frame), PushResult::Ok);
        assert_eq!(rb.len(), 1);
        let popped = rb.pop().unwrap();
        assert_eq!(popped.timestamp_ns, 100);
    }

    #[test]
    fn test_spsc_overwrite() {
        let rb = RingBuffer::with_policy(4, ring_buffer::FillPolicy::OverwriteOldest).unwrap();
        for i in 0..8u64 {
            let frame = EnergyTelemetryFrame::from_parts(i, (i % 256) as u32, 100, 100, 5000, 0);
            rb.push(frame);
        }
        assert_eq!(rb.len(), 4);
        let f0 = rb.pop().unwrap();
        let _f1 = rb.pop().unwrap();
        let _f2 = rb.pop().unwrap();
        let f3 = rb.pop().unwrap();
        assert_eq!(f0.timestamp_ns, 4);
        assert_eq!(f3.timestamp_ns, 7);
    }

    #[test]
    fn test_spsc_drop_newest() {
        let rb = RingBuffer::with_policy(2, ring_buffer::FillPolicy::DropNewest).unwrap();
        for i in 0..4u64 {
            let frame = EnergyTelemetryFrame::from_parts(i, 0, 100, 100, 5000, 0);
            let result = rb.push(frame);
            if i < 2 { assert_eq!(result, PushResult::Ok); }
            else { assert_eq!(result, PushResult::Dropped); }
        }
        assert_eq!(rb.len(), 2);
    }

    #[test]
    fn test_spsc_batch() {
        let rb = RingBuffer::new(256).unwrap();
        let frames: Vec<_> = (0..100).map(|i| {
            EnergyTelemetryFrame::from_parts(i, i as u32, 230000, 10000, 5000, 0)
        }).collect();
        let (written, _) = rb.push_batch(&frames);
        assert_eq!(written, 100);
        let mut out = vec![EnergyTelemetryFrame::new(); 100];
        let count = rb.pop_batch(&mut out);
        assert_eq!(count, 100);
    }

    #[test]
    fn test_spsc_power_of_two() {
        assert!(RingBuffer::new(0).is_none());
        assert!(RingBuffer::new(3).is_none());
        assert!(RingBuffer::new(7).is_none());
        assert!(RingBuffer::new(1).is_some());
        assert!(RingBuffer::new(2).is_some());
        assert!(RingBuffer::new(4).is_some());
        assert!(RingBuffer::new(1024).is_some());
    }
}

mod dma_tests {
    use super::*;

    #[test]
    fn test_dma_buffer() {
        let buf = DmaBuffer::new(1024).unwrap();
        assert_eq!(buf.size(), 1024 * FRAME_SIZE);
        assert!(buf.is_empty());
    }

    #[test]
    fn test_dma_mmap() {
        let buf = DmaBuffer::new(64).unwrap();
        let mmap = buf.mmap();
        assert!(mmap.is_some());
        let mmap = mmap.unwrap();
        assert!(mmap.is_valid());
        assert_eq!(mmap.len(), 64 * FRAME_SIZE);
    }

    #[test]
    fn test_dma_frame_roundtrip() {
        let buf = DmaBuffer::new(64).unwrap();
        let original = EnergyTelemetryFrame::from_parts(42, 7, 120000, 5000, 5995, flags::FLAG_CHECKSUM_OK);
        buf.push_frame(original);
        let readback = buf.pop_frame().unwrap();
        assert_eq!(readback.timestamp_ns, 42);
        assert_eq!(readback.sensor_id, 7);
    }
}

mod driver_tests {
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
    fn test_driver_context() {
        let driver = DriverContext::new(1024, 16, SensorType::SPI).unwrap();
        driver.start_bottom_half();
        let frame = EnergyTelemetryFrame::from_parts(1000, 1, 5000, 1000, 5995, flags::FLAG_CHECKSUM_OK);
        let result = driver.simulate_irq(frame);
        assert!(result);
        assert!(driver.counters.irq_count.load(Ordering::Relaxed) > 0);
    }

    #[test]
    fn test_calibration() {
        let mut driver = DriverContext::new(1024, 16, SensorType::I2C).unwrap();
        let calib = CalibrationConstant {
            voltage_offset_mv: 100,
            voltage_scale: 0x00010000,
            current_offset_ma: 50,
            current_scale: 0x00010000,
            freq_offset_hz_x100: 0,
            calib_timestamp_ns: 1000,
        };
        driver.load_calibration(1, calib);
        let calib_loaded = driver.calibration.get(1);
        assert!(calib_loaded.is_some());
    }
}

mod backpressure_tests {
    use super::*;

    #[test]
    fn test_realtime_overwrite() {
        let mut mgr = BackpressureManager::realtime();
        let ring = RingBuffer::new(2).unwrap();
        for i in 0..4u64 {
            let frame = EnergyTelemetryFrame::from_parts(i, i as u32, 100, 100, 5000, 0);
            let disp = mgr.handle_frame(frame, &ring);
            if i < 2 { assert_eq!(disp, FrameDisposition::Accepted); }
            else { assert_eq!(disp, FrameDisposition::Overwritten); }
        }
        assert_eq!(mgr.state().overwritten_frames.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_scientific_spill() {
        let mut mgr = BackpressureManager::scientific();
        let ring = RingBuffer::new(2).unwrap();
        for i in 0..10u64 {
            let frame = EnergyTelemetryFrame::from_parts(i, i as u32, 100, 100, 5000, 0);
            let disp = mgr.handle_frame(frame, &ring);
            if i < 2 { assert_eq!(disp, FrameDisposition::Accepted); }
            else { assert_eq!(disp, FrameDisposition::Spilled); }
        }
        assert_eq!(mgr.secondary_usage(), 8);
    }

    #[test]
    fn test_utilization() {
        let state = BackpressureState::new(BackpressureMode::Realtime);
        let ring = RingBuffer::new(8).unwrap();
        for i in 0..4 { ring.push(EnergyTelemetryFrame::from_parts(i, 0, 0, 0, 0, 0)); }
        state.update_utilization(&ring);
        let util = state.utilization_f32();
        assert!((util - 0.5).abs() < 0.01);
    }
}

mod simulation_tests {
    use super::*;

    #[test]
    fn test_grid_simulator() {
        let mut sim = GridSimulator::new();
        let frames = sim.step();
        assert_eq!(frames.len(), 1);
        assert!(frames[0].voltage_mv > 0);
    }

    #[test]
    fn test_grid_simulator_sag() {
        let mut sim = GridSimulator::new();
        sim.voltage_sag(30.0, 0.5);
        for _ in 0..500 {
            let frames = sim.step();
            for frame in frames {
                if frame.has_flag(flags::FLAG_OVERVOLTAGE as u16) || frame.voltage_v() < 100.0 {
                }
            }
        }
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
    fn test_forecast_engine() {
        let mut engine = ForecastEngine::new(100);
        for i in 0..20u32 {
            engine.ingest(&EnergyTelemetryFrame::from_parts(i as u64, 0, 230000, 10000, 5000, flags::FLAG_CHECKSUM_OK));
        }
        let forecast = engine.forecast(1.0);
        assert!(forecast.predicted_power_w > 0.0);
    }
}

mod config_tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TelemetryConfig::default();
        assert_eq!(config.backpressure_mode, BackpressureMode::Realtime);
        assert!(config.enable_anomaly_detection);
    }

    #[test]
    fn test_realtime_config() {
        let config = TelemetryConfig::realtime();
        assert_eq!(config.backpressure_mode, BackpressureMode::Realtime);
    }

    #[test]
    fn test_scientific_config() {
        let config = TelemetryConfig::scientific();
        assert_eq!(config.backpressure_mode, BackpressureMode::Scientific);
        assert_eq!(config.ring_buffer_capacity, 262144);
    }

    #[test]
    fn test_emergency_config() {
        let config = TelemetryConfig::emergency();
        assert_eq!(config.backpressure_mode, BackpressureMode::Emergency);
        assert!(!config.enable_anomaly_detection);
    }

    #[test]
    fn test_sensor_type() {
        assert_eq!(SensorType::from_id(0x01000001), SensorType::I2C);
        assert_eq!(SensorType::from_id(0x02000002), SensorType::SPI);
        assert_eq!(SensorType::from_id(0x03000003), SensorType::PCIe);
        assert_eq!(SensorType::from_id(0x04000004), SensorType::PMIC);
        assert_eq!(SensorType::from_id(0x05000005), SensorType::FPGA);
    }

    #[test]
    fn test_calibration_constant() {
        let mut frame = EnergyTelemetryFrame::from_parts(1000, 1, 10000, 5000, 5000, 0);
        let calib = CalibrationConstant {
            voltage_offset_mv: 100,
            voltage_scale: 0x00020000, // 2.0
            current_offset_ma: 0,
            current_scale: 0x00010000,
            freq_offset_hz_x100: 50,
            calib_timestamp_ns: 0,
        };
        calib.apply(&mut frame);
        assert!(frame.has_flag(flags::FLAG_CALIBRATED));
    }
}
