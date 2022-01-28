use std::path::PathBuf;

use directories::ProjectDirs;
use eyre::{eyre, Result, WrapErr};

pub fn ensure_state_dir() -> Result<PathBuf> {
    let state_dir = ProjectDirs::from("me", "lightquantum", "tmexclude")
        .ok_or_else(|| eyre!("Home directory not found"))?
        .data_local_dir()
        .to_path_buf();
    std::fs::create_dir_all(&state_dir).wrap_err("Failed to create state directory")?;
    Ok(state_dir)
}
