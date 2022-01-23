use std::path::PathBuf;

use actix_rt::System;
use console::{style, Emoji};
use dialoguer::Confirm;
use eyre::Result;

use tmexclude_lib::config::Config;
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

pub fn scan(config: Config, uds: Option<PathBuf>, interactive: bool, dry_run: bool) -> Result<()> {
    let mut pending_actions = {
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
    report_pending_actions(&pending_actions, config.no_include);
    if config.no_include {
        pending_actions.remove.clear();
    }

    if dry_run || pending_actions.is_empty() {
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
                let guard = DaemonGuard::new(uds).await;
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

fn report_pending_actions(actions: &ExclusionActionBatch, no_include: bool) {
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
        if no_include {
            let plural = if actions.remove.len() == 1 { "" } else { "s" };
            println!(
                "    {}",
                style(format!(
                    "- {} file{} ignored -",
                    actions.remove.len(),
                    plural
                ))
                .dim()
            );
        } else {
            for path in &actions.remove {
                println!("  {}", style(path.display()).dim());
            }
        }
    }
}

struct DaemonGuard {
    client: Option<Client>,
}

impl DaemonGuard {
    async fn new_impl(uds: Option<PathBuf>) -> Option<Client> {
        let uds = ensure_uds_path(uds).ok()?;

        let mut client = Client::connect(&uds).await.ok()?;

        client
            .send(Request::Pause)
            .await
            .ok()
            .filter(|r| r.body.success())
            .map(|_| client)
    }
    pub async fn new(uds: Option<PathBuf>) -> Self {
        Self {
            client: Self::new_impl(uds).await,
        }
    }
    pub async fn release(mut self) {
        if let Some(mut client) = self.client.take() {
            drop(client.send(Request::Restart).await);
        }
    }
}
