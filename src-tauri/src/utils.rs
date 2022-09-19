use std::fs;
use std::path::{Path, PathBuf};
use directories::BaseDirs;

use eyre::{eyre, Report, Result, WrapErr};
use tmexclude_lib::Config;

const DEFAULT_CONFIG: &str = include_str!("../../config.example.yaml");

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
        .wrap_err(format!("Config file not found: {:?}", path))?;
        // .suggestion("please ensure the config file exists on your given path")?;
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
