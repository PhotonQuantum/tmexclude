use std::env;

use std::io::ErrorKind;

use std::path::PathBuf;
use std::str;
use std::{fs, process};

use directories::UserDirs;
use eyre::{eyre, Report, Result};

use tmexclude_lib::errors::SuggestionExt;

const LAUNCH_PLIST: &str = include_str!("../launch.plist");

fn check_brew_managed() -> bool {
    process::Command::new("brew")
        .arg("--prefix")
        .output()
        .ok()
        .and_then(|output| {
            str::from_utf8(&*output.stdout)
                .ok()
                .map(str::trim)
                .map(PathBuf::from)
        })
        .map_or(false, |prefix| prefix.join("bin/tmexclude").exists())
}

fn plist() -> String {
    LAUNCH_PLIST.replace(
        "SELF_PATH",
        env::current_exe()
            .expect("buf to be large enough")
            .to_str()
            .expect("to be valid utf8"),
    )
}

#[allow(clippy::unnecessary_wraps)]
pub fn display() -> Result<()> {
    println!("{}", plist());
    Ok(())
}

pub fn install() -> Result<()> {
    let agent_path = UserDirs::new()
        .ok_or_else(|| eyre!("Home directory not found"))?
        .home_dir()
        .join("Library/LaunchAgents/me.lightquantum.tmexclude.plist");
    if agent_path.exists() {
        return Err(eyre!("Launch Agent is already installed")
            .suggestion("run `tmexclude agent install -u` first if you want to reinstall it."));
    } else if check_brew_managed() {
        return Err(eyre!("tmexclude is managed by homebrew")
            .suggestion("use `brew services` to install the agent"));
    }
    fs::write(agent_path, plist())?;
    Ok(())
}

pub fn uninstall() -> Result<()> {
    let agent_path = UserDirs::new()
        .ok_or_else(|| eyre!("Home directory not found"))?
        .home_dir()
        .join("Library/LaunchAgents/me.lightquantum.tmexclude.plist");
    fs::remove_file(agent_path).map_err(|e| {
        if e.kind() == ErrorKind::NotFound {
            if check_brew_managed() {
                Report::new(e)
                    .wrap_err("Launch Agent is not installed")
                    .suggestion(
                    "tmexclude is managed by homebrew, use `brew services` to uninstall the agent",
                )
            } else {
                Report::new(e).wrap_err("Launch Agent is not installed")
            }
        } else {
            e.into()
        }
    })
}
