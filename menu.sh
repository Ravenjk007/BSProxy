#!/bin/bash
# BSProxy Manager - Menu interativo com suporte a múltiplos protocolos

PROXY_BIN="/opt/bsproxy/target/release/bsproxy"
SERVICE_PREFIX="bsproxy-"
DEFAULT_TARGET="127.0.0.1:22"
DEFAULT_STATUS="BSPROXY-MULTI"

# Cores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Verifica se o binário existe
check_binary() {
    if [ ! -f "$PROXY_BIN" ]; then
        echo -e "${RED}❌ Binário não encontrado em: $PROXY_BIN${NC}"
        echo -e "${YELLOW}📌 Compilando...${NC}"
        cd /opt/bsproxy && cargo build --release
        if [ $? -eq 0 ]; then
            echo -e "${GREEN}✅ Compilado com sucesso!${NC}"
        else
            echo -e "${RED}❌ Falha na compilação${NC}"
            exit 1
        fi
    fi
}

list_ports() {
    systemctl list-units --type=service --all --no-legend "${SERVICE_PREFIX}*.service" 2>/dev/null \
        | awk '{print $1}' \
        | sed -E "s/^${SERVICE_PREFIX}([0-9]+)\.service\$/\1/"
}

list_protocols() {
    systemctl list-units --type=service --all --no-legend "${SERVICE_PREFIX}*.service" 2>/dev/null \
        | awk '{print $1}' \
        | while read -r service; do
            port=$(echo "$service" | sed -E "s/^${SERVICE_PREFIX}([0-9]+)\.service\$/\1/")
            if [ -f "/etc/systemd/system/${service}" ]; then
                protocol=$(grep -o "PROTOCOL=[a-zA-Z]*" "/etc/systemd/system/${service}" 2>/dev/null | cut -d= -f2)
                echo "$port:$protocol"
            fi
        done
}

show_header() {
    clear
    echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║${GREEN}                    @BSManager v2.0                       ${CYAN}║${NC}"
    echo -e "${CYAN}╠════════════════════════════════════════════════════════════╣${NC}"
    echo -e "${CYAN}║${YELLOW}                 BSPROXY Multi-Protocol                   ${CYAN}║${NC}"
    echo -e "${CYAN}╠════════════════════════════════════════════════════════════╣${NC}"
}

show_menu() {
    local ports
    ports=$(list_ports | tr '\n' ' ')
    [ -z "$ports" ] && ports="${RED}nenhuma${NC}"
    
    local protocols
    protocols=$(list_protocols | tr '\n' ' ')
    [ -z "$protocols" ] && protocols="${RED}nenhum${NC}"
    
    echo -e "${CYAN}║${NC} 📡 Porta(s): ${GREEN}$ports${NC}"
    echo -e "${CYAN}║${NC} 🔌 Protocolos: ${YELLOW}$protocols${NC}"
    echo -e "${CYAN}╠════════════════════════════════════════════════════════════╣${NC}"
    echo -e "${CYAN}║${NC}  ${GREEN}1${NC} - Abrir Porta (Multi-Protocolo)               ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}  ${GREEN}2${NC} - Fechar Porta                                ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}  ${GREEN}3${NC} - Status do Servidor                          ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}  ${GREEN}4${NC} - SSL + SSH (Porta 443)                       ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}  ${GREEN}5${NC} - SSL + WebSocket (Porta 443)                 ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}  ${GREEN}6${NC} - XHTTP + Multi-Status (207)                  ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}  ${GREEN}7${NC} - Abrir todas as portas (80,443,8080,8443)    ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}  ${GREEN}8${NC} - Reiniciar todos os serviços                 ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}  ${RED}0${NC} - Sair                                         ${CYAN}║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
}

open_port() {
    echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║${GREEN}                 ABRIR PORTA                           ${CYAN}║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
    
    read -rp "📌 Digite a porta (1-65535): " port
    if ! [[ "$port" =~ ^[0-9]+$ ]] || [ "$port" -lt 1 ] || [ "$port" -gt 65535 ]; then
        echo -e "${RED}❌ Porta inválida.${NC}"
        sleep 2
        return
    fi

    local service="${SERVICE_PREFIX}${port}.service"
    if [ -f "/etc/systemd/system/${service}" ]; then
        echo -e "${YELLOW}⚠️ Essa porta já está aberta.${NC}"
        sleep 2
        return
    fi

    echo -e "\n${YELLOW}Selecione o protocolo:${NC}"
    echo " 1 - SSH (padrão)"
    echo " 2 - SSL + SSH"
    echo " 3 - SSL + WebSocket"
    echo " 4 - XHTTP + Multi-Status"
    echo " 5 - Multi-Protocolo (detecção automática)"
    read -rp "👉 Opção: " proto_opt
    
    local protocol="SSH"
    local extra_args=""
    case "$proto_opt" in
        2) 
            protocol="SSL+SSH"
            extra_args="--ssl"
            ;;
        3) 
            protocol="SSL+WebSocket"
            extra_args="--websocket --ssl"
            ;;
        4) 
            protocol="XHTTP"
            extra_args="--xhttp"
            ;;
        5) 
            protocol="Multi"
            extra_args="--multi"
            ;;
        *) 
            protocol="SSH"
            extra_args=""
            ;;
    esac
    
    read -rp "🎯 Alvo (padrão: $DEFAULT_TARGET): " target
    [ -z "$target" ] && target="$DEFAULT_TARGET"
    
    read -rp "📊 Status (padrão: $DEFAULT_STATUS): " status
    [ -z "$status" ] && status="$DEFAULT_STATUS"

    # Cria o arquivo de serviço
    cat > "/etc/systemd/system/${service}" <<EOF
