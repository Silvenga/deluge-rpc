use crate::transport::constants::{HEADER_LEN, MAX_FRAME_SIZE, PROTOCOL_VERSION};
use crate::transport::error::TransportError;
use client::TlsStream;
use flate2::read::ZlibDecoder;
use std::io::{self, Read};
use tokio::io::{AsyncReadExt, ReadHalf};
use tokio::net::TcpStream;
use tokio_rustls::client;

/// The read half of a Deluge transport, reading framed zlib-compressed messages.
pub struct DelugeReader {
    read: ReadHalf<TlsStream<TcpStream>>,
}

impl DelugeReader {
    /// Create a new `DelugeReader` from a TLS stream read half.
    pub fn new(read: ReadHalf<TlsStream<TcpStream>>) -> Self {
        Self { read }
    }

    /// Read a framed zlib-compressed message from the transport.
    pub async fn recv(&mut self) -> Result<Vec<u8>, TransportError> {
        let mut header = [0u8; HEADER_LEN];
        self.read.read_exact(&mut header).await?;
        if header[0] != PROTOCOL_VERSION {
            return Err(TransportError::ProtocolVersion(header[0]));
        }
        let body_len = u32::from_be_bytes([header[1], header[2], header[3], header[4]]);
        let body_len = usize::try_from(body_len)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "body length overflow"))?;
        if body_len > MAX_FRAME_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("frame too large: {body_len} bytes (max {MAX_FRAME_SIZE})"),
            )
            .into());
        }
        let mut body = vec![0u8; body_len];
        self.read.read_exact(&mut body).await?;
        zlib_decompress(&body)
    }
}

fn zlib_decompress(data: &[u8]) -> Result<Vec<u8>, TransportError> {
    let mut decoder = ZlibDecoder::new(data);
    let mut out = Vec::new();
    match decoder.read_to_end(&mut out) {
        Ok(_) => Ok(out),
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => Err(TransportError::UnexpectedEof),
        Err(e) => Err(TransportError::Zlib(e.to_string())),
    }
}
