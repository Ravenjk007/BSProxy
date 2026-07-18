mod socks5;
mod tls;
mod tcp_fallback;

use tokio::net::TcpListener;
use tokio::io::AsyncReadExt;
use std::sync::Arc;
use anyhow::Result;
use log::{info, error};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let port = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "8080".to_string());
    
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    info!("🚀 BSProxy Multiprotocol listening on {}", addr);
    info!("📡 Protocols: SOCKS5, TLS/SECURITY, TCP Fallback");

    while let Ok((mut socket, _)) = listener.accept().await {
        tokio::spawn(async move {
            let mut buf = [0u8; 1];
            match socket.peek(&mut buf).await {
                Ok(_) => {
                    match buf[0] {
                        0x05 => {
                            info!("🔐 SOCKS5 connection detected");
                            if let Err(e) = socks5::handle(socket).await {
                                error!("SOCKS5 error: {}", e);
                            }
                        }
                        0x16 => {
                            info!("🔒 TLS/SECURITY connection detected");
                            if let Err(e) = tls::handle(socket).await {
                                error!("TLS error: {}", e);
                            }
                        }
                        _ => {
                            info!("📦 TCP Fallback connection detected");
                            if let Err(e) = tcp_fallback::handle(socket).await {
                                error!("TCP Fallback error: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to peek connection: {}", e);
                }
            }
        });
    }

    Ok(())
}
