# FAQ

## Why does NASty exist?

Because bcachefs deserves a proper NAS appliance, and nobody was building one.

Honestly? Curiosity, caffeine, and Claude.

bcachefs is arguably the most interesting Linux filesystem since... well, since ever. But using it for NAS meant CLI-only. NASty wraps it in an appliance with a WebUI, NFS/SMB/iSCSI/NVMe-oF, a Kubernetes CSI driver, and NixOS for atomic updates. One human, one AI, zero business plan.

## Why bcachefs instead of ZFS?

ZFS is battle-tested and great. I'm not here to trash it. But:

- **bcachefs is GPL Linux.** No CDDL license friction. Kernel module builds are fully automated on NixOS — no manual DKMS steps.
- **Simpler model.** A "filesystem" is just a filesystem. Subvolumes are just directories. Snapshots are just snapshots. No datasets, zvols, pools-within-pools, or property inheritance trees.
- **Modern features out of the box.** Tiering (move cold data to slow disks automatically), erasure coding, and online filesystem repair — things that ZFS either doesn't have or requires third-party tools.
- **Active development.** Kent Overstreet is shipping features at a pace ZFS hasn't seen in years.

The tradeoff: bcachefs is younger and less proven. I'm comfortable with that for a project that's explicitly exploring what's next.

## Why NixOS?

Because a NAS appliance should be a single atomic unit that you can update, roll back, and reproduce.

- **Atomic updates.** `nixos-rebuild switch` either succeeds completely or doesn't change anything. No "halfway upgraded" state.
- **Rollback.** Every update creates a new generation. Boot into the previous one if something breaks.
- **Reproducible.** The entire system is defined in code. Two machines with the same config are identical.
- **No package manager conflicts.** Nix handles all dependencies in isolation. No `pacman -Syu` breaking your storage engine at 2am.

Traditional NAS distros (FreeNAS/TrueNAS, OpenMediaVault) use FreeBSD or Debian with mutable package management. NASty uses NixOS because a storage appliance should be the last thing that breaks during an update.

## Is this production-ready?

No. NASty is experimental and under active development. bcachefs itself is still maturing.

That said, NASty is probably the most thoroughly tested one-person NAS project you'll find:

- **132 end-to-end integration tests** across all 4 protocols (NFS, SMB, iSCSI, NVMe-oF) — snapshots, clones, volume expansion, crash simulation, concurrent operations, StatefulSets, access modes, adoption, and more
- **370 integration tests** across all protocols (NFS, SMB, iSCSI, NVMe-oF) including snapshots, clones, and data integrity
- **76 CSI sanity tests** verifying spec compliance
- **E2E tests per protocol** (NFS, SMB, iSCSI, NVMe-oF) run against a real NASty instance with a real k3s cluster
- CI/CD pipeline builds, lints, tests, and publishes container images automatically
- Tested by an elite team consisting of me, myself, I, my alter ego, my evil twin, my inner child, my future self, my past self, my impostor syndrome, my caffeine-fueled persona, and a rubber duck named QA

Use it for homelabs, development, and learning. Not for storing your only copy of irreplaceable data. Yet.

## What's "vibecoded" mean?

NASty is developed with heavy AI assistance — architecture discussions, code generation, debugging, test analysis, and documentation are all done collaboratively with Claude. The human (Bartosz) makes the decisions; the AI does the heavy lifting.

This isn't a toy or a demo. It's a serious project built faster than a single developer could manage alone. The vibecoding approach lets one person build what would normally require a team — a full NAS appliance with engine, WebUI, CSI driver, NixOS integration, and CI/CD pipeline.

The same approach was used to build [tns-csi](https://github.com/fenio/tns-csi), a TrueNAS CSI driver that has an active userbase running it in production without issues. NASty's CSI driver evolved from that codebase.

## What protocols does NASty support?

- **NFS** — Network File System. Standard Linux/Unix file sharing.
- **SMB** — Server Message Block. Windows/macOS compatible file sharing.
- **iSCSI** — Internet SCSI. Block storage over TCP. Used by Kubernetes for persistent volumes.
- **NVMe-oF** — NVMe over Fabrics. High-performance block storage over TCP. The modern alternative to iSCSI.

All four protocols are managed through the same WebUI and API. The Kubernetes CSI driver supports all four.

## How are snapshots and clones different from ZFS?

Simpler.

In bcachefs, a snapshot IS a subvolume. It's a first-class citizen, not a dependent child of its parent. Delete the parent — the snapshot survives. No "promote", no "detach", no dependency chains.

A clone is just a writable snapshot. One command: `bcachefs subvolume snapshot` (without `-r`). Instant, COW, fully independent. No clone modes, no send/receive for independence.

## Where does the name come from?

NAS + ty. It's the only English word with "NAS" in it. That's it. No backronym, no hidden meaning, no marketing brainstorm. Just a NAS that's a bit nasty.
