use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;
use uuid::Uuid;

const NASTY_EXPORTS_PATH: &str = "/etc/exports.d/nasty.exports";
const NASTY_EXPORTS_DIR: &str = "/etc/exports.d";
const STATE_PATH: &str = "/var/lib/nasty/nfs-shares.json";
const STATE_DIR: &str = "/var/lib/nasty";

#[derive(Debug, Error)]
pub enum NfsError {
    #[error("share not found: {0}")]
    NotFound(String),
    #[error("share already exists for path: {0}")]
    AlreadyExists(String),
    #[error("path does not exist: {0}")]
    PathNotFound(String),
    #[error("path is not within a NASty pool: {0}")]
    PathNotInPool(String),
    #[error("exportfs failed: {0}")]
    ExportFailed(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NfsShare {
    pub id: String,
    pub path: String,
    pub comment: Option<String>,
    pub clients: Vec<NfsClient>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NfsClient {
    /// Network or host: "192.168.1.0/24", "10.0.0.5", "*"
    pub host: String,
    /// NFS export options: "rw,sync,no_subtree_check,no_root_squash"
    pub options: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateNfsShareRequest {
    pub path: String,
    pub comment: Option<String>,
    pub clients: Vec<NfsClient>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNfsShareRequest {
    pub id: String,
    pub comment: Option<String>,
    pub clients: Option<Vec<NfsClient>>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteNfsShareRequest {
    pub id: String,
}

/// Persistent state: list of all NFS shares managed by NASty
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct NfsState {
    shares: Vec<NfsShare>,
}

pub struct NfsService;

impl NfsService {
    pub fn new() -> Self {
        Self
    }

    /// List all NFS shares
    pub async fn list(&self) -> Result<Vec<NfsShare>, NfsError> {
        let state = load_state().await;
        Ok(state.shares)
    }

    /// Get a single share by ID
    pub async fn get(&self, id: &str) -> Result<NfsShare, NfsError> {
        let state = load_state().await;
        state
            .shares
            .into_iter()
            .find(|s| s.id == id)
            .ok_or_else(|| NfsError::NotFound(id.to_string()))
    }

    /// Create a new NFS share
    pub async fn create(&self, req: CreateNfsShareRequest) -> Result<NfsShare, NfsError> {
        // Validate path exists
        if !Path::new(&req.path).exists() {
            return Err(NfsError::PathNotFound(req.path));
        }

        // Validate path is within /mnt/nasty/
        if !req.path.starts_with("/mnt/nasty/") {
            return Err(NfsError::PathNotInPool(req.path));
        }

        let mut state = load_state().await;

        // Check for duplicate path
        if state.shares.iter().any(|s| s.path == req.path) {
            return Err(NfsError::AlreadyExists(req.path));
        }

        let share = NfsShare {
            id: Uuid::new_v4().to_string(),
            path: req.path,
            comment: req.comment,
            clients: req.clients,
            enabled: req.enabled.unwrap_or(true),
        };

        state.shares.push(share.clone());
        save_state(&state).await?;
        apply_exports(&state).await?;

        info!("Created NFS share '{}' for {}", share.id, share.path);
        Ok(share)
    }

    /// Update an existing NFS share
    pub async fn update(&self, req: UpdateNfsShareRequest) -> Result<NfsShare, NfsError> {
        let mut state = load_state().await;

        let share = state
            .shares
            .iter_mut()
            .find(|s| s.id == req.id)
            .ok_or_else(|| NfsError::NotFound(req.id.clone()))?;

        if let Some(comment) = req.comment {
            share.comment = Some(comment);
        }
        if let Some(clients) = req.clients {
            share.clients = clients;
        }
        if let Some(enabled) = req.enabled {
            share.enabled = enabled;
        }

        let updated = share.clone();
        save_state(&state).await?;
        apply_exports(&state).await?;

        info!("Updated NFS share '{}'", updated.id);
        Ok(updated)
    }

    /// Delete an NFS share
    pub async fn delete(&self, req: DeleteNfsShareRequest) -> Result<(), NfsError> {
        let mut state = load_state().await;
        let len_before = state.shares.len();
        state.shares.retain(|s| s.id != req.id);

        if state.shares.len() == len_before {
            return Err(NfsError::NotFound(req.id));
        }

        save_state(&state).await?;
        apply_exports(&state).await?;

        info!("Deleted NFS share '{}'", req.id);
        Ok(())
    }
}

/// Load share state from disk
async fn load_state() -> NfsState {
    match tokio::fs::read_to_string(STATE_PATH).await {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => NfsState::default(),
    }
}

/// Persist share state to disk
async fn save_state(state: &NfsState) -> Result<(), NfsError> {
    tokio::fs::create_dir_all(STATE_DIR).await?;
    let json = serde_json::to_string_pretty(state).unwrap();
    tokio::fs::write(STATE_PATH, json).await?;
    Ok(())
}

/// Generate /etc/exports.d/nasty.exports and reload NFS
async fn apply_exports(state: &NfsState) -> Result<(), NfsError> {
    tokio::fs::create_dir_all(NASTY_EXPORTS_DIR).await?;

    let mut content = String::from("# Managed by NASty — do not edit manually\n\n");

    for share in &state.shares {
        if !share.enabled {
            continue;
        }

        if let Some(ref comment) = share.comment {
            content.push_str(&format!("# {comment}\n"));
        }

        // Format: /path client1(opts) client2(opts)
        let clients: Vec<String> = share
            .clients
            .iter()
            .map(|c| format!("{}({})", c.host, c.options))
            .collect();

        content.push_str(&format!("{}\t{}\n", share.path, clients.join(" ")));
    }

    tokio::fs::write(NASTY_EXPORTS_PATH, &content).await?;

    // Reload NFS exports
    reload_exports().await?;

    Ok(())
}

/// Run `exportfs -ra` to apply changes
async fn reload_exports() -> Result<(), NfsError> {
    let output = tokio::process::Command::new("exportfs")
        .args(["-ra"])
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(NfsError::ExportFailed(stderr.to_string()));
    }

    info!("NFS exports reloaded");
    Ok(())
}
