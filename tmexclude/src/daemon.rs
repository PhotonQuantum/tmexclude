use std::path::PathBuf;

use actix::Actor;
use actix_rt::System;
use eyre::Result;

use tmexclude_lib::config::ConfigFactory;
use tmexclude_lib::daemon::Daemon;
use tmexclude_lib::rpc::server::start_server;

use crate::common::acquire_uds_guard;

pub fn daemon<F>(config_factory: F, uds: Option<PathBuf>) -> Result<()>
where
    F: ConfigFactory,
{
    System::new().block_on(async move {
        let daemon = Daemon::new(config_factory)?;
        let addr = daemon.start();
        let uds = acquire_uds_guard(uds)?;
        Ok(start_server(uds.path(), addr).await?)
    })
}
