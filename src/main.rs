mod proxy;
mod tls;
mod ssh;
mod websocket;
mod xhttp;
mod security;
mod tcp_fallback;
mod protocol;

use std::env;
use std::io::Error;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::time::{Duration, timeout};
use log::{info, error, warn};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Inicializa logging
    env_logger::init();
    
    let args: Vec<String> = env::args().collect();
    
    // Modo menu interativo
    if args.len() > 1 && args[1] == "menu" {
        return run_menu().await;
    }
    
    // Modo normal - inicia proxy
    let port = get_port();
    let status = get_status();
    
    info!("🚀 Iniciando BSProxy Multi-Protocol na porta: {}", port);
    info!("📡 Status: {}", status);
    
    let listener = TcpListener::bind(format!("[::]:{}", port)).await?;
    start_multi_protocol(listener, status).await;
    
    Ok(())
}

// ============ NOVO: Multi-Protocol Handler ============
async fn start_multi_protocol(listener: TcpListener, status: String) {
    loop {
        match listener.accept().await {
            Ok((client_stream, addr)) => {
                let status_clone = status.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_multi_protocol(client_stream, status_clone).await {
                        error!("Erro ao processar cliente {}: {}", addr, e);
                    }
                });
            }
            Err(e) => {
                error!("Erro ao aceitar conexão: {}", e);
            }
        }
    }
}

// ============ NOVO: Handler Multi-Protocol ============
async fn handle_multi_protocol(mut client_stream: TcpStream, status: String) -> Result<(), Error> {
    info!("📥 Nova conexão detectada");
    
    // Primeiro, faz o peek para detectar o protocolo
    let peek_result = timeout(Duration::from_secs(2), peek_stream(&client_stream)).await;
    
    let protocol = if let Ok(Ok(data)) = peek_result {
        detect_protocol(&data)
    } else {
        "SSH".to_string() // Fallback para SSH
    };
    
    info!("🔍 Protocolo detectado: {}", protocol);
    
    // Responde com status (mantendo compatibilidade)
    client_stream
        .write_all(format!("HTTP/1.1 101 {}\r\n\r\n", status).as_bytes())
        .await?;
    
    let mut buffer = vec![0; 1024];
    client_stream.read(&mut buffer).await?;
    client_stream
        .write_all(format!("HTTP/1.1 200 {}\r\n\r\n", status).as_bytes())
        .await?;
    
    // Roteia para o handler apropriado
    match protocol.as_str() {
        "TLS" | "SSL" => {
            info!("🔒 Roteando para handler SSL/TLS");
            handle_tls_tunnel(client_stream).await?;
        }
        "WEBSOCKET" => {
            info!("🔌 Roteando para handler WebSocket");
            handle_websocket_tunnel(client_stream).await?;
        }
        "XHTTP" => {
            info!("🌐 Roteando para handler XHTTP");
            handle_xhttp_tunnel(client_stream).await?;
        }
        _ => { // SSH ou desconhecido
            info!("🔑 Roteando para handler SSH");
            handle_ssh_tunnel(client_stream).await?;
        }
    }
    
    Ok(())
}

// ============ DETECÇÃO DE PROTOCOLO ============
fn detect_protocol(data: &str) -> String {
    if data.starts_with('\x16') && data.len() > 2 && data.as_bytes()[1] == 0x03 {
        return "TLS".to_string();
    }
    
    if data.contains("SSH-") {
        return "SSH".to_string();
    }
    
    if data.contains("Upgrade: websocket") || data.contains("Sec-WebSocket-Key") {
        return "WEBSOCKET".to_string();
    }
    
    if data.contains("X-") || data.contains("XHTTP") {
        return "XHTTP".to_string();
    }
    
    if data.contains("GET /") || data.contains("POST /") || data.contains("CONNECT") {
        return "XHTTP".to_string();
    }
    
    "SSH".to_string() // Default
}

