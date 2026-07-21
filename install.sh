#!/bin/bash
# BSProxy Multi-Protocol Installer v2.0
# Suporte: SSL+SSH, SSL+WebSocket, XHTTP, Multi-Status

REPO_URL="https://github.com/Ravenjk007/BSProxy.git"
REPO_BRANCH="main"
CMD_NAME="bsproxy"
TOTAL_STEPS=12
CURRENT_STEP=0

# Cores
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

show_progress() {
    PERCENT=$((CURRENT_STEP * 100 / TOTAL_STEPS))
    echo -e "${CYAN}Progresso: [${PERCENT}%] - $1${NC}"
}

error_exit() {
    echo -e "\n${RED}❌ Erro: $1${NC}"
    exit 1
}

increment_step() {
    CURRENT_STEP=$((CURRENT_STEP + 1))
}

check_command() {
    if ! command -v "$1" &> /dev/null; then
        echo -e "${YELLOW}⚠️ $1 não encontrado, instalando...${NC}"
        apt install -y "$1" > /dev/null 2>&1 || error_exit "Falha ao instalar $1"
    fi
}

if [ "$EUID" -ne 0 ]; then
    error_exit "EXECUTE COMO ROOT (use: sudo bash install.sh)"
else
    clear

    # Banner BSPROXY
    echo -e "${BLUE}   ██████╗ ███████╗██████╗ ██████╗  ██████╗ ██╗  ██╗██╗   ██╗"
    echo -e "${CYAN}   ██╔══██╗██╔════╝██╔══██╗██╔══██╗██╔═══██╗╚██╗██╔╝╚██╗ ██╔╝"
    echo -e "${BLUE}   ██████╔╝███████╗██████╔╝██████╔╝██║   ██║ ╚███╔╝  ╚████╔╝ "
    echo -e "${CYAN}   ██╔══██╗╚════██║██╔═══╝ ██╔══██╗██║   ██║ ██╔██╗   ╚██╔╝  "
    echo -e "${BLUE}   ██████╔╝███████║██║     ██║  ██║╚██████╔╝██╔╝ ██╗   ██║   "
    echo -e "${CYAN}   ╚═════╝ ╚══════╝╚═╝     ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝   ╚═╝   "
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}           BSProxy Multi-Protocol Installer v2.0${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""

    # ========================================
    # STEP 1: Atualizar repositórios
    # ========================================
    show_progress "Atualizando repositórios..."
    export DEBIAN_FRONTEND=noninteractive
    apt update -y > /dev/null 2>&1 || error_exit "Falha ao atualizar os repositórios"
    increment_step

    # ========================================
    # STEP 2: Verificar sistema
    # ========================================
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
                24.*|22.*|20.*|18.*) echo -e "${GREEN}✅ Sistema Ubuntu $VERSION suportado${NC}" ;;
                *) error_exit "Versão do Ubuntu não suportada. Use 18, 20, 22 ou 24." ;;
            esac
            ;;
        Debian)
            case $VERSION in
                12*|11*|10*|9*) echo -e "${GREEN}✅ Sistema Debian $VERSION suportado${NC}" ;;
                *) error_exit "Versão do Debian não suportada. Use 9, 10, 11 ou 12." ;;
            esac
            ;;
        *) error_exit "Sistema não suportado. Use Ubuntu ou Debian." ;;
    esac
    increment_step

    # ========================================
    # STEP 3: Instalar dependências do sistema
    # ========================================
    show_progress "Instalando dependências do sistema..."
    apt upgrade -y > /dev/null 2>&1 || error_exit "Falha ao atualizar o sistema"
    
    # Dependências essenciais
    apt-get install -y \
        curl \
        build-essential \
        git \
        pkg-config \
        libssl-dev \
        ca-certificates \
        wget \
        gnupg \
        lsb-release \
        systemd \
        > /dev/null 2>&1 || error_exit "Falha ao instalar pacotes essenciais"
    
    increment_step

    # ========================================
    # STEP 4: Instalar Rust
    # ========================================
    show_progress "Instalando Rust..."
    if ! command -v rustc &> /dev/null; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y > /dev/null 2>&1 || error_exit "Falha ao instalar Rust"
        source "$HOME/.cargo/env"
        echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
    fi
    
    # Verifica versão do Rust
    RUST_VERSION=$(rustc --version 2>/dev/null | cut -d' ' -f2)
    echo -e "${GREEN}✅ Rust $RUST_VERSION instalado${NC}"
    increment_step

    # ========================================
    # STEP 5: Criar diretório
    # ========================================
    show_progress "Criando diretório /opt/bsproxy..."
    mkdir -p /opt/bsproxy > /dev/null 2>&1
    increment_step

    # ========================================
    # STEP 6: Clonar repositório
    # ========================================
    show_progress "Clonando repositório BSProxy..."
    if [ -d "/root/BSProxy" ]; then
        rm -rf /root/BSProxy
    fi
    git clone --branch "$REPO_BRANCH" "$REPO_URL" /root/BSProxy > /dev/null 2>&1 || error_exit "Falha ao clonar BSProxy"
    increment_step

    # ========================================
    # STEP 7: Compilar BSProxy
    # ========================================
    show_progress "Compilando BSProxy (isso pode levar alguns minutos)..."
    cd /root/BSProxy || error_exit "Diretório do BSProxy não encontrado"
    
    # Compila com otimizações
    cargo build --release --jobs "$(nproc)" 2>&1 | tee /tmp/bsproxy_build.log | grep -E "(Compiling|Finished|error)" || true
    
    if [ ! -f ./target/release/bsproxy ]; then
        echo -e "${RED}❌ Erro na compilação. Log:${NC}"
        tail -20 /tmp/bsproxy_build.log
        error_exit "Falha ao compilar BSProxy"
    fi
    
    echo -e "${GREEN}✅ Compilação concluída!${NC}"
    increment_step

    # ========================================
    # STEP 8: Instalar binários
    # ========================================
    show_progress "Instalando binários..."
    
    # Instala o proxy principal
    if [ -f ./target/release/bsproxy ]; then
        mv ./target/release/bsproxy /opt/bsproxy/proxy || error_exit "Falha ao mover binário"
        chmod +x /opt/bsproxy/proxy
    else
        error_exit "Binário 'bsproxy' não encontrado após compilação"
    fi
    
    # Copia os scripts
    if [ -f /root/BSProxy/menu.sh ]; then
        cp /root/BSProxy/menu.sh /opt/bsproxy/menu
        chmod +x /opt/bsproxy/menu
    fi
    
    # Cria link simbólico
    ln -sf /opt/bsproxy/proxy /usr/local/bin/bsproxy 2>/dev/null || true
    ln -sf /opt/bsproxy/menu /usr/local/bin/bsproxy-menu 2>/dev/null || true
    
    # Cria link de fallback
    if [ ! -f /usr/local/bin/bsproxy ]; then
        cp /opt/bsproxy/proxy /usr/local/bin/bsproxy
        chmod +x /usr/local/bin/bsproxy
    fi
    
    increment_step

    # ========================================
    # STEP 9: Criar arquivos de serviço systemd
    # ========================================
    show_progress "Configurando serviços systemd..."
    
    # Serviço padrão para porta 80
    cat > /etc/systemd/system/bsproxy-80.service <<EOF
