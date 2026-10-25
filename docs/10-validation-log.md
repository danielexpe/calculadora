# 10 — Registro de validação

> **Regra:** só entra aqui o que foi realmente executado. Hardware não testado
> fica como `PENDENTE`, com a causa e o que falta. Não inventar resultados.

## 1. Ambiente auditado — 2026-10-25

| Item | Resultado |
| --- | --- |
| Sistema | Linux x86_64 local |
| `rustc` | 1.99.1 (58cf8f9c3 2026-10-13) |
| `cargo` | 1.99.1 (5f94df478 2026-08-27) |
| `/dev/fb0` | existe; `crw-rw---- root video` |
| Driver | `amdgpudrmfb` |
| Resolução virtual | 1920 × 1080 |
| bpp | 32 |
| Dispositivos gráficos | `fb0`, `fbcon` |
| Grupo do usuário | `daniel` **não** está em `video` |
| Sessão | Wayland (`XDG_SESSION_TYPE=wayland`, `DISPLAY=:0`) |
| VT ativo | `tty2` (ocupado pela sessão gráfica) |
| `/dev/tty` no shell | **não é TTY** |
| Repositório git | inicializado em 2026-10-25 (`main`); commit inicial `ed07e19` |

Comandos executados:

```bash
ls -l /dev/fb0 /dev/tty
id -nG
rustc --version
cargo --version
cat /sys/class/graphics/fb0/name
cat /sys/class/graphics/fb0/virtual_size
cat /sys/class/graphics/fb0/bits_per_pixel
cat /sys/class/tty/tty0/active
echo "$XDG_SESSION_TYPE $DISPLAY $WAYLAND_DISPLAY"
```

Saídas relevantes:

```text
crw-rw---- 1 root video 29, 0 out 25 01:46 /dev/fb0
crw-rw-rw- 1 root tty    5, 0 out 25 01:46 /dev/tty
daniel adm cdrom sudo dip plugdev users lpadmin sambashare libvirt ollama docker kvm
amdgpudrmfb
1920,1080
32
tty2
wayland :0 wayland-0
```

Interpretação:

- O device existe e o formato alvo está presente.
- O usuário **não** tem permissão de escrita no fb (grupo `video` ausente).
- A sessão atual não é um VT de console; o backend fb exige VT local.
- `/dev/tty` no shell atual aponta para um pty, não para o VT; a validação real
  precisa ser feita a partir de um VT (ex.: tty3).
- O repositório git local foi inicializado em `main`; commit inicial `ed07e19`.
- O binário da Fase 1 existe, mas é um placeholder: a CLI é executada nos
  testes de processo; fb, captura e áudio ainda não foram implementados.

## 2. Cache de crates — 2026-10-25

Presentes no cache local e usadas na Fase 1:

```text
clap 4.6.7
crossterm 0.29.0
fontdue 0.9.5
rust_decimal 1.43.0  (baixada em 2026-10-25)
thiserror 2.0.21
libloading 0.8.9     (opcional/feature audio)
arrayvec 0.7.8
num-traits 0.2.19
clap_derive 4.6.7
proc-macro2 / quote / syn
```

Reservadas para fases seguintes, já em cache:

```text
tiny-skia 0.12.0
image 0.25.10
libc 0.2.191
bytemuck 1.25.2
zerocopy 0.8.63
```

`Cargo.lock` foi gerado e versionado. `rust_decimal_macros` não é necessário.
A licença MIT de `rust_decimal` 1.43.0 e as licenças das crates ativas foram
registradas em `THIRD_PARTY_NOTICES.md`.

### 2.1 Dependências da Fase 2

Adicionadas com aprovação explícita do usuário e já presentes no cache:

```text
tiny-skia 0.12.0      (BSD-3-Clause)
tiny-skia-path 0.12.0 (BSD-3-Clause)
image 0.25.10         (MIT OR Apache-2.0; feature png)
libc 0.2.191          (MIT OR Apache-2.0)
bytemuck 1.25.2       (Zlib OR Apache-2.0 OR MIT)
```

Transitivas de PNG/raster (`png`, `flate2`, `miniz_oxide`, `fdeflate`,
`crc32fast`, `moxcms`, `pxfm`, `adler2`, `simd-adler32`, `byteorder-lite`,
`arrayref`, `strict-num`) entraram em `THIRD_PARTY_NOTICES.md`.

