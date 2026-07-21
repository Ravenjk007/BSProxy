use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub backends: HashMap<String, String>,
    pub status_msg: String,
    pub idle_timeout: u64,
    pub max_connections: usize,
    pub whitelist: Vec<String>,
}

impl Config {
    pub async fn load() -> Result<Self, Box<dyn std::error::Error>> {
        if let Ok(content) = fs::read_to_string("config.toml") {
            toml::from_str(&content).map_err(|e| e.into())
        } else {
            // Configuração padrão
            Ok(Config {
                backends: [
                    ("ssh".to_string(), "127.0.0.1:22".to_string()),
                    ("ovpn".to_string(), "127.0.0.1:1194".to_string()),
                    ("socks5".to_string(), "127.0.0.1:1080".to_string()),
                    ("tls".to_string(), "127.0.0.1:443".to_string()),
                ].into_iter().collect(),
                status_msg: DEFAULT_STATUS.to_string(),
                idle_timeout: 300,
                max_connections: 500,
                whitelist: vec![],
            })
        }
    }
}
