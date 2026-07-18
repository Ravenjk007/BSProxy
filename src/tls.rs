use tokio::net::TcpStream;
use tokio_rustls::TlsAcceptor;
use rustls::{ServerConfig, Certificate, PrivateKey};
use std::sync::Arc;
use std::fs;
use anyhow::Result;
use log::info;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn handle_tls(socket: TcpStream) -> Result<()> {
    info!("🔒 TLS/SECURITY handshake...");
    
    // Tenta carregar certificado REAL (se existir)
    let (cert, key) = match load_certificates() {
        Ok((c, k)) => (c, k),
        Err(_) => {
            // Fallback para self-signed
            info!("⚠️ Certificado real não encontrado, usando self-signed");
            generate_self_signed_cert()?
        }
    };
    
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)?;
    
    let acceptor = TlsAcceptor::from(Arc::new(config));
    let mut tls_stream = acceptor.accept(socket).await?;
    
    info!("🔒 TLS handshake complete!");
    
    // Encaminhar para SSH ou fazer eco
    let mut buf = [0u8; 1024];
    loop {
        match tls_stream.read(&mut buf).await {
            Ok(0) => break,
            Ok(n) => {
                let msg = String::from_utf8_lossy(&buf[..n]);
                let response = format!("SECURE: {}", msg);
                tls_stream.write_all(response.as_bytes()).await?;
            }
            Err(e) => anyhow::bail!("TLS error: {}", e),
        }
    }
    
    Ok(())
}

fn load_certificates() -> Result<(Certificate, PrivateKey)> {
    // Tenta carregar certificados do diretório /etc/letsencrypt
    let cert_paths = [
        "/etc/letsencrypt/live/your-domain/fullchain.pem",
        "/opt/bsproxy/cert.pem",
        "./cert.pem",
    ];
    
    for path in cert_paths {
        if std::path::Path::new(path).exists() {
            let cert_data = fs::read(path)?;
            let cert = Certificate(cert_data);
            
            // Tenta carregar a chave privada (mesmo nome com .key)
            let key_path = path.replace(".pem", ".key");
            if std::path::Path::new(&key_path).exists() {
                let key_data = fs::read(&key_path)?;
                let key = PrivateKey(key_data);
                return Ok((cert, key));
            }
        }
    }
    
    anyhow::bail!("No certificates found")
}

fn generate_self_signed_cert() -> Result<(Certificate, PrivateKey)> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])?;
    let cert_der = cert.serialize_der()?;
    let key_der = cert.serialize_private_key_der();
    Ok((Certificate(cert_der), PrivateKey(key_der)))
}
