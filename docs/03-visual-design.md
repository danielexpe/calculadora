# 03 — Design visual

> Estética: **escuro moderno com acento ciano**, superfícies em camadas,
> tipografia mono para números e sans para rótulos. O objetivo é parecer um
> instrumento de precisão, não um painel de terminal.

## 1. Canvas e composição

| Item | Valor |
| --- | --- |
| Canvas lógico | 1344 × 756 (16:9) |
| Escala no fb | `min(w/1344, h/756) × 0.70` |
| 1920 × 1080 | escala 1,0 (caminho rápido 1:1) |
| Centralização | conteúdo centralizado; fundo preenche a tela |
| Fundo | gradiente vertical sutil `Canvas` → `CanvasGlow`, pré-renderizado no tamanho do dispositivo |

O canvas é sempre a referência de layout. Nenhuma coordenada depende da
resolução do dispositivo.

## 2. Grid e geometria

```text
Canvas 1344 × 756
+---------------------------------------------------------------+
| 64                                                            |
|   +---------------------------+   +-------------------------+  |
|   | Histórico                 |   |  200 +                  |  |
|   |                           |   |              220        |  |
|   | 200 + 10 = 220            |   +-------------------------+  |
|   | 5 × 3 = 15                |   | AC   DEL   %    ÷       |  |
|   | ...                       |   | 7    8     9    ×       |  |
|   |                           |   | 4    5     6    −       |  |
|   |                           |   | 1    2     3    +       |  |
|   | Sem operações ainda       |   | ±    0     ,    =       |  |
|   |                           |   | F2 Idioma · M Som · Q   |  |
|   +---------------------------+   +-------------------------+  |
+---------------------------------------------------------------+
```

| Região | X | Y | Largura | Altura | Raio |
| --- | ---: | ---: | ---: | ---: | ---: |
| Margem do canvas | 64 | 48 | — | — | — |
| Fita de histórico | 64 | 48 | 400 | 660 | 24 |
| Gap entre painéis | 32 | — | — | — | — |
| Card de cálculo | 496 | 48 | 784 | 660 | 28 |
| Padding do card | 40 | 40 | — | — | — |
| Visor | 536 | 88 | 704 | 176 | 20 |
| Keypad | 536 | 296 | 704 | 338 | — |
| Rodapé | 536 | 656 | 704 | 20 | — |

### 2.1 Fita

| Elemento | X | Y | Largura | Altura |
| --- | ---: | ---: | ---: | ---: |
| Padding | 28 | 28 | — | — |
| Título | 92 | 80 | 344 | 32 |
| Divisor | 92 | 124 | 344 | 1 |
| Primeira entrada | 92 | 140 | 344 | 48 |
| Entradas visíveis | 10 | — | — | 480 |
| Dica de rolagem/vazio | 92 | 640 | 344 | 24 |

- Espaçamento entre entradas: 0 (cada uma tem 48 px com padding interno).
- Seleção: fundo `SurfaceSelected`, barra de 3 px no acento à esquerda e
  texto primário. A barra garante indicação sem depender de cor.
- Entrada: expressão (18 px, `TextSecondary`) e resultado (22 px,
  `TextPrimary`), ambos alinhados à direita.
- Título: “Histórico” / “History”, Inter SemiBold 22 px.
- Vazio: “Sem operações ainda” / “No operations yet”, Inter Regular 17 px,
  centralizado verticalmente na área de entradas, `TextMuted`.

### 2.2 Visor

- Fundo `Display`, raio 20, borda 1 px `Border`.
- Linha de expressão: alinhada à direita, JBM Regular 18 px, `TextSecondary`,
  recortada com reticências lógicas se exceder a largura (nunca quebra).
- Valor principal: alinhado à direita, JBM Medium 64 px, `TextPrimary`,
  altura de linha 76.
- Área do valor: x=552, y=128, w=672, h=88. O valor é medido e alinhado à
  direita em x=1224.
- Máximo de 16 caracteres para valores formatados; garantido pelo domínio.
- A entrada ativa pode chegar a 17 caracteres (15 dígitos + separador +
  sinal). Nesse caso, o agrupamento de milhar é omitido; a 64 px, 17
  caracteres de JetBrains Mono cabem nos 672 px úteis. Ver ADR D-017.
