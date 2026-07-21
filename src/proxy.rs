use crate::protocol::{MultiProtocolHandler, ProtocolDetector};
use crate::tls::TLSHandler;
use crate::ssh::SSHHandler;
use crate::websocket::WebSocketHandler;
use crate::xhttp::XHTTPHandler;

pub struct Proxy {
    port: u16,
    multi_protocol: bool,
    ssl_support: bool,
    ssh_support: bool,
    websocket_support: bool,
    xhttp_support: bool,
}

impl Proxy {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            multi_protocol: true,
            ssl_support: true,
            ssh_support: true,
            websocket_support: true,
            xhttp_support: true,
        }
    }

    pub async fn run(&self) -> Result<(), anyhow::Error> {
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", self.port)).await?;
        log::info!("BSProxy listening on port {}", self.port);

        let handler = MultiProtocolHandler::new();

        loop {
            let (stream, addr) = listener.accept().await?;
            log::info!("New connection from {}", addr);

            let handler_clone = handler.clone();
            
            tokio::spawn(async move {
                if let Err(e) = handler_clone.handle_connection(stream).await {
                    log::error!("Error handling connection: {}", e);
                }
            });
        }
    }
}

// Suporte a XHTTP na porta 443
pub async fn run_xhttp_server() -> Result<(), anyhow::Error> {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:443").await?;
    log::info!("XHTTP server listening on port 443");

    let websocket_handler = WebSocketHandler::new();

    loop {
        let (stream, addr) = listener.accept().await?;
        log::info!("XHTTP connection from {}", addr);

        let handler = websocket_handler.clone();
        
        tokio::spawn(async move {
            if let Err(e) = handler.handle_xhttp_connection(stream).await {
                log::error!("Error handling XHTTP: {}", e);
            }
        });
    }
}
