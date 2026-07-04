//! TLS transport + wire framing for the Deluge daemon RPC protocol.
//!
//! The Deluge daemon listens on TCP port 58846 and speaks a framed,
//! zlib-compressed rencode protocol over TLS. The wire format is:
//!
//! ```text
//! +----+------------------+-----------------------------+
//! | v  | body_len : u32   | zlib-compressed rencode     |
//! | 1B | big-endian (4B)  | payload (body_len bytes)    |
//! +----+------------------+-----------------------------+
//! ```
//!
//! `v` is the protocol version (always `1` for Deluge 2.x). The daemon
//! uses self-signed certificates, so certificate verification is always
//! skipped (a hard requirement, not a configurable option).

use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{ClientConfig, DigitallySignedStruct, SignatureScheme};
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Read, Write};
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;

/// Wire protocol version (Deluge 2.x).
const PROTOCOL_VERSION: u8 = 1;
/// Fixed header size: 1 byte version + 4 bytes big-endian body length.
const HEADER_LEN: usize = 5;
/// Maximum allowed frame body size to prevent OOM from malicious daemons.
const MAX_FRAME_SIZE: usize = 16 * 1024 * 1024; // 16 MiB

/// Errors produced by [`DelugeTransport`].
#[derive(Debug, Error)]
pub enum TransportError {
    #[error("TLS connection error: {0}")]
    Tls(#[from] rustls::Error),
    #[error("TCP connection error: {0}")]
    Io(#[from] IoError),
    #[error("protocol version mismatch: expected 1, got {0}")]
    ProtocolVersion(u8),
    #[error("unexpected end of stream")]
    UnexpectedEof,
    #[error("zlib decompression error: {0}")]
    Zlib(String),
}

/// A TLS connection to the Deluge daemon carrying framed rencode payloads.
pub struct DelugeTransport {
    stream: TlsStream<TcpStream>,
}

/// No-op certificate verifier — accepts any server certificate.
///
/// The Deluge daemon uses self-signed certificates, so verification is
/// intentionally skipped. This is a hard requirement, not a config option.
#[derive(Debug)]
struct NoVerifier;

impl ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::ED25519,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
            SignatureScheme::ECDSA_NISTP521_SHA512,
            SignatureScheme::RSA_PSS_SHA512,
        ]
    }
}

/// Build a [`ClientConfig`] that skips certificate verification.
fn client_config() -> ClientConfig {
    ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(NoVerifier))
        .with_no_client_auth()
}

/// Install the `ring` crypto provider as the process default once.
fn ensure_crypto_provider() {
    use std::sync::Once;
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        if let Err(e) = crypto::ring::default_provider().install_default() {
            tracing::warn!(error = ?e, "failed to install ring crypto provider; TLS may use an unexpected backend");
        }
    });
}

/// Resolve a host string into a rustls [`ServerName`], handling both DNS
/// names and IP addresses.
fn server_name(host: &str) -> Result<ServerName<'static>, rustls::Error> {
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Ok(ServerName::IpAddress(ip.into()));
    }
    ServerName::try_from(host.to_owned())
        .map_err(|e| rustls::Error::from(rustls::OtherError(Arc::new(e))))
}

impl DelugeTransport {
    /// Connect to the Deluge daemon at `host:port` over TLS, skipping
    /// certificate verification.
    pub async fn connect(host: &str, port: u16) -> Result<Self, TransportError> {
        ensure_crypto_provider();
        let config = client_config();
        let connector = TlsConnector::from(Arc::new(config));
        let domain = server_name(host)?;
        let tcp = timeout(Duration::from_secs(10), TcpStream::connect((host, port)))
            .await
            .map_err(|_| IoError::new(IoErrorKind::TimedOut, "TCP connect timed out"))??;
        let stream = timeout(Duration::from_secs(10), connector.connect(domain, tcp))
            .await
            .map_err(|_| IoError::new(IoErrorKind::TimedOut, "TLS handshake timed out"))??;
        Ok(Self { stream })
    }

