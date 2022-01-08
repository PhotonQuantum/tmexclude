use std::io::ErrorKind;
use std::path::PathBuf;
use std::{fs, io};

use directories::UserDirs;
use eyre::Result;
use figment::value::Dict;
use figment::Figment;

use crate::{ensure_state_dir, AdhocProvider, FlexiProvider};

pub fn collect_provider(path: Option<PathBuf>, dry_run: bool) -> io::Result<Figment> {
    let path = match path {
        None => UserDirs::new()
            .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "Home directory not found"))?
            .home_dir()
            .join(".tmexclude.yaml"),
        Some(path) => path,
    };
    if !path.is_file() {
        return Err(io::Error::new(
            ErrorKind::NotFound,
            format!("Config file not found: {:?}", path),
        ));
    }

    let adhoc = AdhocProvider({
        let mut dict = Dict::new();
        if dry_run {
            dict.insert("mode".into(), "dry-run".into());
        }
        dict
    });

    Ok(Figment::new().merge(FlexiProvider::from(path)).merge(adhoc))
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
