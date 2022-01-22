use std::path::PathBuf;
use std::time::Duration;

use actix_rt::System;
use console::{style, Emoji};
use eyre::{Result, WrapErr};
use humantime::format_duration;

use tmexclude_lib::errors::SuggestionExt;
use tmexclude_lib::rpc::client::Client;
use tmexclude_lib::rpc::{Body, Request};

use crate::common::ensure_uds_path;

static SPARKLE: Emoji<'_, '_> = Emoji("✨  ", ":-)");

pub fn client(req: Request, uds: Option<PathBuf>) -> Result<()> {
    async fn connect(uds: Option<PathBuf>) -> Result<Client> {
        Ok(Client::connect(ensure_uds_path(uds)?).await?)
    }
    System::new().block_on(async move {
        let mut client = connect(uds)
            .await
            .wrap_err("Unable to talk to daemon")
            .suggestion(
                "check if the daemon is running, or whether the given path to socket exists",
            )?;
        let resp = client
            .send(req)
            .await
            .wrap_err("Error occur when communicating with daemon")
            .suggestion("check whether daemon and client have the same version")?;
        match resp.body {
            Body::Empty => println!(
                "{}",
                style(format!(
                    "{}Done in {}.",
                    SPARKLE,
                    format_duration(round_duration(resp.elapsed))
                ))
                .green()
            ),
            Body::Error(e) => {
                return Err(e.into_report().wrap_err("Failed to execute remote command"));
            }
        }
        Ok(())
    })
}

fn round_duration(dur: Duration) -> Duration {
    Duration::from_secs(dur.as_secs()) + Duration::from_millis(u64::from(dur.subsec_millis()))
}
