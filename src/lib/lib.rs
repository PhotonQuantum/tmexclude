//! Support library for tmexclude binary.
#![allow(clippy::module_name_repetitions)]
#![warn(missing_docs)]

pub mod config;
pub mod daemon;
pub mod errors;
pub mod rpc;
mod tmutil;
pub mod utils;
pub mod walker;
#[doc(hidden)]
pub mod watcher;
