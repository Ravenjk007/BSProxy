cat > menu.sh << 'EOF'
#!/bin/bash

BSPROXY="/usr/local/bin/bsproxy"
PID_FILE="/tmp/bsproxy_"
LOG_FILE="/tmp/bsproxy_"

show_menu() {
    clear
    echo "====================================="
    echo "          QBSManager                 "
    echo "====================================="
    echo "          BSPROXY                    "
    echo ""
    
    # Verificar portas abertas
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
    
    if [ -n "$ACTIVE_PORTS" ]; then
        echo "Porta(s) aberta(s):$ACTIVE_PORTS"
    else
        echo "Porta(s): nenhuma"
    fi
    echo ""
    echo " 1 - Abrir Porta"
    echo " 2 - Fechar Porta"
    echo " 3 - Sair"
    echo ""
    echo -n "--> Selecione uma opção: "
}

open_port() {
    read -p "Digite o número da porta: " PORT
    if [[ -z "$PORT" ]]; then
        echo "❌ Porta inválida!"
        sleep 2
        return
    fi
    
    # Verificar se já está rodando
    if [[ -f "${PID_FILE}${PORT}.pid" ]]; then
        echo "❌ Porta ${PORT} já está aberta!"
        sleep 2
        return
    fi
    
    echo "🔓 Abrindo porta ${PORT} com multiprotocolo..."
    echo "   Protocols: SOCKS5 + TLS/SECURITY + TCP Fallback"
    
    # Verificar se bsproxy existe
    if [ ! -f "$BSPROXY" ]; then
        echo "❌ bsproxy não encontrado! Execute ./install.sh primeiro"
        sleep 3
        return
    fi
    
    # Iniciar o bsproxy com a porta
    nohup ${BSPROXY} -p ${PORT} > "${LOG_FILE}${PORT}.log" 2>&1 &
    echo $! > "${PID_FILE}${PORT}.pid"
    
    sleep 2
    if ps -p $(cat "${PID_FILE}${PORT}.pid") > /dev/null 2>&1; then
        echo "✅ Porta ${PORT} aberta com sucesso!"
        echo "📋 Log: ${LOG_FILE}${PORT}.log"
        echo ""
        echo "🧪 Teste SOCKS5: curl --socks5 localhost:${PORT} http://example.com"
        echo "🧪 Teste TLS: openssl s_client -connect localhost:${PORT}"
    else
        echo "❌ Falha ao abrir porta ${PORT}!"
        rm -f "${PID_FILE}${PORT}.pid"
    fi
    sleep 3
}

close_port() {
    read -p "Digite o número da porta: " PORT
    if [[ -z "$PORT" ]]; then
        echo "❌ Porta inválida!"
        sleep 2
        return
    fi
    
    if [[ -f "${PID_FILE}${PORT}.pid" ]]; then
        PID=$(cat "${PID_FILE}${PORT}.pid")
        kill -9 ${PID} 2>/dev/null
        rm -f "${PID_FILE}${PORT}.pid"
        echo "✅ Porta ${PORT} fechada com sucesso!"
    else
        echo "❌ Porta ${PORT} não está aberta!"
    fi
    sleep 2
}

while true; do
    show_menu
    read OPTION
    
    case $OPTION in
        1) open_port ;;
        2) close_port ;;
        3) 
            echo "👋 Saindo..."
            exit 0
            ;;
        *) 
            echo "❌ Opção inválida!"
            sleep 2
            ;;
    esac
done
EOF

chmod +x menu.sh
