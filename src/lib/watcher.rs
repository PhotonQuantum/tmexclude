//! Filesystem watcher.
use std::collections::HashSet;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use actix::{
    Actor, ActorFutureExt, Addr, AsyncContext, Context, Handler, Message, ResponseActFuture,
    SpawnHandle, StreamHandler, WrapFuture,
};
use atomic::Atomic;
use fsevent_stream::ffi::{kFSEventStreamCreateFlagIgnoreSelf, kFSEventStreamEventIdSinceNow};
use fsevent_stream::flags::StreamFlags;
use fsevent_stream::stream::{create_event_stream, Event, EventStreamHandler};
use itertools::Itertools;
use log::{info, warn};

use crate::config::ApplyMode;
use crate::persistent::{GetState, PersistentState, SetStateWith};
use crate::walker::{Walk, Walker};

/// Filesystem watcher actor.
pub struct Watcher {
    apply_mode: Arc<Atomic<ApplyMode>>,
    handler: Option<(SpawnHandle, EventStreamHandler)>,
    historical_path: Option<HashSet<PathBuf>>,
    state: Addr<PersistentState>,
    walker: Addr<Walker>,
}

impl Watcher {
    /// Create a new watcher actor instance.
    #[must_use]
    pub fn new(
        apply_mode: Arc<Atomic<ApplyMode>>,
        state: Addr<PersistentState>,
        walker: Addr<Walker>,
    ) -> Self {
        Self {
            apply_mode,
            handler: None,
            historical_path: None,
            state,
            walker,
        }
    }
}

impl Actor for Watcher {
    type Context = Context<Self>;

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        if let Some((_, event_handle)) = &mut self.handler {
            event_handle.abort();
        }
    }
}

/// Register paths to the watcher. Former registered paths will be overwritten.
///
/// # Errors
///
/// Return io error if there are invalid paths in `paths` argument.
#[derive(Debug, Message)]
#[rtype("std::io::Result<()>")]
pub struct RegisterWatcher<I> {
    /// Paths to be registered to the watcher
    pub paths: I,
    /// Batch delay for filesystem events.
    pub interval: Duration,
}

impl<I, P> Handler<RegisterWatcher<I>> for Watcher
where
    I: IntoIterator<Item = P> + 'static,
    P: AsRef<Path>,
{
    type Result = ResponseActFuture<Self, std::io::Result<()>>;

    fn handle(&mut self, msg: RegisterWatcher<I>, _ctx: &mut Self::Context) -> Self::Result {
        Box::pin(
            self.state
                .send(GetState)
                .into_actor(self)
                .map(|state, _, _| {
                    state
                        .map(|state| state.last_event_id)
                        .map_err(|e| Error::new(ErrorKind::ConnectionAborted, e))
                })
                .map(move |last_event_id, act, ctx| {
                    let last_event_id = last_event_id?;
                    let (stream, event_handle) = create_event_stream(
                        msg.paths,
                        last_event_id,
                        msg.interval,
                        kFSEventStreamCreateFlagIgnoreSelf,
                    )?;

                    if let Some((spawn_handle, mut event_handle)) = act.handler.take() {
                        event_handle.abort();
                        ctx.cancel_future(spawn_handle);
                    }

                    act.historical_path = if last_event_id == kFSEventStreamEventIdSinceNow {
                        None
                    } else {
                        info!("Start processing historical events");
                        Some(HashSet::new())
                    };
                    let spawn_handle = ctx.add_stream(stream);
                    act.handler = Some((spawn_handle, event_handle));

                    Ok(())
                }),
        )
    }
}

enum HistoryState {
    /// There are still preceding historical events.
    Pending,
    /// There's no more historical events.
    Finished(HashSet<PathBuf>),
}

enum ConsumeState {
    /// Watcher is in history handling mode.
    History(HistoryState),
    /// Watcher is in immediate mode.
    Immediate(HashSet<PathBuf>),
}

fn consume_event_batch(
    historical_events: &mut Option<HashSet<PathBuf>>,
    events: Vec<Event>,
) -> ConsumeState {
    match historical_events {
        None => ConsumeState::Immediate(events.into_iter().map(|item| item.path).collect()),
        Some(historical_events) => {
            let mut item_iter = events.into_iter().peekable();
            historical_events.extend(
                item_iter
                    .peeking_take_while(|event| !event.flags.contains(StreamFlags::HISTORY_DONE))
                    .filter(|event| {
                        if event.flags.contains(StreamFlags::MUST_SCAN_SUBDIRS) {
                            warn!(
                                "System report must scan subdirs of {:?}, ignored.",
                                event.path
                            );
                            false
                        } else {
                            true
                        }
                    })
                    .map(|event| event.path),
            );
            let history_done = item_iter.next().map_or(false, |event| {
                event.flags.contains(StreamFlags::HISTORY_DONE)
            });
            ConsumeState::History(if history_done {
                info!("Historical events done");
                HistoryState::Finished(item_iter.map(|item| item.path).collect::<HashSet<_>>())
            } else {
                HistoryState::Pending
            })
        }
    }
}

impl StreamHandler<Vec<Event>> for Watcher {
    fn handle(&mut self, item: Vec<Event>, _ctx: &mut Self::Context) {
        if let Some(max_event_id) = item
            .iter()
            .max_by_key(|event| event.id)
            .map(|event| event.id)
        {
            self.state.do_send(SetStateWith(move |state| {
                state.last_event_id = max_event_id;
            }));
        }

        let apply = self.apply_mode.load(Ordering::Relaxed);
        match consume_event_batch(&mut self.historical_path, item) {
            ConsumeState::History(HistoryState::Finished(remaining_events)) => {
                for path in self.historical_path.take().expect("to exist") {
                    self.walker.do_send(Walk {
                        root: path,
                        recursive: false,
                        apply,
                    });
                }
                for path in remaining_events {
                    self.walker.do_send(Walk {
                        root: path,
                        recursive: false,
                        apply,
                    });
                }
            }
            ConsumeState::History(HistoryState::Pending) => {} // no-op
            ConsumeState::Immediate(events) => {
                for path in events {
                    self.walker.do_send(Walk {
                        root: path,
                        recursive: false,
                        apply,
                    });
                }
            }
        };
    }

    fn finished(&mut self, _ctx: &mut Self::Context) {} // prevent the actor from being shutdown.
}
