use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use log::{info, error};
use anyhow::Result;

pub async fn handle_socks5(mut client: TcpStream) -> Result<()> {
    info!("🔐 Iniciando handshake SOCKS5");

    // Método negotiation
    let mut header = [0u8; 2];
    client.read_exact(&mut header).await?;
    
    if header[0] != 0x05 {
        anyhow::bail!("Não é um protocolo SOCKS5 válido");
    }

    let nmethods = header[1] as usize;
    let mut methods = vec![0u8; nmethods];
    client.read_exact(&mut methods).await?;

    // Aceita sem autenticação (pode melhorar depois)
    client.write_all(&[0x05, 0x00]).await?;

    // Request
    let mut req = [0u8; 4];
    client.read_exact(&mut req).await?;

    let cmd = req[1];
    let atyp = req[3];

    if cmd != 0x01 {
        send_reply(&mut client, 0x07).await?; // Command not supported
        anyhow::bail!("Comando SOCKS não suportado: {}", cmd);
    }

    let target_addr = match atyp {
        0x01 => { // IPv4
            let mut addr = [0u8; 4];
            client.read_exact(&mut addr).await?;
            let port = read_port(&mut client).await?;
            format!("{}.{}.{}.{}:{}", addr[0], addr[1], addr[2], addr[3], port)
        }
        0x03 => { // Domain name
            let mut len_buf = [0u8; 1];
            client.read_exact(&mut len_buf).await?;
            let len = len_buf[0] as usize;

            let mut domain = vec![0u8; len];
            client.read_exact(&mut domain).await?;
            let port = read_port(&mut client).await?;
            format!("{}:{}", String::from_utf8_lossy(&domain), port)
        }
        0x04 => { // IPv6 (simplificado)
            send_reply(&mut client, 0x08).await?;
            anyhow::bail!("IPv6 ainda não suportado");
        }
        _ => {
            send_reply(&mut client, 0x08).await?;
            anyhow::bail!("Tipo de endereço não suportado");
        }
    };

    info!("SOCKS5 → {}", target_addr);

    // Conecta no destino
    match TcpStream::connect(&target_addr).await {
        Ok(mut remote) => {
            send_reply(&mut client, 0x00).await?; // Success

            // Bidirectional copy
            let (mut client_r, mut client_w) = client.into_split();
            let (mut remote_r, mut remote_w) = remote.into_split();

            tokio::try_join!(
                tokio::io::copy(&mut client_r, &mut remote_w),
                tokio::io::copy(&mut remote_r, &mut client_w)
            )?;

            Ok(())
        }
        Err(e) => {
            error!("Falha ao conectar em {}: {}", target_addr, e);
            send_reply(&mut client, 0x05).await?; // Connection refused
            Err(e.into())
        }
    }
}

async fn read_port(client: &mut TcpStream) -> std::io::Result<u16> {
    let mut port_buf = [0u8; 2];
    client.read_exact(&mut port_buf).await?;
    Ok(u16::from_be_bytes(port_buf))
}

async fn send_reply(client: &mut TcpStream, code: u8) -> std::io::Result<()> {
    let reply = [
        0x05, code, 0x00, 0x01,
        0, 0, 0, 0,     // IP fictício
        0, 0            // Porta fictícia
    ];
    client.write_all(&reply).await
}
