# 02 — Especificação técnica

> Arquitetura derivada do `framework.md`, adaptada à decisão **fb-only** da v1.
> Desvios estão registrados em `docs/07-decisions-and-risks.md` (D-001, D-002,
> D-008, D-011).

## 1. Premissas

- Linux x86_64 local, VT ativo, `/dev/fb0` com framebuffer fbdev.
- Rust stable, edição 2021; MSRV alvo **1.88**, ambiente de desenvolvimento
  atual **1.99.1**.
- Sem rede em runtime, sem telemetria, sem arquivos externos ao binário.
- Entrada apenas por teclado; mouse fora do escopo.
- Fontes e strings embutidas; nenhum asset é lido do disco em runtime.
- Backend de vídeo único: framebuffer. Captura PNG é o caminho de QA sem
  hardware.
- Sem RNG: toda aleatoriedade está fora do produto.

## 2. Camadas e dependências

```text
main.rs / cli.rs
      |
      v
app.rs (estado de UI, loop, timers, áudio, input)
  |          |                |
  v          v                v
state/    render/           audio.rs
  |       /      \             |
  v      v        v            v
core/  scene    fb.rs       assets.rs
(puro)  text     |           theme.rs
        layout   v           i18n/
              platform/fb.rs
              (libc, ioctl, mmap, sinais)
```

Regras:

- `core/` não importa UI, framebuffer, áudio, assets, fontes ou crossterm.
- `core/` pode usar `rust_decimal` e tipos próprios; é testável sem terminal.
- `state/` modela seleção de histórico, idioma, status de som e animação;
  pode depender de `core/`.
- `app.rs` é o único que lê relógio, chama áudio, lê eventos e decide redesenho.
- `render/scene.rs` descreve a cena em primitivas lógicas; não conhece fbdev.
- `render/fb.rs` rasteriza a cena e compõe no framebuffer; usa `tiny-skia`,
  `fontdue` e `theme`.
- `platform/fb.rs` é o único módulo com `unsafe` de sistema (ioctl, `mmap`,
  sinais), sempre com comentário `// SAFETY:`.
- `capture.rs` usa o mesmo renderizador lógico e grava PNG sem abrir fb/tty.
- `util/tty.rs` contém guardas de terminal e checagens de TTY, sem estado
  global.
- `audio.rs` isola ALSA dinâmico; ausência de biblioteca é silêncio.

## 3. Módulos

```text
src/
  main.rs            # CLI, escolha capture/fb, códigos de saída
  lib.rs             # reexporta módulos para testes de integração
  cli.rs             # flags, enums, validação
  app.rs             # App, loop fb, update, draw, roteamento de ações
  input.rs           # Action + map_key (topo e numpad)
  core/
    mod.rs
    decimal.rs       # parse, arredondamento, formatação localizada
    calculator.rs    # máquina de estados pura do cálculo
    operator.rs      # Operator, aplicação, símbolos
    history.rs       # entrada estruturada + fita
    error.rs         # CalcError
  state/
    mod.rs           # SessionState: seleção, idioma, som, animação
  render/
    mod.rs
    layout.rs        # canvas, geometria, breakpoints internos, escala
    scene.rs         # primitivas lógicas (retângulo, texto, linha)
    text.rs          # fontdue, cache de glifos, medição, ellipsis
    blend.rs         # blend RGBA pré-multiplicado com cobertura
    fb.rs            # rasterização, escala, composição e captura
  platform/
    mod.rs
    fb.rs            # device, console, ioctls, mmap, sinais, guardas
  audio.rs           # síntese e ALSA dinâmico (feature audio)
  assets.rs          # include_bytes!, decode e cache de fontes
  capture.rs         # PNG determinístico de QA
  i18n/
    mod.rs           # Lang, catálogo, fallback
    strings.rs       # tabela de strings
  theme.rs           # tokens semânticos e paletas por modo de cor
  util/
    tty.rs           # ensure_tty, guarda de raw mode
assets/
  fonts/             # TTF + OFL
  generated/         # manifesto de assets (sem PNG de arte)
xtask/
  src/main.rs        # valida assets, manifesto, --check
tests/
  golden.rs          # comparação RGBA contra PNGs dourados
  pty_guard.rs       # restauração de raw mode em PTY
  signals.rs         # flag de SIGTERM em processo filho
  snapshots/         # 16 PNGs dourados (1344 × 756)
.cargo/
  config.toml        # alias: xtask = run --package xtask --
.gitignore           # target/, dist/, pacotes de release
docs/
AGENTS.md
```

