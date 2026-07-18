mod socks5;
mod tls;
mod websocket;
mod tcp_fallback;
mod security;

use tokio::net::TcpListener;
use tokio::io::AsyncReadExt;
use clap::Parser;
use anyhow::Result;
use log::{info, error, warn};
use std::process::Command;

#[derive(Parser)]
#[command(name = "bsproxy")]
#[command(about = "Multiprotocol proxy server (SOCKS5 + TLS + WebSocket + TCP + SECURITY)")]
struct Cli {
    #[arg(short = 'p', long = "port", default_value = "")]
    port: String,
    #[arg(short = 'd', long = "debug")]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    if cli.port.is_empty() {
        show_menu();
        return Ok(());
    }
    
    if cli.debug {
        env_logger::init();
    } else {
        env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .init();
    }
    
    let port = cli.port.parse::<u16>().unwrap_or(8080);
    let addr = format!("0.0.0.0:{}", port);
    
    // Verificação especial para porta 443
    if port == 443 {
        warn!("⚠️ Porta 443 requer certificado SSL/TLS!");
        warn!("📂 Certificados procurados em: /opt/bsproxy/cert.pem e /opt/bsproxy/cert.key");
        warn!("   Ou use: certbot certonly --standalone -d seu-dominio.com");
        warn!("   Depois copie: cp /etc/letsencrypt/live/seu-dominio.com/* /opt/bsproxy/");
        
        // Verifica se o certificado existe
        if !std::path::Path::new("/opt/bsproxy/cert.pem").exists() {
            warn!("⚠️ Certificado não encontrado! Usando self-signed (clientes vão ver erro)");
        }
    }
    
    let listener = TcpListener::bind(&addr).await?;
    info!("🚀 BSProxy Multiprotocol listening on {}", addr);
    info!("📡 Protocols: SOCKS5, TLS, WebSocket, SECURITY, TCP");

    while let Ok((socket, _)) = listener.accept().await {
        tokio::spawn(async move {
            let mut buf = [0u8; 16];
            match socket.peek(&mut buf).await {
                Ok(n) if n > 0 => {
                    match buf[0] {
                        0x05 => {
                            info!("🔐 SOCKS5");
                            let _ = socks5::handle_socks5(socket).await;
                        }
                        0x16 => {
                            info!("🔒 TLS/SECURITY");
                            let _ = tls::handle_tls(socket).await;
                        }
                        _ => {
                            let data_str = String::from_utf8_lossy(&buf[..n]);
                            if data_str.starts_with("GET ") || data_str.starts_with("HTTP/") {
                                info!("🌐 WebSocket");
                                let _ = websocket::handle_websocket(socket).await;
                            } else if data_str.starts_with("SECURITY") || data_str.starts_with("AUTH") {
                                info!("🔐 SECURITY (custom)");
                                let _ = security::handle_security(socket).await;
                            } else {
                                info!("📦 TCP");
                                let _ = tcp_fallback::handle_tcp(socket).await;
                            }
                        }
                    }
                }
                Ok(_) => {
                    info!("📦 Connection closed");
                }
                Err(e) => error!("Peek error: {}", e),
            }
        });
    }
    Ok(())
}

fn show_menu() {
    let paths = [
        "/opt/bsproxy/menu",
        "./menu.sh",
        "/usr/local/bin/menu",
    ];
    
    for path in paths {
        if std::path::Path::new(path).exists() {
            let _ = Command::new("bash")
                .arg(path)
                .status();
            return;
        }
    }
    println!("❌ Menu não encontrado!");
    println!("Execute: /opt/bsproxy/menu");
}
