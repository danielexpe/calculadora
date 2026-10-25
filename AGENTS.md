# AGENTS.md — memória operacional

> Projeto: **Calculadora** — calculadora decimal para console Linux, renderizada
> em `/dev/fb0`. Este arquivo é a primeira leitura de qualquer agente.
> Última atualização: **2026-10-25**.

## 1. Retomada rápida

1. Ler este arquivo inteiro.
2. Ler `framework.md` (template obrigatório) e os docs relevantes em `docs/`.
3. Verificar `git status` — repositório local em `main`, commits `ed07e19`,
   `7923129`, `f691b0c` e `ab0dd65`. Não criar novo commit, push, tag ou release sem
   pedido explícito.
4. Conferir `docs/10-validation-log.md` antes de declarar qualquer validação.
5. Não iniciar fase sem pedido explícito; as Fases 0–3 estão concluídas no
   caminho headless (VT/hardware pendente).

## 2. Status atual

| Item | Status |
| --- | --- |
| Fase 0 — Descoberta | **concluída** (2026-10-25) |
| README + docs 01–10 | concluídos |
| AGENTS.md | concluído |
| Código de produção | Fases 1–4 implementadas; execução inicial em VT reportada OK |
| Fase 1 — Fundação | **concluída** (2026-10-25) |
| Fase 2 — Substrato fb | **concluída** (2026-10-25); VT inicial reportado OK |
| Testes | 194 ativos verdes (181 unitários + 9 processo + 1 golden + 1 PTY + 1 sinais + 1 xtask); 2 auxiliares ignorados |
| Captura PNG + douradas | **implementadas**: 1344 × 756; 16 snapshots; ~40 ms |
| Fase 3 — Fluxo funcional | **concluída** (2026-10-25); VT inicial reportado OK |
| Fase 4 — Acabamento | **concluída** (2026-10-25); detalhes de áudio/latência/24-16 pendentes |
| Fase 5 — Release | pendente |
| Validação em hardware | **inicial OK reportado** (2026-10-25); detalhes pendentes |
| Repositório git | inicializado em `main`; commits `ed07e19`, `7923129`, `f691b0c`, `ab0dd65` |
| Commits/releases | 4 commits locais (`ed07e19`, `7923129`, `f691b0c`, `ab0dd65`); nenhum push/release |

## 3. Resumo do produto

- Quatro operações, `±`, `%` contextual, `AC`, `DEL`.
- Decimal exato (`rust_decimal`), 8 casas internas, visor com 12 dígitos
  significativos e milhar.
- Tela única: card de cálculo + fita de histórico (50 entradas, 10 visíveis).
- pt-BR e en (`F2`), som opcional (`M`), teclado apenas.
- Backend único: fbdev 32/24/16 bpp; captura PNG determinística para QA.
- Alvo: `amdgpudrmfb`, 1920 × 1080, 32 bpp, VT local.

## 4. Decisões congeladas

Não reabrir sem pedido explícito e nova ADR em `docs/07`.

- **D-001:** v1 fb-only; sem TUI/`ratatui`; `crossterm` só para raw/input.
- **D-002:** `--backend fb` é o padrão; `auto`/`tui` rejeitados.
- **D-003:** `rust_decimal` 1.43.0; 8 casas, half-up.
- **D-004:** porcentagem contextual; `%` repetido no mesmo operando é ignorado.
- **D-005:** histórico apenas de sessão, sem disco.
- **D-006:** vírgula/ponto na entrada; exibição localizada.
- **D-007:** Inter 4.1 + JetBrains Mono 2.304 embutidas, OFL, sem subset.
- **D-008:** capturas PNG douradas no lugar de snapshots de TUI.
- **D-009:** somente teclado.
- **D-010:** tela única com fita, sem tela de ajuda.
- **D-011:** sem RNG e sem `--seed`.
- **D-012:** 1920 × 1080 32 bpp é o alvo oficial; 24/16 bpp pendentes de
  validação se não houver hardware.
- **D-013:** áudio opcional ALSA dinâmico; sem dispositivo = silêncio.
- **D-014:** 15 dígitos de entrada; 8 casas internas; valores formatados ≤ 16
  caracteres; entrada ativa ≤ 17 (D-017).
