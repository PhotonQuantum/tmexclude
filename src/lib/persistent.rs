use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;
use std::time::Duration;

use actix::dev::{MessageResponse, OneshotSender};
use actix::{Actor, AsyncContext, Context, Handler, Message};
use fs3::FileExt;
use fsevent_stream::ffi::{kFSEventStreamEventIdSinceNow, FSEventStreamEventId};
use log::{debug, error};
use serde::{Deserialize, Serialize};

use crate::errors::PersistentError;

const COMMIT_INTERVAL: Duration = Duration::from_secs(30);

#[inline]
const fn default_event_id() -> FSEventStreamEventId {
    kFSEventStreamEventIdSinceNow
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct State {
    #[serde(default = "default_event_id")]
    pub last_event_id: FSEventStreamEventId,
}

impl<A, M> MessageResponse<A, M> for State
where
    A: Actor,
    M: Message<Result = Self>,
{
    fn handle(self, _ctx: &mut A::Context, tx: Option<OneshotSender<Self>>) {
        if let Some(tx) = tx {
            let _ = tx.send(self);
        }
    }
}

pub struct PersistentState {
    f: File,
    state: State,
    dirty: bool,
}

impl Actor for PersistentState {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(COMMIT_INTERVAL, |act, _| {
            if let Err(e) = act.flush() {
                error!("Error when flushing persistent state to disk: {}", e);
            }
        });
    }
}

/// Get the state.
#[derive(Message)]
#[rtype("State")]
pub struct GetState;

impl Handler<GetState> for PersistentState {
    type Result = State;

    fn handle(&mut self, _msg: GetState, _ctx: &mut Self::Context) -> Self::Result {
        self.state
    }
}

/// Mutate state and flush state to file.
///
/// # Errors
/// `Json` if there's some internal json encode error.
/// `IO` if file can't be truncated or written.
#[derive(Message)]
#[rtype("()")]
pub struct SetStateWith<F: FnMut(&mut State)>(pub F);

impl<F> Handler<SetStateWith<F>> for PersistentState
where
    F: FnMut(&mut State),
{
    type Result = ();

    fn handle(&mut self, mut msg: SetStateWith<F>, _ctx: &mut Self::Context) -> Self::Result {
        (msg.0)(&mut self.state);
        self.dirty = true;
    }
}

impl PersistentState {
    #[must_use]
    pub const fn state(&self) -> &State {
        &self.state
    }
}

impl PersistentState {
    /// Load a persistent state from given path.
    ///
    /// # Errors
    /// `Json` if given file isn't a valid json file.
    /// `IO` if file can't be opened or locked.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, PersistentError> {
        let mut f = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(path)?;
        f.lock_exclusive()?;
        if f.metadata()?.len() == 0 {
            write!(f, "{{}}")?;
            f.seek(SeekFrom::Start(0))?;
        }
        let state = serde_json::from_reader(&f)?;
        Ok(Self {
            f,
            state,
            dirty: false,
        })
    }
    fn flush(&mut self) -> Result<(), PersistentError> {
        if !self.dirty {
            return Ok(());
        }

        debug!("flushing persistent state to disk");
        self.f.set_len(0)?;
        self.f.seek(SeekFrom::Start(0))?;
        serde_json::to_writer(&self.f, &self.state)?;
        self.dirty = false;

        Ok(())
    }
}

impl Drop for PersistentState {
    fn drop(&mut self) {
        drop(self.f.unlock());
    }
}
