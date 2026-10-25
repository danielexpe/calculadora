# 07 — Decisões, premissas e riscos

> Decisões marcadas como **aceita** estão congeladas. Reabrir exige pedido
> explícito do usuário e uma nova ADR que substitua a anterior.

## 1. ADRs

### D-001 — Backend framebuffer único na v1

- **Status:** aceita.
- **Contexto:** o usuário escolheu “somente framebuffer”. O framework recomenda
  TUI primeiro e fb depois; manter os dois dobraria o custo e adiaria a
  validação no console.
- **Decisão:** a v1 tem apenas backend fbdev. Não há `ratatui`, `ratatui-image`
  nem fallback TUI. `crossterm` permanece apenas para raw mode/input.
- **Consequências:** não funciona em SSH/X11/Wayland; exige VT local e
  `/dev/fb0`; QA sem hardware usa `--capture-frame`; TUI volta no backlog
  (Fase 6) se o usuário quiser.

### D-002 — `fb` é o backend padrão e não há `auto`

- **Status:** aceita.
- **Contexto:** o framework exige que `auto` nunca escolha fbdev, mas a v1 não
  tem TUI. Um `auto` que sempre escolhe fb seria enganoso.
- **Decisão:** `--backend <fb>` é o único valor aceito; o padrão é `fb`.
  `tui`/`auto` falham no clap com código 2. Falha de fb produz código 3 com
  causa e ação.
- **Consequências:** desvio explícito do template; documentado em README e
  testes. Quando houver TUI no backlog, `auto` pode ser reintroduzido.

### D-003 — Aritmética decimal via `rust_decimal`

- **Status:** aceita e implementada na Fase 1.
- **Contexto:** `f64` produz resíduos (`0,1 + 0,2`) e prejudica a promessa de
  precisão. Implementar fixed-point próprio custaria testes e manutenção.
- **Decisão:** usar `rust_decimal`, escala interna máxima de 8 casas,
  arredondamento `MidpointAwayFromZero`.
- **Consequências:** dependência MIT na versão **1.43.0**, com
  `default-features = false` e feature `std`, pinada no `Cargo.lock` e nos
  notices. Operações além de 8 casas são arredondadas (documentado).

### D-004 — Porcentagem contextual

- **Status:** aceita.
- **Contexto:** “10%” é ambíguo: `0,10` literal ou fração do acumulador.
- **Decisão:** em `+`/`−`, o percentual é aplicado sobre o acumulador; em
  `×`/`÷`, vira fração; sem pendência, divide por 100. O visor mostra o
  operando convertido e `=`/operador fecha a conta. `%` repetido no mesmo
  operando é ignorado. A fita preserva o operando digitado com o marcador `%`
  (ex.: `200 + 10% = 220`) e usa o efetivo no cálculo.
- **Consequências:** comportamento de calculadora de bolso; precisa de
  exemplos no README e testes dos quatro contextos. Quem quiser `0,10`
  literal digita `0,1`.

### D-005 — Histórico somente de sessão

- **Status:** aceita.
- **Contexto:** persistência adiciona I/O, formato, migração e privacidade;
  não foi pedida.
- **Decisão:** fita em memória, 50 entradas, sem gravar em disco.
- **Consequências:** fechar o app perde o histórico; persistência fica no
  backlog. Nenhum caminho de configuração em `$HOME` na v1.

### D-006 — Separador decimal localizado com entrada tolerante

- **Status:** aceita.
- **Contexto:** pt-BR usa vírgula; teclados e usuários podem digitar ponto.
- **Decisão:** exibir conforme o idioma (`pt` vírgula, `en` ponto); aceitar
  ambos na entrada; um separador por operando.
- **Consequências:** trocar idioma reformata visor e histórico sem alterar
  valores. Testes precisam cobrir ambas as entradas em ambos os idiomas.

### D-007 — Fontes Inter + JetBrains Mono embutidas

- **Status:** aceita e implementada na Fase 1.
- **Contexto:** o visual depende de tipografia consistente e o binário deve
  funcionar sozinho.
- **Decisão:** embutir Inter Regular/SemiBold (4.1) e JetBrains Mono
  Regular/Medium (2.304), OFL 1.1, sem subsetar nem modificar na v1.
- **Consequências:** 1.387.923 bytes de assets (dentro do orçamento); textos
  OFL no pacote; teste de cobertura de glifos verde. Nada de fontes do sistema.

### D-008 — Capturas PNG douradas em vez de snapshots de TUI

- **Status:** aceita.
- **Contexto:** não há TUI; o framework previa snapshots de texto.
- **Decisão:** comparar pixels RGBA crus de PNGs 1344 × 756, com
  `UPDATE_SNAPSHOTS=1`, na mesma toolchain.
- **Consequências:** snapshots detectam regressão visual; são sensíveis a
  versões de fonte/rasterizador, então o lock é pinado e o diff é revisado.
  Não há snapshot textual de terminal.

### D-009 — Somente teclado

