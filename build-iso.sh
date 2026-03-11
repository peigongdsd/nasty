#!/usr/bin/env bash
set -euo pipefail

# Build NASty ISO using Docker (works on macOS with Colima)
# Usage: ./build-iso.sh [aarch64|x86_64]
# Default: aarch64 (native on ARM Colima, for validation)

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ARCH="${1:-aarch64}"

if [ "$ARCH" = "x86_64" ]; then
  CONFIG="nasty-iso"
  PLATFORM_FLAG=""
  echo "==> Building x86_64 ISO (requires x86_64 host or CI)"
else
  CONFIG="nasty-iso-aarch64"
  PLATFORM_FLAG=""
  echo "==> Building aarch64 ISO (native on ARM Colima, for config validation)"
fi

echo "    This will take a while on first run (downloading nixpkgs)."
echo ""

docker run --rm \
  -v "${SCRIPT_DIR}:/NAS" \
  -w /NAS/nixos \
  -e NIX_CONFIG="experimental-features = nix-command flakes
sandbox = false
filter-syscalls = false
max-jobs = auto" \
  nixos/nix:latest \
  bash -c '
    set -euo pipefail

    # Nix flakes require files to be git-tracked
    cp -r /NAS /tmp/NAS
    cd /tmp/NAS
    git init -q
    git add -A
    git -c user.email="build@nasty" -c user.name="build" commit -q -m "build"
    cd nixos

    echo "==> Running nix build (config: '"$CONFIG"')..."
    nix build .#nixosConfigurations.'"$CONFIG"'.config.system.build.isoImage \
      --no-link --print-out-paths -L \
      > /tmp/nix-stdout.txt 2> >(tee /tmp/nix-stderr.txt >&2) || {
      echo ""
      echo "==> Build failed."
      echo "==> Last 30 lines of stderr:"
      tail -30 /tmp/nix-stderr.txt
      exit 1
    }
    ISO_PATH=$(cat /tmp/nix-stdout.txt | tail -1)

    echo "==> Build output: $ISO_PATH"
    ISO_FILE=$(find "$ISO_PATH" -name "*.iso" -type f | head -1)
    if [ -n "$ISO_FILE" ]; then
      cp -f "$ISO_FILE" /NAS/nasty.iso
      echo ""
      echo "==> ISO written to: nasty.iso"
      ls -lh /NAS/nasty.iso
    else
      echo "ERROR: No .iso file found in $ISO_PATH"
      ls -la "$ISO_PATH" || true
      exit 1
    fi
  '
