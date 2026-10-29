# Calculadora

> Cálculo essencial, com elegância.

Calculadora decimal para **console Linux local**, renderizada diretamente no
framebuffer (`/dev/fb0`), sem X11/Wayland, sem rede em runtime e sem servidor
gráfico. Interface escura, tipografia cuidada, fita de histórico e suporte a
português e inglês.

- Repositório: https://github.com/danielexpe/calculadora
- Release 1.0.2: https://github.com/danielexpe/calculadora/releases/tag/v1.0.2
- Licença do código: MIT

## Sumário

- [Para quem só quer usar](#para-quem-só-quer-usar)
  - [Requisitos do sistema](#requisitos-do-sistema)
  - [Baixar e instalar](#baixar-e-instalar)
  - [Liberar acesso ao framebuffer](#liberar-acesso-ao-framebuffer)
  - [Executar no console](#executar-no-console)
  - [Usar a calculadora](#usar-a-calculadora)
  - [Áudio](#áudio)
  - [Atualizar e desinstalar](#atualizar-e-desinstalar)
  - [Solução de problemas](#solução-de-problemas)
- [Para desenvolvedores](#para-desenvolvedores)
  - [Dependências de compilação](#dependências-de-compilação)
  - [Compilar e executar](#compilar-e-executar)
  - [Testes e portões](#testes-e-portões)
  - [Captura PNG e QA sem hardware](#captura-png-e-qa-sem-hardware)
  - [Estrutura do projeto](#estrutura-do-projeto)
  - [Empacotamento](#empacotamento)
  - [Contribuição](#contribuição)
- [Estado, limitações e licenças](#estado-limitações-e-licenças)

---

## Para quem só quer usar

O pacote oficial é um binário Linux x86_64 autossuficiente. **Não é necessário
instalar Rust, compilador, fontes ou arquivos de arte.** As fontes e o layout
estão embutidos no executável.

### Requisitos do sistema

| Item | Necessário |
| --- | --- |
| Sistema | Linux x86_64 |
| Console | VT local com fbdev e `/dev/fb0` |
| Permissão | usuário no grupo `video` (ou ACL equivalente no device) |
| Bibliotecas runtime | `libc.so.6`, `libm.so.6` e `libgcc_s.so.1` (glibc, não musl) |
| Áudio | opcional; ALSA/PipeWire com `libasound.so.2` |
| Rede | não é usada pelo aplicativo em runtime |

O alvo validado é `amdgpudrmfb`, 1920 × 1080, 32 bpp. O app escala o canvas para
outras resoluções e aceita framebuffers de 32, 24 e 16 bpp, mas a validação em
hardware foi feita apenas no alvo acima. **Não funciona** a partir de um
terminal dentro de X11/Wayland, SSH ou tmux; é necessário um VT de console.

Diagnóstico rápido:

```bash
ls -l /dev/fb0
cat /sys/class/graphics/fb0/name
cat /sys/class/graphics/fb0/virtual_size
cat /sys/class/graphics/fb0/bits_per_pixel
id -nG
```

Se `/dev/fb0` não existir, o sistema não expõe framebuffer fbdev e o app não
pode ser usado nele. Se existir mas não for gravável, veja a seção de
permissões.

### Baixar e instalar

Baixe o pacote e o checksum da release `v1.0.2`:

```bash
cd /tmp

curl -fLO \
  https://github.com/danielexpe/calculadora/releases/download/v1.0.2/calculadora-1.0.2-linux-x86_64.tar.gz
curl -fLO \
  https://github.com/danielexpe/calculadora/releases/download/v1.0.2/calculadora-1.0.2-linux-x86_64.tar.gz.sha256

sha256sum -c calculadora-1.0.2-linux-x86_64.tar.gz.sha256
```

O checksum detecta download corrompido; ele não substitui uma assinatura
criptográfica. Depois extraia e execute o instalador:

```bash
tar -xzf calculadora-1.0.2-linux-x86_64.tar.gz
cd calculadora-1.0.2-linux-x86_64
./install.sh
```

O instalador copia o binário para `$HOME/.local/bin/calculadora`, instala a
documentação em `$HOME/.local/share/doc/calculadora` e, se necessário, pede a
senha de `sudo` para adicionar seu usuário ao grupo `video`. Use `--prefix DIR`
para escolher outro destino, `--check` para só validar o pacote e `--skip-fb`
para não mexer em permissões.

Saída esperada: `calculadora 1.0.2`.

Se `$HOME/.local/bin` não estiver no `PATH`, use o caminho completo
`"$HOME/.local/bin/calculadora"` ou adicione a pasta ao `PATH` no seu shell.

O pacote contém:

- `calculadora` — executável;
- `install.sh` — instalador e desinstalador;
- `README.md`, `RELEASE_NOTES.md`;
- `LICENSE`, `LICENSE-ART`;
- `THIRD_PARTY_NOTICES.md`;
- `licenses/OFL-Inter.txt`, `licenses/OFL-JetBrainsMono.txt`.

O diretório `assets/` do código-fonte **não** é necessário para executar.

### Liberar acesso ao framebuffer

A permissão típica é `root:video` com modo `660`. O `install.sh` detecta a
falta de acesso e pede a senha de `sudo` para adicionar seu usuário ao grupo
`video`. Para fazer manualmente:

```bash
sudo usermod -aG video "$USER"
```

O grupo só passa a valer em uma nova sessão. Saia e entre novamente, reinicie,
ou faça login em um VT novo. Para conferir:

```bash
id -nG          # deve listar video
test -w /dev/fb0 && echo "fb gravável"
```

Em alguns sistemas o device usa ACL; nesse caso a escrita pode funcionar mesmo
sem o grupo. Não é necessário executar a calculadora com `sudo`.

### Executar no console

1. Salve o caminho do binário: `$HOME/.local/bin/calculadora`.
2. Troque para um VT livre com `Ctrl+Alt+F3` (ou F4, F5…).
3. Faça login com o mesmo usuário.
4. Execute:

```bash
"$HOME/.local/bin/calculadora"
```

Também é possível trocar de VT a partir da sessão gráfica:

```bash
sudo chvt 3
```

Para voltar à sessão gráfica, use `Ctrl+Alt+F2` ou o VT correspondente.

Ao iniciar, o app entra em modo gráfico do console, esconde o cursor, desenha a
interface e restaura o console quando você sai com `Q`, `Esc`, `Ctrl+C` ou
`SIGTERM`. Evite trocar de VT enquanto o app estiver aberto; se isso acontecer,
volte ao VT e saia normalmente.

### Usar a calculadora

A interface tem duas áreas: a fita de histórico à esquerda e o card de cálculo
à direita. Tudo é operado pelo teclado; não há mouse.

#### Teclas

| Tecla | Ação |
| --- | --- |
| `0`–`9` (fileira superior e numpad) | digita dígito |
| `,` ou `.` | separador decimal |
| `+` `-` `*` `/` | soma, subtração, multiplicação e divisão |
| `Enter` ou `=` | calcula; com histórico selecionado, recupera o resultado |
| `Backspace` | apaga o último dígito |
| `Delete` ou `C` | limpa tudo (`AC`) |
| `S` | troca o sinal (`±`) |
| `%` | porcentagem contextual |
| `↑` / `↓` | navega na fita de histórico |
| `F2` | alterna português (BR) / inglês |
| `M` | liga/desliga o som |
| `Q`, `Esc`, `Ctrl+C` | sai e restaura o console |

**Numpad:** ative `NumLock`; com ele desligado, o teclado numérico envia
setas/Home/End e o app as interpreta como navegação ou teclas desconhecidas.

#### Exemplos

| Entrada | Resultado |
| --- | --- |
| `2 + 3 =` | `5` |
| `0,1 + 0,2 =` | `0,3` |
| `2 + 3 × 4 =` | `20` (avaliação sequencial) |
| `200 + 10 % =` | `220` |
| `200 − 10 % =` | `180` |
| `200 × 10 % =` | `20` |
| `200 ÷ 10 % =` | `2000` |
| `10 %` (sem operação pendente) | `0,1` |
| `1 ÷ 0 =` | `Erro` / `Divisão por zero` |

Após um erro, digite um número ou pressione `C`/`Delete` para recomeçar.

#### Histórico

- Até 50 operações por sessão; 10 visíveis por vez.
- Cada operação resolvida vira uma entrada `lhs op rhs = resultado`.
- `↑` seleciona a entrada mais recente e depois as mais antigas.
- `↓` avança para entradas mais novas; além da mais recente, limpa a seleção.
- `Enter` com uma entrada selecionada copia o resultado para o visor, sem criar
  nova entrada.
- A fita é reformatada ao trocar o idioma.
- O histórico existe somente em memória; fechar o app apaga a fita.

#### Opções de linha de comando

```text
calculadora [OPTIONS]

  --lang <pt|en>                 idioma inicial (padrão: pt)
  --backend <fb>                 backend de vídeo (v1: somente fb)
  --fb-device <PATH>             dispositivo (padrão: /dev/fb0)
  --color <auto|truecolor|256|16|never>
  --no-animations                desliga fade e reduz o flash
  --no-audio                     inicia em silêncio
  --capture-frame <PATH>         gera PNG determinístico e sai
  -h, --help
  -V, --version
```

Exemplos:

```bash
"$HOME/.local/bin/calculadora" --lang en
"$HOME/.local/bin/calculadora" --color 256
"$HOME/.local/bin/calculadora" --color 16
"$HOME/.local/bin/calculadora" --color never
NO_COLOR=1 "$HOME/.local/bin/calculadora"
"$HOME/.local/bin/calculadora" --no-animations --no-audio
```

`NO_COLOR` tem prioridade sobre `--color`. `--color auto` usa truecolor em
32/24 bpp e 256 cores em 16 bpp. A captura PNG não abre framebuffer, terminal
interativo nem áudio e pode ser usada em qualquer sessão:

```bash
"$HOME/.local/bin/calculadora" --capture-frame /tmp/calculadora.png
file /tmp/calculadora.png
```

### Áudio

O áudio é opcional e sintetizado em runtime; não há arquivos de som. O app
carrega `libasound.so.2` dinamicamente:

- Com ALSA/PipeWire disponível: clique nas teclas aceitas e tom grave em erros.
- Sem biblioteca ou dispositivo: o rodapé mostra `Som: indisponível` e o app
  continua funcionando em silêncio.
- `M` alterna mudo; `--no-audio` inicia mudo.

O binário não exige pacote de desenvolvimento ALSA para rodar. Se quiser áudio
em uma instalação mínima, instale o pacote ALSA da sua distribuição (por
exemplo, `libasound2` ou `libasound2t64` no Debian/Ubuntu, `alsa-lib` no
Fedora/Arch) e garanta um dispositivo de saída configurado.

### Atualizar e desinstalar

Para atualizar, baixe a nova release, confira o `.sha256`, extraia e execute
`./install.sh` novamente; o instalador substitui o binário e a documentação.

Para desinstalar:

```bash
./install.sh --uninstall
```

Ou, manualmente:

```bash
rm -f "$HOME/.local/bin/calculadora"
rm -rf "$HOME/.local/share/doc/calculadora"
```

Remova também o usuário do grupo `video` se ele não for mais necessário:

```bash
sudo gpasswd -d "$USER" video
```

### Solução de problemas

| Sintoma | Causa provável | Ação |
| --- | --- | --- |
| `cannot open /dev/fb0: permission denied` | usuário fora de `video` | `sudo usermod -aG video "$USER"`, novo login |
| `cannot open /dev/fb0: No such file` | sistema sem fbdev | use uma máquina/VM com `/dev/fb0` |
| `KDSETMODE failed` ou “run from a local virtual console” | terminal em X11/Wayland/SSH | abra um VT local (`Ctrl+Alt+F3`) e execute lá |
| `stdin is not a TTY` | entrada redirecionada/pipe | execute interativamente no VT |
| `unsupported framebuffer format` | bpp/bitfields diferentes de 32/24/16 | registre `bits_per_pixel` e o driver do fb |
| tela não aparece, mas o processo continua | outro VT está ativo | volte ao VT onde o app foi iniciado |
| teclado numérico não digita | `NumLock` desligado | ative `NumLock` |
| `Som: indisponível` | sem ALSA/dispositivo | app segue em silêncio; configure PipeWire/ALSA |
| som não toca | mudo ativo ou `--no-audio` | pressione `M`; confira o marcador no rodapé |
| captura não cria arquivo | diretório inexistente/sem permissão | use um caminho gravável, como `/tmp/x.png` |

Códigos de saída:

| Código | Significado |
| ---: | --- |
| `0` | sucesso (inclui captura e saída normal) |
| `1` | erro de runtime/IO/captura |
| `2` | erro de CLI (flag inválida; `tui`/`auto` é rejeitado) |
| `3` | backend indisponível: fb sem permissão, fora de VT ou stdin não é TTY |

---

## Para desenvolvedores

### Dependências de compilação

| Dependência | Por quê |
| --- | --- |
| Linux x86_64 | plataforma oficial do backend fbdev |
| Rust stable 1.88+ | `rust-version` do crate; release gerado com 1.99.1 |
| `rustup` | forma recomendada de obter o toolchain |
| linker C (`cc`/`gcc`) | linkedição do binário Rust |
| `cargo` | build, testes e xtask |
| `git` | clonar o repositório |
| `curl` | instalar o rustup e baixar releases |
| ALSA runtime (opcional) | testar áudio; a compilação usa `libloading`, não `-dev` |
| `/dev/fb0` + VT | teste manual real; não é necessário para testes headless |

Não são necessários `libX11`, `libwayland`, `pkg-config`, `libasound2-dev`,
fontes do sistema ou arquivos de arte externos. As fontes TTF são embutidas no
binário. O primeiro build baixa as crates do registry e, portanto, precisa de
rede; o binário final não usa rede em runtime.

Instalação do toolchain:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup toolchain install stable
rustc --version
cargo --version
```

Exemplos de pacotes de build:

```bash
# Debian/Ubuntu
sudo apt update && sudo apt install -y build-essential git curl

# Fedora
sudo dnf install -y gcc make git curl

# Arch
sudo pacman -S --needed base-devel git curl
```

### Compilar e executar

```bash
git clone https://github.com/danielexpe/calculadora.git
cd calculadora

cargo build --release
./target/release/calculadora --version
```

O binário fica em `target/release/calculadora`. Ele já contém fontes e layout;
pode ser copiado para outro diretório e executado sem `assets/`.

Para testar no console real:

```bash
./user_run_now.sh --check   # valida build, fb, grupo, ALSA e captura
./user_run_now.sh           # execute dentro de um VT local
```

O script compila a release com áudio, valida `/dev/fb0` e só inicia o app em um
VT. Use `--no-build` para pular o build e repasse flags normalmente:

```bash
./user_run_now.sh --color 16 --lang en
```

### Testes e portões

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --workspace --all-targets --no-default-features -- -D warnings
cargo test --workspace
cargo test --workspace --no-default-features
cargo xtask assets --check
cargo build --release
cargo build --release --no-default-features
```

Estado atual: **194 testes ativos verdes** — 181 unitários, 9 de processo/CLI,
1 de capturas douradas, 1 de PTY, 1 de sinais e 1 do xtask — além de 2 testes
auxiliares `#[ignore]` usados pelos testes de PTY/sinais.

Testes específicos:

```bash
cargo test --test golden        # 16 capturas douradas por RGBA
cargo test --test pty_guard     # raw mode em PTY via `script`
cargo test --test signals       # flag de SIGTERM em processo filho
cargo test --release            # suíte em otimizado (opcional)
```

Os testes PTY/sinais são pais que executam os filhos `#[ignore]`; precisam de
`script` e `kill` no sistema.

#### Capturas douradas

As 16 capturas ficam em `tests/snapshots/` e são comparadas por bytes RGBA:

```bash
cargo test --test golden
UPDATE_SNAPSHOTS=1 cargo test --test golden   # regenerar após revisão visual
```

Sempre revise o diff visual antes de aceitar snapshots novos. Detalhes em
`docs/05-test-plan.md`.

### Captura PNG e QA sem hardware

```bash
tmp=$(mktemp -d)
./target/release/calculadora --capture-frame "$tmp/pt.png"
./target/release/calculadora --lang en --color never \
  --capture-frame "$tmp/en-mono.png"

file "$tmp/pt.png" "$tmp/en-mono.png"
sha256sum "$tmp/pt.png"
```

Saída esperada: PNG 1344 × 756 RGBA8. Na mesma toolchain, duas capturas iguais
produzem bytes idênticos. A captura não abre fb, TTY nem áudio; ela é o caminho
principal de QA em CI/sem console.

### Estrutura do projeto

```text
src/
  main.rs / cli.rs        entrada, flags, códigos de saída
  app.rs                  estado de UI, roteamento, flash/debounce/fade, loop fb
  input.rs                mapeamento de teclas para Action
  audio.rs                síntese de clique/erro
  capture.rs              PNG determinístico
  core/                   domínio puro (decimal, cálculo, histórico)
  state/                  sessão (idioma, som, seleção)
  render/                 cena, layout, texto, blend, compositor fb
  platform/               fbdev/console/sinais e FFI ALSA
  theme.rs                paletas truecolor/256/16/mono
  i18n/                   catálogo pt/en
  util/                   guardas de TTY/raw mode
tests/
  golden.rs               comparação das capturas
  pty_guard.rs            restauração de raw mode
  signals.rs              SIGTERM
  snapshots/              16 PNGs dourados
assets/fonts/             TTF + textos OFL
docs/                     especificação, design, testes, decisões, release
xtask/                    manifesto/verificação de assets
```

### Empacotamento

A release oficial é montada a partir do binário release, instalador, README,
licenças, notices, notas e textos OFL. O processo completo e o checklist estão
em `docs/09-release-checklist.md`. Resumo:

```bash
cargo build --release
mkdir -p dist/calculadora-1.0.2-linux-x86_64/licenses
cp target/release/calculadora install.sh dist/calculadora-1.0.2-linux-x86_64/
cp README.md RELEASE_NOTES.md LICENSE LICENSE-ART THIRD_PARTY_NOTICES.md \
  dist/calculadora-1.0.2-linux-x86_64/
cp assets/fonts/OFL-Inter.txt assets/fonts/OFL-JetBrainsMono.txt \
  dist/calculadora-1.0.2-linux-x86_64/licenses/
chmod +x dist/calculadora-1.0.2-linux-x86_64/install.sh

tar -czf dist/calculadora-1.0.2-linux-x86_64.tar.gz \
  -C dist calculadora-1.0.2-linux-x86_64
(cd dist && sha256sum calculadora-1.0.2-linux-x86_64.tar.gz \
  > calculadora-1.0.2-linux-x86_64.tar.gz.sha256)
```

Antes de publicar, teste o binário extraído em `/tmp` sem `assets/` e valide o
`.sha256`. O hash do pacote não deve ser colocado nas próprias notas de release.

### Contribuição

- Leia `AGENTS.md` e `docs/01-calculadora.md` antes de mudar comportamento.
- Regras de cálculo, arredondamento, percentual e histórico são congeladas;
  mudanças exigem ADR em `docs/07-decisions-and-risks.md`.
- Identificadores/código em inglês; strings de UI exclusivamente no catálogo
  `src/i18n/strings.rs` (pt-BR/en).
- Não adicione dependências, rede em runtime ou `unsafe` fora de
  `src/platform/fb.rs` e `src/platform/audio.rs` sem discussão prévia.
- Rode todos os portões e revise as capturas douradas antes de abrir PR.
- Preserve a restauração do console em todos os caminhos de saída.

Issues e pull requests: https://github.com/danielexpe/calculadora

---

## Estado, limitações e licenças

**Versão 1.0.2 (2026-10-28).** Fluxo, áudio, histórico, fade, paletas e saída
foram validados manualmente em VT no alvo `amdgpudrmfb` 1920 × 1080 32 bpp.
A 1.0.2 desbloqueia o console com `FBIOBLANK`/`FB_BLANK_UNBLANK` quando ele
chega com `blank`/DPMS desligado, mantém a correção de `yoffset`/double buffer
e melhora o fallback de pan. O instalador automático continua no pacote.

Limitações conhecidas:

- fbdev apenas; não há TUI, DRM/KMS, X11/Wayland nativo ou suporte a SSH.
- Sem mouse, persistência, funções científicas ou repetição da última operação.
- 32 bpp validado; 24/16 bpp e resoluções alternativas têm suporte de código,
  mas validação de hardware pendente.
- Sem vsync; troca de VT durante a execução pode deixar resíduos.
- Áudio depende de ALSA/PipeWire; indisponibilidade não é erro fatal.
- MSRV declarada 1.88; o release foi gerado com Rust 1.99.1.
- `SIGHUP`/pânico no console e medições formais de CPU/latência ainda não foram
  registrados individualmente.

Licenças:

- Código: **MIT** (`LICENSE`).
- Interface/arte original: **CC BY 4.0** (`LICENSE-ART`).
- Fontes: **SIL OFL 1.1** (Inter 4.1 e JetBrains Mono 2.304), textos em
  `assets/fonts/`.
- Dependências: `THIRD_PARTY_NOTICES.md`.

O projeto usa a forma arquitetônica descrita em `framework.md`, mas não faz
parte do projeto original que inspirou esse template.
