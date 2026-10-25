# 06 — Roadmap

> O caminho padrão do framework (TUI → fb) foi adaptado: a v1 é **fb-only**.
> Por isso o substrato de framebuffer sobe para a Fase 2 e a validação de
> console é obrigatória antes do release. Nenhuma fase começa sem a anterior
> aceita.

## Visão geral

| Fase | Objetivo | Saída principal |
| --- | --- | --- |
| 0 — Descoberta | congelar escopo, regras e riscos | docs 01–08 + AGENTS.md |
| 1 — Fundação | domínio puro, CLI, i18n, entrada | cálculo testado, `--capture-frame` ainda não |
| 2 — Substrato fb | fbdev, console, texto, cena, captura | **concluída** (2026-10-25); VT pendente |
| 3 — Fluxo funcional | UI completa, input loop, histórico | **concluída (headless)**; VT pendente |
| 4 — Acabamento | tema, som, a11y, desempenho | **concluída (headless)**; VT pendente |
| 5 — Release v1.0 | validação em VT, pacote, docs | **pacote local concluído**; detalhes VT/público pendentes |
| 6 — Backlog | TUI, DRM/KMS, persistência | priorizado pelo usuário |

## Fase 0 — Descoberta (concluída)

**Escopo:** produto, regras de cálculo, decisões de arquitetura, riscos.

Entregáveis:

- `README.md`.
- `docs/01` a `docs/08`.
- `docs/09-release-checklist.md` e `docs/10-validation-log.md`.
- `AGENTS.md`.

Critérios de saída:

- [x] Escopo v1 e não-objetivos definidos.
- [x] Percentual, arredondamento, formatação e histórico congelados.
- [x] Backend fb-only e ausência de TUI registradas em ADR.
- [x] Dependências aprovadas (incluindo `rust_decimal`).
- [x] Fontes, licenças e alvo de hardware definidos.
- [x] Ambiente auditado e bloqueios registrados.
- [x] Nenhum código de produção iniciado (conforme pedido).

Pendências herdadas (atualizadas em 2026-10-25):

- [x] Adicionar o usuário ao grupo `video` e relogar.
- [x] Escolher um VT local livre para a validação.
- [ ] Confirmar 24/16 bpp ou marcar como pendente no release.
- [x] Validar a execução inicial em VT real (reportada OK).

## Fase 1 — Fundação (concluída em 2026-10-25)

**Escopo:** crate, domínio puro, formatação, i18n, CLI, input, erros.
Sem renderização de framebuffer. Evidência: 112 testes, manifesto de assets e
notices; ver `docs/10-validation-log.md`.

Tarefas:

1. `cargo init --name calculadora .` com workspace e `xtask`; criar
   `.cargo/config.toml` com o alias `xtask`.
2. `Cargo.toml` com features `audio` e dependências aprovadas.
3. `core/decimal.rs`: parse, escala 8, arredondamento, formatação pt/en.
4. `core/calculator.rs`: máquina de estados, operadores, percentual, erros.
5. `core/history.rs`: entradas, limite, seleção de dados.
6. `cli.rs`: flags, enums, validação, mensagens acionáveis.
7. `input.rs`: `Action`, `map_key` topo/numpad, `Ctrl+C`.
8. `i18n/`: catálogo pt/en.
9. `state/`: sessão de UI e roteamento de ações globais.
10. `util/tty.rs`: `ensure_tty` e guarda de raw mode (pode usar crossterm).
11. `assets.rs`: carregar fontes via `include_bytes!` (sem render ainda).
12. `THIRD_PARTY_NOTICES.md` e `LICENSE`/`LICENSE-ART`.
13. Testes de domínio, entrada e invariantes.
14. `xtask assets` e `--check` com manifesto.

Critérios de saída:

- [x] `cargo fmt --all -- --check` verde.
- [x] `cargo clippy --workspace --all-targets -- -D warnings` verde.
- [x] `cargo test` e `cargo test --no-default-features` verdes (112 cada).
- [x] Todas as tabelas de `docs/01` seções 6–8 cobertas por testes de domínio,
  formatação, histórico e navegação.
- [x] Fuzz determinístico sem panic (10.000 sequências de 1–200 ações).
- [x] `THIRD_PARTY_NOTICES.md` confere com `cargo tree`/`cargo metadata`.
- [x] Nenhum import de UI/`libc`/fb em `core/`.