[Unit]
Description=BSProxy Multi-Protocol na porta ${port}
After=network.target

[Service]
Type=simple
Environment="PROTOCOL=${protocol}"
ExecStart=${PROXY_BIN} --port ${port} --status "${status}" --target ${target} ${extra_args}
Restart=always
RestartSec=3
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

    echo -e "${BLUE}📦 Criando serviço para porta ${port}...${NC}"
    
    systemctl daemon-reload
    systemctl enable "${service}" > /dev/null 2>&1
    systemctl start "${service}"

    sleep 2
    if systemctl is-active --quiet "${service}"; then
        echo -e "${GREEN}✅ Porta ${port} aberta com sucesso!${NC}"
        echo -e "${GREEN}🔌 Protocolo: ${protocol}${NC}"
        echo -e "${GREEN}📡 Status: ${status}${NC}"
        echo -e "${GREEN}🎯 Target: ${target}${NC}"
    else
        echo -e "${RED}❌ Falha ao iniciar. Veja:${NC}"
        echo -e "${YELLOW}journalctl -u ${service} --no-pager -n 20${NC}"
        rm -f "/etc/systemd/system/${service}"
        systemctl daemon-reload
    fi
    sleep 2
}

close_port() {
    echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║${RED}                 FECHAR PORTA                          ${CYAN}║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
    
    local ports
    ports=$(list_ports)
    if [ -z "$ports" ]; then
        echo -e "${RED}❌ Nenhuma porta aberta no momento.${NC}"
        sleep 2
        return
    fi

    echo -e "${YELLOW}📡 Portas abertas:${NC} $(echo "$ports" | tr '\n' ' ')"
    read -rp "👉 Digite a porta que deseja fechar: " port
    local service="${SERVICE_PREFIX}${port}.service"

    if [ ! -f "/etc/systemd/system/${service}" ]; then
        echo -e "${RED}❌ Essa porta não está aberta pelo BSProxy.${NC}"
        sleep 2
        return
    fi

    echo -e "${BLUE}🛑 Parando serviço da porta ${port}...${NC}"
    systemctl stop "${service}"
    systemctl disable "${service}" > /dev/null 2>&1
    rm -f "/etc/systemd/system/${service}"
    systemctl daemon-reload

    echo -e "${GREEN}✅ Porta ${port} fechada com sucesso.${NC}"
    sleep 2
}

show_status() {
    echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║${GREEN}                 STATUS DO SERVIDOR                    ${CYAN}║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
    
    echo -e "\n${BLUE}📊 VISÃO GERAL${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    
    # Verifica status do binário
    if [ -f "$PROXY_BIN" ]; then
        echo -e "✅ ${GREEN}BSProxy Binário:${NC} $PROXY_BIN"
        echo -e "✅ ${GREEN}Versão:${NC} $(file "$PROXY_BIN" | cut -d, -f2 | xargs)"
    else
        echo -e "❌ ${RED}BSProxy Binário:${NC} Não encontrado"
    fi
    
    echo -e "\n${BLUE}🔌 PROTOCOLOS SUPORTADOS${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "✅ ${GREEN}SSL + SSH${NC} (Porta 443)"
    echo -e "✅ ${GREEN}SSL + WebSocket${NC} (Porta 443)"
    echo -e "✅ ${GREEN}XHTTP${NC} (Porta 8080)"
    echo -e "✅ ${GREEN}Multi-Status (207)${NC}"
    echo -e "✅ ${GREEN}Multi-Protocolo${NC} (Detecção automática)"
    
    echo -e "\n${BLUE}📡 PORTAS ATIVAS${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    local ports
    ports=$(list_ports)
    if [ -z "$ports" ]; then
        echo -e "${RED}❌ Nenhuma porta ativa${NC}"
    else
        for port in $ports; do
            local service="${SERVICE_PREFIX}${port}.service"
            if systemctl is-active --quiet "${service}"; then
                local protocol=$(grep -o "PROTOCOL=[a-zA-Z+]*" "/etc/systemd/system/${service}" 2>/dev/null | cut -d= -f2)
                [ -z "$protocol" ] && protocol="SSH"
                echo -e "  ${GREEN}●${NC} Porta ${GREEN}$port${NC} - ${YELLOW}${protocol}${NC} - ${GREEN}ATIVO${NC}"
            else
                echo -e "  ${RED}○${NC} Porta ${RED}$port${NC} - ${RED}INATIVO${NC}"
            fi
        done
    fi
    
    echo -e "\n${BLUE}📊 MÉTRICAS${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    local total_ports=$(list_ports | wc -l)
    local active_ports=0
    for port in $(list_ports); do
        if systemctl is-active --quiet "${SERVICE_PREFIX}${port}.service"; then
            ((active_ports++))
        fi
    done
    echo -e "  📌 Total de portas: ${YELLOW}$total_ports${NC}"
    echo -e "  ✅ Portas ativas: ${GREEN}$active_ports${NC}"
    echo -e "  💻 Memória usada: ${BLUE}$(free -h | awk '/Mem:/ {print $3}')${NC}"
    echo -e "  🔥 CPU: ${BLUE}$(top -bn1 | head -5 | awk '/Cpu/ {print $2}')%${NC}"
    
    echo -e "\n${YELLOW}Pressione ENTER para continuar...${NC}"
    read -r
}

