use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use anyhow::Result;
use log::info;

pub async fn handle(mut socket: TcpStream) -> Result<()> {
    info!("📦 TCP");
    socket.write_all(b"TCP OK\n").await?;
    Ok(())
}
