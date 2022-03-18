use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(about, version, author)]
#[clap(propagate_version = true)]
pub struct Arg {
    #[clap(subcommand)]
    pub command: Command,
    /// Specify the config file. Accepted formats are Json and Toml. Defaults to .tmexclude.yaml in home directory.
    #[clap(short, long)]
    pub config: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
#[clap(author)]
pub enum Command {
    /// Manage Launch Agent.
    #[clap(subcommand)]
    Agent(AgentCommand),
    /// Run the daemon to watch the filesystem continuously.
    Run(DaemonArgs),
    /// Perform a full scan and set the exclusion flag to your files.
    Scan(ScanArgs),
    #[clap(flatten)]
    Client(ClientCommand),
    #[cfg(debug_assertions)]
    ReadConfig,
}

#[derive(Debug, Subcommand)]
pub enum AgentCommand {
    /// Start the agent immediately and register it to launch at login.
    Start,
    /// Stop the agent immediately and unregister it from launching at login.
    Stop,
    /// Stop (if necessary) and start the agent immediately and register
    /// it to launch at login.
    Restart,
}

#[derive(Debug, Subcommand)]
pub enum ClientCommand {
    /// Pause daemon.
    Pause(ClientArgs),
    /// Reload config and restart daemon.
    Reload(ClientArgs),
    /// Restart daemon. This method doesn't reload config.
    Restart(ClientArgs),
    /// Shutdown daemon.
    Shutdown(ClientArgs),
}

#[derive(Debug, Args)]
pub struct DaemonArgs {
    /// Bind to this Unix domain socket.
    #[clap(short, long)]
    pub uds: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct ClientArgs {
    /// Connect through this Unix domain socket.
    #[clap(short, long)]
    pub uds: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct ScanArgs {
    /// Don't touch the system.
    #[clap(short, long)]
    pub dry_run: bool,
    /// Bypass any and all confirm messages.
    #[clap(long)]
    pub noconfirm: bool,
    /// Connect to the daemon through the given Unix domain socket.
    #[clap(short, long)]
    pub uds: Option<PathBuf>,
}
