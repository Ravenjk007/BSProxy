#!/bin/bash
# BSProxy Installer - Versão Free
REPO_URL="https://github.com/Ravenjk007/BSProxy.git"
REPO_BRANCH="main"
CMD_NAME="bsproxy"
TOTAL_STEPS=9
CURRENT_STEP=0

show_progress() {
    PERCENT=$((CURRENT_STEP * 100 / TOTAL_STEPS))
    echo "Progresso: [${PERCENT}%] - $1"
}

error_exit() {
    echo -e "\nErro: $1"
    exit 1
}

increment_step() {
    CURRENT_STEP=$((CURRENT_STEP + 1))
}

if [ "$EUID" -ne 0 ]; then
    error_exit "EXECUTE COMO ROOT"
else
    clear
    show_progress "Atualizando repositorios..."
    export DEBIAN_FRONTEND=noninteractive
    apt update -y > /dev/null 2>&1 || error_exit "Falha ao atualizar os repositorios"
    increment_step

    show_progress "Verificando o sistema..."
    if ! command -v lsb_release &> /dev/null; then
        apt install lsb-release -y > /dev/null 2>&1 || error_exit "Falha ao instalar lsb-release"
    fi
    increment_step

    OS_NAME=$(lsb_release -is)
    VERSION=$(lsb_release -rs)
    case $OS_NAME in
        Ubuntu)
            case $VERSION in
                24.*|22.*|20.*|18.*) show_progress "Sistema Ubuntu suportado, continuando..." ;;
                *) error_exit "Versão do Ubuntu não suportada. Use 18, 20, 22 ou 24." ;;
            esac
            ;;
        Debian)
            case $VERSION in
                12*|11*|10*|9*) show_progress "Sistema Debian suportado, continuando..." ;;
                *) error_exit "Versão do Debian não suportada. Use 9, 10, 11 ou 12." ;;
            esac
            ;;
        *) error_exit "Sistema não suportado. Use Ubuntu ou Debian." ;;
    esac
    increment_step

    show_progress "Atualizando o sistema..."
    apt upgrade -y > /dev/null 2>&1 || error_exit "Falha ao atualizar o sistema"
    apt-get install curl build-essential git pkg-config libssl-dev -y > /dev/null 2>&1 || error_exit "Falha ao instalar pacotes"
    increment_step

    show_progress "Criando diretorio /opt/bsproxy..."
    mkdir -p /opt/bsproxy > /dev/null 2>&1
    increment_step

    show_progress "Instalando Rust..."
    if ! command -v rustc &> /dev/null; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y > /dev/null 2>&1 || error_exit "Falha ao instalar Rust"
        source "$HOME/.cargo/env"
    fi
    # Garantir que o cargo está no PATH para a sessão atual
    [ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"
    increment_step

    show_progress "Compilando BSProxy, isso pode levar algum tempo..."
    if [ -d "/root/BSProxy_Build" ]; then
        rm -rf /root/BSProxy_Build
    fi
    git clone --branch "$REPO_BRANCH" "$REPO_URL" /root/BSProxy_Build > /dev/null 2>&1 || error_exit "Falha ao clonar BSProxy"

    cd /root/BSProxy_Build || error_exit "Diretório do BSProxy não encontrado"
    
    # Copiar o menu antes da limpeza
    if [ -f menu.sh ]; then
        cp menu.sh /opt/bsproxy/menu
        chmod +x /opt/bsproxy/menu
    fi

    cargo build --release --jobs "$(nproc)" > /dev/null 2>&1 || error_exit "Falha ao compilar BSProxy"

    if [ -f ./target/release/bsproxy ]; then
        mv ./target/release/bsproxy /opt/bsproxy/proxy || error_exit "Binário compilado não encontrado"
        chmod +x /opt/bsproxy/proxy
    else
        error_exit "Binário 'bsproxy' não encontrado após compilação"
    fi
    increment_step

    show_progress "Configurando permissões..."
    chmod +x /opt/bsproxy/proxy
    [ -f /opt/bsproxy/menu ] && chmod +x /opt/bsproxy/menu

    # Criar o link para o menu
    if [ -f /opt/bsproxy/menu ]; then
        cp /opt/bsproxy/menu /usr/local/bin/bsproxy
    else
        cp /opt/bsproxy/proxy /usr/local/bin/bsproxy
    fi
    chmod +x /usr/local/bin/bsproxy
    increment_step

    show_progress "Limpando diretórios temporários..."
    cd /root/
    rm -rf /root/BSProxy_Build/
    increment_step

    echo ""
    echo -e "\033[0;32m✅ Instalação concluída com sucesso!\033[0m"
    echo ""
    echo "🚀 Digite 'bsproxy' para acessar o menu."
    echo ""
    echo "📡 Novos Protocolos Suportados:"
    echo "   - SSL TUNNEL (SSH + SSL)"
    echo "   - SSL + WEBSOCKET"
    echo "   - SSL + OPENVPN"
    echo "   - SECURITY (SSL + SECURITY)"
    echo "   - XHTTP (Porta 443)"
    echo "   - MULTISTATUS (Ex: 101|200)"
    echo ""
fi
