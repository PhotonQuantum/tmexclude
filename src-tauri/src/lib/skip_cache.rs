//! Cache facilities used in walker.

use std::borrow::Borrow;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use moka::sync::Cache;

const CACHE_MAX_CAPACITY: u64 = 512;

/// Cache for skipped directories to avoid redundant syscall.
#[derive(Clone)]
pub struct SkipCache(Arc<Cache<PathBuf, ()>>);

impl Default for SkipCache {
    fn default() -> Self {
        Self(Arc::new(Cache::new(CACHE_MAX_CAPACITY)))
    }
}

impl Deref for SkipCache {
    type Target = Cache<PathBuf, ()>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Custom `Path` wrapper to implement `Borrow` for Arc<PathBuf>.
#[repr(transparent)]
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct CachedPath(Path);

impl From<&Path> for &CachedPath {
    fn from(p: &Path) -> Self {
        // SAFETY CachedPath is repr(transparent)
        unsafe { &*(p as *const Path as *const CachedPath) }
    }
}

impl Borrow<CachedPath> for PathBuf {
    fn borrow(&self) -> &CachedPath {
        self.as_path().into()
    }
}
