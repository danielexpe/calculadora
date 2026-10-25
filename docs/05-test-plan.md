# 05 — Plano de testes e qualidade

> Meta: provar o domínio sem terminal, provar a imagem sem `/dev/fb0` e validar
> o console em VT real. Nada é declarado validado sem execução registrada em
> `docs/10-validation-log.md`.

## 1. Estratégia

1. Domínio puro e determinístico, testado sem UI.
2. Invariantes e sequências adversariais sem `rand`.
3. Mapeamento de entrada e roteamento de ações.
4. Renderização e formatos de framebuffer sem hardware.
5. Capturas PNG douradas no lugar de snapshots de TUI (não há TUI).
6. Integração de PTY/guardas de terminal onde possível sem fb.
7. Matriz manual em VT real para fbdev, restauração e áudio.

## 2. Pirâmide adaptada

| Nível | O que cobre | Onde |
| --- | --- | --- |
| Unitário de domínio | cálculo, decimal, formatação, histórico | `src/core/**` |
| Invariantes/sequências | estados impossíveis, limites, fuzz determinístico | `src/core/**`, `tests/` |
| Entrada | `map_key`, numpad, modificadores, release | `src/input.rs` |
| Render sem hardware | geometria, blend, formatos, texto, captura | `src/render/**`, `src/platform/**`, `src/capture.rs` |
| Capturas douradas | todas as telas/estados/paletas | `tests/snapshots/` |
| Integração/PTY | raw mode, guardas, resize não aplicável | `tests/` |
| Manual em hardware | fb 32/24/16 bpp, VT, sinais, áudio | `docs/10-validation-log.md` |

## 3. Testes de domínio

### 3.1 Entrada e formatação decimal

- [ ] Dígitos do topo e numpad produzem o mesmo valor.
- [ ] `,` e `.` são aceitos como separador; apenas um por entrada.
- [ ] Limite de 15 dígitos no buffer; dígito excedente é ignorado.
- [ ] Zeros inteiros à esquerda normalizados (`0005` → `5`).
- [ ] `0,5`, `,5` (se aceito) e `0.5` produzem o mesmo decimal.
- [ ] Troca de idioma preserva o valor e muda o separador exibido.
- [ ] Formatação pt-BR: milhar `.`, decimal `,`.
- [ ] Formatação en: milhar `,`, decimal `.`.
- [ ] Até 12 dígitos significativos na notação fixa.
- [ ] Grupo de milhar correto em `1.234.567`.
- [ ] Fallback científico quando a representação fixa passa de 16 caracteres.
- [ ] Notação científica para `|x| ≥ 1e12` e `0 < |x| < 1e-9`.
- [ ] `0` e `−0` exibem `0`.
- [ ] Menos exibido como U+2212.
- [ ] Resultado nunca excede 16 caracteres.

### 3.2 Operações

Tabela mínima:

| Caso | Esperado |
| --- | --- |
| `2 + 3 =` | `5` |
| `2 − 5 =` | `−3` |
| `2 × 3 =` | `6` |
| `7 ÷ 2 =` | `3,5` |
| `0,1 + 0,2 =` | `0,3` |
| `−2 × −3 =` | `6` |
| `2 + 3 × 4 =` | `20` (sequencial) |
| `2 + 3 + 4 =` | `9`, com duas entradas de histórico |
| `5 + =` | `10` |
| `5 =` | `5` (sem pendência, no-op) |
| `2 + + 3 =` | `5` (operador substituído) |
| `2 + 3 = + 4 =` | `9` (resultado como acumulador) |
| `2 + 3 =` `7` | novo cálculo iniciado por `7` |
| `2 + 3 =` `Backspace` | visor inalterado |
| `2 + 3 =` `±` | `−5` |
| `2 + 3 =` `%` | `0,05` |
| `0 − 0 =` | `0` (não `−0`) |

- [ ] Arredondamento em 8 casas, metade para longe do zero.
- [ ] Overflow de `Decimal` vira `Overflow`, sem panic.
- [ ] Divisão por zero vira `DivisionByZero`, sem panic.
- [ ] Após erro, dígito inicia cálculo novo.
- [ ] Após erro, `AC` limpa; operadores/`=`/`%`/`±`/`Backspace` são ignorados.
- [ ] `F2`, `M`, `Q` funcionam em estado de erro.

### 3.3 Porcentagem

