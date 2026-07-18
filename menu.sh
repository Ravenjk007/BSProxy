#!/bin/bash

# ============================================
# BSProxy Menu - Free
# ============================================

BSPROXY="/opt/bsproxy/proxy"
PID_FILE="/tmp/bsproxy_"
LOG_FILE="/tmp/bsproxy_"

# Cores
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Função para mostrar portas abertas
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

# Função para verificar se porta está em uso
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

# Função para verificar certificado SSL
check_ssl_cert() {
    if [ -f "/opt/bsproxy/cert.pem" ] && [ -f "/opt/bsproxy/cert.key" ]; then
        return 0
    else
        return 1
    fi
}

# Função para abrir porta
open_port() {
    echo ""
    read -p "📌 Digite o número da porta: " PORT
    
    if [[ -z "$PORT" ]]; then
        echo -e "${RED}❌ Porta inválida!${NC}"
        sleep 2
        return
    fi
    
    # Verificação especial para porta 443
    if [[ "$PORT" == "443" ]]; then
        echo -e "${YELLOW}⚠️ Porta 443 (HTTPS/TLS) requer certificado SSL!${NC}"
        if ! check_ssl_cert; then
            echo -e "${RED}❌ Certificado SSL não encontrado!${NC}"
            echo -e "   Gerando certificado self-signed para teste..."
            openssl req -x509 -newkey rsa:4096 -keyout /opt/bsproxy/cert.key -out /opt/bsproxy/cert.pem -days 365 -nodes -subj "/CN=localhost" 2>/dev/null
            chmod 644 /opt/bsproxy/cert.pem
            chmod 600 /opt/bsproxy/cert.key
            echo -e "${GREEN}✅ Certificado self-signed gerado!${NC}"
            echo -e "${YELLOW}⚠️ Aviso: Clientes verão erro de certificado inválido${NC}"
            echo -e "   Para um certificado real: certbot certonly --standalone -d seu-dominio.com"
            sleep 3
        else
            echo -e "${GREEN}✅ Certificado SSL encontrado!${NC}"
        fi
    fi
    
    if is_port_in_use $PORT; then
        echo -e "${RED}❌ Porta ${PORT} já está em uso!${NC}"
        sleep 2
        return
    fi
    
    if [ ! -f "$BSPROXY" ]; then
        echo -e "${RED}❌ BSProxy não encontrado em $BSPROXY${NC}"
        sleep 3
        return
    fi
    
    echo -e "${YELLOW}🔓 Abrindo porta ${PORT}...${NC}"
    echo -e "${CYAN}📡 Protocolos: SOCKS5 | TLS | WebSocket | SECURITY | TCP${NC}"
    
    nohup ${BSPROXY} -p ${PORT} > "/tmp/bsproxy_${PORT}.log" 2>&1 &
    echo $! > "${PID_FILE}${PORT}.pid"
    sleep 2
    
    if is_port_in_use $PORT; then
        echo -e "${GREEN}✅ Porta ${PORT} aberta com sucesso!${NC}"
        echo -e "${GREEN}📋 Log: /tmp/bsproxy_${PORT}.log${NC}"
        echo ""
        echo -e "${CYAN}🧪 Testes:${NC}"
        echo -e "   ${YELLOW}SOCKS5:${NC} curl --socks5 localhost:${PORT} http://example.com"
        echo -e "   ${YELLOW}WebSocket:${NC} wscat -c ws://localhost:${PORT}"
        echo -e "   ${YELLOW}TLS:${NC} openssl s_client -connect localhost:${PORT}"
        echo -e "   ${YELLOW}SECURITY:${NC} echo 'SECURITY test' | nc localhost ${PORT}"
        echo -e "   ${YELLOW}TCP:${NC} telnet localhost ${PORT}"
        
        if [[ "$PORT" == "443" ]]; then
            echo ""
            echo -e "${YELLOW}🔒 Teste HTTPS:${NC}"
            echo -e "   curl -k https://localhost:${PORT}"
            echo -e "   wscat -c wss://localhost:${PORT}"
        fi
    else
        echo -e "${RED}❌ Falha ao abrir porta ${PORT}!${NC}"
        rm -f "${PID_FILE}${PORT}.pid"
    fi
    sleep 3
}

# Função para fechar porta
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
        echo -e "${GREEN}✅ Porta ${PORT} fechada com sucesso!${NC}"
    else
        echo -e "${RED}❌ Porta ${PORT} não está aberta!${NC}"
    fi
    sleep 2
}

# Função para reiniciar porta
restart_port() {
    echo ""
    read -p "📌 Digite o número da porta para reiniciar: " PORT
    
    if [[ -z "$PORT" ]]; then
        echo -e "${RED}❌ Porta inválida!${NC}"
        sleep 2
        return
    fi
    
    echo -e "${YELLOW}🔄 Reiniciando porta ${PORT}...${NC}"
    
    if is_port_in_use $PORT; then
        PID=$(cat "${PID_FILE}${PORT}.pid")
        kill -9 $PID 2>/dev/null
        rm -f "${PID_FILE}${PORT}.pid"
        echo -e "${YELLOW}⏹️ Porta ${PORT} fechada...${NC}"
        sleep 1
    fi
    
    echo -e "${YELLOW}🔓 Abrindo porta ${PORT}...${NC}"
    nohup ${BSPROXY} -p ${PORT} > "/tmp/bsproxy_${PORT}.log" 2>&1 &
    echo $! > "${PID_FILE}${PORT}.pid"
    sleep 2
    
    if is_port_in_use $PORT; then
        echo -e "${GREEN}✅ Porta ${PORT} reiniciada com sucesso!${NC}"
    else
        echo -e "${RED}❌ Falha ao reiniciar porta ${PORT}!${NC}"
        rm -f "${PID_FILE}${PORT}.pid"
    fi
    sleep 2
}

