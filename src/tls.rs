use tokio::net::TcpStream;
use tokio_rustls::{TlsAcceptor, server::TlsStream};
use rustls::{ServerConfig, Certificate, PrivateKey};
use std::sync::Arc;
use anyhow::Result;
use rcgen::generate_simple_self_signed;

pub async fn get_tls_acceptor() -> Result<TlsAcceptor> {
    let cert = generate_simple_self_signed(vec!["localhost".to_string()])?;
    let cert_der = cert.serialize_der()?;
    let key_der = cert.serialize_private_key_der();
    
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(vec![Certificate(cert_der)], PrivateKey(key_der))?;
    
    Ok(TlsAcceptor::from(Arc::new(config)))
}

pub async fn handle_tls_stream(socket: TcpStream) -> Result<TlsStream<TcpStream>> {
    let acceptor = get_tls_acceptor().await?;
    let tls_stream = acceptor.accept(socket).await?;
    Ok(tls_stream)
}
