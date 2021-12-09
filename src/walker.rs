use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crossbeam_queue::SegQueue;
use itertools::Itertools;
use jwalk::WalkDirGeneric;
use log::warn;
use tap::TapFallible;

use crate::config::{ConfigView, Directory, Rule};
use crate::tmutil::{ExclusionAction, ExclusionActionBatch};
use crate::Config;

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

/// Walk through a directory with given rules and return a exclusion action plan.
#[must_use]
pub fn walk<'a, 'b>(
    root: impl AsRef<Path> + 'a,
    config: impl Into<ConfigView<'b>>,
    recursive: bool,
) -> ExclusionActionBatch {
    let root = root.as_ref();
    let config = config.into();
    if recursive {
        walk_recursive(root, config)
    } else {
        walk_non_recursive(root, &config)
    }
}

fn walk_recursive(root: &Path, config: ConfigView) -> ExclusionActionBatch {
    let batch_queue = Arc::new(SegQueue::new());
    {
        let batch_queue = batch_queue.clone();
        WalkDirGeneric::<(Config, ())>::new(root)
            .root_read_dir_state(config.into_owned())
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
                            let excluded = xattr::get(
                                &path,
                                "com.apple.metadata:com_apple_backup_excludeItem",
                            )
                            .tap_err(|e| {
                                warn!("Error when querying xattr of file {:?}: {}", path, e);
                            })
                            .ok()?
                            .is_some();
                            Some((entry, excluded))
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

fn walk_non_recursive(root: &Path, config: &ConfigView) -> ExclusionActionBatch {
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
                    if (&*config.skips).contains(&path) {
                        // Skip this entry in all preceding procedures and scans.
                        None
                    } else {
                        let excluded =
                            xattr::get(&path, "com.apple.metadata:com_apple_backup_excludeItem")
                                .tap_err(|e| {
                                    warn!("Error when querying xattr of file {:?}: {}", path, e);
                                })
                                .ok()?
                                .is_some();
                        Some((
                            PathBuf::from(path.file_name().expect("file name").to_os_string()),
                            excluded,
                        ))
                    }
                })
                .collect();
            generate_diff(root, &shallow_list, &*config.directories)
        }
        Err(e) => {
            warn!("Error when scanning dir {:?}: {}", root, e);
            ExclusionActionBatch::default()
        }
    }
}
