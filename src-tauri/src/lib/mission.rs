use std::{io, mem};
use std::sync::Arc;


use arc_swap::ArcSwap;
use parking_lot::Mutex;
use tauri::async_runtime::JoinHandle;

use crate::{Config, Metrics, watch_task};

pub struct Mission {
    config: ArcSwap<Config>,
    watcher_handle: Mutex<JoinHandle<io::Result<()>>>,
    metrics: Arc<Metrics>
}

impl Mission {
    /// Create a new mission.
    ///
    /// This function will start a watcher task.
    #[must_use] pub fn new(config: Arc<Config>) -> Arc<Self> {
        Arc::new_cyclic(move |this| {
            let task = watch_task(this.clone());
            let handle = tauri::async_runtime::spawn(task);
            Self {
                config: ArcSwap::new(config),
                watcher_handle: Mutex::new(handle),
                metrics: Arc::new(Metrics::default())
            }
        })
    }
    /// Get current config.
    pub fn config(&self) -> Arc<Config> {
        self.config.load().clone()
    }
    /// Get metrics.
    pub fn metrics(&self) -> Arc<Metrics> {
        self.metrics.clone()
    }
    /// Set new config.
    ///
    /// This method will restart watcher task.
    pub fn set_config(self: Arc<Self>, config: Arc<Config>) {
        self.config.store(config);
        self.reload();
    }
    /// Reload watcher task to apply new config.
    pub fn reload(self: Arc<Self>) {
        let new_task = watch_task(Arc::downgrade(&self));
        let handle = tauri::async_runtime::spawn(new_task);

        let old_handle = mem::replace(&mut *self.watcher_handle.lock(), handle);
        old_handle.abort();
    }
}
