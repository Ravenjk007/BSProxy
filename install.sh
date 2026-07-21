#!/bin/bash
# ════════════════════════════════════════════════════════════
#  BSProxy Multi-Protocol Installer v3.0
#  Suporte: SSL+SSH | SSL+WebSocket | XHTTP | Multi-Status
# ════════════════════════════════════════════════════════════

# ──────────────────────────────────────────────────────────────
#  CORES E ESTILOS
# ──────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m'
CHECK="${GREEN}✓${NC}"
CROSS="${RED}✗${NC}"
WARN="${YELLOW}⚠${NC}"
INFO="${CYAN}ℹ${NC}"

# ──────────────────────────────────────────────────────────────
#  CONFIGURAÇÕES
# ──────────────────────────────────────────────────────────────
REPO_URL="https://github.com/Ravenjk007/BSProxy.git"
REPO_BRANCH="main"
INSTALL_DIR="/opt/bsproxy"
BIN_NAME="bsproxy"
SERVICE_PREFIX="bsproxy"
TOTAL_STEPS=10
CURRENT_STEP=0
LOG_FILE="/tmp/bsproxy_install_$(date +%s).log"

# ──────────────────────────────────────────────────────────────
#  FUNÇÕES AUXILIARES
# ──────────────────────────────────────────────────────────────
log() {
    echo -e "$1" | tee -a "$LOG_FILE"
}

log_info() {
    log "${INFO} ${1}"
}

log_success() {
    log "${CHECK} ${GREEN}${1}${NC}"
}

log_warn() {
    log "${WARN} ${YELLOW}${1}${NC}"
}

log_error() {
    log "${CROSS} ${RED}${1}${NC}"
}

print_banner() {
    clear
    echo -e "${CYAN}"
    cat << "EOF"
    ╔══════════════════════════════════════════════════════════════╗
    ║                                                              ║
    ║     ██████╗ ███████╗██████╗ ██████╗  ██████╗ ██╗  ██╗██╗   ██╗
    ║     ██╔══██╗██╔════╝██╔══██╗██╔══██╗██╔═══██╗╚██╗██╔╝╚██╗ ██╔╝
    ║     ██████╔╝███████╗██████╔╝██████╔╝██║   ██║ ╚███╔╝  ╚████╔╝ 
    ║     ██╔══██╗╚════██║██╔═══╝ ██╔══██╗██║   ██║ ██╔██╗   ╚██╔╝  
    ║     ██████╔╝███████║██║     ██║  ██║╚██████╔╝██╔╝ ██╗   ██║   
    ║     ╚═════╝ ╚══════╝╚═╝     ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝   ╚═╝   
    ║                                                              ║
    ║              Multi-Protocol Proxy Installer v3.0             ║
    ║          SSL+SSH • SSL+WebSocket • XHTTP • Multi-Status     ║
    ╚══════════════════════════════════════════════════════════════╝
EOF
    echo -e "${NC}"
}

