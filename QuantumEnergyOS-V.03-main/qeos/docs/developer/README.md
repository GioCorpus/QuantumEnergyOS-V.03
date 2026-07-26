# Developer Guide — QuantumEnergyOS V.03

## Overview

> **Warning**: This document is for **existing technology only**. Future features must be explicitly marked `[Research Prototype]`, `[Experimental]`, or `[Long-Term Vision]`. Do not mix future plans with shipped behavior.

## Table of Contents

1. [System Requirements](#system-requirements)
2. [Workspace Structure](#workspace-structure)
3. [Adding a New Daemon](#adding-a-new-daemon)
4. [Adding a Library](#adding-a-library)
5. [Coding Standards](#coding-standards)
6. [Testing](#testing)
7. [Pull Requests](#pull-requests)
8. [Troubleshooting](#troubleshooting)

---

## System Requirements

| Tool | Version | Purpose |
|------|---------|---------|
| `rustc` | ≥1.75 | Compiler |
| `cargo` | ≥1.75 | Build + test |
| `python` | ≥3.12 | Scripts / NanoClaude |
| `git` | ≥2.40 | Version control |
| `pytest` | ≥8.0 | Python tests |
| `docker` | ≥26 | Isolation (optional) |

```bash
# Install Rust (Linux)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Python deps
python -m venv .venv && source .venv/bin/activate
pip install -r packages/python-requirements.txt
```

---

## Workspace Structure

```
qeos/
├── Cargo.toml              # Workspace root
├── Makefile                # Build entry points
├── docs/                   # Documentation
│   ├── api/                # API reference
│   ├── developer/          # This guide
│   ├── deployment/         # Ops / install
│   └── research/           # Scientific context
├── daemons/                # Long-running network services
│   ├── quantumd/           # [Production Ready]
│   ├── energyd/            # [Production Ready]
│   ├── climated/           # [Production Ready]
│   ├── photonicd/          # [Research Prototype]
│   └── aicored/            # [Research Prototype]
├── libs/                   # Shared domain libraries
│   ├── qeos-common/        # Error types, metrics, config models
│   ├── qeos-quantum/       # Circuit simulation
│   ├── qeos-energy/        # Grid modeling
│   ├── qeos-climate/       # Climate models
│   ├── qeos-photonic/      # Signal processing
│   ├── qeos-ml/            # ML training/inference
│   ├── qeos-hpc/           # Distributed compute
│   └── qeos-security/      # AuthZ/AuthN, crypto
├── packages/               # OS-level packaging
│   ├── archiso/            # Arch Linux ISO profile
│   │   ├── setup.sh
│   │   ├── build-iso.sh
│   │   └── airootfs/       # Rootfs overlay
│   └── python-requirements.txt
├── cores/                  # Kernel research (Phase 2 / 3)
│   ├── arch/               # Architecture-specific boot / MM
│   └── kernel/             # Scheduler, memory, sync
│       ├── QuantumKernel.sv
│       └── task/
├── tools/                  # Build helpers
└── tests/                  # Rust + Python integration tests
    ├── unit/
    ├── integration/
    └── benchmarks/
```

---

## Adding a New Daemon

### 1. Create package directory

```bash
cargo new --lib daemons/<name>d
```

### 2. Update workspace Cargo.toml

Add the member to `qeos/Cargo.toml`:

```toml
members = [
    "daemons/<name>d",
]
```

### 3. Implement the daemon skeleton

Every daemon must expose a service with the same lifecycle:

```rust
pub struct MyDaemon {
    config: Arc<MyDaemonConfig>,
    metrics: Arc<QeosMetrics>,
}

impl MyDaemon {
    pub async fn new(config: MyDaemonConfig) -> Result<Self> { .. }
    pub async fn run(self) -> Result<()> { .. }
    pub async fn health_check(&self) -> Result<bool> { .. }
}
```

### 4. Add configuration

Reuse `qeos_common::DaemonConfig`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyDaemonConfig {
    pub base: DaemonConfig,
    // domain-specific fields
}

impl Default for MyDaemonConfig { ... }
```

### 5. Wire into the build

Add a Make target:

```makefile
daemon-<name>:
	cargo run -p <name>d --release
```

---

## Adding a Library

Rules:
- Pure domain logic first; I/O last.
- No `daemon` / `d` in library names.
- Place shared types in `libs/qeos-common` if they cross domains.

```bash
cargo new --lib libs/qeos-<domain>
```

Shared types must live in `qeos-common`:

```rust
// libs/qeos-common/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum QeosError {
    #[error("internal: {0}")]
    Internal(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("not found: {0}")]
    NotFound(String),
}
pub type Result<T> = std::result::Result<T, QeosError>;
```

---

## Coding Standards

| Layer | Rule |
|-------|------|
| Error handling | Use `QeosError` + `Result<T>` from `qeos-common`. |
| Logging | `tracing` only; never `println` in library code. |
| Formatting | `cargo fmt` on save. |
| Linting | `cargo clippy -D warnings` must pass. |
| Naming | `snake_case` for files/functions; `CamelCase` types; `UPPER_SNAKE` constants. |
| Unsafe | Prohibited outside `cores/kernel/*` unless explicitly reviewed. |
| Secrets | Never log tokens / keys; use `QEOS_SECRET_PATH` env var. |

---

## Testing

```bash
# All tests
make test

# Rust only
cargo test --workspace

# Python only
pytest tests/

# Single crate
cargo test -p quantumd

# With logging
RUST_LOG=debug cargo test -p energy
```

### Writing unit tests

Prefer inline modules:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn computes_load() {
        let eng = EnergyEngine::new(10.0, 100.0, 0.5);
        assert!(eng.current_load(18) > eng.current_load(3));
    }
}
```

---

## Pull Requests

1. Run `make lint` before pushing.
2. Add tests for new behavior.
3. Update `ARCHITECTURE.md` if public API changes.
4. For daemons: add a systemd unit under `tools/scripts/systemd/`.

---

## Troubleshooting

- **Workspace resolution errors**: Run `cargo metadata --format-version 1` from `qeos/`.
- **Linker errors on Windows**: Install `llvm` and `mingw-w64` toolchains.
- **Python venv not found**: `python -m venv .venv` then `source .venv/bin/activate`.

---

*QuantumEnergyOS V.03 — Giovanny Anthony Corpus Bernal*