Não existem `render/tui.rs`, `render/image.rs` nem `util/rng.rs` na v1.

O alias do workspace foi criado na Fase 1:

```toml
[alias]
xtask = "run --package xtask --"
```

### 3.1 Status de implementação — 2026-10-25

| Módulo | Estado |
| --- | --- |
| `core/` (decimal, operador, histórico, calculator) | implementado e testado |
| `i18n/` (pt-BR/en) | implementado |
| `input.rs` | implementado |
| `state/` (seleção de histórico, idioma, som) | implementado |
| `cli.rs` | implementado |
| `util/tty.rs` | implementado |
| `assets.rs` (4 TTF, cobertura de glifos) | implementado |
| `xtask` (manifesto FNV-1a) | implementado |
| `theme.rs` (truecolor/256/16/mono) | implementado na Fase 2 |
| `render/` (scene, layout, text, blend, fb) | implementado na Fase 2 |
| `platform/fb.rs` (ABI, mmap, console, sinais) | implementado na Fase 2 |
| `capture.rs` (PNG 1344 × 756 determinístico) | implementado na Fase 2 |
| `app.rs` (`App` puro + loop fb, flash, debounce, histórico, saída) | implementado na Fase 3 |
| `main.rs` | capture e runtime fb ligados; códigos 0/1/2/3 |
| `audio.rs` | Fase 4 |

## 4. Dependências

| Área | Crate | Versão | Cache local | Fase |
| --- | --- | --- | --- | --- |
| CLI | `clap` (derive) | 4.6.7 | sim | 1 |
| Terminal/input | `crossterm` | 0.29.0 | sim | 1 |
| Texto | `fontdue` | 0.9.5 | sim | 1 |
| Decimal | `rust_decimal` | 1.43.0 | sim (baixada) | 1 |
| Erros | `thiserror` | 2.0.21 | sim | 1 |
| Áudio | `libloading` | 0.8.9 | sim | 4 (opcional) |
| Raster | `tiny-skia` | 0.12.0 | sim | 2 |
| PNG | `image` | 0.25.10 | sim | 2 |
| Sistema | `libc` | 0.2.191 | sim | 2 (direto); transitivo na 1 |
| Sistema | `bytemuck` | 1.25.2 | sim | 2 (derive Pod/Zeroable na ABI) |
| xtask | apenas `std` | — | — | 1 |

Política:

- O `Cargo.toml` da Fase 1 inclui `clap`, `crossterm`, `fontdue`,
  `rust_decimal`, `thiserror` e `libloading` opcional. Na Fase 2 entraram
  `tiny-skia`, `image`, `libc` e `bytemuck`, com aprovação explícita.
- `clap`: feature `derive`.
- `image`: `default-features = false`, feature `png` apenas.
- `bytemuck`: `derive` para `Pod`/`Zeroable` nas structs de ABI do fbdev.
- `libloading`: opcional, atrás da feature `audio`.
- **Proibido** adicionar `ratatui`, `ratatui-image`, `memmap2`, `rand`, `chrono`
  ou `anyhow` sem novo ADR e aprovação.
- `rust_decimal` 1.43.0 foi baixada, pinada no `Cargo.lock`, com
  `default-features = false` e feature `std`. Licença e notices foram
  conferidos.
- `xtask` implementa FNV-1a 64 bits localmente para o manifesto. Não é hash
  criptográfico; serve para detectar drift, não para segurança.
- Toda crate nova entra em `THIRD_PARTY_NOTICES.md` e no orçamento de tamanho.

```toml
[features]
default = ["audio"]
audio = ["dep:libloading"]
```

`--no-default-features` deve compilar e passar todos os testes; áudio vira
no-op.

## 5. Domínio puro

### 5.1 Tipos principais

