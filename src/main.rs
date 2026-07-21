mod tls;
mod websocket;
mod proxy;

use std::env;
use tokio::net::TcpListener;
use std::error::Error;
use crate::proxy::handle_connection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let port = get_arg("--port", "80").parse::<u16>()?;
    let status = get_arg("--status", "200 OK");
    let target = get_arg("--target", "127.0.0.1:22");
    let protocol = get_arg("--protocol", "ssh"); // ssh, ws, ovpn, etc.
    let xhttp_port = get_arg("--xhttp", "0").parse::<u16>()?;

    println!("BSProxy iniciado na porta {}", port);
    println!("Protocolo: {}, Status: {}, Alvo: {}", protocol, status, target);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    
    if xhttp_port > 0 {
        let xhttp_listener = TcpListener::bind(format!("0.0.0.0:{}", xhttp_port)).await?;
        println!("XHTTP rodando na porta {}", xhttp_port);
        tokio::spawn(async move {
            if let Err(e) = run_server(xhttp_listener, "127.0.0.1:22", "ssh+ssl", "200 OK").await {
                eprintln!("Erro no servidor XHTTP: {}", e);
            }
        });
    }

    run_server(listener, &target, &protocol, &status).await?;

    Ok(())
}

async fn run_server(listener: TcpListener, target: &str, protocol: &str, status: &str) -> Result<(), Box<dyn Error>> {
    let target = target.to_string();
    let protocol = protocol.to_string();
    let status = status.to_string();

    loop {
        let (stream, _) = listener.accept().await?;
        let target_clone = target.clone();
        let protocol_clone = protocol.clone();
        let status_clone = status.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, &target_clone, &protocol_clone, &status_clone).await {
                eprintln!("Erro ao lidar com conexão: {}", e);
            }
        });
    }
}

fn get_arg(name: &str, default: &str) -> String {
    let args: Vec<String> = env::args().collect();
    for i in 0..args.len() {
        if args[i] == name && i + 1 < args.len() {
            return args[i + 1].clone();
        }
    }
    default.to_string()
}
