use tokio::net::TcpStream;
use anyhow::Result;
use log::info;

pub async fn handle(socket: TcpStream) -> Result<()> {
    info!("🌐 WebSocket (delegating to HTTP handler)");
    super::http_handler::handle(socket).await
}
