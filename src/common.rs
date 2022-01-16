use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use directories::UserDirs;
use eyre::{eyre, ContextCompat, Result};
use figment::Figment;
use log::Level;
use multi_log::MultiLogger;
use oslog::OsLogger;

use tmexclude_lib::errors::SuggestionExt;

use crate::{ensure_state_dir, FlexiProvider};

pub fn initialize_loggers() -> Result<()> {
    let mut env_logger_builder = pretty_env_logger::formatted_builder();
    if let Ok(filter) = std::env::var("RUST_LOG") {
        env_logger_builder.parse_filters(&filter);
    }
    let env_logger = env_logger_builder.build();

    let os_logger = OsLogger::new("me.lightquantum.tmexclude");
    Ok(MultiLogger::init(
        vec![Box::new(env_logger), Box::new(os_logger)],
        Level::max(),
    )?)
}

pub fn collect_provider(path: Option<PathBuf>, dry_run: bool) -> Result<Figment> {
    let default_path = path.is_none();
    let path = match path {
        None => UserDirs::new()
            .wrap_err("Home directory not found")?
            .home_dir()
            .join(".tmexclude.yaml"),
        Some(path) => path,
    };
    if !path.is_file() {
        return Err(eyre!("Config file not found: {:?}", path).with_suggestion(|| if default_path {
            "please ensure the config file exists, or maybe you want to specify your config manually (--config)?"
        } else {
            "please ensure the config file exists on your given path"
        }));
    }

    let mut figment = Figment::new().merge(FlexiProvider::from(path));
    if dry_run {
        figment = figment.merge(("mode", "dry_run"));
    }

    Ok(figment)
}

pub fn ensure_uds_path(maybe_uds: Option<PathBuf>, cleanup: bool) -> Result<PathBuf> {
    let path = match maybe_uds {
        None => ensure_state_dir()?.join("daemon.sock"),
        Some(path) => path,
    };
    if cleanup {
        fs::remove_file(&path).or_else(|e| {
            if e.kind() == ErrorKind::NotFound {
                Ok(())
            } else {
                Err(e)
            }
        })?;
    }
    Ok(path)
}
