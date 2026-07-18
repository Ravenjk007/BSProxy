#!/bin/bash
REPO_URL="https://github.com/Ravenjk007/BSProxy.git"
REPO_BRANCH="main"
CMD_NAME="bsproxy"

echo "🔧 Instalando BSProxy..."

apt update -y
apt install curl build-essential git -y

if ! command -v cargo &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

rm -rf /root/BSProxy
git clone --branch "$REPO_BRANCH" "$REPO_URL" /root/BSProxy

if [ -f /root/BSProxy/menu.sh ]; then
    cp /root/BSProxy/menu.sh /opt/bsproxy/menu
    chmod +x /opt/bsproxy/menu
fi

cd /root/BSProxy
cargo build --release

mkdir -p /opt/bsproxy
cp ./target/release/bsproxy /opt/bsproxy/proxy
chmod +x /opt/bsproxy/proxy

if [ -f /opt/bsproxy/menu ]; then
    cp /opt/bsproxy/menu /usr/local/bin/bsproxy
else
    cp /opt/bsproxy/proxy /usr/local/bin/bsproxy
fi
chmod +x /usr/local/bin/bsproxy

rm -rf /root/BSProxy

echo ""
echo "✅ Instalação concluída!"
echo "🚀 Digite 'bsproxy' para acessar o menu."
echo "   Ou 'bsproxy -p 80' para abrir porta 80."
