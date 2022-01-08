use std::error::Error;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::{fs, io};

use actix::Actor;
use actix_rt::System;
use clap::Parser;
use color_eyre::Section;
use dialoguer::Confirm;
use directories::UserDirs;
use eyre::{Result, WrapErr};
use figment::value::Dict;
use figment::{Figment, Provider};

use tmexclude_lib::config::{ApplyMode, Config};
use tmexclude_lib::daemon::Daemon;
use tmexclude_lib::rpc;
use tmexclude_lib::rpc::client::Client;
use tmexclude_lib::rpc::server::start_server;
use tmexclude_lib::rpc::Request;
use tmexclude_lib::tmutil::ExclusionActionBatch;
use tmexclude_lib::utils::TypeEq;
use tmexclude_lib::walker::walk_recursive;

use crate::args::{Arg, Command, DaemonArgs, ScanArgs};
use crate::utils::{ensure_state_dir, AdhocProvider, FlexiProvider};

mod args;
mod utils;

fn main() -> Result<()> {
    color_eyre::install()?;
    pretty_env_logger::init();

    let args = Arg::parse();

    match &args.command {
        Command::Daemon(DaemonArgs { dry_run, uds }) => {
            let dry_run = *dry_run;
            let uds = uds.clone();
            let provider = move || collect_provider(args.config.clone(), dry_run);
            daemon(provider, uds)
        }
        Command::Scan(ScanArgs {
            dry_run,
            noconfirm,
            uds,
        }) => {
            let config = Config::from(collect_provider(args.config, *dry_run)?)?;
            scan(config, uds.clone(), !*noconfirm);
            Ok(())
        }
        Command::Client(cmd) => {
            let req = Request {
                command: cmd.into(),
            };
            let args = cmd.args();
            talk(req, (&args.uds).clone())
                .wrap_err("Unable to talk to daemon")
                .suggestion(
                    "check if the daemon is running, or whether the given path to socket exists",
                )
        }
    }
}

fn collect_provider(path: Option<PathBuf>, dry_run: bool) -> io::Result<Figment> {
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

struct DaemonGuard {
    client: Option<Client>,
}

impl DaemonGuard {
    const NONE: Self = Self { client: None };
    pub async fn new(uds: Option<PathBuf>, mode: ApplyMode) -> Self {
        if mode == ApplyMode::DryRun {
            return Self::NONE;
        }

        let uds = if let Ok(uds) = ensure_uds_path(uds, false) {
            uds
        } else {
            return Self::NONE;
        };

        println!("Trying to pause daemon...");
        let mut client = if let Ok(client) = Client::connect(&uds).await {
            client
        } else {
            println!("No daemon found.");
            return Self::NONE;
        };

        match client
            .send(Request {
                command: rpc::Command::Pause,
            })
            .await
        {
            Ok(res) if res.success => Self {
                client: Some(client),
            },
            _ => {
                println!("WARN: failed to talk to daemon.");
                Self::NONE
            }
        }
    }
    pub async fn release(mut self) {
        if let Some(mut client) = self.client.take() {
            println!("Trying to restart daemon...");
            match client
                .send(Request {
                    command: rpc::Command::Restart,
                })
                .await
            {
                Ok(res) if res.success => (),
                _ => println!("WARN: failed to talk to daemon."),
            }
        }
    }
}

fn scan(config: Config, uds: Option<PathBuf>, interactive: bool) {
    println!("Scanning filesystem for files to exclude...");
    let pending_actions = walk_recursive(&config.walk.root().expect("No rule found"), config.walk);
    let pending_actions = if config.mode == ApplyMode::DryRun {
        report_pending_actions(&pending_actions);
        pending_actions.filter_by_mode(config.mode)
    } else {
        let filtered_actions = pending_actions.filter_by_mode(config.mode);
        report_pending_actions(&filtered_actions);
        filtered_actions
    };

    if pending_actions.is_empty() {
        println!("No changes to apply.");
    } else {
        let proceed = !interactive
            || Confirm::new()
                .with_prompt("Proceed?")
                .default(false)
                .interact()
                .unwrap_or(false);
        if proceed {
            println!("Applying changes...");
            System::new().block_on(async move {
                let guard = DaemonGuard::new(uds, config.mode).await;
                pending_actions.apply();
                guard.release().await;
            });
            println!("Completed.");
        } else {
            println!("Aborted.");
        }
    }
}

fn report_pending_actions(actions: &ExclusionActionBatch) {
    if !actions.add.is_empty() {
        println!("Files to exclude from backup:");
        for path in &actions.add {
            println!("{}", path.display());
        }
    }
    if !actions.remove.is_empty() {
        println!("Files to include in backup:");
        for path in &actions.remove {
            println!("{}", path.display());
        }
    }
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
