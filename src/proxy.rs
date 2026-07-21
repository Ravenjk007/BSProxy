use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt, copy_bidirectional};
use std::error::Error;
use crate::tls::{handle_tls_stream, get_tls_acceptor};
use crate::websocket::handle_websocket_stream;

pub async fn handle_connection(mut client_stream: TcpStream, target_addr: &str, _protocol: &str, status: &str) -> Result<(), Box<dyn Error>> {
    let mut buffer = [0u8; 1024];
    
    // Espiar o início da conexão para detectar o protocolo
    let n = client_stream.peek(&mut buffer).await?;
    let data = &buffer[..n];
    let request = String::from_utf8_lossy(data);

    // Suporte Multi-Status
    let status_to_send = if status.contains('|') {
        status.split('|').next().unwrap_or("200 OK")
    } else {
        status
    };

    // 1. Detectar se é TLS (Handshake começa com 0x16)
    if n > 0 && buffer[0] == 0x16 {
        let acceptor = get_tls_acceptor().await?;
        let mut tls_stream = acceptor.accept(client_stream).await?;
        
        // Dentro do TLS, podemos ter WS, SSH ou OVPN
        let mut inner_buffer = [0u8; 1024];
        let ni = tls_stream.read(&mut inner_buffer).await?;
        let inner_request = String::from_utf8_lossy(&inner_buffer[..ni]);

        if inner_request.contains("Upgrade: websocket") {
            // É WebSocket sobre SSL
            handle_websocket_stream(tls_stream, target_addr).await?;
        } else if inner_request.contains("Upgrade: security") {
            // É Security sobre SSL
            tls_stream.write_all(b"HTTP/1.1 200 OK\r\nUpgrade: security\r\n\r\n").await?;
            let mut server_stream = TcpStream::connect(target_addr).await?;
            copy_bidirectional(&mut tls_stream, &mut server_stream).await?;
        } else {
            // Provavelmente SSH ou OpenVPN sobre SSL
            let mut server_stream = TcpStream::connect(target_addr).await?;
            server_stream.write_all(&inner_buffer[..ni]).await?;
            copy_bidirectional(&mut tls_stream, &mut server_stream).await?;
        }
        return Ok(());
    }

    // 2. Detectar se é HTTP/WebSocket/Security (Conexão normal)
    if request.contains("Upgrade: websocket") {
        handle_websocket_stream(client_stream, target_addr).await?;
    } else if request.contains("Upgrade: security") {
        client_stream.write_all(b"HTTP/1.1 200 OK\r\nUpgrade: security\r\n\r\n").await?;
        let mut server_stream = TcpStream::connect(target_addr).await?;
        copy_bidirectional(&mut client_stream, &mut server_stream).await?;
    } else if request.contains("HTTP/") || request.contains("CONNECT") || request.contains("GET") {
        // Responder com o status e encaminhar (SSH via HTTP Proxy)
        client_stream.read(&mut buffer).await?; // Consumir o request
        client_stream.write_all(format!("HTTP/1.1 101 {}\r\n\r\n", status_to_send).as_bytes()).await?;
        let mut server_stream = TcpStream::connect(target_addr).await?;
        copy_bidirectional(&mut client_stream, &mut server_stream).await?;
    } else {
        // Protocolo desconhecido ou SSH Direto / OpenVPN
        let mut server_stream = TcpStream::connect(target_addr).await?;
        copy_bidirectional(&mut client_stream, &mut server_stream).await?;
    }

    Ok(())
}
