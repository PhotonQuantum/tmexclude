use std::time::Instant;

use figment::providers::{Format, Yaml};
use log::info;

use tmexclude_lib::walker::walk;
use tmexclude_lib::Config;

fn main() {
    pretty_env_logger::init();
    let config = Config::from(Yaml::file("config.example.yml")).expect("config");
    let root = config.root().expect("common root");

    info!("Start walk");
    let time_start = Instant::now();
    let plan = walk(root, config, true);
    let elapsed = Instant::now() - time_start;

    info!("Completed. Elapsed time: {:?}", elapsed);
    info!("Plan: {:#?}", plan);
}
