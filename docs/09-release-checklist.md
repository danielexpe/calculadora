# 09 — Checklist de release v1.0

> Não marcar itens sem execução real. Itens de hardware não validados ficam
> como **pendentes** com a causa registrada em `docs/10-validation-log.md`.
> O hash do pacote **não** entra nas próprias notas de release (referência
> circular); ele fica aqui e no `AGENTS.md`.

## Funcional

- [ ] Quatro operações, `±`, `%` contextual, `AC`, `DEL` conforme `docs/01`.
- [ ] Encadeamento, `=` e operador duplo conforme D-015.
- [ ] Decimal exato; `0,1 + 0,2 = 0,3`.
- [ ] Formatação pt/en com milhar, 12 dígitos e científico.
- [ ] Histórico de 50 entradas, 10 visíveis, seleção e recall.
- [ ] `F2` alterna idioma em todos os estados; histórico re-renderiza.
- [ ] `M` alterna som; status textual correto.
- [ ] Erros de divisão por zero e overflow com recuperação.
- [ ] `Q`, `Esc`, `Ctrl+C` saem e restauram.
- [ ] Captura determinística 1344 × 756.
- [ ] Binário funciona copiado, sem a pasta `assets/`.

## Qualidade

- [ ] `cargo fmt --all -- --check`.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] `cargo clippy --workspace --all-targets --no-default-features -- -D warnings`.
- [ ] `cargo test --workspace`.
- [ ] `cargo test --workspace --no-default-features`.
- [ ] `cargo xtask assets --check`.
- [ ] `cargo build --release`.
- [ ] `cargo build --release --no-default-features`.
- [ ] Capturas douradas revisadas e aprovadas.
- [ ] Fuzz determinístico sem panic.
- [ ] MSRV 1.88 verificada (se possível).
- [ ] `THIRD_PARTY_NOTICES.md` confere com `cargo metadata`.
- [ ] Nenhum `unsafe` fora de `platform/fb.rs` e `platform/audio.rs`.
- [ ] Nenhum log na tela.

## Manual em hardware

- [ ] Usuário no grupo `video` e sessão reiniciada.
- [ ] Execução em VT local (não Wayland/X11/SSH).
- [ ] 1920 × 1080, 32 bpp, `amdgpudrmfb`.
- [ ] 24 bpp validado **ou** marcado pendente.
- [ ] 16 bpp validado **ou** marcado pendente.
- [ ] Resolução alternativa validada **ou** marcada pendente.
- [ ] Restauração em `Q`.
- [ ] Restauração em `Esc`.
- [ ] Restauração em `Ctrl+C`.
- [ ] Restauração em `SIGTERM`.
- [ ] Restauração em `SIGHUP`.
- [ ] Restauração em pânico (build de teste).
- [ ] Cursor reaparece; `KD_TEXT` restaurado; tela limpa.
- [ ] Troca de VT durante a execução: comportamento registrado.
- [ ] Sem resíduos visuais, sem cursor piscando, sem texto de debug.
- [ ] Fluxo completo em pt e en.
- [ ] Histórico, seleção e recall em 10 entradas.
- [ ] `NO_COLOR`, `--color 256`, `--color 16`, `--color never`.
- [ ] `--no-animations` e `--no-audio`.
- [ ] Áudio real ou silêncio sem dispositivo.
- [ ] Latência perceptível < 50 ms.

## Desempenho

- [ ] Startup < 400 ms.
- [ ] Frame de animação ≤ 16 ms em 1080p.
- [ ] Frame ocioso ≤ 33 ms; CPU ocioso < 2%.
- [ ] RSS < 120 MB.
- [ ] Binário release < 15 MB.
- [ ] Assets embutidos < 4 MB.
- [ ] Captura < 1 s.
- [ ] Números registrados em `docs/10-validation-log.md`.

## Release

- [ ] Escopo congelado; versão em `Cargo.toml` e `Cargo.lock`.
- [ ] `README.md` atualizado e sem drift.
- [ ] `LICENSE` (MIT) e `LICENSE-ART` (CC BY 4.0) no pacote.
- [ ] Textos OFL das duas fontes no pacote.
- [ ] `THIRD_PARTY_NOTICES.md` no pacote.
- [ ] `RELEASE_NOTES.md` sem hash do próprio pacote.
- [ ] Pacote com binário, README, licenças, notices e créditos.
- [ ] `tar.gz` + `.sha256` gerados.
- [ ] `.sha256` verificado no diretório do pacote.
- [ ] Smoke test em `/tmp` sem `assets/`.
- [ ] Tag/release somente se solicitado.
- [ ] URL e hash registrados no `AGENTS.md`.

## Bloqueios conhecidos (2026-10-25)

- [ ] P-01: `daniel` fora do grupo `video`.
- [ ] P-02: sessão atual em Wayland; falta escolher VT local.
- [ ] P-03: `rust_decimal` ausente do cache (download aprovado).
- [ ] P-04: fontes OFL ainda não baixadas.
- [ ] P-06: 24/16 bpp não confirmados.

## Critério de release

Todos os itens acima marcados **ou** explicitamente registrados como pendentes
com justificativa. Nenhum item de hardware pode ser marcado sem execução no
hardware correspondente.