[Unit]
Description=BSProxy Multi-Protocol (Porta 80)
After=network.target
Wants=network.target

[Service]
Type=simple
User=root
Group=root
WorkingDirectory=/opt/bsproxy
ExecStart=/opt/bsproxy/proxy --port 80 --status "BSPROXY-MULTI"
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
EOF

    # Serviço para porta 443 (SSL)
    cat > /etc/systemd/system/bsproxy-443.service <<EOF
[Unit]
Description=BSProxy SSL Multi-Protocol (Porta 443)
After=network.target
Wants=network.target

[Service]
Type=simple
User=root
Group=root
WorkingDirectory=/opt/bsproxy
ExecStart=/opt/bsproxy/proxy --port 443 --status "SSL-PROXY" --ssl
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
EOF

    # Serviço para porta 8080 (XHTTP)
    cat > /etc/systemd/system/bsproxy-8080.service <<EOF
[Unit]
Description=BSProxy XHTTP Multi-Status (Porta 8080)
After=network.target
Wants=network.target

[Service]
Type=simple
User=root
Group=root
WorkingDirectory=/opt/bsproxy
ExecStart=/opt/bsproxy/proxy --port 8080 --status "XHTTP-PROXY" --xhttp
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
EOF

    # Serviço para porta 8443 (SSL + XHTTP)
    cat > /etc/systemd/system/bsproxy-8443.service <<EOF
[Unit]
Description=BSProxy SSL + XHTTP (Porta 8443)
After=network.target
Wants=network.target

