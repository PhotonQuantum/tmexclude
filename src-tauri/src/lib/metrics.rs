use std::path::Path;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::SystemTime;

use arc_swap::ArcSwap;
use serde::{Serialize, Serializer};
use ts_rs::TS;

#[derive(Serialize, TS)]
#[ts(export, export_to = "../src/bindings/")]
pub struct Metrics {
    #[ts(type = "number")]
    #[serde(serialize_with = "serialize_atomic_usize")]
    files_excluded: AtomicUsize,
    #[ts(type = "number")]
    #[serde(serialize_with = "serialize_atomic_usize")]
    files_included: AtomicUsize,
    #[ts(type = "string")]
    #[serde(serialize_with = "serialize_arc_path")]
    last_excluded: ArcSwap<Box<Path>>,
    #[ts(type = "number")]
    #[serde(serialize_with = "serialize_atomic_u64")]
    last_excluded_time: AtomicU64,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            files_excluded: AtomicUsize::new(0),
            files_included: AtomicUsize::new(0),
            last_excluded: ArcSwap::new(Arc::new(Box::from(Path::new("")))),
            last_excluded_time: AtomicU64::new(0),
        }
    }
}

impl Metrics {
    pub fn inc_excluded(&self, n: usize) {
        self.files_excluded.fetch_add(n, Ordering::Relaxed);
    }
    pub fn inc_included(&self, n: usize) {
        self.files_included.fetch_add(n, Ordering::Relaxed);
    }
    pub fn set_last_excluded(&self, path: &Path) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("past is future")
            .as_secs();
        self.last_excluded.store(Arc::new(Box::from(path)));
        self.last_excluded_time.store(now, Ordering::Relaxed);
    }
}

fn serialize_atomic_usize<S>(t: &AtomicUsize, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    t.load(Ordering::Relaxed).serialize(s)
}

fn serialize_atomic_u64<S>(t: &AtomicU64, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    t.load(Ordering::Relaxed).serialize(s)
}

fn serialize_arc_path<S>(t: &ArcSwap<Box<Path>>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    t.load().serialize(s)
}
