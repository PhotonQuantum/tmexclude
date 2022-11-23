//! Filesystem watcher.

use std::io;

use std::sync::{Weak};
use std::time::Duration;


use crate::skip_cache::SkipCache;
use crate::walker::walk_non_recursive;
use fsevent_stream::ffi::{kFSEventStreamCreateFlagIgnoreSelf, kFSEventStreamEventIdSinceNow};
use fsevent_stream::stream::{create_event_stream, EventStreamHandler};
use futures::StreamExt;
use log::{debug, error};

use futures::StreamExt;
use log::debug;
use crate::mission::Mission;

const EVENT_DELAY: Duration = Duration::from_secs(30);

struct DropGuard(Option<EventStreamHandler>);

impl DropGuard {
    pub const fn new(handler: EventStreamHandler) -> Self {
        Self(Some(handler))
    }
}

impl Drop for DropGuard {
    fn drop(&mut self) {
        if let Some(mut handler) = self.0.take() {
            handler.abort();
        }
    }
}

/// # Errors
/// Returns `io::Error` if fs event stream creation fails.
pub async fn watch_task(mission: Weak<Mission>) -> io::Result<()> {
    let mission = mission.upgrade().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::Other,
            "mission is dropped before watch task is started",
        )
    })?;
    let config = mission.config_();
    let metrics = mission.metrics();

    let paths = config
        .walk
        .directories
        .iter()
        .map(|directory| directory.path.as_path());
    let no_include = config.no_include;

    let (mut stream, event_handle) = create_event_stream(
        paths,
        kFSEventStreamEventIdSinceNow,
        EVENT_DELAY,
        kFSEventStreamCreateFlagIgnoreSelf,
    )?;
    let _guard = DropGuard::new(event_handle);

    let cache = SkipCache::default();
    while let Some(items) = stream.next().await {
        for item in items {
            if !item.path.as_os_str().is_empty() {
                tauri::async_runtime::spawn_blocking({
                    let _path = item.path.clone();
                    // TODO make walk_config Arc
                    let walk_config = config.walk.clone();
                    let cache = cache.clone();
                    let metrics = metrics.clone();
                    move || {
                        let mut batch = walk_non_recursive(&item.path, &walk_config, &cache);
                        if batch.is_empty() {
                            return;
                        }
                        debug!("Apply batch {:?}", batch);
                        if no_include {
                            batch.remove.clear();
                        }
                        metrics.inc_excluded(batch.add.len());
                        metrics.inc_included(batch.remove.len());
                        if let Some(last_file) = batch.add.last() {
                            metrics.set_last_excluded(last_file.as_path());
                        }
                        batch.apply();
                    }
                });
            }
        }
    }

    Ok(())
}
