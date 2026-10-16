# `setup.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

## `setup.*` — o passo a passo oficial de instalação, por distro

```text
setup.list {}  ->  { distroId, distroName, family, tools: [SetupToolInfo] }
```

```text
SetupToolInfo   id · name · summary · website · installed · guide?
SetupGuide      family · sourceUrl · checkedAt · steps: [SetupStep]
SetupStep       explanation · command
```

**Não exige workspace**: instalar o PostgreSQL não depende de projeto aberto, e
quem está começando abre a IDE antes de ter projeto — que é exatamente quando
este guia serve.

**A regra que governa o domínio inteiro: sem fonte oficial, a IDE não afirma.**
Cada `SetupGuide` carrega `sourceUrl` e `checkedAt`, e uma família de distro sem
fonte oficial **não recebe guia** — o `guide` volta `None` e a tela mostra o site
do projeto dizendo que não tem passo a passo. É a decisão registrada em
`../roadmaps/40` §5, e é por isso que Arch e openSUSE continuam sem passos: a
fonte dos três projetos não cobre essas famílias.

**Embarcados (A5 do `roadmaps/41`, 2026-09-17, sem mudança de contrato):**
o `setup.list` passou a unir dois catálogos — o geral (`setup/catalog.rs`) e
o de embarcados (`setup/catalog_embedded.rs`): `esptool`, `mpremote`,
`espflash`, `probe-rs`, `picotool`, `dfu-util`, `tio`, `picocom`,
`arm-none-eabi` (gcc + gdb), `qemu-embedded` (ARM e RISC-V), `openocd`.
Duas classes de fonte, ditas no arquivo: a página da ferramenta (comando
verbatim — `pip install esptool` num venv, `pipx install mpremote`,
`cargo install espflash --locked`, o instalador do probe-rs, o `BUILDING.md`
do picotool) e o índice de pacotes da distro (a página do pacote prova o
nome; o comando é a forma padrão do gerenciador). Onde o índice não tem o
pacote — `tio` e `picotool` no Arch oficial, `picotool` e `espflash` no
Fedora, `espflash` no Ubuntu 26.04 (a família `debian` cobre Debian e
Ubuntu; um guia que falha em metade da família não entra) — a entrada não
existe. Medido nesta máquina (Ubuntu 26.04): 18 ferramentas, 7 já instaladas
entre as de embarcado.

O `installed` vem do `ToolDetector` — detectar é capacidade, e a política de o
que fazer com a detecção fica na UI (`27-modulos-por-dominio.md` §6).
