#![allow(clippy::use_self)]

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::{io, mem};

use arc_swap::ArcSwap;
use parking_lot::{Mutex, RwLock};
use serde::Serialize;
use serde_json::Value;
use tauri::async_runtime::{channel, JoinHandle};
use tauri::{AppHandle, Manager};
use ts_rs::TS;

use crate::config::{Config, ConfigManager, PreConfig};
use crate::error::ConfigError;
use crate::metrics::Metrics;
use crate::properties::Store;
use crate::tmutil::ExclusionActionBatch;
use crate::walker::walk_recursive;
use crate::watcher::watch_task;

pub struct Mission {
    app: AppHandle,
    properties: Store,
    config_manager: ConfigManager,
    pre_config: ArcSwap<PreConfig>,
    config: ArcSwap<Config>,
    watcher_handle: Mutex<JoinHandle<io::Result<()>>>,
    metrics: Arc<Metrics>,
    scan_status: RwLock<ScanStatus>,
    scan_handle: Mutex<Option<ScanHandle>>,
}

pub struct ScanHandle {
    abort_flag: Arc<AtomicBool>,
    task_handle: JoinHandle<()>,
}

impl ScanHandle {
    pub fn stop(self) {
        self.abort_flag.store(true, Ordering::Relaxed);
        self.task_handle.abort();
    }
}

#[derive(Debug, Clone, Default, Serialize, TS)]
#[ts(export, export_to = "../src/bindings/")]
#[serde(tag = "step", content = "content", rename_all = "kebab-case")]
pub enum ScanStatus {
    #[default]
    Idle,
    Scanning {
        current_path: PathBuf,
        found: usize,
    },
    Result(ExclusionActionBatch),
}

impl Mission {
    /// Create a new mission.
    ///
    /// This function will start a watcher task.
    ///
    /// # Errors
    /// Returns error if can't load config.
    pub fn new_arc(
        app: AppHandle,
        config_manager: ConfigManager,
        properties: Store,
    ) -> Result<Arc<Self>, ConfigError> {
        let pre_config = config_manager.load()?;
        let config = Config::try_from(pre_config.clone())?;
        Ok(Arc::new_cyclic(move |this| {
            let task = watch_task(this.clone());
            let handle = tauri::async_runtime::spawn(task);
            Self {
                app,
                properties,
                config_manager,
                pre_config: ArcSwap::from_pointee(pre_config),
                config: ArcSwap::from_pointee(config),
                watcher_handle: Mutex::new(handle),
                metrics: Arc::new(Metrics::default()),
                scan_status: Default::default(),
                scan_handle: Default::default(),
            }
        }))
    }
    pub fn store_get(&self, key: &str) -> Option<Value> {
        self.properties.get(key)
    }
    pub fn store_set(&self, key: String, value: Value) {
        self.properties.set(&self.app, key, value);
    }
    pub fn store_del(&self, key: &str) {
        self.properties.del(&self.app, key);
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
    ///
    /// # Errors
    /// Returns error if can't persist config, or can't parse surface config (shouldn't happen).
    pub fn set_config(self: Arc<Self>, config: PreConfig) -> Result<(), ConfigError> {
        self.config_manager.save(&config)?;
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
    fn set_scan_status(&self, status: ScanStatus) {
        *self.scan_status.write() = status.clone();
        self.app
            .emit_all("scan_status_changed", status)
            .expect("failed to broadcast event");
    }
    pub fn stop_full_scan(&self) {
        if let Some(handle) = self.scan_handle.lock().take() {
            handle.stop();
        }
        self.set_scan_status(ScanStatus::Idle);
    }
    pub fn scan_status(&self) -> ScanStatus {
        self.scan_status.read().clone()
    }
    pub fn full_scan(self: Arc<Self>) {
        self.stop_full_scan();

        let abort = Arc::new(AtomicBool::new(false));
        let found = Arc::new(AtomicUsize::new(0));
        let this = self.clone();

        let scan_task = {
            let abort = abort.clone();
            async move {
                let (curr_tx, mut curr_rx) = channel(128);
                std::thread::spawn({
                    let this = this.clone();
                    let found = found.clone();
                    move || {
                        let walk_config = (*this.config_().walk).clone();
                        let result = walk_recursive(walk_config, curr_tx, found, abort.clone());
                        if !abort.load(Ordering::Relaxed) {
                            this.set_scan_status(ScanStatus::Result(result));
                        }
                    }
                });

                while let Some(curr) = curr_rx.recv().await {
                    this.set_scan_status(ScanStatus::Scanning {
                        current_path: curr,
                        found: found.load(Ordering::Relaxed),
                    });
                }
            }
        };

        let handle = ScanHandle {
            abort_flag: abort,
            task_handle: tauri::async_runtime::spawn(scan_task),
        };
        self.scan_handle.lock().replace(handle);
    }
}
