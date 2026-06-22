use climated::{ClimateEngine, ClimateDaemonConfig, ClimateSnapshot};

#[test]
fn climate_engine_accumulates() {
    let mut engine = ClimateEngine::new(10);
    for _ in 0..5 {
        engine.ingest(ClimateSnapshot {
            timestamp: 0,
            temperature_c: 25.0,
            humidity_percent: 50.0,
            pressure_hpa: 1013.0,
            wind_speed_kmh: 10.0,
            precipitation_mm: 0.0,
            location: (0.0, 0.0),
        });
    }
}
