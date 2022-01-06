//! Utils and actors to walk directories recursively (or not) and perform `TimeMachine` operations on demand.
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fs;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use actix::{Actor, Context, Handler, Message};
use crossbeam_queue::SegQueue;
use itertools::Itertools;
use jwalk::WalkDirGeneric;
use log::{debug, info, warn};
use moka::sync::Cache;
use tap::TapFallible;

use crate::config::{ApplyMode, Directory, Rule, WalkConfig};
use crate::tmutil::{is_excluded, ExclusionAction, ExclusionActionBatch};

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
struct CachedPath(Path);

impl From<&Path> for &CachedPath {
    fn from(p: &Path) -> Self {
        // SAFETY CachedPath is repr(transparent)
        unsafe { &*(p as *const Path as *const CachedPath) }
    }
}

impl Borrow<CachedPath> for Arc<PathBuf> {
    fn borrow(&self) -> &CachedPath {
        self.as_path().into()
    }
}

/// Actor to walk directories and perform `TimeMachine` operations on demand.
pub struct Walker {
    config: WalkConfig,
    skip_cache: SkipCache,
}

impl Walker {
    /// Create a new instance.
    #[must_use]
    pub const fn new(config: WalkConfig, skip_cache: SkipCache) -> Self {
        Self { config, skip_cache }
    }
}

impl Actor for Walker {
    type Context = Context<Self>;
}

/// Walk through a directory with given rules and apply the plan.
#[derive(Debug, Clone, Eq, PartialEq, Message)]
#[rtype("()")]
pub struct Walk {
    /// The root of the scan
    pub root: PathBuf,
    /// Whether this walk is recursive or not.
    pub recursive: bool,
    /// [`ApplyMode`](ApplyMode) of this walk.
    pub apply: ApplyMode,
}

impl Handler<Walk> for Walker {
    type Result = ();

    fn handle(&mut self, msg: Walk, _ctx: &mut Self::Context) -> Self::Result {
        let batch = walk_non_recursive(&*msg.root, &self.config, &*self.skip_cache);
        if batch.is_empty() {
            return;
        }
        debug!("Apply batch {:?}", batch);
        match msg.apply {
            ApplyMode::DryRun => {}
            ApplyMode::AddOnly => batch.apply(false),
            ApplyMode::All => batch.apply(true),
        }
    }
}

/// Walk through a directory with given rules recursively and return an exclusion action plan.
#[must_use]
pub fn walk_recursive(root: &Path, config: WalkConfig) -> ExclusionActionBatch {
    let batch_queue = Arc::new(SegQueue::new());
    {
        let batch_queue = batch_queue.clone();
        WalkDirGeneric::<(_, ())>::new(root)
            .root_read_dir_state(config)
            .skip_hidden(false)
            .process_read_dir(move |_, path, config, children| {
                // Remove effect-less directories & skips.
                config.directories.retain(|directory| {
                    path.starts_with(&directory.path) || directory.path.starts_with(path)
                });
                config.skips.retain(|skip| skip.starts_with(path));

                if config.directories.is_empty() {
                    // There's no need to go deeper.
                    for child in children.iter_mut().filter_map(|child| child.as_mut().ok()) {
                        child.read_children_path = None;
                    }
                    return;
                }

                // Acquire excluded state.
                let children = children
                    .iter_mut()
                    .filter_map(|entry| {
                        entry
                            .as_mut()
                            .tap_err(|e| warn!("Error when scanning dir {:?}: {}", path, e))
                            .ok()
                    })
                    .filter_map(|entry| {
                        let path = entry.path();
                        if config.skips.contains(&path) {
                            // Skip this entry in all preceding procedures and scans.
                            entry.read_children_path = None;
                            None
                        } else {
                            Some((entry, is_excluded(&path).ok()?))
                        }
                    })
                    .collect_vec();

                // Generate diff.
                let shallow_list: HashMap<_, _> = children
                    .iter()
                    .map(|(path, excluded)| {
                        (PathBuf::from(path.file_name().to_os_string()), *excluded)
                    })
                    .collect();
                let diff = generate_diff(path, &shallow_list, &*config.directories);

                // Exclude already excluded or uncovered children.
                for (entry, excluded) in children {
                    let path = entry.path();
                    if (excluded && !diff.remove.contains(&path)) || diff.add.contains(&path) {
                        entry.read_children_path = None;
                    }
                }
                batch_queue.push(diff);
            })
            .into_iter()
            .for_each(|_| {});
    }
    let mut actions = ExclusionActionBatch::default();
    while let Some(action) = batch_queue.pop() {
        actions += action;
    }
    actions
}

