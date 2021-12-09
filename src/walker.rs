use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crossbeam_queue::SegQueue;
use itertools::Itertools;
use jwalk::WalkDirGeneric;
use log::warn;
use tap::TapFallible;

use crate::config::Directory;
use crate::tmutil::{ExclusionAction, ExclusionActionBatch};

fn generate_diff<'a, 'b>(
    cwd: &'a Path,
    shallow_list: &'a HashMap<PathBuf, bool>,
    directories: impl IntoIterator<Item = &'b Directory>,
) -> ExclusionActionBatch {
    let candidate_rules = directories
        .into_iter()
        .filter(|directory| directory.path.starts_with(cwd) || cwd.starts_with(&directory.path))
        .flat_map(|directory| &directory.rules)
        .collect_vec();
    shallow_list
        .iter()
        .filter_map(|(name, excluded)| {
            let expected_excluded = candidate_rules.iter().any(|rule| {
                (&rule.excludes).contains(name)
                    && rule
                        .if_exists
                        .iter()
                        .any(|if_exist| shallow_list.contains_key(if_exist.as_path()))
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
    directories: impl Into<Cow<'b, [Directory]>>, // impl IntoIterator<Item=&'b Directory>,
    recursive: bool,
) -> ExclusionActionBatch {
    let root = root.as_ref();
    if recursive {
        let directories = directories.into().into_owned();
        let batch_queue = Arc::new(SegQueue::new());
        {
            let batch_queue = batch_queue.clone();
            WalkDirGeneric::<(Vec<Directory>, ())>::new(root)
                .root_read_dir_state(directories)
                .skip_hidden(false)
                .process_read_dir(move |_, path, directories, children| {
                    // Remove effect-less directories.
                    directories.retain(|directory| {
                        path.starts_with(&directory.path) || directory.path.starts_with(path)
                    });
                    if directories.is_empty() {
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
                        })
                        .collect_vec();

                    // Generate diff.
                    let shallow_list: HashMap<_, _> = children
                        .iter()
                        .map(|(path, excluded)| {
                            (PathBuf::from(path.file_name().to_os_string()), *excluded)
                        })
                        .collect();
                    let diff = generate_diff(path, &shallow_list, &*directories);

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
    } else {
        match fs::read_dir(root) {
            Ok(dir) => {
                let shallow_list: HashMap<_, _> = dir
                    .filter_map(|entry| {
                        entry
                            .tap_err(|e| warn!("Error when scanning dir {:?}: {}", root, e))
                            .ok()
                    })
                    .map(|entry| {
                        let path = entry.path();
                        let excluded =
                            xattr::get(&path, "com.apple.metadata:com_apple_backup_excludeItem")
                                .tap_err(|e| {
                                    warn!("Error when querying xattr of file {:?}: {}", path, e);
                                })
                                .ok()
                                .is_some();
                        (
                            PathBuf::from(path.file_name().expect("file name").to_os_string()),
                            excluded,
                        )
                    })
                    .collect();
                generate_diff(root, &shallow_list, &*directories.into())
            }
            Err(e) => {
                warn!("Error when scanning dir {:?}: {}", root, e);
                ExclusionActionBatch::default()
            }
        }
    }
}