## Fase 2 — Substrato framebuffer (concluída em 2026-10-25)

**Escopo:** abrir `/dev/fb0`, validar formato, mapear memória, renderizar cena
estática, texto e captura PNG. Ainda sem fluxo completo de cálculo na tela.

Tarefas:

1. `platform/fb.rs`: open/ioctl/mmap/validação, 32/24/16 bpp.
2. `platform/fb.rs`: console KD_GRAPHICS, guardas RAII, sinais, panic hook.
3. `render/text.rs`: fontdue, cache de glifos, medição, cobertura.
4. `render/scene.rs`, `render/layout.rs`, `render/fb.rs`.
5. `theme.rs`: tokens e paletas truecolor/256/16/mono.
6. `capture.rs`: PNG determinístico 1344 × 756.
7. Testes de ABI, formatos, padding, blend, escala e texto.
8. Primeira imagem estática da calculadora (headless/captura).

Critérios de saída:

- [x] Captura 1344 × 756 determinística e < 1 s (medida: ~40 ms).
- [x] Testes de 32/24/16 bpp e rejeição de formato desconhecido.
- [x] Blend pré-multiplicado correto (0, opaco, parcial).
- [x] Geometria correta em 1080p/1440p/768p/1024×768.
- [x] Todos os glifos do catálogo cobertos.
- [x] `--capture-frame` não abre fb, tty interativo nem áudio.
- [x] Erros de permissão com código 3 e mensagens acionáveis; erros de VT/TTY
  têm mensagens e código 3, testados por unidade.
- [x] Nenhum `unsafe` fora de `platform/fb.rs` na Fase 2 (D-019 estendeu
  para `platform/audio.rs` na Fase 4).
- [x] Em VT real: execução inicial em 32 bpp reportada **OK** pelo usuário
  (2026-10-25, `amdgpudrmfb` 1920 × 1080); detalhes de 24/16 bpp pendentes.

Evidência headless: captura determinística, 154 testes unitários + 8 de
processo + 1 do xtask; ver `docs/10-validation-log.md`.

## Fase 3 — Fluxo funcional (concluída em 2026-10-25; VT testado)

**Escopo:** ligar input → domínio → UI → fb; histórico e interação completos.

Tarefas:

1. Loop `App` com `update`/`draw`/`poll` e pacing.
2. Mapeamento de `CalcEvent` para som/animação.
3. Visor com expressão, valor, erro e modo resultado.
4. Keypad com flash de tecla física.
5. Fita com seleção, rolagem e recall.
6. `F2`, `M`, `Q`/`Esc`, debounce de `=`.
7. Capturas douradas de todos os estados de `docs/05`.
8. Comparador RGBA e `UPDATE_SNAPSHOTS=1`.
9. Integração/PTY de raw mode e sinais.
10. Medições iniciais de frame e latência.

Critérios de saída:

- [x] Fluxo completo em 32 bpp no VT: execução inicial reportada **OK** pelo
  usuário em 2026-10-25. Detalhes de latência e restauração por caminho ainda
  não foram registrados.
- [x] Todas as capturas de `docs/05` aprovadas: 16 PNGs em `tests/snapshots/`.
- [x] `Ctrl+C` mapeado para `Quit`; raw mode restaurado em PTY; `SIGTERM`
  marca a flag em teste de processo. Restauração visual de `KD_TEXT`/cursor
  em VT real segue **pendente**.
- [x] Histórico navegável e recall sem criar entrada.
- [x] Percentual e erros conforme `docs/01`.
- [x] `fmt`, `clippy`, `test` (com/sem features) verdes.
- [x] Nenhum log na tela; erros após restauração.

Evidência headless: 16 capturas douradas, testes de `App` (flash, debounce,
seleção/recall), PTY de raw mode e flag de `SIGTERM`; ver
`docs/10-validation-log.md`.

## Fase 4 — Acabamento (concluída em 2026-10-25; VT inicial testado)

**Escopo:** polimento visual, áudio, acessibilidade e desempenho.

Tarefas:

