use std::path::PathBuf;

use actix_rt::System;
use console::{style, Emoji};
use eyre::{Result, WrapErr};

use tmexclude_lib::errors::SuggestionExt;
use tmexclude_lib::rpc::client::Client;
use tmexclude_lib::rpc::{Request, Response};

use crate::common::ensure_uds_path;

static PARTY_POPPER: Emoji<'_, '_> = Emoji("🎉  ", "");

pub fn client(req: Request, uds: Option<PathBuf>) -> Result<()> {
    async fn send(req: Request, uds: Option<PathBuf>) -> Result<Response> {
        let mut client = Client::connect(ensure_uds_path(uds, false)?).await?;
        Ok(client.send(req).await?)
    }
    System::new().block_on(async move {
        let resp = send(req, uds)
            .await
            .wrap_err("Unable to talk to daemon")
            .suggestion(
                "check if the daemon is running, or whether the given path to socket exists",
            )?;
        match resp {
            Response::Empty => println!(
                "{}",
                style(format!("{}Operation finished successfully.", PARTY_POPPER)).green()
            ),
            Response::Error(e) => {
                return Err(e.into_report().wrap_err("Failed to execute remote command"));
            }
        }
        Ok(())
    })
}
