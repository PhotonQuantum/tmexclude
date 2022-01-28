use std::fs;
use std::fs::File;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use directories::BaseDirs;
use eyre::{eyre, Report, Result, WrapErr};
use fs2::FileExt;
use log::Level;
use multi_log::MultiLogger;
use oslog::OsLogger;

use tmexclude_lib::config::Config;
use tmexclude_lib::errors::SuggestionExt;
use tmexclude_lib::rpc::Request;

use crate::args::{ClientArgs, ClientCommand};
use crate::ensure_state_dir;

const DEFAULT_CONFIG: &str = include_str!("../../config.example.yaml");

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

pub fn collect_config(path: Option<PathBuf>) -> Result<Config> {
    let path = path
        .map(|p| p.canonicalize().unwrap_or_else(|_| p.clone()))
        .ok_or(())
        .or_else::<Report, _>(|_| {
            let config_dir = BaseDirs::new()
                .ok_or_else(|| eyre!("Home directory not found"))?
                .home_dir()
                .join(".config");
            fs::create_dir_all(&config_dir).wrap_err("Failed to create config directory")?;

            let path = config_dir.join("tmexclude.yaml");
            if !path.exists() {
                fs::write(&path, DEFAULT_CONFIG).wrap_err("Failed to write default config")?;
            }

            Ok(path)
        })?;

    let body = fs::read_to_string(&path)
        .wrap_err(format!("Config file not found: {:?}", path))
        .suggestion("please ensure the config file exists on your given path")?;
    Ok(
        if path.extension().unwrap_or_default().eq(Path::new("toml")) {
            let de = &mut toml::Deserializer::new(body.as_str());
            Config::from(de)?
        } else {
            let de = serde_yaml::Deserializer::from_str(body.as_str());
            Config::from(de)?
        },
    )
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

impl ClientCommand {
    pub const fn args(&self) -> &ClientArgs {
        match self {
            ClientCommand::Pause(a)
            | ClientCommand::Reload(a)
            | ClientCommand::Restart(a)
            | ClientCommand::Shutdown(a) => a,
        }
    }
}

impl From<&ClientCommand> for Request {
    fn from(cmd: &ClientCommand) -> Self {
        match cmd {
            ClientCommand::Pause(_) => Self::Pause,
            ClientCommand::Reload(_) => Self::Reload,
            ClientCommand::Restart(_) => Self::Restart,
            ClientCommand::Shutdown(_) => Self::Shutdown,
        }
    }
}
