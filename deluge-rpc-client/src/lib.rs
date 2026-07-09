//! Client library for the Deluge daemon RPC protocol (TLS + framed zlib + rencode).

mod client;
mod error;
mod protocol;
#[cfg(feature = "recorder")]
mod recorder;
mod transport;

pub use client::*;
pub use deluge_rpc_models as models;
pub use deluge_rpc_rencode::{RencodeError, RencodeValue};
pub use error::DelugeRpcError;
pub use protocol::*;
#[cfg(feature = "recorder")]
pub use recorder::*;
pub use transport::*;

pub(crate) use deluge_rpc_rencode::to_rencode_value;