open_all_ports() {
    echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║${GREEN}             ABRIR TODAS AS PORTAS                    ${CYAN}║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
    
    local ports="80 443 8080 8443"
    
    for port in $ports; do
        local service="${SERVICE_PREFIX}${port}.service"
        if [ -f "/etc/systemd/system/${service}" ]; then
            echo -e "${YELLOW}⚠️ Porta ${port} já está aberta, pulando...${NC}"
            continue
        fi
        
        echo -e "${BLUE}📦 Abrindo porta ${port}...${NC}"
        
        local protocol="Multi"
        local extra_args="--multi"
        
        # Define protocolo específico por porta
        if [ "$port" == "443" ]; then
            protocol="SSL+SSH/WebSocket"
            extra_args="--ssl --multi"
        elif [ "$port" == "8080" ]; then
            protocol="XHTTP"
            extra_args="--xhttp"
        elif [ "$port" == "8443" ]; then
            protocol="SSL+XHTTP"
            extra_args="--ssl --xhttp"
        fi
        
        cat > "/etc/systemd/system/${service}" <<EOF
[Unit]
Description=BSProxy na porta ${port} (${protocol})
After=network.target

[Service]
Type=simple
Environment="PROTOCOL=${protocol}"
ExecStart=${PROXY_BIN} --port ${port} --status "BSPROXY-MULTI" --target ${DEFAULT_TARGET} ${extra_args}
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target
EOF

        systemctl daemon-reload
        systemctl enable "${service}" > /dev/null 2>&1
        systemctl start "${service}"
        
        sleep 1
        if systemctl is-active --quiet "${service}"; then
            echo -e "${GREEN}✅ Porta ${port} aberta (${protocol})${NC}"
        else
            echo -e "${RED}❌ Falha ao abrir porta ${port}${NC}"
        fi
    done
    
    echo -e "\n${GREEN}✅ Todas as portas abertas!${NC}"
    sleep 2
}

restart_all_services() {
    echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║${YELLOW}           REINICIAR TODOS OS SERVIÇOS                ${CYAN}║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
    
    local ports
    ports=$(list_ports)
    if [ -z "$ports" ]; then
        echo -e "${RED}❌ Nenhuma porta ativa para reiniciar.${NC}"
        sleep 2
        return
    fi
    
    echo -e "${BLUE}🔄 Reiniciando todas as portas...${NC}"
    for port in $ports; do
        local service="${SERVICE_PREFIX}${port}.service"
        if [ -f "/etc/systemd/system/${service}" ]; then
            echo -e "  🔄 Reiniciando porta ${port}..."
            systemctl restart "${service}"
            sleep 1
        fi
    done
    
    echo -e "${GREEN}✅ Todos os serviços reiniciados!${NC}"
    sleep 2
}

# ========================================
# MAIN
# ========================================

# Verifica se está rodando como root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}❌ Este script precisa ser executado como root.${NC}"
    echo -e "${YELLOW}Use: sudo $0${NC}"
    exit 1
fi

# Verifica binário
check_binary

# Loop principal
while true; do
    show_header
    show_menu
    read -rp "👉 Selecione uma opção: " opt
    case "$opt" in
        1) open_port ;;
        2) close_port ;;
        3) show_status ;;
        4) 
            echo -e "${GREEN}🔒 Ativando SSL + SSH na porta 443...${NC}"
            read -rp "Porta (padrão: 443): " port
            [ -z "$port" ] && port="443"
            # Chama open_port com protocolo 2
            open_port
            ;;
        5) 
            echo -e "${GREEN}🔌 Ativando SSL + WebSocket na porta 443...${NC}"
            read -rp "Porta (padrão: 443): " port
            [ -z "$port" ] && port="443"
            open_port
            ;;
        6) 
            echo -e "${GREEN}🌐 Ativando XHTTP + Multi-Status na porta 8080...${NC}"
            read -rp "Porta (padrão: 8080): " port
            [ -z "$port" ] && port="8080"
            open_port
            ;;
        7) open_all_ports ;;
        8) restart_all_services ;;
        0) 
            echo -e "${GREEN}👋 Saindo...${NC}"
            exit 0
            ;;
        *) 
            echo -e "${RED}❌ Opção inválida.${NC}"
            sleep 1
            ;;
    esac
done
