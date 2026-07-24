# QuantumEnergyOS `kernel/` — Refactor Report

Scope: `kernel/quantum_photonic/photonic_core.py` and
`kernel/quantum_photonic/quantum_crypto_bridge.rs` (V.03). One fix outside
that scope is included because it was found while tracing `kernel/`'s
dependencies and blocks the whole `photonic-core` crate from compiling.

**Not verified by compiling** — this sandbox has no Rust toolchain
(`cargo`/`rustc` unavailable), so the Rust changes are reviewed and
internally consistent but not build-checked. The Python changes *are*
verified: 13 tests pass, including regression tests for the entanglement
bug (see below).

---

## 1. Architecture review

**`kernel/` was dead code.** Neither file was reachable by the build:
- `quantum_crypto_bridge.rs` had no `Cargo.toml` and wasn't listed in the
  workspace `[members]` in the root `Cargo.toml` — it could not compile,
  link, or be tested by anything.
- `photonic_core.py` had no `__init__.py` anywhere above it and nothing in
  `api/` or elsewhere imports from `kernel.quantum_photonic`.

**Name collision:** `kernel/quantum_photonic/quantum_crypto_bridge.rs`
defines `pub struct PhotonicBridge` for QKD session management. The
*separate*, actually-wired-in `photonic-bridge` crate also defines
`pub struct PhotonicBridge` — for a Vulkan/display color pipeline. Same
name, two unrelated responsibilities, in two different crates.

**Bonus find, not in original scope:** `photonic-core/src/lib.rs` declares
`pub mod quartz_4d;` but the file on disk is `quartz_5d.rs` — there is no
`quartz_4d.rs`. As shipped, `cargo build -p photonic-core` (and therefore
the whole workspace) fails with a "file not found for module" error. This
is exactly the legacy-naming issue your refactor spec calls out
("Replace legacy Quartz4D terminology") — it just wasn't fully finished.
One-line fix included in `photonic-core-patch/lib.rs`.

**`photonic_core.py` mixed four responsibilities in one 606-line file:**
component models, error correction, hardware-manager orchestration, and
quantum algorithms (VQE/QAOA/optical matmul) — no separation of concerns,
hard to test any one piece in isolation.

## 2. Detected problems

| # | File | Problem | Severity |
|---|------|---------|----------|
| 1 | `photonic_core.py` | `entangled_with` set via Python `id()`, never matches the hardware manager's own registry ids — the field is write-only, never resolves | Correctness bug |
| 2 | `photonic_core.py` | `detector_type: str` / `quadrature: str` flags — a typo silently falls through to the wrong branch instead of erroring | Correctness / API safety |
| 3 | `photonic_core.py` | `allocate_detector('SPD')` builds a `HomodyneDetector` — the SPD branch never built an SPD-specific model, silently mislabeling it | Correctness bug |
| 4 | `photonic_core.py` | Error-correction handlers mutated the caller's `PhotonicState` in place while returning it | API footgun |
| 5 | `quantum_crypto_bridge.rs` | `.unwrap()` on every mutex lock | No error recovery, spec violation |
| 6 | `quantum_crypto_bridge.rs` | `Result<_, &'static str>` — no error type, can't `match` on failure kind | Not idiomatic Rust |
| 7 | `quantum_crypto_bridge.rs` | `timeout_ms` and `photon_repetitions` config fields defined, never read anywhere | Dead config, misleading API |
| 8 | `quantum_crypto_bridge.rs` | `active_sessions` grows forever, nothing ever removes an entry | Unbounded memory growth |
| 9 | `quantum_crypto_bridge.rs` | Hand-rolled `Complex { re, im }` duplicating `num_complex::Complex64`, already a dependency elsewhere in the workspace | Duplicated logic |
| 10 | `quantum_crypto_bridge.rs` | `struct PhotonicBridge` — name collision with `photonic-bridge` crate | Module boundary confusion |
| 11 | both files | No tests at all | Spec violation |

