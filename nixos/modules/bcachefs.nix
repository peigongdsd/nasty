# bcachefs kernel module + userspace tools
#
# bcachefs was removed from the mainline kernel in 6.18 and is now an
# out-of-tree DKMS module.  This module wires up both pieces.
#
# Setting boot.bcachefs.package tells the NixOS bcachefs module to use our
# package for both the userspace tools AND the DKMS kernel module (derived
# via passthru.kernelModule).  A single setting keeps both in sync.
#
# Switching to upstream HEAD (needed for bcachefs subvolume list /
# list-snapshots which were added after v1.36.1):
#
#   1. In flake.nix, uncomment the bcachefs-tools input block.
#   2. This file requires no changes — nasty-bcachefs-tools defaults to
#      pkgs.bcachefs-tools when the flake input is absent.
#
# To revert to pure NixOS: comment out the flake input (step 1 above).

{ config, pkgs, nasty-bcachefs-tools ? pkgs.bcachefs-tools, ... }:

{
  boot.supportedFilesystems = [ "bcachefs" ];

  # NixOS derives boot.extraModulePackages automatically from
  # boot.bcachefs.package.passthru.kernelModule — no need to set it manually.
  boot.bcachefs.package = nasty-bcachefs-tools;

  environment.systemPackages = [ nasty-bcachefs-tools ];
}
