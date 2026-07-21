pub struct ProtocolDetector;

impl ProtocolDetector {
    pub fn detect(data: &[u8]) -> String {
        let text = String::from_utf8_lossy(data);

        if data.starts_with(b"SSH-") || text.contains("SSH") {
            "ssh".to_string()
        } else if data.starts_with(&[0x00]) && data.len() > 2 { // OpenVPN signature aproximada
            "openvpn".to_string()
        } else if text.contains("SOCKS") || data.starts_with(b"\x05") {
            "socks5".to_string()
        } else {
            "ssh".to_string()
        }
    }
}