# Função para ver log da porta
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
        echo ""
        echo -e "${YELLOW}Pressione ENTER para voltar...${NC}"
        read
    else
        echo -e "${RED}❌ Log da porta ${PORT} não encontrado!${NC}"
        sleep 2
    fi
}

# Função para gerenciar certificado SSL
manage_cert() {
    echo ""
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${YELLOW}  GERENCIAR CERTIFICADO SSL${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo -e " ${GREEN}[1]${NC} - Ver certificado atual"
    echo -e " ${GREEN}[2]${NC} - Gerar certificado self-signed"
    echo -e " ${GREEN}[3]${NC} - Instalar certificado real (Let's Encrypt)"
    echo -e " ${GREEN}[4]${NC} - Voltar"
    echo ""
    echo -n "🔍 Escolha uma opção: "
    read CERT_OPTION
    
    case $CERT_OPTION in
        1)
            echo ""
            if check_ssl_cert; then
                echo -e "${GREEN}✅ Certificado encontrado:${NC}"
                openssl x509 -in /opt/bsproxy/cert.pem -text -noout | grep -E "Subject:|Issuer:|Not Before|Not After"
            else
                echo -e "${RED}❌ Nenhum certificado encontrado!${NC}"
            fi
            echo ""
            read -p "Pressione ENTER para voltar..."
            ;;
        2)
            echo ""
            echo -e "${YELLOW}📦 Gerando certificado self-signed...${NC}"
            openssl req -x509 -newkey rsa:4096 -keyout /opt/bsproxy/cert.key -out /opt/bsproxy/cert.pem -days 365 -nodes -subj "/CN=localhost" 2>/dev/null
            chmod 644 /opt/bsproxy/cert.pem
            chmod 600 /opt/bsproxy/cert.key
            echo -e "${GREEN}✅ Certificado self-signed gerado em /opt/bsproxy/${NC}"
            sleep 2
            ;;
        3)
            echo ""
            read -p "Digite seu domínio (ex: exemplo.com): " DOMAIN
            if [[ -z "$DOMAIN" ]]; then
                echo -e "${RED}❌ Domínio inválido!${NC}"
                sleep 2
                return
            fi
            echo -e "${YELLOW}🔒 Obtendo certificado real para $DOMAIN...${NC}"
            certbot certonly --standalone -d "$DOMAIN" -d "www.$DOMAIN" --non-interactive --agree-tos --email admin@"$DOMAIN"
            if [ -f "/etc/letsencrypt/live/$DOMAIN/fullchain.pem" ]; then
                cp "/etc/letsencrypt/live/$DOMAIN/fullchain.pem" /opt/bsproxy/cert.pem
                cp "/etc/letsencrypt/live/$DOMAIN/privkey.pem" /opt/bsproxy/cert.key
                chmod 644 /opt/bsproxy/cert.pem
                chmod 600 /opt/bsproxy/cert.key
                echo -e "${GREEN}✅ Certificado real instalado com sucesso!${NC}"
            else
                echo -e "${RED}❌ Falha ao obter certificado!${NC}"
            fi
            sleep 2
            ;;
        *) return ;;
    esac
}

# Função principal de menu
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
    
    # Verificar certificado
    if check_ssl_cert; then
        echo -e "${GREEN}✅ Certificado SSL:${NC} Instalado"
    else
        echo -e "${RED}❌ Certificado SSL:${NC} Não instalado (porta 443 não funcionará)"
    fi
    echo ""
    
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}[01]${NC} - ${YELLOW}ABRIR PORTA${NC}"
    echo -e "${GREEN}[02]${NC} - ${YELLOW}FECHAR PORTA${NC}"
    echo -e "${GREEN}[03]${NC} - ${YELLOW}REINICIAR PORTA${NC}"
    echo -e "${GREEN}[04]${NC} - ${YELLOW}VER LOG DA PORTA${NC}"
    echo -e "${GREEN}[05]${NC} - ${YELLOW}GERENCIAR CERTIFICADO SSL${NC}"
    echo -e "${GREEN}[80]${NC} - ${RED}SAIR${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo -e "${CYAN}📡 Protocolos: SOCKS5 | TLS | WebSocket | SECURITY | TCP${NC}"
    echo ""
    echo -n "🔍 Digite sua opção: "
}

# Loop principal
while true; do
    show_menu
    read OPTION
    
    case $OPTION in
        1|01) open_port ;;
        2|02) close_port ;;
        3|03) restart_port ;;
        4|04) view_log ;;
        5|05) manage_cert ;;
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