- [ ] Sem pendência: `10 %` → `0,1`.
- [ ] `+`: `200 + 10 % =` → `220`.
- [ ] `−`: `200 − 10 % =` → `180`.
- [ ] `×`: `200 × 10 % =` → `20`.
- [ ] `÷`: `200 ÷ 10 % =` → `2000`.
- [ ] `%` repetido no mesmo operando é ignorado e não entra em loop.
- [ ] Dígito após `%` inicia novo operando; `Backspace` não edita o convertido.
- [ ] `%` em modo resultado sem pendência divide por 100.
- [ ] `%` após erro é ignorado.
- [ ] Percentual com dízima é arredondado em 8 casas.

### 3.4 Histórico

- [ ] Operação resolvida por `=` cria uma entrada.
- [ ] Operação resolvida por operador encadeado cria uma entrada.
- [ ] Substituição de operador pendente **não** cria entrada.
- [ ] Erro **não** cria entrada.
- [ ] Limite de 50 entradas com descarte da mais antiga.
- [ ] Texto da entrada reflete `lhs op rhs = resultado`; com `%`, exibe
  `lhs op rhs% = resultado` e usa o operando efetivo no cálculo.
- [ ] Re-renderização ao trocar idioma mantém valores e muda separadores.
- [ ] `↑` a partir de nenhuma seleção seleciona a mais recente.
- [ ] `↑` navega para mais antigas; `↓` para mais novas.
- [ ] `↓` além da mais recente limpa a seleção.
- [ ] `Enter` com seleção recupera o resultado e limpa a pendência.
- [ ] Recall não cria histórico.
- [ ] Dígito/operador/AC/`Backspace`/`%`/`±` cancelam a seleção.
- [ ] Seleção não altera o visor.
- [ ] Fita vazia exibe mensagem localizada.

### 3.5 Sequências adversariais

- [x] Gerador LCG determinístico no teste produz 10.000 sequências de 1–200
  ações; nenhum panic, nenhum estado inválido.
- [x] Invariantes verificadas após cada ação:
  - `pending => accumulator`;
  - `error => pending.is_none()`;
  - buffer ≤ 15 dígitos e ≤ 1 separador;
  - valores formatados ≤ 16 caracteres; entrada ativa ≤ 17;
  - histórico ≤ 50;
  - toda entrada tem resultado finito/consistente.
- [x] A mesma sequência produz sempre o mesmo histórico e o mesmo visor.

## 4. Entrada e aplicação

- [x] `0`–`9` topo e numpad → `Digit`.
- [x] `,`/`.` topo e numpad → `DecimalSeparator`.
- [x] `+`/`-`/`*`/`/` e numpad → `Operator`.
- [x] `Enter`, `=`, `KP_ENTER` → `Equals` (recall é decidido no app).
- [x] `Backspace` → `Backspace`; `Delete`/`C` → `Clear`.
- [x] `S` → `ToggleSign`; `%` → `Percent`.
- [x] `↑`/`↓` → histórico.
- [x] `F2` → idioma; `M` → som.
- [x] `Q`/`Esc`/`Ctrl+C` → `Quit`.
- [x] `KeyEventKind::Release` ignorado.
- [x] Teclas desconhecidas ignoradas sem som/erro.
- [ ] Ações globais têm prioridade sobre a tela.
- [ ] Debounce de `=`/`Enter` em 120 ms; dígitos não têm debounce.
- [ ] Clique de tecla física dispara flash da tecla correspondente.

## 5. Renderização sem hardware

### 5.1 Framebuffer

- [ ] `fb_var_screeninfo` = 160 bytes; `fb_fix_screeninfo` = 80 bytes.
- [ ] Parse de 32 bpp RGBX/BGRX/XRGB/XBGR.
- [ ] Parse de 24 bpp RGB/BGR little-endian.
- [ ] Parse de 16 bpp RGB565.
- [ ] Formato desconhecido retorna erro com bpp/offsets.
- [ ] Conversão de canal 8→5 e 8→6 com arredondamento.
- [ ] Escrita de linha respeita `line_length` e padding.
- [ ] `xres/yres` inválidos são rejeitados antes do mmap.
- [ ] `line_length < xres * bpp/8` é rejeitado.
- [ ] `yres_virtual` insuficiente é rejeitado.
- [ ] Buffer completo tem exatamente `line_length * yres` bytes.
- [ ] `ypan` só é usado com `ypanstep > 0`, memória suficiente e alinhamento.
- [ ] Sem `ypan`, o frame é escrito por inteiro.

