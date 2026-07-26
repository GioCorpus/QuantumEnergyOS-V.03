# AI_GUIDELINES.md — QuantumEnergyOS

## Scope
These guidelines apply to all AI/ML subsystems within QuantumEnergyOS, including forecasting, anomaly detection, optimization, and simulation acceleration.

## Core Principles

### No Hallucinations in Production
- All model outputs must be validated against physics constraints (conservation of energy, thermodynamics).
- Provide confidence intervals for all predictions.
- Never expose raw model weights or unvalidated predictions to end users.

### Reproducibility First
- Pin all model versions and random seeds in `requirements-pinned.txt` and Cargo.toml.
- Log hyperparameters, dataset checksums, and environment variables for every run.
- Use deterministic pipelines where latency permits.

### Data Governance
- Telemetry data is classified. Do not log raw sensor readings or PII externally.
- Anonymize grid data before exporting to external ML platforms.
- Retain raw data only as long as required by research or compliance policies.

## Model Requirements

### Real-Time Constraints
- Inference latency must not exceed 50 ms on the AI/Simulation Core layer.
- Use streaming inference (batch size 1 or micro-batching) for live grid monitoring.
- Scientific mode may use larger batches but must not block telemetry ingestion.

### Accuracy and Drift
- Monitor prediction drift daily. Retrain models if MAE exceeds 20% of baseline.
- Maintain a fallback rule-based controller if model confidence drops below threshold.
- Log all anomalies with timestamps, sensor IDs, and model version.

### Hardware Acceleration
- Prefer ONNX Runtime or Torch with CUDA/Vulkan backends where available.
- Quantize models to INT8 for edge deployment if accuracy loss < 2%.
- Keep a CPU fallback path for headless/server environments without GPUs.

## Safety and Security

### Safe Failure Modes
- If the AI layer fails, the system must degrade gracefully to rule-based control.
- Do not allow AI outputs to write directly to hardware registers without validation.
- Implement circuit breakers for model inference with configurable timeout.

### Adversarial Robustness
- Validate all training data for poisoning and backdoor triggers.
- Apply differential privacy when training on multi-tenant or customer datasets.
- Rate-limit inference endpoints to prevent resource exhaustion attacks.

### Auditability
- Every model decision must be traceable to a model ID, version, and input hash.
- Retain inference logs for 90 days in `data/logs/ai_inference/`.
- Provide an `ai explain` command that outputs reasoning chains for critical decisions.

## Development Workflow

### Training
- Use GPU-enabled containers for training (`docker-compose.override.yml`).
- Track experiments with MLflow or Weights & Biases; commit only config files, not artifacts.
- Split validation data by time, never randomly, for time-series telemetry.

### Testing
- Unit tests for preprocessing, postprocessing, and feature engineering.
- Integration tests for end-to-end pipeline with `tests/integration_test.rs`.
- Property tests to verify energy conservation bounds across model outputs.
- Benchmark inference latency with `cargo bench` or pytest-cov for Python modules.

### Code Quality
- Document model assumptions, expected input distributions, and failure modes.
- Use type hints and schema validation (Pydantic for Python, strong typing for Rust).
- Prohibit `unwrap()` and `expect()` in inference paths; use structured error handling.

## Compliance
- Follow IEC 62443 for industrial control system security when auto-tuning controllers.
- Comply with GDPR/CCPA for any personal energy usage data.
- Maintain SBOMs for all ML dependencies (`pip-audit --sbom cyclonedx`, `cargo audit`).
