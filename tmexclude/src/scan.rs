use std::path::PathBuf;

use actix_rt::System;
use console::{style, Emoji};
use dialoguer::Confirm;
use eyre::Result;

use tmexclude_lib::config::{ApplyMode, Config};
use tmexclude_lib::errors::SuggestionExt;
use tmexclude_lib::rpc::client::Client;
use tmexclude_lib::rpc::Request;
use tmexclude_lib::tmutil::ExclusionActionBatch;
use tmexclude_lib::walker::walk_recursive;

use crate::common::ensure_uds_path;
use crate::spinner::Spinner;

static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍  ", "");
static SPARKLE: Emoji<'_, '_> = Emoji("✨  ", ":-)");
static HAMMER: Emoji<'_, '_> = Emoji("🔨  ", "");

pub fn scan(config: Config, uds: Option<PathBuf>, interactive: bool) -> Result<()> {
    let pending_actions = {
        let _spinner = Spinner::new(format!(
            "{}Scanning filesystem for files to exclude...",
            LOOKING_GLASS
        ));
        walk_recursive(
            &config
                .walk
                .root()
                .suggestion("try to add some directories to your config")?,
            config.walk,
        )
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
        println!("\n{}Done. No changes to apply.", SPARKLE);
    } else {
        let proceed = !interactive
            || Confirm::new()
                .with_prompt("Proceed?")
                .default(false)
                .interact()
                .unwrap_or(false);
        if proceed {
            System::new().block_on(async move {
                let _spinner = Spinner::new(format!("{}Applying changes...", HAMMER));
                let guard = DaemonGuard::new(uds, config.mode).await;
                pending_actions.apply();
                guard.release().await;
            });
            println!("\n{}Done.", SPARKLE);
        } else {
            println!("\n{}", style("Aborted.").red());
        }
    }

    Ok(())
}

fn report_pending_actions(actions: &ExclusionActionBatch) {
    println!(
        "{}",
        style(format!(
            "Scan complete. There are {} action(s) to be reviewed.",
            actions.count()
        ))
        .green()
    );
    if !actions.add.is_empty() {
        println!("Files to exclude from backup:");
        for path in &actions.add {
            println!("  {}", style(path.display()).dim());
        }
    }
    if !actions.remove.is_empty() {
        println!("Files to include in backup:");
        for path in &actions.remove {
            println!("  {}", style(path.display()).dim());
        }
    }
}

struct DaemonGuard {
    client: Option<Client>,
}

impl DaemonGuard {
    async fn new_impl(uds: Option<PathBuf>, mode: ApplyMode) -> Option<Client> {
        if mode == ApplyMode::DryRun {
            return None;
        }

        let uds = ensure_uds_path(uds).ok()?;

        let mut client = Client::connect(&uds).await.ok()?;

        client
            .send(Request::Pause)
            .await
            .ok()
            .filter(|r| r.body.success())
            .map(|_| client)
    }
    pub async fn new(uds: Option<PathBuf>, mode: ApplyMode) -> Self {
        Self {
            client: Self::new_impl(uds, mode).await,
        }
    }
    pub async fn release(mut self) {
        if let Some(mut client) = self.client.take() {
            drop(client.send(Request::Restart).await);
        }
    }
}
