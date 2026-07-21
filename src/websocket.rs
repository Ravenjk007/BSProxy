cd /tmp/BSProxy

cat > src/websocket.rs << 'EOF'
use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
use anyhow::{Result, anyhow};
use log::{info, warn, error, debug};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Lê e descarta os headers HTTP até encontrar \r\n\r\n
async fn consume_http_headers(socket: &mut TcpStream) -> std::io::Result<()> {
    let mut buf: Vec<u8> = Vec::new();
    let mut tmp = [0u8; 1];

    loop {
        socket.read_exact(&mut tmp).await?;
        buf.push(tmp[0]);

        if buf.len() >= 4 && &buf[buf.len() - 4..] == b"\r\n\r\n" {
            break;
        }
        if buf.len() > 8192 {
            break;
        }
    }
    Ok(())
}

/// Detecta se é WebSocket ou XHTTP
fn detect_websocket_or_xhttp(data: &[u8]) -> (&str, bool) {
    let data_str = String::from_utf8_lossy(data);
    
    if data_str.contains("Upgrade: websocket") || data_str.contains("Sec-WebSocket-Key") {
        return ("WEBSOCKET", true);
    }
    
    if data_str.contains("X-") || data_str.contains("XHTTP") {
        return ("XHTTP", false);
    }
    
    if data_str.contains("GET /") || data_str.contains("POST /") || data_str.contains("CONNECT") {
        return ("HTTP", false);
    }
    
    ("UNKNOWN", false)
}

/// Handler principal para WebSocket
pub async fn handle_websocket(mut socket: TcpStream) -> Result<()> {
    info!("🌐 WebSocket/HTTP handshake...");
    
    consume_http_headers(&mut socket).await?;
    
    let response = "HTTP/1.1 101 Switching Protocols\r\n\
                    Upgrade: websocket\r\n\
                    Connection: Upgrade\r\n\
                    Sec-WebSocket-Accept: dGhlIHNhbXBsZSBub25jZQ==\r\n\
                    X-Supported: SSL,SSH,WebSocket,XHTTP\r\n\
                    \r\n";
    
    socket.write_all(response.as_bytes()).await?;
    info!("🌐 WebSocket handshake complete! Encaminhando para SSH...");
    
    let target = "127.0.0.1:22";
    
    match TcpStream::connect(target).await {
        Ok(remote) => {
            info!("✅ Conectado ao SSH na porta 22");
            let (mut client_reader, mut client_writer) = socket.into_split();
            let (mut remote_reader, mut remote_writer) = remote.into_split();
            
            tokio::try_join!(
                tokio::io::copy(&mut client_reader, &mut remote_writer),
                tokio::io::copy(&mut remote_reader, &mut client_writer)
            )?;
            
            info!("🔚 Conexão WebSocket->SSH encerrada");
            Ok(())
        }
        Err(e) => {
            error!("❌ Falha ao conectar ao SSH: {}", e);
            Err(anyhow!("SSH connection failed: {}", e))
        }
    }
}

/// Handler para WebSocket com SSL/TLS
pub async fn handle_websocket_ssl(mut socket: TcpStream) -> Result<()> {
    info!("🔒 WebSocket com SSL/TLS handshake...");
    
    consume_http_headers(&mut socket).await?;
    
    let response = "HTTP/1.1 101 Switching Protocols\r\n\
                    Upgrade: websocket\r\n\
                    Connection: Upgrade\r\n\
                    Sec-WebSocket-Accept: dGhlIHNhbXBsZSBub25jZQ==\r\n\
                    X-SSL: enabled\r\n\
                    X-Supported: SSL,SSH,WebSocket,XHTTP\r\n\
                    \r\n";
    
    socket.write_all(response.as_bytes()).await?;
    info!("🔒 WebSocket SSL handshake complete!");
    
    let target = "127.0.0.1:443";
    
    match TcpStream::connect(target).await {
        Ok(remote) => {
            info!("✅ Conectado ao SSH via SSL na porta 443");
            let (mut client_reader, mut client_writer) = socket.into_split();
            let (mut remote_reader, mut remote_writer) = remote.into_split();
            
            tokio::try_join!(
                tokio::io::copy(&mut client_reader, &mut remote_writer),
                tokio::io::copy(&mut remote_reader, &mut client_writer)
            )?;
            
            info!("🔚 Conexão WebSocket SSL encerrada");
            Ok(())
        }
        Err(e) => {
            error!("❌ Falha ao conectar ao SSH SSL: {}", e);
            Err(anyhow!("SSL connection failed: {}", e))
        }
    }
}

