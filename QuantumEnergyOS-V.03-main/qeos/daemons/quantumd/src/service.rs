use std::sync::Arc;

use crate::{QuantumBackend, QuantumDaemon, QuantumDaemonConfig};
use qeos_common::QeosMetrics;
use tokio::net::TcpListener;
use tracing::{info, instrument};

/// QuantumEnergyOS Quantum Daemon
/// 
/// Manages quantum computing resources, simulation backends, and quantum circuit execution.
/// 
/// # Classification: [Production Ready]
pub struct QuantumDaemon {
    config: Arc<QuantumDaemonConfig>,
    backend: QuantumBackend,
    metrics: Arc<QeosMetrics>,
    shutdown_tx: tokio::sync::watch::Sender<bool>,
}

impl QuantumDaemon {
    #[instrument(skip(config))]
    pub async fn new(config: QuantumDaemonConfig) -> Result<Self> {
        let config = Arc::new(config);
        let backend = QuantumBackend::new(config.clone());
        let metrics = Arc::new(QeosMetrics::new());
        let (shutdown_tx, _) = tokio::sync::watch::channel(false);

        info!(
            name = config.base.name,
            version = config.base.version,
            max_qubits = config.max_qubits,
            "initializing quantumd"
        );

        backend.initialize().await?;

        Ok(Self {
            config,
            backend,
            metrics,
            shutdown_tx,
        })
    }

    #[instrument(skip(self))]
    pub async fn run(&self) -> Result<()> {
        let addr = format!("{}:{}", self.config.base.bind_address, self.config.base.bind_port);
        let listener = TcpListener::bind(&addr).await.map_err(|e| {
            qeos_common::QeosError::Internal(format!("Failed to bind to {addr}: {e}"))
        })?;

        info!(addr = %addr, "quantumd listening");

        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, peer)) => {
                            info!(peer = %peer, "incoming connection");
                            let metrics = self.metrics.clone();
                            tokio::spawn(async move {
                                metrics.increment_counter("connections.accepted").await;
                            });
                        }
                        Err(e) => {
                            tracing::error!(error = %e, "accept error");
                            self.metrics.increment_counter("errors.accept").await;
                        }
                    }
                }
                _ = self.shutdown_tx.changed() => {
                    info!("shutdown signal received");
                    break;
                }
            }
        }

        Ok(())
    }

    pub async fn health(&self) -> Result<bool> {
        self.backend.health_check().await
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.shutdown_tx.send(true)?;
        self.backend.shutdown().await?;
        Ok(())
    }
}