spinner() {
    local pid=$1
    local delay=0.1
    local spinstr='⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏'
    while ps -p "$pid" > /dev/null 2>&1; do
        local temp=${spinstr#?}
        printf " [%c]  " "$spinstr"
        local spinstr=$temp${spinstr%"$temp"}
        sleep $delay
        printf "\b\b\b\b\b\b"
    done
    printf "    \b\b\b\b"
}

show_progress() {
    local message="$1"
    local percent=$((CURRENT_STEP * 100 / TOTAL_STEPS))
    local filled=$((percent / 2))
    local empty=$((50 - filled))
    local bar=$(printf "%${filled}s" | tr ' ' '█')
    local space=$(printf "%${empty}s" | tr ' ' '░')
    
    echo -ne "\r${CYAN}[${bar}${space}]${NC} ${percent}% ${message}         "
}

increment_step() {
    CURRENT_STEP=$((CURRENT_STEP + 1))
}

error_exit() {
    echo ""
    log_error "$1"
    echo ""
    log_info "Log completo em: ${LOG_FILE}"
    echo -e "${YELLOW}Últimas linhas do log:${NC}"
    tail -10 "$LOG_FILE"
    exit 1
}

check_root() {
    if [ "$EUID" -ne 0 ]; then
        error_exit "Este script precisa ser executado como ROOT.\nUse: ${BOLD}sudo bash $0${NC}"
    fi
}

check_command() {
    if ! command -v "$1" &> /dev/null; then
        log_warn "$1 não encontrado. Instalando..."
        apt-get install -y "$1" >> "$LOG_FILE" 2>&1 || error_exit "Falha ao instalar $1"
        log_success "$1 instalado com sucesso"
    fi
}

check_port() {
    local port=$1
    if ss -tln | grep -q ":$port "; then
        log_warn "Porta $port já está em uso!"
        return 1
    fi
    return 0
}

# ──────────────────────────────────────────────────────────────
#  VERIFICAÇÕES INICIAIS
# ──────────────────────────────────────────────────────────────
pre_checks() {
    log_info "Verificando pré-requisitos..."
    
    # Verifica sistema operacional
    if ! command -v lsb_release &> /dev/null; then
        apt-get update -qq && apt-get install -y lsb-release >> "$LOG_FILE" 2>&1
    fi
    
    OS_NAME=$(lsb_release -is 2>/dev/null)
    VERSION=$(lsb_release -rs 2>/dev/null)
    
    case $OS_NAME in
        Ubuntu)
            case $VERSION in
                24.*|22.*|20.*|18.*) 
                    log_success "Ubuntu $VERSION - Compatível"
                    ;;
                *)
                    error_exit "Ubuntu $VERSION não é suportado (use 18, 20, 22 ou 24)"
                    ;;
            esac
            ;;
        Debian)
            case $VERSION in
                12*|11*|10*|9*)
                    log_success "Debian $VERSION - Compatível"
                    ;;
                *)
                    error_exit "Debian $VERSION não é suportado (use 9, 10, 11 ou 12)"
                    ;;
            esac
            ;;
        *)
            error_exit "Sistema $OS_NAME não é suportado (use Ubuntu ou Debian)"
            ;;
    esac
    
    # Verifica arquitetura
    ARCH=$(uname -m)
    case $ARCH in
        x86_64|aarch64)
            log_success "Arquitetura $ARCH - Compatível"
            ;;
        *)
            error_exit "Arquitetura $ARCH não é suportada (use x86_64 ou aarch64)"
            ;;
    esac
    
    # Verifica espaço em disco
    AVAILABLE_SPACE=$(df /opt | awk 'NR==2 {print $4}')
    if [ "$AVAILABLE_SPACE" -lt 1048576 ]; then
        error_exit "Espaço em disco insuficiente (mínimo 1GB)"
    fi
    
    # Verifica memória RAM
    TOTAL_RAM=$(free -m | awk '/Mem:/ {print $2}')
    if [ "$TOTAL_RAM" -lt 512 ]; then
        log_warn "Memória RAM baixa: ${TOTAL_RAM}MB (recomendado 1GB)"
    fi
    
    log_success "Todos os pré-requisitos verificados"
    sleep 1
}

# ──────────────────────────────────────────────────────────────
#  INSTALAÇÃO
# ──────────────────────────────────────────────────────────────
install_packages() {
    log_info "Instalando dependências do sistema..."
    
    export DEBIAN_FRONTEND=noninteractive
    apt-get update -qq >> "$LOG_FILE" 2>&1
    
    local packages=(
        curl
        wget
        git
        build-essential
        pkg-config
        libssl-dev
        ca-certificates
        gnupg
        lsb-release
        systemd
        net-tools
        dnsutils
        ufw
        iptables
        jq
        htop
        screen
        tmux
    )
    
    for pkg in "${packages[@]}"; do
        check_command "$pkg"
    done
    
    log_success "Todas as dependências instaladas"
    sleep 1
}

install_rust() {
    log_info "Instalando Rust..."
    
    if ! command -v rustc &> /dev/null; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y >> "$LOG_FILE" 2>&1
        source "$HOME/.cargo/env"
        echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
    fi
    
    rustup update stable >> "$LOG_FILE" 2>&1
    rustup default stable >> "$LOG_FILE" 2>&1
    
    RUST_VERSION=$(rustc --version | cut -d' ' -f2)
    CARGO_VERSION=$(cargo --version | cut -d' ' -f2)
    
    log_success "Rust $RUST_VERSION instalado"
    log_success "Cargo $CARGO_VERSION instalado"
    sleep 1
}

compile_bsproxy() {
    log_info "Compilando BSProxy (isso pode levar alguns minutos)..."
    
    cd /tmp || error_exit "Falha ao acessar /tmp"
    
    if [ -d "BSProxy" ]; then
        rm -rf BSProxy
    fi
    
    git clone --branch "$REPO_BRANCH" "$REPO_URL" BSProxy >> "$LOG_FILE" 2>&1 || error_exit "Falha ao clonar repositório"
    cd BSProxy || error_exit "Falha ao acessar diretório"
    
    # Compila com todas as cores disponíveis
    if cargo build --release --jobs "$(nproc)" >> "$LOG_FILE" 2>&1; then
        log_success "Compilação concluída com sucesso"
    else
        log_error "Falha na compilação"
        echo ""
        echo -e "${YELLOW}Últimas linhas do log:${NC}"
        tail -20 "$LOG_FILE"
        error_exit "Erro ao compilar BSProxy"
    fi
}

