#!/bin/bash
# Instalação BS Proxy compatível Ubuntu e Debian todas as versões

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
    echo ""
    # Banner BSPROXY em arte ASCII
    echo -e "\033[0;34m   ██████╗ ███████╗██████╗ ██████╗  ██████╗ ██╗  ██╗██╗   ██╗"
    echo -e "\033[0;37m   ██╔══██╗██╔════╝██╔══██╗██╔══██╗██╔═══██╗╚██╗██╔╝╚██╗ ██╔╝"
    echo -e "\033[0;34m   ██████╔╝███████╗██████╔╝██████╔╝██║   ██║ ╚███╔╝  ╚████╔╝ "
    echo -e "\033[0;37m   ██╔══██╗╚════██║██╔═══╝ ██╔══██╗██║   ██║ ██╔██╗   ╚██╔╝  "
    echo -e "\033[0;34m   ██████╔╝███████║██║     ██║  ██║╚██████╔╝██╔╝ ██╗   ██║   "
    echo -e "\033[0;37m   ╚═════╝ ╚══════╝╚═╝     ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝   ╚═╝   "
    echo -e "\033[0;34m--------------------------------------------------------------\033[0m"
    echo -e "\033[31m              DEV:@BS  ED:@BS \033[0m              "              
    echo -e " "
    show_progress "ATUALIZANDO REPOSITÓRIO..."
    export DEBIAN_FRONTEND=noninteractive
    apt update -y > /dev/null 2>&1 || error_exit "Falha ao atualizar os repositorios"
    increment_step

    # ---->>>> Verificação do sistema
    show_progress "VERIFICANDO SISTEMA..."
    if ! command -v lsb_release &> /dev/null; then
        apt install lsb-release -y > /dev/null 2>&1 || error_exit "Falha ao instalar lsb-release"
    fi

    if [ ! -f /etc/os-release ]; then
        error_exit "Arquivo /etc/os-release não encontrado. Sistema não identificado."
    fi

    OS_NAME=$(lsb_release -is || grep ^ID= /etc/os-release | cut -d'=' -f2)
    VERSION=$(lsb_release -rs || grep ^VERSION_ID= /etc/os-release | cut -d'=' -f2 | tr -d '"')

    case $OS_NAME in
        Ubuntu|ubuntu|debian|Debian)
            show_progress "SISTEMA $OS_NAME DETECTADO. CONTINUANDO..."
            ;;
        *)
            error_exit "SISTEMA NÃO SUPORTADO. USE UBUNTU OU DEBIAN."
            ;;
    esac
    increment_step

    # ---->>>> Instalação de pacotes requisitos e atualização do sistema
    show_progress "ATUALIZANDO O SISTEMA, AGUARDE..."
    apt upgrade -y > /dev/null 2>&1 || error_exit "Falha ao atualizar o sistema"
    apt-get install curl build-essential git -y > /dev/null 2>&1 || error_exit "Falha ao instalar pacotes"
    increment_step

    # ---->>>> Criando o diretório do script
    show_progress "CRIANDO DIRETÓRIO..."
    mkdir -p /opt/bsproxy > /dev/null 2>&1
    increment_step

    # ---->>>> Instalar rust
    show_progress "INSTALANDO RUST..."
    if ! command -v rustc &> /dev/null; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y > /dev/null 2>&1 || error_exit "Falha ao instalar Rust"
        echo 'source "$HOME/.cargo/env"' >> ~/.bashrc
        echo 'source "$HOME/.cargo/env"' >> ~/.zshrc
        source "$HOME/.cargo/env"
    fi
    increment_step

    # ---->>>> Instalar o BSProxy
    show_progress "COMPILANDO BSPROXY, ISSO PODE LEVAR ALGUM TEMPO, AGUARDE..."

    if [ -d "/root/BSProxy" ]; then
        rm -rf /root/BSProxy
    fi

    git clone --branch "main" https://github.com/WorldSsh/BSProxy.git /root/BSProxy > /dev/null 2>&1 || error_exit "Falha ao clonar o repositório"
    mv /root/BSProxy/menu.sh /opt/bsproxy/menu
    cd /root/BSProxy
    cargo build --release --jobs $(nproc) > /dev/null 2>&1 || error_exit "Falha ao compilar o BSProxy"
    mv ./target/release/BSProxy /opt/bsproxy/proxy
    increment_step

    # ---->>>> Configuração de permissões
    show_progress "CONFIGURANDO PERMISSÕES..."
    chmod +x /opt/bsproxy/proxy
    chmod +x /opt/bsproxy/menu
    ln -sf /opt/bsproxy/menu /usr/local/bin/bsproxy
    increment_step

    # ---->>>> Limpeza
    show_progress "LIMPANDO DIRETÓRIOS TEMPORÁRIOS, AGUARDE..."
    cd /root/
    rm -rf /root/BSProxy/
    increment_step

    # ---->>>> Instalação finalizada :)
    clear
    echo -e " "
    echo -e "\033[0;34m--------------------------------------------------------------\033[0m"
    echo -e "\033[40;1;37m            INSTALAÇÃO FINALIZADA COM SUCESSO                 \E[0m"
    echo -e "\033[0;34m--------------------------------------------------------------\033[0m"
    echo -e " "
    echo -e "\033[1;31m \033[1;33mDIGITE O COMANDO PARA ACESSAR O MENU: \033[1;32mbsproxy\033[0m"
    echo -e " "
fi
