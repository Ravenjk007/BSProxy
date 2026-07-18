cat > menu.sh << 'EOF'
#!/bin/bash

BSPROXY="/opt/bsproxy/proxy"
PID_FILE="/tmp/bsproxy_"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

show_ports() {
    ACTIVE_PORTS=""
    for pidfile in ${PID_FILE}*.pid; do
        if [ -f "$pidfile" ]; then
            PORT=$(basename "$pidfile" .pid | sed 's/bsproxy_//')
            if ps -p $(cat "$pidfile") > /dev/null 2>&1; then
                ACTIVE_PORTS="$ACTIVE_PORTS $PORT"
            else
                rm -f "$pidfile"
            fi
        fi
    done
    echo "$ACTIVE_PORTS"
}

is_port_in_use() {
    local PORT=$1
    if [[ -f "${PID_FILE}${PORT}.pid" ]]; then
        PID=$(cat "${PID_FILE}${PORT}.pid")
        if ps -p $PID > /dev/null 2>&1; then
            return 0
        else
            rm -f "${PID_FILE}${PORT}.pid"
            return 1
        fi
    fi
    return 1
}

open_port() {
    echo ""
    read -p "📌 Digite o número da porta: " PORT
    
    if [[ -z "$PORT" ]]; then
        echo -e "${RED}❌ Porta inválida!${NC}"
        sleep 2
        return
    fi
    
    if is_port_in_use $PORT; then
        echo -e "${RED}❌ Porta ${PORT} já está em uso!${NC}"
        sleep 2
        return
    fi
    
    if [ ! -f "$BSPROXY" ]; then
        echo -e "${RED}❌ BSProxy não encontrado!${NC}"
        sleep 3
        return
    fi
    
    echo -e "${YELLOW}🔓 Abrindo porta ${PORT}...${NC}"
    echo -e "${CYAN}📡 Protocols: SOCKS5 | TLS | WebSocket | SECURITY | TCP${NC}"
    
    nohup ${BSPROXY} -p ${PORT} > "/tmp/bsproxy_${PORT}.log" 2>&1 &
    echo $! > "${PID_FILE}${PORT}.pid"
    sleep 2
    
    if is_port_in_use $PORT; then
        echo -e "${GREEN}✅ Porta ${PORT} aberta com sucesso!${NC}"
        echo -e "${GREEN}📋 Log: /tmp/bsproxy_${PORT}.log${NC}"
    else
        echo -e "${RED}❌ Falha ao abrir porta ${PORT}!${NC}"
        rm -f "${PID_FILE}${PORT}.pid"
    fi
    sleep 2
}

close_port() {
    echo ""
    read -p "📌 Digite o número da porta: " PORT
    
    if [[ -z "$PORT" ]]; then
        echo -e "${RED}❌ Porta inválida!${NC}"
        sleep 2
        return
    fi
    
    if is_port_in_use $PORT; then
        PID=$(cat "${PID_FILE}${PORT}.pid")
        kill -9 $PID 2>/dev/null
        rm -f "${PID_FILE}${PORT}.pid"
        echo -e "${GREEN}✅ Porta ${PORT} fechada!${NC}"
    else
        echo -e "${RED}❌ Porta ${PORT} não está aberta!${NC}"
    fi
    sleep 2
}

view_log() {
    echo ""
    read -p "📌 Digite o número da porta para ver o log: " PORT
    
    if [[ -z "$PORT" ]]; then
        echo -e "${RED}❌ Porta inválida!${NC}"
        sleep 2
        return
    fi
    
    LOG_FILE="/tmp/bsproxy_${PORT}.log"
    
    if [ -f "$LOG_FILE" ]; then
        echo -e "${CYAN}📋 Log da porta ${PORT}:${NC}"
        echo "====================================="
        tail -30 "$LOG_FILE"
        echo "====================================="
        read -p "Pressione ENTER para voltar..."
    else
        echo -e "${RED}❌ Log não encontrado!${NC}"
        sleep 2
    fi
}

show_menu() {
    clear
    echo -e "${CYAN}=====================================${NC}"
    echo -e "${CYAN}          BSProxy Menu              ${NC}"
    echo -e "${CYAN}=====================================${NC}"
    echo ""
    
    PORTS=$(show_ports)
    if [ -n "$PORTS" ]; then
        echo -e "${GREEN}✅ Porta(s) ativa(s):${NC} ${YELLOW}$PORTS${NC}"
    else
        echo -e "${RED}❌ Nenhuma porta ativa${NC}"
    fi
    echo ""
    
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}[01]${NC} - ${YELLOW}ABRIR PORTA${NC}"
    echo -e "${GREEN}[02]${NC} - ${YELLOW}FECHAR PORTA${NC}"
    echo -e "${GREEN}[03]${NC} - ${YELLOW}VER LOG DA PORTA${NC}"
    echo -e "${GREEN}[80]${NC} - ${RED}SAIR${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo -e "${CYAN}📡 Protocols: SOCKS5 | TLS | WebSocket | SECURITY | TCP${NC}"
    echo ""
    echo -n "🔍 Digite sua opção: "
}

while true; do
    show_menu
    read OPTION
    
    case $OPTION in
        1|01) open_port ;;
        2|02) close_port ;;
        3|03) view_log ;;
        80) 
            echo -e "${GREEN}👋 Saindo...${NC}"
            exit 0
            ;;
        *) 
            echo -e "${RED}❌ Opção inválida!${NC}"
            sleep 2
            ;;
    esac
done
EOF

chmod +x menu.sh
