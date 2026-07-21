let proto = ProtocolDetector::detect(&buf[..n]);

match proto.as_str() {
    "socks5" => {
        // Passa o controle para o handler específico
        return crate::socks5::handle_socks5(client).await;
    }
    "tls" => {
        // Futuro: tls::handle_tls(...)
    }
    _ => {
        // Proxy TCP normal (SSH, OVPN, etc)
        let target = config.backends.get(&proto).unwrap_or(&config.backends["ssh"]);
        // ... resto do código de proxy normal
    }
}