- Estado de erro: borda `Danger`; valor “Erro” em `Danger` 56 px; linha de
  expressão mostra a causa localizada.
- Estado de resultado: a linha de expressão mostra `lhs op rhs =` até a
  próxima entrada.
- Sem indicadores piscando.

### 2.3 Keypad

Grid 4 × 5, gap 12, chaves 167 × 58, raio 14.

| Linha | Coluna 1 | Coluna 2 | Coluna 3 | Coluna 4 |
| --- | --- | --- | --- | --- |
| 1 | `AC` | `DEL` | `%` | `÷` |
| 2 | `7` | `8` | `9` | `×` |
| 3 | `4` | `5` | `6` | `−` |
| 4 | `1` | `2` | `3` | `+` |
| 5 | `±` | `0` | `,` | `=` |

Coordenadas:

| Coluna | X | Largura |
| --- | ---: | ---: |
| 1 | 536 | 167 |
| 2 | 715 | 167 |
| 3 | 894 | 167 |
| 4 | 1073 | 167 |

| Linha | Y | Altura |
| --- | ---: | ---: |
| 1 | 296 | 58 |
| 2 | 366 | 58 |
| 3 | 436 | 58 |
| 4 | 506 | 58 |
| 5 | 576 | 58 |

- Rótulos de dígitos: JBM Medium 30 px, `TextPrimary`, centralizados.
- Operadores: JBM Medium 32 px, `Operator`, fundo com leve tinta de acento.
- `AC`: Inter SemiBold 20 px, `Danger`.
- `DEL`: Inter SemiBold 20 px, `TextSecondary`.
- `%`: Inter SemiBold 22 px, `Warning`.
- `=`: fundo `Accent`, texto `OnAccent`, JBM Medium 34 px.
- O separador decimal exibido na tecla é `,` em pt-BR e `.` em en; o valor
  interno não muda.

### 2.4 Rodapé do card

- Esquerda: dicas `F2 Idioma · M Som · Q Sair · ↑↓ Histórico`
  (en: `F2 Language · M Sound · Q Quit · ↑↓ History`), Inter Regular 14 px,
  `TextMuted`.
- Direita: chip de idioma (`PT`/`EN`) e status de som
  (`Som: ligado` / `Som: desligado` / `Som: indisponível`), Inter Regular
  14 px, `TextSecondary`.
- O status de som inclui um marcador de forma: círculo preenchido (ligado),
  círculo vazado (desligado), traço (indisponível). Não depende só de cor.
- `Q` em destaque sutil quando o app está ocioso, para descoberta da saída.

## 3. Paleta

### 3.1 Tokens semânticos (truecolor)

| Token | Hex | Uso |
| --- | --- | --- |
| `Canvas` | `#0B0D11` | fundo base |
| `CanvasGlow` | `#151A22` | topo do gradiente |
| `Panel` | `#12161E` | fita |
| `Card` | `#171C25` | card |
| `Display` | `#0F131A` | visor |
| `Key` | `#1F2430` | tecla numérica |
| `KeyUtility` | `#252B38` | AC, DEL, % |
| `KeyPressed` | `#2B3240` | tecla pressionada |
| `SurfaceSelected` | `#1F2632` | entrada selecionada |
| `Border` | `#2B323F` | bordas padrão |
| `BorderStrong` | `#3A4454` | foco/estado |
| `TextPrimary` | `#EDEFF3` | números e títulos |
| `TextSecondary` | `#A2ABBA` | rótulos e expressão |
| `TextMuted` | `#6F7888` | dicas e vazio |
| `Accent` | `#5EEAD4` | `=`, seleção, marca |
| `AccentStrong` | `#2DD4BF` | hover/realce lógico |
| `OnAccent` | `#07110F` | texto sobre acento |
| `Operator` | `#5EEAD4` | símbolos de operação |
| `Warning` | `#FBBF24` | `%` |
| `Danger` | `#F87171` | AC e erros |
| `Success` | `#34D399` | som ligado (com forma) |
| `Shadow` | `rgba(0,0,0,0.45)` | sombra do card |

Regras de uso:

- Acento é reservado para `=`, seleção e realces; não usar em todos os
  operadores ao mesmo tempo.
