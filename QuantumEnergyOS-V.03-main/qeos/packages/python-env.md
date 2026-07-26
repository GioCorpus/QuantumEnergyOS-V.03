# Python Environment — QuantumEnergyOS V.03

> **Scope**: Base Python environment and setup scripts.
> Future dependencies are pinned and labeled.

## Requirements Groups

```text
qeos/packages/python-requirements.txt
```

| Group | Scope |
|-------|-------|
| `core` | Base scientific stack (numpy, scipy, pandas, matplotlib) |
| `research` | Quantum (qiskit, pennylane) and AI (torch, transformers) |
| `dev` | Lint, test, formatting (pytest, black, ruff) |

## Setup Script

```bash
python scripts/setup-python-env.py          # Generates requirements.txt
python -m venv .venv
source .venv/bin/activate
pip install -r packages/python-requirements.txt
```

## Version Pinning

All experimental packages are pinned to minor versions in
`packages/python-requirements-lock.txt` and updated via `pip-compile`.

### Known Future Items (not in current requirements)

| Package | Proposed Use | Classification |
|---------|-------------|----------------|
| `xarray` | Climate data cubes | [Production Ready] |
| `optuna` | Hyperparameter search | [Research Prototype] |
| `nanoq` | Quantum annealing | [Experimental] |
| `triton` | GPU kernels | [Research Prototype] |
| `openvdb` | Volumetric simulation | [Long-Term Vision] |

---

*QuantumEnergyOS V.03*
