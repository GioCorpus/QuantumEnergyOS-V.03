#!/usr/bin/env bash
set -euo pipefail

ISO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK_DIR="/tmp/qeos-airootfs"
OUT_DIR="${ISO_DIR}/out"

# Base system (mirror of host for convenience)
PKGS=(
  base base-devel linux linux-headers linux-firmware
  git curl wget jq neofetch htop btop
  zsh zsh-completions zsh-syntax-highlighting
  neovim tmux python python-pip
  systemd systemd-sysvcompat udev networkmanager
)

echo "[*] Populating airootfs..."
rm -rf "${WORK_DIR}"
mkdir -p "${WORK_DIR}"

for pkg in "${PKGS[@]}"; do
  mkdir -p "${WORK_DIR}/var/lib/pacman/local"
done

# Bind host /usr for convenience (toolchain reuse)
mkdir -p "${WORK_DIR}/usr"
mount --bind /usr "${WORK_DIR}/usr"

# Copy profile overlay
cp -r "${ISO_DIR}/." "${WORK_DIR}/"

# ============================================================
# QEOS daemon packages (placeholders)
# ============================================================
echo "[*] Creating QEOS daemon package placeholders..."
QEOS_PKGS=(quantumd energyd climated photonicd aicored)
for pkg in "${QEOS_PKGS[@]}"; do
  mkdir -p "${WORK_DIR}/var/lib/pacman/local/${pkg}"
done

mkdir -p "${WORK_DIR}/usr/bin"
for daemon in "${QEOS_PKGS[@]}"; do
  install -Dm755 /dev/null "${WORK_DIR}/usr/bin/${daemon}"
  cat > "${WORK_DIR}/usr/lib/systemd/system/${daemon}.service" <<EOF
[Unit]
Description=QuantumEnergyOS ${daemon}
After=network.target

[Service]
Type=simple
ExecStart=/usr/bin/${daemon}
Restart=on-failure
User=qeos
Group=qeos

[Install]
WantedBy=multi-user.target
EOF
done

echo "[*] setup complete."
echo "    Work: ${WORK_DIR}"
echo "    Out: ${OUT_DIR}"
