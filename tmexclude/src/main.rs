#![allow(clippy::non_ascii_literal, clippy::module_name_repetitions)]

use std::panic;
use std::panic::PanicInfo;

use backtrace::Backtrace;
use clap::Parser;
use console::Emoji;
use eyre::Result;
use log::error;
use once_cell::sync::OnceCell;

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
mod utils;

static EXCLAIMING: Emoji<'_, '_> = Emoji("❗️  ", "");
static OLD_HOOK: OnceCell<Box<dyn Fn(&PanicInfo<'_>) + 'static + Sync + Send>> = OnceCell::new();

fn main() {
    if let Err(e) = run() {
        eprintln!("{}{:?}", EXCLAIMING, e);
        std::process::exit(1);
    }
}

fn panic_log(info: &PanicInfo<'_>) {
    let bt = Backtrace::new();
    error!("!!! PANIC\n{}\nBacktrace:\n{:#?}", info, bt);
    if let Some(hook) = OLD_HOOK.get() {
        hook(info);
    }
}

fn register_panic_report() {
    let last_hook = panic::take_hook();
    OLD_HOOK
        .set(last_hook)
        .ok()
        .expect("Unable to record old panic hook");
    panic::set_hook(Box::new(panic_log));
}

fn run() -> Result<()> {
    template_eyre::Hook::new(include_str!("error.hbs"))?.install()?;
    register_panic_report();

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
