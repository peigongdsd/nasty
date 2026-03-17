# Override the Limine boot entry installer to include build date in each
# generation label, making all generations identifiable at a glance without
# needing to select each entry individually.
#
# The upstream installer puts the date only in `comment:`, which Limine
# shows solely for the focused entry (in the bottom status bar). There is
# no Limine or NixOS option to show comments inline for all entries.
#
# We replicate limineInstallConfig (the JSON the installer reads at runtime)
# from the same config.* sources the upstream module uses, then call
# pkgs.replaceVarsWith on our patched limine-install.py.  lib.mkForce
# overrides the upstream system.build.installBootLoader (priority 100)
# without requiring disabledModules.

{ config, lib, pkgs, ... }:

let
  cfg = config.boot.loader.limine;
  efi = config.boot.loader.efi;

  limineInstallConfig = pkgs.writeText "limine-install.json" (
    builtins.toJSON {
      nixPath                = config.nix.package;
      efiBootMgrPath         = pkgs.efibootmgr;
      liminePath             = cfg.package;
      efiMountPoint          = efi.efiSysMountPoint;
      fileSystems            = config.fileSystems;
      luksDevices            = builtins.attrNames config.boot.initrd.luks.devices;
      canTouchEfiVariables   = efi.canTouchEfiVariables;
      efiSupport             = cfg.efiSupport;
      efiRemovable           = cfg.efiInstallAsRemovable;
      secureBoot             = cfg.secureBoot;
      biosSupport            = cfg.biosSupport;
      biosDevice             = cfg.biosDevice;
      partitionIndex         = cfg.partitionIndex;
      force                  = cfg.force;
      enrollConfig           = cfg.enrollConfig;
      style                  = cfg.style;
      resolution             = cfg.resolution;
      maxGenerations         = if cfg.maxGenerations == null then 0 else cfg.maxGenerations;
      hostArchitecture       = pkgs.stdenv.hostPlatform.parsed.cpu;
      timeout                = if config.boot.loader.timeout != null then config.boot.loader.timeout else 10;
      enableEditor           = cfg.enableEditor;
      extraConfig            = cfg.extraConfig;
      extraEntries           = cfg.extraEntries;
      additionalFiles        = cfg.additionalFiles;
      validateChecksums      = cfg.validateChecksums;
      panicOnChecksumMismatch = cfg.panicOnChecksumMismatch;
    }
  );
in

{
  system.build.installBootLoader = lib.mkForce (pkgs.replaceVarsWith {
    src = ./limine-install.py;
    isExecutable = true;
    replacements = {
      python3     = pkgs.python3.withPackages (ps: [ ps.psutil ]);
      configPath  = limineInstallConfig;
    };
  });
}