    /// Send a rencode payload: zlib-compress, prepend the 5-byte header,
    /// and write to the TLS stream.
    pub async fn send(&mut self, data: &[u8]) -> Result<(), TransportError> {
        let compressed = zlib_compress(data)?;
        let len = u32::try_from(compressed.len()).map_err(|_| {
            IoError::new(IoErrorKind::InvalidInput, "payload too large for u32 len")
        })?;
        let mut frame = Vec::with_capacity(HEADER_LEN + compressed.len());
        frame.push(PROTOCOL_VERSION);
        frame.extend_from_slice(&len.to_be_bytes());
        frame.extend_from_slice(&compressed);
        self.stream.write_all(&frame).await?;
        self.stream.flush().await?;
        Ok(())
    }

    /// Receive a framed rencode payload: read the 5-byte header, read the
    /// zlib-compressed body, decompress, and return the rencode bytes.
    pub async fn recv(&mut self) -> Result<Vec<u8>, TransportError> {
        let mut header = [0u8; HEADER_LEN];
        self.stream.read_exact(&mut header).await?;
        if header[0] != PROTOCOL_VERSION {
            return Err(TransportError::ProtocolVersion(header[0]));
        }
        let body_len = u32::from_be_bytes([header[1], header[2], header[3], header[4]]);
        let body_len = usize::try_from(body_len)
            .map_err(|_| IoError::new(IoErrorKind::InvalidInput, "body length overflow"))?;
        if body_len > MAX_FRAME_SIZE {
            return Err(IoError::new(
                IoErrorKind::InvalidInput,
                format!("frame too large: {body_len} bytes (max {MAX_FRAME_SIZE})"),
            )
            .into());
        }
        let mut body = vec![0u8; body_len];
        self.stream.read_exact(&mut body).await?;
        zlib_decompress(&body)
    }
}

/// zlib-compress `data` (zlib wrapper, not gzip/deflate).
fn zlib_compress(data: &[u8]) -> Result<Vec<u8>, TransportError> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    encoder
        .finish()
        .map_err(|e| TransportError::Zlib(e.to_string()))
}

/// zlib-decompress `data`.
fn zlib_decompress(data: &[u8]) -> Result<Vec<u8>, TransportError> {
    let mut decoder = ZlibDecoder::new(data);
    let mut out = Vec::new();
    match decoder.read_to_end(&mut out) {
        Ok(_) => Ok(out),
        Err(e) if e.kind() == IoErrorKind::UnexpectedEof => Err(TransportError::UnexpectedEof),
        Err(e) => Err(TransportError::Zlib(e.to_string())),
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test setup uses expect for clarity")]
#[expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
#[expect(
    clippy::as_conversions,
    reason = "test helpers use as-casts for byte conversions"
)]
#[expect(
    clippy::cast_possible_truncation,
    reason = "test helpers truncate lengths into u32 in controlled ranges"
)]
mod tests {
    use super::*;
    use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
    use rustls::server::ServerConfig;
    use std::sync::Arc;
    use tokio::io::AsyncReadExt;
    use tokio::net::TcpListener;

