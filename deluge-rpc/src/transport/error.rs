use std::io;
use thiserror::Error;

/// Errors produced by [`DelugeTransport`].
#[derive(Debug, Error)]
pub enum TransportError {
    #[error("TLS connection error: {0}")]
    Tls(#[from] rustls::Error),
    #[error("TCP connection error: {0}")]
    Io(#[from] io::Error),
    #[error("protocol version mismatch: expected 1, got {0}")]
    ProtocolVersion(u8),
    #[error("unexpected end of stream")]
    UnexpectedEof,
    #[error("zlib decompression error: {0}")]
    Zlib(String),
}
