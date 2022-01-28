#![allow(clippy::non_ascii_literal, clippy::module_name_repetitions)]

use clap::Parser;
use console::Emoji;
use eyre::Result;

use tmexclude_lib::rpc::Request;

use crate::args::{AgentCommand, Arg, Command, DaemonArgs, ScanArgs};
use crate::client::client;
use crate::common::{collect_config, initialize_loggers};
use crate::daemon::daemon;
use crate::scan::scan;
use crate::utils::ensure_state_dir;

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
        Command::Run(DaemonArgs { uds }) => {
            initialize_loggers()?;

            let uds = uds.clone();
            let config_factory = move || collect_config(args.config.clone());
            daemon(config_factory, uds)
        }
        Command::Scan(ScanArgs {
            dry_run,
            noconfirm,
            uds,
        }) => {
            let config = collect_config(args.config)?;
            scan(config, uds.clone(), !*noconfirm, *dry_run)
        }
        Command::Client(cmd) => {
            let req = Request::from(cmd);
            let args = cmd.args();
            client(req, (&args.uds).clone())
        }
        Command::Agent(AgentCommand::Start) => agent::start(),
        Command::Agent(AgentCommand::Stop) => agent::stop(),
        Command::Agent(AgentCommand::Restart) => agent::restart(),
        #[cfg(debug_assertions)]
        Command::ReadConfig => {
            let config = collect_config(args.config)?;
            println!("{:#?}", config);
            Ok(())
        }
    }
}
