# 36 — Ferramentas de embarcados: o levantamento

> **Classe: ESTADO.** Tem que ser verdade hoje. Toda licença e versão foi
> **verificada na fonte em 2026-09-03** — no arquivo de licença do projeto
> quando o detector automático falhou, o que aconteceu em **3 das 4**.
>
> **Etapa 21 do [`roadmaps/35`](../roadmaps/35-ambiente-cpp-e-embarcados.md)
> §5.4.** Documento, não código: sem ele, escolher entre probe-rs e OpenOCD
> seria palpite, e a escolha **decide o desenho** da etapa 22.

## 0. Por que este levantamento existe antes de qualquer linha

A frente de embarcados é **plug and play** por decisão do autor
(`roadmaps/35` §5.1), e plug and play é a experiência do **usuário** — não
atalho no gate de adoção. Cada ferramenta aqui é candidata, **nenhuma está
adotada**: a adoção passa pelo checklist de
[`README.md`](README.md) §"Checklist para adicionar uma integração nova".

E há a lição de 2026-09-03, que se repetiu: o campo `license` da API do GitHub
errou em **spdlog, CLI11, zlib, asio** (catálogo de bibliotecas) e agora em
**OpenOCD e QEMU**. Ler a fonte não é rigor decorativo; é o que impede o
catálogo de mentir.

## 1. As quatro, medidas

| ferramenta | licença | última versão | data | protocolo com a IDE |
| --- | --- | --- | --- | --- |
| **probe-rs** | MIT **e** Apache-2.0 (dual) | v0.32.0 | 2026-07-22 | **DAP nativo**, stdin/stdout |
| **OpenOCD** | GPL-2.0-or-later | v0.12.0 | — | GDB remote (TCP) |
| **pyOCD** | Apache-2.0 | v0.45.1 | 2026-07-21 | GDB remote (TCP) |
| **QEMU** | GPL-2.0 | v11.1.1 | — | GDB remote (TCP) |

**Todas ativas:** as quatro tiveram push nos últimos 45 dias, nenhuma
arquivada.

## 2. A licença não elimina ninguém, e a regra que diz isso já existe

Duas são **GPL** (OpenOCD, QEMU). Isso **não** as desqualifica, e a regra é
anterior a esta frente — `LEITURA_TECNICA` §4 fato 5:

> *"Ferramenta externa com licença copyleft (o GDB é GPL-3) é **executada como
> processo**, nunca linkada."*

O `gdb` já é GPL-3 e já é orquestrado assim. OpenOCD e QEMU entram pela mesma
porta: processo filho, comunicação por protocolo, zero código linkado.

**O que seria proibido**, e não é o caso de nenhuma: linkar uma biblioteca
copyleft dentro do core ou da UI.

## 3. O achado que decide o desenho da etapa 22

**probe-rs fala DAP nativamente, e por stdin/stdout.**

A documentação do projeto é explícita: o `probe-rs dap-server` implementa o
Debug Adapter Protocol, e **quando `--port` é omitido ele se comunica por
stdin/stdout em vez de TCP**, para clientes DAP que sobem o adaptador como
processo filho.

Isso é *exatamente* a forma que o domínio `dap/` da Kinein já usa:

```text
o que a Kinein ja faz hoje          o que probe-rs oferece
──────────────────────────          ──────────────────────
spawn do adaptador como filho   ==  dap-server como processo filho
Content-Length sobre stdin/out  ==  stdin/stdout quando --port e omitido
dap/wire.rs leva e traz         ==  o MESMO wire, sem mudanca
```

**Consequência concreta:** adotar probe-rs custa tornar `ADAPTER_BINARY` uma
escolha do kit — e mais nada no transporte. O `dap/` foi cortado em quatro
donos na etapa 15 e o spawn está isolado em `session.rs`.

**OpenOCD, pyOCD e QEMU falam GDB remote**, não DAP. Usá-los exigiria uma das
duas:

```text
(a) subir `gdb` como adaptador e falar MI/DAP com ele    ponte a mais
(b) implementar cliente GDB remote no core               protocolo novo inteiro
```

Nenhuma das duas é impossível; as duas são **muito** mais caras que trocar uma
constante por um campo do kit.

> **CORRIGIDO em 2026-09-11 — a (a) já existe pronta, e não é ponte "a mais".**
> O GDB fala DAP **nativamente** desde a versão 14 (fonte: `/usr/share/doc/gdb/NEWS`,
> seção *Changes in GDB 14*: *"GDB has initial built-in support for the Debugger
> Adapter Protocol"*; exige GDB compilado com Python). Medido nesta máquina com o
> gdb 17.2 do Fedora: `gdb -i dap` responde `initialize` com `success: true`, e
> ele é multiarch (`set architecture arm` e `riscv:rv32` aceitos). O manual
> (sourceware, capítulo *Debugger Adapter Protocol*) documenta o pedido `attach`
> com o campo `target`, *"passed to the `target remote` command"*, mais `program`.
>
> **Consequência:** OpenOCD, pyOCD, QEMU e o próprio `probe-rs gdb` entram pelo
> `dap/` existente como um **segundo candidato do papel `debugAdapter`** —
> `gdb` com `-i dap` — sem cliente GDB remote no core. O custo real, que a
> versão anterior deste parágrafo não via: gerenciar o **processo servidor**
> (openocd/qemu) ao lado do adaptador, e um caminho de `attach` na sessão, que
> hoje só faz `launch`. Decisão do autor em 2026-09-11: entra
> (`roadmaps/35` §5.7).

## 4. A recomendação, e o que ela não decide

**Começar por probe-rs**, e a razão é arquitetural, não preferência:

- é o único que encaixa no `dap/` existente sem protocolo novo;
- cobre ARM **e** RISC-V, que são os dois alvos que o autor citou como escopo;
- dual MIT/Apache-2.0 é a licença mais folgada das quatro;
- ativo (2.9k estrelas, push de hoje).

**O que isso NÃO decide:** OpenOCD cobre alvos que o probe-rs não cobre, e
quem já tem um `openocd.cfg` afinado não deve ser forçado a trocar
(`roadmaps/35` §5.1: *"reconhece e usa, em vez de sobrescrever"*). A ponte GDB
remote continua na fila — só não é a **primeira** coisa a construir.

**QEMU é caso à parte e não concorre com os outros três.** Ele não é sonda: é o
que permite exercitar flash e debug **sem placa**, e por isso é o que torna o
gate da frente possível em CI. Sem ele, a frente de embarcados seria a única do
projeto verificada apenas na mão.

**Medido em 2026-09-11, e uma linha do §3 caiu:** a doc web do probe-rs mostra
só o modo `--port`; o binário 0.32.0 é a fonte — `probe-rs dap-server --help`:
*"When omitted, the DAP server communicates over stdin/stdout"*. O §3 estava
certo. E o `qemu-system-arm` 10.2.2 desta máquina traz 18 máquinas Cortex-M
(`lm3s6965evb`, `mps2-an385/386/500/505`, `microbit`, `netduinoplus2`,
`stm32vldiscovery`…): o gate tem onde rodar.

## 5. O que falta medir antes de adotar

Este documento fecha a pergunta *"qual protocolo cada uma fala e sob que
licença?"*. **Não** fecha:

```text
alvos concretos   quais chips o probe-rs suporta HOJE, e quais faltam
                  (a lista dele e' grande e muda por release)
permissao USB     udev rules no Linux: a sonda precisa de regra para
                  aparecer sem root, e "plug and play" morre se o
                  primeiro contato exigir sudo
velocidade        flash de um binario tipico, medido — nao estimado
QEMU no gate      qual maquina/alvo usar para o ciclo caber em CI
```

O item de **udev** é o mais subestimado: é exatamente o tipo de coisa que
transforma "plug and play" em "plug, pesquise no fórum, e play".
