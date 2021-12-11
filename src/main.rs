use std::path::PathBuf;

use actix_rt::System;
use clap::Parser;
use directories::UserDirs;
use eyre::{bail, eyre, Result};
use figment::value::Dict;
use figment::Figment;

use tmexclude_lib::config::Config;

use crate::args::{Args, Command};
use crate::daemon::app;
use crate::utils::{ensure_state_dir, AdhocProvider, FlexiProvider};

mod args;
mod daemon;
mod utils;

fn main() -> Result<()> {
    color_eyre::install()?;
    pretty_env_logger::init();

    let args = Args::parse();
    let config = collect_config(args.config.clone(), &args)?;

    match args.command {
        Command::Daemon => daemon(config, args.uds),
    }
}

fn collect_config(path: Option<PathBuf>, args: &Args) -> Result<Config> {
    let path = match path {
        None => UserDirs::new()
            .ok_or_else(|| eyre!("Home directory not found"))?
            .home_dir()
            .join(".tmexclude.yaml"),
        Some(path) => path,
    };
    if !path.is_file() {
        bail!("Config file not found: {:?}", path);
    }

    let adhoc = AdhocProvider({
        let mut dict = Dict::new();
        if args.dry_run {
            dict.insert("mode".into(), "dry-run".into());
        }
        dict
    });

    Ok(Config::from(move || {
        Figment::new()
            .merge(FlexiProvider::from(path.clone()))
            .merge(adhoc.clone())
    })?)
}

fn daemon(config: Config, addr: Option<PathBuf>) -> Result<()> {
    let path = match addr {
        None => ensure_state_dir()?.join("daemon.sock"),
        Some(path) => path,
    };
    System::new().block_on(app(config, path))
}
