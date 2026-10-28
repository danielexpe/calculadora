#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

BIN="$ROOT/target/release/calculadora"
BUILD=1
RUN=1
APP_ARGS=()

usage() {
    cat <<'EOF'
Uso: ./user_run_now.sh [--check] [--no-build] [opções da calculadora]

  --check      valida ambiente, build e captura; não abre o framebuffer
  --no-build   usa o binário release já compilado
  -h, --help   mostra esta ajuda

Opções repassadas para a calculadora:
  --lang pt|en
  --color auto|truecolor|256|16|never
  --no-animations
  --no-audio
  --fb-device PATH

Fluxo recomendado:
  1. sudo usermod -aG video "$USER"
  2. Ctrl+Alt+F3 (ou outro VT livre) e login
  3. ./user_run_now.sh

Checklist dentro do app:
  - 200 + 10 % = 220
  - 0,1 + 0,2 = 0,3
  - 2 + 3 × 4 = 20
  - 1 ÷ 0 = Erro; depois digite 7 para recuperar
  - ↑/↓ navegam na fita; Enter recupera o resultado
  - F2 alterna idioma; M alterna mudo
  - Q, Esc ou Ctrl+C restauram o console
EOF
}

step() {
    printf '\n== %s ==\n' "$*"
}

fail() {
    printf 'erro: %s\n' "$*" >&2
    exit 1
}

for argument in "$@"; do
    case "$argument" in
        --check)
            RUN=0
            ;;
        --no-build)
            BUILD=0
            ;;
        -h | --help)
            usage
            exit 0
            ;;
        *)
            APP_ARGS+=("$argument")
            ;;
    esac
done

step "Ambiente"
printf 'diretório: %s\n' "$ROOT"
USER_NAME="$(id -un)"
printf 'usuário: %s\n' "$USER_NAME"
printf 'sessão: %s\n' "${XDG_SESSION_TYPE:-desconhecida}"
printf 'grupos: '
id -nG

if [ ! -e /dev/fb0 ]; then
    fail "/dev/fb0 não existe; o backend fb não pode ser usado neste sistema"
fi

ls -l /dev/fb0
printf 'driver: '
cat /sys/class/graphics/fb0/name 2>/dev/null || printf 'desconhecido\n'
printf 'resolução virtual: '
cat /sys/class/graphics/fb0/virtual_size 2>/dev/null || printf 'desconhecida\n'
printf 'bpp: '
cat /sys/class/graphics/fb0/bits_per_pixel 2>/dev/null || printf 'desconhecido\n'
if command -v fbset >/dev/null 2>&1; then
    printf '\n'
    fbset -i 2>/dev/null || printf 'fbset -i indisponível\n'
fi

if [ "$BUILD" -eq 1 ]; then
    step "Build release com áudio"
    cargo build --release
else
    step "Build"
    printf 'ignorado (--no-build)\n'
fi

[ -x "$BIN" ] || fail "binário não encontrado: $BIN"

if command -v strings >/dev/null 2>&1 && strings "$BIN" | grep 'libasound.so.2' >/dev/null 2>&1; then
    printf 'áudio: backend ALSA dinâmico embutido\n'
else
    printf 'áudio: backend ALSA não detectado no binário\n'
fi

if [ -e /dev/snd ] && ldconfig -p 2>/dev/null | grep 'libasound.so.2' >/dev/null 2>&1; then
    printf 'ALSA: biblioteca e /dev/snd presentes; som deve estar disponível\n'
else
    printf 'ALSA: indisponível; o app deve mostrar "Som: indisponível" e seguir em silêncio\n'
fi

step "Sanidade headless"
TMP_PNG="$(mktemp "${TMPDIR:-/tmp}/calculadora-preflight-XXXXXX.png")"
trap 'rm -f "$TMP_PNG"' EXIT

"$BIN" --capture-frame "$TMP_PNG"
if command -v file >/dev/null 2>&1; then
    file "$TMP_PNG"
    if ! file "$TMP_PNG" | grep '1344 x 756' >/dev/null 2>&1; then
        fail "captura não gerou PNG 1344x756: $TMP_PNG"
    fi
elif [ ! -s "$TMP_PNG" ]; then
    fail "captura não gerou arquivo: $TMP_PNG"
fi
rm -f "$TMP_PNG"
trap - EXIT
printf 'captura de sanidade: OK\n'

step "Permissão no framebuffer"
if [ ! -w /dev/fb0 ]; then
    cat >&2 <<EOF
erro: o usuário $USER_NAME não tem permissão de escrita em /dev/fb0

Adicione o usuário ao grupo video:
  sudo usermod -aG video "$USER_NAME"

Depois entre em um VT local (Ctrl+Alt+F3), faça login e rode:
  cd "$ROOT"
  ./user_run_now.sh
EOF
    exit 3
fi
printf 'permissão de escrita em /dev/fb0: OK\n'

if [ "$RUN" -eq 0 ]; then
    step "Modo --check"
    printf 'ambiente, build e captura validados.\n'
    printf 'Para testar a interface: Ctrl+Alt+F3, login e execute ./user_run_now.sh\n'
    exit 0
fi

if [ ! -t 0 ] || [ ! -t 1 ]; then
    fail "execute este script em um terminal interativo no VT"
fi

TTY_NAME="$(tty 2>/dev/null || true)"
case "$TTY_NAME" in
    /dev/tty[0-9]*)
        ;;
    *)
        cat >&2 <<EOF
erro: este shell não é um VT local (tty atual: ${TTY_NAME:-desconhecido}).

O backend fb exige um console local. Saia de X11/Wayland/SSH e faça:
  1. Ctrl+Alt+F3 (ou outro VT livre)
  2. faça login
  3. cd "$ROOT"
  4. ./user_run_now.sh
EOF
        exit 3
        ;;
esac

step "Iniciando calculadora"
printf 'tty: %s\n' "$TTY_NAME"
printf 'atalhos: F2 idioma | M som | Q/Esc/Ctrl+C sair | ↑/↓ histórico | Enter recall\n'
printf 'flags: %s\n' "${APP_ARGS[*]:-nenhuma}"

set +e
"$BIN" "${APP_ARGS[@]}"
STATUS=$?
set -e

printf '\ncalculadora encerrou com código %s\n' "$STATUS"
exit "$STATUS"
