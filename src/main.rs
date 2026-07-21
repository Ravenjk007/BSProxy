use std::env;
use std::io::Error;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::{time::{Duration}};
use tokio::time::timeout;
use std::net::SocketAddr;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Iniciando o proxy
    let port = get_port();
    
    // Tenta abrir a porta com fallback
    let listener = try_bind_port(port).await?;
    println!("✅ Serviço iniciado com sucesso na porta: {}", port);
    println!("🌐 Proxy escutando em todas as interfaces ([::]:{})", port);
    start_http(listener).await;
    Ok(())
}

async fn try_bind_port(port: u16) -> Result<TcpListener, Error> {
    // Tenta bind nas duas interfaces
    let addresses = vec![
        format!("[::]:{}", port),  // IPv6
        format!("0.0.0.0:{}", port), // IPv4
    ];
    
    for addr in addresses {
        match TcpListener::bind(&addr).await {
            Ok(listener) => {
                println!("📡 Conectado em: {}", addr);
                return Ok(listener);
            }
            Err(e) => {
                if port < 1024 {
                    println!("⚠️  Porta {} requer privilégios especiais", port);
                    println!("💡 Tente executar com: sudo ./seu_programa --port {}", port);
                    println!("💡 Ou use uma porta acima de 1024 (ex: 8080)");
                    
                    // Tenta automaticamente com sudo se for Linux
                    #[cfg(target_os = "linux")]
                    {
                        if is_root_available() {
                            println!("🔄 Tentando reexecutar com sudo...");
                            let _ = std::process::Command::new("sudo")
                                .arg(env::current_exe().unwrap())
                                .args(env::args().skip(1))
                                .status();
                            std::process::exit(0);
                        }
                    }
                }
                println!("❌ Erro ao abrir porta {}: {}", port, e);
            }
        }
    }
    
    Err(Error::new(std::io::ErrorKind::Other, "Não foi possível abrir nenhuma porta"))
}

#[cfg(target_os = "linux")]
fn is_root_available() -> bool {
    // Verifica se sudo está disponível
    std::process::Command::new("sudo")
        .arg("-n")
        .arg("true")
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

async fn start_http(listener: TcpListener) {
    println!("🚀 Proxy em execução...");
    loop {
        match listener.accept().await {
            Ok((client_stream, addr)) => {
                println!("🔗 Nova conexão de: {}", addr);
                tokio::spawn(async move {
                    if let Err(e) = handle_client(client_stream).await {
                        println!("❌ Erro ao processar cliente {}: {}", addr, e);
                    } else {
                        println!("✅ Conexão com {} finalizada com sucesso", addr);
                    }
                });
            }
            Err(e) => {
                println!("❌ Erro ao aceitar conexão: {}", e);
            }
        }
    }
}

async fn handle_client(mut client_stream: TcpStream) -> Result<(), Error> {
    let status = get_status();
    let client_addr = client_stream.peer_addr()?;
    println!("📨 Processando cliente: {}", client_addr);
    
    // Primeiro handshake
    client_stream
        .write_all(format!("HTTP/1.1 101 {}\r\n\r\n", status).as_bytes())
        .await?;

    let mut buffer = vec![0; 1024];
    client_stream.read(&mut buffer).await?;
    client_stream
        .write_all(format!("HTTP/1.1 200 {}\r\n\r\n", status).as_bytes())
        .await?;

    // Detecta o tipo de tráfego
    let mut addr_proxy = "0.0.0.0:22";
    let result = timeout(Duration::from_secs(2), peek_stream(&mut client_stream)).await
        .unwrap_or_else(|_| Ok(String::new()));

    if let Ok(data) = result {
        if data.contains("SSH") || data.is_empty() {
            addr_proxy = "0.0.0.0:22";
            println!("🔍 Tráfego SSH detectado para {}", client_addr);
        } else {
            addr_proxy = "0.0.0.0:1194";
            println!("🔍 Tráfego OpenVPN detectado para {}", client_addr);
        }
    } else {
        addr_proxy = "0.0.0.0:22";
        println!("⏰ Timeout na detecção, usando SSH por padrão para {}", client_addr);
    }

    println!("🔄 Conectando ao proxy destino: {}", addr_proxy);
    let server_connect = TcpStream::connect(addr_proxy).await;
    if server_connect.is_err() {
        println!("❌ Erro ao conectar ao proxy destino {}: {}", addr_proxy, server_connect.err().unwrap());
        return Ok(());
    }

    let server_stream = server_connect?;
    println!("✅ Conectado ao proxy destino: {}", addr_proxy);

    let (client_read, client_write) = client_stream.into_split();
    let (server_read, server_write) = server_stream.into_split();

    let client_read = Arc::new(Mutex::new(client_read));
    let client_write = Arc::new(Mutex::new(client_write));
    let server_read = Arc::new(Mutex::new(server_read));
    let server_write = Arc::new(Mutex::new(server_write));

    let client_to_server = transfer_data(client_read, server_write);
    let server_to_client = transfer_data(server_read, client_write);

    tokio::try_join!(client_to_server, server_to_client)?;

    Ok(())
}

async fn transfer_data(
    read_stream: Arc<Mutex<tokio::net::tcp::OwnedReadHalf>>,
    write_stream: Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>,
) -> Result<(), Error> {
    let mut buffer = [0; 8192];
    loop {
        let bytes_read = {
            let mut read_guard = read_stream.lock().await;
            match read_guard.read(&mut buffer).await {
                Ok(n) => n,
                Err(e) => {
                    println!("⚠️  Erro na leitura: {}", e);
                    return Err(e);
                }
            }
        };

        if bytes_read == 0 {
            break;
        }

        let mut write_guard = write_stream.lock().await;
        if let Err(e) = write_guard.write_all(&buffer[..bytes_read]).await {
            println!("⚠️  Erro na escrita: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

async fn peek_stream(stream: &TcpStream) -> Result<String, Error> {
    let mut peek_buffer = vec![0; 8192];
    let bytes_peeked = stream.peek(&mut peek_buffer).await?;
    let data = &peek_buffer[..bytes_peeked];
    let data_str = String::from_utf8_lossy(data);
    Ok(data_str.to_string())
}

fn get_port() -> u16 {
    let args: Vec<String> = env::args().collect();
    let mut port = 8080; // Mudando padrão para 8080 (menos problemas)

    for i in 1..args.len() {
        if args[i] == "--port" {
            if i + 1 < args.len() {
                match args[i + 1].parse() {
                    Ok(p) => port = p,
                    Err(_) => println!("⚠️  Porta inválida, usando padrão: {}", port),
                }
            }
        }
    }
    
    if port < 1024 {
        println!("ℹ️  Porta {} requer privilégios especiais", port);
        println!("💡 Tente: sudo {} --port {}", env::current_exe().unwrap().display(), port);
    }
    
    port
}

fn get_status() -> String {
    let args: Vec<String> = env::args().collect();
    let mut status = String::from("SSHPRO");

    for i in 1..args.len() {
        if args[i] == "--status" {
            if i + 1 < args.len() {
                status = args[i + 1].clone();
            }
        }
    }

    status
}
