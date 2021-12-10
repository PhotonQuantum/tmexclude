#![allow(clippy::future_not_send)]

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use actix::clock::sleep;
use actix::{Actor, SyncArbiter};
use directories::ProjectDirs;
use figment::providers::{Format, Yaml};
use itertools::Itertools;
use parking_lot::Mutex;

use tmexclude_lib::config::Config;
use tmexclude_lib::persistent::PersistentState;
use tmexclude_lib::walker::Walker;
use tmexclude_lib::watcher::{RegisterWatcher, Watcher};

#[actix_rt::main]
async fn main() {
    run().await;
}

async fn run() {
    pretty_env_logger::init();
    let config = Config::from(Yaml::file("config.example.yml")).expect("config");

    let dir = ProjectDirs::from("me", "lightquantum", "tmexclude").expect("home dir to exist");
    let state_dir = dir.data_local_dir();
    std::fs::create_dir_all(&state_dir).expect("dir to be created");
    let state = Arc::new(Mutex::new(
        PersistentState::load(state_dir.join("state.json")).expect("to load"),
    ));

    let walker_config = config.walk.clone();
    let walker_addr =
        SyncArbiter::start(num_cpus::get(), move || Walker::new(walker_config.clone()));
    let watcher = Watcher::new(config.mode.clone(), state, walker_addr);

    let watcher_addr = watcher.start();
    watcher_addr
        .send(RegisterWatcher {
            paths: config
                .walk
                .read()
                .directories
                .iter()
                .map(|directory| directory.path.clone())
                .collect_vec(),
            interval: config.interval.load(Ordering::Relaxed).watch,
        })
        .await
        .expect("message to be delivered")
        .expect("watcher to be registered");

    sleep(Duration::from_secs(999)).await;
}
