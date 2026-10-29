#!/usr/bin/env bash
set -euo pipefail

PACKAGE_VERSION="1.0.2"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PREFIX="${PREFIX:-$HOME/.local}"
MODE="install"
CHECK_FB=1

usage() {
    cat <<'EOF'
Uso: ./install.sh [opções]

Instala a calculadora em $HOME/.local (ou no prefixo escolhido) e garante
acesso ao framebuffer pelo grupo "video".

Opções:
  --prefix DIR   instala em DIR/bin e DIR/share/doc/calculadora
  --skip-fb      não verifica nem altera permissões do framebuffer
  --check        valida o pacote e o ambiente; não instala nada
  --uninstall    remove o binário e a documentação instalados
  -h, --help     mostra esta ajuda

Exemplos:
  ./install.sh
  ./install.sh --prefix "$HOME/.local"
  ./install.sh --check
EOF
}

die() {
    printf 'erro: %s\n' "$*" >&2
    exit 1
}

info() {
    printf '\n== %s ==\n' "$*"
}

warn() {
    printf 'aviso: %s\n' "$*" >&2
}

has_video_group() {
    id -nG 2>/dev/null | tr ' ' '\n' | grep -qx video
}

fb_summary() {
    if [ ! -e /dev/fb0 ]; then
        printf '/dev/fb0: ausente (o backend fb não funcionará neste sistema)\n'
        return 1
    fi
    if [ -w /dev/fb0 ]; then
        printf '/dev/fb0: gravável\n'
        return 0
    fi
    printf '/dev/fb0: sem permissão de escrita\n'
    ls -l /dev/fb0 2>/dev/null || true
    if has_video_group; then
        printf 'grupo video: presente, mas a sessão atual ainda não tem acesso\n'
    else
        printf 'grupo video: ausente\n'
    fi
    return 1
}

ensure_fb_access() {
    [ "$CHECK_FB" -eq 1 ] || return 0

    if [ ! -e /dev/fb0 ]; then
        warn "/dev/fb0 não existe; o aplicativo exige um framebuffer fbdev."
        return 0
    fi

    if [ -w /dev/fb0 ]; then
        printf '/dev/fb0 já é gravável por %s.\n' "$(id -un)"
        return 0
    fi

    if has_video_group; then
        warn "o usuário já está no grupo video, mas /dev/fb0 não está gravável."
        ls -l /dev/fb0 2>/dev/null || true
        printf 'Saia e entre novamente; se persistir, verifique ACL/modo do device.\n'
        return 0
    fi

    printf 'O framebuffer é acessado pelo grupo "video" (normalmente root:video 660).\n'
    if [ ! -t 0 ]; then
        warn "sem terminal interativo; não foi possível pedir a senha do sudo."
        printf 'Execute manualmente: sudo usermod -aG video %s\n' "$(id -un)"
        return 0
    fi

    if ! command -v sudo >/dev/null 2>&1; then
        warn "sudo não encontrado; não foi possível adicionar o usuário ao grupo video."
        printf 'Execute como root: usermod -aG video %s\n' "$(id -un)"
        return 0
    fi

    local reply
    printf 'Adicionar %s ao grupo video agora? A senha de sudo pode ser pedida.\n' "$(id -un)"
    read -r -p 'Continuar? [S/n] ' reply || reply="n"
    case "$reply" in
        "" | s | S | y | Y) ;;
        *)
            warn "usuário não foi adicionado ao grupo video."
            printf 'Para liberar depois: sudo usermod -aG video %s\n' "$(id -un)"
            return 0
            ;;
    esac

    if sudo usermod -aG video "$(id -un)"; then
        printf 'Usuário adicionado ao grupo video.\n'
        printf 'O grupo só vale em uma nova sessão: saia e entre novamente,\n'
        printf 'reinicie, ou faça login em um VT novo antes de executar a calculadora.\n'
    else
        warn "não foi possível adicionar o usuário ao grupo video."
        printf 'Execute manualmente: sudo usermod -aG video %s\n' "$(id -un)"
    fi
}

check_payload() {
    local required
    [ -x "$SCRIPT_DIR/calculadora" ] || die "binário ausente ou sem permissão: $SCRIPT_DIR/calculadora"

    for required in README.md RELEASE_NOTES.md LICENSE LICENSE-ART \
        THIRD_PARTY_NOTICES.md licenses/OFL-Inter.txt licenses/OFL-JetBrainsMono.txt; do
        [ -r "$SCRIPT_DIR/$required" ] || die "arquivo ausente no pacote: $required"
    done

    local reported
    reported="$("$SCRIPT_DIR/calculadora" --version 2>/dev/null)" \
        || die "não foi possível executar $SCRIPT_DIR/calculadora --version"
    [ "$reported" = "calculadora $PACKAGE_VERSION" ] \
        || die "versão inesperada: '$reported' (esperado 'calculadora $PACKAGE_VERSION')"
    printf 'pacote: calculadora %s\n' "$PACKAGE_VERSION"
}

