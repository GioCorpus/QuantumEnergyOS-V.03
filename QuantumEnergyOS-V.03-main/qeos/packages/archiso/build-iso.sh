#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ISO_DIR="${SCRIPT_DIR}/airootfs"
WORK_DIR="${SCRIPT_DIR}/work"
OUT_DIR="${ISO_DIR}/out"
ARCHISO_PROFILE_DIR="${SCRIPT_DIR}"

# ============================================================
# Validate environment
# ============================================================
if [[ "$(uname -s)" != "Linux" ]]; then
  echo "[!] ISO build requires Linux host. Exiting."
  exit 1
fi

for cmd in mkarchiso pacstrap arch-chroot; do
  command -v "$cmd" &>/dev/null || { echo "[!] Missing: $cmd. Install archiso first."; exit 1; }
done

# ============================================================
# Versioning
# ============================================================
VER="03.0"
ISO_NAME="qeos-${VER}-x86_64-$(date +%Y%m%d).iso"

# ============================================================
# Prepare packaging
# ============================================================
echo "[*] Preparing airootfs overlay..."
mkdir -p "${WORK_DIR}"
mkdir -p "${OUT_DIR}"

# Recreate airootfs structure
rm -rf "${WORK_DIR}/airootfs"
mkdir -p "${WORK_DIR}/airootfs/"{etc,usr/bin,usr/lib/systemd/system,var/lib/pacman/local}

# Bind /usr for convenience (mirrors host toolchain)
mkdir -p "${WORK_DIR}/airootfs/usr"
mount --bind /usr "${WORK_DIR}/airootfs/usr"

# Copy archiso profile files
cp -r "${ARCHISO_PROFILE_DIR}/" "${WORK_DIR}/airootfs/"

# ============================================================
# Build
# ============================================================
echo "[*] Starting mkarchiso..."
mkarchiso -v -w "${WORK_DIR}/airootfs-work" -o "${OUT_DIR}" "${WORK_DIR}/airootfs"

echo "[*] ISO build complete: ${OUT_DIR}/${ISO_NAME}"
ls -lh "${OUT_DIR}"
