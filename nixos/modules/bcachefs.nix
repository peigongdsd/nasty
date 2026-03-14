# bcachefs kernel module + userspace tools
#
# bcachefs was removed from the mainline kernel in 6.18 and is now an
# out-of-tree DKMS module.  This module wires up both pieces.
#
# Switching to upstream HEAD tools (needed for `bcachefs subvolume list`
# and `bcachefs subvolume list-snapshots` which were added after v1.36.1):
#
#   1. In flake.nix, uncomment the bcachefs-tools input block.
#   2. In flake.nix mkNixosConfigs, the `nasty-bcachefs-tools` specialArg
#      is already wired — no change needed there.
#   3. This file already uses `nasty-bcachefs-tools` — no change needed here.
#
# To revert to pure NixOS: undo step 1 above. The `nasty-bcachefs-tools`
# arg defaults to `pkgs.bcachefs-tools` when the flake input is absent,
# so the rest of the config just works.

{ config, pkgs, nasty-bcachefs-tools ? pkgs.bcachefs-tools, ... }:

{
  boot.supportedFilesystems = [ "bcachefs" ];

  # DKMS module versioned to match the running kernel exactly
  boot.extraModulePackages = [ config.boot.kernelPackages.bcachefs ];

  environment.systemPackages = [ nasty-bcachefs-tools ];
}
