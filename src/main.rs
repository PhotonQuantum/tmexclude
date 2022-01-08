#![allow(clippy::non_ascii_literal)]

use clap::Parser;
use color_eyre::Section;
use eyre::{Result, WrapErr};

use tmexclude_lib::config::Config;
use tmexclude_lib::rpc::Request;

use crate::args::{Arg, Command, DaemonArgs, ScanArgs};
use crate::client::client;
use crate::common::collect_provider;
use crate::daemon::daemon;
use crate::scan::scan;
use crate::utils::{ensure_state_dir, AdhocProvider, FlexiProvider};

mod args;
mod client;
mod common;
mod daemon;
mod scan;
mod spinner;
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
            client(req, (&args.uds).clone())
                .wrap_err("Unable to talk to daemon")
                .suggestion(
                    "check if the daemon is running, or whether the given path to socket exists",
                )
        }
    }
}
