#!/bin/bash

echo "🔧 Instalando BSProxy Multiprotocol..."

# Instalar Rust se não tiver
if ! command -v cargo &> /dev/null; then
    echo "📦 Instalando Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

# Compilar o projeto
echo "📦 Compilando BSProxy..."
cargo build --release

# Copiar para /usr/local/bin (opcional)
if [ -f "./target/release/bsproxy" ]; then
    echo "📦 Instalando bsproxy no sistema..."
    sudo cp ./target/release/bsproxy /usr/local/bin/
    sudo chmod +x /usr/local/bin/bsproxy
    echo "✅ bsproxy instalado globalmente!"
fi

# Tornar scripts executáveis
chmod +x menu.sh

echo ""
echo "✅ Instalação concluída!"
echo "🚀 Para iniciar: ./menu.sh"
echo "💡 Ou diretamente: bsproxy -p 80"
echo ""
echo "📡 Protocolos suportados:"
echo "   - SOCKS5 (byte 0x05)"
echo "   - TLS/SECURITY (byte 0x16)"
echo "   - TCP Fallback (qualquer outro)"
