#!/usr/bin/env bash
set -euo pipefail
ISO_DIR="airootfs"
DEST="/tmp/airootfs"

PKGS+=(
    qeos-daemons
)

for pkg in "${PKGS[@]}"; do
    mkdir -p "$DEST/var/lib/pacman/local"
done
