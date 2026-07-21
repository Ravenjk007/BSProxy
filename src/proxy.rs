use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt, copy_bidirectional};
use std::error::Error;
use crate::tls::handle_tls_stream;
use crate::websocket::handle_websocket_stream;

pub async fn handle_connection(mut client_stream: TcpStream, target_addr: &str, protocol: &str, status: &str) -> Result<(), Box<dyn Error>> {
    // Suporte Multi-Status: Se o status contiver '|', escolhemos um aleatoriamente ou usamos o primeiro
    let status_to_send = if status.contains('|') {
        status.split('|').next().unwrap_or("200 OK")
    } else {
        status
    };

    match protocol {
        "ssh" | "ssh+ssl" => {
            // Se for SSL, primeiro fazemos o handshake
            if protocol.contains("ssl") {
                let mut tls_stream = handle_tls_stream(client_stream).await?;
                let mut server_stream = TcpStream::connect(target_addr).await?;
                copy_bidirectional(&mut tls_stream, &mut server_stream).await?;
            } else {
                let mut server_stream = TcpStream::connect(target_addr).await?;
                // Enviar status se necessário (HTTP Proxy style)
                client_stream.write_all(format!("HTTP/1.1 101 {}\r\n\r\n", status_to_send).as_bytes()).await?;
                copy_bidirectional(&mut client_stream, &mut server_stream).await?;
            }
        },
        "websocket" | "ws+ssl" => {
             if protocol.contains("ssl") {
                let tls_stream = handle_tls_stream(client_stream).await?;
                handle_websocket_stream(tls_stream, target_addr).await?;
            } else {
                handle_websocket_stream(client_stream, target_addr).await?;
            }
        },
        "openvpn" | "ovpn+ssl" => {
            if protocol.contains("ssl") {
                let mut tls_stream = handle_tls_stream(client_stream).await?;
                let mut server_stream = TcpStream::connect(target_addr).await?;
                copy_bidirectional(&mut tls_stream, &mut server_stream).await?;
            } else {
                let mut server_stream = TcpStream::connect(target_addr).await?;
                copy_bidirectional(&mut client_stream, &mut server_stream).await?;
            }
        },
        "security" | "security+ssl" => {
            if protocol.contains("ssl") {
                let mut tls_stream = handle_tls_stream(client_stream).await?;
                // Handshake Security: Enviar Upgrade Security
                tls_stream.write_all(b"HTTP/1.1 200 OK\r\nUpgrade: security\r\n\r\n").await?;
                let mut server_stream = TcpStream::connect(target_addr).await?;
                copy_bidirectional(&mut tls_stream, &mut server_stream).await?;
            } else {
                client_stream.write_all(b"HTTP/1.1 200 OK\r\nUpgrade: security\r\n\r\n").await?;
                let mut server_stream = TcpStream::connect(target_addr).await?;
                copy_bidirectional(&mut client_stream, &mut server_stream).await?;
            }
        },
        _ => {
            let mut server_stream = TcpStream::connect(target_addr).await?;
            copy_bidirectional(&mut client_stream, &mut server_stream).await?;
        }
    }
    Ok(())
}