### 2.2 Assets baixados

| Asset | Versão | SHA-256 (prefixo) |
| --- | --- | --- |
| Inter-Regular.ttf | 4.1 | `40d692fc…` |
| Inter-SemiBold.ttf | 4.1 | `78a843fa…` |
| JetBrainsMono-Regular.ttf | 2.304 | `a0bf60ef…` |
| JetBrainsMono-Medium.ttf | 2.304 | `31c92d01…` |
| OFL-Inter.txt | — | `262481e8…` |
| OFL-JetBrainsMono.txt | — | `30f0c136…` |

Total: 1.387.923 bytes. Hashes completos em
`docs/04-assets-and-licenses.md`; manifesto em
`assets/generated/manifest.txt`.

## 3. Validações executadas

| Data | Validação | Resultado | Evidência |
| --- | --- | --- | --- |
| 2026-10-25 | Auditoria de ambiente | concluída | seção 1 |
| 2026-10-25 | Cache de crates | concluída | seção 2 |
| 2026-10-25 | Documentos 01–10 + AGENTS | concluída | arquivos no repo |
| 2026-10-25 | Repositório git local | concluída | `git init -b main`; commit inicial `ed07e19` |
| 2026-10-25 | Dependências e fontes | concluída | `rust_decimal` 1.43.0; Inter 4.1; JBM 2.304 |
| 2026-10-25 | `cargo fmt --all -- --check` | verde | Fase 1 |
| 2026-10-25 | `cargo clippy --workspace --all-targets -- -D warnings` | verde | Fase 1 |
| 2026-10-25 | `cargo clippy --workspace --all-targets --no-default-features -- -D warnings` | verde | Fase 1 |
| 2026-10-25 | `cargo test --workspace` | **112 verdes** | 106 unitários + 5 processo + 1 xtask |
| 2026-10-25 | `cargo test --workspace --no-default-features` | **112 verdes** | Fase 1 |
| 2026-10-25 | `cargo xtask assets --check` | verde | manifesto atualizado |
| 2026-10-25 | `cargo build --release` | verde | binário placeholder |
| 2026-10-25 | `cargo build --release --no-default-features` | verde | binário placeholder |
| 2026-10-25 | Binário copiado para `/tmp` sem assets | verde | `--version`/`--help`; runtime sai com 3 |
| 2026-10-25 | `cargo fmt`/`clippy`/`test` com/sem features (Fase 2) | verde | 154 unit + 8 processo + 1 xtask |
| 2026-10-25 | `--capture-frame` release | verde | PNG 1344 × 756; ~40 ms; hash estável |
| 2026-10-25 | Build release com/sem features | verde | binário 3,4 MB |
| 2026-10-25 | Abertura `/dev/fb0` sem permissão | verde (erro esperado) | código 3; instrução do grupo `video` |
| — | Execução em `/dev/fb0` em VT | **PENDENTE** | P-01/P-02: grupo `video` e VT livre |
| — | 24 bpp / 16 bpp | **PENDENTE** | hardware/VM não confirmados |
| — | Áudio real | **PENDENTE** | Fase 4 |
| — | Restauração de console | **PENDENTE** | implementada com RAII/sinais/panic; falta VT real |

## 3.1 Evidência da Fase 2 (headless)

Comando:

```bash
./target/release/calculadora --capture-frame /tmp/calculadora.png
```

Resultado:

```text
wrote /tmp/calculadora.png (1344x756)
elapsed=0.04 s
PNG 1344 x 756, 8-bit/color RGBA
sha256 67611a5c...74bb1f (idêntico em duas execuções)
```

- A cena inicial foi revisada visualmente: fundo em gradiente, fita vazia,
  visor `0`, keypad 4 × 5, rodapé com idioma e som.
- O codepath de captura não chama `Framebuffer`, `ConsoleGuard`, raw mode nem
  áudio; o teste de processo usa `--fb-device` inexistente para provar que a
  captura o ignora.
- Testes cobrem ABI 160/80, formatos 32/24/16, rejeição de formato inválido,
  padding de `line_length`, geometria 1080p/1440p/768p/1024×768, blend
  pré-multiplicado, cobertura de glifos, determinismo e PNG válido.
