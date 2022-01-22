//! Filesystem watcher.

use std::path::PathBuf;
use std::time::Duration;

use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, SpawnHandle, StreamHandler};
use actix_signal::SignalHandler;
use fsevent_stream::ffi::{kFSEventStreamCreateFlagIgnoreSelf, kFSEventStreamEventIdSinceNow};
use fsevent_stream::stream::{create_event_stream, Event, EventStreamHandler};

use crate::config::ApplyMode;
use crate::walker::{Walk, Walker};

/// Filesystem watcher actor.
#[derive(SignalHandler)]
pub struct Watcher {
    apply_mode: ApplyMode,
    handler: Option<(SpawnHandle, EventStreamHandler)>,
    walker: Addr<Walker>,
}

impl Watcher {
    /// Create a new watcher actor instance.
    #[must_use]
    pub const fn new(apply_mode: ApplyMode, walker: Addr<Walker>) -> Self {
        Self {
            apply_mode,
            handler: None,
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
pub struct RegisterWatcher {
    /// Paths to be registered to the watcher
    pub paths: Vec<PathBuf>,
    /// Batch delay for filesystem events.
    pub delay: Duration,
}

impl Handler<RegisterWatcher> for Watcher {
    type Result = std::io::Result<()>;

    fn handle(&mut self, msg: RegisterWatcher, ctx: &mut Self::Context) -> Self::Result {
        // TODO exclude path (need modify fsevent_stream)
        let (stream, event_handle) = create_event_stream(
            msg.paths,
            kFSEventStreamEventIdSinceNow,
            msg.delay,
            kFSEventStreamCreateFlagIgnoreSelf,
        )?;

        if let Some((spawn_handle, mut event_handle)) = self.handler.take() {
            event_handle.abort();
            ctx.cancel_future(spawn_handle);
        }

        let spawn_handle = ctx.add_stream(stream.into_flatten());
        self.handler = Some((spawn_handle, event_handle));

        Ok(())
    }
}

impl StreamHandler<Event> for Watcher {
    fn handle(&mut self, item: Event, _ctx: &mut Self::Context) {
        if !item.path.as_os_str().is_empty() {
            self.walker.do_send(Walk {
                root: item.path,
                recursive: false,
                apply: self.apply_mode,
            });
        }
    }

    fn finished(&mut self, _ctx: &mut Self::Context) {} // prevent the actor from being shutdown.
}
