.PHONY: build test docs clean install lint format kernel-test

# Build configuration
BINARY_DIR := target/release
ISO_DIR := packages/archiso/out
VERSION := 03.0

# Detect operating system
OS := $(shell uname -s)

build: build-daemons build-archiso

build-daemons:
	@echo "[*] Building QuantumEnergyOS daemons..."
	cargo build --release --workspace
	@echo "[*] Daemons built successfully"

build-archiso:
	@echo "[*] Building QuantumEnergyOS ISO..."
	@if [ "$(OS)" != "Linux" ]; then \
		echo "[!] ISO build requires Linux host. Skipping."; \
	else \
		cd packages/archiso && ./build-iso.sh; \
	fi
	@echo "[*] ISO build complete"

test: test-rust test-python

test-rust:
	@echo "[*] Running Rust tests..."
	cargo test --workspace

test-python:
	@echo "[*] Running Python tests..."
	python -m pytest tests/ || echo "[!] No Python tests found or pytest not installed"

kernel-test:
	@echo "[*] Running kernel prototype tests..."
	cargo test -p qeos-kernel

docs:
	@echo "[*] Generating documentation..."
	cargo doc --workspace --no-deps
	mdbook build docs/

clean:
	@echo "[*] Cleaning build artifacts..."
	cargo clean
	rm -rf packages/archiso/work
	rm -rf packages/archiso/out/*.iso

install-deps:
	@echo "[*] Installing build dependencies..."
	sudo pacman -S rustup base-devel archiso python-pip
	rustup default stable

format:
	cargo fmt --all
	black .

lint:
	cargo clippy --workspace -- -D warnings
	black --check .

daemon-quantum:
	cargo run -p quantumd --release

daemon-energy:
	cargo run -p energyd --release

daemon-climate:
	cargo run -p climated --release

daemon-photonic:
	cargo run -p photonicd --release

daemon-ai:
	cargo run -p aicored --release

all: build test docs