```text
Decimal          rust_decimal::Decimal
Operator         Add | Sub | Mul | Div
NumberLocale     PtBr | En
CalcError        DivisionByZero | Overflow | Other
HistoryEntry     { lhs, op, rhs_input, rhs_effective, percent, result }
Calculator       máquina de estados
CalcEvent        DigitAccepted | SeparatorAccepted | OperatorApplied |
                 EqualsApplied | PercentApplied | Cleared | BackspaceApplied |
                 SignToggled | Recalled | Error(CalcError) | Ignored
```

`Action` (em `input.rs`) inclui tanto ações de cálculo quanto globais:

```text
Digit(char) | DecimalSeparator | Operator(Operator) | Equals | Backspace |
Clear | ToggleSign | Percent | HistoryPrev | HistoryNext |
ToggleLanguage | ToggleMute | Quit | Ignored
```

`Equals` vira recall quando o app detecta uma entrada de histórico
selecionada; essa decisão fica em `app.rs`/`state/`, não no domínio.

### 5.2 Invariantes

- `pending.is_some()` implica `accumulator.is_some()`.
- `error.is_some()` implica `pending.is_none()` e nenhuma mutação de histórico.
- O buffer de entrada tem no máximo 15 dígitos e no máximo um
  separador decimal.
- `result_mode` implica `pending.is_none()`.
- Toda entrada de histórico tem `result` consistente com a operação e o
  arredondamento aplicado.
- Nenhuma operação produz `NaN`/`Infinity`; erros são estados.
- Valores formatados nunca excedem 16 caracteres; a entrada ativa pode chegar
  a 17 (15 dígitos + separador + sinal), sem agrupamento quando necessário.
  Ver ADR D-017.

### 5.3 Regras

As regras de cálculo, porcentagem, erros, formatação e histórico estão
congeladas em `docs/01-calculadora.md`, seções 6–8. Testes de domínio devem
cobrir cada linha das tabelas.

### 5.4 Pureza

- `core::calculator::handle(action) -> CalcEvent` não toca em relógio, áudio,
  terminal ou fontes.
- `App` traduz `CalcEvent` em som e animação.
- A seleção da fita é estado de UI (`state/`), não do domínio; `Calculator`
  guarda apenas entradas.

## 6. Aplicação, loop e tempo

### 6.1 Estado da aplicação

```text
App {
  screen: Screen,              // Calculator (único na v1)
  calculator: Calculator,
  session: SessionState,       // idioma, seleção, animação, som
  theme: Theme,
  audio: Audio,
  quit: bool,
  dirty: bool,
}
```

### 6.2 Loop fb

```text
enter_guards()
while !quit:
    if signal_flag: quit = true
    update(now)                  # vence flash de tecla, debounce, animação
    if dirty or animating:
        draw()                   # constrói cena a partir de &App
        present()                # ypan ou escrita direta
        dirty = false
    timeout = animating ? 16ms : 100ms
    poll(timeout) -> eventos
    handle(action)
restore()                        # RAII + sinais + panic hook
```

- `update` nunca dorme; usa `Instant`.
- Flash de tecla: 90 ms.
- Debounce de `=`/`Enter`: 120 ms.
- Fade de nova entrada de histórico: 120 ms; desligado por
  `--no-animations`.
- Sem animação ambiente; o app fica ocioso e para de redesenhar.
- `Ctrl+C` em raw mode chega como `Digit`/`Char('c')` com modificador e deve ser
  mapeado para `Quit` antes da tela.

### 6.3 Determinismo

- Não há RNG nem `--seed`.
- Capturas usam estados fixos construídos pelo teste ou o estado inicial no
  CLI.
- Fontes, tema, idioma e modo de cor são explícitos na captura.
- Nenhum timestamp, PID ou caminho é impresso no PNG.

## 7. Entrada

- `input.rs` mapeia `crossterm::event::KeyEvent` para `Action`.
- Teclas do topo e numpad produzem o mesmo `Digit`.
- `,` e `.` produzem `DecimalSeparator`, independentemente do idioma.
- `+`, `-`, `*`, `/` e equivalentes do numpad produzem `Operator`.
- `Enter`, `=` e `KP_ENTER` produzem `Equals`; o app converte em recall se
  houver seleção ativa no histórico.
- `KeyEventKind::Release` é ignorado.
- Modificadores: `Ctrl+C` → `Quit`; `Ctrl+qualquer` não mapeado é ignorado.
- Teclas desconhecidas são ignoradas sem som e sem erro.
- Repetição automática é permitida para dígitos, `Backspace` e navegação;
  `=` tem debounce.