- **D-015:** avaliação sequencial; operador duplo substitui; `=` sem pendência
  é no-op; sem repetição da última operação; recall por `Enter`.
- **D-016:** `xtask` mínimo com FNV-1a; sem pipeline de arte raster.
- **D-017:** valores formatados ≤ 16 caracteres; entrada ativa pode ter 17
  (15 dígitos + separador + sinal), sem agrupamento quando necessário.

## 5. Fontes de verdade

| Documento | Conteúdo |
| --- | --- |
| `framework.md` | template de arquitetura e processo |
| `README.md` | visão do usuário, requisitos, teclas, licenças |
| `docs/01-calculadora.md` | comportamento, regras, aceite |
| `docs/02-technical-spec.md` | arquitetura, crates, fbdev, CLI |
| `docs/03-visual-design.md` | paleta, layout, tipografia, estados |
| `docs/04-assets-and-licenses.md` | fontes, pipeline, licenças |
| `docs/05-test-plan.md` | testes, capturas, matriz manual |
| `docs/06-roadmap.md` | fases e critérios de saída |
| `docs/07-decisions-and-risks.md` | ADRs, riscos, questões |
| `docs/08-i18n-catalog.md` | strings e limites de layout |
| `docs/09-release-checklist.md` | checklist de release |
| `docs/10-validation-log.md` | ambiente e validações reais |
| `THIRD_PARTY_NOTICES.md` | crates, licenças e créditos |

Em conflito: `docs/01` define comportamento; `docs/02` define arquitetura;
`docs/07` define decisões. Atualizar os três na mesma tarefa se algo mudar.

## 6. Pendências e questões abertas

| # | Pendência | Prazo | Dono |
| --- | --- | --- | --- |
| P-06 | Confirmar 24/16 bpp disponíveis ou marcar pendente | Fase 5 | usuário/agente |
| P-07 | Definir se haverá release público | Fase 5 | usuário |
| P-08 | Decidir se TUI volta no backlog | Fase 6 | usuário |

P-01/P-02 foram concluídas em 2026-10-25: o usuário entrou no grupo `video`,
abriu um VT local e reportou a execução inicial como OK. P-03/P-04/P-05 foram
concluídas na Fase 1 (`rust_decimal` 1.43.0, quatro TTF OFL e licenças
confirmadas). Na Fase 2, `tiny-skia` 0.12.0, `image` 0.25.10 (feature `png`),
`libc` 0.2.191 e `bytemuck` 1.25.2 foram aprovados e adicionados. Na Fase 4,
`libloading` foi usado para ALSA. Detalhes de 24/16 bpp, áudio audível,
latência e restauração por caminho continuam pendentes.

## 7. Comandos e portões

Portões atuais (Fase 4 verde):

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

QA sem hardware (Fases 2–3):

```bash
./target/release/calculadora --capture-frame /tmp/calculadora.png
# esperado: PNG 1344x756 determinístico, ~40 ms, sem abrir fb/tty/áudio
cargo test --test golden
# compara 16 estados por RGBA; regenerar: UPDATE_SNAPSHOTS=1 cargo test --test golden
cargo test --test pty_guard --test signals
# raw mode em PTY e flag de SIGTERM (processos filhos)
```

Validação em VT (execução inicial reportada OK em 2026-10-25; detalhes
pendentes):

```bash
ls -l /dev/fb0
cat /sys/class/graphics/fb0/name
id -nG
./user_run_now.sh
```

Ambiente auditado em 2026-10-25:

- `rustc`/`cargo` 1.99.1; edição 2021; MSRV alvo 1.88.
- `/dev/fb0` existe: `amdgpudrmfb`, 1920 × 1080, 32 bpp.
- `daniel` **não** está no grupo `video`.
- Sessão atual em Wayland no tty2; `/dev/tty` não é TTY no shell atual.
- `rust_decimal` 1.43.0 baixada e pinada no `Cargo.lock`.
- Fontes Inter 4.1 e JetBrains Mono 2.304 baixadas, com hashes registrados.
- Fases 2–4 implementadas: fbdev, console/sinais, tema, cena/texto, captura,
  `App` completo, flash/debounce/fade, histórico/recall, 16 capturas douradas,
  áudio ALSA dinâmico com fallback e contraste testado. Binário ~3,6 MB.
