# 09 — Checklist de release v1.0.2

> Não marcar itens sem execução real. Itens de hardware não validados ficam
> como **pendentes** com a causa registrada em `docs/10-validation-log.md`.
> O hash do pacote **não** entra nas próprias notas de release (referência
> circular); ele fica aqui e no `AGENTS.md`.

Repositório público: https://github.com/danielexpe/calculadora

Release público: https://github.com/danielexpe/calculadora/releases/tag/v1.0.2

Pacote local: `dist/calculadora-1.0.2-linux-x86_64.tar.gz`
Hash SHA-256: `59ecbc55be2543c20c4a9fe6bd9fb4f9811fa385e3a40f7021e9bf436400cbc6`

## Funcional

- [x] Quatro operações, `±`, `%` contextual, `AC`, `DEL` conforme `docs/01`.
- [x] Encadeamento, `=` e operador duplo conforme D-015.
- [x] Decimal exato; `0,1 + 0,2 = 0,3`.
- [x] Formatação pt/en com milhar, 12 dígitos e científico.
- [x] Histórico de 50 entradas, 10 visíveis, seleção e recall.
- [x] `F2` alterna idioma em todos os estados; histórico re-renderiza.
- [x] `M` alterna som; status textual correto (áudio real: ver manual).
- [x] Erros de divisão por zero e overflow com recuperação.
- [x] `Q`, `Esc`, `Ctrl+C` mapeados para saída (restauração manual: ver abaixo).
- [x] Captura determinística 1344 × 756.
- [x] Binário funciona copiado, sem a pasta `assets/`.

## Qualidade

- [x] `cargo fmt --all -- --check`.
- [x] `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] `cargo clippy --workspace --all-targets --no-default-features -- -D warnings`.
- [x] `cargo test --workspace` (196 ativos verdes; 2 auxiliares ignorados).
- [x] `cargo test --workspace --no-default-features`.
- [x] `cargo xtask assets --check`.
- [x] `cargo build --release`.
- [x] `cargo build --release --no-default-features`.
- [x] Capturas douradas revisadas e aprovadas (16 PNGs).
- [x] Fuzz determinístico sem panic (10.000 sequências).
- [ ] MSRV 1.88 verificada — **pendente**: só o toolchain 1.99.1 está instalado.
- [x] `THIRD_PARTY_NOTICES.md` confere com `cargo tree`.
- [x] Nenhum `unsafe` fora de `platform/fb.rs` e `platform/audio.rs`.
- [x] Nenhum log na tela.

## Manual em hardware

Teste manual completo em VT reportado **OK** pelo usuário em 2026-10-25
(`amdgpudrmfb`, 1920 × 1080, 32 bpp). Itens não citados nesse teste permanecem
explicitamente pendentes abaixo.

- [x] Usuário no grupo `video` e sessão reiniciada.
- [x] Execução em VT local (não Wayland/X11/SSH).
- [x] 1920 × 1080, 32 bpp, `amdgpudrmfb`.
- [ ] 24 bpp validado **ou** marcado pendente — **pendente** (sem hardware/VM).
- [ ] 16 bpp validado **ou** marcado pendente — **pendente** (sem hardware/VM).
- [ ] Resolução alternativa validada **ou** marcada pendente — **pendente**.
- [x] Restauração em `Q`.
- [x] Restauração em `Esc`.
- [x] Restauração em `Ctrl+C`.
- [ ] Restauração em `SIGTERM`.
- [ ] Restauração em `SIGHUP`.
- [ ] Restauração em pânico (build de teste).
- [x] Cursor reaparece; `KD_TEXT` restaurado; tela limpa.
- [ ] Troca de VT durante a execução: comportamento registrado.
- [x] Sem resíduos visuais, sem cursor piscando, sem texto de debug.
- [x] Fluxo completo em pt e en.
- [x] Histórico, seleção e recall em 10 entradas.
- [x] `NO_COLOR`, `--color 256`, `--color 16`, `--color never`.
- [x] `--no-animations` e `--no-audio`.
- [x] Áudio real ou silêncio sem dispositivo.
- [x] Latência perceptível < 50 ms.

## Desempenho

- [x] Startup < 400 ms — ~40 ms no caminho de captura headless.
- [x] Frame de animação ≤ 16 ms em 1080p — ~15 ms/quadro indicativo na suíte
  dourada release; medir no VT.
- [ ] Frame ocioso ≤ 33 ms; CPU ocioso < 2% — **pendente em VT**.
- [x] RSS < 120 MB — ~39 MB na captura release.
- [x] Binário release < 15 MB — 3,6 MB.
- [x] Assets embutidos < 4 MB — ~1,4 MB.
- [x] Captura < 1 s — ~40 ms.
- [x] Números registrados em `docs/10-validation-log.md`.

## Release

- [x] Escopo congelado; versão 1.0.2 em `Cargo.toml` e `Cargo.lock`.
- [x] `README.md` atualizado e sem drift.
- [x] `LICENSE` (MIT) e `LICENSE-ART` (CC BY 4.0) no pacote.
- [x] Textos OFL das duas fontes no pacote (`licenses/`).
- [x] `THIRD_PARTY_NOTICES.md` no pacote.
- [x] `RELEASE_NOTES.md` sem hash do próprio pacote.
- [x] Pacote com binário, `install.sh`, README, licenças, notices e créditos.
- [x] `tar.gz` + `.sha256` gerados.
- [x] `.sha256` verificado no diretório do pacote.
- [x] Smoke test em `/tmp` sem `assets/`.
- [x] `install.sh` testado em `/tmp` com prefixo temporário (`--skip-fb`),
  incluindo `--check`, instalação, captura e `--uninstall`.
- [ ] Fix de `yoffset`/double buffer confirmado no segundo computador —
  **pendente**. Registro em `docs/10-validation-log.md` §3.6.
- [ ] `FBIOBLANK`/`FB_BLANK_UNBLANK` confirmado no segundo computador: primeira
  tela sem `Ctrl+C` e `blank` passando a `0` — **pendente**. Registro em
  `docs/10-validation-log.md` §3.8.
- [x] Tag `v1.0.2` e GitHub release público com `tar.gz` + `.sha256`.
- [x] Hash e caminho registrados no `AGENTS.md`.

## Bloqueios conhecidos (2026-10-25)

- [x] P-01: resolvido; usuário entrou no grupo `video`.
- [x] P-02: resolvido; execução em VT local reportada OK.
- [x] P-03: resolvido; `rust_decimal` 1.43.0 pinada/testada.
- [x] P-04: resolvido; fontes OFL baixadas e verificadas.
- [ ] P-06: 24/16 bpp não confirmados — **pendente**.
- [x] P-07: repositório público, tag `v1.0.2` e GitHub release publicados.
- [ ] MSRV 1.88 não instalada no ambiente.

## Critério de release

O pacote v1.0.2 está gerado, verificado e testado isoladamente em `/tmp`;
o instalador foi exercitado com prefixo temporário; tag e GitHub release
públicos foram publicados com os dois artefatos. 24/16 bpp, resoluções alternativas,
`SIGHUP`/pânico, medições formais de CPU/latência e MSRV 1.88 permanecem
explicitamente pendentes.
