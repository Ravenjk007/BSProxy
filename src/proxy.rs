use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt, copy_bidirectional};
use std::error::Error;
use std::sync::Arc;
use tokio::time::{timeout, Duration};
use log::{info, warn, error, debug};

use crate::tls::{handle_tls_stream, get_tls_acceptor};
use crate::websocket::{handle_websocket, handle_websocket_ssl, handle_xhttp, handle_xhttp_ssl};
use crate::ssh::handle_ssh_tunnel;
use crate::protocol::detect_protocol;

/// Handler principal de conexão - Multi-Protocol
pub async fn handle_connection(
    mut client_stream: TcpStream, 
    target_addr: &str, 
    _protocol: &str, 
    status: &str
) -> Result<(), Box<dyn Error>> {
    let mut buffer = [0u8; 4096];
    
    // Suporte Multi-Status
    let status_to_send = if status.contains('|') {
        status.split('|').next().unwrap_or("200 OK")
    } else {
        status
    };

    info!("📥 Nova conexão recebida");

    // Tentar ler o início da conexão (peek) para detectar protocolo
    let n = match timeout(Duration::from_millis(1000), client_stream.peek(&mut buffer)).await {
        Ok(Ok(n)) => n,
        _ => 0,
    };

    if n == 0 {
        info!("⏰ Timeout ou dados vazios - assumindo SSH");
        return handle_ssh_fallback(client_stream, target_addr).await;
    }

    let request = String::from_utf8_lossy(&buffer[..n]);
    debug!("📨 Dados iniciais: {} bytes", n);
    
    // ========================================
    // 1. DETECÇÃO DE TLS/SSL (0x16)
    // ========================================
    if n > 0 && buffer[0] == 0x16 {
        info!("🔒 TLS/SSL detectado");
        return handle_tls_connection(client_stream, target_addr, &request).await;
    }

    // ========================================
    // 2. DETECÇÃO DE SSH
    // ========================================
    if request.contains("SSH-") {
        info!("🔑 SSH detectado");
        return handle_ssh_tunnel(client_stream, target_addr).await;
    }

    // ========================================
    // 3. DETECÇÃO DE WEBSOCKET
    // ========================================
    if request.contains("Upgrade: websocket") || request.contains("Sec-WebSocket-Key") {
        info!("🔌 WebSocket detectado");
        return handle_websocket_connection(client_stream, target_addr, &request).await;
    }

    // ========================================
    // 4. DETECÇÃO DE XHTTP (cabeçalhos customizados)
    // ========================================
    if request.contains("X-") || request.contains("XHTTP") {
        info!("🌐 XHTTP detectado");
        return handle_xhttp_connection(client_stream, target_addr, &request, status_to_send).await;
    }

    // ========================================
    // 5. DETECÇÃO DE HTTP/HTTPS NORMAL
    // ========================================
    if request.contains("HTTP/") || request.contains("GET ") || request.contains("POST ") || request.contains("CONNECT") {
        info!("🌐 HTTP/HTTPS detectado");
        return handle_http_connection(client_stream, target_addr, &request, status_to_send).await;
    }

    // ========================================
    // 6. FALLBACK: SSH direto
    // ========================================
    info!("❓ Protocolo desconhecido - fallback SSH");
    handle_ssh_fallback(client_stream, target_addr).await
}

// ========================================
// HANDLER TLS/SSL (com suporte a WebSocket e XHTTP)
// ========================================
async fn handle_tls_connection(
    client_stream: TcpStream,
    target_addr: &str,
    request: &str,
) -> Result<(), Box<dyn Error>> {
    info!("🔒 Processando conexão TLS/SSL");
    
    let acceptor = get_tls_acceptor().await?;
    let mut tls_stream = acceptor.accept(client_stream).await?;
    
    let mut inner_buffer = [0u8; 4096];
    let ni = match timeout(Duration::from_millis(1000), tls_stream.read(&mut inner_buffer)).await {
        Ok(Ok(ni)) => ni,
        _ => 0,
    };
    
    if ni == 0 {
        info!("⏰ TLS timeout - encaminhando para SSH");
        let mut server_stream = TcpStream::connect(target_addr).await?;
        copy_bidirectional(&mut tls_stream, &mut server_stream).await?;
        return Ok(());
    }
    
    let inner_request = String::from_utf8_lossy(&inner_buffer[..ni]);
    info!("📨 TLS inner request: {}", inner_request.lines().next().unwrap_or(""));
    
    // TLS + WebSocket (SSL + WebSocket)
    if inner_request.contains("Upgrade: websocket") || inner_request.contains("Sec-WebSocket-Key") {
        info!("🔒🔌 TLS + WebSocket (SSL + WebSocket)");
        return handle_websocket_ssl(tls_stream).await.map_err(|e| e.into());
    }
    
    // TLS + XHTTP (SSL + XHTTP)
    if inner_request.contains("X-") || inner_request.contains("XHTTP") {
        info!("🔒🌐 TLS + XHTTP (SSL + XHTTP)");
        return handle_xhttp_ssl(tls_stream).await.map_err(|e| e.into());
    }
    
    // TLS + SSH (SSL + SSH)
    if inner_request.contains("Upgrade: security") || inner_request.contains("SSH") {
        info!("🔒🔑 TLS + SSH (SSL + SSH)");
        tls_stream.write_all(b"HTTP/1.1 200 OK\r\nUpgrade: security\r\n\r\n").await?;
        let mut server_stream = TcpStream::connect(target_addr).await?;
        copy_bidirectional(&mut tls_stream, &mut server_stream).await?;
        return Ok(());
    }
    
    // TLS puro - encaminha
    info!("🔒 TLS puro - encaminhando");
    let mut server_stream = TcpStream::connect(target_addr).await?;
    if ni > 0 {
        server_stream.write_all(&inner_buffer[..ni]).await?;
    }
    copy_bidirectional(&mut tls_stream, &mut server_stream).await?;
    
    Ok(())
}

