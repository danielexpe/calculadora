# 01 — Calculadora: especificação do produto

> **Fonte de verdade do comportamento.** Alterações aqui exigem ADR em
> `docs/07-decisions-and-risks.md` e atualização de testes/capturas.

## 1. Identidade

| Campo | Valor |
| --- | --- |
| Nome / marca | **Calculadora** |
| Subtítulo | Cálculo essencial, com elegância |
| Binário | `calculadora` |
| Gênero | calculadora decimal de console |
| Público | usuários de Linux que trabalham em console local e valorizam precisão e estética |
| Telas | tela única `Calculator` (card de cálculo + fita de histórico) |
| Plataforma | Linux x86_64 local, VT ativo, `/dev/fb0` |
| Licença do código | MIT |
| Licença da arte | CC BY 4.0 |
| Licença das fontes | OFL 1.1 (Inter, JetBrains Mono) |

## 2. Objetivo

Oferecer uma calculadora básica que seja **correta, bonita e previsível** no
console Linux, sem servidor gráfico. O diferencial não é ter funções
científicas: é ter aritmética decimal exata, porcentagem contextual, histórico
de sessão elegante e uma interface que pareça um produto, não um utilitário
improvisado.

## 3. Escopo da v1

### 3.1 Incluído

- Soma, subtração, multiplicação e divisão.
- Troca de sinal (`±`).
- Porcentagem contextual (`%`).
- Limpar tudo (`AC`) e apagar último dígito (`Backspace`).
- Entrada decimal com vírgula ou ponto; exibição conforme o idioma.
- Operações encadeadas.
- Fita de histórico da sessão, navegável e com recall do resultado.
- Idiomas pt-BR e en, com separadores numéricos localizados.
- Clique de tecla opcional, sintetizado em runtime.
- Modos de cor truecolor/256/16/mono e `NO_COLOR`.
- Captura PNG determinística para QA sem hardware.
- Backend framebuffer fbdev (32/24/16 bpp).

### 3.2 Não incluído (v1)

- Funções científicas, parênteses, potência, raiz, trigonometria, logaritmos.
- Memória (`M+`, `M-`, `MR`, `MC`).
- Repetição da última operação ao pressionar `=` novamente.
- Persistência de histórico, configurações ou tema.
- Mouse, gamepad ou toque.
- Edição de entradas anteriores na fita.
- Rede, contas, nuvem, telemetria.
- Windows, macOS, Wayland nativo, X11 nativo.
- DRM/KMS na v1; fbdev é o backend oficial.
- Backend TUI interativo (ver ADR D-001; fora de escopo, sem fase de backlog).

## 4. Tela e fluxo

### 4.1 Tela única

`Screen = Calculator`.

```text
+---------------------------------------------------------------+
|  +-------------------------+   +---------------------------+  |
|  | Histórico               |   |  200 + 10                 |  |
|  |                         |   |              220          |  |
|  | 200 + 10 = 220          |   +---------------------------+  |
|  | 5 × 3 = 15              |   | AC  DEL  %   ÷            |  |
|  |                         |   | 7   8    9   ×            |  |
|  |                         |   | 4   5    6   −            |  |
|  |                         |   | 1   2    3   +            |  |
|  |                         |   | ±   0    ,   =            |  |
|  | Sem operações ainda     |   | F2 Idioma · M Som · Q Sair|  |
|  +-------------------------+   +---------------------------+  |
+---------------------------------------------------------------+
```

- A fita ocupa a coluna esquerda; o card de cálculo, a direita.
- Não há tela de ajuda separada; o rodapé mostra os atalhos essenciais.
- Erros são um estado do visor, não uma tela.

### 4.2 Fluxo principal

1. App abre com visor `0`, fita vazia e foco no cálculo.
2. Usuário digita o primeiro operando.
3. Pressiona um operador; o visor mantém o operando e a linha de expressão
   mostra `operando op`.
4. Digita o segundo operando.
5. Pressiona `=`, `Enter` ou outro operador:
   - a operação é resolvida;
   - o resultado aparece no visor;
   - uma entrada é acrescentada à fita.
6. O resultado pode ser usado como base para a próxima operação.
7. `Q`/`Esc` encerra e restaura o console.

## 5. Modelo de entrada

### 5.1 Buffer de entrada

