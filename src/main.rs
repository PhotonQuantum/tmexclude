use std::error::Error;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::{fs, io};

use actix::Actor;
use actix_rt::System;
use clap::Parser;
use color_eyre::Section;
use directories::UserDirs;
use eyre::{Result, WrapErr};
use figment::value::Dict;
use figment::{Figment, Provider};

use tmexclude_lib::daemon::Daemon;
use tmexclude_lib::rpc;
use tmexclude_lib::rpc::client::Client;
use tmexclude_lib::rpc::server::start_server;
use tmexclude_lib::rpc::Request;
use tmexclude_lib::utils::TypeEq;

use crate::args::{Arg, ClientCommand, Command, DaemonArgs};
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
        Command::Scan(_) => unimplemented!(),
        Command::Client(cmd) => {
            let req = match cmd {
                ClientCommand::Pause(_) => Request {
                    command: rpc::Command::Pause,
                },
                ClientCommand::Reload(_) => Request {
                    command: rpc::Command::Reload,
                },
                ClientCommand::Restart(_) => Request {
                    command: rpc::Command::Restart,
                },
                ClientCommand::Shutdown(_) => Request {
                    command: rpc::Command::Shutdown,
                },
            };
            let args = cmd.args();
            talk(req, args.uds.as_ref().cloned())
                .wrap_err("Unable to talk to daemon")
                .suggestion(
                    "check if the daemon is running, or whether the given path to socket exists",
                )
        }
    }
}

fn collect_provider(path: Option<PathBuf>, args: &Arg) -> io::Result<Figment> {
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
        if args.dry_run {
            dict.insert("mode".into(), "dry-run".into());
        }
        dict
    });

    Ok(Figment::new().merge(FlexiProvider::from(path)).merge(adhoc))
}

fn ensure_uds_path(maybe_uds: Option<PathBuf>, cleanup: bool) -> Result<PathBuf> {
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

fn talk(req: Request, uds: Option<PathBuf>) -> Result<()> {
    System::new().block_on(async move {
        let mut client = Client::connect(ensure_uds_path(uds, false)?).await?;
        let resp = client.send(req).await?;
        println!("{:#?}", resp);
        Ok(())
    })
}

fn daemon<F, O, E, P>(provider: F, uds: Option<PathBuf>) -> Result<()>
where
    F: Fn() -> O + Unpin + 'static,
    O: TypeEq<Rhs = Result<P, E>>,
    E: 'static + Error + Send + Sync,
    P: Provider,
{
    System::new().block_on(async move {
        let daemon = Daemon::new(provider)?;
        let addr = daemon.start();
        Ok(start_server(ensure_uds_path(uds, true)?, addr).await?)
    })
}
