use std::borrow::Cow;
use std::path::PathBuf;

use directories::ProjectDirs;
use eyre::{eyre, Result, WrapErr};
use indicatif::ProgressBar;

pub fn ensure_state_dir() -> Result<PathBuf> {
    let state_dir = ProjectDirs::from("me", "lightquantum", "tmexclude")
        .ok_or_else(|| eyre!("Home directory not found"))?
        .data_local_dir()
        .to_path_buf();
    std::fs::create_dir_all(&state_dir).wrap_err("Failed to create state directory")?;
    Ok(state_dir)
}

pub fn spinner(msg: impl Into<Cow<'static, str>> + Send + 'static) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_message(msg);
    spinner.enable_steady_tick(100);
    spinner
}
