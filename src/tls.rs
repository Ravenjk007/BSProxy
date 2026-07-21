use tokio::net::TcpStream;
use tokio::io::{copy_bidirectional};
use std::error::Error;
use log::info;

pub async fn handle_tls_stream(
    mut client_stream: TcpStream,
    target_addr: &str,
) -> Result<(), Box<dyn Error>> {
    info!("🔒 TLS Handler para {}", target_addr);
    
    let mut server_stream = TcpStream::connect(target_addr).await?;
    copy_bidirectional(&mut client_stream, &mut server_stream).await?;
    
    Ok(())
}