install_binary() {
    log_info "Instalando binários..."
    
    mkdir -p "$INSTALL_DIR"
    
    # Copia binário principal
    cp /tmp/BSProxy/target/release/bsproxy "$INSTALL_DIR/proxy"
    chmod +x "$INSTALL_DIR/proxy"
    
    # Cria link simbólico
    ln -sf "$INSTALL_DIR/proxy" /usr/local/bin/bsproxy
    
    # Copia menu
    if [ -f /tmp/BSProxy/menu.sh ]; then
        cp /tmp/BSProxy/menu.sh "$INSTALL_DIR/menu"
        chmod +x "$INSTALL_DIR/menu"
        ln -sf "$INSTALL_DIR/menu" /usr/local/bin/bsproxy-menu
    fi
    
    log_success "Binários instalados em $INSTALL_DIR"
    sleep 1
}

setup_services() {
    log_info "Configurando serviços systemd..."
    
    # Remove serviços antigos
    systemctl stop "${SERVICE_PREFIX}"-*.service 2>/dev/null || true
    
    local ports=(80 443 8080 8443)
    local descriptions=(
        "Multi-Protocol (HTTP)"
        "SSL+SSH/WebSocket (HTTPS)"
        "XHTTP + Multi-Status"
        "SSL + XHTTP"
    )
    local extra_args=("" "--ssl" "--xhttp" "--ssl --xhttp")
    local statuses=("BSPROXY" "SSL-PROXY" "XHTTP-PROXY" "SSL-XHTTP")
    
    for i in "${!ports[@]}"; do
        local port="${ports[$i]}"
        local desc="${descriptions[$i]}"
        local args="${extra_args[$i]}"
        local status="${statuses[$i]}"
        local service="${SERVICE_PREFIX}-${port}.service"
        
        cat > "/etc/systemd/system/${service}" <<EOF
[Unit]
Description=BSProxy ${desc} (Porta ${port})
Documentation=https://github.com/Ravenjk007/BSProxy
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=root
Group=root
WorkingDirectory=${INSTALL_DIR}
Environment="RUST_LOG=info"
Environment="RUST_BACKTRACE=1"
ExecStart=${INSTALL_DIR}/proxy --port ${port} --status "${status}" ${args}
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal
SyslogIdentifier=${service}
LimitNOFILE=65535
LimitNPROC=65535
TasksMax=infinity
MemoryMax=512M
CPUQuota=80%

[Install]
WantedBy=multi-user.target
EOF
        
        log_success "Serviço criado: $service (porta $port)"
    done
    
    systemctl daemon-reload
    
    # Inicia serviços
    for port in "${ports[@]}"; do
        systemctl enable "${SERVICE_PREFIX}-${port}.service" >> "$LOG_FILE" 2>&1
        systemctl start "${SERVICE_PREFIX}-${port}.service" >> "$LOG_FILE" 2>&1
        if systemctl is-active --quiet "${SERVICE_PREFIX}-${port}.service"; then
            log_success "Serviço porta $port iniciado com sucesso"
        else
            log_warn "Serviço porta $port não iniciou - verifique logs"
        fi
    done
    
    sleep 1
}

setup_firewall() {
    log_info "Configurando firewall..."
    
    # UFW
    if command -v ufw &> /dev/null; then
        ufw allow 80/tcp >> "$LOG_FILE" 2>&1
        ufw allow 443/tcp >> "$LOG_FILE" 2>&1
        ufw allow 8080/tcp >> "$LOG_FILE" 2>&1
        ufw allow 8443/tcp >> "$LOG_FILE" 2>&1
        log_success "UFW: Portas 80,443,8080,8443 liberadas"
    fi
    
    # iptables
    if command -v iptables &> /dev/null; then
        iptables -A INPUT -p tcp --dport 80 -j ACCEPT 2>/dev/null || true
        iptables -A INPUT -p tcp --dport 443 -j ACCEPT 2>/dev/null || true
        iptables -A INPUT -p tcp --dport 8080 -j ACCEPT 2>/dev/null || true
        iptables -A INPUT -p tcp --dport 8443 -j ACCEPT 2>/dev/null || true
        log_success "iptables: Regras adicionadas"
    fi
}

cleanup() {
    log_info "Limpando arquivos temporários..."
    rm -rf /tmp/BSProxy
    log_success "Limpeza concluída"
}

