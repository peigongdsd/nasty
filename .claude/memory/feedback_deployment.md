---
name: Deployment process
description: How new versions are deployed to the NASty appliance
type: feedback
---

Deploy via NixOS rebuild from the WebUI, never by copying binaries directly.

**Why:** The appliance runs NixOS — binaries compiled on macOS won't run on Linux, and the Nix store is the source of truth for all binaries.

**How to apply:** After code changes, just commit and push. The user deploys via the WebUI (NixOS rebuild). Never `scp` binaries or restart services manually.
