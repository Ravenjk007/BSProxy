use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{copy_bidirectional, AsyncReadExt};
use log::{info, error};

use crate::{Config, SecurityManager, ProtocolDetector};

pub async fn start_proxy(listener: TcpListener, config: Arc<Config>) {
    let security = Arc::new(SecurityManager::new());

    loop {
        let (client, addr) = match listener.accept().await {
            Ok(c) => c,
            Err(e) => { error!("Accept error: {}", e); continue; }
        };

        let ip = addr.ip();
        if !config.whitelist.is_empty() && !config.whitelist.contains(&ip.to_string()) {
            continue;
        }

        if !security.allow_connection(ip) {
            continue;
        }

        let config_clone = config.clone();
        let security_clone = security.clone();

        tokio::spawn(async move {
            let _ = handle_client(client, config_clone, security_clone, ip).await;
        });
    }
}

async fn handle_client(
    mut client: TcpStream,
    config: Arc<Config>,
    security: Arc<SecurityManager>,
    ip: std::net::IpAddr,
) -> Result<(), std::io::Error> {
    // Handshake
    client.write_all(format!("HTTP/1.1 101 {}\r\n\r\n", config.status_msg).as_bytes()).await?;

    let mut buf = [0u8; 8192];
    let n = client.peek(&mut buf).await.unwrap_or(0);

    let proto = ProtocolDetector::detect(&buf[..n]);
    let target = config.backends.get(&proto).unwrap_or(&config.backends["ssh"]);

    info!("🔀 {} → {} | IP: {}", proto, target, ip);

    let mut server = match TcpStream::connect(target.as_str()).await {
        Ok(s) => s,
        Err(e) => {
            error!("Falha ao conectar {}: {}", target, e);
            security.release(ip);
            return Ok(());
        }
    };

    let _ = copy_bidirectional(&mut client, &mut server).await;
    security.release(ip);
    info!("✅ Conexão finalizada: {}", ip);

    Ok(())
}
