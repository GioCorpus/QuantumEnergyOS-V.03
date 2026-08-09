Quantum Browser Platform — scaffold

This folder contains a scaffold for the Quantum Browser Platform: a native OS subsystem that integrates a Rust backend (daemon) with a Tauri-based frontend (React + TypeScript).

Goals of the scaffold
- Provide a minimal, working Rust backend exposing a small HTTP management API (axum + tokio).
- Provide a minimal Tauri app skeleton (React + TypeScript) that can call native commands.
- Include a GitHub Actions CI workflow that runs Rust tests and builds the frontend.

Tech choices (justification):
- tokio + axum: lightweight async runtime and ergonomic HTTP API for the backend control plane.
- serde: for JSON (configuration and API payloads).
- tauri + React + Vite: embed a performant, secure WebView with native commands and cross-platform packaging.

Next steps (what to implement after this scaffold):
- Implement Browser Manager providers for Brave/LibreWolf (install, detect, profile management).
- Implement the Dashboard Manager, Security Manager, Profile Manager domain services.
- Add storage (sqlx + Postgres or embedded redb/sled) for profiles and policy storage.
- Expand unit and integration tests (backend + frontend) and add Playwright end-to-end tests.

See files:
- backend/: Rust daemon (axum) exposing the Dashboard API.
- tauri-app/: Tauri application skeleton (frontend + src-tauri bridge).
- .github/workflows/quantum-browser-platform-ci.yml: CI for tests and builds.

License: follow repository license.