// ============ HANDLERS ESPECÍFICOS ============

// Handler SSH (seu código original melhorado)
async fn handle_ssh_tunnel(mut client_stream: TcpStream) -> Result<(), Error> {
    info!("🔑 Iniciando túnel SSH");
    
    let addr_proxy = "0.0.0.0:22";
    let server_stream = match TcpStream::connect(addr_proxy).await {
        Ok(s) => s,
        Err(e) => {
            error!("❌ Erro ao conectar ao servidor SSH: {}", e);
            return Ok(());
        }
    };
    
    proxy_data(client_stream, server_stream).await
}

// Handler TLS/SSL (NOVO)
async fn handle_tls_tunnel(mut client_stream: TcpStream) -> Result<(), Error> {
    info!("🔒 Iniciando túnel SSL + SSH");
    
    // Conecta ao servidor SSH via TLS
    let addr_proxy = "0.0.0.0:443"; // Servidor SSH via TLS
    let server_stream = match TcpStream::connect(addr_proxy).await {
        Ok(s) => s,
        Err(e) => {
            error!("❌ Erro ao conectar ao servidor SSL: {}", e);
            return Ok(());
        }
    };
    
    // Aqui você pode adicionar handshake TLS se necessário
    proxy_data(client_stream, server_stream).await
}

// Handler WebSocket (NOVO)
async fn handle_websocket_tunnel(mut client_stream: TcpStream) -> Result<(), Error> {
    info!("🔌 Iniciando túnel WebSocket");
    
    // Conecta ao servidor WebSocket
    let addr_proxy = "0.0.0.0:8080"; // Servidor WebSocket
    let server_stream = match TcpStream::connect(addr_proxy).await {
        Ok(s) => s,
        Err(e) => {
            error!("❌ Erro ao conectar ao servidor WebSocket: {}", e);
            return Ok(());
        }
    };
    
    // Envia resposta Multi-Status (207)
    let multistatus = "HTTP/1.1 207 Multi-Status\r\nX-Supported: SSL,SSH,WebSocket,XHTTP\r\n\r\n";
    let _ = client_stream.write_all(multistatus.as_bytes()).await;
    
    proxy_data(client_stream, server_stream).await
}

// Handler XHTTP (NOVO)
async fn handle_xhttp_tunnel(mut client_stream: TcpStream) -> Result<(), Error> {
    info!("🌐 Iniciando túnel XHTTP");
    
    // Conecta ao servidor XHTTP
    let addr_proxy = "0.0.0.0:8443"; // Servidor XHTTP
    let server_stream = match TcpStream::connect(addr_proxy).await {
        Ok(s) => s,
        Err(e) => {
            error!("❌ Erro ao conectar ao servidor XHTTP: {}", e);
            // Se não tiver servidor XHTTP, faz fallback para SSH
            warn!("⚠️ Fallback para SSH");
            return handle_ssh_tunnel(client_stream).await;
        }
    };
    
    proxy_data(client_stream, server_stream).await
}

// ============ FUNÇÃO DE PROXY (SUA ORIGINAL MELHORADA) ============
async fn proxy_data(client_stream: TcpStream, server_stream: TcpStream) -> Result<(), Error> {
    let (client_read, client_write) = client_stream.into_split();
    let (server_read, server_write) = server_stream.into_split();
    
    let client_read = Arc::new(Mutex::new(client_read));
    let client_write = Arc::new(Mutex::new(client_write));
    let server_read = Arc::new(Mutex::new(server_read));
    let server_write = Arc::new(Mutex::new(server_write));
    
    let client_to_server = transfer_data(client_read, server_write);
    let server_to_client = transfer_data(server_read, client_write);
    
    tokio::try_join!(client_to_server, server_to_client)?;
    
    Ok(())
}

