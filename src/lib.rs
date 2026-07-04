//! Library facade exposing internal modules for integration tests.
//!
//! The binary (`src/main.rs`) is the real entry point. This lib target exists
//! so integration tests under `tests/` can link against the crate's internal
//! modules (notably `rencode`, `client`, `torrent`) which are private in a
//! pure-binary crate.

#![allow(
    unfulfilled_lint_expectations,
    reason = "src modules carry #[expect(dead_code)] that is fulfilled only in the binary target; the lib re-exports make those items reachable so the expectations go unfulfilled here"
)]
#![allow(
    clippy::missing_errors_doc,
    reason = "internal modules exposed for integration tests; not a published API surface"
)]
#![allow(
    clippy::must_use_candidate,
    reason = "internal modules exposed for integration tests; not a published API surface"
)]

pub mod cli;
pub mod client;
pub mod config;
pub mod engine;
pub mod rencode;
pub mod torrent;
pub mod tracing_setup;
pub mod transport;

pub use client::{DelugeClient, DelugeRpc};
pub use config::{Config, HostConfig, Rules};
pub use rencode::{decode, encode, RencodeValue};
pub use torrent::TorrentInfo;