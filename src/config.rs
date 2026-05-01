//! ~/.glctl/config — token storage and retrieval.

use crate::{CliError, CliResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote: Option<String>,
}

pub fn config_path() -> CliResult<PathBuf> {
    let home = dirs_home()?;
    Ok(home.join(".glctl").join("config"))
}

pub fn load() -> CliResult<Config> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }
    let text = fs::read_to_string(&path)
        .map_err(|e| CliError::Error(format!("cannot read {}: {}", path.display(), e)))?;
    serde_json::from_str(&text)
        .map_err(|e| CliError::Error(format!("cannot parse {}: {}", path.display(), e)))
}

pub fn save(cfg: &Config) -> CliResult<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| CliError::Error(format!("cannot create config dir: {}", e)))?;
    }
    let text = serde_json::to_string_pretty(cfg)
        .map_err(|e| CliError::Error(format!("cannot serialize config: {}", e)))?;
    fs::write(&path, text)
        .map_err(|e| CliError::Error(format!("cannot write {}: {}", path.display(), e)))?;
    // restrict permissions to owner-only on unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
            .map_err(|e| CliError::Error(format!("cannot chmod config: {}", e)))?;
    }
    Ok(())
}

fn dirs_home() -> CliResult<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| CliError::Error("HOME environment variable not set".to_string()))
}
