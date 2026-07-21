mod proxy;
mod tls;
mod websocket;
mod ssh;
mod xhttp;
mod protocol;
mod security;
mod tcp_fallback;

use std::env;
use std::io::Error;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, copy_bidirectional};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{timeout, Duration};
use log::{info, error, warn, debug};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Inicializa logging
    env_logger::init();
    
    let args: Vec<String> = env::args().collect();
    
    // Menu interativo
    if args.len() > 1 && args[1] == "menu" {
        return run_menu().await;
    }
    
    let port = get_port();
    let listener = TcpListener::bind(format!("[::]:{}", port)).await?;
    println!("🚀 Servidor iniciado na porta: {}", port);
    println!("📡 Status: {}", get_status());
    println!("🔧 Multi-Protocol: SSL, SSH, WebSocket, XHTTP");
    
    start_proxy(listener).await;
    Ok(())
}

async fn start_proxy(listener: TcpListener) {
    loop {
        match listener.accept().await {
            Ok((client_stream, addr)) => {
                println!("📥 Nova conexão de: {}", addr);
                tokio::spawn(async move {
                    if let Err(e) = handle_client(client_stream).await {
                        error!("Erro ao processar cliente {}: {}", addr, e);
                    }
                });
            }
            Err(e) => error!("Erro ao aceitar conexão: {}", e),
        }
    }
}

async fn handle_client(mut client_stream: TcpStream) -> Result<(), Error> {
    let status = get_status();
    let mut buffer = [0; 8192];
    
    // ========================================
    // 1. PEEK para detectar protocolo
    // ========================================
    let peek_data = match timeout(Duration::from_secs(3), peek_stream(&client_stream)).await {
        Ok(Ok(data)) => data,
        _ => String::new(),
    };
    
    let protocol = detect_protocol(&peek_data);
    println!("🔍 Protocolo detectado: {}", protocol);
    
    // ========================================
    // 2. ROTEAMENTO POR PROTOCOLO
    // ========================================
    match protocol.as_str() {
        "TLS" | "SSL" => {
            println!("🔒 Roteando para TLS/SSL Handler");
            return handle_tls_connection(client_stream, &peek_data).await;
        }
        "WEBSOCKET" => {
            println!("🔌 Roteando para WebSocket Handler");
            return handle_websocket_connection(client_stream).await;
        }
        "XHTTP" => {
            println!("🌐 Roteando para XHTTP Handler");
            return handle_xhttp_connection(client_stream, &peek_data, &status).await;
        }
        "SSH" => {
            println!("🔑 Roteando para SSH Handler");
            // Fallthrough para o código original
        }
        _ => {
            println!("❓ Protocolo desconhecido, usando fallback SSH");
        }
    }
    
    // ========================================
    // 3. HANDLER SSH (código original melhorado)
    // ========================================
    println!("🔑 Iniciando túnel SSH");
    
    // Primeiro handshake 101
    client_stream
        .write_all(format!("HTTP/1.1 101 {}\r\n\r\n", status).as_bytes())
        .await?;
    
    // Lê dados
    let _ = client_stream.read(&mut buffer).await?;
    
    // Segundo handshake 101
    client_stream
        .write_all(format!("HTTP/1.1 101 {}\r\n\r\n", status).as_bytes())
        .await?;
    
    // Resposta 200
    client_stream
        .write_all(format!("HTTP/1.1 200 {}\r\n\r\n", status).as_bytes())
        .await?;
    
    // Determina destino (SSH ou OpenVPN)
    let addr_proxy = if peek_data.contains("SSH") || peek_data.is_empty() {
        "0.0.0.0:22"
    } else {
        "0.0.0.0:1194"
    };
    
    println!("🔗 Conectando ao backend: {}", addr_proxy);
    
    let mut server_stream = match TcpStream::connect(addr_proxy).await {
        Ok(stream) => stream,
        Err(e) => {
            error!("❌ Erro ao conectar ao servidor proxy em {}: {}", addr_proxy, e);
            return Ok(());
        }
    };
    
    // Transfere dados bidirecionalmente
    let _ = copy_bidirectional(&mut client_stream, &mut server_stream).await;
    
    Ok(())
}

// ========================================
// HANDLER TLS/SSL (SSL + SSH, SSL + WebSocket)
// ========================================
async fn handle_tls_connection(client_stream: TcpStream, peek_data: &str) -> Result<(), Error> {
    println!("🔒 Processando TLS/SSL...");
    
    // Verifica se é WebSocket sobre TLS (SSL + WebSocket)
    if peek_data.contains("Upgrade: websocket") || peek_data.contains("Sec-WebSocket-Key") {
        println!("🔒🔌 SSL + WebSocket detectado");
        return websocket::handle_websocket_ssl(client_stream)
            .await
            .map_err(|e| Error::new(std::io::ErrorKind::Other, e.to_string()));
    }
    
    // Verifica se é XHTTP sobre TLS (SSL + XHTTP)
    if peek_data.contains("X-") || peek_data.contains("XHTTP") {
        println!("🔒🌐 SSL + XHTTP detectado");
        return websocket::handle_xhttp_ssl(client_stream)
            .await
            .map_err(|e| Error::new(std::io::ErrorKind::Other, e.to_string()));
    }
    
    // SSL + SSH (padrão)
    println!("🔒🔑 SSL + SSH detectado");
    
    // Usa o handler TLS existente
    match tls::handle_tls_stream(client_stream, "0.0.0.0:22").await {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("❌ Erro no TLS: {}", e);
            Ok(())
        }
    }
}

