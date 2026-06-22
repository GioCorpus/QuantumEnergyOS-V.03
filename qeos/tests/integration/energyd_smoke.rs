use energyd::{EnergyEngine, EnergyDaemon, EnergyDaemonConfig};

#[test]
fn energy_engine_baseline() {
    let engine = EnergyEngine::new(10.0, 100.0, 0.5);
    assert!(engine.current_load(18) >= engine.current_load(3));
}

#[test]
fn energy_daemon_initializes() {
    let cfg = EnergyDaemonConfig::default();
    assert!(cfg.base.metrics_port > 0);
}