1. Refinar paleta, sombras e tipografia com capturas.
2. Implementar som (clique e erro) via ALSA dinâmico.
3. Implementar 256/16/mono de ponta a ponta.
4. `NO_COLOR`, `--no-animations`, `--no-audio`.
5. Otimizações: cache de glifos, fundo, card estático, caminho 1:1.
6. Testes de acessibilidade e contraste.
7. Medir orçamentos de `docs/02` em release.
8. Revisar strings longas nos dois idiomas.

Critérios de saída:

- [x] Áudio liga/desliga sem bloquear; sem ALSA/dispositivo = silêncio e
  status “Som indisponível”. **Ouvir clique/erro em VT real segue pendente.**
- [x] Capturas em truecolor/256/16/mono aprovadas (16 douradas).
- [x] Contraste documentado e testado; estados usam forma/borda além de cor.
- [x] Startup/captura headless ~40 ms; suíte de 16 quadros ~0,24 s em release
  (~15 ms/quadro, indicativo). **Frame em VT e ocioso ≤ 33 ms pendentes.**
- [x] RSS ~39 MB na captura; binário 3,6 MB; assets 1,4 MB.
- [ ] CPU ocioso < 2% e latência de tecla < 50 ms **pendentes** (requerem VT).
- [x] Números medidos registrados em `docs/10`.

Evidência headless: síntese testada sem hardware, backend ALSA com handshake e
fallback silencioso, fade de 120 ms, contraste ≥ 4,5:1/10:1 e paletas
douradas. Ver `docs/10-validation-log.md`.

## Fase 5 — Release v1.0 (pacote local e teste manual em VT concluídos em 2026-10-25)

**Escopo:** congelar, validar, empacotar e documentar.

Tarefas:

1. Congelar escopo e versão.
2. Rodar todos os portões com e sem features.
3. Regenerar/revisar capturas.
4. Validar manualmente 32 bpp obrigatório; 24/16 se disponível.
5. Testar restauração em todos os caminhos.
6. Criar `RELEASE_NOTES.md`, pacote e `.sha256`.
7. Smoke test em `/tmp` sem assets.
8. Tag/release **somente se solicitado**.

Critérios de saída:

- [x] `docs/09-release-checklist.md` com pendências explícitas.
- [x] Pacote com binário, README, licenças, notices, notas e créditos.
- [x] `.sha256` verificado.
- [x] Binário copiado sozinho funciona (smoke em `/tmp` sem `assets/`).
- [x] Nenhum terminal/console corrompido no teste manual completo em VT
  (`Q`/`Esc`/`Ctrl+C`); `SIGHUP`/pânico seguem pendentes.
- [x] `AGENTS.md` com hash do pacote, data e pendências.

Pacote: `dist/calculadora-1.0.0-linux-x86_64.tar.gz`, SHA-256
`daf707a5aeb67c13cbb7d5eaee20707d12e06e3a2b029d6e75ce73cfc2a15b84`.
Pendentes: 24/16 bpp, resoluções alternativas, `SIGHUP`/pânico, medições
formais de CPU/latência e MSRV 1.88. Repositório público criado; tag/GitHub
release não solicitados.

## Fase 6 — Backlog

Priorizar com o usuário; não iniciar sem pedido.

1. Backend TUI (`ratatui`) para SSH/desenvolvimento.
2. DRM/KMS.
3. Persistência opcional de histórico/preferências.
4. Mouse/toque.
5. Funções científicas e parênteses.
6. Repetição de `=`.
7. Remapeamento de teclas.
8. Temas claro e alto contraste.

## Dependências entre fases

```text
0 -> 1 -> 2 -> 3 -> 4 -> 5
                 \-> 6 (backlog)
```

- A Fase 2 pode produzir capturas estáticas antes do fluxo completo.
- A Fase 3 depende da captura determinística para snapshots.
- A Fase 4 depende do fluxo completo para medir interação.
- A Fase 5 só começa com portões verdes.
- Bloqueios de hardware não impedem o que for testável sem fb; ficam
  registrados e não são declarados validados.

## Marcos verificáveis

| Marco | Evidência |
| --- | --- |
| M1 — Domínio correto | testes de `docs/01` seções 6–8 verdes |
| M2 — Imagem determinística | captura 1344 × 756 idêntica |
| M3 — Fluxo completo | capturas de todos os estados |
| M4 — Console seguro | restauração em 5 caminhos |
| M5 — Release | pacote + sha256 + smoke `/tmp` |
