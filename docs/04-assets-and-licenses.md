# 04 — Assets e licenças

## 1. Princípios

- O binário deve funcionar **copiado sozinho**; nenhum asset é lido do disco
  em runtime.
- Fontes e strings entram via `include_bytes!`.
- A v1 não usa arte raster própria: card, teclas, visor e fita são desenhados
  com primitivas (`tiny-skia`).
- Não há download, telemetria ou consulta remota em runtime.
- Todo asset de terceiro precisa de licença compatível, crédito e texto de
  licença no pacote.
- Fontes não são modificadas nem subsetadas na v1 (evita cláusulas de nome
  reservado e simplifica a licença).

## 2. Inventário de assets

```text
assets/
  fonts/
    Inter-Regular.ttf
    Inter-SemiBold.ttf
    JetBrainsMono-Regular.ttf
    JetBrainsMono-Medium.ttf
    OFL-Inter.txt
    OFL-JetBrainsMono.txt
  generated/
    manifest.txt
```

| Asset | Versão | Origem | Licença | Uso | Tamanho |
| --- | --- | --- | --- | --- | ---: |
| Inter Regular/SemiBold | 4.1 | Projeto Inter (Rasmus Andersson) | OFL 1.1 | rótulos, títulos, rodapé | 831.384 B |
| JetBrains Mono Regular/Medium | 2.304 | JetBrains | OFL 1.1 | números, visor, histórico | 547.760 B |
| Textos OFL | — | distribuição das fontes | OFL 1.1 | licença | 8.779 B |
| Manifesto | v1 | gerado | — | integridade | < 1 KB |

Total baixado: **1.387.923 bytes** (~1,39 MB). Orçamento total de assets
embutidos: **< 4 MB** (meta: < 2,5 MB). Dentro do orçamento.

Links de referência:

- Inter: <https://github.com/rsms/inter>
- JetBrains Mono: <https://github.com/JetBrains/JetBrainsMono>
- SIL OFL 1.1: <https://openfontlicense.org/>

Os arquivos foram baixados em 2026-10-25 dos releases oficiais, sem
modificação, e versionados em `assets/fonts/`. Não baixar em build/runtime.

SHA-256 dos arquivos:

```text
40d692fce188e4471e2b3cba937be967878f631ad3ebbbdcd587687c7ebe0c82  Inter-Regular.ttf
78a843fade9d4612a5567302fb595b56976eb5fcebf4fea5a5912d638bafcde3  Inter-SemiBold.ttf
a0bf60ef0f83c5ed4d7a75d45838548b1f6873372dfac88f71804491898d138f  JetBrainsMono-Regular.ttf
31c92d01a8a08528b718a43addf0ad3df0af2ca4b7b3290a452f70f358e14d3d  JetBrainsMono-Medium.ttf
262481e844521b326f5ecd053e59b98c8b2da78c8ee1bdbb6e8174305e54935a  OFL-Inter.txt
30f0c136e3c88e422d0791acd97238870f9054a9729bc34cf2ff0d4ed8cac4ad  OFL-JetBrainsMono.txt
```

O manifesto versionado é `assets/generated/manifest.txt` e usa FNV-1a 64.

## 3. Pipeline `xtask`

### 3.1 Comandos

```bash
cargo xtask assets           # regenera o manifesto
cargo xtask assets --check   # falha se o manifesto divergir ou faltar fonte
```

- `xtask` é um crate do workspace, sem dependências externas.
- Hash: **FNV-1a 64 bits** implementado localmente. Não é criptográfico; serve
  para detectar drift de arquivo, não para segurança.
- O manifesto é texto determinístico, ordenado por caminho.
- `--check` compara hash e tamanho; também valida que cada fonte referenciada
  por `assets.rs` existe.
- O CI/portão de release roda `cargo xtask assets --check`.
- Nenhum PNG de arte é gerado na v1; capturas de QA vão para `tests/snapshots/`
  e não são assets de produção.

### 3.2 Formato do manifesto

```text
# calculadora assets manifest v1
# fnv1a64  size  path
3f2a...c1  312448  fonts/Inter-Regular.ttf
...
```

- Caminhos relativos a `assets/`.
- Ordem lexicográfica.
- Sem timestamp, sem usuário, sem caminho absoluto.

### 3.3 Testes do pipeline

