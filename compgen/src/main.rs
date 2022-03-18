#![allow(clippy::module_name_repetitions)]

use std::path::PathBuf;

use clap::{IntoApp, Parser};
use clap_complete::Shell;

mod args {
    include!("../../tmexclude/src/args.rs");
}

/// Completion generator for tmexclude.
#[derive(Debug, Parser)]
struct Args {
    /// Output directory path.
    output: PathBuf,
    /// Target shell.
    #[clap(short, long)]
    shell: Shell,
}

fn main() {
    let gen_args = Args::parse();
    let mut app = args::Arg::command();
    std::fs::create_dir_all(&gen_args.output).expect("directory can't be created");
    clap_complete::generate_to(gen_args.shell, &mut app, "tmexclude", gen_args.output)
        .expect("completion file can't be generated");
}
