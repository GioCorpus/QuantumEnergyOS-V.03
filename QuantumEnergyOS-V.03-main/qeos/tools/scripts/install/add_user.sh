#!/usr/bin/env bash
#
# qeos-add-user - Add user to QuantumEnergyOS live environment

set -euo pipefail

username="${1:-qeos}"
password="${2:-quantum}"

echo "[*] Adding user: ${username}"

# Create user
useradd -m -G wheel,audio,video,storage,optical -s /bin/zsh "${username}"

# Set password
echo "${username}:${password}" | chpasswd

# Enable sudo for wheel group
sed -i 's/^# %wheel ALL=(ALL:ALL) ALL/%wheel ALL=(ALL:ALL) ALL/' /etc/sudoers

# Create user directories
mkdir -p "/home/${username}/.config"
mkdir -p "/home/${username}/.local/share"
mkdir -p "/home/${username}/Documents"
mkdir -p "/home/${username}/Projects"
mkdir -p "/home/${username}/.ssh"

# Set ownership
chown -R "${username}:${username}" "/home/${username}"

echo "[*] User ${username} created"
