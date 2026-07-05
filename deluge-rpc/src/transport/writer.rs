use crate::transport::constants::{HEADER_LEN, PROTOCOL_VERSION};
use crate::transport::error::TransportError;
use client::TlsStream;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::{self, Write};
use tokio::io::{AsyncWriteExt, WriteHalf};
use tokio::net::TcpStream;
use tokio_rustls::client;

pub struct DelugeWriter {
    write: WriteHalf<TlsStream<TcpStream>>,
}

impl DelugeWriter {
    pub fn new(write: WriteHalf<TlsStream<TcpStream>>) -> Self {
        Self { write }
    }

    pub async fn send(&mut self, data: &[u8]) -> Result<(), TransportError> {
        let compressed = zlib_compress(data)?;
        let len = u32::try_from(compressed.len()).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidInput, "payload too large for u32 len")
        })?;
        let mut frame = Vec::with_capacity(HEADER_LEN + compressed.len());
        frame.push(PROTOCOL_VERSION);
        frame.extend_from_slice(&len.to_be_bytes());
        frame.extend_from_slice(&compressed);
        self.write.write_all(&frame).await?;
        self.write.flush().await?;
        Ok(())
    }
}

fn zlib_compress(data: &[u8]) -> Result<Vec<u8>, TransportError> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    encoder
        .finish()
        .map_err(|e| TransportError::Zlib(e.to_string()))
}
