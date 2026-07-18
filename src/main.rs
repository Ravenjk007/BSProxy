mod socks5;
mod tls;
mod websocket;
mod tcp_fallback;
mod security;
mod http_handler;

use tokio::net::TcpListener;
use tokio::io::AsyncReadExt;
use clap::Parser;
use anyhow::Result;
use log::{info, error};
use std::process::Command;

#[derive(Parser)]
#[command(name = "bsproxy")]
#[command(about = "Multiprotocol proxy server (SOCKS5, TLS, WebSocket, SECURITY, HTTP)")]
struct Cli {
    #[arg(short = 'p', long = "port", default_value = "8080")]
    port: u16,
    #[arg(short = 'd', long = "debug")]
    debug: bool,
    #[arg(short = 's', long = "ssl")]
    ssl: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    if cli.debug {
        env_logger::init();
    } else {
        env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .init();
    }
    
    let addr = format!("0.0.0.0:{}", cli.port);
    let listener = TcpListener::bind(&addr).await?;
    info!("🚀 BSProxy Multiprotocol listening on {}", addr);
    info!("📡 Protocols: SOCKS5, TLS, WebSocket, SECURITY, HTTP");

    while let Ok((socket, _)) = listener.accept().await {
        tokio::spawn(async move {
            let mut buf = [0u8; 32];
            match socket.peek(&mut buf).await {
                Ok(n) if n > 0 => {
                    match buf[0] {
                        0x05 => {
                            info!("🔐 SOCKS5");
                            let _ = socks5::handle_socks5(socket).await;
                        }
                        0x16 => {
                            info!("🔒 TLS");
                            let _ = tls::handle_tls(socket).await;
                        }
                        _ => {
                            let data_str = String::from_utf8_lossy(&buf[..n]);
                            if data_str.contains("HTTP") || 
                               data_str.starts_with("GET ") || 
                               data_str.starts_with("POST ") || 
                               data_str.starts_with("PUT ") || 
                               data_str.starts_with("DELETE ") || 
                               data_str.starts_with("PATCH ") || 
                               data_str.starts_with("HEAD ") || 
                               data_str.starts_with("CONNECT ") || 
                               data_str.starts_with("OPTIONS ") || 
                               data_str.starts_with("TRACE ") {
                                info!("🌐 HTTP/WebSocket");
                                let _ = http_handler::handle_http(socket).await;
                            } else if data_str.starts_with("SECURITY") || data_str.starts_with("AUTH") {
                                info!("🔐 SECURITY");
                                let _ = security::handle_security(socket).await;
                            } else {
                                info!("📦 TCP Fallback");
                                let _ = tcp_fallback::handle_tcp(socket).await;
                            }
                        }
                    }
                }
                Ok(_) => info!("Connection closed"),
                Err(e) => error!("Peek error: {}", e),
            }
        });
    }
    Ok(())
}