- Dígitos aceitos: `0`–`9`, do teclado principal e do numpad.
- Separador decimal: `,` ou `.` (ambos aceitos, independentemente do idioma).
- Um único separador por entrada.
- Máximo de **15 dígitos no buffer** por operando, sem contar sinal e
  separador. Dígitos além do limite são ignorados.
- Zeros inteiros à esquerda são normalizados: `0005` exibe `5`; `0,5` é válido.
- Trocar o idioma não altera o valor; apenas o separador exibido.

### 5.2 Teclas oficiais

| Entrada | `Action` |
| --- | --- |
| `0`–`9`, numpad | `Digit(n)` |
| `,` `.` | `DecimalSeparator` |
| `+` `-` `*` `/`, numpad | `Operator(Add/Sub/Mul/Div)` |
| `Enter`, `=` | `Equals` (recall se houver seleção) |
| `Backspace` | `Backspace` |
| `Delete`, `C` | `Clear` |
| `S` | `ToggleSign` |
| `%` | `Percent` |
| `↑` | `HistoryPrev` |
| `↓` | `HistoryNext` |
| `F2` | `ToggleLanguage` |
| `M` | `ToggleMute` |
| `Q`, `Esc`, `Ctrl+C` | `Quit` |

`KeyEventKind::Release` é ignorado. Teclas repetidas só são aceitas para
dígitos, `Backspace` e navegação do histórico; confirmações (`=`, `Enter`)
têm debounce de **120 ms** para evitar disparo duplo.

## 6. Regras de cálculo

### 6.1 Aritmética

- Tipo interno: `rust_decimal::Decimal`.
- Escala interna máxima: **8 casas decimais**.
- Após cada operação binária e cada conversão de porcentagem, o resultado é
  arredondado para 8 casas com `MidpointAwayFromZero` (metade para longe do
  zero).
- Zero negativo é normalizado para zero.
- Divisão por zero produz `Error::DivisionByZero`.
- Resultado fora do intervalo de `Decimal` produz `Error::Overflow`
  (“Número muito grande”).
- Não há `NaN`/`Infinity`: erros são estados explícitos.

### 6.2 Operações encadeadas

Estado conceitual:

```text
accumulator: Option<Decimal>
pending:     Option<Operator>
entry:       buffer de dígitos (String)
display:     valor visível
result_mode: bool
error:       Option<Error>
```

Regras:

- Primeiro operador: `accumulator = valor atual`, `pending = op`, inicia nova
  entrada.
- Operador com segundo operando já digitado: resolve a operação pendente,
  acrescenta entrada ao histórico, usa o resultado como novo acumulador e
  define o novo operador.
- Operador pressionado duas vezes seguidas (sem novo operando): substitui o
  operador pendente, sem calcular e sem criar histórico.
- `=` com operação pendente e segundo operando: resolve, registra no
  histórico, limpa `pending` e entra em modo resultado.
- `=` com operação pendente e sem segundo operando: usa o acumulador como
  segundo operando (`5 + =` → `10`) e registra no histórico.
- `=` sem operação pendente: não faz nada (v1 não repete a última operação).
- Operador em modo resultado: usa o resultado como acumulador, sem novo
  histórico.
- Dígito em modo resultado: inicia um novo cálculo; o visor é substituído.
- `Backspace` em modo resultado: ignorado.
- `±` em modo resultado: nega o resultado.
- `%` em modo resultado sem operação pendente: divide o resultado por 100.

### 6.3 Porcentagem contextual

Seja `acc` o acumulador, `x` o operando atual e `%` a tecla:

| Contexto | Operando efetivo | Exemplo |
| --- | --- | --- |
| Sem operação pendente | `x / 100` | `10 %` → `0,1` |
| Pendente `+` ou `−` | `acc × x / 100` | `200 + 10 %` → operando `20` |
| Pendente `×` ou `÷` | `x / 100` | `200 × 10 %` → operando `2` |
| Modo resultado, sem pendência | `resultado / 100` | `50 %` → `0,5` |

Após `%`, o visor mostra o operando efetivo. A conta só é fechada por `=`,
`Enter` ou pelo próximo operador:

