use crate::server::constants::{HEADER_LEN, PROTOCOL_VERSION};
use flate2::read::ZlibDecoder;
use std::io;
use std::io::ErrorKind;
use std::io::Read;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;

pub enum ReadFrameError {
    Eof,
    Io(io::Error),
}

pub async fn read_frame(tls: &mut TlsStream<TcpStream>) -> Result<Vec<u8>, ReadFrameError> {
    let mut header = [0u8; HEADER_LEN];
    match tls.read_exact(&mut header).await {
        Ok(_) => {}
        Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Err(ReadFrameError::Eof),
        Err(e) => return Err(ReadFrameError::Io(e)),
    }
    if header[0] != PROTOCOL_VERSION {
        return Err(ReadFrameError::Io(io::Error::new(
            ErrorKind::InvalidData,
            "protocol version mismatch",
        )));
    }
    let body_len = u32::from_be_bytes([header[1], header[2], header[3], header[4]]);
    let mut body = vec![
        0u8;
        usize::try_from(body_len).map_err(|_| {
            ReadFrameError::Io(io::Error::new(
                ErrorKind::InvalidData,
                "body length overflow",
            ))
        })?
    ];
    match tls.read_exact(&mut body).await {
        Ok(_) => {}
        Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Err(ReadFrameError::Eof),
        Err(e) => return Err(ReadFrameError::Io(e)),
    }
    zlib_decompress(&body)
        .map_err(|e| ReadFrameError::Io(io::Error::new(ErrorKind::InvalidData, e)))
}

fn zlib_decompress(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut decoder = ZlibDecoder::new(data);
    let mut out = Vec::new();
    decoder.read_to_end(&mut out).map_err(|e| e.to_string())?;
    Ok(out)
}
