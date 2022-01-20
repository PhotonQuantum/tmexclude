use std::fs;
use std::fs::File;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use directories::UserDirs;
use eyre::{eyre, Result, WrapErr};
use figment::Figment;
use fs2::FileExt;
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
            .ok_or_else(|| eyre!("Home directory not found"))?
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

pub struct UdsGuard {
    uds_path: PathBuf,
    lock_path: PathBuf,
    lock_file: File,
}

impl UdsGuard {
    fn new(uds_path: PathBuf, lock_path: PathBuf) -> Result<Self> {
        fs::remove_file(&uds_path).or_else(|e| {
            if e.kind() == ErrorKind::NotFound {
                Ok(())
            } else {
                Err(e)
            }
        })?;
        let lock_file = File::create(&lock_path)?;
        lock_file
            .try_lock_exclusive()
            .wrap_err("Unable to obtain exclusive lock to given socket")
            .suggestion("check whether there's another instance running")?;
        Ok(Self {
            uds_path,
            lock_path,
            lock_file,
        })
    }
    pub fn path(&self) -> &Path {
        self.uds_path.as_path()
    }
}

impl Drop for UdsGuard {
    fn drop(&mut self) {
        drop(fs::remove_file(&self.uds_path));
        drop(self.lock_file.unlock());
        drop(fs::remove_file(&self.lock_path));
    }
}

pub fn acquire_uds_guard(maybe_uds: Option<PathBuf>) -> Result<UdsGuard> {
    let uds_path = ensure_uds_path(maybe_uds)?;
    let lock_path = uds_path.with_extension("lock");
    UdsGuard::new(uds_path, lock_path)
}

pub fn ensure_uds_path(maybe_uds: Option<PathBuf>) -> Result<PathBuf> {
    let path = match maybe_uds {
        None => ensure_state_dir()?.join("daemon.sock"),
        Some(path) => path,
    };
    if !path.parent().map_or(true, Path::exists) {
        return Err(eyre!(
            "Parent directory of socket path not found: {:?}",
            path.parent().expect("has parent")
        )
        .suggestion("please ensure it exists"));
    }
    Ok(path)
}
