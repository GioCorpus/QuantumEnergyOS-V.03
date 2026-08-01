# Build Guide — QuantumEnergyOS V.03 ISO

> **Technical Specification for Compilation, Packaging, and ISO Image Generation**  
> **Document Version:** 2.0.3  
> **Target Operating System:** Linux x86_64  
> **Architecture Profile:** ArchISO / LiveCD / LiveUSB  
> **Status:** Standard Deployment Document  

---

## 1. Executive Summary & General Architecture

**QuantumEnergyOS V.03** is a high-performance, specialized Linux distribution designed for hybrid quantum-photonic energy orchestration, climate grid balancing, and real-time smart grid optimization.

This guide establishes the standardized build protocol for compiling native Rust binaries, setting up the Python orchestration environment, packaging the React/TypeScript web management dashboard, and assembling the bootable **ISO (LiveCD/LiveUSB)** image using the `mkarchiso` build engine.

> ⚠️ **Critical Security & Permissions Note:**  
> Generating the *chroot* namespace and the compressed *SquashFS* filesystem requires **superuser privileges (`root` or `sudo`)** on an Arch Linux-based host environment or via privileged Docker containers.

---

## 2. System & Infrastructure Prerequisites

Prior to initializing the build pipeline, the host node must meet the following technical hardware and software requirements:

### 2.1 Hardware Specifications

| Resource / Component | Minimum Requirement | Recommended Specification |
| :--- | :--- | :--- |
| **CPU Architecture** | x86_64 with Virtualization Support (VT-x / AMD-V) | Multi-core x86_64 CPU (8+ physical cores) |
| **RAM Memory** | 8 GB RAM | 16 GB+ RAM (Optimizes compilation in RAM-disk) |
| **Free Storage** | 25 GB Available Disk Space | 50 GB NVMe SSD (For dynamic workspace) |

### 2.2 Software Tools & Toolchains

* **ISO Environment Management:** `archiso`, `mkarchiso`, `git`
* **Rust Build Environment:** `rustc` 1.70+, `cargo`, stable toolchain `x86_64-unknown-linux-gnu`
* **Python & AI Runtime:** `python` 3.10+, `python-pip`, `python-numpy`, `python-scipy`
* **Web Environment & Dashboard:** `nodejs` 18+ (LTS recommended), `npm`
* **Virtualization & Testing:** `qemu-system-x86_64`, `kvm`
* **Containers (Optional):** `docker` or `podman`

---

## 3. Code Repository Directory Structure

The ISO packaging process relies on the hierarchical organization of folders and files within the main project repository:

```text
QuantumEnergyOS-V.03-main/
├── AppData/
│   └── Local/Programs/Microsoft VS Code/QuantumEnergyOS/
│       ├── buildsystem/
│       │   └── build.ps1             # PowerShell automation script for Windows/cross-builds
│       ├── climate_orchestrator/     # Python module for thermal & climate orchestration
│       ├── livecd/
│       │   └── profiledef/
│       │       └── packages.x86_64   # Master Arch Linux package list for the ISO
│       ├── photonic-bridge/          # Low-level Rust interface for optical circuits
│       ├── photonic-core/            # Rust optical optimization & calculation core
│       ├── web-dashboard/            # React / TypeScript user interface
│       ├── Dockerfile                # Isolated container build environment config
│       └── Makefile                  # Unified automation pipeline for Linux
├── Build-ISO.ps1                     # Root execution script for PowerShell
├── Makefile                          # Primary entrypoint for 'make iso'
└── GUIA-CONSTRUCCION-ISO.pdf         # Reference technical documentation
```

---

## 4. Module Pre-Compilation & Component Assembly

Before injecting applications into the root filesystem template (`airootfs`), all underlying core services must be compiled.

### 4.1 Compiling Photonic Cores (Rust)

Compilation in *Release* mode with microarchitecture-level performance optimizations:

```bash
# 1. Compile the Photonic Optimizer (Photonic Core)
cd photonic-core
cargo build --release --target x86_64-unknown-linux-gnu

# 2. Compile the Optical Interconnect Bridge (Photonic Bridge)
cd ../photonic-bridge
cargo build --release --target x86_64-unknown-linux-gnu
```

### 4.2 Packaging the Web Dashboard (React & TypeScript)

Generating static production assets for the web monitoring interface:

```bash
cd ../web-dashboard
npm install
npm run build
```

---

## 5. ISO Profile Configuration (`livecd/profiledef`)