check_capture() {
    local binary="$1"
    local tmp
    tmp="$(mktemp "${TMPDIR:-/tmp}/calculadora-install-XXXXXX.png")"
    if "$binary" --capture-frame "$tmp" >/dev/null 2>&1; then
        printf 'captura de sanidade: OK\n'
        rm -f "$tmp"
        return 0
    fi
    rm -f "$tmp"
    warn "a captura de sanidade falhou; o binário pode não estar íntegro."
    return 1
}

do_check() {
    info "Verificando pacote"
    check_payload
    check_capture "$SCRIPT_DIR/calculadora" || true
    info "Verificando framebuffer"
    fb_summary || true
    if [ "$CHECK_FB" -eq 1 ] && [ ! -w /dev/fb0 ] && ! has_video_group; then
        printf 'O instalador pode adicionar o usuário ao grupo video com sudo.\n'
    fi
    printf '\nVerificação concluída; nada foi instalado.\n'
}

do_uninstall() {
    info "Removendo instalação"
    rm -f "$PREFIX/bin/calculadora"
    rm -rf "$PREFIX/share/doc/calculadora"
    printf 'Removido: %s\n' "$PREFIX/bin/calculadora"
    printf 'Removido: %s\n' "$PREFIX/share/doc/calculadora"
    printf 'O grupo video não foi alterado; remova-o manualmente se desejar:\n'
    printf '  sudo gpasswd -d %s video\n' "$(id -un)"
}

do_install() {
    info "Validando pacote"
    check_payload

    info "Instalando em $PREFIX"
    install -d -m 755 "$PREFIX/bin" "$PREFIX/share/doc/calculadora/licenses"
    rm -f "$PREFIX/bin/calculadora"
    install -m 755 "$SCRIPT_DIR/calculadora" "$PREFIX/bin/calculadora"
    install -m 644 "$SCRIPT_DIR/README.md" "$SCRIPT_DIR/RELEASE_NOTES.md" \
        "$SCRIPT_DIR/LICENSE" "$SCRIPT_DIR/LICENSE-ART" \
        "$SCRIPT_DIR/THIRD_PARTY_NOTICES.md" \
        "$PREFIX/share/doc/calculadora/"
    install -m 644 "$SCRIPT_DIR/licenses/OFL-Inter.txt" \
        "$SCRIPT_DIR/licenses/OFL-JetBrainsMono.txt" \
        "$PREFIX/share/doc/calculadora/licenses/"

    info "Verificando instalação"
    local reported
    reported="$("$PREFIX/bin/calculadora" --version)"
    [ "$reported" = "calculadora $PACKAGE_VERSION" ] \
        || die "versão inesperada após instalar: '$reported'"
    printf 'binário: %s (%s)\n' "$PREFIX/bin/calculadora" "$reported"
    check_capture "$PREFIX/bin/calculadora" || true

    ensure_fb_access

    case ":$PATH:" in
        *":$PREFIX/bin:"*) ;;
        *) warn "$PREFIX/bin não está no PATH; use o caminho completo." ;;
    esac

    info "Pronto"
    printf 'Para executar:\n'
    printf '  1. Se o grupo video mudou, saia e entre novamente (ou reinicie).\n'
    printf '  2. Troque para um VT livre com Ctrl+Alt+F3.\n'
    printf '  3. Faça login e execute: %s/bin/calculadora\n' "$PREFIX"
    printf '\nDesinstalar: %s/install.sh --uninstall\n' "$SCRIPT_DIR"
}

for argument in "$@"; do
    case "$argument" in
        --help | -h)
            usage
            exit 0
            ;;
    esac
done

while [ "$#" -gt 0 ]; do
    case "$1" in
        --prefix)
            [ "$#" -ge 2 ] || die "--prefix exige um diretório"
            PREFIX="$2"
            shift 2
            ;;
        --prefix=*)
            PREFIX="${1#*=}"
            shift
            ;;
        --skip-fb)
            CHECK_FB=0
            shift
            ;;
        --check)
            MODE="check"
            shift
            ;;
        --uninstall)
            MODE="uninstall"
            shift
            ;;
        --help | -h)
            usage
            exit 0
            ;;
        *)
            die "opção desconhecida: $1 (use --help)"
            ;;
    esac
done

[ -n "$PREFIX" ] || die "prefixo vazio"
case "$PREFIX" in
    /*) ;;
    *) PREFIX="$(pwd)/$PREFIX" ;;
esac

if [ "$MODE" != "uninstall" ] && [ "$(id -u)" -eq 0 ]; then
    die "não execute o instalador como root; use seu usuário normal e deixe o sudo cuidar do grupo video"
fi

case "$MODE" in
    check) do_check ;;
    uninstall) do_uninstall ;;
    install) do_install ;;
esac
