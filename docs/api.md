# NASty JSON-RPC API

NASty exposes a **JSON-RPC 2.0** API over **WebSocket** at `/ws`.

## Transport

Connect to `ws://<host>/ws` with a valid session cookie or `Authorization: Bearer <token>` header.

**Request:**
```json
{"jsonrpc": "2.0", "id": 1, "method": "pool.list", "params": {}}
```

**Response:**
```json
{"jsonrpc": "2.0", "id": 1, "result": [...]}
```

**Error:**
```json
{"jsonrpc": "2.0", "id": 1, "error": {"code": -32603, "message": "pool not found: mypool"}}
```

## Authentication

Send `POST /api/login` with `{"username": "...", "password": "..."}` to receive a session token. Pass it as a cookie (`session=<token>`) or `Authorization: Bearer <token>` header on the WebSocket upgrade.

## Roles

| Role | Description |
|------|-------------|
| `admin` | Full access to all methods |
| `operator` | Create/delete subvolumes and snapshots; read pools. Cannot manage users, destroy pools, or change system settings. |
| `readonly` | Read-only access to all list/get methods |

API tokens can additionally be scoped to a single **pool** (restricts visibility) and for operator tokens to a single **owner** (restricts to subvolumes owned by that token).

## Real-time Events

After any successful mutation the server broadcasts an event on the same WebSocket:
```json
{"event": "pool"}
```
Clients should re-fetch the relevant resource when they receive an event. Event types: `pool`, `subvolume`, `snapshot`, `share.nfs`, `share.smb`, `share.iscsi`, `share.nvmeof`, `protocol`, `settings`, `alert`.

---

## Contents

