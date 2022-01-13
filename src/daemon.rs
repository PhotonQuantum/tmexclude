use std::path::PathBuf;

use actix::Actor;
use actix_rt::System;
use eyre::Result;

use tmexclude_lib::daemon::{Daemon, ProviderFactory};
use tmexclude_lib::rpc::server::start_server;

use crate::common::ensure_uds_path;

pub fn daemon<F>(provider: F, uds: Option<PathBuf>) -> Result<()>
where
    F: ProviderFactory,
{
    System::new().block_on(async move {
        let daemon = Daemon::new(provider)?;
        let addr = daemon.start();
        Ok(start_server(ensure_uds_path(uds, true)?, addr).await?)
    })
}
