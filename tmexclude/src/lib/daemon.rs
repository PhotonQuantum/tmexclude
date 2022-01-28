//! Facilities that setup components and maintain states on daemon mode.

use std::time::Duration;

use actix::{Actor, Addr, Context, Handler, Message};
use actix_signal::AddrSignalExt;
use eyre::Report;

use crate::config::{Config, ConfigFactory};
use crate::walker::{SkipCache, Walker};
use crate::watcher::{RegisterWatcher, Watcher};

const EVENT_DELAY: Duration = Duration::from_secs(30);

/// Daemon actor.
pub struct Daemon<F> {
    config_factory: F,
    config: Config,
    handler: Option<Addr<Watcher>>,
}

impl<F> Daemon<F>
where
    F: ConfigFactory,
{
    /// Construct a new daemon actor.
    ///
    /// # Errors
    /// Returns `ConfigError` if fails to load config with given factory.
    pub fn new(deserializer_factory: F) -> Result<Self, Report> {
        let config = deserializer_factory.call()?;
        Ok(Self {
            config_factory: deserializer_factory,
            config,
            handler: None,
        })
    }
}

impl<F> Daemon<F> {
    fn start(&mut self) {
        let walker = Walker::new(self.config.walk.clone(), SkipCache::default());
        let watcher = Watcher::new(self.config.no_include, walker.start());
        let addr = watcher.start();
        addr.do_send(RegisterWatcher {
            paths: self
                .config
                .walk
                .directories
                .iter()
                .map(|directory| directory.path.clone())
                .collect(),
            delay: EVENT_DELAY,
        });

        if let Some(old_handler) = &self.handler {
            old_handler.stop();
        }
        self.handler = Some(addr);
    }
}

impl<F> Daemon<F> {
    fn running(&self) -> bool {
        matches!(&self.handler, Some(addr) if addr.connected())
    }
}

impl<F> Actor for Daemon<F>
where
    F: Unpin + 'static,
{
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        self.start();
    }
}

/// Reload config and restart daemon.
#[derive(Debug, Message)]
#[rtype("Result<(), Report>")]
pub struct Reload;

impl<F> Handler<Reload> for Daemon<F>
where
    F: ConfigFactory,
{
    type Result = Result<(), Report>;

    fn handle(&mut self, _: Reload, _: &mut Self::Context) -> Self::Result {
        self.config = self.config_factory.call()?;
        self.start();
        Ok(())
    }
}

/// Pause daemon.
#[derive(Debug, Message)]
#[rtype("()")]
pub struct Pause;

impl<F> Handler<Pause> for Daemon<F>
where
    F: Unpin + 'static,
{
    type Result = ();

    fn handle(&mut self, _: Pause, _: &mut Self::Context) -> Self::Result {
        if let Some(old_handler) = &self.handler {
            old_handler.stop();
        }
        self.handler = None;
    }
}

/// Restart daemon. This method doesn't reload config.
#[derive(Debug, Message)]
#[rtype("()")]
pub struct Restart;

impl<F> Handler<Restart> for Daemon<F>
where
    F: Unpin + 'static,
{
    type Result = ();

    fn handle(&mut self, _: Restart, _: &mut Self::Context) -> Self::Result {
        self.start();
    }
}

/// Is watcher running?
#[derive(Debug, Message)]
#[rtype("bool")]
pub struct IsRunning;

impl<F> Handler<IsRunning> for Daemon<F>
where
    F: Unpin + 'static,
{
    type Result = bool;

    fn handle(&mut self, _: IsRunning, _: &mut Self::Context) -> Self::Result {
        self.running()
    }
}
