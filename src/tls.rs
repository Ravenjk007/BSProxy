use tokio::net::TcpStream;
use tokio_rustls::TlsAcceptor;
use rustls::{ServerConfig, Certificate, PrivateKey};
use std::sync::Arc;
use anyhow::Result;
use log::info;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn handle_tls(socket: TcpStream) -> Result<()> {
    info!("🔒 TLS/SECURITY handshake...");
    
    // Gerar certificado self-signed (igual wssecury)
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])?;
    let cert_der = cert.serialize_der()?;
    let key_der = cert.serialize_private_key_der();
    
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(vec![Certificate(cert_der)], PrivateKey(key_der))?;
    
    let acceptor = TlsAcceptor::from(Arc::new(config));
    let mut tls_stream = acceptor.accept(socket).await?;
    
    info!("🔒 TLS handshake complete!");
    
    // Encaminhar para SSH (igual wssecury)
    let target = "127.0.0.1:22";
    
    match TcpStream::connect(target).await {
        Ok(remote) => {
            info!("✅ Conectado ao SSH na porta 22");
            
            let (mut client_reader, mut client_writer) = tls_stream.into_split();
            let (mut remote_reader, mut remote_writer) = remote.into_split();
            
            tokio::try_join!(
                tokio::io::copy(&mut client_reader, &mut remote_writer),
                tokio::io::copy(&mut remote_reader, &mut client_writer)
            )?;
            
            info!("🔚 Conexão TLS->SSH encerrada");
            Ok(())
        }
        Err(e) => {
            info!("❌ Falha ao conectar ao SSH: {}", e);
            anyhow::bail!("SSH connection failed: {}", e)
        }
    }
}
