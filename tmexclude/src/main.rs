#![allow(clippy::non_ascii_literal, clippy::module_name_repetitions)]

use clap::Parser;
use console::Emoji;
use eyre::Result;

use tmexclude_lib::config::Config;
use tmexclude_lib::rpc::Request;

use crate::args::{AgentCommand, Arg, Command, DaemonArgs, ScanArgs};
use crate::client::client;
use crate::common::{collect_provider, initialize_loggers};
use crate::daemon::daemon;
use crate::scan::scan;
use crate::utils::{ensure_state_dir, FlexiProvider};

mod agent;
mod args;
mod client;
mod common;
mod consts;
mod daemon;
mod scan;
mod spinner;
mod utils;

static EXCLAIMING: Emoji<'_, '_> = Emoji("❗️  ", "");

fn main() {
    if let Err(e) = run() {
        eprintln!("{}{:?}", EXCLAIMING, e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    template_eyre::Hook::new(include_str!("error.hbs"))?.install()?;

    let args = Arg::parse();

    match &args.command {
        Command::Run(DaemonArgs { dry_run, uds }) => {
            initialize_loggers()?;

            let dry_run = *dry_run;
            let uds = uds.clone();
            let config_path = args.config.as_ref().and_then(|p| p.canonicalize().ok());
            let provider = move || collect_provider(config_path.clone(), dry_run);
            daemon(provider, uds)
        }
        Command::Scan(ScanArgs {
            dry_run,
            noconfirm,
            uds,
        }) => {
            let config_path = args.config.as_ref().and_then(|p| p.canonicalize().ok());
            let config = Config::from(collect_provider(config_path, *dry_run)?)?;
            scan(config, uds.clone(), !*noconfirm)
        }
        Command::Client(cmd) => {
            let req = Request::from(cmd);
            let args = cmd.args();
            client(req, (&args.uds).clone())
        }
        Command::Agent(AgentCommand::Start) => agent::start(),
        Command::Agent(AgentCommand::Stop) => agent::stop(),
        Command::Agent(AgentCommand::Restart) => agent::restart(),
    }
}