- Operadores usam `Operator` no símbolo e fundo `Key` com tinta de acento
  (`#1B2A2C`).
- `AC` usa `Danger`; `%` usa `Warning`; `DEL` usa `TextSecondary`.
- Nenhum texto sobre fundo claro sem contraste ≥ 4,5:1.

### 3.2 256 cores

Mapeamento inicial (ajustar após capturas):

| Token | xterm-256 |
| --- | --- |
| `Canvas` | 233 |
| `CanvasGlow` | 234 |
| `Panel` | 234 |
| `Card` | 235 |
| `Display` | 233 |
| `Key` | 236 |
| `KeyUtility` | 237 |
| `KeyPressed` | 238 |
| `Border` | 238 |
| `TextPrimary` | 255 |
| `TextSecondary` | 250 |
| `TextMuted` | 244 |
| `Accent` | 79 |
| `Operator` | 79 |
| `Warning` | 214 |
| `Danger` | 203 |
| `Success` | 78 |
| `OnAccent` | 232 |

### 3.3 16 cores

- Fundo: preto (`0`).
- Superfícies: preto/cinza (`0`, `8`).
- Texto: branco/cinza claro (`15`, `7`).
- Acento: ciano (`14`).
- `%`: amarelo (`11`).
- Erro/AC: vermelho (`9`).
- Som ligado: verde (`10`).
- Estados usam borda/relevo além da cor.

### 3.4 Monocromático (`NO_COLOR`, `--color never`)

- Fundo preto, texto branco, superfícies em três níveis de cinza derivados de
  `TextPrimary` com alpha.
- Tecla pressionada: borda interna de 2 px e deslocamento de 1 px, sem mudança
  de cor.
- Seleção de histórico: barra branca de 3 px + contorno.
- Erro: texto “Erro” em branco com borda dupla; a causa aparece na linha de
  expressão.
- Acento vira branco; `=` usa fundo branco e texto preto.

## 4. Tipografia

| Uso | Fonte | Peso | Tamanho | Linha |
| --- | --- | --- | ---: | ---: |
| Valor do visor | JetBrains Mono | Medium | 64 | 76 |
| Rótulo de dígito | JetBrains Mono | Medium | 30 | 36 |
| Símbolo de operador | JetBrains Mono | Medium | 32 | 38 |
| `=` | JetBrains Mono | Medium | 34 | 40 |
| Resultado do histórico | JetBrains Mono | Regular | 22 | 28 |
| Expressão do histórico/visor | JetBrains Mono | Regular | 18 | 24 |
| Título do histórico | Inter | SemiBold | 22 | 28 |
| Rótulos AC/DEL | Inter | SemiBold | 20 | 24 |
| `%` | Inter | SemiBold | 22 | 26 |
| Rodapé e status | Inter | Regular | 14 | 20 |

- Números usam fonte monoespaçada para alinhamento estável.
- Não usar glifos exóticos fora dos cobertos pelo teste de fonte.
- Texto não é escalado por nearest; a cena é escalada com bilinear.
- Textos nunca quebram linha no visor; no rodapé, truncar com reticências se
  necessário.

## 5. Superfícies e profundidade

- Card: `Card`, raio 28, borda 1 px `Border`, sombra suave em duas camadas
  (alpha 0,18 e 0,27) deslocada 10 px para baixo.
- Fita: `Panel`, raio 24, borda 1 px `Border`.
- Visor: `Display`, raio 20, borda 1 px `Border`, leve brilho interno no topo
  (`rgba(255,255,255,0.03)`).
- Teclas: `Key`, raio 14, borda 1 px `Border`, realce superior de 1 px
  (`rgba(255,255,255,0.05)`).
- Operadores: fundo `#1B2A2C`, borda `#24403F`, símbolo `Operator`.
- `=`: `Accent`, sem borda, sombra interna sutil.
- Nada de gradientes fortes ou ruído; o fundo tem gradiente discreto.

## 6. Estados

### 6.1 Tecla pressionada

- Feedback para tecla física mapeada e para clique (v1 não tem mouse; apenas
  teclado).
