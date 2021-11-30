use std::path::PathBuf;
use std::sync::Arc;

use futures::{Stream, StreamExt};
use log::{debug, error};
use notify::event::{ModifyKind, RenameMode};
use notify::{
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Result, Watcher as NotifyWatcher,
};
use parking_lot::RwLock;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::config::Directory;
use crate::tmutil::ExclusionAction;

pub struct Watcher {
    // to be converted into stream
    rx: Option<UnboundedReceiver<Event>>,
    watcher: RecommendedWatcher,
    directories: Arc<RwLock<Vec<Directory>>>,
}

enum MatchKind {
    IfExists,
    Excludes,
}

impl Watcher {
    pub fn new() -> Result<Self> {
        let (tx, rx) = unbounded_channel();
        notify::recommended_watcher(move |ev: Result<Event>| match ev {
            Ok(ev) => tx.send(ev).expect("channel closed"),
            Err(e) => error!("{}", e),
        })
        .and_then(move |mut watcher| {
            watcher
                .configure(Config::PreciseEvents(true))
                .map(move |_| Self {
                    rx: Some(rx),
                    watcher,
                    directories: Default::default(),
                })
        })
    }
    pub fn register_directory(&mut self, directory: Directory) -> Result<()> {
        self.watcher
            .watch(directory.path.as_path(), RecursiveMode::Recursive)
            .map(|_| {
                self.directories.write().push(directory.clone());
            })
    }
    pub fn take_stream(&mut self) -> impl Stream<Item = ExclusionAction> {
        let directories = self.directories.clone();
        UnboundedReceiverStream::new(self.rx.take().expect("stream gone")).flat_map(move |ev| {
            debug!("raw event: {:?}", ev);
            tokio_stream::iter(match_rule(&*directories.read(), ev))
        })
    }
}

fn action_when_appear<'a>(
    directories: impl IntoIterator<Item = &'a Directory>,
    path: PathBuf,
) -> Vec<ExclusionAction> {
    // Something is created, we first check which directory it's in.
    // When there are multiple choices, we choose the deepest one.
    // We can do this because directory.path is canonical.
    directories
        .into_iter()
        .filter(|directory| path.starts_with(&directory.path))
        .max_by_key(|directory| directory.path.components().count())
        // Now we consider if the file hits any rule defined under this directory.
        .and_then(|directory: &Directory| {
            directory
                .rules
                .iter()
                .filter_map(|rule| {
                    // There are two cases we need to deal with:
                    // 1. An if-exists object is created.
                    // 2. An excluded object is created.
                    if (&rule.excludes)
                        .iter()
                        .any(|exclude| path.ends_with(exclude))
                    {
                        Some((rule, MatchKind::Excludes))
                    } else if rule
                        .if_exists
                        .iter()
                        .any(|if_exist| path.ends_with(if_exist))
                    {
                        Some((rule, MatchKind::IfExists))
                    } else {
                        None
                    }
                })
                .next() // We just need to consider the first match.
        })
        .map(|(rule, kind)| {
            let parent = path.parent().expect("parent path");
            match kind {
                MatchKind::IfExists => {
                    // If any excludes object exists we need to exclude them.
                    (&rule.excludes)
                        .iter()
                        .filter(|exclude| parent.join(exclude).exists())
                        .filter_map(|exclude| exclude.canonicalize().ok())
                        .map(ExclusionAction::Add)
                        .collect()
                }
                MatchKind::Excludes => {
                    // If all if-exists objects exist we need to exclude this object.
                    (&rule.if_exists)
                        .iter()
                        .all(|if_exist| parent.join(if_exist).exists())
                        .then(|| path)
                        .map(ExclusionAction::Add)
                        .into_iter()
                        .collect()
                }
            }
        })
        .unwrap_or_else(Vec::new)
}

pub fn match_rule<'a>(
    directories: impl IntoIterator<Item = &'a Directory>,
    event: Event,
) -> Vec<ExclusionAction> {
    match event.kind {
        EventKind::Create(_) => {
            let path = event
                .paths
                .first()
                .expect("missing path in watcher message");
            action_when_appear(directories, path.clone())
        }
        EventKind::Modify(ModifyKind::Name(mode)) => {
            // match mode {
            //     RenameMode::To => {}
            //     RenameMode::From => {}
            //     RenameMode::Both => {}
            //     RenameMode::Other => {}
            // }
            vec![]
        }
        EventKind::Remove(_) => {
            // This is just the co-op of creation.
            let path = event
                .paths
                .first()
                .expect("missing path in watcher message");
            // TODO BUG This might cause problems when deleting a folder
            // Consider the following scenario:
            // User deletes the whole project folder. A sequence of DEL(target), ... has come.
            // We check that if Cargo.toml exists to avoid non-sense exclude.
            // However at this time Cargo.toml has gone. We exclude no directory.
            // Possible solution:
            // Exclude the to-be-exclude object blindly.
            // Alternative:
            // The whole procedure might be totally non-sense because it seems that tmutil marks the
            // no-backup flag on xattrs, and it's gone when file gets deleted. So we just take
            // measures when if-exists files get deleted, and this won't cause any problem.
            action_when_appear(directories, path.clone())
                .into_iter()
                .map(ExclusionAction::flip)
                .collect()
        }
        _ => vec![],
    }
}
