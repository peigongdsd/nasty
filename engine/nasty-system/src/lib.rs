pub mod alerts;
pub mod network;
pub mod protocol;
pub mod settings;
pub mod update;

// Re-export metrics types from nasty-common so downstream code
// (nasty-engine, alerts) can still use `nasty_system::SystemStats` etc.
pub use nasty_common::metrics_types::*;

use schemars::JsonSchema;
use serde::Serialize;

use std::sync::Arc;
use tokio::sync::RwLock;

/// Cached values that only change on bcachefs switch or reboot.
#[derive(Clone)]
struct CachedInfo {
    bcachefs_version: String,
    bcachefs_commit: Option<String>,
    bcachefs_pinned_ref: Option<String>,
    debug_symbols: bool,
    debug_checks: bool,
    /// Whether the RUNNING module is custom (version differs from default).
    bcachefs_is_custom_running: bool,
    /// Whether the RUNNING module has debug checks (sysfs reflects loaded module).
    bcachefs_debug_checks_running: bool,
}

pub struct SystemService {
    cached: Arc<RwLock<Option<CachedInfo>>>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct SystemInfo {
    /// System hostname.
    pub hostname: String,
    /// NASty engine version string.
    pub version: String,
    /// System uptime in seconds.
    pub uptime_seconds: u64,
    /// Running Linux kernel version string.
    pub kernel: String,
    /// Output of `bcachefs version` (first line).
    pub bcachefs_version: String,
    /// Short (12-char) commit SHA of the pinned bcachefs-tools in flake.lock
    pub bcachefs_commit: Option<String>,
    /// The ref stored in the state file: tag name (e.g. "v1.37.1") or short SHA
    pub bcachefs_pinned_ref: Option<String>,
    /// True when the RUNNING bcachefs module version differs from the default.
    pub bcachefs_is_custom: bool,
    /// IANA timezone string (e.g. `America/New_York`).
    pub timezone: String,
    /// Whether the system clock is NTP-synchronized.
    pub ntp_synced: bool,
    /// Whether the loaded bcachefs kernel module contains debug symbols.
    pub bcachefs_debug_symbols: bool,
    /// Whether the RUNNING bcachefs module was built with debug checks.
    /// Only true when debug checks are configured AND the system has been rebooted into it.
    pub bcachefs_debug_checks: bool,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct SystemHealth {
    /// Overall health status string (e.g. `ok`, `degraded`).
    pub status: String,
    /// Status of individual systemd services.
    pub services: Vec<ServiceStatus>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ServiceStatus {
    /// systemd service name.
    pub name: String,
    /// Whether the service is currently active/running.
    pub running: bool,
}

// SystemStats, CpuStats, MemoryStats, NetIfStats, DiskIoStats,
// DiskHealth, SmartAttribute — now defined in nasty_common::metrics_types
// and re-exported via `pub use` at the top of this file.

impl SystemService {
    pub fn new() -> Self {
        Self { cached: Arc::new(RwLock::new(None)) }
    }

    /// Invalidate cached bcachefs info — call after bcachefs switch or reboot.
    pub async fn invalidate_bcachefs_cache(&self) {
        *self.cached.write().await = None;
    }

    /// Return cached debug_symbols and debug_checks for the loaded module.
    /// Used by UpdateService to avoid re-running expensive detection on every page load.
    pub async fn cached_debug_flags(&self) -> (bool, bool) {
        let cached = self.get_cached_bcachefs().await;
        (cached.debug_symbols, cached.debug_checks)
    }

    async fn get_cached_bcachefs(&self) -> CachedInfo {
        {
            let guard = self.cached.read().await;
            if let Some(ref c) = *guard {
                return c.clone();
            }
        }
        // Compute — run subprocess calls in parallel.
        let (bcachefs_version, bcachefs_commit, pinned_ref_raw, debug_symbols, debug_checks, default_ref) = tokio::join!(
            bcachefs_version(),
            read_bcachefs_commit(),
            async {
                tokio::fs::read_to_string("/var/lib/nasty/bcachefs-tools-ref").await
                    .ok()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
            },
            bcachefs_has_debug_symbols(),
            bcachefs_has_debug_checks(),
            crate::update::read_flake_nix_default_ref_pub(),
        );
        // Running state: compare actual loaded module against default.
        // Strip leading 'v' from default ref for comparison (e.g. "v1.37.2" vs "1.37.2").
        let default_bare = default_ref.strip_prefix('v').unwrap_or(&default_ref);
        let bcachefs_is_custom_running = bcachefs_version != default_bare && bcachefs_version != "unknown";
        // Debug checks running: sysfs reflects the actually loaded module, so no
        // reboot_required guard needed (unlike the old state-file approach).
        let bcachefs_debug_checks_running = debug_checks;
        let info = CachedInfo {
            bcachefs_version,
            bcachefs_commit,
            bcachefs_pinned_ref: pinned_ref_raw,
            debug_symbols,
            debug_checks,
            bcachefs_is_custom_running,
            bcachefs_debug_checks_running,
        };
        *self.cached.write().await = Some(info.clone());
        info
    }

    pub async fn info(&self) -> SystemInfo {
        let cached = self.get_cached_bcachefs().await;
        let (timezone, ntp_synced) = timedatectl_info().await;

        SystemInfo {
            hostname: hostname(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: uptime_seconds(),
            kernel: kernel_version(),
            bcachefs_version: cached.bcachefs_version,
            bcachefs_commit: cached.bcachefs_commit,
            bcachefs_pinned_ref: cached.bcachefs_pinned_ref,
            bcachefs_is_custom: cached.bcachefs_is_custom_running,
            timezone,
            ntp_synced,
            bcachefs_debug_symbols: cached.debug_symbols,
            bcachefs_debug_checks: cached.bcachefs_debug_checks_running,
        }
    }

    pub async fn health(&self) -> SystemHealth {
        // TODO: check actual service status via systemd D-Bus
        SystemHealth {
            status: "ok".to_string(),
            services: vec![
                ServiceStatus { name: "nasty-engine".into(), running: true },
            ],
        }
    }

}

fn hostname() -> String {
    std::fs::read_to_string("/etc/hostname")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

fn kernel_version() -> String {
    std::fs::read_to_string("/proc/version")
        .map(|s| {
            s.split_whitespace()
                .nth(2)
                .unwrap_or("unknown")
                .to_string()
        })
        .unwrap_or_else(|_| "unknown".to_string())
}

async fn bcachefs_version() -> String {
    // Read the version of the currently loaded kernel module — this is the authoritative
    // running version. bcachefs version (userspace) can differ when a reboot is pending.
    let output = tokio::process::Command::new("modinfo")
        .args(["bcachefs", "--field", "version"])
        .output()
        .await;
    match output {
        Ok(o) if o.status.success() => {
            let v = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if v.is_empty() { "unknown".to_string() } else { v }
        }
        _ => "unknown".to_string(),
    }
}

/// Detect whether the loaded bcachefs kernel module contains debug symbols.
/// Decompresses the .ko.xz and pipes through `file` looking for "debug_info".
pub async fn bcachefs_has_debug_symbols() -> bool {
    // Get the module file path from modinfo
    let filename_out = tokio::process::Command::new("modinfo")
        .args(["bcachefs", "--field", "filename"])
        .output()
        .await;
    let ko_path = match filename_out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => return false,
    };
    if ko_path.is_empty() {
        return false;
    }
    // xz -dc <file> | file - → look for "debug_info"
    let xz = tokio::process::Command::new("sh")
        .args(["-c", &format!("xz -dc '{}' | file -", ko_path)])
        .output()
        .await;
    match xz {
        Ok(o) => String::from_utf8_lossy(&o.stdout).contains("debug_info"),
        Err(_) => false,
    }
}

/// Detect whether the loaded bcachefs kernel module was built with CONFIG_BCACHEFS_DEBUG.
/// BCH_DEBUG_PARAMS_DEBUG() params (e.g. journal_seq_verify) are only compiled in
/// when CONFIG_BCACHEFS_DEBUG is set. We check /sys/module/ which reflects the actually
/// loaded module, not the .ko on disk (which may have been rebuilt already).
pub async fn bcachefs_has_debug_checks() -> bool {
    tokio::fs::metadata("/sys/module/bcachefs/parameters/journal_seq_verify")
        .await
        .is_ok()
}

async fn timedatectl_info() -> (String, bool) {
    let output = tokio::process::Command::new("timedatectl")
        .args(["show", "--property=Timezone,NTPSynchronized"])
        .output()
        .await;

    let mut timezone = "UTC".to_string();
    let mut ntp_synced = false;

    if let Ok(out) = output {
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            if let Some(tz) = line.strip_prefix("Timezone=") {
                timezone = tz.trim().to_string();
            }
            if let Some(v) = line.strip_prefix("NTPSynchronized=") {
                ntp_synced = v.trim() == "yes";
            }
        }
    }

    // NTPSynchronized=yes only flips when timesyncd itself adjusts the clock.
    // On VMs the hypervisor pre-sets the clock so timesyncd never needs to step it,
    // leaving the flag as "no" even though the service is healthy and polling.
    // Fall back to checking whether timesyncd is actively running.
    if !ntp_synced {
        ntp_synced = systemd_unit_active("systemd-timesyncd").await;
    }

    (timezone, ntp_synced)
}

async fn systemd_unit_active(unit: &str) -> bool {
    tokio::process::Command::new("systemctl")
        .args(["is-active", "--quiet", unit])
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}

fn uptime_seconds() -> u64 {
    std::fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|s| s.split_whitespace().next().map(String::from))
        .and_then(|s| s.parse::<f64>().ok())
        .map(|f| f as u64)
        .unwrap_or(0)
}

// System stats collection (cpu_stats, memory_stats, network_stats, disk_io_stats)
// and SMART disk health (disk_health, query_smartctl) have moved to nasty-metrics.
// The engine fetches these via HTTP from nasty-metrics on port 2138.

/// Read the pinned bcachefs-tools commit SHA from flake.lock (12-char short form).
async fn read_bcachefs_commit() -> Option<String> {
    let content = tokio::fs::read_to_string("/etc/nixos/nixos/flake.lock").await.ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    let rev = v["nodes"]["bcachefs-tools"]["locked"]["rev"].as_str()?;
    Some(rev[..rev.len().min(12)].to_string())
}