## 3. What changed

**Python** — `photonic_core.py` (606 lines, one file) →
`kernel/quantum_photonic/` package:

- `types.py` — enums only (`Quadrature`, `DetectorKind`, `ErrorSyndrome`,
  `QubitEncoding`, `PhotonicComponentType`), replacing the `str` flags (#2, #3)
- `components.py` — `PhotonicState`, `Waveguide`, `MachZehnderInterferometer`,
  `HomodyneDetector`, `EntangledPhotonSource`
- `error_correction.py` — `PhotonicErrorCorrection`, now returns a new
  state via `dataclasses.replace` instead of mutating in place (#4)
- `hardware_manager.py` — `PhotonicHardwareManager`; `create_entanglement`
  now writes the *same* registry ids onto `state.entangled_with` that it
  stores in `_entanglement_registry`, so the field actually resolves (#1)
- `algorithms.py` — `PhotonicQuantumOperations` (VQE/QAOA/optical matmul),
  unchanged math, now depends only on the manager's public API
- `tests/test_photonic_core.py` — 13 tests, **all passing**, including a
  regression test asserting `hw._entanglement_registry[qubit_a] == qubit_b`

**Rust** — `quantum_crypto_bridge.rs` (orphaned, unbuildable) → new
workspace member `qkd-bridge/`:

- `Cargo.toml` added, crate added to root workspace `[members]` — this
  code is now actually part of the build for the first time
- `PhotonicBridge` renamed to `QuantumCryptoBridge` (#10)
- New `BridgeError` enum (`LockPoisoned`, `Timeout`, `SessionNotFound`)
  implementing `std::error::Error`; all `.unwrap()` calls replaced with
  `?`-propagated errors (#5, #6)
- `timeout_ms` now actually checked; `photon_repetitions` now actually
  feeds a repetition-code-style error-rate model (#7)
- `active_sessions` capped at `MAX_TRACKED_SESSIONS`, oldest evicted on
  overflow (#8)
- `quantum_ops::PhotonicQubit` now uses `num_complex::Complex64` instead
  of a hand-rolled complex type (#9)
- 7 unit tests added (session tracking, typed-error matching, timeout
  math, eviction bound, Hadamard involution, normalization)

**Bonus:** `photonic-core-patch/lib.rs` — one-line fix,
`quartz_4d` → `quartz_5d`, unbreaks the workspace build.

## 4. Compatibility notes

- Public function signatures changed for `allocate_detector` (now takes
  `DetectorKind`, not `str`) and `measure_homodyne`/`measure_quadrature`
  (now take `Quadrature`, not `str`). Any caller outside this package
  using the old string API will need updating — I didn't find any (no
  imports of `kernel.quantum_photonic` elsewhere in the repo).
- `QuantumCryptoBridge::run_bb84` and `::get_session` now return
  `Result<_, BridgeError>` instead of `Result<_, &'static str>` — same
  shape, different error type, so `match`/`.unwrap()` call sites need a
  one-line update if any exist. None found (the file wasn't wired in
  anywhere, so there are no call sites yet).
- To actually apply: drop `kernel/` in place of the old one, drop
  `qkd-bridge/` at the workspace root, apply `photonic-core-patch/`
  (`lib.rs` and the `Cargo.toml` members list) to `photonic-core/src/lib.rs`
  and the root `Cargo.toml`.

## 5. Not done (would need a separate pass)

- `photonic-core/src/photonic_core.rs` (598 lines, a *third*, differently-scoped
  "photonic core" module — this one bridges to real IBM Qiskit/Q# backends)
  overlaps in name and concept with `kernel/quantum_photonic/photonic_core.py`.
  Worth a naming/ownership decision: which one is "the" photonic core?
- Compile-verifying the Rust changes (no toolchain in this environment).
- The rest of the 408-file project — this was scoped to `kernel/` only,
  per your earlier choice.
