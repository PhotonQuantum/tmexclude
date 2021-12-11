use std::path::PathBuf;

use clap::{AppSettings, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(about, version, author, setting(AppSettings::PropagateVersion))]
pub struct Args {
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
    Daemon,
}