- A seleção de histórico muda o significado de `Enter` (ver docs/01, seção 8).

## 8. Renderização

### 8.1 Canvas e escala

- Canvas lógico fixo: **1344 × 756** (70% de 1920 × 1080).
- Fator de escala em fb:

```text
scale = min(screen_w / 1344, screen_h / 756) * 0.70
```

- Em 1920 × 1080, `scale = 1.0`: caminho rápido 1:1, sem reamostragem.
- Em outras resoluções, escala fracionária com filtro **bilinear**
  (nunca nearest, especialmente para texto).
- A cena é centralizada; o fundo preenche toda a tela.
- `scale ≈ 1` (epsilon 0.001) usa caminho rápido.

### 8.2 Cena lógica

`render/scene.rs` expõe primitivas independentes de backend:

```text
Scene {
  fill: Rgba8,
  items: Vec<Item>,
}
Item =
  Rect { x, y, w, h, radius, fill, stroke, shadow? }
  Text { x, y, w, text, font, size, color, align, weight }
  Line { x1, y1, x2, y2, width, color }
```

- A cena é construída a partir de `&App`, sem mutação.
- Todos os textos vêm do catálogo i18n ou de valores formatados no domínio.
- Nenhum log ou string de debug entra na cena.

### 8.3 Texto

- `fontdue` para rasterização; `fontdue::layout` para medição e alinhamento.
- Cache de glifos por `(font_id, char, px_size)`; nunca rasterizar por frame
  repetidamente.
- Medir antes de centralizar/alinchar à direita.
- Cobrir os caracteres: `0-9`, `,`, `.`, `+`, `−` (U+2212), `×` (U+00D7),
  `÷` (U+00F7), `±` (U+00B1), `%`, `=`, `e`, `−` do expoente, letras de AC/DEL,
  textos pt/en. Teste de cobertura obrigatório.
- Em resoluções diferentes de 1080p, a cena é rasterizada em 1344 × 756 e
  escalada; o texto passa por bilinear. Aceitável e documentado.

### 8.4 Composição

- `tiny-skia` trabalha com RGBA **pré-multiplicado**; respeitar no blend.
- Fundo (gradiente) é pintado no tamanho do alvo: canvas na captura e
  dispositivo no fb.
- A cena lógica é rasterizada uma vez em 1344 × 756 e composta com
  `draw_pixmap`: caminho 1:1 em 1080p, bilinear nas demais resoluções.
- Captura: canvas 1344 × 756 com fundo, cena e PNG RGBA8.
- Encoder PNG com parâmetros fixos; sem metadados variáveis.

### 8.5 Tema

- `theme.rs` define tokens semânticos: `Canvas`, `Panel`, `Card`, `Key`,
  `KeyPressed`, `Border`, `TextPrimary`, `TextSecondary`, `TextMuted`,
  `Accent`, `AccentStrong`, `Operator`, `Danger`, `Warning`, `Success`.
- Variantes: truecolor, 256, 16, mono.
- `--color auto` usa truecolor em fb 32 bpp; 256 em 16 bpp; mono em
  `--color never`/`NO_COLOR`.
- Cores exatas em `docs/03-visual-design.md`.
- Nenhum estado depende só de cor; ver acessibilidade.

## 9. Backend framebuffer

### 9.1 Abertura e validação

```text
1. open("/dev/fb0", O_RDWR | O_CLOEXEC)
2. ioctl FBIOGET_VSCREENINFO (0x4600)
3. ioctl FBIOGET_FSCREENINFO (0x4602)
4. validar xres, yres, bpp, line_length, yres_virtual
5. mmap(PROT_READ|PROT_WRITE, MAP_SHARED)
```

- `fb_var_screeninfo` = 160 bytes e `fb_fix_screeninfo` = 80 bytes em x86_64;
  `const` assert nos tamanhos.
- `line_length >= xres * bytes_per_pixel`; respeitar padding entre linhas.
- `yres_virtual` deve cobrir o buffer usado.
- Little-endian é premissa documentada.
- Formato desconhecido: erro acionável com bpp e offsets, nunca “chutar”.

### 9.2 Formatos

