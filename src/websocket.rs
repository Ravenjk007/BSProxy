use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt, copy_bidirectional};
use tokio::net::TcpStream;
use anyhow::Result;
use sha1::{Sha1, Digest};
use base64::{engine::general_purpose, Engine as _};
use log::{info, warn, error};

/// Handler principal para WebSocket com handshake completo
pub async fn handle_websocket_stream<S>(mut stream: S, target_addr: &str) -> Result<()> 
where S: AsyncRead + AsyncWrite + Unpin 
{
    let mut buffer = [0u8; 4096];
    let n = stream.read(&mut buffer).await?;
    let request = String::from_utf8_lossy(&buffer[..n]);

    info!("🌐 WebSocket request recebida");

    if request.contains("Upgrade: websocket") {
        let mut key = "";
        for line in request.lines() {
            let line_lower = line.to_lowercase();
            if line_lower.starts_with("sec-websocket-key:") {
                key = line.split(':').nth(1).unwrap_or("").trim();
                break;
            }
        }

        // Se não encontrou a chave, tenta pegar de forma mais flexível
        if key.is_empty() {
            for line in request.lines() {
                if line.contains("Sec-WebSocket-Key") || line.contains("sec-websocket-key") {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() >= 2 {
                        key = parts[1].trim();
                        break;
                    }
                }
            }
        }

        // Se ainda não tem chave, usa uma padrão para teste
        if key.is_empty() {
            warn!("⚠️ Sec-WebSocket-Key não encontrada, usando padrão");
            key = "dGhlIHNhbXBsZSBub25jZQ==";
        }

        let mut hasher = Sha1::new();
        hasher.update(key.as_bytes());
        hasher.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
        let result = hasher.finalize();
        let accept_key = general_purpose::STANDARD.encode(result);

        let response = format!(
            "HTTP/1.1 101 Switching Protocols\r\n\
             Upgrade: websocket\r\n\
             Connection: Upgrade\r\n\
             Sec-WebSocket-Accept: {}\r\n\
             X-Supported: SSL,SSH,WebSocket,XHTTP\r\n\
             \r\n",
            accept_key
        );
        stream.write_all(response.as_bytes()).await?;
        info!("✅ WebSocket handshake completo");
    }

    // Conecta ao alvo e faz proxy
    let mut server_stream = TcpStream::connect(target_addr).await?;
    info!("🔗 Conectado ao backend: {}", target_addr);
    copy_bidirectional(&mut stream, &mut server_stream).await?;
    
    Ok(())
}

/// Handler para WebSocket com SSL/TLS
pub async fn handle_websocket_ssl_stream<S>(stream: S, target_addr: &str) -> Result<()> 
where S: AsyncRead + AsyncWrite + Unpin 
{
    info!("🔒 WebSocket com SSL/TLS");
    handle_websocket_stream(stream, target_addr).await
}

/// Handler para XHTTP com Multi-Status (207)
pub async fn handle_xhttp_stream<S>(mut stream: S) -> Result<()> 
where S: AsyncRead + AsyncWrite + Unpin 
{
    let mut buffer = [0u8; 4096];
    let n = stream.read(&mut buffer).await?;
    let request = String::from_utf8_lossy(&buffer[..n]);
    
    info!("🌐 XHTTP request recebida");
    
    // Detecta se é XHTTP ou WebSocket
    if request.contains("Upgrade: websocket") || request.contains("Sec-WebSocket-Key") {
        info!("🔄 Redirecionando para WebSocket");
        return handle_websocket_stream(stream, "127.0.0.1:22").await;
    }
    
    // Resposta Multi-Status (207)
    let response = format!(
        "HTTP/1.1 207 Multi-Status\r\n\
         Content-Type: application/json\r\n\
         X-Supported: SSL,SSH,WebSocket,XHTTP\r\n\
         X-Protocol: XHTTP\r\n\
         X-Status: connected\r\n\
         \r\n\
         {{
             \"status\": 207,
             \"message\": \"Multi-Status - Multiple protocols supported\",
             \"protocols\": [\"SSL\", \"SSH\", \"WebSocket\", \"XHTTP\"],
             \"ports\": [80, 443, 8080, 8443],
             \"connection\": \"established\",
             \"detected_protocol\": \"XHTTP\"
         }}\n"
    );
    
    stream.write_all(response.as_bytes()).await?;
    info!("✅ XHTTP Multi-Status (207) enviado");
    
    Ok(())
}

/// Handler para XHTTP com SSL
pub async fn handle_xhttp_ssl_stream<S>(stream: S) -> Result<()> 
where S: AsyncRead + AsyncWrite + Unpin 
{
    info!("🔒 XHTTP com SSL/TLS");
    handle_xhttp_stream(stream).await
}

/// Detector de protocolo baseado nos dados iniciais
pub fn detect_protocol(data: &[u8]) -> &'static str {
    let data_str = String::from_utf8_lossy(data);
    
    // TLS/SSL: começa com 0x16 0x03
    if data.len() >= 2 && data[0] == 0x16 && data[1] == 0x03 {
        return "TLS";
    }
    
    // SSH
    if data_str.contains("SSH-") {
        return "SSH";
    }
    
    // WebSocket
    if data_str.contains("Upgrade: websocket") || data_str.contains("Sec-WebSocket-Key") {
        return "WEBSOCKET";
    }
    
    // XHTTP
    if data_str.contains("X-") || data_str.contains("XHTTP") {
        return "XHTTP";
    }
    
    // HTTP/HTTPS
    if data_str.contains("HTTP/") || data_str.contains("GET ") || data_str.contains("POST ") {
        return "HTTP";
    }
    
    "UNKNOWN"
}
