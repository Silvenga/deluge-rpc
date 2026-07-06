use crate::server::constants::{HEADER_LEN, PROTOCOL_VERSION};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::io;
use std::io::ErrorKind;
use std::io::Write;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;

pub async fn write_frame(tls: &mut TlsStream<TcpStream>, data: &[u8]) -> io::Result<()> {
    let compressed = zlib_compress(data)?;
    let len = u32::try_from(compressed.len())
        .map_err(|_| io::Error::new(ErrorKind::InvalidInput, "payload too large for u32 len"))?;
    let mut frame = Vec::with_capacity(HEADER_LEN + compressed.len());
    frame.push(PROTOCOL_VERSION);
    frame.extend_from_slice(&len.to_be_bytes());
    frame.extend_from_slice(&compressed);
    tls.write_all(&frame).await?;
    tls.flush().await?;
    Ok(())
}

fn zlib_compress(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    encoder.finish()
}
