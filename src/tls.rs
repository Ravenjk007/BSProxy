use tokio::net::TcpStream;
use anyhow::Result;
use log::info;
use tokio::io::AsyncWriteExt;

pub async fn handle_tls(mut socket: TcpStream) -> Result<()> {
    info!("🔒 TLS/SECURITY");
    socket.write_all(b"TLS/SECURITY OK\n").await?;
    Ok(())
}
