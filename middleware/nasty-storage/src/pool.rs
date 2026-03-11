use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;

use crate::cmd;

const NASTY_MOUNT_BASE: &str = "/mnt/nasty";

#[derive(Debug, Error)]
pub enum PoolError {
    #[error("bcachefs command failed: {0}")]
    CommandFailed(String),
    #[error("pool not found: {0}")]
    NotFound(String),
    #[error("pool already exists: {0}")]
    AlreadyExists(String),
    #[error("device {0} is already in use")]
    DeviceInUse(String),
    #[error("no devices specified")]
    NoDevices,
    #[error("device not found: {0}")]
    DeviceNotFound(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pool {
    pub name: String,
    pub uuid: String,
    pub devices: Vec<String>,
    pub mount_point: Option<String>,
    pub mounted: bool,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub compression: Option<String>,
    pub replicas: u32,
}

#[derive(Debug, Deserialize)]
pub struct CreatePoolRequest {
    pub name: String,
    pub devices: Vec<String>,
    #[serde(default = "default_replicas")]
    pub replicas: u32,
    pub compression: Option<String>,
    pub encryption: Option<bool>,
    pub label: Option<String>,
}

fn default_replicas() -> u32 {
    1
}

#[derive(Debug, Deserialize)]
pub struct DestroyPoolRequest {
    pub name: String,
    pub force: Option<bool>,
}

pub struct PoolService;

impl PoolService {
    pub fn new() -> Self {
        Self
    }

    /// List all bcachefs filesystems (mounted and known via blkid)
    pub async fn list(&self) -> Result<Vec<Pool>, PoolError> {
        let mounts = read_bcachefs_mounts().await?;
        let mut pools = Vec::new();

        for (mount_point, devices) in &mounts {
            let uuid = get_fs_uuid(devices.first().map(|s| s.as_str()).unwrap_or(""))
                .await
                .unwrap_or_default();

            let (total, used, available) = get_mount_usage(mount_point).await.unwrap_or((0, 0, 0));

            let name = Path::new(mount_point)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            pools.push(Pool {
                name,
                uuid,
                devices: devices.clone(),
                mount_point: Some(mount_point.clone()),
                mounted: true,
                total_bytes: total,
                used_bytes: used,
                available_bytes: available,
                compression: None,
                replicas: 1,
            });
        }

        Ok(pools)
    }

    /// Get a single pool by name
    pub async fn get(&self, name: &str) -> Result<Pool, PoolError> {
        let pools = self.list().await?;
        pools
            .into_iter()
            .find(|p| p.name == name)
            .ok_or_else(|| PoolError::NotFound(name.to_string()))
    }

    /// Create a new bcachefs pool: format devices, create mount point, mount
    pub async fn create(&self, req: CreatePoolRequest) -> Result<Pool, PoolError> {
        if req.devices.is_empty() {
            return Err(PoolError::NoDevices);
        }

        // Validate devices exist
        for dev in &req.devices {
            if !Path::new(dev).exists() {
                return Err(PoolError::DeviceNotFound(dev.clone()));
            }
        }

        // Check devices aren't already in use by a bcachefs filesystem
        for dev in &req.devices {
            if is_device_bcachefs(dev).await {
                return Err(PoolError::DeviceInUse(dev.clone()));
            }
        }

        // Check mount point doesn't already exist with content
        let mount_point = format!("{NASTY_MOUNT_BASE}/{}", req.name);
        if Path::new(&mount_point).exists() {
            return Err(PoolError::AlreadyExists(req.name.clone()));
        }

        // Build bcachefs format command
        let mut args: Vec<String> = vec!["format".to_string()];

        let label = req.label.as_deref().unwrap_or(&req.name);
        args.push("--label".to_string());
        args.push(label.to_string());

        if req.replicas > 1 {
            args.push(format!("--replicas={}", req.replicas));
        }

        if let Some(ref comp) = req.compression {
            args.push(format!("--compression={comp}"));
        }

        if req.encryption == Some(true) {
            args.push("--encrypted".to_string());
        }

        for dev in &req.devices {
            args.push(dev.clone());
        }

        // Format
        let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        info!("Formatting bcachefs pool '{}' on {:?}", req.name, req.devices);
        cmd::run_ok("bcachefs", &arg_refs)
            .await
            .map_err(PoolError::CommandFailed)?;

        // Create mount point
        tokio::fs::create_dir_all(&mount_point).await?;

        // Mount — bcachefs mount takes device(s) colon-separated
        let device_arg = req.devices.join(":");
        info!("Mounting pool '{}' at {}", req.name, mount_point);
        cmd::run_ok("bcachefs", &["mount", &device_arg, &mount_point])
            .await
            .map_err(PoolError::CommandFailed)?;

        // Read back the pool info
        let uuid = get_fs_uuid(&req.devices[0]).await.unwrap_or_default();
        let (total, used, available) = get_mount_usage(&mount_point).await.unwrap_or((0, 0, 0));

        Ok(Pool {
            name: req.name,
            uuid,
            devices: req.devices,
            mount_point: Some(mount_point),
            mounted: true,
            total_bytes: total,
            used_bytes: used,
            available_bytes: available,
            compression: req.compression,
            replicas: req.replicas,
        })
    }

    /// Unmount and optionally wipe a pool
    pub async fn destroy(&self, req: DestroyPoolRequest) -> Result<(), PoolError> {
        let pool = self.get(&req.name).await?;

        // Unmount if mounted
        if let Some(ref mp) = pool.mount_point {
            info!("Unmounting pool '{}' from {}", req.name, mp);
            cmd::run_ok("umount", &[mp.as_str()])
                .await
                .map_err(PoolError::CommandFailed)?;

            // Remove mount point directory
            let _ = tokio::fs::remove_dir(mp).await;
        }

        // If force, wipe the superblocks
        if req.force == Some(true) {
            for dev in &pool.devices {
                info!("Wiping bcachefs superblock on {dev}");
                let _ = cmd::run_ok("wipefs", &["-a", dev]).await;
            }
        }

        Ok(())
    }

    /// Mount an existing unmounted pool
    pub async fn mount(&self, name: &str) -> Result<Pool, PoolError> {
        let pool = self.get(name).await?;
        if pool.mounted {
            return Ok(pool);
        }

        let mount_point = format!("{NASTY_MOUNT_BASE}/{name}");
        tokio::fs::create_dir_all(&mount_point).await?;

        let device_arg = pool.devices.join(":");
        cmd::run_ok("bcachefs", &["mount", &device_arg, &mount_point])
            .await
            .map_err(PoolError::CommandFailed)?;

        self.get(name).await
    }

    /// Unmount a pool
    pub async fn unmount(&self, name: &str) -> Result<(), PoolError> {
        let pool = self.get(name).await?;
        if let Some(ref mp) = pool.mount_point {
            cmd::run_ok("umount", &[mp.as_str()])
                .await
                .map_err(PoolError::CommandFailed)?;
        }
        Ok(())
    }

    /// List block devices available for pool creation
    pub async fn list_devices(&self) -> Result<Vec<BlockDevice>, PoolError> {
        let output = cmd::run_ok("lsblk", &["-Jbno", "NAME,SIZE,TYPE,MOUNTPOINT,FSTYPE"])
            .await
            .map_err(PoolError::CommandFailed)?;

        let parsed: serde_json::Value =
            serde_json::from_str(&output).unwrap_or(serde_json::Value::Null);

        let mut devices = Vec::new();
        if let Some(blockdevices) = parsed.get("blockdevices").and_then(|v| v.as_array()) {
            fn collect_devices(devs: &[serde_json::Value], out: &mut Vec<BlockDevice>) {
                for dev in devs {
                    let name = dev.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let dev_type = dev.get("type").and_then(|v| v.as_str()).unwrap_or("");
                    let size = dev
                        .get("size")
                        .and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
                        .unwrap_or(0);
                    let mountpoint = dev.get("mountpoint").and_then(|v| v.as_str()).map(String::from);
                    let fstype = dev.get("fstype").and_then(|v| v.as_str()).map(String::from);

                    if dev_type == "disk" || dev_type == "part" {
                        out.push(BlockDevice {
                            path: format!("/dev/{name}"),
                            size_bytes: size,
                            dev_type: dev_type.to_string(),
                            mount_point: mountpoint,
                            fs_type: fstype,
                            in_use: dev.get("mountpoint").and_then(|v| v.as_str()).is_some(),
                        });
                    }

                    if let Some(children) = dev.get("children").and_then(|v| v.as_array()) {
                        collect_devices(children, out);
                    }
                }
            }
            collect_devices(blockdevices, &mut devices);
        }

        Ok(devices)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDevice {
    pub path: String,
    pub size_bytes: u64,
    pub dev_type: String,
    pub mount_point: Option<String>,
    pub fs_type: Option<String>,
    pub in_use: bool,
}

/// Parse /proc/mounts for bcachefs entries.
/// Returns map of mount_point -> list of devices.
async fn read_bcachefs_mounts() -> Result<HashMap<String, Vec<String>>, PoolError> {
    let content = tokio::fs::read_to_string("/proc/mounts")
        .await
        .unwrap_or_default();

    let mut mounts: HashMap<String, Vec<String>> = HashMap::new();

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 && parts[2] == "bcachefs" {
            let device_str = parts[0]; // could be "dev1:dev2" for multi-device
            let mount_point = parts[1].to_string();
            let devices: Vec<String> = device_str.split(':').map(String::from).collect();
            mounts.insert(mount_point, devices);
        }
    }

    Ok(mounts)
}

/// Get the bcachefs UUID for a device using blkid
async fn get_fs_uuid(device: &str) -> Option<String> {
    let output = cmd::run_ok("blkid", &["-s", "UUID", "-o", "value", device])
        .await
        .ok()?;
    let uuid = output.trim().to_string();
    if uuid.is_empty() { None } else { Some(uuid) }
}

/// Get filesystem usage via statvfs-style info from `df`
async fn get_mount_usage(mount_point: &str) -> Option<(u64, u64, u64)> {
    let output = cmd::run_ok("df", &["-B1", "--output=size,used,avail", mount_point])
        .await
        .ok()?;

    // Skip header line, parse second line
    let line = output.lines().nth(1)?;
    let nums: Vec<u64> = line.split_whitespace().filter_map(|s| s.parse().ok()).collect();
    if nums.len() == 3 {
        Some((nums[0], nums[1], nums[2]))
    } else {
        None
    }
}

/// Check if a device already has a bcachefs filesystem
async fn is_device_bcachefs(device: &str) -> bool {
    cmd::run_ok("blkid", &["-s", "TYPE", "-o", "value", device])
        .await
        .map(|s| s.trim() == "bcachefs")
        .unwrap_or(false)
}