/// Walk through a directory with given rules non-recursively and return an exclusion action plan.
#[must_use]
pub fn walk_non_recursive(
    root: &Path,
    config: &WalkConfig,
    skip_cache: &Cache<PathBuf, ()>,
) -> ExclusionActionBatch {
    if skip_cache.get::<CachedPath>(root.into()).is_some() {
        // Skip cache hit, early exit.
        info!("hit skip cache");
        return ExclusionActionBatch::default();
    }

    if config.skips.iter().any(|skip| root.starts_with(skip)) {
        // The directory should be skipped.
        skip_cache.insert(root.to_path_buf(), ());
        return ExclusionActionBatch::default();
    }

    let mut directories = config
        .directories
        .iter()
        .filter(|directory| root.starts_with(&directory.path) || directory.path.starts_with(root))
        .peekable();
    if directories.peek().is_none() {
        // There's no need to scan because no rules is applicable.
        skip_cache.insert(root.to_path_buf(), ());
        return ExclusionActionBatch::default();
    }

    if root
        .ancestors()
        .any(|path| is_excluded(path).unwrap_or(false))
    {
        // One of its parents is excluded.
        // Note that we don't put this dir into cache because the exclusion state of ancestors is unknown.
        return ExclusionActionBatch::default();
    }

    debug!("Walk through {:?}", root);
    match fs::read_dir(root) {
        Ok(dir) => {
            let shallow_list: HashMap<_, _> = dir
                .filter_map(|entry| {
                    entry
                        .tap_err(|e| warn!("Error when scanning dir {:?}: {}", root, e))
                        .ok()
                })
                .filter_map(|entry| {
                    let path = entry.path();
                    if config.skips.contains(&path) {
                        // Skip this entry in all preceding procedures and scans.
                        None
                    } else {
                        Some((
                            PathBuf::from(path.file_name().expect("file name").to_os_string()),
                            is_excluded(&path).ok()?,
                        ))
                    }
                })
                .collect();
            generate_diff(root, &shallow_list, directories)
        }
        Err(e) => {
            warn!("Error when scanning dir {:?}: {}", root, e);
            ExclusionActionBatch::default()
        }
    }
}

fn generate_diff<'a, 'b>(
    cwd: &'a Path,
    shallow_list: &'a HashMap<PathBuf, bool>,
    directories: impl IntoIterator<Item = &'b Directory>,
) -> ExclusionActionBatch {
    let candidate_rules: Vec<&Rule> = directories
        .into_iter()
        .filter(|directory| directory.path.starts_with(cwd) || cwd.starts_with(&directory.path))
        .flat_map(|directory| &directory.rules)
        .collect();
    shallow_list
        .iter()
        .filter_map(|(name, excluded)| {
            let expected_excluded = candidate_rules.iter().any(|rule| {
                (&rule.excludes).contains(name)
                    && (rule.if_exists.is_empty()
                        || rule
                            .if_exists
                            .iter()
                            .any(|if_exist| shallow_list.contains_key(if_exist.as_path())))
            });
            match (expected_excluded, *excluded) {
                (true, false) => Some(ExclusionAction::Add(cwd.join(name))),
                (false, true) => Some(ExclusionAction::Remove(cwd.join(name))),
                _ => None,
            }
        })
        .into()
}
