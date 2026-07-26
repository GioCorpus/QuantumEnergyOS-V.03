use tracing::{field, info, instrument};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Thread-safe metrics collector for QuantumEnergyOS daemons
#[derive(Debug, Default)]
pub struct QeosMetrics {
    counters: Arc<RwLock<HashMap<String, AtomicU64>>>,
    gauges: Arc<RwLock<HashMap<String, AtomicU64>>>,
    histograms: Arc<RwLock<HashMap<String, Vec<f64>>>>,
    start_time: Instant,
}

impl QeosMetrics {
    pub fn new() -> Self {
        Self {
            counters: Arc::new(RwLock::new(HashMap::new())),
            gauges: Arc::new(RwLock::new(HashMap::new())),
            histograms: Arc::new(RwLock::new(HashMap::new())),
            start_time: Instant::now(),
        }
    }

    #[instrument(skip(self))]
    pub async fn increment_counter(&self, name: impl Into<String>) {
        let name = name.into();
        let mut counters = self.counters.write().await;
        counters
            .entry(name.clone())
            .and_modify(|c| c.fetch_add(1, Ordering::Relaxed))
            .or_insert(AtomicU64::new(1));
        info!(metric = "counter", name = %name, "incremented");
    }

    #[instrument(skip(self))]
    pub async fn set_gauge(&self, name: impl Into<String>, value: u64) {
        let name = name.into();
        let mut gauges = self.gauges.write().await;
        gauges.insert(name.clone(), AtomicU64::new(value));
        info!(metric = "gauge", name = %name, value = %value, "set");
    }

    #[instrument(skip(self))]
    pub async fn record_histogram(&self, name: impl Into<String>, value: f64) {
        let name = name.into();
        let mut histograms = self.histograms.write().await;
        histograms.entry(name.clone()).or_default().push(value);
    }

    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    pub async fn snapshot(&self) -> MetricsSnapshot {
        let counters = self.counters.read().await;
        let gauges = self.gauges.read().await;
        let histograms = self.histograms.read().await;

        let counter_map: HashMap<String, u64> = counters
            .iter()
            .map(|(k, v)| (k.clone(), v.load(Ordering::Relaxed)))
            .collect();

        let gauge_map: HashMap<String, u64> = gauges
            .iter()
            .map(|(k, v)| (k.clone(), v.load(Ordering::Relaxed)))
            .collect();

        let histogram_map: HashMap<String, HistogramStats> = histograms
            .iter()
            .map(|(k, v)| {
                let stats = Self::compute_histogram_stats(v);
                (k.clone(), stats)
            })
            .collect();

        MetricsSnapshot {
            uptime_seconds: self.uptime_seconds(),
            counters: counter_map,
            gauges: gauge_map,
            histograms: histogram_map,
        }
    }

    fn compute_histogram_stats(values: &[f64]) -> HistogramStats {
        if values.is_empty() {
            return HistogramStats::default();
        }
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let sum: f64 = sorted.iter().sum();
        let count = sorted.len();
        let mean = sum / count as f64;
        let p99 = sorted[(count * 99) / 100];
        let min = sorted[0];
        let max = sorted[count - 1];

        HistogramStats {
            count,
            min,
            max,
            mean,
            p99,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricsSnapshot {
    pub uptime_seconds: u64,
    pub counters: HashMap<String, u64>,
    pub gauges: HashMap<String, u64>,
    pub histograms: HashMap<String, HistogramStats>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct HistogramStats {
    pub count: usize,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub p99: f64,
}
