use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use eyre::{eyre, Result, WrapErr};
use figment::providers::{Data, Format, Toml, Yaml};
use figment::value::{Dict, Map};
use figment::{Error, Metadata, Profile, Provider};

pub enum FlexiProvider {
    Yaml(Data<Yaml>),
    Toml(Data<Toml>),
}

impl<P: AsRef<Path>> From<P> for FlexiProvider {
    fn from(path: P) -> Self {
        let path = path.as_ref();
        if path.extension().unwrap_or_default().eq(Path::new("toml")) {
            Self::Toml(Toml::file(path))
        } else {
            Self::Yaml(Yaml::file(path))
        }
    }
}

impl Provider for FlexiProvider {
    fn metadata(&self) -> Metadata {
        match self {
            Self::Yaml(p) => p.metadata(),
            Self::Toml(p) => p.metadata(),
        }
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        match self {
            Self::Yaml(p) => p.data(),
            Self::Toml(p) => p.data(),
        }
    }
}

pub fn ensure_state_dir() -> Result<PathBuf> {
    let state_dir = ProjectDirs::from("me", "lightquantum", "tmexclude")
        .ok_or_else(|| eyre!("Home directory not found"))?
        .data_local_dir()
        .to_path_buf();
    std::fs::create_dir_all(&state_dir).wrap_err("Failed to create state directory")?;
    Ok(state_dir)
}