// ========================================
// HANDLER WEBSOCKET (com suporte SSL)
// ========================================
async fn handle_websocket_connection(
    client_stream: TcpStream,
    target_addr: &str,
    request: &str,
) -> Result<(), Box<dyn Error>> {
    info!("🔌 Processando WebSocket");
    
    // Verifica se é WebSocket com SSL
    if request.contains("X-SSL:") || request.contains("wss://") {
        info!("🔒 WebSocket com SSL detectado");
        return handle_websocket_ssl(client_stream).await.map_err(|e| e.into());
    }
    
    // WebSocket normal
    handle_websocket(client_stream).await.map_err(|e| e.into())
}

// ========================================
// HANDLER XHTTP (com Multi-Status e SSL)
// ========================================
async fn handle_xhttp_connection(
    client_stream: TcpStream,
    _target_addr: &str,
    request: &str,
    status: &str,
) -> Result<(), Box<dyn Error>> {
    info!("🌐 Processando XHTTP");
    
    // Verifica se é XHTTP com SSL
    if request.contains("X-SSL:") {
        info!("🔒 XHTTP com SSL detectado");
        return handle_xhttp_ssl(client_stream).await.map_err(|e| e.into());
    }
    
    // XHTTP normal com Multi-Status (207)
    info!("🌐 XHTTP com Multi-Status");
    handle_xhttp(client_stream).await.map_err(|e| e.into())
}

// ========================================
// HANDLER HTTP NORMAL
// ========================================
async fn handle_http_connection(
    mut client_stream: TcpStream,
    target_addr: &str,
    request: &str,
    status: &str,
) -> Result<(), Box<dyn Error>> {
    info!("🌐 Processando HTTP/HTTPS");
    
    // Envia o primeiro status (101 Switching) conforme o código original
    client_stream.write_all(format!("HTTP/1.1 101 {}\r\n\r\n", status).as_bytes()).await?;
    
    // Lê o request real do cliente
    let mut buffer = [0u8; 4096];
    let _ = client_stream.read(&mut buffer).await?;
    
    // Envia o segundo status (200 OK)
    client_stream.write_all(format!("HTTP/1.1 200 {}\r\n\r\n", status).as_bytes()).await?;
    
    // Conecta ao servidor alvo
    let mut server_stream = TcpStream::connect(target_addr).await?;
    copy_bidirectional(&mut client_stream, &mut server_stream).await?;
    
    Ok(())
}

// ========================================
// HANDLER SSH FALLBACK
// ========================================
async fn handle_ssh_fallback(
    mut client_stream: TcpStream,
    target_addr: &str,
) -> Result<(), Box<dyn Error>> {
    info!("🔑 Fallback SSH para {}", target_addr);
    
    let mut server_stream = TcpStream::connect(target_addr).await?;
    copy_bidirectional(&mut client_stream, &mut server_stream).await?;
    
    Ok(())
}

// ========================================
// FUNÇÃO DE DETECÇÃO DE PROTOCOLO (PUBLIC)
// ========================================
pub async fn detect_protocol_from_stream(
    stream: &mut TcpStream,
) -> Result<String, Box<dyn Error>> {
    let mut buffer = [0u8; 1024];
    let n = stream.peek(&mut buffer).await?;
    
    if n == 0 {
        return Ok("SSH".to_string());
    }
    
    let data = String::from_utf8_lossy(&buffer[..n]);
    Ok(detect_protocol(&data))
}

// ========================================
// INICIALIZADOR DO PROXY
// ========================================
pub async fn start_proxy(
    port: u16,
    target_addr: &str,
    status: &str,
) -> Result<(), Box<dyn Error>> {
    use tokio::net::TcpListener;
    use std::sync::Arc;
    
    let listener = TcpListener::bind(format!("[::]:{}", port)).await?;
    info!("🚀 BSProxy Multi-Protocol iniciado na porta {}", port);
    info!("📡 Target: {}", target_addr);
    info!("📊 Status: {}", status);
    
    let target = Arc::new(target_addr.to_string());
    let status_str = Arc::new(status.to_string());
    
    loop {
        let (client_stream, addr) = listener.accept().await?;
        info!("📥 Conexão de: {}", addr);
        
        let target = target.clone();
        let status_str = status_str.clone();
        
        tokio::spawn(async move {
            if let Err(e) = handle_connection(client_stream, &target, "multi", &status_str).await {
                error!("❌ Erro ao processar {}: {}", addr, e);
            }
        });
    }
}