### 5.2 Alpha, escala e geometria

- [ ] Blend com alpha 0, opaco e parcial usando RGBA pré-multiplicado.
- [ ] Escala 1:1 não reamostra.
- [ ] Escala fracionária usa bilinear.
- [ ] Geometria em 1920 × 1080 → escala 1,0 e offsets (0,0).
- [ ] Geometria em 2560 × 1440 → escala 1,333… centralizada.
- [ ] Geometria em 1366 × 768 e 1024 × 768 → canvas cabe e centraliza.
- [ ] Canvas não fica vazio nem transborda.
- [ ] Fundo cobre toda a tela.
- [ ] Card e fita respeitam as coordenadas de `docs/03`.

### 5.3 Texto

- [ ] Cobertura de glifos para todos os caracteres do catálogo.
- [ ] Medição antes de alinhar à direita/centro.
- [ ] Cache de glifos evita rasterização repetida.
- [ ] Nenhum texto essencial quebra ou sai do card.
- [ ] Strings longas pt/en são medidas e truncadas apenas onde permitido.
- [ ] Display com 16 caracteres cabe em 672 px a 64 px.
- [ ] Fonte ausente/inválida falha com mensagem clara (teste de build).

### 5.4 Cores e tema

- [ ] Tokens truecolor, 256, 16 e mono resolvem para valores válidos.
- [ ] `NO_COLOR` força mono.
- [ ] `--color never` força mono.
- [ ] `--color auto` escolhe truecolor em 32 bpp e 256 em 16 bpp.
- [ ] Contraste mínimo verificado nos tokens principais.
- [ ] Estado de tecla pressionada tem relevo/borda além de cor.
- [ ] Seleção de histórico tem barra além de fundo.

## 6. Capturas douradas (PNG)

### 6.1 Estados obrigatórios

1. Inicial pt-BR truecolor, histórico vazio.
2. Inicial en truecolor.
3. Operação pendente `200 +`.
4. Resultado `220` com histórico.
5. Porcentagem `200 + 10 %`.
6. Erro de divisão por zero.
7. Histórico longo com 10 entradas.
8. Entrada selecionada.
9. Tecla `=` pressionada (flash).
10. Paleta 256.
11. Paleta 16.
12. Monocromático.
13. Som desligado.
14. Som indisponível.
15. Textos longos pt-BR.
16. Textos longos en.

Cada captura: 1344 × 756, RGBA8, sem metadados variáveis.

### 6.2 Comparação

- Comparar **bytes RGBA crus**, não apenas o arquivo PNG.
- Tolerância zero na mesma toolchain (versões de Rust, `fontdue`,
  `tiny-skia` e `image` fixadas no lock).
- Divergência exige inspeção visual e justificativa no commit/atividade.
- `UPDATE_SNAPSHOTS=1 cargo test` regenera; revisar diff antes de aceitar.
- Versão do app visível no rodapé? **Não** na v1 para evitar quebra de
  snapshots a cada release. A versão fica em `--version` e no pacote.

### 6.3 Determinismo da captura CLI

- [ ] `--capture-frame` gera 1344 × 756.
- [ ] Duas execuções produzem bytes idênticos.
- [ ] Captura não abre `/dev/fb0`, não entra em raw mode e não toca áudio.
- [ ] Captura com `--lang en`, `--color never`, `--no-audio` reflete as flags.
- [ ] Caminho inválido/sem permissão retorna código 1 e mensagem acionável.
- [ ] Captura termina em < 1 s.

## 7. Integração e PTY

- [ ] Guarda de raw mode restaura termios em `Drop` (teste com PTY).
- [ ] `ensure_tty` rejeita stdin não-TTY com código 3.
- [ ] `SIGTERM`/`SIGHUP` marcam flag e o loop sai restaurando.
- [ ] Panic hook restaura antes de imprimir.
- [ ] `Ctrl+C` em raw mode vira `Quit`, não sinal.
- [ ] Mudança de idioma/som não corrompe o estado do cálculo.
- [ ] Sem fb: mensagem de permissão e de VT são distintas.

## 8. Falhas e códigos de saída

