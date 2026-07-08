use std::io;
use thiserror::Error;

/// Errors produced by the transport layer.
#[derive(Debug, Error)]
pub enum TransportError {
    /// A TLS connection error.
    #[error("TLS connection error: {0}")]
    Tls(#[from] rustls::Error),
    /// A TCP connection error.
    #[error("TCP connection error: {0}")]
    Io(#[from] io::Error),
    /// The protocol version byte in the frame header did not match the expected value (1).
    #[error("protocol version mismatch: expected 1, got {0}")]
    ProtocolVersion(u8),
    /// The stream ended before the full frame could be read.
    #[error("unexpected end of stream")]
    UnexpectedEof,
    /// A zlib compression or decompression error.
    #[error("zlib decompression error: {0}")]
    Zlib(String),
}
