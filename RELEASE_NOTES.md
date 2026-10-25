# Calculadora 1.0.0 — notas de release

Data: 2026-10-25

Calculadora decimal para console Linux local, renderizada diretamente no
framebuffer (`/dev/fb0`), sem X11/Wayland, sem rede e sem arquivos externos em
runtime.

## Destaques

- Quatro operações, `±`, porcentagem contextual, `AC` e `DEL`.
- Decimal exato com `rust_decimal`, 8 casas internas e formatação pt-BR/en com
  milhar e notação científica.
- Visor com expressão, resultado e erros; avaliação sequencial.
- Fita de histórico da sessão: 50 entradas, 10 visíveis, rolagem, seleção e
  recall com `Enter`.
- `F2` alterna idioma; `M` alterna mudo; `--no-animations`, `--no-audio`,
  `NO_COLOR` e `--color auto|truecolor|256|16|never`.
- Flash de tecla, debounce de confirmação e fade da nova entrada da fita.
- Áudio opcional sintetizado (clique e erro) via ALSA dinâmico; ausência de
  biblioteca/dispositivo resulta em silêncio e status “Som indisponível”.
- Backend fbdev com validação de ABI e formatos 32/24/16 bpp.
- Captura PNG determinística de 1344 × 756 para QA sem hardware.
- Fontes Inter e JetBrains Mono embutidas; o binário funciona copiado sozinho.

## Requisitos

- Linux x86_64 com VT local e `/dev/fb0` (alvo oficial: `amdgpudrmfb`,
  1920 × 1080, 32 bpp).
- Usuário no grupo `video` e sessão/VT com o grupo aplicado.
- Para áudio: ALSA/PipeWire disponível; caso contrário, o app permanece
  silencioso.

## Instalação e execução

Extraia o pacote e execute no VT local:

```bash
tar -xzf calculadora-1.0.0-linux-x86_64.tar.gz
cd calculadora-1.0.0-linux-x86_64
./calculadora
```

Fora de um VT local (Wayland/X11/SSH) o backend fb recusa com código `3` e
mensagem acionável. Para QA sem framebuffer:

```bash
./calculadora --capture-frame /tmp/calculadora.png
```

## Validação desta versão

- Portões automatizados verdes: `fmt`, `clippy` (com/sem features), 194 testes
  ativos, manifesto de assets e builds release.
- 16 capturas douradas em `tests/snapshots/` (1344 × 756, comparação RGBA).
- Execução inicial em VT real (`amdgpudrmfb`, 1920 × 1080, 32 bpp) reportada
  **OK** pelo usuário em 2026-10-25.
- **Pendências registradas:** detalhes de áudio audível, latência de tecla, CPU
  ocioso, restauração por cada caminho (`Q`/`Esc`/`Ctrl+C`/`SIGTERM`/`SIGHUP`/
  pânico), troca de VT e validação em 24/16 bpp ou resoluções alternativas.
- A MSRV alvo é 1.88; o ambiente de release usou Rust 1.99.1.

## Limitações conhecidas

- Backend único fbdev; sem TUI, DRM/KMS, mouse, persistência ou funções
  científicas.
- Sem vsync; troca de VT durante a execução pode deixar resíduos.
- Modos 24/16 bpp têm fidelidade de cor menor e ainda não foram validados em
  hardware neste ambiente.
- Áudio depende de ALSA/PipeWire; não é um requisito para o app funcionar.

## Licenças

- Código: MIT (`LICENSE`).
- Interface/arte original: CC BY 4.0 (`LICENSE-ART`).
- Fontes: SIL Open Font License 1.1 (textos em `licenses/`).
- Dependências: `THIRD_PARTY_NOTICES.md`.