| Cenário | Código | Verificação |
| --- | ---: | --- |
| Execução normal / captura ok | 0 | teste de integração |
| Captura sem permissão de escrita | 1 | mensagem com caminho |
| Flag inválida / `--backend tui` | 2 | clap |
| `/dev/fb0` inexistente | 3 | mensagem |
| Sem permissão no device | 3 | mencionar grupo `video` |
| `KDSETMODE` com `ENOTTY` | 3 | mencionar VT local |
| stdin não é TTY | 3 | mensagem |
| Falha de mmap/ioctl | 3 | causa e ação |

## 9. Matriz manual mínima

Ambiente auditado em 2026-10-25 (ver `docs/10-validation-log.md`):

- `amdgpudrmfb`, 1920 × 1080, 32 bpp.
- Usuário **não** está no grupo `video`.
- Sessão Wayland ativa no tty2; execução precisa de VT local.

Checklist:

- [ ] Adicionar usuário ao grupo `video` e relogar.
- [ ] Trocar para VT local (ex.: Ctrl+Alt+F3) e executar o binário.
- [ ] Validar 1920 × 1080 32 bpp.
- [ ] Validar restauração em `Q`, `Esc`, `Ctrl+C`.
- [ ] Validar restauração em `SIGTERM` e `SIGHUP`.
- [ ] Validar restauração em panic induzido em build de teste.
- [ ] Validar que cursor reaparece e `KD_TEXT` é restaurado.
- [ ] Validar 24 bpp e 16 bpp em VM/hardware, ou marcar **pendente**.
- [ ] Validar resolução alternativa (se disponível), ou marcar **pendente**.
- [ ] Trocar de VT durante a execução e registrar o comportamento.
- [ ] Fluxo completo: quatro operações, percentual, sinal, AC, DEL.
- [ ] Histórico: 10 entradas, rolagem, recall.
- [ ] pt-BR ↔ en em todos os estados.
- [ ] Som ligado/desligado; ausência de ALSA degrada em silêncio.
- [ ] `NO_COLOR`, `--color 256`, `--color 16`, `--color never`.
- [ ] `--no-animations`.
- [ ] Sem resíduos visuais, sem cursor piscando, sem logs na tela.

**Não marcar como validado sem executar.** O que não puder ser testado fica
explicitamente pendente.

## 10. Portões automatizados

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

Adicionais quando viável:

```bash
cargo test --release
cargo test -- --nocapture
```

## 11. Cobertura e meta

- Meta: **≥ 150 casos de teste** no total (tabelas contam casos).
- Fase 1: **112 casos** (106 unitários + 5 de processo/CLI + 1 do xtask).
- Domínio puro deve ter a maior densidade; renderização cobre formatos e
  estados.
- Nenhuma linha de `unsafe` sem teste de ABI/limites correspondente.
- Cobertura de linha é indicativa; o critério é comportamental.

## 12. Dados de teste

- Fixtures de cálculo em tabelas legíveis, com valores decimais explícitos.
- Sequências adversariais com LCG determinístico definido no teste.
- Estados visuais construídos por API, não por cliques reais.
- Nenhum teste depende de rede, relógio, locale do sistema ou `/dev/fb0`.
- Testes de captura usam fonte e tema fixos.
- Testes de áudio não exigem hardware: validam buffers sintetizados.

## 13. Status atual

| Área | Status |
| --- | --- |
| Documentos e decisões | concluído |
| Testes de domínio | **concluído** na Fase 1 (103 unitários) |
| Testes de entrada/i18n/CLI/state/assets | **concluído** na Fase 1 |
| Testes de processo/CLI | **concluído** na Fase 1 (5) |
| Manifesto/xtask | **concluído** na Fase 1 (1) |
| Testes de framebuffer | pendente (Fase 2) |
| Capturas douradas | pendente (Fase 2/3) |
| Integração/PTY | pendente (Fase 3) |
| Validação em hardware | **pendente** (grupo `video` e VT) |

Portões executados em 2026-10-25:

| Comando | Resultado |
| --- | --- |
| `cargo fmt --all -- --check` | verde |
| `cargo clippy --workspace --all-targets -- -D warnings` | verde |
| `cargo clippy --workspace --all-targets --no-default-features -- -D warnings` | verde |
| `cargo test --workspace` | 112 verdes |
| `cargo test --workspace --no-default-features` | 112 verdes |
| `cargo xtask assets --check` | verde |
| `cargo build --release` | verde |
| `cargo build --release --no-default-features` | verde |