- Manifesto atualizado ≠ manifesto versionado → `--check` falha.
- Fonte ausente → falha com caminho esperado.
- Fonte alterada → hash diverge.
- Ordem do manifesto é estável entre execuções.
- `cargo xtask assets` é idempotente.

## 4. Carregamento em runtime

- `assets.rs` expõe `Fonts` com quatro `fontdue::Font` construídos uma única
  vez (`OnceLock`/`LazyLock`).
- `include_bytes!` aponta para os TTF versionados.
- Decode de fonte acontece uma vez; falha é fatal com mensagem clara (build
  quebrado), não silêncio.
- O cache de glifos fica no renderizador de texto, não em `assets.rs`.
- Não há leitura de arquivo, `std::fs`, em `assets.rs`.

## 5. Licenças do projeto

| Elemento | Licença | Arquivo no pacote |
| --- | --- | --- |
| Código-fonte | MIT | `LICENSE` |
| Design/layout/arte original | CC BY 4.0 | `LICENSE-ART` + crédito |
| Fonte Inter | OFL 1.1 | `OFL-Inter.txt` |
| Fonte JetBrains Mono | OFL 1.1 | `OFL-JetBrainsMono.txt` |
| Dependências | conforme crates | `THIRD_PARTY_NOTICES.md` |

Crédito de arte sugerido:

> Interface e layout da Calculadora © 2026, licenciados sob CC BY 4.0.

O crédito deve viajar no pacote e no README da release.

## 6. Dependências e notices

`THIRD_PARTY_NOTICES.md` foi criado na Fase 1 a partir de
`cargo tree -e normal,build --format '{p}|{l}'` e contém todas as crates
ativas, inclusive transitivas e de build. Resumo das dependências diretas:

| Crate | Licença | Status |
| --- | --- | --- |
| `clap` | MIT OR Apache-2.0 | confirmada |
| `crossterm` | MIT | confirmada |
| `fontdue` | MIT OR Apache-2.0 OR Zlib | confirmada |
| `thiserror` | MIT OR Apache-2.0 | confirmada |
| `rust_decimal` | MIT | confirmada |
| `libloading` | ISC | confirmada |
| `libc` | MIT OR Apache-2.0 | confirmada (transitiva) |
| `tiny-skia` | a confirmar | entra na Fase 2 |
| `image` | a confirmar | entra na Fase 2 |

Regras:

- A tabela acima foi conferida com `cargo tree`/`cargo metadata`; o notices
  lista as 58 crates ativas com a feature padrão (57 sem `audio`).
- Crates com licença incompatível, desconhecida ou sem texto são bloqueio.
- Toda crate nova entra no notices e no orçamento de tamanho.
- Não embutir asset de terceiro sem licença e crédito.

## 7. Fontes: requisitos técnicos

- Formato TTF estático (não variável) para previsibilidade do `fontdue`.
- Verificar cobertura dos caracteres listados em `docs/03-visual-design.md`,
  seção 9.
- Se um caractere faltar, a decisão é desenhar primitiva ou trocar o símbolo
  por alternativa coberta; registrar em ADR.
- Não usar fontes do sistema; nenhum fallback silencioso.
- O texto é renderizado em 1344 × 756 e escalado bilinear em resoluções
  diferentes de 1080p.
- Versão/hash das fontes fica no manifesto e no `AGENTS.md` após o download.

## 8. Empacotamento de licenças

O pacote de release inclui:

```text
calculadora
README.md
LICENSE
LICENSE-ART
THIRD_PARTY_NOTICES.md
OFL-Inter.txt
OFL-JetBrainsMono.txt
RELEASE_NOTES.md
```

- O binário não incorpora os textos de licença, mas o pacote sim.
- `tar.gz` + `.sha256` verificados no diretório do pacote.
- Smoke test em `/tmp` sem `assets/`, com hash do binário conferido.

## 9. Critérios de aceite de assets

- [x] Quatro TTF presentes, com hash e tamanho no manifesto.
- [x] Textos OFL das duas famílias presentes e legíveis.
- [x] `cargo xtask assets --check` verde.
- [x] Todos os caracteres do catálogo têm glifo.
- [x] Binário copiado para `/tmp` funciona sem `assets/` (placeholder da Fase 1: `--version`/`--help` ok; runtime completo na Fase 2).
- [x] Assets embutidos < 4 MB (1,39 MB).
- [x] `THIRD_PARTY_NOTICES.md` confere com `cargo tree`/`cargo metadata`.
- [ ] Créditos e licenças no pacote de release.
