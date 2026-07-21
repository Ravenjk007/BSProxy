pub struct ProtocolDetector;

impl ProtocolDetector {
    pub fn detect(data: &[u8]) -> String {
        if data.is_empty() {
            return "ssh".to_string();
        }

        match data[0] {
            0x05 => "socks5".to_string(),           // SOCKS5
            0x16 if data.len() > 2 && data[1] == 0x03 => "tls".to_string(), // TLS
            _ => {
                let text = String::from_utf8_lossy(data).to_uppercase();
                if text.contains("SSH") {
                    "ssh".to_string()
                } else if text.contains("OPENVPN") || data.starts_with(&[0x00, 0x00]) {
                    "ovpn".to_string()
                } else {
                    "ssh".to_string()
                }
            }
        }
    }
}