[Service]
Type=simple
User=root
Group=root
WorkingDirectory=/opt/bsproxy
ExecStart=/opt/bsproxy/proxy --port 8443 --status "SSL-XHTTP" --ssl --xhttp
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
EOF

    systemctl daemon-reload
    increment_step

    # ========================================
    # STEP 10: Configurar firewall
    # ========================================
    show_progress "Configurando firewall..."
    
    # UFW
    if command -v ufw &> /dev/null; then
        ufw allow 80/tcp > /dev/null 2>&1 || true
        ufw allow 443/tcp > /dev/null 2>&1 || true
        ufw allow 8080/tcp > /dev/null 2>&1 || true
        ufw allow 8443/tcp > /dev/null 2>&1 || true
        echo -e "${GREEN}✅ UFW configurado${NC}"
    fi
    
    # iptables
    if command -v iptables &> /dev/null; then
        iptables -A INPUT -p tcp --dport 80 -j ACCEPT 2>/dev/null || true
        iptables -A INPUT -p tcp --dport 443 -j ACCEPT 2>/dev/null || true
        iptables -A INPUT -p tcp --dport 8080 -j ACCEPT 2>/dev/null || true
        iptables -A INPUT -p tcp --dport 8443 -j ACCEPT 2>/dev/null || true
    fi
    
    increment_step

    # ========================================
    # STEP 11: Limpeza
    # ========================================
    show_progress "Limpando diretórios temporários..."
    cd /root/
    rm -rf /root/BSProxy/
    rm -f /tmp/bsproxy_build.log
    increment_step

    # ========================================
    # STEP 12: Finalizar
    # ========================================
    show_progress "Finalizando instalação..."
    
    # Tenta iniciar os serviços
    systemctl start bsproxy-80.service 2>/dev/null || true
    systemctl enable bsproxy-80.service 2>/dev/null || true
    
    # Verifica se os serviços estão rodando
    if systemctl is-active --quiet bsproxy-80.service; then
        echo -e "${GREEN}✅ Serviço bsproxy-80 ativo${NC}"
    else
        echo -e "${YELLOW}⚠️ Serviço bsproxy-80 não iniciou automaticamente${NC}"
    fi
    
    increment_step

    # ========================================
    # FINAL
    # ========================================
    echo ""
    echo -e "${GREEN}╔══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║      ✅ INSTALAÇÃO CONCLUÍDA COM SUCESSO!               ║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${CYAN}🚀 COMANDOS DISPONÍVEIS:${NC}"
    echo -e "  ${GREEN}bsproxy${NC}           - Iniciar proxy com portas padrão"
    echo -e "  ${GREEN}bsproxy-menu${NC}      - Menu interativo de gerenciamento"
    echo -e "  ${GREEN}bsproxy --port 80${NC}  - Iniciar na porta 80"
    echo -e "  ${GREEN}bsproxy --port 443 --ssl${NC} - Iniciar SSL+SSH na porta 443"
    echo ""
    echo -e "${CYAN}📡 PROTOCOLOS SUPORTADOS:${NC}"
    echo -e "  ✅ ${GREEN}SSH${NC} (Porta 22)"
    echo -e "  ✅ ${GREEN}SSL + SSH${NC} (Porta 443)"
    echo -e "  ✅ ${GREEN}SSL + WebSocket${NC} (Porta 443)"
    echo -e "  ✅ ${GREEN}XHTTP${NC} (Porta 8080)"
    echo -e "  ✅ ${GREEN}SSL + XHTTP${NC} (Porta 8443)"
    echo -e "  ✅ ${GREEN}Multi-Status (207)${NC}"
    echo -e "  ✅ ${GREEN}Multi-Protocolo${NC} (Detecção automática)"
    echo ""
    echo -e "${CYAN}🔧 SERVIÇOS SYSTEMD:${NC}"
    echo -e "  📌 bsproxy-80.service  - Porta 80"
    echo -e "  📌 bsproxy-443.service - Porta 443 (SSL)"
    echo -e "  📌 bsproxy-8080.service - Porta 8080 (XHTTP)"
    echo -e "  📌 bsproxy-8443.service - Porta 8443 (SSL+XHTTP)"
    echo ""
    echo -e "${YELLOW}💡 Para gerenciar serviços:${NC}"
    echo -e "  systemctl start/stop/restart bsproxy-{porta}.service"
    echo -e "  journalctl -u bsproxy-80.service -f  # Ver logs"
    echo ""
    echo -e "${GREEN}🎯 Digite 'bsproxy-menu' para abrir o gerenciador!${NC}"
    echo ""
fi
