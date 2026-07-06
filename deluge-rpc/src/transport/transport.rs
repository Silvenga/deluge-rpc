use crate::transport::error::TransportError;
use crate::transport::reader::DelugeReader;
use crate::transport::verifier::NoVerifier;
use crate::transport::writer::DelugeWriter;
use rustls::ClientConfig;
use rustls::crypto;
use rustls::pki_types::ServerName;
use std::io;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::split;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;

pub struct DelugeTransport {
    stream: TlsStream<TcpStream>,
}

impl DelugeTransport {
    pub async fn connect(host: &str, port: u16) -> Result<Self, TransportError> {
        ensure_crypto_provider();
        let config = client_config();
        let connector = TlsConnector::from(Arc::new(config));
        let domain = server_name(host)?;
        let tcp = timeout(Duration::from_secs(10), TcpStream::connect((host, port)))
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "TCP connect timed out"))??;
        let stream = timeout(Duration::from_secs(10), connector.connect(domain, tcp))
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "TLS handshake timed out"))??;
        Ok(Self { stream })
    }

    pub fn split(self) -> (DelugeReader, DelugeWriter) {
        let (read, write) = split(self.stream);
        (DelugeReader::new(read), DelugeWriter::new(write))
    }
}

fn client_config() -> ClientConfig {
    ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(NoVerifier))
        .with_no_client_auth()
}

fn ensure_crypto_provider() {
    use std::sync::Once;
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        if let Err(e) = crypto::ring::default_provider().install_default() {
            tracing::warn!(error = ?e, "failed to install ring crypto provider; TLS may use an unexpected backend");
        }
    });
}

fn server_name(host: &str) -> Result<ServerName<'static>, rustls::Error> {
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Ok(ServerName::IpAddress(ip.into()));
    }
    ServerName::try_from(host.to_owned())
        .map_err(|e| rustls::Error::from(rustls::OtherError(Arc::new(e))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::constants::{HEADER_LEN, PROTOCOL_VERSION};
    use flate2::Compression;
    use flate2::read::ZlibDecoder;
    use flate2::write::ZlibEncoder;
    use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
    use rustls::server::ServerConfig;
    use std::io::{Read, Write};
    use std::net::SocketAddr;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

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

    async fn spawn_echo_server() -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("local addr");
        let server_config = self_signed_server_config();
        let acceptor = tokio_rustls::TlsAcceptor::from(server_config);

        tokio::spawn(async move {
            if let Ok((tcp, _)) = listener.accept().await {
                if let Ok(mut tls) = acceptor.accept(tcp).await {
                    let mut header = [0u8; HEADER_LEN];
                    if tls.read_exact(&mut header).await.is_ok() {
                        let body_len =
                            u32::from_be_bytes([header[1], header[2], header[3], header[4]]);
                        let mut body = vec![0u8; body_len as usize];
                        if tls.read_exact(&mut body).await.is_ok() {
                            let decoded = {
                                let mut dec = ZlibDecoder::new(&body[..]);
                                let mut out = Vec::new();
                                let _ = dec.read_to_end(&mut out);
                                out
                            };
                            let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
                            let _ = enc.write_all(&decoded);
                            if let Ok(recompressed) = enc.finish() {
                                let len = recompressed.len() as u32;
                                let mut frame = Vec::with_capacity(HEADER_LEN + recompressed.len());
                                frame.push(PROTOCOL_VERSION);
                                frame.extend_from_slice(&len.to_be_bytes());
                                frame.extend_from_slice(&recompressed);
                                let _ = tls.write_all(&frame).await;
                                let _ = tls.flush().await;
                            }
                        }
                    }
                }
            }
        });
        addr
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn when_roundtrip_send_recv_then_returns_same_bytes() {
        let addr = spawn_echo_server().await;
        let transport = DelugeTransport::connect("127.0.0.1", addr.port())
            .await
            .expect("connect to echo server");
        let (mut reader, mut writer) = transport.split();
        let payload = b"hello deluge daemon rpc rencode payload";
        writer.send(payload).await.expect("send");
        let received = reader.recv().await.expect("recv");
        assert_eq!(received, payload);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn when_reader_in_spawned_task_then_receives_data() {
        let addr = spawn_echo_server().await;
        let transport = DelugeTransport::connect("127.0.0.1", addr.port())
            .await
            .expect("connect to echo server");
        let (mut reader, mut writer) = transport.split();
        let payload = b"hello from spawned reader test";

        let reader_handle = tokio::spawn(async move { reader.recv().await });

        writer.send(payload).await.expect("send");

        let received = timeout(Duration::from_secs(5), reader_handle)
            .await
            .expect("reader task timed out")
            .expect("reader task panicked")
            .expect("recv failed");
        assert_eq!(received, payload);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn when_connect_to_closed_port_then_returns_error() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        drop(listener);
        let result = DelugeTransport::connect("127.0.0.1", addr.port()).await;
        assert!(result.is_err());
    }
}
