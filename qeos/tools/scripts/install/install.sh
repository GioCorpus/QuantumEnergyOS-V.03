#!/usr/bin/env bash
#
# install - QuantumEnergyOS installation script

set -euo pipefail

echo "==========================================="
echo "  QuantumEnergyOS V.03 Installer"
echo "==========================================="
echo ""
echo "WARNING: This will install QuantumEnergyOS to your system."
echo "Make sure you have backed up all important data."
echo ""

read -p "Continue? [y/N] " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Installation aborted."
    exit 1
fi

# Detect disks
echo "[*] Available disks:"
lsblk -d -o NAME,SIZE,TYPE,MODEL | grep disk
echo ""

read -p "Enter target disk (e.g., sda, nvme0n1): " DISK
DISK="/dev/${DISK}"

echo "[*] Target disk: ${DISK}"
read -p "This will ERASE ALL DATA on ${DISK}. Confirm? [y/N] " -n 1 -r
echo

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Installation aborted."
    exit 1
fi

# Partition disk
echo "[*] Partitioning disk..."
parted -s "${DISK}" mklabel gpt
parted -s "${DISK}" mkpart ESP fat32 1MiB 512MiB
parted -s "${DISK}" set 1 boot esp
parted -s "${DISK}" mkpart primary btrfs 512MiB 100%

# Format filesystems
echo "[*] Formatting filesystems..."
mkfs.fat -F32 "${DISK}p1"
mkfs.btrfs -f "${DISK}p2"

# Mount
echo "[*] Mounting filesystems..."
mount "${DISK}p2" /mnt
mkdir -p /mnt/boot
mount "${DISK}p1" /mnt/boot

# Install base system
echo "[*] Installing base system..."
pacstrap /mnt base base-devel linux linux-firmware linux-headers

# Generate fstab
echo "[*] Generating fstab..."
genfstab -U /mnt >> /mnt/etc/fstab

# Chroot setup
echo "[*] Configuring system..."
arch-chroot /mnt ln -sf /usr/share/zoneinfo/UTC /etc/localtime
arch-chroot /mnt hwclock --systohc
arch-chroot /mnt locale-gen
arch-chroot /mnt mkinitcpio -P

# Install GRUB
echo "[*] Installing GRUB bootloader..."
arch-chroot /mnt grub-install --target=x86_64-pc "${DISK}"
arch-chroot /mnt grub-mkconfig -o /boot/grub/grub.cfg

# Set root password
echo "[*] Set root password:"
arch-chroot /mnt passwd

# Create qeos user
echo "[*] Creating qeos user..."
arch-chroot /mnt useradd -m -G wheel,audio,video -s /bin/zsh qeos
echo "qeos:qeos" | arch-chroot /mnt chpasswd
arch-chroot /mnt sed -i 's/^# %wheel ALL=(ALL:ALL) ALL/%wheel ALL=(ALL:ALL) ALL/' /etc/sudoers

echo ""
echo "==========================================="
echo "  Installation Complete!"
echo "==========================================="
echo ""
echo "Reboot and remove the installation media."
echo "Login as root or qeos (password: qeos)"
