use figment::providers::{Format, Yaml};
use futures::StreamExt;
use log::info;

use tmexclude_lib::{Config, Watcher};

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    pretty_env_logger::init();
    let config = Config::from(Yaml::file("config.example.yml")).expect("config");
    let mut watcher = Watcher::new().expect("create watcher");
    for (_, directory) in config.directories {
        watcher
            .register_directory(directory)
            .expect("register directory");
    }
    let stream = watcher.take_stream();
    stream
        .for_each_concurrent(None, |event| async move {
            info!("event: {:?}", event);
        })
        .await;
}