- Fluxo, capturas, áudio sintetizado e fade validados no headless; execução,
  áudio audível e latência em VT real **não** validados.

## 8. Regras de trabalho do agente

- Ler `AGENTS.md`, docs e `git status` antes de agir.
- Não reimplementar fases concluídas; não iniciar fase sem pedido.
- Não reabrir decisão congelada; propor ADR.
- Não adicionar dependência/rede sem aprovação explícita. Aprovadas:
  `rust_decimal`, as fontes, `libc`/`bytemuck`/`tiny-skia`/`image` (Fase 2) e
  `libloading` (áudio opcional da Fase 4, já previsto no `Cargo.toml`).
- Não fazer commit, push, amend, PR ou configurar remote sem pedido.
- Não criar commit, push, tag ou release sem pedido; o repositório local já
  existe.
- Manter `unsafe` só em `platform/fb.rs` e `platform/audio.rs`, com `// SAFETY:`
  (ADR D-019).
- Testar antes de declarar concluído; rodar todos os portões.
- Não inventar validação manual; registrar “pendente” e o que falta.
- Preservar terminal/console em sucesso, erro, pânico e sinais.
- Código/identificadores em inglês; UI em pt-BR/en.
- Sem comentários desnecessários; seguir o estilo existente.
- Não imprimir logs na tela do framebuffer.
- Atualizar docs e este arquivo ao final de tarefas relevantes.
- Diante de hardware/permissão indisponível, registrar bloqueio e seguir com o
  que for testável sem hardware.
- Preservar o escopo: nada de científica, mouse, persistência ou TUI na v1.

## 9. Protocolo de atualização deste arquivo

Ao final de cada tarefa relevante:

1. Atualizar a tabela de status (seção 2).
2. Atualizar pendências (seção 6) e remover as concluídas.
3. Acrescentar linha no registro de atividades (seção 10) com data, resumo,
   arquivos e commit (se houver).
4. Atualizar “Notas para a próxima sessão” (seção 12).
5. Se uma decisão mudou, criar/substituir ADR em `docs/07` e refletir em
   `docs/01`/`docs/02`.
6. Registrar hash de commits/releases quando existirem.

Não registrar segredos. Não inventar progresso.

## 10. Registro de atividades

