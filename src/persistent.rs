use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

use fs3::FileExt;
use fsevent_stream::ffi::{kFSEventStreamEventIdSinceNow, FSEventStreamEventId};
use serde::{Deserialize, Serialize};

use crate::errors::PersistentError;

#[inline]
const fn default_event_id() -> FSEventStreamEventId {
    kFSEventStreamEventIdSinceNow
}

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    #[serde(default = "default_event_id")]
    pub last_event_id: FSEventStreamEventId,
}

pub struct PersistentState {
    f: File,
    state: State,
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
        Ok(Self { f, state })
    }
    /// Mutate state and flush state into file.
    ///
    /// # Errors
    /// `Json` if there's some internal json encode error.
    /// `IO` if file can't be truncated or written.
    pub fn set_with(&mut self, f: impl FnOnce(&mut State)) -> Result<(), PersistentError> {
        f(&mut self.state);
        self.flush()
    }
    fn flush(&mut self) -> Result<(), PersistentError> {
        self.f.set_len(0)?;
        self.f.seek(SeekFrom::Start(0))?;
        serde_json::to_writer(&self.f, &self.state)?;
        Ok(())
    }
}

impl Drop for PersistentState {
    fn drop(&mut self) {
        drop(self.f.unlock());
    }
}
