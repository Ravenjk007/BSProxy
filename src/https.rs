use tokio::net::TcpStream;
use tokio_rustls::TlsAcceptor;
use rustls::{ServerConfig, Certificate, PrivateKey};
use std::sync::Arc;
use std::collections::HashMap;
use anyhow::Result;
use log::info;

pub async fn handle_https(socket: TcpStream, domain: &str) -> Result<()> {
    info!("🔒 HTTPS connection for: {}", domain);
    
    // Mapear domínios para certificados
    let certs = load_certificates_for_domain(domain)?;
    
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(vec![certs.0], certs.1)?;
    
    let acceptor = TlsAcceptor::from(Arc::new(config));
    let mut tls_stream = acceptor.accept(socket).await?;
    
    // ... resto do código
}

fn load_certificates_for_domain(domain: &str) -> Result<(Certificate, PrivateKey)> {
    let cert_path = format!("/etc/letsencrypt/live/{}/fullchain.pem", domain);
    let key_path = format!("/etc/letsencrypt/live/{}/privkey.pem", domain);
    
    let cert_data = std::fs::read(cert_path)?;
    let key_data = std::fs::read(key_path)?;
    
    Ok((Certificate(cert_data), PrivateKey(key_data)))
}
