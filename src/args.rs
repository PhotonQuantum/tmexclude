use std::path::PathBuf;

use clap::{AppSettings, Args, Parser, Subcommand};
use tmexclude_lib::daemon::Daemon;

#[derive(Debug, Parser)]
#[clap(about, version, author, setting(AppSettings::PropagateVersion))]
pub struct Arg {
    #[clap(subcommand)]
    pub command: Command,
    /// Specify the config file. Accepted formats are Json and Toml. Defaults to .tmexclude.yaml in home directory.
    #[clap(short, long)]
    pub config: Option<PathBuf>,
    /// When in daemon mode, bind to this UNIX domain socket. Otherwise, try to connect to this socket.
    #[clap(short, long)]
    pub uds: Option<PathBuf>,
    /// Don't touch the system. This flag overrides the config file.
    #[clap(short, long)]
    pub dry_run: bool,
}

#[derive(Debug, Subcommand)]
#[clap(author)]
pub enum Command {
    /// Start the daemon to watch the filesystem continuously.
    Daemon(DaemonArgs),
    /// Perform a full scan and set the exclusion flag to your files.
    Scan(ScanArgs),
}

#[derive(Debug, Args)]
pub struct DaemonArgs {
    /// Bind to this Unix domain socket.
    #[clap(short, long)]
    pub uds: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct ScanArgs {
    /// Don't touch the system. This flag overrides the config file.
    #[clap(short, long)]
    pub dry_run: bool,
    /// Connect to the daemon through the given Unix domain socket.
    #[clap(short, long)]
    pub uds: Option<PathBuf>,
}