| bpp | Layout | Conversão |
| --- | --- | --- |
| 32 | RGBX/BGRX/XRGB/XBGR | canais de 8 bits; alpha 0 ou 8 |
| 24 | RGB/BGR | 3 bytes little-endian |
| 16 | RGB565 | R 5, G 6, B 5 |

```text
canal(valor8, offset, len) =
    len == 0 ? 0 : ((valor8 * ((1 << len) - 1) + 127) / 255) << offset
```

- Em 16 bpp, fidelidade de cor é menor; documentado.
- `--color 16` reduz a paleta antes de converter.

### 9.3 Apresentação

- Double buffer com `ypan` apenas se:
  `ypanstep > 0 && yres_virtual >= 2*yres && yres % ypanstep == 0`.
- Alternar buffers e chamar `FBIOPAN_DISPLAY` (0x4606) com `yoffset`
  correto.
- Sem `ypan`: escrever o frame completo direto no mmap; tearing é aceitável.
- Em `amdgpudrmfb`, o suporte a `ypan` deve ser detectado, não presumido.
- Pacing: 16 ms durante animação; ocioso apenas sob evento (poll 100 ms).
- Sem vsync garantido.

### 9.4 Console, guardas e ordem

```text
1. Abrir e validar /dev/fb0 e mapear memória
2. Abrir /dev/tty, KDGETMODE, KDSETMODE(KD_GRAPHICS)
3. ensure_tty(stdin) + enable_raw_mode via crossterm
4. esconder cursor e limpar a tela via /dev/tty
5. loop
6. desabilitar raw mode
7. restaurar KD_TEXT, mostrar cursor, limpar
8. munmap e fechar descritores
```

- Cada passo tem guarda RAII; falha desfaz os anteriores.
- `KDGETMODE` = 0x4B3B; `KDSETMODE` = 0x4B3A; `KD_GRAPHICS` = 1;
  `KD_TEXT` = 0.
- Fora de VT (X/Wayland/SSH), `KDSETMODE` falha com `ENOTTY`: recusar com
  código 3 e mensagem “execute em um VT local (Ctrl+Alt+F3)”.
- Sem permissão no device: código 3 e instrução para entrar no grupo `video`
  e relogar.
- `stdin` precisa ser TTY para input; `stdout` pode estar redirecionado.
- Nada de logs na tela; erros fatais são impressos em stderr **após** restaurar
  o console.

### 9.5 Sinais e pânico

- `SIGINT`, `SIGTERM`, `SIGHUP` com `sigaction` e flag atômica (`AtomicBool`),
  `SA_RESTART`.
- Handler apenas marca a flag; restauração ocorre no loop/`Drop`.
- `Ctrl+C` em raw mode chega como tecla; `input.rs` mapeia para `Quit`.
- Panic hook restaura console antes de imprimir o erro.
- `unsafe` apenas em `platform/fb.rs`, com `// SAFETY:`.

### 9.6 Limitações documentadas

- Troca de VT durante a execução pode deixar resíduos; sair e reiniciar.
- Sem vsync; tearing leve aceitável.
- Sem DRM/KMS; hardware sem `/dev/fb0` não é suportado.
- Resolução lida no início; mudança de modo exige reinício.
- `--backend auto`, TUI e SSH não existem na v1 (ADR D-001/D-002).

## 10. CLI e códigos de saída

```text
calculadora [OPTIONS]

  --lang <pt|en>                     padrão: pt
  --backend <fb>                     v1: somente fb (padrão)
  --fb-device <PATH>                 padrão: /dev/fb0
  --color <auto|truecolor|256|16|never>
  --no-animations
  --no-audio
  --capture-frame <PATH>
  -h, --help
  -V, --version
```

Regras:

- `--capture-frame` não abre fb, terminal interativo nem áudio; renderiza o
  estado inicial e sai.
- `NO_COLOR` tem prioridade sobre `--color`.
- Sem detecção automática de locale; `--lang pt` é o padrão.
- Sem `--seed` (não há RNG).
- `--backend tui|auto` é rejeitado pelo clap com código 2 e mensagem clara.
- CLI/diagnósticos em inglês (ferramenta); UI em pt-BR/en.

