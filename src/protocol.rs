use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    TLS,
    SSH,
    WebSocket,
    XHTTP,
    Unknown,
}

pub struct ProtocolDetector;

impl ProtocolDetector {
    pub async fn detect_protocol<T: AsyncRead + Unpin>(
        reader: &mut T,
    ) -> Result<Protocol, anyhow::Error> {
        let mut buf = [0u8; 1024];
        let n = reader.read(&mut buf).await?;
        
        if n == 0 {
            return Ok(Protocol::Unknown);
        }

        // TLS handshake: 0x16 0x03
        if n >= 2 && buf[0] == 0x16 && (buf[1] == 0x03) {
            return Ok(Protocol::TLS);
        }

        // SSH banner: "SSH-"
        if n >= 4 && &buf[0..4] == b"SSH-" {
            return Ok(Protocol::SSH);
        }

        // HTTP/WebSocket
        let data = String::from_utf8_lossy(&buf[0..n]);
        if data.starts_with("GET /") || data.starts_with("CONNECT") || data.starts_with("POST /") {
            if data.contains("Upgrade: websocket") || data.contains("Sec-WebSocket-Key") {
                return Ok(Protocol::WebSocket);
            }
            return Ok(Protocol::XHTTP);
        }

        Ok(Protocol::Unknown)
    }
}

// Estrutura para gerenciar múltiplos protocolos
pub struct MultiProtocolHandler {
    tls_handler: Arc<TLSHandler>,
    ssh_handler: Arc<SSHHandler>,
    websocket_handler: Arc<WebSocketHandler>,
    xhttp_handler: Arc<XHTTPHandler>,
}

impl MultiProtocolHandler {
    pub fn new() -> Self {
        Self {
            tls_handler: Arc::new(TLSHandler::new()),
            ssh_handler: Arc::new(SSHHandler::new()),
            websocket_handler: Arc::new(WebSocketHandler::new()),
            xhttp_handler: Arc::new(XHTTPHandler::new()),
        }
    }

    pub async fn handle_connection<T: AsyncRead + AsyncWrite + Unpin + Send + 'static>(
        &self,
        mut stream: T,
    ) -> Result<(), anyhow::Error> {
        // Detecta o protocolo
        let protocol = ProtocolDetector::detect_protocol(&mut stream).await?;
        
        match protocol {
            Protocol::TLS => {
                log::info!("Detected TLS connection");
                self.tls_handler.handle(stream).await?;
            }
            Protocol::SSH => {
                log::info!("Detected SSH connection");
                self.ssh_handler.handle(stream).await?;
            }
            Protocol::WebSocket => {
                log::info!("Detected WebSocket connection");
                self.websocket_handler.handle(stream).await?;
            }
            Protocol::XHTTP => {
                log::info!("Detected XHTTP connection");
                self.xhttp_handler.handle(stream).await?;
            }
            Protocol::Unknown => {
                log::warn!("Unknown protocol detected");
                // Tenta fallback TCP
                // handle_tcp_fallback(stream).await?;
            }
        }
        
        Ok(())
    }
}
