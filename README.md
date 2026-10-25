# Calculadora

> Cálculo essencial, com elegância.

Calculadora decimal para **console Linux local**, renderizada diretamente no
framebuffer (`/dev/fb0`), sem X11/Wayland, sem rede e sem dependência de
servidor gráfico. Interface escura, tipografia cuidada, fita de histórico e
suporte a português e inglês.

**Versão inicial 0.1.0 — Fases 1 a 3 concluídas em 2026-10-25 no caminho
testável sem hardware.** O fluxo completo já funciona: digitação, quatro
operações, porcentagem, sinal, AC/DEL, erros, fita de histórico com rolagem e
recall, `F2`, `M`, flash de tecla e debounce de confirmação. A captura PNG e as
16 capturas douradas estão implementadas. **A execução em VT real continua sem
validação neste ambiente** (falta permissão no `/dev/fb0` e um VT livre), e o
áudio/fade de histórico ficam para a Fase 4. Consulte `docs/06-roadmap.md`.

## O que funciona agora

- Cálculo completo pelo teclado: `+`, `−`, `×`, `÷`, `±`, `%`, `AC`, `DEL`.
- Visor com expressão, resultado, erro e modo resultado; aritmética decimal
  exata e avaliação sequencial.
- Fita com até 50 entradas, 10 visíveis, rolagem orientada pela seleção e recall
  do resultado com `Enter`/`=`.
- `F2` alterna pt-BR/en (inclusive histórico e separadores); `M` alterna o
  estado do som; `Q`/`Esc`/`Ctrl+C` saem restaurando o console.
- Flash de tecla de 90 ms (60 ms com `--no-animations`), debounce de 120 ms em
  `=`/`Enter` e marcador de som por forma (preenchido/vazado/traço).
- Paletas truecolor, 256, 16 e monocromática.
- Captura PNG determinística de 1344 × 756, sem abrir framebuffer, TTY ou áudio.
- 16 capturas douradas em `tests/snapshots/` cobrindo inicial pt/en, pendência,
  resultado, porcentagem, erro, histórico, seleção, flash, paletas, som e
  textos longos.
- fbdev com validação de ABI e formatos 32, 24 e 16 bpp (a validação em
  hardware dos modos 24/16 ainda está pendente).

## O que ainda não funciona

- Áudio real (clique/erro) e fade de nova entrada da fita (Fase 4). O status de
  som já é exibido, mas o backend ALSA ainda não foi ligado.
- Validação em VT real, restauração observada no console e modos 24/16 bpp
  (Fase 5; bloqueada por permissão e hardware).
- Troca de VT durante a execução pode deixar resíduos; saia do app antes.

## Visão do produto

- Quatro operações básicas, sinal, porcentagem contextual e apagar dígito.
- Aritmética decimal exata (`rust_decimal`), não ponto flutuante binário.
- Visor com até 12 dígitos significativos, separador de milhar e notação
  científica para valores extremos.
- Fita de histórico da sessão, navegável por teclado.
- Interface em pt-BR e en, alternável com `F2`.
- Clique de tecla opcional sintetizado em runtime; sem arquivos de áudio.
- Acesso por teclado; mouse fora do escopo da v1.

## Requisitos

- Linux x86_64 local, com VT ativo para usar o framebuffer (não serve sessão
  X11/Wayland/SSH).
- `/dev/fb0` disponível. Alvo oficial: `amdgpudrmfb`, 1920 × 1080, 32 bpp.
- Usuário no grupo `video` (a permissão típica é `root:video 660`).
- Rust stable, edição 2021, MSRV sugerida 1.88+ (ambiente atual: 1.99.1).
- Fontes Inter e JetBrains Mono embutidas no binário (OFL 1.1).

Diagnóstico rápido:

```bash
ls -l /dev/fb0
cat /sys/class/graphics/fb0/name
cat /sys/class/graphics/fb0/virtual_size
cat /sys/class/graphics/fb0/bits_per_pixel
id -nG            # deve conter video
```

Se `video` não aparecer, adicione o usuário e **faça novo login**:

```bash
sudo usermod -aG video "$USER"
newgrp video      # alternativa imediata para a sessão atual
id -nG            # confirme que 'video' aparece
```

## Como testar esta versão

### 1. Compilar

```bash
cargo build --release
./target/release/calculadora --version
```

Saída esperada: `calculadora 0.1.0`.

