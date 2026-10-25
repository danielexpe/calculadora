# 08 — Catálogo de i18n

> Toda string visível vem deste catálogo. Nenhum texto de UI é hardcoded em
> `app.rs`, `render/` ou `core/`. pt-BR e en são obrigatórios na v1.
>
> **Status (2026-10-25):** catálogo implementado em `src/i18n/strings.rs`,
> com testes de completude, tradução e cobertura de glifos nas quatro fontes.

## 1. Idiomas

| Código | Nome | Padrão |
| --- | --- | --- |
| `pt` | Português (Brasil) | sim |
| `en` | English | não |

- `--lang <pt|en>` define o idioma inicial.
- `F2` alterna em qualquer estado, inclusive erro, resultado e histórico.
- A marca “Calculadora” não é traduzida.
- Rótulos de teclas são neutros: `AC`, `DEL`, `%`, `÷`, `×`, `−`, `+`, `±`,
  `=`. A tecla decimal mostra `,` em pt e `.` em en.
- CLI e diagnósticos são em inglês (ferramenta); a UI é localizada.

## 2. Catálogo de UI

| Chave | pt-BR | en | Onde | Limite |
| --- | --- | --- | --- | ---: |
| `app.name` | Calculadora | Calculadora | marca/docs | 11 |
| `history.title` | Histórico | History | título da fita | 12 |
| `history.empty` | Sem operações ainda | No operations yet | fita vazia | 20 |
| `footer.hints` | F2 Idioma · M Som · Q Sair · ↑↓ Histórico | F2 Language · M Sound · Q Quit · ↑↓ History | rodapé esquerdo | 48 |
| `footer.lang_pt` | PT | PT | chip de idioma | 2 |
| `footer.lang_en` | EN | EN | chip de idioma | 2 |
| `sound.on` | Som: ligado | Sound: on | rodapé direito | 20 |
| `sound.off` | Som: desligado | Sound: off | rodapé direito | 20 |
| `sound.unavailable` | Som: indisponível | Sound: unavailable | rodapé direito | 22 |
| `error.title` | Erro | Error | valor do visor | 5 |
| `error.div_zero` | Divisão por zero | Division by zero | linha de expressão | 20 |
| `error.overflow` | Número muito grande | Number too large | linha de expressão | 20 |
| `error.other` | Erro de cálculo | Calculation error | linha de expressão | 20 |
| `lang.name_pt` | Português (BR) | Portuguese (BR) | docs/futuro | 14 |
| `lang.name_en` | Inglês | English | docs/futuro | 7 |

Formatos montados em runtime (não são strings traduzíveis):

| Chave | Formato | Exemplo pt | Exemplo en |
| --- | --- | --- | --- |
| `history.format` | `{lhs} {op} {rhs} = {result}` | `200 + 10 = 210` | `200 + 10 = 210` |
| `history.format_percent` | `{lhs} {op} {rhs}% = {result}` | `200 + 10% = 220` | `200 + 10% = 220` |
| `number.fixed` | agrupamento + decimal local | `1.234,5` | `1,234.5` |
| `number.scientific` | `{mantissa}e{expoente}` | `1,234568e12` | `1.234568e12` |
| `number.minus` | U+2212 | `−42` | `−42` |

Regras dos formatos:

- `rhs` em `history.format_percent` é o operando **digitado** (ex.: `10`), não
  o efetivo (`20`). O cálculo usa o efetivo, mas a fita preserva a intenção.
- `%` é colado ao operando, sem espaço.
- O separador decimal do histórico segue o idioma atual, inclusive para
  entradas antigas (re-renderização).
- `=` e os símbolos de operação não são traduzidos.
- Não há pluralização na v1.

## 3. Números e separadores

| Regra | pt-BR | en |
| --- | --- | --- |
| Separador decimal | `,` | `.` |
| Separador de milhar | `.` | `,` |
| Entrada aceita | `,` e `.` | `.` e `,` |
| Sinal negativo | `−` (U+2212) | `−` (U+2212) |
| Marcador científico | `e` | `e` |
| Sinal do expoente | `−` quando negativo | `−` quando negativo |

Exemplos:

| Valor interno | pt-BR | en |
| --- | --- | --- |
| `0` | `0` | `0` |
| `1234.5` | `1.234,5` | `1,234.5` |
| `0.000000001` | `0,000000001` | `0.000000001` |
| `1234567890123` | `1,234568e12` | `1.234568e12` |
| `-42` | `−42` | `−42` |