# ──────────────────────────────────────────────────────────────
#  FINALIZAÇÃO
# ──────────────────────────────────────────────────────────────
print_summary() {
    echo ""
    echo -e "${GREEN}╔══════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║              ✅  INSTALAÇÃO CONCLUÍDA COM SUCESSO               ║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${BOLD}${WHITE}📦  BSProxy Multi-Protocol v3.0${NC}"
    echo -e "${DIM}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    
    echo -e "${BOLD}${CYAN}🚀  COMANDOS DISPONÍVEIS:${NC}"
    echo -e "  ${GREEN}bsproxy${NC}           - Iniciar proxy com portas padrão"
    echo -e "  ${GREEN}bsproxy-menu${NC}      - Menu interativo de gerenciamento"
    echo -e "  ${GREEN}bsproxy --port 80${NC}  - Iniciar na porta 80"
    echo -e "  ${GREEN}bsproxy --port 443 --ssl${NC} - Iniciar SSL+SSH na porta 443"
    echo ""
    
    echo -e "${BOLD}${CYAN}📡  PROTOCOLOS SUPORTADOS:${NC}"
    echo -e "  ${CHECK} ${GREEN}SSH${NC} (Porta 22)"
    echo -e "  ${CHECK} ${GREEN}SSL + SSH${NC} (Porta 443)"
    echo -e "  ${CHECK} ${GREEN}SSL + WebSocket${NC} (Porta 443)"
    echo -e "  ${CHECK} ${GREEN}XHTTP${NC} (Porta 8080)"
    echo -e "  ${CHECK} ${GREEN}SSL + XHTTP${NC} (Porta 8443)"
    echo -e "  ${CHECK} ${GREEN}Multi-Status (207)${NC}"
    echo -e "  ${CHECK} ${GREEN}Multi-Protocolo${NC} (Detecção automática)"
    echo ""
    
    echo -e "${BOLD}${CYAN}🔧  SERVIÇOS SYSTEMD:${NC}"
    for port in 80 443 8080 8443; do
        local status=$(systemctl is-active "${SERVICE_PREFIX}-${port}.service" 2>/dev/null)
        if [ "$status" = "active" ]; then
            echo -e "  ${CHECK} ${GREEN}bsproxy-${port}.service${NC} - Porta ${port} ${GREEN}● ATIVO${NC}"
        else
            echo -e "  ${WARN} ${YELLOW}bsproxy-${port}.service${NC} - Porta ${port} ${RED}○ INATIVO${NC}"
        fi
    done
    echo ""
    
    echo -e "${BOLD}${YELLOW}💡  PARA GERENCIAR OS SERVIÇOS:${NC}"
    echo -e "  systemctl start/stop/restart ${SERVICE_PREFIX}-{porta}.service"
    echo -e "  journalctl -u ${SERVICE_PREFIX}-80.service -f  # Ver logs"
    echo -e "  systemctl status ${SERVICE_PREFIX}-*.service   # Ver status"
    echo ""
    
    echo -e "${BOLD}${GREEN}🎯  Digite 'bsproxy-menu' para abrir o gerenciador!${NC}"
    echo ""
    echo -e "${DIM}Log completo: ${LOG_FILE}${NC}"
}

# ──────────────────────────────────────────────────────────────
#  MAIN
# ──────────────────────────────────────────────────────────────
main() {
    print_banner
    check_root
    
    echo -e "${DIM}Log: ${LOG_FILE}${NC}"
    echo ""
    
    # Step 1: Pré-verificações
    CURRENT_STEP=1
    show_progress "Verificando sistema..."
    pre_checks
    increment_step
    
    # Step 2: Pacotes
    show_progress "Instalando dependências..."
    install_packages
    increment_step
    
    # Step 3: Rust
    show_progress "Instalando Rust..."
    install_rust
    increment_step
    
    # Step 4: Clone
    show_progress "Clonando repositório..."
    increment_step
    
    # Step 5: Compilação
    show_progress "Compilando BSProxy..."
    compile_bsproxy
    increment_step
    
    # Step 6: Binários
    show_progress "Instalando binários..."
    install_binary
    increment_step
    
    # Step 7: Serviços
    show_progress "Configurando serviços..."
    setup_services
    increment_step
    
    # Step 8: Firewall
    show_progress "Configurando firewall..."
    setup_firewall
    increment_step
    
    # Step 9: Limpeza
    show_progress "Limpando..."
    cleanup
    increment_step
    
    # Step 10: Finalização
    show_progress "Finalizando..."
    sleep 1
    increment_step
    
    echo ""
    print_summary
}

# ──────────────────────────────────────────────────────────────
#  EXECUTA
# ──────────────────────────────────────────────────────────────
main