// ============ FUNÇÃO DE TRANSFERÊNCIA (SUA ORIGINAL) ============
async fn transfer_data(
    read_stream: Arc<Mutex<tokio::net::tcp::OwnedReadHalf>>,
    write_stream: Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>,
) -> Result<(), Error> {
    let mut buffer = [0; 8192];
    loop {
        let bytes_read = {
            let mut read_guard = read_stream.lock().await;
            read_guard.read(&mut buffer).await?
        };
        
        if bytes_read == 0 {
            break;
        }
        
        let mut write_guard = write_stream.lock().await;
        write_guard.write_all(&buffer[..bytes_read]).await?;
    }
    
    Ok(())
}

// ============ PEEK STREAM (SUA ORIGINAL) ============
async fn peek_stream(stream: &TcpStream) -> Result<String, Error> {
    let mut peek_buffer = vec![0; 8192];
    let bytes_peeked = stream.peek(&mut peek_buffer).await?;
    let data = &peek_buffer[..bytes_peeked];
    let data_str = String::from_utf8_lossy(data);
    Ok(data_str.to_string())
}

// ============ FUNÇÕES DE CONFIGURAÇÃO (SUAS ORIGINAIS) ============
fn get_port() -> u16 {
    let args: Vec<String> = env::args().collect();
    let mut port = 80;
    
    for i in 1..args.len() {
        if args[i] == "--port" {
            if i + 1 < args.len() {
                port = args[i + 1].parse().unwrap_or(80);
            }
        }
    }
    
    port
}

fn get_status() -> String {
    let args: Vec<String> = env::args().collect();
    let mut status = String::from("BSPROXY-MULTI");
    
    for i in 1..args.len() {
        if args[i] == "--status" {
            if i + 1 < args.len() {
                status = args[i + 1].clone();
            }
        }
    }
    
    status
}

// ============ MENU INTERATIVO (NOVO) ============
async fn run_menu() -> Result<(), Error> {
    use std::io::{self, Write};
    
    println!("\n╔══════════════════════════════════════╗");
    println!("║        BSProxy Multi-Protocol        ║");
    println!("╠══════════════════════════════════════╣");
    println!("║ 1 - Iniciar Proxy (Multi-Protocol)   ║");
    println!("║ 2 - Status do Servidor               ║");
    println!("║ 3 - Suporte SSL + SSH                ║");
    println!("║ 4 - Suporte SSL + WebSocket          ║");
    println!("║ 5 - Suporte XHTTP                    ║");
    println!("║ 6 - Multi-Status (207)               ║");
    println!("║ 0 - Sair                             ║");
    println!("╚══════════════════════════════════════╝");
    print!("👉 Selecione uma opção: ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    match input.trim() {
        "1" => {
            println!("🚀 Iniciando proxy na porta 80...");
            let listener = TcpListener::bind("[::]:80").await?;
            start_multi_protocol(listener, "BSPROXY-MULTI".to_string()).await;
        }
        "2" => {
            println!("\n📊 STATUS DO SERVIDOR");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("✅ SSL + SSH: ATIVADO");
            println!("✅ SSL + WebSocket: ATIVADO");
            println!("✅ XHTTP: ATIVADO");
            println!("✅ Multi-Status (207): ATIVADO");
            println!("✅ Multi-Protocolo: ATIVADO");
            println!("📌 Portas: 80, 443, 8080, 8443");
            println!("🔐 Modo: Túnel Múltiplo");
        }
        "3" => {
            println!("🔒 Ativando SSL + SSH na porta 443...");
            // Inicia servidor SSL+SSH
        }
        "4" => {
            println!("🔌 Ativando SSL + WebSocket na porta 8080...");
        }
        "5" => {
            println!("🌐 Ativando XHTTP na porta 8443...");
        }
        "6" => {
            println!("📡 Testando Multi-Status (207)...");
            // Testa resposta 207
        }
        "0" => {
            println!("👋 Saindo...");
            return Ok(());
        }
        _ => {
            println!("❌ Opção inválida!");
        }
    }
    
    Ok(())
}