- A troca de idioma não altera valores, histórico lógico, seleção nem operação
  pendente.
- O chip `PT`/`EN` no rodapé reflete o idioma atual.
- O separador da tecla decimal muda junto.

## 4. Mensagens de erro

| Estado | pt-BR | en |
| --- | --- | --- |
| Título | Erro | Error |
| Divisão por zero | Divisão por zero | Division by zero |
| Overflow | Número muito grande | Number too large |
| Outro | Erro de cálculo | Calculation error |

- O título ocupa o valor principal (56 px, `Danger`).
- A causa ocupa a linha de expressão (18 px, `Danger`/`TextSecondary`).
- “Erro”/“Error” nunca depende só de cor; há borda e texto.

## 5. Rodapé e status

- Esquerda: `footer.hints`.
- Direita: chip de idioma + status de som.
- O status de som inclui marcador de forma (preenchido/vazado/traço), além do
  texto.
- Se a string não couber, o rodapé trunca com `…`; título, erros e vazio
  **nunca** são truncados.

## 6. Diagnósticos de CLI (inglês)

Estas mensagens não são traduzidas na v1; são voltadas a quem opera o console.

| Situação | Mensagem |
| --- | --- |
| fb inexistente | `error: cannot open /dev/fb0: No such file or directory` |
| sem permissão | `error: cannot open /dev/fb0: Permission denied` |
| ação de permissão | `help: add your user to the 'video' group and log in again` |
| não é VT | `error: cannot set graphics mode: Inappropriate ioctl for device (ENOTTY)` |
| ação de VT | `help: run from a local VT (for example, Ctrl+Alt+F3)` |
| stdin não-TTY | `error: stdin is not a TTY` |
| formato inválido | `error: unsupported framebuffer format: 10 bpp (expected 32, 24 or 16)` |
| bpp/line_length | `error: invalid framebuffer geometry: ...` |
| captura | `error: cannot write capture to '<path>': <cause>` |
| CLI inválida | mensagem do clap, código 2 |

Toda mensagem deve incluir causa e ação quando a falha for recuperável pelo
usuário.

## 7. Cobertura de caracteres

O teste de fonte deve cobrir, no mínimo:

- Dígitos e pontuação: `0-9`, `,`, `.`, `+`, `−` (U+2212), `×` (U+00D7),
  `÷` (U+00F7), `±` (U+00B1), `%`, `=`, `e`, `·`, `:`, `↑`, `↓`, `…`.
- Letras: `A-Z`, `a-z`.
- Acentos do português usados no catálogo: `á`, `ã`, `ç`, `é`, `í`, `ó`, `õ`,
  `ú`.
- Se algum glifo faltar, a decisão é desenhar primitiva ou trocar por
  alternativa coberta; registrar em ADR.

## 8. Testes de i18n

- [ ] Toda chave existe em pt e en, sem string vazia.
- [ ] Nenhum texto de UI hardcoded fora do catálogo.
- [ ] `F2` alterna todas as strings, chips e separadores.
- [ ] Histórico antigo é re-renderizado com o novo separador.
- [ ] Valor e operação pendente não mudam ao trocar idioma.
- [ ] Strings nos limites de tamanho não estouram a área.
- [ ] Título/erros/vazio nunca são truncados.
- [ ] Rodapé trunca com `…` se necessário.
- [ ] Capturas pt/en aprovadas.
- [ ] Cobertura de glifos verde para todos os caracteres acima.
- [ ] Textos de erro cabem em 672 px a 18 px.

## 9. Limites de layout

| Área | Largura disponível | Fonte | Limite prático |
| --- | ---: | --- | ---: |
| Título da fita | 344 | Inter 22 | 16 caracteres |
| Entrada de histórico (resultado) | 344 | JBM 22 | 16 caracteres + sinal |
| Linha de expressão do visor | 672 | JBM 18 | ~50 caracteres |
| Valor do visor | 672 | JBM 64 | 16 caracteres |
| Rodapé esquerdo | ~500 | Inter 14 | 48 caracteres |
| Rodapé direito | ~180 | Inter 14 | 22 caracteres |
| Vazio da fita | 344 | Inter 17 | 20 caracteres |

Medir antes de liberar. Não confiar apenas na contagem de caracteres.