| Data | Resumo | Arquivos | Commit |
| --- | --- | --- | --- |
| 2026-10-25 | Auditoria do ambiente: fb0/amdgpudrmfb 1080p 32bpp, usuário fora de `video`, sessão Wayland no tty2, Rust 1.99.1, cache de crates | — | n/a |
| 2026-10-25 | Fase 0: criados README, docs 01–10 e AGENTS.md; decisões e riscos congelados; nenhum código | `README.md`, `docs/*`, `AGENTS.md` | `ed07e19` |
| 2026-10-25 | Repositório git local inicializado em `main`; `.gitignore` criado | `.gitignore`, `AGENTS.md`, `docs/10-validation-log.md` | `ed07e19` |
| 2026-10-25 | Fase 1: domínio decimal/calculator/history, CLI, i18n, input, state, TTY, assets e xtask; 112 testes; fontes e `rust_decimal` baixados | `Cargo.toml`, `Cargo.lock`, `src/**`, `tests/**`, `xtask/**`, `assets/**`, `LICENSE`, `LICENSE-ART`, `THIRD_PARTY_NOTICES.md`, `docs/**`, `AGENTS.md` | `ed07e19` |
| 2026-10-25 | Fase 2: `platform/fb` (ABI 160/80, 32/24/16 bpp, mmap, console KD, sinais, panic), tema 4 paletas, cena/layout/texto/blend, captura PNG e app estático; 163 testes; captura 40 ms | `Cargo.toml`, `Cargo.lock`, `src/app.rs`, `src/capture.rs`, `src/platform/**`, `src/render/**`, `src/theme.rs`, `src/lib.rs`, `src/main.rs`, `src/cli.rs`, `tests/cli_process.rs`, `THIRD_PARTY_NOTICES.md`, `docs/**`, `AGENTS.md` | `f691b0c` |
| 2026-10-25 | README reescrito para uso/teste da versão inicial 0.1.0: captura headless, determinismo, portões, execução em VT, códigos de saída e limitações | `README.md`, `AGENTS.md` | `f691b0c` |
| 2026-10-25 | Fase 3: `App` (input→cálculo→cena, histórico/recall, flash, debounce), marcador de som, rolagem da fita, 16 capturas douradas, testes PTY/sinais e README atualizado; 182 testes ativos + 2 auxiliares | `src/app.rs`, `src/input.rs`, `src/capture.rs`, `src/render/scene.rs`, `src/render/layout.rs`, `tests/golden.rs`, `tests/pty_guard.rs`, `tests/signals.rs`, `tests/snapshots/**`, `README.md`, `docs/**`, `AGENTS.md` | `ab0dd65` |
| 2026-10-25 | Fase 4: áudio sintetizado + ALSA dinâmico (feature audio), fade de 120 ms, contraste/paletas corrigidos, compositor com fundo em cache, 16 douradas regeneradas e README/docs atualizados; 194 testes ativos + 2 auxiliares | `src/audio.rs`, `src/platform/audio.rs`, `src/app.rs`, `src/theme.rs`, `src/render/fb.rs`, `src/render/scene.rs`, `tests/cli_process.rs`, `tests/snapshots/**`, `Cargo.toml`, `README.md`, `docs/**`, `AGENTS.md` | pendente (sem commit) |
| 2026-10-25 | Script `user_run_now.sh` para build/preflight (fb, grupo, ALSA, captura) e execução no VT, com `--check`, `--no-build` e flags repassadas | `user_run_now.sh`, `README.md`, `AGENTS.md` | pendente (sem commit) |
| 2026-10-25 | Validação inicial em VT reportada OK pelo usuário; B-01/B-02 resolvidos, detalhes de áudio/latência/24-16 pendentes | `docs/06-roadmap.md`, `docs/10-validation-log.md`, `README.md`, `AGENTS.md` | pendente (sem commit) |

## 11. Fases

| Fase | Objetivo | Estado |
| --- | --- | --- |
| 0 | especificação e planejamento | concluída |
| 1 | fundação pura, CLI, i18n, input, testes | **concluída** |
| 2 | fbdev, console, texto, cena, captura | **concluída (headless)**; VT pendente |
| 3 | fluxo completo, histórico, capturas douradas | **concluída**; VT inicial OK |
| 4 | tema, som, a11y, desempenho | **concluída**; detalhes em VT pendentes |
| 5 | validação em VT, pacote, docs | pendente |
| 6 | backlog (TUI, DRM/KMS, persistência) | não iniciado |

## 12. Notas para a próxima sessão

- As Fases 1–4 estão implementadas e a execução inicial em VT foi reportada
  **OK** pelo usuário (2026-10-25). A Fase 5 (validação detalhada, pacote e
  release) só começa com pedido explícito.
- **Próximo passo (quando solicitado):** detalhar em VT áudio audível,
  latência, CPU ocioso, restauração por `Q`/`Esc`/`Ctrl+C`/`SIGTERM`/pânico,
  `--no-animations`, `--color 256/16/never` e 24/16 bpp.
- Áudio disponível depende de ALSA/PipeWire; sem dispositivo o status fica
  “Som indisponível” e o app segue.
- Capturas douradas: `tests/snapshots/` (16 PNGs) com `cargo test --test golden`;
  regenerar com `UPDATE_SNAPSHOTS=1 cargo test --test golden` e revisar.
- Em VT, validar 32 bpp primeiro; registrar 24/16 bpp como pendentes se não
  houver hardware. Nada de declarar validação sem executar.
- Qualquer mudança de comportamento deve atualizar `docs/01`, `docs/05` e as
  capturas correspondentes.
- Não publicar release, criar commit/push ou tag sem pedido.