O binário é autossuficiente: as fontes estão embutidas e nada é lido do disco
em runtime. Para confirmar, copie-o para outro diretório e gere uma captura:

```bash
cp target/release/calculadora /tmp/
cd /tmp
./calculadora --capture-frame /tmp/calculadora-embedded.png
cd - >/dev/null
```

### 2. Teste headless (recomendado, não abre framebuffer)

Gere a tela inicial em dois estados:

```bash
tmp=$(mktemp -d)
./target/release/calculadora --capture-frame "$tmp/pt.png"
./target/release/calculadora --lang en --color never \
  --capture-frame "$tmp/en-mono.png"
file "$tmp/pt.png" "$tmp/en-mono.png"
```

Saída esperada: ambos com `PNG image data, 1344 x 756, 8-bit/color RGBA`.
Abra os arquivos em qualquer visualizador de imagens.

Verifique o determinismo:

```bash
./target/release/calculadora --capture-frame "$tmp/pt-2.png"
cmp "$tmp/pt.png" "$tmp/pt-2.png" && echo "captura determinística"
sha256sum "$tmp/pt.png" "$tmp/pt-2.png"
```

Os hashes devem ser iguais na mesma toolchain. Para provar que a captura não
toca no framebuffer, aponte `--fb-device` para um caminho inexistente:

```bash
./target/release/calculadora \
  --fb-device /dev/nao-existe \
  --capture-frame "$tmp/sem-fb.png"
```

O comando deve terminar com sucesso.

### 3. Portões automatizados

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

Estado esperado desta versão: **182 testes ativos verdes** — 170 unitários,
8 de processo/CLI, 1 de capturas douradas, 1 de PTY, 1 de sinais e 1 do xtask —
além de 2 testes auxiliares `#[ignore]` usados pelos testes de PTY e sinais.
O clippy deve ficar sem avisos e o manifesto de assets atualizado.

Para validar apenas as capturas douradas (16 estados):

```bash
cargo test --test golden
# regenerar após revisar a mudança visual:
UPDATE_SNAPSHOTS=1 cargo test --test golden
```

### 4. Execução em VT local (framebuffer real)

Pré-requisitos: usuário no grupo `video` após novo login e um VT livre, por
exemplo `tty3`. Não execute a partir de um terminal dentro de X11/Wayland.

```bash
# no diretório do projeto, ainda na sessão gráfica
cargo build --release

# confira device e permissões
ls -l /dev/fb0
cat /sys/class/graphics/fb0/name
cat /sys/class/graphics/fb0/bits_per_pixel
id -nG
```

Troque para o VT livre com `Ctrl+Alt+F3`, faça login e execute:

```bash
cd /caminho/para/calculadora
./target/release/calculadora
```

O que observar nesta versão:

- A tela inicial deve aparecer centralizada, com a fita vazia, o visor `0` e o
  keypad.
- Digite `200 + 10 % =` e confirme `220` no visor e uma entrada na fita.
- Faça `0,1 + 0,2 =` e confirme `0,3`.
- Faça `1 ÷ 0 =` e confirme `Erro`/`Divisão por zero`; digite `7` para
  recuperar.
- A fita registra as operações; `↑`/`↓` navegam e `Enter` recupera o resultado
  sem criar nova entrada.
- `F2` deve trocar `PT`/`EN`, o rodapé, o histórico e o separador decimal.
- `M` deve alternar `Som: ligado`/`Som: desligado` (o áudio real chega na
  Fase 4).
- `Q`, `Esc` ou `Ctrl+C` devem sair e devolver o console em modo texto.

Para testar a restauração por sinal, abra outro VT, descubra o PID e envie
`SIGTERM`:

```bash
pgrep -n calculadora
kill -TERM <pid>
```

O console deve voltar ao modo texto. Evite trocar de VT enquanto o app estiver
em execução; se isso acontecer, saia e reinicie.

### 5. Checklist rápido de aceite

- [ ] `--version` mostra `calculadora 0.1.0`.
- [ ] A captura gera PNG de 1344 × 756.
- [ ] Duas capturas iguais têm o mesmo SHA-256.
- [ ] `--lang en`, `--color never` e `--no-audio` alteram a captura.
- [ ] A captura funciona com `--fb-device` inexistente.
- [ ] Os portões automatizados passam com e sem features.
- [ ] No VT, a tela aparece e o fluxo `200 + 10 % =` mostra `220`.
- [ ] No VT, `↑`/`↓`/`Enter` navegam e recuperam o histórico.
- [ ] No VT, `F2`/`M` funcionam e `Q` restaura o console.
- [ ] Sem permissão no fb, a mensagem orienta o grupo `video` (código 3).

