use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt, copy_bidirectional};
use tokio::net::TcpStream;
use anyhow::Result;
use sha1::{Sha1, Digest};
use base64::{engine::general_purpose, Engine as _};

pub async fn handle_websocket_stream<S>(mut stream: S, target_addr: &str) -> Result<()> 
where S: AsyncRead + AsyncWrite + Unpin 
{
    let mut buffer = [0u8; 4096];
    let n = stream.read(&mut buffer).await?;
    let request = String::from_utf8_lossy(&buffer[..n]);

    if request.contains("Upgrade: websocket") {
        let mut key = "";
        for line in request.lines() {
            if line.to_lowercase().starts_with("sec-websocket-key:") {
                key = line.split(':').nth(1).unwrap_or("").trim();
                break;
            }
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
             Sec-WebSocket-Accept: {}\r\n\r\n",
            accept_key
        );
        stream.write_all(response.as_bytes()).await?;
    }

    let mut server_stream = TcpStream::connect(target_addr).await?;
    copy_bidirectional(&mut stream, &mut server_stream).await?;
    Ok(())
}