- Execução sem permissão no fb retorna código 3 com instrução acionável.
- **Não validado em VT real**; P-01/P-02 continuam abertos.

## 4. Bloqueios ativos

| ID | Bloqueio | O que falta | Dono |
| --- | --- | --- | --- |
| B-01 | Sem permissão no fb | `sudo usermod -aG video "$USER"` + novo login (ou `newgrp video`) | usuário |
| B-02 | Sessão Wayland no tty2 | escolher VT livre (ex.: tty3) e executar a partir dele | usuário |
| B-05 | 24/16 bpp não confirmados | testar em hardware/VM ou marcar pendente no release | usuário/agente |

Resolvidos na Fase 1:

- B-03: `rust_decimal` 1.43.0 baixada, pinada e registrada.
- B-04: quatro TTF OFL e textos de licença baixados e verificados.
- Binário ainda é placeholder; fb/captura não são bloqueios de hardware, são
  escopo da Fase 2.

Nenhum bloqueio de hardware foi declarado validado.

## 5. Procedimento de validação futura

### 5.1 Preparação

```bash
sudo usermod -aG video "$USER"
# sair da sessão e entrar novamente, ou:
newgrp video
id -nG   # conferir 'video'
```

### 5.2 Captura sem hardware (CI/dev)

```bash
cargo build --release
./target/release/calculadora --capture-frame /tmp/calculadora.png
file /tmp/calculadora.png
sha256sum /tmp/calculadora.png
```

- Executar duas vezes e comparar os bytes RGBA/hash.
- Confirmar que não abriu `/dev/fb0` (teste com `strace -e openat`, se
  disponível).
- Conferir 1344 × 756.

### 5.3 Framebuffer em VT

```bash
ls -l /dev/fb0
cat /sys/class/graphics/fb0/name
cat /sys/class/graphics/fb0/bits_per_pixel
id -nG
./target/release/calculadora
```

- Trocar para um VT local antes de executar (ex.: Ctrl+Alt+F3).
- Registrar driver, resolução, bpp e escala observada.
- Sair por `Q`, `Esc` e `Ctrl+C`; conferir restauração.
- Testar `kill -TERM`, `kill -HUP` e pânico induzido em build de teste.
- Registrar resíduos ou cursor piscando.

### 5.4 Matriz manual

Seguir `docs/05-test-plan.md`, seção 9. Registrar cada item como
`OK`, `FALHOU` ou `PENDENTE`.

## 6. Modelo de entrada

```text
## <data> — <tipo de validação>

- Ambiente: <VT/driver/resolução/bpp>
- Build: <hash/versão>
- Comando: <...>
- Resultado: OK | FALHOU | PENDENTE
- Evidência: <captura, log, hash>
- Observações: <...>
```

## 7. Medições de desempenho

| Métrica | Alvo | Medido | Data/ambiente |
| --- | --- | --- | --- |
| Startup | < 400 ms | — | — |
| Frame animação 1080p | ≤ 16 ms | — | — |
| Frame ocioso | ≤ 33 ms | — | — |
| RSS | < 120 MB | — | — |
| CPU ocioso | < 2% | — | — |
| Binário release | < 15 MB | 3,4 MB (Fase 2) | 2026-10-25 |
| Assets | < 4 MB | 1.387.923 B (1,39 MB) | 2026-10-25 |
| Latência de tecla | < 50 ms | — | — |
| Captura | < 1 s | ~40 ms (release, headless) | 2026-10-25 |

Preencher apenas com medições reais em release.

## 8. Histórico de atualizações

| Data | Mudança |
| --- | --- |
| 2026-10-25 | Criação do log; auditoria inicial; nenhuma validação de software executada |
| 2026-10-25 | Repositório git local inicializado em `main`; `.gitignore` criado |
| 2026-10-25 | Fase 1: domínio, CLI, i18n, input, state, TTY, assets e xtask; 112 testes; fontes e `rust_decimal` baixados |
| 2026-10-25 | Commit inicial `ed07e19` com especificação e Fase 1 |
| 2026-10-25 | Fase 2: fbdev/ABI/formatos, console e sinais, tema, cena/texto, captura PNG, app estático; 163 testes no total |
