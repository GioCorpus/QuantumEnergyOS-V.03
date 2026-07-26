# QuantumEnergyOS V.03
# Based on Arch Linux

# Build configuration
QEOS_VERSION = "03.0"
QEOS_NAME = "QuantumEnergyOS"
QEOS_ISO_NAME = "qeos-${QEOS_VERSION}-x86_64"
QEOS_LABEL = "QEOS ${QEOS_VERSION}"

# Architecture
ARCH = "x86_64"

# Output directory
OUTDIR = "out"

# Packages to install
PACMAN_PACKAGES = (
    base
    base-devel
    linux
    linux-firmware
    linux-headers
    
    # System utilities
    sudo
    systemd
    systemd-sysvcompat
    udev
    efibootmgr
    grub
    dosfstools
    mtools
    os-prober
    
    # Shell and terminal
    zsh
    tmux
    htop
    neofetch
    
    # System tools
    git
    curl
    wget
    jq
    openssh
    rsync
    python
    python-pip
    
    # Filesystems
    btrfs-progs
    xfsprogs
    nilfs-utils
    
    # Network
    networkmanager
    iptables-nft
    
    # Security
    polkit
    audit
)

AUR_PACKAGES = (
    # Development tools
    rustup
    go
    
    # Python scientific stack
    python-pip
    
    # Utilities
    zsh-autosuggestions
    zsh-syntax-highlighting
)

# Custom scripts
CUSTOM_SCRIPTS = (
    "add_group_wheel"
    "enable_systemd"
    "add_user"
    "set_pacman_mirrors"
    "add_default_services"
    "add_qeos_user"
)

# Bootloader configuration
GRUB_TIMEOUT = 5
GRUB_DISTRIBUTOR = "QuantumEnergyOS"
GRUB_CMDLINE_LINUX_DEFAULT = "loglevel=3 quiet splash"
GRUB_CMDLINE_LINUX = ""

# Filesystem configuration
MOUNT_POINTS = (
    "/dev/BOOT_DEVICE /boot vfat defaults 0 0"
    "/dev/ROOT_DEVICE / btrfs defaults,compress=zstd 0 0"
    "/dev/HOME_DEVICE /home btrfs defaults,compress=zstd 0 0"
    "/dev/SWAP_DEVICE none swap defaults 0 0"
)
