use std::error::Error;
use std::path::PathBuf;

use actix::Actor;
use actix_rt::System;
use eyre::Result;
use figment::Provider;

use tmexclude_lib::daemon::Daemon;
use tmexclude_lib::rpc::server::start_server;
use tmexclude_lib::utils::TypeEq;

use crate::common::ensure_uds_path;

pub fn daemon<F, O, E, P>(provider: F, uds: Option<PathBuf>) -> Result<()>
where
    F: Fn() -> O + Unpin + 'static,
    O: TypeEq<Rhs = Result<P, E>>,
    E: 'static + Error + Send + Sync,
    P: Provider,
{
    System::new().block_on(async move {
        let daemon = Daemon::new(provider)?;
        let addr = daemon.start();
        Ok(start_server(ensure_uds_path(uds, true)?, addr).await?)
    })
}
