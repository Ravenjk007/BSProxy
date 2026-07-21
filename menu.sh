#!/bin/bash
# BSProxy Manager - menu interativo de portas (estilo RustyManager)

PROXY_BIN="/opt/bsproxy/proxy"
SERVICE_PREFIX="bsproxy-"
DEFAULT_TARGET="127.0.0.1:22"
DEFAULT_STATUS="200 OK"

list_ports() {
    systemctl list-units --type=service --all --no-legend "${SERVICE_PREFIX}*.service" 2>/dev/null \
        | awk '{print $1}' \
        | sed -E "s/^${SERVICE_PREFIX}([0-9]+)\.service\$/\1/"
}

show_menu() {
    clear
    local ports
    ports=$(list_ports | tr '\n' ' ')
    [ -z "$ports" ] && ports="nenhuma"

    echo "================= @BSManager ================="
    echo "|                 BSPROXY                      |"
    echo "------------------------------------------------"
    echo "| Porta(s) Ativa(s): $ports"
    echo "------------------------------------------------"
    echo "| 1 - Abrir Porta (SSH/WS/OVPN)"
    echo "| 2 - Abrir Porta com SSL (SSH+SSL/WS+SSL/OVPN+SSL)"
    echo "| 3 - Ativar XHTTP (Porta 443)"
    echo "| 4 - Fechar Porta"
    echo "| 0 - Sair"
    echo "------------------------------------------------"
}

open_port() {
    local proto_mode=$1
    read -rp "Digite a porta que deseja abrir: " port
    if ! [[ "$port" =~ ^[0-9]+$ ]] || [ "$port" -lt 1 ] || [ "$port" -gt 65535 ]; then
        echo "Porta inválida."
        sleep 2
        return
    fi

    echo "Selecione o protocolo:"
    echo "1) SSH"
    echo "2) WebSocket"
    echo "3) OpenVPN"
    echo "4) Security"
    read -rp "Opção: " proto_opt
    
    case $proto_opt in
        1) protocol="ssh" ;;
        2) protocol="websocket" ;;
        3) protocol="openvpn" ;;
        4) protocol="security" ;;
        *) protocol="ssh" ;;
    esac

    if [ "$proto_mode" == "ssl" ]; then
        protocol="${protocol}+ssl"
    fi

    read -rp "Digite o Status (ex: 200 OK ou 101|200 para multi): " status
    [ -z "$status" ] && status="$DEFAULT_STATUS"

    read -rp "Digite o Alvo (padrão: $DEFAULT_TARGET): " target
    [ -z "$target" ] && target="$DEFAULT_TARGET"

    local service="${SERVICE_PREFIX}${port}.service"
    cat > "/etc/systemd/system/${service}" <<EOF
[Unit]
Description=BSProxy na porta ${port} (${protocol})
After=network.target

[Service]
Type=simple
ExecStart=${PROXY_BIN} --port ${port} --protocol ${protocol} --status "${status}" --target ${target}
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target
EOF

    systemctl daemon-reload
    systemctl enable "${service}" > /dev/null 2>&1
    systemctl start "${service}"

    echo "Porta ${port} (${protocol}) aberta com sucesso."
    sleep 2
}

activate_xhttp() {
    local service="${SERVICE_PREFIX}xhttp.service"
    echo "Ativando XHTTP na porta 443..."
    
    cat > "/etc/systemd/system/${service}" <<EOF
[Unit]
Description=BSProxy XHTTP na porta 443
After=network.target

[Service]
Type=simple
ExecStart=${PROXY_BIN} --port 443 --protocol ssh+ssl --status "200 OK" --target 127.0.0.1:22
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target
EOF

    systemctl daemon-reload
    systemctl enable "${service}" > /dev/null 2>&1
    systemctl start "${service}"
    echo "XHTTP ativado na porta 443."
    sleep 2
}

close_port() {
    local ports
    ports=$(list_ports)
    echo "Portas abertas: $ports"
    read -rp "Digite a porta para fechar (ou 'xhttp'): " port
    
    local service
    if [ "$port" == "xhttp" ]; then
        service="${SERVICE_PREFIX}xhttp.service"
    else
        service="${SERVICE_PREFIX}${port}.service"
    fi

    systemctl stop "${service}"
    systemctl disable "${service}" > /dev/null 2>&1
    rm -f "/etc/systemd/system/${service}"
    systemctl daemon-reload
    echo "Porta/Serviço fechado."
    sleep 2
}

while true; do
    show_menu
    read -rp "--> Selecione uma opção: " opt
    case "$opt" in
        1) open_port "normal" ;;
        2) open_port "ssl" ;;
        3) activate_xhttp ;;
        4) close_port ;;
        0) exit 0 ;;
        *) echo "Opção inválida."; sleep 1 ;;
    esac
done
