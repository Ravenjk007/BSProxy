mod socks5;
mod tls;
mod tcp_fallback;

use tokio::net::TcpListener;
use tokio::io::AsyncReadExt;
use clap::Parser;
use anyhow::Result;
use log::{info, error};
use std::process::Command;

#[derive(Parser)]
#[command(name = "bsproxy")]
#[command(about = "Multiprotocol proxy server (SOCKS5 + TLS + TCP)", long_about = None)]
struct Cli {
    #[arg(short = 'p', long = "port", default_value = "")]
    port: String,
    #[arg(short = 'd', long = "debug")]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Se não foi passada porta, abre o menu
    if cli.port.is_empty() {
        show_menu();
        return Ok(());
    }
    
    // Se foi passada porta, inicia o proxy
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
    info!("📡 Protocols: SOCKS5, TLS/SECURITY, TCP Fallback");

    while let Ok((mut socket, _)) = listener.accept().await {
        tokio::spawn(async move {
            let mut buf = [0u8; 1];
            match socket.peek(&mut buf).await {
                Ok(_) => {
                    match buf[0] {
                        0x05 => {
                            info!("🔐 SOCKS5 connection");
                            let _ = socks5::handle(socket).await;
                        }
                        0x16 => {
                            info!("🔒 TLS/SECURITY connection");
                            let _ = tls::handle(socket).await;
                        }
                        _ => {
                            info!("📦 TCP Fallback connection");
                            let _ = tcp_fallback::handle(socket).await;
                        }
                    }
                }
                Err(e) => error!("Failed to peek: {}", e),
            }
        });
    }
    Ok(())
}

fn show_menu() {
    // Procura o menu em vários lugares
    let menu_paths = [
        "./menu.sh",
        "/opt/bsproxy/menu",
        "/usr/local/bin/menu",
        &format!("{}/BSProxy/menu.sh", std::env::var("HOME").unwrap_or_else(|_| "~".to_string())),
    ];
    
    for path in menu_paths {
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