/// Handler para XHTTP (com Multi-Status)
pub async fn handle_xhttp(mut socket: TcpStream) -> Result<()> {
    info!("🌐 XHTTP request recebida...");
    
    let mut buffer = [0u8; 4096];
    let n = socket.read(&mut buffer).await?;
    
    if n == 0 {
        return Err(anyhow!("Empty request"));
    }
    
    let request = String::from_utf8_lossy(&buffer[..n]);
    info!("📨 XHTTP Request: {}", request.lines().next().unwrap_or(""));
    
    let (protocol, _is_websocket) = detect_websocket_or_xhttp(&buffer[..n]);
    info!("🔍 Protocolo detectado: {}", protocol);
    
    if _is_websocket {
        info!("🔄 Redirecionando para WebSocket handler");
        return handle_websocket(socket).await;
    }
    
    let response = format!(
        "HTTP/1.1 207 Multi-Status\r\n\
         Content-Type: application/json\r\n\
         X-Supported: SSL,SSH,WebSocket,XHTTP\r\n\
         X-Protocol: {}\r\n\
         X-Status: connected\r\n\
         \r\n\
         {{
             \"status\": 207,
             \"message\": \"Multi-Status - Multiple protocols supported\",
             \"protocols\": [\"SSL\", \"SSH\", \"WebSocket\", \"XHTTP\"],
             \"ports\": [80, 443, 8080, 8443],
             \"connection\": \"established\",
             \"detected_protocol\": \"{}\"
         }}",
        protocol, protocol
    );
    
    socket.write_all(response.as_bytes()).await?;
    info!("✅ XHTTP Multi-Status (207) enviado");
    
    let target = match protocol {
        "WEBSOCKET" => "127.0.0.1:8080",
        "XHTTP" => "127.0.0.1:8443",
        _ => "127.0.0.1:22",
    };
    
    info!("🔄 Encaminhando para {}", target);
    
    match TcpStream::connect(target).await {
        Ok(remote) => {
            let (mut client_reader, mut client_writer) = socket.into_split();
            let (mut remote_reader, mut remote_writer) = remote.into_split();
            
            tokio::try_join!(
                tokio::io::copy(&mut client_reader, &mut remote_writer),
                tokio::io::copy(&mut remote_reader, &mut client_writer)
            )?;
            
            info!("🔚 XHTTP conexão encerrada");
            Ok(())
        }
        Err(e) => {
            warn!("⚠️ Falha ao conectar ao backend {}: {}", target, e);
            Ok(())
        }
    }
}

/// Handler para XHTTP com SSL
pub async fn handle_xhttp_ssl(mut socket: TcpStream) -> Result<()> {
    info!("🔒 XHTTP com SSL/TLS...");
    
    let mut buffer = [0u8; 4096];
    let _n = socket.read(&mut buffer).await?;
    
    let response = "HTTP/1.1 207 Multi-Status\r\n\
                    X-SSL: enabled\r\n\
                    X-Supported: SSL,SSH,WebSocket,XHTTP\r\n\
                    \r\n\
                    {\"status\":207,\"message\":\"SSL + XHTTP Multi-Status\"}";
    
    socket.write_all(response.as_bytes()).await?;
    info!("✅ XHTTP SSL Multi-Status (207) enviado");
    
    let target = "127.0.0.1:443";
    
    match TcpStream::connect(target).await {
        Ok(remote) => {
            let (mut client_reader, mut client_writer) = socket.into_split();
            let (mut remote_reader, mut remote_writer) = remote.into_split();
            
            tokio::try_join!(
                tokio::io::copy(&mut client_reader, &mut remote_writer),
                tokio::io::copy(&mut remote_reader, &mut client_writer)
            )?;
            
            Ok(())
        }
        Err(e) => {
            error!("❌ Falha ao conectar ao SSL backend: {}", e);
            Err(anyhow!("SSL backend connection failed: {}", e))
        }
    }
}

/// Função para detectar protocolo e rotear
pub async fn handle_multi_protocol(mut socket: TcpStream) -> Result<()> {
    info!("🔄 Multi-Protocol handler...");
    
    let mut buffer = [0u8; 4096];
    
    let n = match timeout(Duration::from_secs(5), socket.read(&mut buffer)).await {
        Ok(Ok(n)) => n,
        Ok(Err(e)) => return Err(anyhow!("Read error: {}", e)),
        Err(_) => {
            info!("⏰ Timeout - assumindo SSH");
            return handle_websocket(socket).await;
        }
    };
    
    if n == 0 {
        return Err(anyhow!("Empty request"));
    }
    
    let (protocol, _is_websocket) = detect_websocket_or_xhttp(&buffer[..n]);
    info!("🔍 Protocolo detectado: {}", protocol);
    
    match protocol {
        "WEBSOCKET" => {
            info!("🔌 Redirecionando para WebSocket");
            handle_websocket(socket).await
        }
        "XHTTP" | "HTTP" => {
            info!("🌐 Redirecionando para XHTTP");
            handle_xhttp(socket).await
        }
        _ => {
            info!("🔑 Fallback para WebSocket->SSH");
            handle_websocket(socket).await
        }
    }
}
EOF