## Uso

```text
calculadora [OPTIONS]

  --lang <pt|en>                 idioma inicial (padrão: pt)
  --backend <fb>                 backend de vídeo (v1: somente fb)
  --fb-device <PATH>             dispositivo (padrão: /dev/fb0)
  --color <auto|truecolor|256|16|never>
  --no-animations                reduz movimento
  --no-audio                     inicia em silêncio
  --capture-frame <PATH>         gera PNG determinístico e sai
  -h, --help
  -V, --version
```

`NO_COLOR` tem prioridade sobre `--color`. O app nunca tenta detectar
framebuffer automaticamente: falhas de permissão ou de VT produzem mensagem
acionável e código de saída `3`. A captura não abre framebuffer, terminal
interativo nem áudio.

### Teclas nesta versão

| Tecla | Ação | Estado |
| --- | --- | --- |
| `F2` | alterna pt-BR / en | funciona |
| `M` | liga/desliga o estado do som | funciona (áudio na Fase 4) |
| `Q`, `Esc`, `Ctrl+C` | sai e restaura o console | funciona |
| `0`–`9`, `,`/`.`, `+` `-` `*` `/` | dígitos, separador e operações | funciona |
| `Enter` ou `=` | calcula; com seleção, recupera o resultado | funciona |
| `Backspace`, `Delete`/`C`, `S`, `%` | apagar, limpar, sinal e porcentagem | funciona |
| `↑` / `↓` | navega na fita de histórico | funciona |

## Códigos de saída

| Código | Significado |
| ---: | --- |
| `0` | sucesso (inclui captura e saída normal) |
| `1` | erro de runtime/IO/captura (por exemplo, caminho PNG inválido) |
| `2` | erro de CLI (flag inválida; `--backend tui`/`auto` é rejeitado) |
| `3` | backend indisponível: fb sem permissão, fora de VT ou stdin não é TTY |

## Solução de problemas

| Mensagem/situação | Causa provável | Ação |
| --- | --- | --- |
| `cannot open /dev/fb0: permission denied` | usuário fora de `video` | `sudo usermod -aG video "$USER"`, novo login |
| `KDSETMODE failed` / `run from a local virtual console` | terminal em X11/Wayland/SSH | use `Ctrl+Alt+F3` e execute no VT |
| `stdin is not a TTY` | entrada redirecionada | execute interativamente no VT |
| `unsupported framebuffer format` | bpp/bitfields fora de 32/24/16 esperados | registre driver e saída de `bits_per_pixel` |
| captura não gera arquivo | diretório inexistente ou sem permissão | use um caminho gravável, como `/tmp/x.png` |

## Documentação

| Documento | Conteúdo |
| --- | --- |
| `framework.md` | template de arquitetura obrigatório |
| `docs/01-calculadora.md` | escopo, regras de cálculo, fluxos e aceite |
| `docs/02-technical-spec.md` | arquitetura, crates, CLI, fbdev, desempenho |
| `docs/03-visual-design.md` | paleta, tipografia, layout, estados e movimento |
| `docs/04-assets-and-licenses.md` | fontes, pipeline, manifesto e licenças |
| `docs/05-test-plan.md` | pirâmide de testes, capturas douradas e matriz manual |
| `docs/06-roadmap.md` | fases e critérios de saída |
| `docs/07-decisions-and-risks.md` | ADRs, premissas, riscos e questões |
| `docs/08-i18n-catalog.md` | catálogo de strings pt/en e limites de layout |
| `docs/09-release-checklist.md` | checklist de release v1.0 |
| `docs/10-validation-log.md` | ambiente auditado e validações pendentes |
| `THIRD_PARTY_NOTICES.md` | crates, licenças e créditos |
| `AGENTS.md` | memória operacional e regras para agentes de IA |

## Licenças

- Código: **MIT**.
- Arte original: **CC BY 4.0**.
- Fontes: **SIL Open Font License 1.1** (Inter 4.1 e JetBrains Mono 2.304),
  com textos em `assets/fonts/`.
- Dependências: ver `THIRD_PARTY_NOTICES.md`.

Este projeto não faz parte do projeto original que inspirou o `framework.md`;
apenas reutiliza a forma arquitetônica.
