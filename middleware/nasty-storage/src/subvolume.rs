use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;

use crate::cmd;
use crate::pool::PoolService;

#[derive(Debug, Error)]
pub enum SubvolumeError {
    #[error("pool not found: {0}")]
    PoolNotFound(String),
    #[error("pool not mounted: {0}")]
    PoolNotMounted(String),
    #[error("subvolume already exists: {0}")]
    AlreadyExists(String),
    #[error("subvolume not found: {0}")]
    NotFound(String),
    #[error("bcachefs command failed: {0}")]
    CommandFailed(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subvolume {
    pub name: String,
    pub pool: String,
    pub path: String,
    pub size_bytes: Option<u64>,
    pub snapshots: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub name: String,
    pub subvolume: String,
    pub pool: String,
    pub path: String,
    pub read_only: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateSubvolumeRequest {
    pub pool: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteSubvolumeRequest {
    pub pool: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateSnapshotRequest {
    pub pool: String,
    pub subvolume: String,
    pub name: String,
    pub read_only: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteSnapshotRequest {
    pub pool: String,
    pub subvolume: String,
    pub name: String,
}

pub struct SubvolumeService {
    pools: PoolService,
}

impl SubvolumeService {
    pub fn new(pools: PoolService) -> Self {
        Self { pools }
    }

    /// Get the mount point for a pool, or error if not mounted
    async fn pool_mount_point(&self, pool_name: &str) -> Result<String, SubvolumeError> {
        let pool = self
            .pools
            .get(pool_name)
            .await
            .map_err(|_| SubvolumeError::PoolNotFound(pool_name.to_string()))?;

        pool.mount_point
            .ok_or_else(|| SubvolumeError::PoolNotMounted(pool_name.to_string()))
    }

    /// List subvolumes in a pool by scanning direct children of the mount point
    pub async fn list(&self, pool_name: &str) -> Result<Vec<Subvolume>, SubvolumeError> {
        let mount_point = self.pool_mount_point(pool_name).await?;
        let mut subvolumes = Vec::new();

        let mut entries = tokio::fs::read_dir(&mount_point).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip the .snapshots directory
            if name == ".snapshots" {
                continue;
            }

            // Check if this is a bcachefs subvolume by trying to list snapshots
            // A subvolume is a directory created via `bcachefs subvolume create`
            if path.is_dir() && is_subvolume(&path).await {
                let snapshots = self
                    .list_snapshots_for(pool_name, &name)
                    .await
                    .unwrap_or_default();

                let size = dir_usage(&path).await;

                subvolumes.push(Subvolume {
                    name: name.clone(),
                    pool: pool_name.to_string(),
                    path: path.to_string_lossy().to_string(),
                    size_bytes: size,
                    snapshots: snapshots.iter().map(|s| s.name.clone()).collect(),
                });
            }
        }

        Ok(subvolumes)
    }

    /// Get a single subvolume
    pub async fn get(
        &self,
        pool_name: &str,
        name: &str,
    ) -> Result<Subvolume, SubvolumeError> {
        let mount_point = self.pool_mount_point(pool_name).await?;
        let path = format!("{mount_point}/{name}");

        if !Path::new(&path).exists() {
            return Err(SubvolumeError::NotFound(name.to_string()));
        }

        let snapshots = self
            .list_snapshots_for(pool_name, name)
            .await
            .unwrap_or_default();

        let size = dir_usage(Path::new(&path)).await;

        Ok(Subvolume {
            name: name.to_string(),
            pool: pool_name.to_string(),
            path,
            size_bytes: size,
            snapshots: snapshots.iter().map(|s| s.name.clone()).collect(),
        })
    }

    /// Create a new subvolume
    pub async fn create(&self, req: CreateSubvolumeRequest) -> Result<Subvolume, SubvolumeError> {
        let mount_point = self.pool_mount_point(&req.pool).await?;
        let subvol_path = format!("{mount_point}/{}", req.name);

        if Path::new(&subvol_path).exists() {
            return Err(SubvolumeError::AlreadyExists(req.name.clone()));
        }

        info!("Creating subvolume '{}' in pool '{}'", req.name, req.pool);
        cmd::run_ok("bcachefs", &["subvolume", "create", &subvol_path])
            .await
            .map_err(SubvolumeError::CommandFailed)?;

        self.get(&req.pool, &req.name).await
    }

    /// Delete a subvolume
    pub async fn delete(&self, req: DeleteSubvolumeRequest) -> Result<(), SubvolumeError> {
        let mount_point = self.pool_mount_point(&req.pool).await?;
        let subvol_path = format!("{mount_point}/{}", req.name);

        if !Path::new(&subvol_path).exists() {
            return Err(SubvolumeError::NotFound(req.name.clone()));
        }

        // Delete all snapshots for this subvolume first
        let snapshots = self.list_snapshots_for(&req.pool, &req.name).await.unwrap_or_default();
        for snap in snapshots {
            info!("Deleting snapshot '{}' before subvolume deletion", snap.name);
            cmd::run_ok("bcachefs", &["subvolume", "delete", &snap.path])
                .await
                .map_err(SubvolumeError::CommandFailed)?;
        }

        info!("Deleting subvolume '{}' from pool '{}'", req.name, req.pool);
        cmd::run_ok("bcachefs", &["subvolume", "delete", &subvol_path])
            .await
            .map_err(SubvolumeError::CommandFailed)?;

        Ok(())
    }

    /// Create a snapshot of a subvolume
    pub async fn create_snapshot(
        &self,
        req: CreateSnapshotRequest,
    ) -> Result<Snapshot, SubvolumeError> {
        let mount_point = self.pool_mount_point(&req.pool).await?;
        let source_path = format!("{mount_point}/{}", req.subvolume);
        let snap_dir = format!("{mount_point}/.snapshots/{}", req.subvolume);
        let snap_path = format!("{snap_dir}/{}", req.name);

        if !Path::new(&source_path).exists() {
            return Err(SubvolumeError::NotFound(req.subvolume.clone()));
        }

        if Path::new(&snap_path).exists() {
            return Err(SubvolumeError::AlreadyExists(req.name.clone()));
        }

        // Ensure snapshot directory exists
        tokio::fs::create_dir_all(&snap_dir).await?;

        let mut args = vec!["subvolume", "snapshot"];
        if req.read_only == Some(true) {
            args.push("-r");
        }
        args.push(&source_path);
        args.push(&snap_path);

        info!(
            "Creating snapshot '{}' of subvolume '{}/{}'",
            req.name, req.pool, req.subvolume
        );
        cmd::run_ok("bcachefs", &args)
            .await
            .map_err(SubvolumeError::CommandFailed)?;

        Ok(Snapshot {
            name: req.name,
            subvolume: req.subvolume,
            pool: req.pool,
            path: snap_path,
            read_only: req.read_only.unwrap_or(false),
        })
    }

    /// Delete a snapshot
    pub async fn delete_snapshot(
        &self,
        req: DeleteSnapshotRequest,
    ) -> Result<(), SubvolumeError> {
        let mount_point = self.pool_mount_point(&req.pool).await?;
        let snap_path = format!(
            "{mount_point}/.snapshots/{}/{}",
            req.subvolume, req.name
        );

        if !Path::new(&snap_path).exists() {
            return Err(SubvolumeError::NotFound(req.name.clone()));
        }

        info!(
            "Deleting snapshot '{}' of subvolume '{}/{}'",
            req.name, req.pool, req.subvolume
        );
        cmd::run_ok("bcachefs", &["subvolume", "delete", &snap_path])
            .await
            .map_err(SubvolumeError::CommandFailed)?;

        Ok(())
    }

    /// List snapshots for a specific subvolume
    pub async fn list_snapshots_for(
        &self,
        pool_name: &str,
        subvol_name: &str,
    ) -> Result<Vec<Snapshot>, SubvolumeError> {
        let mount_point = self.pool_mount_point(pool_name).await?;
        let snap_dir = format!("{mount_point}/.snapshots/{subvol_name}");

        if !Path::new(&snap_dir).exists() {
            return Ok(vec![]);
        }

        let mut snapshots = Vec::new();
        let mut entries = tokio::fs::read_dir(&snap_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.path().is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                snapshots.push(Snapshot {
                    name,
                    subvolume: subvol_name.to_string(),
                    pool: pool_name.to_string(),
                    path: entry.path().to_string_lossy().to_string(),
                    read_only: false, // TODO: detect from bcachefs attributes
                });
            }
        }

        Ok(snapshots)
    }

    /// List all snapshots across all subvolumes in a pool
    pub async fn list_snapshots(
        &self,
        pool_name: &str,
    ) -> Result<Vec<Snapshot>, SubvolumeError> {
        let mount_point = self.pool_mount_point(pool_name).await?;
        let snap_base = format!("{mount_point}/.snapshots");

        if !Path::new(&snap_base).exists() {
            return Ok(vec![]);
        }

        let mut all_snapshots = Vec::new();
        let mut entries = tokio::fs::read_dir(&snap_base).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.path().is_dir() {
                let subvol_name = entry.file_name().to_string_lossy().to_string();
                let mut snaps = self
                    .list_snapshots_for(pool_name, &subvol_name)
                    .await
                    .unwrap_or_default();
                all_snapshots.append(&mut snaps);
            }
        }

        Ok(all_snapshots)
    }
}

/// Check if a directory is a bcachefs subvolume.
/// bcachefs subvolumes have a different inode number structure,
/// but the simplest heuristic is checking via the subvolume list command.
/// For now we treat all direct children dirs of the mount as subvolumes.
async fn is_subvolume(path: &Path) -> bool {
    // TODO: use `bcachefs subvolume list` or check xattrs to distinguish
    // real subvolumes from regular directories. For now, assume all
    // top-level dirs (except .snapshots) are subvolumes.
    path.is_dir()
}

/// Get disk usage for a directory using `du`
async fn dir_usage(path: &Path) -> Option<u64> {
    let path_str = path.to_string_lossy();
    let output = cmd::run_ok("du", &["-sb", &path_str]).await.ok()?;
    output
        .split_whitespace()
        .next()
        .and_then(|s| s.parse().ok())
}
