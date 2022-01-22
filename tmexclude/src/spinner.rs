use std::borrow::Cow;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::Duration;

use indicatif::ProgressBar;

pub struct Spinner {
    handle: Option<thread::JoinHandle<()>>,
    tx: Sender<()>,
}

impl Spinner {
    pub fn new(msg: impl Into<Cow<'static, str>> + Send + 'static) -> Self {
        let (tx, rx) = channel();
        let handle = thread::spawn(move || {
            let pb = ProgressBar::new_spinner();
            pb.set_message(msg);

            loop {
                match rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(()) => {
                        pb.finish_and_clear();
                        break;
                    }
                    Err(_) => pb.tick(),
                }
            }
        });
        Self {
            handle: Some(handle),
            tx,
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.tx.send(()).expect("connected");
        if let Some(handle) = self.handle.take() {
            handle.join().expect("no panic");
        };
    }
}
