use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use anyhow::Result;
use log::info;

/// Processa múltiplas requisições HTTP/WebSocket na mesma conexão
pub async fn handle_http(mut socket: TcpStream) -> Result<()> {
    info!("🌐 HTTP/WebSocket connection established");
    
    let mut buffer = Vec::new();
    let mut tmp = [0u8; 8192];
    let mut request_count = 0;
    
    loop {
        match socket.read(&mut tmp).await {
            Ok(0) => {
                info!("🔚 Connection closed by client");
                break;
            }
            Ok(n) => {
                buffer.extend_from_slice(&tmp[..n]);
                info!("📥 Received {} bytes", n);
                
                // Processar todas as requisições no buffer
                while let Some(response) = process_request(&mut buffer) {
                    request_count += 1;
                    info!("📤 Sending response #{} ({} bytes)", request_count, response.len());
                    
                    if let Err(e) = socket.write_all(response.as_bytes()).await {
                        info!("❌ Error writing response: {}", e);
                        break;
                    }
                    socket.flush().await?;
                }
            }
            Err(e) => {
                info!("❌ Read error: {}", e);
                break;
            }
        }
    }
    
    info!("📊 Total requests processed: {}", request_count);
    Ok(())
}

/// Processa uma requisição do buffer e retorna a resposta
fn process_request(buffer: &mut Vec<u8>) -> Option<String> {
    let data = String::from_utf8_lossy(buffer);
    
    // Procura pelo fim dos headers (\r\n\r\n)
    if let Some(header_end) = data.find("\r\n\r\n") {
        let header_part = &data[..header_end];
        let body_start = header_end + 4;
        
        // Parse da primeira linha
        let lines: Vec<&str> = header_part.lines().collect();
        if lines.is_empty() {
            return None;
        }
        
        let first_line: Vec<&str> = lines[0].split_whitespace().collect();
        if first_line.len() < 2 {
            return None;
        }
        
        let method = first_line[0];
        let path = first_line[1];
        
        // Verificar se é WebSocket (Upgrade)
        let is_websocket = header_part.contains("Upgrade: websocket") || 
                          header_part.contains("upgrade: websocket");
        
        // Verificar se é CONNECT (tunnel)
        let is_connect = method == "CONNECT";
        
        // Calcular o tamanho da requisição
        let mut content_length = 0;
        for line in &lines[1..] {
            if line.to_lowercase().contains("content-length:") {
                if let Some(len) = line.split(':').nth(1) {
                    if let Ok(l) = len.trim().parse::<usize>() {
                        content_length = l;
                        break;
                    }
                }
            }
        }
        
        let total_len = header_end + 4 + content_length;
        
        // Verificar se temos dados suficientes
        if buffer.len() < total_len {
            return None;
        }
        
        // Remover a requisição do buffer
        buffer.drain(..total_len);
        
        // Gerar resposta apropriada
        Some(generate_response(method, path, is_websocket, is_connect, &header_part))
    } else {
        None
    }
}

/// Gera resposta para a requisição
fn generate_response(method: &str, path: &str, is_websocket: bool, is_connect: bool, headers: &str) -> String {
    if is_websocket {
        // Resposta WebSocket (101 Switching Protocols)
        format!(
            "HTTP/1.1 101 Switching Protocols\r\n\
             Upgrade: websocket\r\n\
             Connection: Upgrade\r\n\
             Sec-WebSocket-Accept: dGhlIHNhbXBsZSBub25jZQ==\r\n\
             \r\n"
        )
    } else if is_connect {
        // Resposta CONNECT (tunnel estabelecido)
        format!(
            "HTTP/1.1 200 Connection established\r\n\
             Connection: keep-alive\r\n\
             \r\n"
        )
    } else if method == "HEAD" {
        // HEAD não tem corpo
        format!(
            "HTTP/1.1 200 OK\r\n\
             Server: BSProxy\r\n\
             Content-Length: 0\r\n\
             Connection: keep-alive\r\n\
             \r\n"
        )
    } else if method == "OPTIONS" {
        // OPTIONS
        format!(
            "HTTP/1.1 204 No Content\r\n\
             Server: BSProxy\r\n\
             Allow: GET, POST, PUT, DELETE, PATCH, HEAD, CONNECT, OPTIONS, TRACE\r\n\
             Connection: keep-alive\r\n\
             \r\n"
        )
    } else {
        // Resposta padrão para qualquer método
        let body = format!(
            "Method: {}\nPath: {}\nStatus: OK\nTime: {}\n",
            method,
            path,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        );
        
        format!(
            "HTTP/1.1 200 OK\r\n\
             Server: BSProxy\r\n\
             Content-Type: text/plain\r\n\
             Content-Length: {}\r\n\
             Connection: keep-alive\r\n\
             \r\n\
             {}",
            body.len(),
            body
        )
    }
}
