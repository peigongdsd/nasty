{ config, pkgs, nasty-middleware, ... }:

{
  boot.supportedFilesystems = [ "bcachefs" ];

  environment.systemPackages = with pkgs; [
    bcachefs-tools
    parted
    gptfdisk
    nvme-cli
    util-linux
    e2fsprogs
    dosfstools     # for EFI system partition
    nixos-install-tools

    # Guided installer script
    (writeShellScriptBin "nasty-install" ''
      set -euo pipefail

      echo "=== NASty NAS Guided Installer ==="
      echo ""

      # List available disks
      echo "Available disks:"
      lsblk -d -o NAME,SIZE,MODEL | grep -v loop
      echo ""

      read -p "Enter disk to install to (e.g., sda): " DISK
      DISK="/dev/$DISK"

      if [ ! -b "$DISK" ]; then
        echo "Error: $DISK is not a block device"
        exit 1
      fi

      echo ""
      echo "WARNING: This will ERASE all data on $DISK"
      read -p "Continue? (yes/no): " CONFIRM
      if [ "$CONFIRM" != "yes" ]; then
        echo "Aborted."
        exit 0
      fi

      echo ""
      echo "==> Partitioning $DISK..."
      parted -s "$DISK" -- \
        mklabel gpt \
        mkpart ESP fat32 1MiB 512MiB \
        set 1 esp on \
        mkpart root ext4 512MiB 100%

      PART1="''${DISK}1"
      PART2="''${DISK}2"

      # Handle NVMe naming
      if [[ "$DISK" == *nvme* ]]; then
        PART1="''${DISK}p1"
        PART2="''${DISK}p2"
      fi

      echo "==> Formatting partitions..."
      mkfs.fat -F32 "$PART1"
      mkfs.ext4 -F "$PART2"

      echo "==> Mounting..."
      mount "$PART2" /mnt
      mkdir -p /mnt/boot
      mount "$PART1" /mnt/boot

      echo "==> Generating NixOS configuration..."
      nixos-generate-config --root /mnt

      # Append NASty config
      cat >> /mnt/etc/nixos/configuration.nix << 'NIXEOF'

  # NASty NAS
  services.nasty.enable = true;
  boot.supportedFilesystems = [ "bcachefs" ];
NIXEOF

      echo "==> Installing NixOS (this may take a while)..."
      nixos-install --no-root-passwd

      echo ""
      echo "=== Installation complete! ==="
      echo "Set a root password, then reboot."
      echo "The NASty WebUI will be available at https://<ip>/"
      echo "Default login: admin / admin"
      echo ""
      read -p "Set root password now? (yes/no): " SET_PW
      if [ "$SET_PW" = "yes" ]; then
        nixos-enter --root /mnt -c 'passwd'
      fi

      echo ""
      echo "Run 'reboot' when ready."
    '')
  ];

  # Enable networking for NixOS install
  networking.wireless.enable = pkgs.lib.mkForce false;
  networking.useDHCP = pkgs.lib.mkForce true;

  # Installer welcome message
  services.getty.helpLine = ''

    ███╗   ██╗ █████╗ ███████╗████████╗██╗   ██╗
    ████╗  ██║██╔══██╗██╔════╝╚══██╔══╝╚██╗ ██╔╝
    ██╔██╗ ██║███████║███████╗   ██║    ╚████╔╝
    ██║╚██╗██║██╔══██║╚════██║   ██║     ╚██╔╝
    ██║ ╚████║██║  ██║███████║   ██║      ██║
    ╚═╝  ╚═══╝╚═╝  ╚═╝╚══════╝   ╚═╝      ╚═╝

    NASty NAS Installer

    To install NASty:
      1. Partition your disk:    cfdisk /dev/sdX
      2. Format boot partition:  mkfs.fat -F32 /dev/sdX1
      3. Format root partition:  mkfs.ext4 /dev/sdX2
      4. Mount root:             mount /dev/sdX2 /mnt
      5. Mount boot:             mkdir -p /mnt/boot && mount /dev/sdX1 /mnt/boot
      6. Generate config:        nixos-generate-config --root /mnt
      7. Edit configuration:     nano /mnt/etc/nixos/configuration.nix
         - Add: services.nasty.enable = true;
      8. Install:                nixos-install --flake github:your-org/nasty#nasty
      9. Reboot:                 reboot

    Or run the guided installer:  nasty-install

  '';
}
