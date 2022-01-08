use std::path::PathBuf;

use actix_rt::System;
use console::Emoji;
use dialoguer::Confirm;

use tmexclude_lib::config::{ApplyMode, Config};
use tmexclude_lib::rpc;
use tmexclude_lib::rpc::client::Client;
use tmexclude_lib::rpc::Request;
use tmexclude_lib::tmutil::ExclusionActionBatch;
use tmexclude_lib::walker::walk_recursive;

use crate::common::ensure_uds_path;
use crate::spinner::Spinner;

static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍  ", "");

pub fn scan(config: Config, uds: Option<PathBuf>, interactive: bool) {
    let pending_actions = {
        let _spinner = Spinner::new(format!(
            "{}Scanning filesystem for files to exclude...",
            LOOKING_GLASS
        ));
        walk_recursive(&config.walk.root().expect("No rule found"), config.walk)
    };
    let pending_actions = if config.mode == ApplyMode::DryRun {
        report_pending_actions(&pending_actions);
        pending_actions.filter_by_mode(config.mode)
    } else {
        let filtered_actions = pending_actions.filter_by_mode(config.mode);
        report_pending_actions(&filtered_actions);
        filtered_actions
    };

    if pending_actions.is_empty() {
        println!("No changes to apply.");
    } else {
        let proceed = !interactive
            || Confirm::new()
                .with_prompt("Proceed?")
                .default(false)
                .interact()
                .unwrap_or(false);
        if proceed {
            println!("Applying changes...");
            System::new().block_on(async move {
                let guard = DaemonGuard::new(uds, config.mode).await;
                pending_actions.apply();
                guard.release().await;
            });
            println!("Completed.");
        } else {
            println!("Aborted.");
        }
    }
}

fn report_pending_actions(actions: &ExclusionActionBatch) {
    if !actions.add.is_empty() {
        println!("Files to exclude from backup:");
        for path in &actions.add {
            println!("{}", path.display());
        }
    }
    if !actions.remove.is_empty() {
        println!("Files to include in backup:");
        for path in &actions.remove {
            println!("{}", path.display());
        }
    }
}

struct DaemonGuard {
    client: Option<Client>,
}

impl DaemonGuard {
    const NONE: Self = Self { client: None };
    pub async fn new(uds: Option<PathBuf>, mode: ApplyMode) -> Self {
        if mode == ApplyMode::DryRun {
            return Self::NONE;
        }

        let uds = if let Ok(uds) = ensure_uds_path(uds, false) {
            uds
        } else {
            return Self::NONE;
        };

        println!("Trying to pause daemon...");
        let mut client = if let Ok(client) = Client::connect(&uds).await {
            client
        } else {
            println!("No daemon found.");
            return Self::NONE;
        };

        match client
            .send(Request {
                command: rpc::Command::Pause,
            })
            .await
        {
            Ok(res) if res.success => Self {
                client: Some(client),
            },
            _ => {
                println!("WARN: failed to talk to daemon.");
                Self::NONE
            }
        }
    }
    pub async fn release(mut self) {
        if let Some(mut client) = self.client.take() {
            println!("Trying to restart daemon...");
            match client
                .send(Request {
                    command: rpc::Command::Restart,
                })
                .await
            {
                Ok(res) if res.success => (),
                _ => println!("WARN: failed to talk to daemon."),
            }
        }
    }
}