- `200 + 10 % =` → `220`.
- `200 − 10 % =` → `180`.
- `200 × 10 % =` → `20`.
- `200 ÷ 10 % =` → `2000`.
- Pressionar `%` novamente no mesmo operando é ignorado: o operando já está
  convertido.
- Após `%`, o operando é considerado fechado: um dígito inicia um novo operando
  (descartando o convertido) e `Backspace` não edita o valor convertido. Uma
  nova operação também libera `%` para o próximo operando.

### 6.4 Erros

Estados:

| Estado | Visor principal | Linha de expressão |
| --- | --- | --- |
| `DivisionByZero` | `Erro` | `Divisão por zero` / `Division by zero` |
| `Overflow` | `Erro` | `Número muito grande` / `Number too large` |
| `Other` | `Erro` | `Erro de cálculo` / `Calculation error` |

- Erro não entra no histórico.
- Em estado de erro, dígitos e separador iniciam um novo cálculo limpo.
- `AC`/`Delete`/`C` limpam o erro.
- `Backspace`, operadores, `=`, `%` e `±` são ignorados.
- `F2`, `M`, `Q`/`Esc` continuam funcionando.

## 7. Formatação do visor

### 7.1 Regras

- Separadores por idioma:
  - pt-BR: decimal `,`, milhar `.`.
  - en: decimal `.`, milhar `,`.
- Até **12 dígitos significativos** na notação fixa.
- Parte inteira agrupada de 3 em 3.
- Zeros à direita da parte fracionária são removidos; inteiro não exibe
  separador decimal.
- Se a representação fixa passar de **16 caracteres**, usar notação científica
  com mantissa de até 6 casas e expoente inteiro (`1,234568e12`).
- Notação científica também é usada quando `|x| ≥ 1e12` ou
  `0 < |x| < 1e-9`.
- Sinal de menos exibido como `−` (U+2212); expoente negativo idem.
- `0` exibe `0`; `−0` exibe `0`.
- Valores formatados nunca mostram mais de 16 caracteres.
- A entrada ativa pode chegar a 17 caracteres (15 dígitos + separador +
  sinal); ela não usa agrupamento quando isso excederia o limite e ainda cabe
  nos 672 px do visor a 64 px. Ver ADR D-017.

### 7.2 Exemplos (pt-BR)

| Valor interno | Visor |
| --- | --- |
| `0` | `0` |
| `1234.5` | `1.234,5` |
| `0.000000001` | `0,000000001` |
| `1234567890123` | `1,234568e12` |
| `-42` | `−42` |

## 8. Histórico

- Escopo: **somente a sessão**; nada é gravado em disco.
- Capacidade: 50 entradas; 10 visíveis por vez.
- Uma entrada é criada sempre que uma operação binária é resolvida (por `=`,
  `Enter` ou por um operador que fecha a conta anterior).
- Estrutura da entrada: `lhs`, operador, `rhs` digitado, `rhs` efetivo,
  indicador de porcentagem e `resultado`.
- A fita exibe `lhs op rhs = resultado` ou `lhs op rhs% = resultado`, mantendo
  o operando digitado quando houve `%` (o cálculo usa o operando efetivo).
- A fita é re-renderizada no idioma atual, inclusive separadores.
- Navegação:
  - `↑` seleciona a entrada mais recente e depois as mais antigas.
  - `↓` avança para entradas mais novas; além da mais recente, limpa a
    seleção.
  - A seleção não altera o visor.
- `Enter` com seleção ativa recupera o resultado: visor recebe o valor,
  `pending` é limpo, modo resultado é ativado e a seleção é limpa. Recall não
  cria histórico.
- Qualquer dígito, separador, operador, `AC`, `Backspace`, `%` ou `±` cancela
  a seleção e executa sua ação normal.
- Fita vazia exibe a mensagem localizada de vazio.

## 9. Idioma, som e acessibilidade

### 9.1 Idioma

- `F2` alterna pt-BR ↔ en em qualquer estado, inclusive erro e resultado.
- Marca “Calculadora” não é traduzida.
- Rótulos de teclas não dependem de idioma (`AC`, `DEL`, `%`, `÷`, `×`, `−`,
  `+`, `±`, `=`), mas o rodapé, títulos, erros e status são traduzidos.
- Strings longas devem caber no layout; ver `docs/08-i18n-catalog.md`.

### 9.2 Som