### 5.1 Package Dependency Declarations (`packages.x86_64`)

Edit or verify `livecd/profiledef/packages.x86_64` to include the required runtime dependencies:

```ini
# Base System & Kernel
linux
linux-firmware
base
base-devel

# Networking & Utilities
dhcpcd
iwd
networkmanager
openssh
curl
git

# QuantumEnergyOS Runtimes & Libraries
python
python-pip
python-numpy
python-scipy
rust
hwloc
opencl-icd-loader
```

### 5.2 Injecting Custom Binaries & Artifacts into LiveCD Overlay

Copy pre-compiled binaries into the `airootfs` profile directory layout:

```bash
# Create target directories inside the ISO filesystem
mkdir -p livecd/airootfs/usr/local/bin
mkdir -p livecd/airootfs/var/www/quantum-dashboard

# Copy compiled Rust executable binaries
cp photonic-core/target/release/photonic-core livecd/airootfs/usr/local/bin/
cp photonic-bridge/target/release/photonic-bridge livecd/airootfs/usr/local/bin/

# Copy compiled frontend web artifacts
cp -r web-dashboard/build/* livecd/airootfs/var/www/quantum-dashboard/
```

---

## 6. ISO Image Construction Workflow Execution

Choose the execution workflow best suited to your development environment:

### Option A: Makefile Automation (Recommended for Linux / Arch)

```bash
# Clean previous build workspaces
make clean

# Trigger full compilation and ISO build sequence
make iso
```

### Option B: Manual Execution via `mkarchiso` (Command Line)

```bash
# Create output directory
mkdir -p out/

# Run mkarchiso specifying work directory and profile definition
sudo mkarchiso -v -w work/ -o out/ livecd/profiledef/
```

> **Expected Result:**  
> Upon successful completion, the bootable image will be available at:  
> `out/QuantumEnergyOS-v0.03-x86_64.iso`

### Option C: Containerized Build (Docker / Isolated Environment)

To build the ISO on non-Arch Linux host distributions (Ubuntu, Debian, Fedora, etc.):

```bash
# 1. Build the compilation container image
docker build -t quantum-iso-builder -f Dockerfile .

# 2. Run container in privileged mode to generate the .iso file
docker run --privileged -v $(pwd)/out:/build/out quantum-iso-builder
```

---

## 7. Verification, Testing & Virtual Environment Validation

Prior to flashing the image onto physical installation media (USB/DVD), validate booting and service operations using QEMU with KVM acceleration:

```bash
qemu-system-x86_64     -enable-kvm     -m 4096     -smp 4     -cdrom out/QuantumEnergyOS-v0.03-x86_64.iso     -boot d     -vga virtio     -net nic,model=virtio     -net user,hostfwd=tcp::8080-:80
```

---

## 8. ISO Standards Compliance & Software Quality

The packaging process and integrated components in **QuantumEnergyOS V.03** comply with international industry standards:

* **ISO/IEC 27001 (Information Security):** Superuser privilege isolation, cryptographic integrity checks, and strict purging of default private keys.
* **ISO 9001 (Quality Management):** Reproducible compilation workflow via automated scripts (`Makefile` and `mkarchiso`).
* **ISO 14001 (Environmental Management):** Embedded optimization algorithms in `photonic-core` designed to reduce data center carbon footprints.
* **ISO/IEC 25010 (Software System Quality):** Native memory safety guarantees provided by the Rust codebase.

---

## 9. Troubleshooting & Common Failures Matrix

| Issue / Error | Probable Root Cause | Resolution Procedure |
| :--- | :--- | :--- |
| `pacman lock file error` | Abrupt termination of a prior build left an active lock file. | Execute: `sudo rm -f work/x86_64/airootfs/var/lib/pacman/db.lck` and restart build. |
| `No space left on device` | Insufficient space in temporary `/tmp` or `work/` directory. | Redirect workspace to a partition with 30GB+ space using `-w /path/to/disk`. |
| Dynamic library error (`libopencl.so`) | Missing dependency declaration in profile file. | Add missing package name to `livecd/profiledef/packages.x86_64` and rebuild. |
| Rust target triple failure | Mismatch of *target triple* between host machine and guest ISO. | Explicitly enforce the build target: `cargo build --target x86_64-unknown-linux-gnu --release`. |

---

> 🔒 **Final Deployment Warning:** Always ensure all default `.env` files, temporary SSH private keys, and test certificates are purged or replaced before official or commercial distribution of the final ISO image.
