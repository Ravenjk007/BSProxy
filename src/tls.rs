use tokio::net::TcpStream;
use tokio_rustls::TlsAcceptor;
use rustls::{ServerConfig, Certificate, PrivateKey};
use std::sync::Arc;
use anyhow::Result;
use log::info;

pub async fn handle(socket: TcpStream) -> Result<()> {
    info!("🔒 Establishing TLS connection...");
    
    // Carregar certificado (self-signed para demo)
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])?;
    let cert_der = cert.serialize_der()?;
    let key_der = cert.serialize_private_key_der();
    
    let cert = Certificate(cert_der);
    let key = PrivateKey(key_der);
    
    let mut config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)?;
    
    let acceptor = TlsAcceptor::from(Arc::new(config));
    let mut tls_stream = acceptor.accept(socket).await?;
    
    info!("🔒 TLS handshake complete!");
    
    // Echo server simples (exemplo)
    let mut buf = [0u8; 1024];
    loop {
        match tls_stream.read(&mut buf).await {
            Ok(0) => break, // EOF
            Ok(n) => {
                let msg = String::from_utf8_lossy(&buf[..n]);
                info!("📩 Received: {}", msg);
                
                // Resposta eco com "SECURE: "
                let response = format!("SECURE: {}", msg);
                tls_stream.write_all(response.as_bytes()).await?;
            }
            Err(e) => {
                anyhow::bail!("TLS read error: {}", e);
            }
        }
    }
    
    Ok(())
}
