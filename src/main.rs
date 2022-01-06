use actix::Actor;
use actix_rt::System;
use std::error::Error;
use std::path::PathBuf;

use clap::Parser;
use directories::UserDirs;
use eyre::{bail, eyre, Result};
use figment::value::Dict;
use figment::{Figment, Provider};

use tmexclude_lib::config::Config;
use tmexclude_lib::daemon::Daemon;
use tmexclude_lib::utils::TypeEq;

use crate::args::{Arg, Command, DaemonArgs};
use crate::utils::{ensure_state_dir, AdhocProvider, FlexiProvider};

mod args;
mod utils;

fn main() -> Result<()> {
    color_eyre::install()?;
    pretty_env_logger::init();

    let args = Arg::parse();

    match &args.command {
        Command::Daemon(DaemonArgs { uds }) => {
            let uds = uds.clone();
            let provider = move || collect_provider(args.config.clone(), &args);
            daemon(provider, uds)
        }
        _ => unimplemented!(),
    }
}
use std::io;
use std::io::ErrorKind;

fn collect_provider(path: Option<PathBuf>, args: &Arg) -> io::Result<Figment> {
    let path = match path {
        None => UserDirs::new()
            .ok_or(io::Error::new(
                ErrorKind::NotFound,
                "Home directory not found",
            ))?
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
        if args.dry_run {
            dict.insert("mode".into(), "dry-run".into());
        }
        dict
    });

    Ok(Figment::new().merge(FlexiProvider::from(path)).merge(adhoc))
}

fn daemon<F, O, E, P>(provider: F, addr: Option<PathBuf>) -> Result<()>
where
    F: Fn() -> O + Unpin + 'static,
    O: TypeEq<Rhs = Result<P, E>>,
    E: 'static + Error + Send + Sync,
    P: Provider,
{
    let path = match addr {
        None => ensure_state_dir()?.join("daemon.sock"),
        Some(path) => path,
    };
    System::new().block_on(async move {
        let daemon = Daemon::new(provider)?;
        let addr = daemon.start();
        todo!()
    })
}
