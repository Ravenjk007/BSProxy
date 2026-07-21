use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub backends: HashMap<String, String>,
    pub status_msg: String,
    pub idle_timeout: u64,
}

pub async fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let default = Config {
        backends: [
            ("ssh".to_string(), "127.0.0.1:22".to_string()),
            ("ovpn".to_string(), "127.0.0.1:1194".to_string()),
            ("socks5".to_string(), "127.0.0.1:1080".to_string()),
        ].into_iter().collect(),
        status_msg: "@BSPROXY".to_string(),
        idle_timeout: 300,
    };
    Ok(default) // Você pode carregar de config.toml depois
}
