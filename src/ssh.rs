use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt, copy_bidirectional};
use std::error::Error;
use log::info;

pub async fn handle_ssh_tunnel(
    mut client_stream: TcpStream,
    target_addr: &str,
) -> Result<(), Box<dyn Error>> {
    info!("🔑 Estabelecendo túnel SSH para {}", target_addr);
    
    let mut server_stream = TcpStream::connect(target_addr).await?;
    copy_bidirectional(&mut client_stream, &mut server_stream).await?;
    
    Ok(())
}
