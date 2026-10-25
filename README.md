# Calculadora

> Cálculo essencial, com elegância.

Calculadora decimal para **console Linux local**, renderizada diretamente no
framebuffer (`/dev/fb0`), sem X11/Wayland, sem rede e sem dependência de
servidor gráfico. Interface escura, tipografia cuidada, fita de histórico e
suporte a português e inglês.

**Status:** Fase 1 (fundação) concluída em 2026-10-25. Domínio, CLI, i18n,
input, guardas de TTY e assets estão implementados e testados. O runtime de
framebuffer e a captura PNG entram na Fase 2; até lá, executar o binário
informa que o backend ainda não está disponível. Consulte `docs/06-roadmap.md`.

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

- Linux x86_64 local, com VT ativo (não serve sessão X11/Wayland/SSH).
- `/dev/fb0` disponível. Alvo oficial: `amdgpudrmfb`, 1920 × 1080, 32 bpp
  (detectado no ambiente em 2026-10-25; execução ainda pendente).
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
```

## Compilação e testes

```bash
cargo build --release
cargo test --workspace
cargo xtask assets --check
```

A Fase 1 entrega a fundação testável; executar `calculadora` ainda termina com
código `3` e a mensagem de que o backend fb será implementado na Fase 2.

Quando a captura estiver disponível (Fase 2), o comando será:

```bash
./target/release/calculadora --capture-frame /tmp/calculadora.png
```

A captura será determinística, terá 1344 × 756 e não abrirá framebuffer,
terminal interativo nem áudio.

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
acionável e código de saída `3`.

### Teclas

| Tecla | Ação |
| --- | --- |
| `0`–`9` (topo e numpad) | digita dígito |
| `,` ou `.` | separador decimal |
| `+` `-` `*` `/` (e numpad) | operação |
| `Enter` ou `=` | calcula; com histórico selecionado, recupera o resultado |
| `Backspace` | apaga o último dígito |
| `Delete` ou `C` | limpa tudo (AC) |
| `S` | troca o sinal (±) |
| `%` | porcentagem contextual |
| `↑` / `↓` | navega na fita de histórico |
| `F2` | alterna pt-BR / en |
| `M` | liga/desliga o som |
| `Q` ou `Esc` | sai e restaura o console |

Qualquer dígito ou operador cancela a seleção do histórico. `Ctrl+C` em raw
mode chega como tecla e é tratado como `Q`.

### Exemplos de porcentagem

| Entrada | Resultado |
| --- | --- |
| `200` `+` `10` `%` `=` | `220` |
| `200` `-` `10` `%` `=` | `180` |
| `200` `×` `10` `%` `=` | `20` |
| `10` `%` (sem operação pendente) | `0,1` |

Após `%`, o visor mostra o operando já convertido; `=` ou um operador seguinte
fecha a conta. Em pt-BR usa-se vírgula decimal; em en, ponto.

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