    /// Build a self-signed cert + matching rustls server config.
    fn self_signed_server_config() -> Arc<ServerConfig> {
        ensure_crypto_provider();
        let key_pair = rcgen::KeyPair::generate().expect("generate key pair");
        let cert_params =
            rcgen::CertificateParams::new(vec!["localhost".to_owned()]).expect("cert params");
        let cert = cert_params
            .self_signed(&key_pair)
            .expect("self-signed cert");
        let cert_der = CertificateDer::from(cert.der().to_vec());
        let key_der: PrivateKeyDer = PrivatePkcs8KeyDer::from(key_pair.serialize_der()).into();
        let server_config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![cert_der], key_der)
            .expect("server config");
        Arc::new(server_config)
    }

    /// Spawn an echo TLS server that speaks the 5-byte header + zlib framing.
    ///
    /// Returns the bound address. The server reads one framed payload,
    /// decompresses it, then writes it back framed. For the truncated-header
    /// test, `truncate_header` controls how many header bytes are sent.
    async fn spawn_echo_server(behavior: ServerBehavior) -> std::net::SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("local addr");
        let server_config = self_signed_server_config();
        let acceptor = tokio_rustls::TlsAcceptor::from(server_config);

        tokio::spawn(async move {
            // Best-effort server: errors are swallowed since the test
            // inspects the client-side result.
            if let Ok((tcp, _)) = listener.accept().await {
                if let Ok(mut tls) = acceptor.accept(tcp).await {
                    let _ = handle_client(&mut tls, behavior).await;
                }
            }
        });
        addr
    }

    enum ServerBehavior {
        /// Echo: read one framed payload, write it back framed.
        Echo,
        /// Send only `n` bytes of header, then close.
        TruncateHeader(usize),
    }

    async fn handle_client(
        tls: &mut tokio_rustls::server::TlsStream<TcpStream>,
        behavior: ServerBehavior,
    ) -> std::io::Result<()> {
        match behavior {
            ServerBehavior::Echo => {
                // Read header.
                let mut header = [0u8; HEADER_LEN];
                tls.read_exact(&mut header).await?;
                let body_len = u32::from_be_bytes([header[1], header[2], header[3], header[4]]);
                let mut body = vec![0u8; body_len as usize];
                tls.read_exact(&mut body).await?;
                // Decompress to verify it's valid zlib.
                let decoded = {
                    let mut dec = ZlibDecoder::new(&body[..]);
                    let mut out = Vec::new();
                    dec.read_to_end(&mut out)?;
                    out
                };
                // Re-compress and write back with a fresh header.
                let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
                enc.write_all(&decoded)?;
                let recompressed = enc.finish()?;
                let len = recompressed.len() as u32;
                let mut frame = Vec::with_capacity(HEADER_LEN + recompressed.len());
                frame.push(PROTOCOL_VERSION);
                frame.extend_from_slice(&len.to_be_bytes());
                frame.extend_from_slice(&recompressed);
                tls.write_all(&frame).await?;
                tls.flush().await?;
            }
            ServerBehavior::TruncateHeader(n) => {
                let partial = &header_bytes()[..n];
                tls.write_all(partial).await?;
                tls.flush().await?;
                tls.shutdown().await?;
            }
        }
        Ok(())
    }

    fn header_bytes() -> [u8; HEADER_LEN] {
        let mut h = [0u8; HEADER_LEN];
        h[0] = PROTOCOL_VERSION;
        h
    }

    #[tokio::test]
    async fn when_roundtrip_send_recv_then_returns_same_bytes() {
        let addr = spawn_echo_server(ServerBehavior::Echo).await;
        let mut transport = DelugeTransport::connect("127.0.0.1", addr.port())
            .await
            .expect("connect to echo server");
        let payload = b"hello deluge daemon rpc rencode payload";
        transport.send(payload).await.expect("send");
        let received = transport.recv().await.expect("recv");
        assert_eq!(received, payload);
    }

    #[tokio::test]
    async fn when_connect_to_closed_port_then_returns_error() {
        // Bind and immediately drop to obtain a closed port.
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        drop(listener);
        let result = DelugeTransport::connect("127.0.0.1", addr.port()).await;
        assert!(
            result.is_err(),
            "connecting to a closed port must return an error"
        );
    }

    #[tokio::test]
    async fn when_recv_truncated_header_then_returns_error() {
        let addr = spawn_echo_server(ServerBehavior::TruncateHeader(2)).await;
        let mut transport = DelugeTransport::connect("127.0.0.1", addr.port())
            .await
            .expect("connect to truncate server");
        let result = transport.recv().await;
        assert!(
            result.is_err(),
            "recv with a truncated header must return an error"
        );
    }
}
