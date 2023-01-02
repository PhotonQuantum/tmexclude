/*
 * Copyright (c) 2022 LightQuantum.
 * SPDX-License-Identifier: MIT
 */

use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::Mutex;
use serde_json::{Map, Value};
use tauri::{AppHandle, Manager, Runtime};
use tracing::error;

#[derive(Debug, Clone)]
pub struct Store {
    data: Arc<Mutex<Map<String, Value>>>,
    path: PathBuf,
}

fn read_from_path(path: &Path) -> Map<String, Value> {
    if path.exists() {
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap_or_else(|e| {
            error!(?e, ".properties file is corrupted, deleting it");
            std::fs::remove_file(path).unwrap();
            Default::default()
        })
    } else {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        Default::default()
    }
}

impl Store {
    pub fn new(base: &Path) -> Self {
        let path = base.join(".properties");
        let data = read_from_path(&path);
        Self {
            data: Arc::new(Mutex::new(data)),
            path,
        }
    }
    pub fn reload(&self) {
        let mut data = self.data.lock();
        *data = read_from_path(&self.path);
    }
    pub fn get(&self, key: &str) -> Option<Value> {
        let data = self.data.lock();
        data.get(key).cloned()
    }
    pub fn set<R: Runtime>(&self, handle: &AppHandle<R>, key: String, value: Value) {
        let mut data = self.data.lock();
        data.insert(key, value);
        std::fs::write(&self.path, serde_json::to_vec(&*data).unwrap()).unwrap();
        drop(handle.emit_all("properties_changed", data.clone()));
    }
    pub fn del<R: Runtime>(&self, handle: &AppHandle<R>, key: &str) {
        let mut data = self.data.lock();
        data.remove(key);
        std::fs::write(&self.path, serde_json::to_vec(&*data).unwrap()).unwrap();
        drop(handle.emit_all("properties_changed", data.clone()));
    }
}