- **Status:** aceita.
- **Contexto:** mouse exigiria hit-testing, foco e estados extras; o framework
  também o deixa fora da v1.
- **Decisão:** mouse/toque fora de escopo; toda interação por teclado.
- **Consequências:** teclas são exibidas como affordance visual, mas não
  respondem a clique. Documentado no README.

### D-010 — Tela única com fita, sem tela de ajuda

- **Status:** aceita.
- **Contexto:** o produto é pequeno; uma tela de ajuda separada adicionaria
  navegação sem necessidade.
- **Decisão:** `Screen = Calculator`; fita lateral; atalhos no rodapé; erros no
  visor.
- **Consequências:** descoberta depende do rodapé; textos devem ser curtos.
  Ajuda completa fica nos docs, não no app.

### D-011 — Sem RNG e sem `--seed`

- **Status:** aceita.
- **Contexto:** não há sorteio, IA ou aleatoriedade de jogo.
- **Decisão:** remover `--seed` e `util/rng.rs`; determinismo vem de estados
  fixos e da aritmética decimal.
- **Consequências:** capturas são determinísticas por construção; testes não
  precisam de seed.

### D-012 — Alvo de hardware oficial

- **Status:** aceita.
- **Contexto:** o ambiente auditado tem `amdgpudrmfb`, 1920 × 1080, 32 bpp.
- **Decisão:** alvo oficial 1920 × 1080 32 bpp. 24 e 16 bpp são suportados no
  código e validados se houver hardware/VM; caso contrário, ficam **pendentes**.
- **Consequências:** 32 bpp é portão de release; 24/16 não bloqueiam se
  explicitamente pendentes. Resoluções alternativas usam escala bilinear.

### D-013 — Áudio opcional sintetizado

- **Status:** aceita.
- **Contexto:** feedback sonoro sutil melhora a experiência, mas console pode
  não ter ALSA/dispositivo.
- **Decisão:** feature `audio` padrão, ALSA dinâmico via `libloading`, volume
  0,32, `M`/`--no-audio`, silêncio em falha.
- **Consequências:** dependência opcional; `--no-default-features` precisa
  compilar/testar; testes de síntese sem hardware.

### D-014 — Política de precisão e exibição

- **Status:** aceita.
- **Contexto:** precisão interna, arredondamento e largura do visor precisam ser
  previsíveis.
- **Decisão:** 15 dígitos de entrada no buffer; 8 casas internas com
  half-up; visor com até 12 dígitos significativos, milhar, máximo 16
  caracteres; científico fora da faixa.
- **Consequências:** resultados previsíveis e encaixe garantido; perda de
  precisão além de 8 casas é intencional e documentada.

### D-015 — Semântica de avaliação e histórico

- **Status:** aceita.
- **Contexto:** calculadoras variam em `=`, encadeamento e repetição.
- **Decisão:** avaliação sequencial (sem precedência); operador encadeado
  resolve e registra; operador duplo substitui sem registrar; `=` sem
  pendência é no-op; `=` sem segundo operando usa o acumulador; não repete a
  última operação; recall por `Enter` com seleção ativa.
- **Consequências:** `2 + 3 × 4 =` é `20`; a fita registra operações
  intermediárias; comportamentos precisam de testes explícitos.

### D-016 — `xtask` mínimo, sem pipeline de arte raster

- **Status:** aceita e implementada na Fase 1.
- **Contexto:** não há SVG/PNG de arte; o visual é primitivo + fontes.
- **Decisão:** `xtask` valida presença/hash/tamanho de fontes e gera manifesto
  FNV-1a 64. Sem download, sem SVG→PNG, sem dependências externas.
- **Consequências:** `cargo xtask assets --check` continua sendo portão; FNV
  detecta drift, não é segurança. Arte raster pode voltar no backlog.

### D-017 — Entrada ativa pode ocupar 17 caracteres

- **Status:** aceita.
- **Contexto:** a regra de 15 dígitos de entrada + separador + sinal pode
  produzir 17 caracteres, enquanto o visor formata resultados em até 16.
  Truncar a entrada esconderia dígitos; usar notação científica durante a
  digitação seria confuso.
- **Decisão:** resultados formatados usam até 16 caracteres, com agrupamento e
  fallback científico. A entrada ativa pode chegar a 17 caracteres, sem
  agrupamento quando necessário; JetBrains Mono a 64 px cabe nos 672 px úteis.
- **Consequências:** o domínio expõe a exceção; capturas e testes de layout
  precisam cobrir o caso de 17 caracteres. `MAX_DISPLAY_CHARS` continua 16 para
  valores formatados.

### D-018 — Substrato fb da Fase 2

- **Status:** aceita e implementada na Fase 2.
- **Contexto:** a captura PNG e o backend fb exigem rasterização, blend,
  texto e acesso a ioctl/mmap sem GPU. As crates `tiny-skia`, `image`,
  `libc` e `bytemuck` já estavam em cache e foram aprovadas pelo usuário.
