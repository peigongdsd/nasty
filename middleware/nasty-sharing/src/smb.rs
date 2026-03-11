use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;
use uuid::Uuid;

const NASTY_SMB_CONF_PATH: &str = "/etc/samba/smb.nasty.conf";
const STATE_PATH: &str = "/var/lib/nasty/smb-shares.json";
const STATE_DIR: &str = "/var/lib/nasty";

#[derive(Debug, Error)]
pub enum SmbError {
    #[error("share not found: {0}")]
    NotFound(String),
    #[error("share name already exists: {0}")]
    NameExists(String),
    #[error("path does not exist: {0}")]
    PathNotFound(String),
    #[error("path is not within a NASty pool: {0}")]
    PathNotInPool(String),
    #[error("invalid share name: {0}")]
    InvalidName(String),
    #[error("samba reload failed: {0}")]
    ReloadFailed(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmbShare {
    pub id: String,
    /// Share name as seen by clients (e.g. "documents")
    pub name: String,
    pub path: String,
    pub comment: Option<String>,
    pub read_only: bool,
    pub browseable: bool,
    pub guest_ok: bool,
    /// Restrict to these users (empty = all authenticated users)
    pub valid_users: Vec<String>,
    /// Extra smb.conf parameters for this share
    pub extra_params: HashMap<String, String>,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateSmbShareRequest {
    pub name: String,
    pub path: String,
    pub comment: Option<String>,
    pub read_only: Option<bool>,
    pub browseable: Option<bool>,
    pub guest_ok: Option<bool>,
    pub valid_users: Option<Vec<String>>,
    pub extra_params: Option<HashMap<String, String>>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSmbShareRequest {
    pub id: String,
    pub name: Option<String>,
    pub comment: Option<String>,
    pub read_only: Option<bool>,
    pub browseable: Option<bool>,
    pub guest_ok: Option<bool>,
    pub valid_users: Option<Vec<String>>,
    pub extra_params: Option<HashMap<String, String>>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteSmbShareRequest {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SmbState {
    shares: Vec<SmbShare>,
}

pub struct SmbService;

impl SmbService {
    pub fn new() -> Self {
        Self
    }

    pub async fn list(&self) -> Result<Vec<SmbShare>, SmbError> {
        let state = load_state().await;
        Ok(state.shares)
    }

    pub async fn get(&self, id: &str) -> Result<SmbShare, SmbError> {
        let state = load_state().await;
        state
            .shares
            .into_iter()
            .find(|s| s.id == id)
            .ok_or_else(|| SmbError::NotFound(id.to_string()))
    }

    pub async fn create(&self, req: CreateSmbShareRequest) -> Result<SmbShare, SmbError> {
        validate_share_name(&req.name)?;

        if !Path::new(&req.path).exists() {
            return Err(SmbError::PathNotFound(req.path));
        }
        if !req.path.starts_with("/mnt/nasty/") {
            return Err(SmbError::PathNotInPool(req.path));
        }

        let mut state = load_state().await;

        if state.shares.iter().any(|s| s.name == req.name) {
            return Err(SmbError::NameExists(req.name));
        }

        let share = SmbShare {
            id: Uuid::new_v4().to_string(),
            name: req.name,
            path: req.path,
            comment: req.comment,
            read_only: req.read_only.unwrap_or(false),
            browseable: req.browseable.unwrap_or(true),
            guest_ok: req.guest_ok.unwrap_or(false),
            valid_users: req.valid_users.unwrap_or_default(),
            extra_params: req.extra_params.unwrap_or_default(),
            enabled: req.enabled.unwrap_or(true),
        };

        state.shares.push(share.clone());
        save_state(&state).await?;
        apply_config(&state).await?;

        info!("Created SMB share '{}' at {}", share.name, share.path);
        Ok(share)
    }

    pub async fn update(&self, req: UpdateSmbShareRequest) -> Result<SmbShare, SmbError> {
        let mut state = load_state().await;

        // Check name uniqueness if changing
        if let Some(ref new_name) = req.name {
            validate_share_name(new_name)?;
            if state
                .shares
                .iter()
                .any(|s| s.name == *new_name && s.id != req.id)
            {
                return Err(SmbError::NameExists(new_name.clone()));
            }
        }

        let share = state
            .shares
            .iter_mut()
            .find(|s| s.id == req.id)
            .ok_or_else(|| SmbError::NotFound(req.id.clone()))?;

        if let Some(name) = req.name {
            share.name = name;
        }
        if let Some(comment) = req.comment {
            share.comment = Some(comment);
        }
        if let Some(read_only) = req.read_only {
            share.read_only = read_only;
        }
        if let Some(browseable) = req.browseable {
            share.browseable = browseable;
        }
        if let Some(guest_ok) = req.guest_ok {
            share.guest_ok = guest_ok;
        }
        if let Some(valid_users) = req.valid_users {
            share.valid_users = valid_users;
        }
        if let Some(extra_params) = req.extra_params {
            share.extra_params = extra_params;
        }
        if let Some(enabled) = req.enabled {
            share.enabled = enabled;
        }

        let updated = share.clone();
        save_state(&state).await?;
        apply_config(&state).await?;

        info!("Updated SMB share '{}'", updated.name);
        Ok(updated)
    }

    pub async fn delete(&self, req: DeleteSmbShareRequest) -> Result<(), SmbError> {
        let mut state = load_state().await;
        let len_before = state.shares.len();
        state.shares.retain(|s| s.id != req.id);

        if state.shares.len() == len_before {
            return Err(SmbError::NotFound(req.id));
        }

        save_state(&state).await?;
        apply_config(&state).await?;

        info!("Deleted SMB share '{}'", req.id);
        Ok(())
    }
}

fn validate_share_name(name: &str) -> Result<(), SmbError> {
    if name.is_empty()
        || name.len() > 80
        || name.contains(['/', '\\', '[', ']', ':', '|', '<', '>', '+', '=', ';', ',', '?', '*'])
    {
        return Err(SmbError::InvalidName(
            "Share name must be 1-80 chars without special characters".to_string(),
        ));
    }
    Ok(())
}

async fn load_state() -> SmbState {
    match tokio::fs::read_to_string(STATE_PATH).await {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => SmbState::default(),
    }
}

async fn save_state(state: &SmbState) -> Result<(), SmbError> {
    tokio::fs::create_dir_all(STATE_DIR).await?;
    let json = serde_json::to_string_pretty(state).unwrap();
    tokio::fs::write(STATE_PATH, json).await?;
    Ok(())
}

/// Generate smb.nasty.conf with share sections and reload samba.
///
/// The main /etc/samba/smb.conf should include this file:
///   include = /etc/samba/smb.nasty.conf
fn generate_conf(state: &SmbState) -> String {
    let mut conf = String::from("# Managed by NASty — do not edit manually\n\n");

    for share in &state.shares {
        if !share.enabled {
            continue;
        }

        conf.push_str(&format!("[{}]\n", share.name));
        conf.push_str(&format!("    path = {}\n", share.path));

        if let Some(ref comment) = share.comment {
            conf.push_str(&format!("    comment = {comment}\n"));
        }

        conf.push_str(&format!(
            "    read only = {}\n",
            if share.read_only { "yes" } else { "no" }
        ));
        conf.push_str(&format!(
            "    browseable = {}\n",
            if share.browseable { "yes" } else { "no" }
        ));
        conf.push_str(&format!(
            "    guest ok = {}\n",
            if share.guest_ok { "yes" } else { "no" }
        ));

        if !share.valid_users.is_empty() {
            conf.push_str(&format!(
                "    valid users = {}\n",
                share.valid_users.join(" ")
            ));
        }

        // Sort extra params for deterministic output
        let mut extra: Vec<_> = share.extra_params.iter().collect();
        extra.sort_by_key(|(k, _)| *k);
        for (key, value) in extra {
            conf.push_str(&format!("    {key} = {value}\n"));
        }

        conf.push('\n');
    }

    conf
}

async fn apply_config(state: &SmbState) -> Result<(), SmbError> {
    let conf = generate_conf(state);
    tokio::fs::write(NASTY_SMB_CONF_PATH, &conf).await?;
    reload_samba().await?;
    Ok(())
}

async fn reload_samba() -> Result<(), SmbError> {
    let output = tokio::process::Command::new("smbcontrol")
        .args(["all", "reload-config"])
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SmbError::ReloadFailed(stderr.to_string()));
    }

    info!("Samba configuration reloaded");
    Ok(())
}