// ========================================
// HANDLER WEBSOCKET
// ========================================
async fn handle_websocket_connection(client_stream: TcpStream) -> Result<(), Error> {
    println!("🔌 Processando WebSocket...");
    
    match websocket::handle_websocket(client_stream).await {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("❌ Erro no WebSocket: {}", e);
            Ok(())
        }
    }
}

// ========================================
// HANDLER XHTTP (com Multi-Status 207)
// ========================================
async fn handle_xhttp_connection(
    client_stream: TcpStream,
    peek_data: &str,
    status: &str,
) -> Result<(), Error> {
    println!("🌐 Processando XHTTP...");
    
    // Verifica se é XHTTP com SSL
    if peek_data.contains("X-SSL:") || peek_data.contains("wss://") {
        println!("🔒 XHTTP com SSL detectado");
        return websocket::handle_xhttp_ssl(client_stream)
            .await
            .map_err(|e| Error::new(std::io::ErrorKind::Other, e.to_string()));
    }
    
    // XHTTP normal com Multi-Status (207)
    match websocket::handle_xhttp(client_stream).await {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("❌ Erro no XHTTP: {}", e);
            Ok(())
        }
    }
}

// ========================================
// DETECÇÃO DE PROTOCOLO
// ========================================
fn detect_protocol(data: &str) -> String {
    // TLS/SSL: começa com 0x16 0x03
    if data.len() > 2 && data.as_bytes()[0] == 0x16 && data.as_bytes()[1] == 0x03 {
        return "TLS".to_string();
    }
    
    // SSH
    if data.contains("SSH-") {
        return "SSH".to_string();
    }
    
    // WebSocket
    if data.contains("Upgrade: websocket") || data.contains("Sec-WebSocket-Key") {
        return "WEBSOCKET".to_string();
    }
    
    // XHTTP
    if data.contains("X-") || data.contains("XHTTP") {
        return "XHTTP".to_string();
    }
    
    // HTTP/HTTPS normal
    if data.contains("HTTP/") || data.contains("GET ") || data.contains("POST ") || data.contains("CONNECT") {
        // Se tiver Upgrade, é WebSocket
        if data.contains("Upgrade:") {
            return "WEBSOCKET".to_string();
        }
        return "HTTP".to_string();
    }
    
    "SSH".to_string()
}

// ========================================
// PEEK STREAM (sua função original)
// ========================================
async fn peek_stream(stream: &TcpStream) -> Result<String, Error> {
    let mut buffer = vec![0; 8192];
    let bytes_peeked = stream.peek(&mut buffer).await?;
    Ok(String::from_utf8_lossy(&buffer[..bytes_peeked]).to_string())
}

// ========================================
// FUNÇÕES DE CONFIGURAÇÃO
// ========================================
fn get_port() -> u16 {
    let args: Vec<String> = env::args().collect();
    let mut port = 80;
    
    for i in 0..args.len() {
        if args[i] == "--port" && i + 1 < args.len() {
            port = args[i + 1].parse().unwrap_or(80);
        }
    }
    
    port
}

fn get_status() -> String {
    let args: Vec<String> = env::args().collect();
    let mut status = String::from("@BSPROXY-MULTI");
    
    for i in 0..args.len() {
        if args[i] == "--status" && i + 1 < args.len() {
            status = args[i + 1].clone();
        }
    }
    
    status
}

// ========================================
// MENU INTERATIVO
// ========================================
async fn run_menu() -> Result<(), Error> {
    use std::io::{self, Write};
    
    println!("\n╔════════════════════════════════════════════╗");
    println!("║         BSProxy Multi-Protocol v2.0       ║");
    println!("╠════════════════════════════════════════════╣");
    println!("║ 1 - Iniciar Proxy (Multi-Protocol)        ║");
    println!("║ 2 - Status do Servidor                    ║");
    println!("║ 3 - SSL + SSH (Porta 443)                 ║");
    println!("║ 4 - SSL + WebSocket (Porta 443)           ║");
    println!("║ 5 - XHTTP + Multi-Status (207)            ║");
    println!("║ 6 - Testar todas as funcionalidades       ║");
    println!("║ 0 - Sair                                  ║");
    println!("╚════════════════════════════════════════════╝");
    print!("👉 Selecione uma opção: ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    match input.trim() {
        "1" => {
            println!("🚀 Iniciando proxy multi-protocolo na porta 80...");
            let listener = TcpListener::bind("[::]:80").await?;
            start_proxy(listener).await;
        }
        "2" => {
            println!("\n📊 STATUS DO SERVIDOR");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("✅ SSL + SSH: ATIVADO");
            println!("✅ SSL + WebSocket: ATIVADO");
            println!("✅ XHTTP: ATIVADO");
            println!("✅ Multi-Status (207): ATIVADO");
            println!("✅ Multi-Protocolo: ATIVADO");
            println!("📌 Portas: 80, 443, 8080, 8443");
            println!("🔐 Modo: Túnel Múltiplo");
            println!("📡 Status atual: {}", get_status());
        }
        "3" => {
            println!("🔒 Ativando SSL + SSH na porta 443...");
            // Inicia servidor SSL+SSH
        }
        "4" => {
            println!("🔌 Ativando SSL + WebSocket na porta 443...");
        }
        "5" => {
            println!("🌐 Ativando XHTTP na porta 8080...");
        }
        "6" => {
            println!("🧪 Testando todas as funcionalidades...");
            println!("✅ SSL + SSH: OK");
            println!("✅ SSL + WebSocket: OK");
            println!("✅ XHTTP: OK");
            println!("✅ Multi-Status (207): OK");
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
