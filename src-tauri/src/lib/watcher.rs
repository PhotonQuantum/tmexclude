//! Filesystem watcher.

use std::collections::HashSet;
use std::io;
use std::mem;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::sync::Arc;
use std::sync::Weak;
use std::time::Duration;

use fsevent_stream::ffi::{kFSEventStreamCreateFlagIgnoreSelf, kFSEventStreamEventIdSinceNow};
use fsevent_stream::stream::{create_event_stream, EventStreamHandler};
use futures::StreamExt;
use parking_lot::Mutex;
use tracing::{debug, error, warn};

use crate::config::WalkConfig;
use crate::metrics::Metrics;
use crate::mission::Mission;
use crate::skip_cache::SkipCache;
use crate::walker::walk_non_recursive;

const EVENT_DELAY: Duration = Duration::from_secs(30);
/// Maximum number of concurrently running batch walks.
const MAX_IN_FLIGHT_BATCHES: usize = 2;
/// Hard cap of unique paths held by the bounded pending set.
const MAX_PENDING_PATHS: usize = 65536;
/// Ticker interval driving pending-set drains without new `FSEvents`.
const DRAIN_TICK: Duration = Duration::from_secs(1);

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

/// RAII guard for one in-flight batch. Decrementing on scope exit — including
/// during unwinding — keeps the [`MAX_IN_FLIGHT_BATCHES`] cap accounting
/// accurate under early returns and panics.
struct InFlightGuard(Arc<AtomicUsize>);

impl InFlightGuard {
    /// Reserve one in-flight slot; `None` when the cap is already reached.
    fn acquire(counter: &Arc<AtomicUsize>, max: usize) -> Option<Self> {
        counter
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                if current < max {
                    Some(current + 1)
                } else {
                    None
                }
            })
            .ok()
            .map(|_| Self(Arc::clone(counter)))
    }
}

impl Drop for InFlightGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Release);
    }
}

/// Result of merging a batch into the pending set.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct MergeOutcome {
    accepted: usize,
    dropped: usize,
}

/// Bounded, deduplicated collection of paths waiting for an in-flight slot.
struct PendingSet {
    paths: HashSet<PathBuf>,
    cap: usize,
}

impl PendingSet {
    fn new(cap: usize) -> Self {
        Self {
            paths: HashSet::new(),
            cap,
        }
    }

    fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

    /// Merge `paths` into the set, deduplicating. Unique paths that would
    /// exceed the cap are dropped with `warn!` (bounded loss, by design).
    fn merge<I>(&mut self, paths: I) -> MergeOutcome
    where
        I: IntoIterator<Item = PathBuf>,
    {
        let mut outcome = MergeOutcome {
            accepted: 0,
            dropped: 0,
        };
        for path in paths {
            if self.paths.contains(&path) {
                continue;
            }
            if self.paths.len() >= self.cap {
                outcome.dropped += 1;
                warn!(
                    "Pending set cap reached ({}), dropping path {}",
                    self.cap,
                    path.display()
                );
                continue;
            }
            self.paths.insert(path);
            outcome.accepted += 1;
        }
        outcome
    }

    /// Swap out the whole set as one batch, deduplicated and ancestor-first
    /// ordered, ready for [`process_batch`].
    fn drain(&mut self) -> Vec<PathBuf> {
        let taken = mem::take(&mut self.paths);
        prepare_paths(taken.into_iter().collect())
    }
}

/// Normalize a batch of paths: dedup exact duplicates, then stable-sort by
/// component count so ancestors precede descendants (their apply lets
/// descendants hit the ancestor-exclusion early return in
/// `walk_non_recursive`).
fn prepare_paths(mut paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths.sort_unstable();
    paths.dedup();
    paths.sort_by_key(|path| path.components().count());
    paths
}

