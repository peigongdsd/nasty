{ config, pkgs, lib, nasty-middleware, nasty-webui, ... }:

let
  nastySrc = lib.cleanSource ./..;
in
{
  # Pre-built packages in the ISO's Nix store so nixos-install
  # can reuse them instead of recompiling from source.
  system.extraDependencies = [ nasty-middleware ]
    ++ lib.optional (nasty-webui != null) nasty-webui;

  # Bundle NASty source on the ISO for flake-based installation
  environment.etc."nasty-src".source = nastySrc;

  boot.supportedFilesystems = [ "bcachefs" ];

  environment.systemPackages = with pkgs; [
    bcachefs-tools
    parted
    gptfdisk
    nvme-cli
    util-linux
    e2fsprogs
    dosfstools
    nixos-install-tools
    git  # required for Nix flakes

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

      # Handle NVMe / MMC naming
      if [[ "$DISK" == *nvme* ]] || [[ "$DISK" == *mmcblk* ]]; then
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

      echo "==> Copying NASty source..."
      cp -r /etc/nasty-src /mnt/etc/nixos
      chmod -R u+w /mnt/etc/nixos

      echo "==> Generating hardware configuration..."
      nixos-generate-config --root /mnt --dir /tmp/hw-config
      cp /tmp/hw-config/hardware-configuration.nix /mnt/etc/nixos/nixos/hardware-configuration.nix

      # Flakes require a git repo to resolve paths
      cd /mnt/etc/nixos
      git init -q
      git add .

      echo "==> Installing NixOS with NASty..."
      echo "    (this may take a while on first install)"
      nixos-install --flake /mnt/etc/nixos/nixos#nasty --no-root-passwd

      echo ""
      echo "=== Installation complete! ==="
      echo ""
      echo "  The NASty WebUI will be available at https://<ip>/"
      echo "  Default login: admin / admin"
      echo ""
      echo "  To reconfigure later:"
      echo "    nixos-rebuild switch --flake /etc/nixos/nixos#nasty"
      echo ""

      read -p "Set root password now? (yes/no): " SET_PW
      if [ "$SET_PW" = "yes" ]; then
        nixos-enter --root /mnt -c 'passwd'
      fi

      echo ""
      echo "Run 'reboot' when ready."
    '')
  ];

  # Enable networking for installation
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

    Run the guided installer:  nasty-install

    Or install manually:
      1. Partition your disk:    cfdisk /dev/sdX
      2. Format & mount partitions
      3. Copy source:            cp -r /etc/nasty-src /mnt/etc/nixos
      4. Generate hardware config:
           nixos-generate-config --root /mnt --dir /tmp/hw
           cp /tmp/hw/hardware-configuration.nix /mnt/etc/nixos/nixos/
      5. Init flake repo:        cd /mnt/etc/nixos && git init && git add .
      6. Install:                nixos-install --flake /mnt/etc/nixos/nixos#nasty
      7. Reboot:                 reboot

  '';
}
