use std::env;
use std::io::Error;
use std::time::Duration;

use clap::Parser;
use log::{error, info, warn};
use tokio::io::copy_bidirectional;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;

mod protocol;
mod security;
mod socks5;
mod ssh;
mod tcp_fallback;
mod tls;
mod websocket;

use protocol::ProtocolDetector;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Porta do servidor
    #[arg(short, long, default_value_t = 80)]
    port: u16,

    /// Status message nos handshakes
    #[arg(short, long, default_value = "@BSPROXY")]
    status: String,

    /// Tempo máximo de inatividade (segundos)
    #[arg(long, default_value_t = 300)]
    idle_timeout: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args = Args::parse();

    info!("🚀 BSPoxy v0.2.0 iniciando na porta {}", args.port);

    let listener = TcpListener::bind(format!("[::]:{}", args.port)).await?;
    info!("✅ Servidor ouvindo em [::]:{}", args.port);

    start_proxy(listener, args).await;
    Ok(())
}

async fn start_proxy(listener: TcpListener, args: Args) {
    loop {
        match listener.accept().await {
            Ok((client_stream, addr)) => {
                info!("🔗 Nova conexão de {}", addr);
                let args_clone = args.clone(); // Clone barato

                tokio::spawn(async move {
                    if let Err(e) = handle_client(client_stream, &args_clone).await {
                        error!("❌ Erro ao processar {}: {}", addr, e);
                    }
                });
            }
            Err(e) => error!("Erro ao aceitar conexão: {}", e),
        }
    }
}

async fn handle_client(mut client: TcpStream, args: &Args) -> Result<(), Error> {
    // Handshake inicial (padrão para proxies tipo SSH/SSL)
    client
        .write_all(format!("HTTP/1.1 101 {}\r\n\r\n", args.status).as_bytes())
        .await?;

    // Pequeno delay para o cliente enviar dados
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Detectar protocolo
    let backend = match timeout(Duration::from_secs(5), detect_protocol(&mut client)).await {
        Ok(Ok(proto)) => proto,
        Ok(Err(e)) => {
            warn!("Falha na detecção: {}, usando fallback SSH", e);
            "ssh".to_string()
        }
        Err(_) => {
            info!("Timeout na detecção → fallback SSH");
            "ssh".to_string()
        }
    };

    let target_addr = match backend.as_str() {
        "ssh" => "127.0.0.1:22",
        "openvpn" | "ovpn" => "127.0.0.1:1194",
        "socks5" => "127.0.0.1:1080",
        _ => "127.0.0.1:22",
    };

    info!("🔀 Encaminhando para {} → {}", backend, target_addr);

    let mut server = match TcpStream::connect(target_addr).await {
        Ok(s) => s,
        Err(e) => {
            error!("❌ Falha ao conectar em {}: {}", target_addr, e);
            return Ok(());
        }
    };

    // Bidirectional copy com timeout opcional
    let _ = copy_bidirectional(&mut client, &mut server).await;

    info!("✅ Conexão encerrada");
    Ok(())
}

async fn detect_protocol(stream: &mut TcpStream) -> Result<String, Error> {
    let mut buffer = [0u8; 8192];
    let n = stream.peek(&mut buffer).await?;

    if n == 0 {
        return Ok("ssh".to_string());
    }

    let data = &buffer[..n];

    // Delegar para módulo de detecção (recomendado)
    Ok(ProtocolDetector::detect(data))
}
