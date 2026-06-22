# QuantumEnergyOS V.03 Deployment Guide

## Table of Contents

1. [System Requirements](#system-requirements)
2. [Quick Start](#quick-start)
3. [Installation](#installation)
4. [Configuration](#configuration)
5. [Service Management](#service-management)
6. [Testing](#testing)

---

## System Requirements

### Minimum Requirements

| Component | Specification |
|-----------|---------------|
| CPU | x86_64 with AVX2 support |
| RAM | 8 GB (16 GB recommended) |
| Storage | 50 GB free space |
| Display | 1920×1080 resolution |
| Network | Internet connection (initial setup) |

### Recommended Requirements

| Component | Specification |
|-----------|---------------|
| CPU | AMD Ryzen 7 / Intel i7 (6+ cores) |
| RAM | 32 GB |
| Storage | 200 GB NVMe SSD |
| GPU | NVIDIA RTX 3060+ (for AI workloads) |
| Network | Gigabit Ethernet |

---

## Quick Start

```bash
# Clone the repository
git clone https://github.com/GiovannyCorpus/QuantumEnergyOS.git
cd QuantumEnergyOS/qeos

# Build all daemons
make build-daemons

# Run tests
make test

# Build ISO (requires Arch Linux host)
sudo make build-archiso
```

---

## Installation

### ISO Installation

1. Download `qeos-03.0-x86_64.iso`
2. Create bootable USB:
   ```bash
   dd if=qeos-03.0-x86_64.iso of=/dev/sdX bs=4M status=progress
   sync
   ```
3. Boot from USB and select "Install QuantumEnergyOS"
4. Follow the installation wizard

### AUR Installation (Future)

```bash
yay -S qeos
```

### Manual Installation

```bash
# Extract the ISO
sudo mount -o loop qeos-03.0-x86_64.iso /mnt

# Copy files to target
sudo cp -r /mnt/* /target/

# Chroot and configure
sudo arch-chroot /target
```

---

## Configuration

### Daemon Configuration

Each daemon is configured via `/etc/qeos/<daemon>/config.toml`. Example for `quantumd`:

```toml
[quantum]
enabled = true
max_qubits = 32
default_shots = 1024

[backends]
local.enabled = true
qiskit.enabled = false
azure_quantum.enabled = false

[security]
tls = true
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `QEOS_LOG_LEVEL` | Logging verbosity | `info` |
| `QEOS_DATA_DIR` | Data persistence path | `/var/lib/qeos` |
| `QEOS_CONFIG_DIR` | Config directory | `/etc/qeos` |

---

## Service Management

### Starting Services

```bash
# Start all QuantumEnergyOS services
sudo systemctl start qeos.target

# Or start individually
sudo systemctl start quantumd
sudo systemctl start energyd
sudo systemctl start climated
sudo systemctl start photonicd
sudo systemctl start aicored
```

### Checking Status

```bash
sudo systemctl status qeos.target
sudo systemctl status quantumd
```

### Logs

```bash
# Journal logs
journalctl -u quantumd -f

# Application logs
tail -f /var/log/qeos/quantumd.log
```

---

## Testing

### Unit Tests

```bash
cargo test --workspace
pytest tests/
```

### Integration Tests

```bash
cargo test --test '*' --workspace
make integration
```

### Benchmarks

```bash
cargo bench --workspace
python -m benchmarks.run
```

---

## Troubleshooting

### Common Issues

1. **Port already in use**: Check with `ss -tlnp | grep <port>`
2. **Permission denied**: Ensure user is in `qeos` group
3. **Build fails**: Ensure Rust 1.75+ and Python 3.12+ installed

### Getting Help

- Documentation: https://qeos.dev/docs
- Issues: https://github.com/GiovannyCorpus/QuantumEnergyOS/issues

---

*QuantumEnergyOS V.03 - Giovanny Anthony Corpus Bernal*