- **Decisão:** `tiny-skia` rasteriza a cena lógica e compõe com
  `draw_pixmap` (1:1 em 1080p, bilinear nas demais escalas); `image` com
  feature `png` encoda a captura; `libc` isola ioctl/mmap/sinais em
  `platform/fb.rs`; `bytemuck` fornece `Pod`/`Zeroable` para a ABI de 160
  bytes. Não foi adicionado pipeline de arte nem `memmap2`.
- **Consequências:** `unsafe` continua restrito a `platform/fb.rs`; a captura
  não abre fb/tty/áudio; 24/16 bpp têm conversão por bitfield testada; a
  validação em VT real segue bloqueada por P-01/P-02.

## 2. Premissas

- O usuário pode entrar no grupo `video` e fazer novo login.
- Há um VT local acessível (ex.: Ctrl+Alt+F3) e `/dev/fb0` funcional.
- O console alvo é 1920 × 1080 32 bpp; 24/16 bpp podem não existir no hardware.
- `rust_decimal` tem licença MIT e permanece compatível (1.43.0 verificada).
- As fontes Inter e JetBrains Mono são OFL 1.1, foram baixadas sem modificação
  e tiveram hashes registrados.
- O produto não precisa de persistência nem de rede.
- Não há prazo externo; a qualidade e a validação real precedem a pressa.
- `stdin` é TTY no VT; `stdout` pode estar redirecionado.

## 3. Riscos

| ID | Risco | Prob. | Impacto | Mitigação | Status |
| --- | --- | --- | --- | --- | --- |
| R-001 | Usuário fora do grupo `video` | alta | bloqueia fb | `usermod -aG video` + relogin; mensagem clara | **aberto** |
| R-002 | Sessão Wayland/X11 ativa; `KDSETMODE` falha | alta | bloqueia fb | executar em VT local; código 3 com instrução | **aberto** |
| R-003 | `rust_decimal` ausente do cache; download falha | média | atrasa Fase 1 | baixada 1.43.0, pinada e testada | resolvido |
| R-004 | Capturas variam por versão de fonte/rasterizador | média | falsos negativos | lock pinado; mesma toolchain; revisão de diff | aberto |
| R-005 | `amdgpudrmfb` sem `ypan` ou com comportamento distinto | média | tearing/performance | detectar `ypan`; fallback escrita direta | aberto |
| R-006 | Sem TUI, desenvolvimento/SSH ficam limitados | média | produtividade | `--capture-frame`; testes headless; TUI no backlog | aceito |
| R-007 | Contraste ruim em 16/mono | média | acessibilidade | capturas por paleta; testes de contraste; relevo | aberto |
| R-008 | Usuário esperar `0,10` literal em `%` | baixa | confusão | exemplos no README/UI; testes | aceito |
| R-009 | Arredondamento em 8 casas surpreender | baixa | confusão | documentar limite; mensagem nos docs | aceito |
| R-010 | Áudio sem dispositivo | baixa | silêncio inesperado | status “Som indisponível”; sem erro fatal | aceito |
| R-011 | Glifo faltante em fonte | média | texto quebrado | teste de cobertura; primitiva/alternativa | aberto |
| R-012 | Troca de VT deixa resíduos | baixa | console sujo | documentar; restaurar em Drop/sinais | aberto |
| R-013 | Escopo crescer (científica, mouse, persistência) | baixa | atraso | não-objetivos congelados; backlog | aceito |
| R-014 | Licença de dependência divergir da expectativa | baixa | bloqueio de release | `cargo metadata` antes do release | aberto |
| R-015 | `unsafe` de ioctl/mmap com ABI errada | baixa | crash/corrupção | const asserts 160/80; validação; testes implementados; falta VT real | aberto |
| R-016 | Fonte grande demais para o orçamento | baixa | binário > 15 MB | assets medidos em 1,39 MB; subset no backlog | resolvido |

## 4. Questões abertas

Nenhuma questão **bloqueante** de produto. Pendências operacionais:

1. Adicionar `daniel` ao grupo `video` e relogar (R-001).
2. Escolher o VT para validação manual (ex.: tty3) e confirmar que está livre
   (R-002).
3. Confirmar disponibilidade de hardware/VM 24/16 bpp; se não houver, marcar
   como pendente no release (D-012).
4. Definir se haverá release público; por padrão, **não** publicar sem pedido.

Defaults já congelados, sem necessidade de nova pergunta:

- Sem persistência, sem mouse, sem científica, sem repetição de `=`.
- Sem TUI na v1.
- `pt-BR` como idioma inicial.
- 50 entradas de histórico, 10 visíveis.
- Capturas determinísticas na mesma toolchain.

## 5. Critério para reabrir uma decisão

1. Pedido explícito do usuário.
2. Nova ADR com status `proposta`, contexto, decisão e consequências.
3. Atualização de docs afetados, testes e capturas na mesma tarefa.
4. Nenhuma decisão congelada é alterada silenciosamente por um agente.
