use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::{crypto, ServerConfig};
use std::sync::Once;

pub fn self_signed_server_config() -> ServerConfig {
    ensure_crypto_provider();
    let key_pair = rcgen::KeyPair::generate().expect("generate key pair");
    let cert_params =
        rcgen::CertificateParams::new(vec!["localhost".to_owned()]).expect("cert params");
    let cert = cert_params
        .self_signed(&key_pair)
        .expect("self-signed cert");
    let cert_der = CertificateDer::from(cert.der().to_vec());
    let key_der: PrivateKeyDer = PrivatePkcs8KeyDer::from(key_pair.serialize_der()).into();
    ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert_der], key_der)
        .expect("server config")
}

fn ensure_crypto_provider() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        let _ = crypto::ring::default_provider().install_default();
    });
}
