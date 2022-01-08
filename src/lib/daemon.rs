//! Facilities that setup components and maintain states on daemon mode.
use std::error::Error;

use actix::{Actor, Addr, Context, Handler, Message};
use actix_signal::AddrSignalExt;
use figment::Provider;

use crate::config::Config;
use crate::errors::ConfigError;
use crate::utils::TypeEq;
use crate::walker::{SkipCache, Walker};
use crate::watcher::{RegisterWatcher, Watcher};

/// Daemon actor.
pub struct Daemon<F> {
    provider_factory: F,
    config: Config,
    handler: Option<Addr<Watcher>>,
}

impl<F> Daemon<F> {
    /// Construct a new daemon actor.
    ///
    /// # Errors
    /// Returns `ConfigError` if fails to load config with given factory.
    pub fn new<P, O, E>(provider_factory: F) -> Result<Self, ConfigError>
    where
        F: Fn() -> O + Unpin + 'static,
        O: TypeEq<Rhs = Result<P, E>>,
        E: 'static + Error + Send + Sync,
        P: Provider,
    {
        let provider = (provider_factory)()
            .cast()
            .map_err(|e| ConfigError::Factory(e.into()))?;
        let config = Config::from(provider)?;
        Ok(Self {
            provider_factory,
            config,
            handler: None,
        })
    }

    fn start(&mut self) {
        let walker = Walker::new(self.config.walk.clone(), SkipCache::default());
        let watcher = Watcher::new(self.config.mode, walker.start());
        let addr = watcher.start();
        addr.do_send(RegisterWatcher {
            paths: self
                .config
                .walk
                .directories
                .iter()
                .map(|directory| directory.path.clone())
                .collect(),
            interval: self.config.interval.watch,
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
#[rtype("Result<(), ConfigError>")]
pub struct Reload;

impl<F, O, E, P> Handler<Reload> for Daemon<F>
where
    F: Fn() -> O + Unpin + 'static,
    O: TypeEq<Rhs = Result<P, E>>,
    E: 'static + Error + Send + Sync,
    P: Provider,
{
    type Result = Result<(), ConfigError>;

    fn handle(&mut self, _: Reload, _: &mut Self::Context) -> Self::Result {
        if self.running() {
            let provider = (self.provider_factory)()
                .cast()
                .map_err(|e| ConfigError::Factory(e.into()))?;
            self.config = Config::from(provider)?;
            self.start();
        }
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