/// Shared per-task context cloned into each blocking batch task.
#[derive(Clone)]
struct BatchContext {
    walk_config: Arc<WalkConfig>,
    support_dump: bool,
    no_include: bool,
    cache: SkipCache,
    metrics: Arc<Metrics>,
}

impl BatchContext {
    /// Spawn one blocking task that walks and applies `paths` sequentially.
    fn spawn_batch(self, paths: Vec<PathBuf>, guard: InFlightGuard) {
        tauri::async_runtime::spawn_blocking(move || {
            process_batch(
                &paths,
                &self.walk_config,
                self.support_dump,
                self.no_include,
                &self.cache,
                &self.metrics,
            );
            drop(guard);
        });
    }
}

/// Walk and apply each path sequentially, preserving today's per-path metrics
/// semantics exactly: each path's diff is counted and applied individually,
/// and apply errors are logged per file.
fn process_batch(
    paths: &[PathBuf],
    walk_config: &WalkConfig,
    support_dump: bool,
    no_include: bool,
    cache: &SkipCache,
    metrics: &Metrics,
) {
    for path in paths {
        let mut batch = walk_non_recursive(path, walk_config, support_dump, no_include, cache);
        if batch.is_empty() {
            continue;
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
        if let Err(errors) = batch.apply(support_dump) {
            for (path, e) in errors {
                error!("Error when applying on file {}: {}", path.display(), e);
            }
        }
    }
}

/// Attempt one pending-set drain: when an in-flight slot is free and the set
/// is non-empty, swap the whole set out and process it as one normal batch
/// (same dedup + ancestor-first ordering). Cheap no-op when saturated or
/// empty; driven by the ticker so the drain never requires new `FSEvents`.
fn try_drain(context: &BatchContext, pending: &Mutex<PendingSet>, in_flight: &Arc<AtomicUsize>) {
    let guard = match InFlightGuard::acquire(in_flight, MAX_IN_FLIGHT_BATCHES) {
        Some(guard) => guard,
        None => return,
    };
    let paths = {
        let mut set = pending.lock();
        if set.is_empty() {
            return;
        }
        set.drain()
    };
    context.clone().spawn_batch(paths, guard);
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
    let support_dump = config.support_dump;

    let (mut stream, event_handle) = create_event_stream(
        paths,
        kFSEventStreamEventIdSinceNow,
        EVENT_DELAY,
        kFSEventStreamCreateFlagIgnoreSelf,
    )?;
    let _guard = DropGuard::new(event_handle);

    let cache = SkipCache::default();
    let in_flight = Arc::new(AtomicUsize::new(0));
    let pending = Arc::new(Mutex::new(PendingSet::new(MAX_PENDING_PATHS)));
    let context = BatchContext {
        walk_config: Arc::clone(&config.walk),
        support_dump,
        no_include,
        cache,
        metrics,
    };

    // Ticker thread: attempts the pending-set drain once per tick so queued
    // batches proceed without new FSEvents. When `watch_task` returns (stream
    // end, task abort on config reload, error path), `_shutdown_tx` is
    // dropped, the channel disconnects, and the ticker exits promptly.
    let (_shutdown_tx, shutdown_rx) = channel::<()>();
    let _ticker_handle = std::thread::Builder::new()
        .name("watcher-drain-ticker".to_string())
        .spawn({
            let context = context.clone();
            let in_flight = Arc::clone(&in_flight);
            let pending = Arc::clone(&pending);
            move || loop {
                match shutdown_rx.recv_timeout(DRAIN_TICK) {
                    Ok(()) | Err(RecvTimeoutError::Disconnected) => return,
                    Err(RecvTimeoutError::Timeout) => try_drain(&context, &pending, &in_flight),
                }
            }
        })?;

    while let Some(items) = stream.next().await {
        // One unit of work per delivered FSEvent batch: paths are merged,
        // deduplicated, and ancestor-first ordered before a single blocking
        // task walks and applies them sequentially.
        let paths: Vec<PathBuf> = items
            .into_iter()
            .filter(|item| !item.path.as_os_str().is_empty())
            .map(|item| item.path)
            .collect();
        if paths.is_empty() {
            continue;
        }
        let paths = prepare_paths(paths);
        let guard = match InFlightGuard::acquire(&in_flight, MAX_IN_FLIGHT_BATCHES) {
            Some(guard) => guard,
            None => {
                // In-flight at cap: absorb the whole batch into the set.
                let outcome = pending.lock().merge(paths);
                debug!(
                    "Watcher saturated, pending batch merged: +{} accepted, {} dropped",
                    outcome.accepted, outcome.dropped
                );
                continue;
            }
        };
        context.clone().spawn_batch(paths, guard);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use super::{prepare_paths, InFlightGuard, MergeOutcome, PendingSet, MAX_PENDING_PATHS};

    fn path(s: &str) -> PathBuf {
        PathBuf::from(s)
    }

    #[test]
    fn prepare_paths_dedups_exact_duplicates() {
        let paths = vec![
            path("/a/b"),
            path("/a"),
            path("/a/b"),
            path("/a"),
            path("/c"),
        ];
        // Dedup first; then depth ordering places shallow paths before deep ones.
        assert_eq!(
            prepare_paths(paths),
            vec![path("/a"), path("/c"), path("/a/b")]
        );
    }

    #[test]
    fn prepare_paths_orders_ancestors_before_descendants() {
        let paths = vec![
            path("/a/b/c"),
            path("/a/b"),
            path("/x/y"),
            path("/x"),
            path("/deep/path/here/now"),
        ];
        let prepared = prepare_paths(paths);
        let depths: Vec<usize> = prepared.iter().map(|p| p.components().count()).collect();
        assert!(
            depths.windows(2).all(|window| window[0] <= window[1]),
            "depths must be non-decreasing: {:?} ({:?})",
            depths,
            prepared
        );
        let a = prepared.iter().position(|p| p == &path("/a/b")).unwrap();
        let a_b_c = prepared.iter().position(|p| p == &path("/a/b/c")).unwrap();
        let x = prepared.iter().position(|p| p == &path("/x")).unwrap();
        let x_y = prepared.iter().position(|p| p == &path("/x/y")).unwrap();
        assert!(a < a_b_c);
        assert!(x < x_y);
    }

    #[test]
    fn prepare_paths_same_depth_order_stable() {
        // Equal-depth groups keep their relative (lexicographic) order,
        // regardless of the depth of interleaved paths.
        let paths = vec![path("/b"), path("/a/x"), path("/a")];
        assert_eq!(
            prepare_paths(paths),
            vec![path("/a"), path("/b"), path("/a/x")]
        );
    }

    #[test]
    fn pending_set_merge_dedups() {
        let mut set = PendingSet::new(8);
        let outcome = set.merge(vec![path("/a"), path("/b"), path("/a")]);
        assert_eq!(
            outcome,
            MergeOutcome {
                accepted: 2,
                dropped: 0
            }
        );
        assert_eq!(set.drain(), vec![path("/a"), path("/b")]);
    }

    #[test]
    fn pending_set_merge_dedups_across_merges() {
        let mut set = PendingSet::new(8);
        assert_eq!(
            set.merge(vec![path("/a")]),
            MergeOutcome {
                accepted: 1,
                dropped: 0
            }
        );
        // Re-merging an already-pending path is a no-op, not a drop.
        assert_eq!(
            set.merge(vec![path("/a"), path("/b")]),
            MergeOutcome {
                accepted: 1,
                dropped: 0
            }
        );
    }

    #[test]
    fn pending_set_cap_drops_excess_paths() {
        let mut set = PendingSet::new(3);
        let outcome = set.merge(vec![
            path("/1"),
            path("/2"),
            path("/3"),
            path("/4"),
            path("/5"),
        ]);
        assert_eq!(
            outcome,
            MergeOutcome {
                accepted: 3,
                dropped: 2
            }
        );
        assert_eq!(set.drain(), vec![path("/1"), path("/2"), path("/3")]);
    }

    #[test]
    fn pending_set_default_cap_boundary() {
        let mut set = PendingSet::new(MAX_PENDING_PATHS);
        let paths: Vec<PathBuf> = (0..=MAX_PENDING_PATHS)
            .map(|i| PathBuf::from(format!("/overflow/{}", i)))
            .collect();
        let outcome = set.merge(paths);
        assert_eq!(outcome.accepted, MAX_PENDING_PATHS);
        assert_eq!(outcome.dropped, 1);
        assert_eq!(set.drain().len(), MAX_PENDING_PATHS);
    }

    #[test]
    fn pending_set_drain_empties_and_returns_contents() {
        let mut set = PendingSet::new(8);
        set.merge(vec![path("/b"), path("/a"), path("/b")]);
        assert_eq!(set.drain(), vec![path("/a"), path("/b")]);
        assert!(set.is_empty());
        // Draining an empty set yields an empty batch.
        assert!(set.drain().is_empty());
    }

    #[test]
    fn pending_set_drain_content_ancestor_first() {
        let mut set = PendingSet::new(16);
        set.merge(vec![
            path("/a/b/c"),
            path("/x/y/z"),
            path("/a"),
            path("/x/y"),
        ]);
        let drained = set.drain();
        let depths: Vec<usize> = drained.iter().map(|p| p.components().count()).collect();
        assert!(depths.windows(2).all(|window| window[0] <= window[1]));
        let a = drained.iter().position(|p| p == &path("/a")).unwrap();
        let a_b_c = drained.iter().position(|p| p == &path("/a/b/c")).unwrap();
        let x_y = drained.iter().position(|p| p == &path("/x/y")).unwrap();
        let x_y_z = drained.iter().position(|p| p == &path("/x/y/z")).unwrap();
        assert!(a < a_b_c);
        assert!(x_y < x_y_z);
    }

    #[test]
    fn in_flight_guard_acquires_and_releases() {
        let counter = Arc::new(AtomicUsize::new(0));
        {
            let guard = InFlightGuard::acquire(&counter, 2);
            assert!(guard.is_some());
            assert_eq!(counter.load(Ordering::SeqCst), 1);
        }
        assert_eq!(counter.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn in_flight_guard_rejects_at_cap() {
        let counter = Arc::new(AtomicUsize::new(0));
        let first = InFlightGuard::acquire(&counter, 2);
        let second = InFlightGuard::acquire(&counter, 2);
        assert!(first.is_some());
        assert!(second.is_some());
        assert_eq!(counter.load(Ordering::SeqCst), 2);
        assert!(InFlightGuard::acquire(&counter, 2).is_none());
        // A rejected acquisition leaves the counter untouched.
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn in_flight_guard_releases_on_early_return() {
        let counter = Arc::new(AtomicUsize::new(0));

        // The guard is dropped via scope exit on the early-return path.
        fn early_return(counter: &Arc<AtomicUsize>, max: usize) -> bool {
            let guard = match InFlightGuard::acquire(counter, max) {
                Some(guard) => guard,
                None => return false,
            };
            guard.0.load(Ordering::SeqCst) == 1
        }

        assert!(early_return(&counter, 2));
        assert_eq!(counter.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn in_flight_guard_releases_on_panic() {
        let counter = Arc::new(AtomicUsize::new(0));
        let result = std::panic::catch_unwind(|| {
            let _guard = InFlightGuard::acquire(&counter, 2).expect("below cap");
            assert_eq!(counter.load(Ordering::SeqCst), 1);
            panic!("simulated panic while holding the guard");
        });
        assert!(result.is_err());
        // The guard decrements during unwinding (panic-safe).
        assert_eq!(counter.load(Ordering::SeqCst), 0);
    }
}