- Duração: 90 ms, ease-out cúbico.
- Efeito: fundo `KeyPressed`, deslocamento de 1 px para baixo, borda
  `AccentStrong`, sombra reduzida.
- Em mono: deslocamento + borda interna, sem cor.
- `--no-animations`: troca instantânea por 60 ms.

### 6.2 Histórico selecionado

- Fundo `SurfaceSelected`, barra de 3 px `Accent` à esquerda, texto
  `TextPrimary`.
- A seleção é visual e não altera o visor.
- Ao recuperar com `Enter`, o visor recebe o resultado e a seleção some.

### 6.3 Erro

- Borda do visor `Danger`, valor “Erro” em `Danger`.
- Linha de expressão com a causa (“Divisão por zero”, “Número muito grande”).
- Sem tremor, sem piscar; a cor é acompanhada de texto.
- Dígito ou AC limpam o estado.

### 6.4 Som

- Ligado: marcador preenchido + “Som: ligado”.
- Desligado: marcador vazado + “Som: desligado”.
- Indisponível: traço + “Som: indisponível”.
- `M` não altera o layout; apenas o status.

### 6.5 Idiomas

- Chip `PT`/`EN` no rodapé.
- `F2` alterna sem mudar a geometria.
- Separador decimal da tecla e do visor muda com o idioma.

## 7. Movimento

| Movimento | Duração | Curva | Desligado por |
| --- | ---: | --- | --- |
| Flash de tecla | 90 ms | ease-out cúbico | `--no-animations` |
| Fade/slide de nova entrada | 120 ms | ease-out | `--no-animations` |
| Troca de idioma | instantânea | — | — |
| Erro | instantâneo | — | — |

- Sem animação contínua; o app para de redesenhar quando ocioso.
- O fade de histórico move 8 px para cima e varia alpha de 0 → 1.
- Nada de movimento que possa causar desconforto vestibular.

## 8. Acessibilidade

- Contraste alvo: `TextPrimary` sobre `Card` ≥ 12:1; `TextSecondary` sobre
  `Card` ≥ 7:1; `TextMuted` ≥ 4,5:1; `Accent` sobre `Card` ≥ 10:1.
- `NO_COLOR` e `--color never` produzem paleta mono legível.
- Estado nunca depende só de cor: uso de borda, barra, marcador e texto.
- Rodapé sempre visível com os atalhos essenciais.
- `--no-animations` elimina fades e reduz o flash.
- Tamanho mínimo de texto: 14 px lógicos (≈ 14 px em 1080p).
- Nenhum texto essencial é cortado; strings longas são testadas nos dois
  idiomas.

## 9. Fontes e cobertura

- Inter SemiBold e Inter Regular.
- JetBrains Mono Regular e Medium.
- Caracteres obrigatórios: `0-9`, `,`, `.`, `+`, `−` (U+2212), `×` (U+00D7),
  `÷` (U+00F7), `±` (U+00B1), `%`, `=`, `e`, `A-Z`, `a-z`, `↑`, `↓`, `·`,
  `:` e acentos do português (`á`, `ã`, `ç`, `é`, `í`, `ó`, `õ`, `ú`).
- Teste automatizado falha se algum caractere do catálogo não tiver glifo.
- Se `↑`/`↓` não estiverem disponíveis na fonte escolhida, desenhar setas como
  primitivas ou usar `PgUp`/`PgDn` no rodapé (decisão registrada em ADR).

## 10. Capturas de referência

Toda captura tem 1344 × 756 e é determinística. Estados obrigatórios:

1. Inicial, pt-BR, truecolor, som ligado, histórico vazio.
2. Inicial, en, truecolor.
3. Operação pendente (`200 +`).
4. Resultado (`220`) com histórico.
5. Porcentagem (`200 + 10 %`).
6. Erro de divisão por zero.
7. Histórico longo com 10 entradas visíveis.
8. Entrada de histórico selecionada.
9. Tecla pressionada (`=`).
10. Paleta 256.
11. Paleta 16.
12. Monocromático (`--color never`).
13. Som desligado e indisponível.
14. Textos longos em pt-BR e en.

Comparação por bytes RGBA (tolerância zero na mesma toolchain). Atualização
com `UPDATE_SNAPSHOTS=1 cargo test` e revisão visual obrigatória.