- Clique sutil para entrada aceita; tom grave curto para erro.
- Volume padrão 0,32.
- `M` alterna mudo; `--no-audio` inicia mudo.
- Status no rodapé é textual: `Som: ligado`, `Som: desligado`,
  `Som: indisponível`.
- Ausência de ALSA ou de dispositivo resulta em silêncio, nunca erro fatal.

### 9.3 Acessibilidade

- Contraste mínimo: 7:1 para texto primário e 4,5:1 para secundário.
- `NO_COLOR` e `--color never` forçam paleta monocromática.
- Estado de tecla pressionada é indicado por relevo/borda, não só por cor.
- `--no-animations` remove fades e reduz o flash de tecla a troca instantânea.
- Nenhum estado depende exclusivamente de cor.
- O rodapé documenta os atalhos essenciais.

## 10. Requisitos funcionais

| ID | Requisito |
| --- | --- |
| RF-01 | Somar, subtrair, multiplicar e dividir decimais com exatidão de 8 casas. |
| RF-02 | Encadear operações sem perder precisão além do limite definido. |
| RF-03 | Aceitar vírgula e ponto como separador decimal. |
| RF-04 | Formatar números por idioma, com milhar e notação científica. |
| RF-05 | Implementar porcentagem contextual nos quatro contextos da seção 6.3. |
| RF-06 | Alternar sinal do operando ou resultado. |
| RF-07 | Limpar tudo e apagar último dígito com semânticas distintas. |
| RF-08 | Exibir erros de divisão por zero e overflow, com recuperação por dígito/AC. |
| RF-09 | Manter fita de histórico de até 50 entradas, com navegação e recall. |
| RF-10 | Alternar pt-BR/en com `F2`, incluindo separadores e histórico. |
| RF-11 | Ligar/desligar som com `M` e indicar estado textualmente. |
| RF-12 | Renderizar em fbdev 32/24/16 bpp e restaurar o console em qualquer saída. |
| RF-13 | Gerar captura PNG determinística 1344 × 756 sem abrir fb/tty/áudio. |
| RF-14 | Funcionar sem rede e sem arquivos externos ao binário. |
| RF-15 | Respeitar `NO_COLOR`, `--color never`, `--no-animations` e `--no-audio`. |

## 11. Critérios de aceite

| ID | Critério verificável |
| --- | --- |
| CA-01 | `200 + 10 % =` exibe `220` em pt-BR e `220` em en. |
| CA-02 | `0,1 + 0,2 =` exibe `0,3` (sem resíduo binário). |
| CA-03 | `1 ÷ 0 =` exibe `Erro` + `Divisão por zero`; digitar `7` inicia novo cálculo. |
| CA-04 | `2 + 3 × 4 =` respeita avaliação sequencial de calculadora: `20`. |
| CA-05 | Fita registra cada operação resolvida e preserva os valores ao alternar idioma. |
| CA-06 | `↑` + `Enter` recupera o resultado selecionado sem criar nova entrada. |
| CA-07 | `--capture-frame` gera PNG 1344 × 756 idêntico em execuções repetidas. |
| CA-08 | `Q`, `Esc`, `Ctrl+C`, `SIGTERM` e pânico restauram `KD_TEXT` e cursor. |
| CA-09 | Sem `/dev/fb0` ou sem permissão, saída é código `3` com causa e ação. |
| CA-10 | `--no-default-features` compila e passa testes sem áudio. |
| CA-11 | 32 bpp validado no hardware alvo; 24/16 bpp validados ou marcados pendentes. |
| CA-12 | Portões `fmt`, `clippy`, `test`, `xtask assets --check` e builds release verdes. |
| CA-13 | Nenhuma string traduzida estoura o layout em pt-BR ou en. |
| CA-14 | Nenhum log é impresso no meio do framebuffer. |

## 12. Escopo encerrado

A v1.0.0 encerra o escopo: não há Fase 6 nem backlog planejado (D-020). Os
itens abaixo permanecem fora do produto:

1. Backend TUI (`ratatui`) para SSH/desenvolvimento.
2. DRM/KMS.
3. Persistência de histórico e preferências.
4. Mouse/toque nas teclas.
5. Funções científicas e parênteses.
6. Repetição da última operação com `=`.
7. Remapeamento de teclas.
8. Temas claro e alto contraste completos.
