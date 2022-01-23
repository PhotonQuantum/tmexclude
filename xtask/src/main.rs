use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use xshell::{cmd, cp, mkdir_p, rm_rf, write_file};

const FORMULA_TEMPLATE: &str = include_str!("../../formula.rb");

macro_rules! p {
    ($path: literal) => {
        Path::new($path)
    };
}

#[derive(Debug, Parser)]
struct Args {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Build the binary.
    Build {
        /// Architecture
        arch: String,
    },
    /// Build binaries for all supported architectures.
    BuildAll,
    /// Generate completion files.
    Compgen {
        /// Output directory.
        #[clap(default_value = "./completion")]
        output: PathBuf,
    },
    /// Build universal binary.
    Lipo {
        /// Output path.
        #[clap(default_value = "./tmexclude")]
        output: PathBuf,
    },
    /// Build files for final distribution and output them to `dist` directory.
    Dist,
    /// Pack files to be distributed and generate formula to `release` directory.
    Release,
    /// Clean build artifacts
    Clean {
        /// Ignore target directory.
        #[clap(long)]
        keep_target: bool,
    },
}

fn main() {
    std::env::set_current_dir(env!("CARGO_WORKSPACE_DIR")).unwrap();
    let args = Args::parse();
    match args.cmd {
        Command::Build { arch } => build(arch),
        Command::BuildAll => build_all(),
        Command::Compgen { output } => compgen(output),
        Command::Lipo { output } => lipo(output),
        Command::Dist => dist(),
        Command::Release => release(),
        Command::Clean { keep_target } => clean(!keep_target),
    }
}

fn build(arch: impl AsRef<OsStr>) {
    cmd!("cargo build --package tmexclude --release --target={arch}-apple-darwin")
        .run()
        .unwrap();
}

fn build_all() {
    build("aarch64");
    build("x86_64");
}

fn lipo(output: impl AsRef<Path>) {
    let output = output.as_ref();
    build_all();
    mkdir_p(output.parent().unwrap()).unwrap();
    cmd!("lipo -create -output {output} target/aarch64-apple-darwin/release/tmexclude target/x86_64-apple-darwin/release/tmexclude").run().unwrap();
}

fn compgen(output: impl AsRef<Path>) {
    let output = output.as_ref();
    cmd!("cargo compgen -s bash {output}").run().unwrap();
    cmd!("cargo compgen -s zsh {output}").run().unwrap();
    cmd!("cargo compgen -s fish {output}").run().unwrap();
}

fn dist() {
    compgen(p!("./dist/completion"));
    lipo(p!("./dist/tmexclude"));
    cp(p!("launch.plist"), p!("./dist/launch.plist")).unwrap();
}

fn release() {
    dist();
    mkdir_p("./release").unwrap();

    let tag = tag();
    let tar_file = format!("tmexclude-{tag}.tar.gz");

    cmd!("tar czvf ./release/{tar_file} --strip=2 ./dist")
        .run()
        .unwrap();
    let checksum = cmd!("shasum -a 256 ./release/{tar_file}")
        .read()
        .unwrap()
        .split_once(" ")
        .unwrap()
        .0
        .to_string();

    let formula = gen_formula(tag.as_str(), checksum.as_str());
    write_file("./release/tmexclude.rb", formula).unwrap();
}

fn clean(target: bool) {
    rm_rf("./dist").unwrap();
    rm_rf("./release").unwrap();
    if target {
        cmd!("cargo clean").run().unwrap();
    }
}

fn gen_formula(tag: &str, sha256: &str) -> String {
    FORMULA_TEMPLATE
        .replace("VERSION", tag)
        .replace("SHA256", sha256)
}

fn tag() -> String {
    cmd!("git describe --tags --abbrev=0")
        .read()
        .unwrap()
        .trim_start_matches('v')
        .to_string()
}
