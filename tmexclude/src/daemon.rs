use std::path::PathBuf;

use actix::Actor;
use actix_rt::System;
use eyre::Result;

use tmexclude_lib::daemon::{Daemon, ProviderFactory};
use tmexclude_lib::rpc::server::start_server;

use crate::common::acquire_uds_guard;

pub fn daemon<F>(provider: F, uds: Option<PathBuf>) -> Result<()>
where
    F: ProviderFactory,
{
    System::new().block_on(async move {
        let daemon = Daemon::new(provider)?;
        let addr = daemon.start();
        let uds = acquire_uds_guard(uds)?;
        Ok(start_server(uds.path(), addr).await?)
    })
}
