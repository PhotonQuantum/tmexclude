//! Utils and actors to walk directories recursively (or not) and perform `TimeMachine` operations on demand.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use crossbeam::queue::SegQueue;
use itertools::Itertools;
use jwalk::WalkDirGeneric;
use moka::sync::Cache;
use tap::TapFallible;
use tauri::async_runtime::Sender;
use tracing::{debug, warn};

use crate::config::{Directory, Rule, WalkConfig};
use crate::skip_cache::CachedPath;
use crate::tmutil::{is_excluded, ExclusionAction, ExclusionActionBatch};

/// Walk through a directory with given rules recursively and return an exclusion action plan.
#[allow(clippy::needless_pass_by_value)]
#[must_use]
pub fn walk_recursive(
    config: WalkConfig,
    curr_tx: Sender<PathBuf>,
    found: Arc<AtomicUsize>,
    abort: Arc<AtomicBool>,
) -> ExclusionActionBatch {
    let batch_queue = Arc::new(SegQueue::new());
    {
        let batch_queue = batch_queue.clone();
        let Ok(root) = config.root() else { return ExclusionActionBatch::default() };
        let counter = AtomicUsize::new(0);
        WalkDirGeneric::<(_, ())>::new(root)
            .root_read_dir_state(config)
            .skip_hidden(false)
            .process_read_dir({
                let abort = abort.clone();
                move |_, path, config, children| {
                    // Remove effect-less directories & skips.
                    config.directories.retain(|directory| {
                        path.starts_with(&directory.path) || directory.path.starts_with(path)
                    });
                    config.skips.retain(|skip| skip.starts_with(path));

                    if config.directories.is_empty() || abort.load(Ordering::Relaxed) {
                        // There's no need to go deeper.
                        for child in children.iter_mut().filter_map(|child| child.as_mut().ok()) {
                            child.read_children_path = None;
                        }
                        return;
                    }

                    if counter
                        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |i| {
                            Some(if i > 1000 { 0 } else { i + 1 })
                        })
                        .expect("f never returns None")
                        == 0
                    {
                        if let Err(e) = curr_tx.try_send(path.to_path_buf()) {
                            warn!("Failed to send current path: {}", e);
                        }
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
                    found.fetch_add(diff.count(), Ordering::Relaxed);

                    // Exclude already excluded or uncovered children.
                    for (entry, excluded) in children {
                        let path = entry.path();
                        if (excluded && !diff.remove.contains(&path)) || diff.add.contains(&path) {
                            entry.read_children_path = None;
                        }
                    }
                    batch_queue.push(diff);
                }
            })
            .into_iter()
            .for_each(|_| {});
    }
    if abort.load(Ordering::Relaxed) {
        // Aborted, the return value is irrelevant.
        return ExclusionActionBatch::default();
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
                rule.excludes.contains(name)
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
