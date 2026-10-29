# Calculadora 1.0.2 — notas de release

Data: 2026-10-28

Calculadora decimal para console Linux local, renderizada diretamente no
framebuffer (`/dev/fb0`), sem X11/Wayland, sem rede e sem arquivos externos em
runtime.

## Destaques

- Corrige a tela invisível em VTs que chegam ao app com `blank`/DPMS desligado:
  o backend fb agora chama `FBIOBLANK`/`FB_BLANK_UNBLANK` logo após entrar em
  `KD_GRAPHICS`. Falha nesse ioctl é não fatal.
- Mantém a correção de `yoffset`/double buffer da 1.0.1: lê o `yoffset` atual,
  valida `yres_virtual`, `ypanstep` e `smem_len`, alterna buffers com
  `FBIOPAN_DISPLAY` e usa escrita direta no buffer visível como fallback.
- Melhora o mapeamento de memória para o fallback: cobre também o buffer visível
  quando o driver usa pan, evitando falha em geometrias com três ou mais
  buffers virtuais.
- `install.sh` continua no pacote: valida o arquivo, instala o binário e a
  documentação e adiciona o usuário ao grupo `video` com `sudo` interativo
  quando necessário.
- Quatro operações, `±`, porcentagem contextual, `AC` e `DEL`.
- Decimal exato com `rust_decimal`, 8 casas internas e formatação pt-BR/en com
  milhar e notação científica.
- Fita de histórico da sessão: 50 entradas, 10 visíveis, rolagem, seleção e
  recall com `Enter`.
- `F2` alterna idioma; `M` alterna mudo; `--no-animations`, `--no-audio`,
  `NO_COLOR` e `--color auto|truecolor|256|16|never`.
- Áudio opcional sintetizado via ALSA dinâmico; ausência de biblioteca ou
  dispositivo resulta em silêncio e status “Som indisponível”.
- Captura PNG determinística de 1344 × 756 para QA sem hardware.
- Fontes Inter e JetBrains Mono embutidas; o binário funciona copiado sozinho.

## Causa raiz da tela em branco

O diagnóstico no segundo computador mostrou:

- `i915drmfb`, 1920 × 1080, 32 bpp, sem ypan (`yres_virtual == yres`).
- `/sys/class/graphics/fb0/blank: 4` antes do app.
- O app entrava em `KD_GRAPHICS` e desenhava, mas não chamava `FBIOBLANK`.
- O `Ctrl+C` gerava entrada de teclado, acordava o console e revelava a imagem
  por instantes.

A correção não assume `yoffset`, permissão, Wayland ou instalação: ela
desbloqueia explicitamente o console no startup.

## Requisitos

- Linux x86_64 com VT local e `/dev/fb0` (alvo oficial: `amdgpudrmfb`,
  1920 × 1080, 32 bpp).
- Usuário no grupo `video` e sessão/VT com o grupo aplicado.
- Para áudio: ALSA/PipeWire disponível; caso contrário, o app permanece
  silencioso.

## Instalação e execução

Extraia o pacote e execute o instalador:

```bash
tar -xzf calculadora-1.0.2-linux-x86_64.tar.gz
cd calculadora-1.0.2-linux-x86_64
./install.sh
```

O instalador copia o binário para `$HOME/.local/bin/calculadora`, instala a
documentação em `$HOME/.local/share/doc/calculadora` e, se necessário, pede a
senha de `sudo` para adicionar seu usuário ao grupo `video`. Opções úteis:

```bash
./install.sh --check       # valida pacote e ambiente sem instalar
./install.sh --skip-fb     # não verifica nem altera permissões do framebuffer
./install.sh --prefix DIR  # instala em outro prefixo
./install.sh --uninstall   # remove binário e documentação
```

Depois de mudar o grupo `video`, saia e entre novamente (ou reinicie). Troque
para um VT livre com `Ctrl+Alt+F3`, faça login e execute:

```bash
"$HOME/.local/bin/calculadora"
```

Fora de um VT local (Wayland/X11/SSH) o backend fb recusa com código `3` e
mensagem acionável. Para QA sem framebuffer:

```bash
./calculadora --capture-frame /tmp/calculadora.png
```

## Validação desta versão

- Portões automatizados verdes: `fmt`, `clippy` (com/sem features), **196 testes
  ativos**, manifesto de assets e builds release.
- 16 capturas douradas em `tests/snapshots/` (1344 × 756, comparação RGBA).
- Teste manual completo em VT real (`amdgpudrmfb`, 1920 × 1080, 32 bpp)
  reportado **OK** em 2026-10-25 para a 1.0.0.
- A correção de blank/DPMS foi validada pela mesma técnica na aplicação irmã
  (tela com `blank: 4` passou a aparecer imediatamente após o `FBIOBLANK`).
- **Pendente:** confirmar esta versão no segundo computador e registrar driver,
  `blank`, `virtual_size`, bpp, `yoffset`, `yres_virtual`, `ypanstep` e
  `fbset -i`.
- **Pendências registradas:** 24/16 bpp e resoluções alternativas, detalhes de
  `SIGHUP`/pânico, medições formais de CPU/latência e MSRV 1.88 (o release usou
  Rust 1.99.1).

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
