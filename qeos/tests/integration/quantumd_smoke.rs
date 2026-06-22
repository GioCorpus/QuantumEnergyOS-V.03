use std::time::Duration;

use tokio::time::timeout;
use tokio_test::assert_ok;

use quantumd::{QuantumDaemon, QuantumDaemonConfig};

#[tokio::test]
async fn quantumd_initializes() {
    let cfg = QuantumDaemonConfig::default();
    let daemon = timeout(Duration::from_secs(5), QuantumDaemon::new(cfg))
        .await
        .expect("timeout");
    assert_ok!(daemon);
}

#[tokio::test]
async fn quantumd_health_returns_true() {
    let cfg = QuantumDaemonConfig::default();
    let daemon = QuantumDaemon::new(cfg).await.unwrap();
    assert!(daemon.health().await.unwrap());
}
