# API Reference — QuantumEnergyOS V.03

> **Existing endpoints only.** Endpoints marked [Planned] are proposed; do not treat as committed.

## Table of Contents

1. [Common Conventions](#common-conventions)
2. [quantumd](#quantumd)
3. [energyd](#energyd)
4. [climated](#climated)
5. [photonicd](#photonicd)
6. [aicored](#aicored)

---

## Common Conventions

| Header | Description |
|--------|-------------|
| `X-Correlation-ID` | Distributed trace ID |
| `X-Request-ID` | Per-request identifier |

Errors:

```json
{
  "error": { "code": "INTERNAL", "message": "...", "request_id": "..." }
}
```

Health endpoints return:

```json
{ "status": "healthy|degraded|unhealthy", "timestamp": 1690000000, "version": "03.0" }
```

---

## quantumd

> **Classification**: [Production Ready]
>
> Listens on `config.bind_address:config.bind_port` (default `[::1]:8080`).

### Lifecycle

| Method | Path |
|--------|------|
| `GET` | `/health` |
| `POST` | `/shutdown` |

### Circuits

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/v1/circuits/execute` | Execute a circuit |

### Backends

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/v1/backends` | List available simulators |
| `GET` | `/v1/backends/{id}/health` | Backend health check |

### Metrics

| Method | Path |
|--------|------|
| `GET` | `/metrics` |

---

## energyd

> **Classification**: [Production Ready]

### Lifecycle

| Method | Path |
|--------|------|
| `GET` | `/health` |

### Grid Operations

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/v1/energy/latest` | Submit a reading |
| `GET` | `/v1/energy/readings` | Query readings |
| `POST` | `/v1/energy/optimize` | Compute optimization |

### Metrics

| Method | Path |
|--------|------|
| `GET` | `/metrics` |

---

## climated

> **Classification**: [Production Ready]

### Lifecycle

| Method | Path |
|--------|------|
| `GET` | `/health` |

### Climate Operations

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/v1/climate/snapshot` | Ingest observation |
| `GET` | `/v1/climate/snapshot/{id}` | Retrieve snapshot |
| `POST` | `/v1/climate/forecast` | Generate forecast |
| `GET` | `/v1/climate/alerts` | List active extreme-event alerts |

### Metrics

| Method | Path |
|--------|------|
| `GET` | `/metrics` |

---

## photonicd

> **Classification**: [Research Prototype]

### Lifecycle

| Method | Path |
|--------|------|
| `GET` | `/health` |

### Signal Processing

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/v1/photonic/generate` | Generate optical carrier |
| `POST` | `/v1/photonic/modulate` | Apply modulation |
| `POST` | `/v1/photonic/propagate` | Fiber simulation |

### Metrics

| Method | Path |
|--------|------|
| `GET` | `/metrics` |

---

## aicored

> **Classification**: [Research Prototype]

### Lifecycle

| Method | Path |
|--------|------|
| `GET` | `/health` |

### Inference & Training

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/v1/ai/predict` | Run local inference |
| `POST` | `/v1/ai/train` | Queue training job |
| `GET` | `/v1/ai/models` | List hosted models |

### Metrics

| Method | Path |
|--------|------|
| `GET` | `/metrics` |

---

*QuantumEnergyOS V.03 — Giovanny Anthony Corpus Bernal*
