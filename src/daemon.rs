#![allow(clippy::future_not_send)]

use std::path::Path;

use actix::{Actor, SyncArbiter};
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use eyre::Result;
use itertools::Itertools;

use tmexclude_lib::config::Config;
use tmexclude_lib::walker::{SkipCache, Walker};
use tmexclude_lib::watcher::{RegisterWatcher, Watcher};

pub async fn app(config: Config, addr: impl AsRef<Path>) -> Result<()> {
    let walker_skip_cache = SkipCache::default();
    let walker_config = config.walk.clone();
    let walker_addr = SyncArbiter::start(num_cpus::get(), move || {
        Walker::new(walker_config.clone(), walker_skip_cache.clone())
    });

    let watcher = Watcher::new(config.mode, walker_addr);
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
