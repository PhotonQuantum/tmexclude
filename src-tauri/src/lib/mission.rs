use std::sync::Arc;
use std::{io, mem};

use arc_swap::ArcSwap;
use parking_lot::Mutex;
use tauri::async_runtime::{JoinHandle};
use tauri::{AppHandle, Manager};

use crate::{watch_task, Metrics};
use crate::config::{Config, PreConfig};
use crate::error::ConfigError;

pub struct Mission {
    app: AppHandle,
    pre_config: ArcSwap<PreConfig>,
    config: ArcSwap<Config>,
    watcher_handle: Mutex<JoinHandle<io::Result<()>>>,
    metrics: Arc<Metrics>,
}

impl Mission {
    /// Create a new mission.
    ///
    /// This function will start a watcher task.
    pub fn new_arc(app: AppHandle, pre_config: PreConfig) -> Result<Arc<Self>, ConfigError> {
        let config = Config::try_from(pre_config.clone())?;
        Ok(Arc::new_cyclic(move |this| {
            let task = watch_task(this.clone());
            let handle = tauri::async_runtime::spawn(task);
            Self {
                app,
                pre_config: ArcSwap::from_pointee(pre_config),
                config: ArcSwap::from_pointee(config),
                watcher_handle: Mutex::new(handle),
                metrics: Arc::new(Metrics::default()),
            }
        }))
    }
    /// Get internal config.
    pub(crate) fn config_(&self) -> Arc<Config> {
        self.config.load().clone()
    }
    /// Get current pre-config.
    pub fn config(&self) -> Arc<PreConfig> {
        self.pre_config.load().clone()
    }
    /// Get metrics.
    pub fn metrics(&self) -> Arc<Metrics> {
        self.metrics.clone()
    }
    /// Set new config.
    ///
    /// This method will restart watcher task.
    pub fn set_config(self: Arc<Self>, config: PreConfig) -> Result<(), ConfigError> {
        let config_ = Config::try_from(config.clone())?;
        self.pre_config.store(Arc::new(config));
        self.config.store(Arc::new(config_));
        self.reload();
        Ok(())
    }
    /// Reload watcher task to apply new config.
    pub fn reload(self: Arc<Self>) {
        // Create and spawn new watch task.
        let new_task = watch_task(Arc::downgrade(&self));
        let handle = tauri::async_runtime::spawn(new_task);

        // Stop old watch task.
        let old_handle = mem::replace(&mut *self.watcher_handle.lock(), handle);
        old_handle.abort();

        // Broadcast new config.
        self.app
            .emit_all("config_changed", self.config())
            .expect("failed to broadcast event");
    }
}
