use crate::transport::constants::{HEADER_LEN, MAX_FRAME_SIZE, PROTOCOL_VERSION};
use crate::transport::error::TransportError;
use client::TlsStream;
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use std::io::{self, Read, Write};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio_rustls::client;

pub struct DelugeReader {
    read: ReadHalf<TlsStream<TcpStream>>,
}

impl DelugeReader {
    pub(crate) fn new(read: ReadHalf<TlsStream<TcpStream>>) -> Self {
        Self { read }
    }

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

pub struct DelugeWriter {
    write: WriteHalf<TlsStream<TcpStream>>,
}

impl DelugeWriter {
    pub(crate) fn new(write: WriteHalf<TlsStream<TcpStream>>) -> Self {
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

fn zlib_decompress(data: &[u8]) -> Result<Vec<u8>, TransportError> {
    let mut decoder = ZlibDecoder::new(data);
    let mut out = Vec::new();
    match decoder.read_to_end(&mut out) {
        Ok(_) => Ok(out),
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => Err(TransportError::UnexpectedEof),
        Err(e) => Err(TransportError::Zlib(e.to_string())),
    }
}
