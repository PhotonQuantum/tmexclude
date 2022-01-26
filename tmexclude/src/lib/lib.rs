//! Support library for tmexclude binary.
#![allow(clippy::module_name_repetitions)]
#![warn(missing_docs)]

extern crate core;

pub mod config;
pub mod daemon;
pub mod errors;
pub mod rpc;
pub mod tmutil;
pub mod walker;
#[doc(hidden)]
pub mod watcher;
