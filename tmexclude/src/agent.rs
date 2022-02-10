use std::env;
use std::ffi::OsStr;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::str;
use std::{fs, process};

use directories::UserDirs;
use eyre::{bail, eyre, Result};

use crate::consts::LABEL;
use crate::utils::spinner;

const LAUNCH_PLIST: &str = include_str!("../../launch.plist");

fn brew(op: &str) -> Result<()> {
    let output = process::Command::new("brew")
        .arg("services")
        .arg(op)
        .arg("tmexclude")
        .output()?;
    if !output.status.success() {
        let stderr = convert_stderr(&*output.stderr, "Homebrew failed");
        bail!(stderr.to_string())
    }
    Ok(())
}

fn launchctl(op: &str, plist: impl AsRef<OsStr>) -> Result<()> {
    let output = process::Command::new("launchctl")
        .arg(op)
        .arg(format!("gui/{}", unsafe { libc::getuid() }))
        .arg(plist)
        .output()?;
    let stderr = convert_stderr(&*output.stderr, "Launchctl failed");
    if !output.status.success() || stderr.contains("fail") || stderr.contains("error") {
        bail!(stderr.to_string());
    }
    Ok(())
}

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

// clion false positive
// noinspection RsTypeCheck
fn convert_stderr<'a>(s: &'a [u8], default: &'a str) -> &'a str {
    str::from_utf8(s)
        .map(str::trim)
        .ok()
        .unwrap_or(default)
        .trim()
}

fn plist(label: &str) -> String {
    LAUNCH_PLIST
        .replace(
            "SELF_PATH",
            env::current_exe()
                .expect("buf to be large enough")
                .to_str()
                .expect("to be valid utf8"),
        )
        .replace("LABEL", label)
}

fn agent_path() -> Result<PathBuf> {
    Ok(UserDirs::new()
        .ok_or_else(|| eyre!("Home directory not found"))?
        .home_dir()
        .join("Library/LaunchAgents/me.lightquantum.tmexclude.plist"))
}

pub fn start() -> Result<()> {
    let _spinner = spinner("Starting tmexclude...");
    start_impl()
}

pub fn stop() -> Result<()> {
    let _spinner = spinner("Stopping tmexclude...");
    stop_impl()
}

pub fn restart() -> Result<()> {
    let _spinner = spinner("Restarting tmexclude...");
    stop_impl().and_then(|_| start_impl())
}

fn start_impl() -> Result<()> {
    if check_brew_managed() {
        brew("start")
    } else {
        let agent_path = agent_path()?;
        if agent_path.exists() {
            return Ok(());
        }

        fs::write(&agent_path, plist(LABEL))?;
        launchctl("bootstrap", &agent_path)
    }
}

fn stop_impl() -> Result<()> {
    if check_brew_managed() {
        brew("stop")
    } else {
        let agent_path = agent_path()?;
        // We delay the error report until plist file is removed.
        let launchctl_result = launchctl("bootout", &agent_path);
        if let Err(e) = fs::remove_file(&agent_path) {
            if e.kind() == ErrorKind::NotFound {
                return Ok(()); // We don't report any error if there's no service to stop.
            }
            bail!(e);
        }
        // Now that we are sure that the plist existed, the bootout operation should have succeeded.
        // If it wasn't, we report an error.
        launchctl_result
    }
}
