#![allow(clippy::future_not_send)]


use std::path::Path;


use actix::{Actor, SyncArbiter};
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use eyre::{Result, WrapErr};
use itertools::Itertools;

use tmexclude_lib::config::Config;
use tmexclude_lib::persistent::PersistentState;
use tmexclude_lib::walker::{SkipCache, Walker};
use tmexclude_lib::watcher::{RegisterWatcher, Watcher};

use crate::utils::ensure_state_dir;

pub async fn app(config: Config, addr: impl AsRef<Path>) -> Result<()> {
    let state_dir = ensure_state_dir()?;
    let state = PersistentState::load(state_dir.join("state.json"))
        .wrap_err("Failed to load persisted state")?;
    let state_addr = state.start();

    let walker_skip_cache = SkipCache::default();
    let walker_config = config.walk.clone();
    let walker_addr = SyncArbiter::start(num_cpus::get(), move || {
        Walker::new(walker_config.clone(), walker_skip_cache.clone())
    });

    let watcher = Watcher::new(config.mode, state_addr, walker_addr);
    let watcher_addr = watcher.start();
    watcher_addr
        .send(RegisterWatcher {
            paths: config
                .walk
                .directories
                .iter()
                .map(|directory| directory.path.clone())
                .collect_vec(),
            interval: config.interval.watch,
        })
        .await
        .expect("message to be delivered")
        .expect("watcher to be registered");

    Ok(HttpServer::new(move || {
        App::new()
            .app_data(Data::new(config.clone()))
            .app_data(Data::new(watcher_addr.clone()))
    })
    .bind_uds(addr)?
    .run()
    .await?)
}