- [Authentication](#authentication)
- [System](#system)
- [System Update](#system-update)
- [bcachefs-tools](#bcachefs-tools)
- [Settings](#settings)
- [Network](#network)
- [Protocols & Services](#protocols--services)
- [Alert Rules](#alert-rules)
- [Block Devices](#block-devices)
- [Pools](#pools)
- [Pool Devices](#pool-devices)
- [Subvolumes](#subvolumes)
- [Snapshots](#snapshots)
- [NFS Shares](#nfs-shares)
- [SMB Shares](#smb-shares)
- [iSCSI Targets](#iscsi-targets)
- [NVMe-oF Subsystems](#nvme-of-subsystems)

## Authentication

### `auth.me`

Return the current session's username and role.

**Role:** `any`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `pool` | string | no | If set, token can only see subvolumes in this pool. |
| `role` | `Role` | yes |  |
| `token` | string | yes |  |
| `username` | string | yes |  |


### `auth.logout`

Invalidate the current session token.

**Role:** `any`


### `auth.change_password`

Change a user's password. Admins can change any user; users can change their own.

**Role:** `any`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `new_password` | string | yes |  |
| `username` | string | yes |  |


### `auth.create_user`

Create a new local user account.

**Role:** `admin`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `password` | string | yes |  |
| `role` | `Role` | yes |  |
| `username` | string | yes |  |


### `auth.delete_user`

Delete a user. Cannot delete your own account.

**Role:** `admin`

**Params:** `{"username": string}`


### `auth.list_users`

List all users (no password hashes).

**Role:** `any`

**Returns:**

``UserInfo`[]`


### `auth.token.list`

List all API tokens (without token values).

**Role:** `admin`

**Returns:**

``ApiTokenInfo`[]`


### `auth.token.create`

Create a long-lived API token. Returns the token value — shown only once.

**Role:** `admin`

**Params:** `{"name": string, "role": Role, "pool": string?, "expires_in_secs": integer?}`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `created_at` | integer | yes |  |
| `expires_at` | integer | no | Unix timestamp after which the token is rejected. None = never expires. |
| `id` | string | yes |  |
| `name` | string | yes |  |
| `pool` | string | no |  |
| `role` | `Role` | yes |  |
| `token` | string | yes | The actual token value — shown only once on creation. |


### `auth.token.delete`

Delete an API token by ID.

**Role:** `admin`

**Params:** `{"id": string}`


## System

### `system.info`

Return hostname, OS version, uptime, bcachefs-tools version info.

**Role:** `any`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `bcachefs_commit` | string | no | Short (12-char) commit SHA of the pinned bcachefs-tools in flake.lock |
| `bcachefs_is_custom` | boolean | yes | True when the user has overridden the default bcachefs-tools version |
| `bcachefs_pinned_ref` | string | no | The ref stored in the state file: tag name (e.g. "v1.37.1") or short SHA |
| `bcachefs_version` | string | yes |  |
| `hostname` | string | yes |  |
| `kernel` | string | yes |  |
| `ntp_synced` | boolean | yes |  |
| `timezone` | string | yes |  |
| `uptime_seconds` | integer | yes |  |
| `version` | string | yes |  |


### `system.health`

Return health status of all systemd services.

**Role:** `any`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `services` | `ServiceStatus`[] | yes |  |
| `status` | string | yes |  |


### `system.stats`

Return current CPU, memory, network interface, and disk I/O statistics.

**Role:** `any`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `cpu` | `CpuStats` | yes |  |
| `disk_io` | `DiskIoStats`[] | yes |  |
| `memory` | `MemoryStats` | yes |  |
| `network` | `NetIfStats`[] | yes |  |


### `system.disks`

Return S.M.A.R.T. health data for all drives. Requires SMART protocol to be enabled.

**Role:** `any`

**Returns:**

``DiskHealth`[]`


### `system.alerts`

Evaluate alert rules against current system state and return any active alerts.

**Role:** `any`

**Returns:**

``ActiveAlert`[]`


### `system.reboot`

Reboot the system.

**Role:** `admin`


### `system.shutdown`

Shut down the system.

**Role:** `admin`


## System Update

### `system.update.version`

Return current version and latest available version.

**Role:** `any`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `current_version` | string | yes |  |
| `latest_version` | string | no |  |
| `update_available` | boolean | no |  |


### `system.update.check`

Check for available updates against the upstream repository.

**Role:** `any`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `current_version` | string | yes |  |
| `latest_version` | string | no |  |
| `update_available` | boolean | no |  |


### `system.update.apply`

Fetch and apply the latest NixOS generation. Runs `nixos-rebuild switch` in the background.

**Role:** `admin`


### `system.update.rollback`

Roll back to the previous NixOS generation.

**Role:** `admin`


### `system.update.status`

Return the current update operation status and log.

**Role:** `any`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `log` | string | yes |  |
| `reboot_required` | boolean | yes | True when the activated system has a different kernel than the booted one |
| `state` | string | yes | "idle", "running", "success", "failed" |
| `webui_changed` | boolean | yes | True when the webui store path changed during this update (browser reload needed) |


## bcachefs-tools

### `bcachefs.tools.info`

Return bcachefs-tools version info (pinned ref, running version, custom/default flag).

**Role:** `any`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `default_ref` | string | yes | The default ref from flake.nix (e.g. "v1.37.0") |
| `is_custom` | boolean | yes | True when the user has overridden the default bcachefs-tools version |
| `kernel_rust` | boolean | no | Whether the running kernel was built with Rust support (CONFIG_RUST=y) |
| `pinned_ref` | string | no | The ref in flake.lock original (e.g. "v1.37.0", "master", commit sha) |
| `pinned_rev` | string | no | The resolved full commit sha from flake.lock locked |
| `running_version` | string | yes | Output of `bcachefs version` |


### `bcachefs.tools.switch`

Switch bcachefs-tools to a specific git ref (tag, branch, or commit SHA). Runs `nixos-rebuild switch` in background.

**Role:** `admin`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `git_ref` | string | yes | A git ref: tag (v1.37.0), branch (master), or commit hash |


### `bcachefs.tools.status`

Return the current bcachefs-tools switch operation status.

**Role:** `any`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `log` | string | yes |  |
| `reboot_required` | boolean | yes | True when the activated system has a different kernel than the booted one |
| `state` | string | yes | "idle", "running", "success", "failed" |
| `webui_changed` | boolean | yes | True when the webui store path changed during this update (browser reload needed) |


## Settings

### `system.settings.get`

Return current system settings.

**Role:** `any`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `clock_24h` | boolean | no |  |
| `hostname` | string | no |  |
| `timezone` | string | no |  |


### `system.settings.update`

Update system settings. Only provided fields are changed.

**Role:** `admin`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `clock_24h` | boolean | no |  |
| `hostname` | string | no |  |
| `timezone` | string | no |  |

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `clock_24h` | boolean | no |  |
| `hostname` | string | no |  |
| `timezone` | string | no |  |


### `system.settings.timezones`

Return list of valid IANA timezone strings.

**Role:** `any`

**Returns:**

`string[]`


## Network

### `system.network.get`

Return current network configuration including live interface state.

**Role:** `any`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `address` | string | no |  |
| `dhcp` | boolean | yes |  |
| `gateway` | string | no |  |
| `interface` | string | no |  |
| `live_addresses` | string[] | no |  |
| `live_gateway` | string | no |  |
| `nameservers` | string[] | no |  |
| `prefix_length` | integer | no |  |


### `system.network.update`

Update network configuration (DHCP or static). Applied immediately without rebooting.

**Role:** `admin`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `address` | string | no |  |
| `dhcp` | boolean | yes |  |
| `gateway` | string | no |  |
| `interface` | string | no |  |
| `live_addresses` | string[] | no |  |
| `live_gateway` | string | no |  |
| `nameservers` | string[] | no |  |
| `prefix_length` | integer | no |  |


## Protocols & Services

### `service.protocol.list`

List all protocols and their current status.

**Role:** `any`

**Returns:**

``ProtocolStatus`[]`


### `service.protocol.enable`

Enable a protocol service. Available names: `nfs`, `smb`, `iscsi`, `nvmeof`, `ssh`, `avahi`, `smart`.

**Role:** `admin`

**Params:** `{"name": string}`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `display_name` | string | yes |  |
| `enabled` | boolean | yes |  |
| `name` | string | yes |  |
| `running` | boolean | yes |  |
| `system_service` | boolean | yes |  |


### `service.protocol.disable`

Disable a protocol service.

**Role:** `admin`

**Params:** `{"name": string}`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `display_name` | string | yes |  |
| `enabled` | boolean | yes |  |
| `name` | string | yes |  |
| `running` | boolean | yes |  |
| `system_service` | boolean | yes |  |


## Alert Rules

### `alert.rules.list`

List all alert rules.

**Role:** `any`

**Returns:**

``AlertRule`[]`


### `alert.rules.create`

Create a new alert rule.

**Role:** `admin`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `condition` | `AlertCondition` | yes |  |
| `enabled` | boolean | yes |  |
| `id` | string | yes |  |
| `metric` | `AlertMetric` | yes |  |
| `name` | string | yes |  |
| `severity` | `AlertSeverity` | yes |  |
| `threshold` | number | yes |  |

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `condition` | `AlertCondition` | yes |  |
| `enabled` | boolean | yes |  |
| `id` | string | yes |  |
| `metric` | `AlertMetric` | yes |  |
| `name` | string | yes |  |
| `severity` | `AlertSeverity` | yes |  |
| `threshold` | number | yes |  |


### `alert.rules.update`

Update an existing alert rule. Only provided fields are changed.

**Role:** `admin`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `enabled` | boolean | no |  |
| `id` | string | yes |  |
| `name` | string | no |  |
| `severity` | `AlertSeverity` \| null | no |  |
| `threshold` | number | no |  |

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `condition` | `AlertCondition` | yes |  |
| `enabled` | boolean | yes |  |
| `id` | string | yes |  |
| `metric` | `AlertMetric` | yes |  |
| `name` | string | yes |  |
| `severity` | `AlertSeverity` | yes |  |
| `threshold` | number | yes |  |


### `alert.rules.delete`

Delete an alert rule by ID.

**Role:** `admin`

**Params:** `{"id": string}`


## Block Devices

### `device.list`

List all block devices and partitions visible to the system.

**Role:** `any`

**Returns:**

``BlockDevice`[]`


### `device.wipe`

Erase all filesystem signatures from a device (wipefs). The device must not be in use.

**Role:** `admin`

**Params:** `{"path": string}`


## Pools

### `pool.list`

List all storage pools. Pool-scoped tokens see only their assigned pool.

**Role:** `any`

**Returns:**

``Pool`[]`


### `pool.get`

Get a single pool by name.

**Role:** `any`

**Params:** `{"name": string}`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `available_bytes` | integer | yes |  |
| `devices` | `PoolDevice`[] | yes |  |
| `mount_point` | string | no |  |
| `mounted` | boolean | yes |  |
| `name` | string | yes |  |
| `options` | `PoolOptions` | yes | Filesystem-level options read from sysfs or show-super. |
| `total_bytes` | integer | yes |  |
| `used_bytes` | integer | yes |  |
| `uuid` | string | yes |  |


### `pool.create`

Format and mount a new bcachefs pool.

**Role:** `admin`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `background_target` | string | no |  |
| `compression` | string | no |  |
| `devices` | `DeviceSpec`[] | yes |  |
| `encryption` | boolean | no |  |
| `erasure_code` | boolean | no |  |
| `foreground_target` | string | no | Tiering targets set at format time. |
| `label` | string | no | Filesystem-wide label (used as default when no per-device labels set). |
| `metadata_target` | string | no |  |
| `name` | string | yes |  |
| `promote_target` | string | no |  |
| `replicas` | integer | no |  |

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `available_bytes` | integer | yes |  |
| `devices` | `PoolDevice`[] | yes |  |
| `mount_point` | string | no |  |
| `mounted` | boolean | yes |  |
| `name` | string | yes |  |
| `options` | `PoolOptions` | yes | Filesystem-level options read from sysfs or show-super. |
| `total_bytes` | integer | yes |  |
| `used_bytes` | integer | yes |  |
| `uuid` | string | yes |  |


### `pool.destroy`

Unmount and unregister a pool. Does not wipe the devices.

**Role:** `admin`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `force` | boolean | no |  |
| `name` | string | yes |  |


### `pool.mount`

Mount a known pool.

**Role:** `admin`

**Params:** `{"name": string}`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `available_bytes` | integer | yes |  |
| `devices` | `PoolDevice`[] | yes |  |
| `mount_point` | string | no |  |
| `mounted` | boolean | yes |  |
| `name` | string | yes |  |
| `options` | `PoolOptions` | yes | Filesystem-level options read from sysfs or show-super. |
| `total_bytes` | integer | yes |  |
| `used_bytes` | integer | yes |  |
| `uuid` | string | yes |  |


### `pool.unmount`

Unmount a pool.

**Role:** `admin`

**Params:** `{"name": string}`


### `pool.options.update`

Update runtime-mutable bcachefs filesystem options (written to sysfs).

**Role:** `admin`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `background_compression` | string | no |  |
| `background_target` | string | no |  |
| `compression` | string | no |  |
| `erasure_code` | boolean | no |  |
| `error_action` | string | no |  |
| `foreground_target` | string | no |  |
| `metadata_target` | string | no |  |
| `name` | string | yes |  |
| `promote_target` | string | no |  |

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `available_bytes` | integer | yes |  |
| `devices` | `PoolDevice`[] | yes |  |
| `mount_point` | string | no |  |
| `mounted` | boolean | yes |  |
| `name` | string | yes |  |
| `options` | `PoolOptions` | yes | Filesystem-level options read from sysfs or show-super. |
| `total_bytes` | integer | yes |  |
| `used_bytes` | integer | yes |  |
| `uuid` | string | yes |  |


### `pool.usage`

Return detailed bcachefs `fs usage` breakdown.

**Role:** `any`

**Params:** `{"name": string}`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `data_bytes` | integer | yes | Total data stored (before replication). |
| `devices` | `DeviceUsage`[] | yes | Per-device usage breakdown. |
| `metadata_bytes` | integer | yes | Total metadata stored. |
| `raw` | string | yes | Raw output from `bcachefs fs usage`, structured where possible. |
| `reserved_bytes` | integer | yes | Reserved bytes. |


### `pool.scrub.start`

Start a scrub on a mounted pool.

**Role:** `admin`

**Params:** `{"name": string}`


### `pool.scrub.status`

Return current scrub status.

**Role:** `any`

**Params:** `{"name": string}`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `raw` | string | yes |  |
| `running` | boolean | yes |  |


### `pool.reconcile.status`

Return bcachefs background work (reconcile) status.

**Role:** `any`

**Params:** `{"name": string}`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `raw` | string | yes |  |


### `bcachefs.usage`

Return raw `bcachefs fs usage` output for a pool.

**Role:** `any`

**Params:** `{"name": string}`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `data_bytes` | integer | yes | Total data stored (before replication). |
| `devices` | `DeviceUsage`[] | yes | Per-device usage breakdown. |
| `metadata_bytes` | integer | yes | Total metadata stored. |
| `raw` | string | yes | Raw output from `bcachefs fs usage`, structured where possible. |
| `reserved_bytes` | integer | yes | Reserved bytes. |


## Pool Devices

### `pool.device.add`

Add a device to an existing mounted pool.

**Role:** `admin`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `device` | `DeviceSpec` | yes |  |
| `pool` | string | yes |  |

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `available_bytes` | integer | yes |  |
| `devices` | `PoolDevice`[] | yes |  |
| `mount_point` | string | no |  |
| `mounted` | boolean | yes |  |
| `name` | string | yes |  |
| `options` | `PoolOptions` | yes | Filesystem-level options read from sysfs or show-super. |
| `total_bytes` | integer | yes |  |
| `used_bytes` | integer | yes |  |
| `uuid` | string | yes |  |


### `pool.device.remove`

Remove a device from a pool. The device should be fully evacuated first.

**Role:** `admin`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `device` | string | yes |  |
| `pool` | string | yes |  |

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `available_bytes` | integer | yes |  |
| `devices` | `PoolDevice`[] | yes |  |
| `mount_point` | string | no |  |
| `mounted` | boolean | yes |  |
| `name` | string | yes |  |
| `options` | `PoolOptions` | yes | Filesystem-level options read from sysfs or show-super. |
| `total_bytes` | integer | yes |  |
| `used_bytes` | integer | yes |  |
| `uuid` | string | yes |  |


### `pool.device.evacuate`

Evacuate all data from a device to the remaining pool members. Long-running — returns `{"status": "started"}` immediately. Pool events are broadcast every 3s during evacuation.

**Role:** `admin`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `device` | string | yes |  |
| `pool` | string | yes |  |


### `pool.device.set_state`

Set persistent device state (`rw`, `ro`, `failed`, `spare`).

**Role:** `admin`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `device` | string | yes |  |
| `pool` | string | yes |  |
| `state` | string | yes | One of: rw, ro, failed, spare |

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `available_bytes` | integer | yes |  |
| `devices` | `PoolDevice`[] | yes |  |
| `mount_point` | string | no |  |
| `mounted` | boolean | yes |  |
| `name` | string | yes |  |
| `options` | `PoolOptions` | yes | Filesystem-level options read from sysfs or show-super. |
| `total_bytes` | integer | yes |  |
| `used_bytes` | integer | yes |  |
| `uuid` | string | yes |  |


### `pool.device.set_label`

Set or update the hierarchical label on a device in a mounted pool. Written live via sysfs.

**Role:** `admin`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `device` | string | yes |  |
| `label` | string | yes |  |
| `pool` | string | yes |  |

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `available_bytes` | integer | yes |  |
| `devices` | `PoolDevice`[] | yes |  |
| `mount_point` | string | no |  |
| `mounted` | boolean | yes |  |
| `name` | string | yes |  |
| `options` | `PoolOptions` | yes | Filesystem-level options read from sysfs or show-super. |
| `total_bytes` | integer | yes |  |
| `used_bytes` | integer | yes |  |
| `uuid` | string | yes |  |


### `pool.device.online`

Bring a device back online.

**Role:** `admin`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `device` | string | yes |  |
| `pool` | string | yes |  |

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `available_bytes` | integer | yes |  |
| `devices` | `PoolDevice`[] | yes |  |
| `mount_point` | string | no |  |
| `mounted` | boolean | yes |  |
| `name` | string | yes |  |
| `options` | `PoolOptions` | yes | Filesystem-level options read from sysfs or show-super. |
| `total_bytes` | integer | yes |  |
| `used_bytes` | integer | yes |  |
| `uuid` | string | yes |  |


### `pool.device.offline`

Take a device offline.

**Role:** `admin`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `device` | string | yes |  |
| `pool` | string | yes |  |

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `available_bytes` | integer | yes |  |
| `devices` | `PoolDevice`[] | yes |  |
| `mount_point` | string | no |  |
| `mounted` | boolean | yes |  |
| `name` | string | yes |  |
| `options` | `PoolOptions` | yes | Filesystem-level options read from sysfs or show-super. |
| `total_bytes` | integer | yes |  |
| `used_bytes` | integer | yes |  |
| `uuid` | string | yes |  |


## Subvolumes

### `subvolume.list`

List subvolumes in a pool.

**Role:** `any`

**Params:** `{"pool": string}`

**Returns:**

``Subvolume`[]`


### `subvolume.list_all`

List all subvolumes across all pools.

**Role:** `any`

**Returns:**

``Subvolume`[]`


### `subvolume.get`

Get a single subvolume.

**Role:** `any`

**Params:** `{"pool": string, "name": string}`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `block_device` | string | no |  |
| `comments` | string | no |  |
| `compression` | string | no |  |
| `name` | string | yes |  |
| `owner` | string | no | Token name that created this subvolume; None for subvolumes created by human users. |
| `path` | string | yes |  |
| `pool` | string | yes |  |
| `properties` | object | no | Arbitrary key-value metadata stored as POSIX xattrs (user.* namespace).
Used by nasty-csi to track CSI volume metadata without sidecar files. |
| `snapshots` | string[] | yes |  |
| `subvolume_type` | `SubvolumeType` | yes |  |
| `used_bytes` | integer | no |  |
| `volsize_bytes` | integer | no |  |


### `subvolume.create`

Create a new bcachefs subvolume (filesystem or block-backed).

**Role:** `operator`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `comments` | string | no |  |
| `compression` | string | no |  |
| `name` | string | yes |  |
| `pool` | string | yes |  |
| `subvolume_type` | `SubvolumeType` | no |  |
| `volsize_bytes` | integer | no |  |

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `block_device` | string | no |  |
| `comments` | string | no |  |
| `compression` | string | no |  |
| `name` | string | yes |  |
| `owner` | string | no | Token name that created this subvolume; None for subvolumes created by human users. |
| `path` | string | yes |  |
| `pool` | string | yes |  |
| `properties` | object | no | Arbitrary key-value metadata stored as POSIX xattrs (user.* namespace).
Used by nasty-csi to track CSI volume metadata without sidecar files. |
| `snapshots` | string[] | yes |  |
| `subvolume_type` | `SubvolumeType` | yes |  |
| `used_bytes` | integer | no |  |
| `volsize_bytes` | integer | no |  |


### `subvolume.delete`

Delete a subvolume and all its snapshots.

**Role:** `operator`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `name` | string | yes |  |
| `pool` | string | yes |  |


### `subvolume.attach`

Attach the loop device for a block subvolume (mounts `vol.img` via losetup).

**Role:** `operator`

**Params:** `{"pool": string, "name": string}`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `block_device` | string | no |  |
| `comments` | string | no |  |
| `compression` | string | no |  |
| `name` | string | yes |  |
| `owner` | string | no | Token name that created this subvolume; None for subvolumes created by human users. |
| `path` | string | yes |  |
| `pool` | string | yes |  |
| `properties` | object | no | Arbitrary key-value metadata stored as POSIX xattrs (user.* namespace).
Used by nasty-csi to track CSI volume metadata without sidecar files. |
| `snapshots` | string[] | yes |  |
| `subvolume_type` | `SubvolumeType` | yes |  |
| `used_bytes` | integer | no |  |
| `volsize_bytes` | integer | no |  |


### `subvolume.detach`

Detach the loop device for a block subvolume.

**Role:** `operator`

**Params:** `{"pool": string, "name": string}`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `block_device` | string | no |  |
| `comments` | string | no |  |
| `compression` | string | no |  |
| `name` | string | yes |  |
| `owner` | string | no | Token name that created this subvolume; None for subvolumes created by human users. |
| `path` | string | yes |  |
| `pool` | string | yes |  |
| `properties` | object | no | Arbitrary key-value metadata stored as POSIX xattrs (user.* namespace).
Used by nasty-csi to track CSI volume metadata without sidecar files. |
| `snapshots` | string[] | yes |  |
| `subvolume_type` | `SubvolumeType` | yes |  |
| `used_bytes` | integer | no |  |
| `volsize_bytes` | integer | no |  |


### `subvolume.resize`

Resize a block subvolume's backing image.

**Role:** `operator`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `name` | string | yes |  |
| `pool` | string | yes |  |
| `volsize_bytes` | integer | yes |  |

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `block_device` | string | no |  |
| `comments` | string | no |  |
| `compression` | string | no |  |
| `name` | string | yes |  |
| `owner` | string | no | Token name that created this subvolume; None for subvolumes created by human users. |
| `path` | string | yes |  |
| `pool` | string | yes |  |
| `properties` | object | no | Arbitrary key-value metadata stored as POSIX xattrs (user.* namespace).
Used by nasty-csi to track CSI volume metadata without sidecar files. |
| `snapshots` | string[] | yes |  |
| `subvolume_type` | `SubvolumeType` | yes |  |
| `used_bytes` | integer | no |  |
| `volsize_bytes` | integer | no |  |


### `subvolume.set_properties`

Set arbitrary key-value metadata on a subvolume (stored as POSIX xattrs in the `user.*` namespace). Used by the CSI driver.

**Role:** `operator`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `name` | string | yes |  |
| `pool` | string | yes |  |
| `properties` | object | yes | Key-value pairs to set (merged with existing properties). |

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `block_device` | string | no |  |
| `comments` | string | no |  |
| `compression` | string | no |  |
| `name` | string | yes |  |
| `owner` | string | no | Token name that created this subvolume; None for subvolumes created by human users. |
| `path` | string | yes |  |
| `pool` | string | yes |  |
| `properties` | object | no | Arbitrary key-value metadata stored as POSIX xattrs (user.* namespace).
Used by nasty-csi to track CSI volume metadata without sidecar files. |
| `snapshots` | string[] | yes |  |
| `subvolume_type` | `SubvolumeType` | yes |  |
| `used_bytes` | integer | no |  |
| `volsize_bytes` | integer | no |  |


### `subvolume.remove_properties`

Remove specific metadata keys from a subvolume.

**Role:** `operator`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `keys` | string[] | yes | Property keys to remove. |
| `name` | string | yes |  |
| `pool` | string | yes |  |

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `block_device` | string | no |  |
| `comments` | string | no |  |
| `compression` | string | no |  |
| `name` | string | yes |  |
| `owner` | string | no | Token name that created this subvolume; None for subvolumes created by human users. |
| `path` | string | yes |  |
| `pool` | string | yes |  |
| `properties` | object | no | Arbitrary key-value metadata stored as POSIX xattrs (user.* namespace).
Used by nasty-csi to track CSI volume metadata without sidecar files. |
| `snapshots` | string[] | yes |  |
| `subvolume_type` | `SubvolumeType` | yes |  |
| `used_bytes` | integer | no |  |
| `volsize_bytes` | integer | no |  |


### `subvolume.find_by_property`

Find subvolumes matching a specific metadata key-value pair.

**Role:** `any`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `key` | string | yes |  |
| `pool` | string | no | Optional pool to restrict the search to. |
| `value` | string | yes |  |

**Returns:**

``Subvolume`[]`


## Snapshots

### `snapshot.list`

List snapshots for all subvolumes in a pool.

**Role:** `any`

**Params:** `{"pool": string}`

**Returns:**

``Snapshot`[]`


### `snapshot.create`

Create a snapshot of a subvolume.

**Role:** `operator`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `name` | string | yes |  |
| `pool` | string | yes |  |
| `read_only` | boolean | no |  |
| `subvolume` | string | yes |  |

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `block_device` | string | no | Loop device path if this snapshot's vol.img is currently attached (block snapshots only). |
| `name` | string | yes |  |
| `path` | string | yes |  |
| `pool` | string | yes |  |
| `read_only` | boolean | yes |  |
| `subvolume` | string | yes |  |


### `snapshot.delete`

Delete a snapshot.

**Role:** `operator`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `name` | string | yes |  |
| `pool` | string | yes |  |
| `subvolume` | string | yes |  |


### `snapshot.clone`

Clone a snapshot into a new independent subvolume.

**Role:** `operator`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `new_name` | string | yes |  |
| `pool` | string | yes |  |
| `snapshot` | string | yes |  |
| `subvolume` | string | yes |  |

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `block_device` | string | no |  |
| `comments` | string | no |  |
| `compression` | string | no |  |
| `name` | string | yes |  |
| `owner` | string | no | Token name that created this subvolume; None for subvolumes created by human users. |
| `path` | string | yes |  |
| `pool` | string | yes |  |
| `properties` | object | no | Arbitrary key-value metadata stored as POSIX xattrs (user.* namespace).
Used by nasty-csi to track CSI volume metadata without sidecar files. |
| `snapshots` | string[] | yes |  |
| `subvolume_type` | `SubvolumeType` | yes |  |
| `used_bytes` | integer | no |  |
| `volsize_bytes` | integer | no |  |


## NFS Shares

### `share.nfs.list`

List all NFS shares.

**Role:** `any`

**Returns:**

``NfsShare`[]`


### `share.nfs.get`

Get an NFS share by ID.

**Role:** `any`

**Params:** `{"id": string}`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `clients` | `NfsClient`[] | yes |  |
| `comment` | string | no |  |
| `enabled` | boolean | yes |  |
| `id` | string | yes |  |
| `path` | string | yes |  |


### `share.nfs.create`

Create an NFS share.

**Role:** `admin`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `clients` | `NfsClient`[] | yes |  |
| `comment` | string | no |  |
| `enabled` | boolean | no |  |
| `path` | string | yes |  |

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `clients` | `NfsClient`[] | yes |  |
| `comment` | string | no |  |
| `enabled` | boolean | yes |  |
| `id` | string | yes |  |
| `path` | string | yes |  |


### `share.nfs.update`

Update an NFS share.

**Role:** `admin`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `clients` | `NfsClient`[] | no |  |
| `comment` | string | no |  |
| `enabled` | boolean | no |  |
| `id` | string | yes |  |

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `clients` | `NfsClient`[] | yes |  |
| `comment` | string | no |  |
| `enabled` | boolean | yes |  |
| `id` | string | yes |  |
| `path` | string | yes |  |


### `share.nfs.delete`

Delete an NFS share.

**Role:** `admin`

**Params:** `{"id": string}`


## SMB Shares

### `share.smb.list`

List all SMB shares.

**Role:** `any`

**Returns:**

``SmbShare`[]`


### `share.smb.get`

Get an SMB share by ID.

**Role:** `any`

**Params:** `{"id": string}`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `browseable` | boolean | yes |  |
| `comment` | string | no |  |
| `enabled` | boolean | yes |  |
| `extra_params` | object | yes |  |
| `guest_ok` | boolean | yes |  |
| `id` | string | yes |  |
| `name` | string | yes |  |
| `path` | string | yes |  |
| `read_only` | boolean | yes |  |
| `valid_users` | string[] | yes |  |


### `share.smb.create`

Create an SMB share.

**Role:** `admin`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `browseable` | boolean | no |  |
| `comment` | string | no |  |
| `enabled` | boolean | no |  |
| `extra_params` | object | no |  |
| `guest_ok` | boolean | no |  |
| `name` | string | yes |  |
| `path` | string | yes |  |
| `read_only` | boolean | no |  |
| `valid_users` | string[] | no |  |

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `browseable` | boolean | yes |  |
| `comment` | string | no |  |
| `enabled` | boolean | yes |  |
| `extra_params` | object | yes |  |
| `guest_ok` | boolean | yes |  |
| `id` | string | yes |  |
| `name` | string | yes |  |
| `path` | string | yes |  |
| `read_only` | boolean | yes |  |
| `valid_users` | string[] | yes |  |


### `share.smb.update`

Update an SMB share.

**Role:** `admin`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `browseable` | boolean | no |  |
| `comment` | string | no |  |
| `enabled` | boolean | no |  |
| `extra_params` | object | no |  |
| `guest_ok` | boolean | no |  |
| `id` | string | yes |  |
| `name` | string | no |  |
| `read_only` | boolean | no |  |
| `valid_users` | string[] | no |  |

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `browseable` | boolean | yes |  |
| `comment` | string | no |  |
| `enabled` | boolean | yes |  |
| `extra_params` | object | yes |  |
| `guest_ok` | boolean | yes |  |
| `id` | string | yes |  |
| `name` | string | yes |  |
| `path` | string | yes |  |
| `read_only` | boolean | yes |  |
| `valid_users` | string[] | yes |  |


### `share.smb.delete`

Delete an SMB share.

**Role:** `admin`

**Params:** `{"id": string}`


## iSCSI Targets

### `share.iscsi.list`

List all iSCSI targets.

**Role:** `any`

**Returns:**

``IscsiTarget`[]`


### `share.iscsi.get`

Get an iSCSI target by ID.

**Role:** `any`

**Params:** `{"id": string}`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `acls` | `Acl`[] | yes |  |
| `alias` | string | no |  |
| `enabled` | boolean | yes |  |
| `id` | string | yes |  |
| `iqn` | string | yes |  |
| `luns` | `Lun`[] | yes |  |
| `portals` | `Portal`[] | yes |  |


### `share.iscsi.create_quick`

Create an iSCSI target + LUN in one call.

**Role:** `admin`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `device_path` | string | yes | Block device path (e.g. /dev/loop0) |
| `name` | string | yes | Short name for the IQN |

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `acls` | `Acl`[] | yes |  |
| `alias` | string | no |  |
| `enabled` | boolean | yes |  |
| `id` | string | yes |  |
| `iqn` | string | yes |  |
| `luns` | `Lun`[] | yes |  |
| `portals` | `Portal`[] | yes |  |


### `share.iscsi.create`

Create an iSCSI target (no LUNs). Add LUNs separately.

**Role:** `admin`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `alias` | string | no |  |
| `name` | string | yes | Short name used to generate the IQN: iqn.2137-01.com.nasty:<name> |
| `portals` | `Portal`[] | no | Defaults to 0.0.0.0:3260 |

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `acls` | `Acl`[] | yes |  |
| `alias` | string | no |  |
| `enabled` | boolean | yes |  |
| `id` | string | yes |  |
| `iqn` | string | yes |  |
| `luns` | `Lun`[] | yes |  |
| `portals` | `Portal`[] | yes |  |


### `share.iscsi.delete`

Delete an iSCSI target.

**Role:** `admin`

**Params:** `{"id": string}`


### `share.iscsi.add_lun`

Add a LUN to an iSCSI target.

**Role:** `admin`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `backstore_path` | string | yes | Block device path (/dev/sdb) or file path (/mnt/nasty/pool/disk.img) |
| `backstore_type` | string | no | "block" or "fileio" — auto-detected if omitted |
| `size_bytes` | integer | no | Required for fileio if file doesn't exist yet |
| `target_id` | string | yes |  |

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `acls` | `Acl`[] | yes |  |
| `alias` | string | no |  |
| `enabled` | boolean | yes |  |
| `id` | string | yes |  |
| `iqn` | string | yes |  |
| `luns` | `Lun`[] | yes |  |
| `portals` | `Portal`[] | yes |  |


### `share.iscsi.remove_lun`

Remove a LUN from an iSCSI target.

**Role:** `admin`

**Params:** `{"target_id": string, "lun_id": integer}`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `acls` | `Acl`[] | yes |  |
| `alias` | string | no |  |
| `enabled` | boolean | yes |  |
| `id` | string | yes |  |
| `iqn` | string | yes |  |
| `luns` | `Lun`[] | yes |  |
| `portals` | `Portal`[] | yes |  |


### `share.iscsi.add_acl`

Allow an iSCSI initiator IQN to connect.

**Role:** `admin`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `initiator_iqn` | string | yes |  |
| `password` | string | no |  |
| `target_id` | string | yes |  |
| `userid` | string | no |  |

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `acls` | `Acl`[] | yes |  |
| `alias` | string | no |  |
| `enabled` | boolean | yes |  |
| `id` | string | yes |  |
| `iqn` | string | yes |  |
| `luns` | `Lun`[] | yes |  |
| `portals` | `Portal`[] | yes |  |


### `share.iscsi.remove_acl`

Remove an iSCSI initiator ACL.

**Role:** `admin`

**Params:** `{"target_id": string, "initiator_iqn": string}`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `acls` | `Acl`[] | yes |  |
| `alias` | string | no |  |
| `enabled` | boolean | yes |  |
| `id` | string | yes |  |
| `iqn` | string | yes |  |
| `luns` | `Lun`[] | yes |  |
| `portals` | `Portal`[] | yes |  |


## NVMe-oF Subsystems

### `share.nvmeof.list`

List all NVMe-oF subsystems.

**Role:** `any`

**Returns:**

``NvmeofSubsystem`[]`


### `share.nvmeof.get`

Get an NVMe-oF subsystem by ID.

**Role:** `any`

**Params:** `{"id": string}`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `allow_any_host` | boolean | yes |  |
| `allowed_hosts` | string[] | yes |  |
| `enabled` | boolean | yes |  |
| `id` | string | yes |  |
| `namespaces` | `Namespace`[] | yes |  |
| `nqn` | string | yes |  |
| `ports` | `Port`[] | yes |  |


### `share.nvmeof.create_quick`

Create an NVMe-oF subsystem + namespace + port in one call.

**Role:** `admin`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `addr` | string | no | Listen address (default 0.0.0.0) |
| `device_path` | string | yes | Block device path (e.g. /dev/loop0) |
| `name` | string | yes | Short name for the NQN |
| `port` | integer | no | Port number (default 4420) |

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `allow_any_host` | boolean | yes |  |
| `allowed_hosts` | string[] | yes |  |
| `enabled` | boolean | yes |  |
| `id` | string | yes |  |
| `namespaces` | `Namespace`[] | yes |  |
| `nqn` | string | yes |  |
| `ports` | `Port`[] | yes |  |


### `share.nvmeof.create`

Create an NVMe-oF subsystem (empty).

**Role:** `admin`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `allow_any_host` | boolean | no |  |
| `name` | string | yes | Short name appended to NQN prefix |

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `allow_any_host` | boolean | yes |  |
| `allowed_hosts` | string[] | yes |  |
| `enabled` | boolean | yes |  |
| `id` | string | yes |  |
| `namespaces` | `Namespace`[] | yes |  |
| `nqn` | string | yes |  |
| `ports` | `Port`[] | yes |  |


### `share.nvmeof.delete`

Delete an NVMe-oF subsystem.

**Role:** `admin`

**Params:** `{"id": string}`


### `share.nvmeof.add_namespace`

Add a namespace (block device) to a subsystem.

**Role:** `admin`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `device_path` | string | yes | Block device path (e.g. /dev/sdc) |
| `subsystem_id` | string | yes |  |

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `allow_any_host` | boolean | yes |  |
| `allowed_hosts` | string[] | yes |  |
| `enabled` | boolean | yes |  |
| `id` | string | yes |  |
| `namespaces` | `Namespace`[] | yes |  |
| `nqn` | string | yes |  |
| `ports` | `Port`[] | yes |  |


### `share.nvmeof.remove_namespace`

Remove a namespace from a subsystem.

**Role:** `admin`

**Params:** `{"subsystem_id": string, "nsid": integer}`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `allow_any_host` | boolean | yes |  |
| `allowed_hosts` | string[] | yes |  |
| `enabled` | boolean | yes |  |
| `id` | string | yes |  |
| `namespaces` | `Namespace`[] | yes |  |
| `nqn` | string | yes |  |
| `ports` | `Port`[] | yes |  |


### `share.nvmeof.add_port`

Add a transport port to a subsystem.

**Role:** `admin`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `addr` | string | no |  |
| `addr_family` | string | no |  |
| `service_id` | integer | no | Port number (default 4420) |
| `subsystem_id` | string | yes |  |
| `transport` | string | no | "tcp" or "rdma" |

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `allow_any_host` | boolean | yes |  |
| `allowed_hosts` | string[] | yes |  |
| `enabled` | boolean | yes |  |
| `id` | string | yes |  |
| `namespaces` | `Namespace`[] | yes |  |
| `nqn` | string | yes |  |
| `ports` | `Port`[] | yes |  |


### `share.nvmeof.remove_port`

Remove a transport port from a subsystem.

**Role:** `admin`

**Params:** `{"subsystem_id": string, "port_id": integer}`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `allow_any_host` | boolean | yes |  |
| `allowed_hosts` | string[] | yes |  |
| `enabled` | boolean | yes |  |
| `id` | string | yes |  |
| `namespaces` | `Namespace`[] | yes |  |
| `nqn` | string | yes |  |
| `ports` | `Port`[] | yes |  |


### `share.nvmeof.add_host`

Allow a host NQN to connect to a subsystem.

**Role:** `admin`

**Params:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `host_nqn` | string | yes |  |
| `subsystem_id` | string | yes |  |

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `allow_any_host` | boolean | yes |  |
| `allowed_hosts` | string[] | yes |  |
| `enabled` | boolean | yes |  |
| `id` | string | yes |  |
| `namespaces` | `Namespace`[] | yes |  |
| `nqn` | string | yes |  |
| `ports` | `Port`[] | yes |  |


### `share.nvmeof.remove_host`

Disallow a host NQN from a subsystem.

**Role:** `admin`

**Params:** `{"subsystem_id": string, "nqn": string}`

**Returns:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `allow_any_host` | boolean | yes |  |
| `allowed_hosts` | string[] | yes |  |
| `enabled` | boolean | yes |  |
| `id` | string | yes |  |
| `namespaces` | `Namespace`[] | yes |  |
| `nqn` | string | yes |  |
| `ports` | `Port`[] | yes |  |


---

## Object Definitions

### `Acl`

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `initiator_iqn` | string | yes | Initiator IQN allowed to connect |
| `password` | string | no |  |
| `userid` | string | no |  |

### `ActiveAlert`

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `current_value` | number | yes |  |
| `message` | string | yes |  |
| `metric` | `AlertMetric` | yes |  |
| `rule_id` | string | yes |  |
| `rule_name` | string | yes |  |
| `severity` | `AlertSeverity` | yes |  |
| `source` | string | yes |  |
| `threshold` | number | yes |  |

### `AlertCondition`

Enum: `above`, `below`, `equals`

### `AlertMetric`

Enum: `pool_usage_percent`, `cpu_load_percent`, `memory_usage_percent`, `disk_temperature`, `smart_health`, `swap_usage_percent`

### `AlertRule`

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `condition` | `AlertCondition` | yes |  |
| `enabled` | boolean | yes |  |
| `id` | string | yes |  |
| `metric` | `AlertMetric` | yes |  |
| `name` | string | yes |  |
| `severity` | `AlertSeverity` | yes |  |
| `threshold` | number | yes |  |

### `AlertSeverity`

Enum: `warning`, `critical`

### `ApiTokenInfo`

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `created_at` | integer | yes |  |
| `expires_at` | integer | no |  |
| `id` | string | yes |  |
| `name` | string | yes |  |
| `pool` | string | no |  |
| `role` | `Role` | yes |  |

### `BlockDevice`

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `dev_type` | string | yes |  |
| `device_class` | string | yes | Device speed class: "nvme", "ssd", or "hdd". |
| `fs_type` | string | no |  |
| `in_use` | boolean | yes |  |
| `mount_point` | string | no |  |
| `path` | string | yes |  |
| `rotational` | boolean | yes | Whether the underlying disk spins (false for NVMe/SSD, true for HDD). |
| `size_bytes` | integer | yes |  |

### `CpuStats`

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `count` | integer | yes |  |
| `load_1` | number | yes |  |
| `load_15` | number | yes |  |
| `load_5` | number | yes |  |

### `DeviceSpec`

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `durability` | integer | no | Durability: 0 = cache, 1 = normal, 2 = hardware RAID. |
| `label` | string | no | Hierarchical label (e.g. "ssd.fast", "hdd.archive"). |
| `path` | string | yes |  |

### `DeviceUsage`

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `free_bytes` | integer | yes |  |
| `path` | string | yes |  |
| `total_bytes` | integer | yes |  |
| `used_bytes` | integer | yes |  |

### `DiskHealth`

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `attributes` | `SmartAttribute`[] | yes |  |
| `capacity_bytes` | integer | yes |  |
| `device` | string | yes |  |
| `firmware` | string | yes |  |
| `health_passed` | boolean | yes |  |
| `model` | string | yes |  |
| `power_on_hours` | integer | no |  |
| `serial` | string | yes |  |
| `smart_status` | string | yes |  |
| `temperature_c` | integer | no |  |

### `DiskIoStats`

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `io_in_progress` | integer | yes |  |
| `name` | string | yes |  |
| `read_bytes` | integer | yes |  |
| `read_ios` | integer | yes |  |
| `write_bytes` | integer | yes |  |
| `write_ios` | integer | yes |  |

### `IscsiTarget`

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `acls` | `Acl`[] | yes |  |
| `alias` | string | no |  |
| `enabled` | boolean | yes |  |
| `id` | string | yes |  |
| `iqn` | string | yes |  |
| `luns` | `Lun`[] | yes |  |
| `portals` | `Portal`[] | yes |  |

### `Lun`

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `backstore_name` | string | yes | LIO backstore name (auto-generated) |
| `backstore_path` | string | yes | Path to block device or file used as backstore |
| `backstore_type` | string | yes | "block" or "fileio" |
| `lun_id` | integer | yes |  |
| `size_bytes` | integer | no |  |

### `MemoryStats`

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `available_bytes` | integer | yes |  |
| `swap_total_bytes` | integer | yes |  |
| `swap_used_bytes` | integer | yes |  |
| `total_bytes` | integer | yes |  |
| `used_bytes` | integer | yes |  |

### `Namespace`

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `device_path` | string | yes |  |
| `enabled` | boolean | yes |  |
| `nsid` | integer | yes |  |

### `NetIfStats`

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `addresses` | string[] | yes |  |
| `name` | string | yes |  |
| `rx_bytes` | integer | yes |  |
| `rx_packets` | integer | yes |  |
| `speed_mbps` | integer | no |  |
| `tx_bytes` | integer | yes |  |
| `tx_packets` | integer | yes |  |
| `up` | boolean | yes |  |

### `NfsClient`

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `host` | string | yes | Network or host: "192.168.1.0/24", "10.0.0.5", "*" |
| `options` | string | yes | NFS export options: "rw,sync,no_subtree_check,no_root_squash" |

### `NfsShare`

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `clients` | `NfsClient`[] | yes |  |
| `comment` | string | no |  |
| `enabled` | boolean | yes |  |
| `id` | string | yes |  |
| `path` | string | yes |  |

### `NvmeofSubsystem`

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `allow_any_host` | boolean | yes |  |
| `allowed_hosts` | string[] | yes |  |
| `enabled` | boolean | yes |  |
| `id` | string | yes |  |
| `namespaces` | `Namespace`[] | yes |  |
| `nqn` | string | yes |  |
| `ports` | `Port`[] | yes |  |

### `Pool`

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `available_bytes` | integer | yes |  |
| `devices` | `PoolDevice`[] | yes |  |
| `mount_point` | string | no |  |
| `mounted` | boolean | yes |  |
| `name` | string | yes |  |
| `options` | `PoolOptions` | yes | Filesystem-level options read from sysfs or show-super. |
| `total_bytes` | integer | yes |  |
| `used_bytes` | integer | yes |  |
| `uuid` | string | yes |  |

### `PoolDevice`

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `data_allowed` | string | no | Which data types are allowed on this device (e.g. "journal,btree,user"). |
| `discard` | boolean | no | Whether TRIM/discard is enabled on this device. |
| `durability` | integer | no | How many replicas a copy on this device counts for.
0 = cache only, 1 = normal (default), 2 = hardware RAID. |
| `has_data` | string | no | Which data types are currently stored on this device (e.g. "btree,user"). |
| `label` | string | no | Hierarchical label (e.g. "ssd.fast", "hdd.archive").
Used for target-based tiering. |
| `path` | string | yes |  |
| `state` | string | no | Persistent device state: rw, ro, evacuating, spare. |

### `PoolOptions`

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `background_compression` | string | no |  |
| `background_target` | string | no |  |
| `compression` | string | no |  |
| `data_checksum` | string | no |  |
| `data_replicas` | integer | no |  |
| `encrypted` | boolean | no |  |
| `erasure_code` | boolean | no |  |
| `error_action` | string | no |  |
| `foreground_target` | string | no |  |
| `metadata_checksum` | string | no |  |
| `metadata_replicas` | integer | no |  |
| `metadata_target` | string | no |  |
| `promote_target` | string | no |  |

### `Port`

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `addr` | string | yes |  |
| `addr_family` | string | yes |  |
| `port_id` | integer | yes |  |
| `service_id` | string | yes |  |
| `transport` | string | yes |  |

### `Portal`

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `ip` | string | yes |  |
| `port` | integer | yes |  |

### `ProtocolStatus`

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `display_name` | string | yes |  |
| `enabled` | boolean | yes |  |
| `name` | string | yes |  |
| `running` | boolean | yes |  |
| `system_service` | boolean | yes |  |

### `Role`

Enum: `admin`, `readonly`, `operator`

### `ServiceStatus`

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `name` | string | yes |  |
| `running` | boolean | yes |  |

### `SmartAttribute`

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `failing` | boolean | yes |  |
| `id` | integer | yes |  |
| `name` | string | yes |  |
| `raw_value` | integer | yes |  |
| `threshold` | integer | yes |  |
| `value` | integer | yes |  |
| `worst` | integer | yes |  |

### `SmbShare`

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `browseable` | boolean | yes |  |
| `comment` | string | no |  |
| `enabled` | boolean | yes |  |
| `extra_params` | object | yes |  |
| `guest_ok` | boolean | yes |  |
| `id` | string | yes |  |
| `name` | string | yes |  |
| `path` | string | yes |  |
| `read_only` | boolean | yes |  |
| `valid_users` | string[] | yes |  |

### `Snapshot`

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `block_device` | string | no | Loop device path if this snapshot's vol.img is currently attached (block snapshots only). |
| `name` | string | yes |  |
| `path` | string | yes |  |
| `pool` | string | yes |  |
| `read_only` | boolean | yes |  |
| `subvolume` | string | yes |  |

### `Subvolume`

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `block_device` | string | no |  |
| `comments` | string | no |  |
| `compression` | string | no |  |
| `name` | string | yes |  |
| `owner` | string | no | Token name that created this subvolume; None for subvolumes created by human users. |
| `path` | string | yes |  |
| `pool` | string | yes |  |
| `properties` | object | no | Arbitrary key-value metadata stored as POSIX xattrs (user.* namespace).
Used by nasty-csi to track CSI volume metadata without sidecar files. |
| `snapshots` | string[] | yes |  |
| `subvolume_type` | `SubvolumeType` | yes |  |
| `used_bytes` | integer | no |  |
| `volsize_bytes` | integer | no |  |

### `SubvolumeType`

Enum: `filesystem`, `block`

### `UserInfo`

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `role` | `Role` | yes |  |
| `username` | string | yes |  |