| Código | Significado |
| --- | --- |
| 0 | sucesso |
| 1 | erro de runtime/IO/captura |
| 2 | erro de CLI |
| 3 | backend indisponível (fb, permissão, VT, TTY) |

Mensagens de erro devem conter **causa** e **ação**.

## 11. Áudio

- Síntese em runtime: clique curto (ruído filtrado + envelope) e tom de erro.
- Sem arquivos de áudio, sem música.
- Volume padrão 0,32; `M` alterna mudo.
- ALSA carregado dinamicamente (`libasound.so.2` + `libloading`), atrás da
  feature `audio`.
- Ausência de biblioteca/dispositivo: silêncio e status “Som indisponível”.
- Thread separada com canal limitado; nunca bloquear o loop.
- Eventos: `KeyAccepted` (clique), `Error` (tom grave), `Toggle` (nada).
- Teste de síntese sem hardware: sem NaN, pico ≤ 1,0, duração esperada,
  envelope finito.

## 12. i18n e tema

- `Lang = PtBr | En`; padrão pt-BR.
- `F2` alterna; strings, erros, status e separadores numéricos mudam.
- Entradas de histórico são estruturadas e re-renderizadas no idioma atual.
- Rótulos de teclas são neutros (`AC`, `DEL`, `%`, `÷`, `×`, `−`, `+`, `±`,
  `=`).
- Catálogo e limites em `docs/08-i18n-catalog.md`.
- Tokens de tema e paletas em `docs/03-visual-design.md`.

## 13. Desempenho e orçamentos

| Métrica | Alvo | Status |
| --- | --- | --- |
| Startup até primeira tela | < 400 ms | a medir |
| Frame de animação (fb 1080p) | ≤ 16 ms | a medir |
| Frame ocioso | ≤ 33 ms; só sob evento | a medir |
| RSS | < 120 MB | a medir |
| CPU ocioso | < 2% | a medir |
| Binário release | < 15 MB | 1,08 MB (placeholder da Fase 1) |
| Assets embutidos | < 4 MB | 1,39 MB (medido) |
| Latência de tecla | < 50 ms | a medir |
| Captura 1344 × 756 | determinística e < 1 s | a medir |

Medir em release, no hardware alvo (1920 × 1080, 32 bpp). Registrar números e
desvios em `docs/10-validation-log.md`.

## 14. Segurança e robustez

- `unsafe` restrito a `platform/fb.rs`, com `// SAFETY:` e testes de ABI.
- Validar todo valor do driver antes de mapear/escrever.
- Não logar segredos; não armazenar credenciais.
- Não executar comandos externos em runtime.
- Não depender de rede.
- Restaurar terminal/console em sucesso, erro, pânico e sinais.
- Erros de backend são fatais e passam pela restauração.
- Ignorar teclas desconhecidas; limitar o buffer de entrada.
- Caminho de captura escolhido pelo usuário; reportar falhas com causa.

## 15. Testes e portões

Detalhes em `docs/05-test-plan.md`. Portões mínimos:

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

## 16. Empacotamento

1. Congelar escopo e versão.
2. Rodar portões com e sem features.
3. Regenerar capturas douradas e revisar.
4. Validar em VT real (32 bpp obrigatório; 24/16 se disponível).
5. `cargo build --release`.
6. Pacote: binário, `README.md`, `LICENSE`, `THIRD_PARTY_NOTICES.md`,
   licenças das fontes, notas da release.
7. `tar.gz` + `.sha256` (hash gerado por ferramenta de release, não em runtime).
8. Smoke test em `/tmp` sem assets.
9. Tag/release somente sob pedido explícito.

## 17. Desvios do framework

| Desvio | Motivo | ADR |
| --- | --- | --- |
| Sem TUI/`ratatui` na v1 | decisão do usuário: fb-only | D-001 |
| Sem `--backend auto`; default `fb` | não há fallback TUI | D-002 |
| Capturas douradas em PNG em vez de snapshots de TUI | não há TUI | D-008 |
| Sem RNG/`--seed` | produto não tem aleatoriedade | D-011 |
| Sem pipeline SVG→PNG de arte | visual é desenhado com primitivas + fontes | D-007 |
| xtask valida fontes/manifesto, não arte raster | não há arte raster própria | D-007 |
| Sem `render/image.rs`/protocolos de imagem | não há imagens no fb | D-001 |
