# Integrações (a aba "Plugins") — como o projeto escala

Este é o ponto de entrada para **adicionar uma ferramenta, protocolo, formato ou
biblioteca** à Kinein. Existe porque a fila de candidatos é longa (53 itens
triados em `../../PONTO_ATUAL.md` A5.3) e cada adoção precisa seguir o mesmo
caminho, sem virar improviso.

## O que é (e o que não é)

O nome de produto pode ser **Plugins**, mas a arquitetura registra a natureza
real de cada item. Os nomes internos são `integration` / `capability` /
`adapter`.

```text
NÃO existe extension host, marketplace executável, código de terceiros
carregado em runtime, nem download silencioso.
Uma integração é o core ORQUESTRANDO uma ferramenta madura por um contrato
tipado, com a UI apenas listando, configurando e pedindo ações.
```

Fluxo obrigatório, sem atalho:

```text
QML (apresenta e solicita)
        ↓ CoreClient
kinein-protocol::integration (descriptor, state, action, permission)
        ↓
kinein-core/src/integration/ (registry + policy + health)
        ↓
adapter pequeno do domínio → Job cancelável → ferramenta/protocolo externo
        ↓
eventos tipados → Problems / Tests / Profiler / Trace / Simulation
```

## As quatro fontes que governam uma adoção

| Fonte | Papel |
| --- | --- |
| [`../roadmaps/KINEIN_VECTIS_OPEN_PLUGIN_ADAPTATION_ROADMAP.md`](../roadmaps/KINEIN_VECTIS_OPEN_PLUGIN_ADAPTATION_ROADMAP.md) | **Norte**: modos A–D, gate de auditoria, preferência de licença, política de referência profissional |
| [`../tooling/OPEN_COMPONENT_REGISTRY.json`](../tooling/OPEN_COMPONENT_REGISTRY.json) | **Registro auditável**: pin, licença, telemetria/rede, escopo, verificação |
| [`../adr/`](../adr/) | **Decisão**: por que este componente, alternativas, riscos, rollback |
| `../../PONTO_ATUAL.md` (A5.1–A5.3) | **Ordem**: arquitetura do registry e a sequência de níveis L0–L10 |

## Modos de adoção (resumo operacional)

```text
MODE-A  integração direta do executável/protocolo   ← PREFERIDO
MODE-B  referência de comportamento/UX, código próprio
MODE-C  port seletivo, só com licença compatível + auditoria + ADR
MODE-D  somente estudo (copyleft forte, dependência do host, imaturo)
```

Licenças: MIT > Apache-2.0 > BSD > MPL-2.0 após revisão. GPL/AGPL **não** entram
no core MIT/Apache — só referência.

## Regra que ordena a fila: capacidade antes de ferramenta

Um nível só abre quando o contrato e o serviço do anterior estão comprovados. A
facilidade de chamar uma CLI **não** antecipa sua integração. A tabela completa
L0–L10 está em `../../PONTO_ATUAL.md` A5.2; o resumo:

```text
L0  baseline medida (A3)                    L6  Target/Device/Probe, flash
L1  integration v1 + editor diário          L7  streaming/trace com backpressure
L2  diagnóstico/teste/cobertura comuns      L8  armazenamento + visualização
L3  Project Graph, artefatos, cache         L9  sandbox de pacote, co-simulação
L4  DAP sólido, profiling                   L10 laboratório opt-in
L5  RemoteContext (SSH, path mapping)
```

**Gate de promoção de nível:** ao menos uma integração vertical real, testes de
falha/cancelamento, orçamento medido, configuração reversível e nenhum processo
ou handle órfão. A aba visual não desbloqueia nível; o contrato e o serviço
comprovados desbloqueiam.

### Embarcados sobe de L6 — decisão do autor em 2026-09-03

A ordem acima punha embarcados em **L6**, depois de L5 (RemoteContext/SSH) e
L5.5 (banco). **O autor decidiu que embarcados sobe na fila**, e este parágrafo
é o registro: contrariar decisão registrada exige registro novo, que é como este
repositório muda de regra.

**E a frente é PLUG AND PLAY** (decisão do autor, 2026-09-03): a IDE detecta a
sonda, deduz o alvo e roda build → flash → debug sem o usuário editar arquivo na
mão — e, quando **não** consegue deduzir, diz o que faltou e onde procurou, em
vez de falhar calada. O critério verificável está em
`../roadmaps/35-ambiente-cpp-embarcados-simulacao.md` §5.1.

**Plug and play é a experiência do USUÁRIO, não atalho no gate.** Cada
ferramenta (probe-rs, OpenOCD, pyOCD, QEMU) entra pelo checklist abaixo, com
licença e manutenção verificadas na fonte. A conveniência de quem usa a IDE não
diminui a exigência sobre o que a IDE adota — se diminuísse, "plug and play"
significaria "adotamos sem olhar".

**O que NÃO muda:** o gate de promoção de nível acima continua valendo integral.
Subir na fila muda a ordem, não o critério — a frente só abre depois do
levantamento de licença, manutenção e alvos de cada ferramenta candidata
(`../roadmaps/35-ambiente-cpp-embarcados-simulacao.md` §5), pelo mesmo motivo
que o catálogo de bibliotecas é auditado: recomendar ferramenta é afirmar que
ela serve.

## Checklist para adicionar uma integração nova

```text
[ ] 1. O problema é real e o nível dela já está aberto?
[ ] 2. Gate de auditoria do roadmap: repositório oficial, licença SPDX,
       manutenção, releases, telemetria, rede em runtime, shell, segredos,
       dependências transitivas, compatibilidade Linux, testes, pin possível
[ ] 3. Modo A–D decidido e justificado
[ ] 4. ADR quando a decisão atravessa arquitetura, segurança ou dependências
[ ] 5. Entrada no OPEN_COMPONENT_REGISTRY.json (pin + checksum + escopo +
       verificação)
[ ] 6. Contrato tipado no kinein-protocol; nada de JSON solto
[ ] 7. Adapter pequeno no core; operação longa vira Job cancelável
[ ] 8. UI burra: lista, configura, pede ação — nunca inicia processo
[ ] 9. Testes de comportamento, erro, cancelamento e ausência de órfão
[ ] 10. Gate integral verde + documentação sincronizada
```

Permissões, rede, USB, privilégios e instalação são sempre **visíveis e
confirmáveis**. Ativar/desativar não pode alterar arquivos ou toolchains do
projeto do usuário silenciosamente.

## Levantamentos (candidatas, NAO adotadas)

| Documento | Assunto |
| --- | --- |
| [37-banco-e-observabilidade.md](37-banco-e-observabilidade.md) | **Grafana e TimescaleDB medidos** (2026-09-03): Grafana é **AGPL-3.0**, o que decide a FORMA da integração (API, nunca embutido); TimescaleDB é Apache-2.0 **mais** a Timescale License, que é source-available e **não** OSI |
| [36-ferramentas-de-embarcados.md](36-ferramentas-de-embarcados.md) | **probe-rs, OpenOCD, pyOCD e QEMU medidos** (2026-09-03): licença verificada na fonte, versão, manutenção e — o que decide o desenho — **qual protocolo cada uma fala**. probe-rs é o único com DAP nativo sobre stdin/stdout, que é a forma que o `dap/` já usa |

## Integrações já adotadas

Estado real hoje (detalhe e pins no registry):

| Componente | Modo | Papel | Decisão |
| --- | --- | --- | --- |
| clangd | A | Inteligência semântica C/C++ | — |
| rust-analyzer | A | Inteligência semântica Rust | — |
| Tree-sitter (+ gramáticas C/C++/Rust) | A | Sintaxe incremental local | [ADR-0002](../adr/ADR-0002-tree-sitter-syntax-foundation.md) |
| lldb-dap | A | Debug via DAP | — |
| CMake (File API/Presets) · Cargo | A | Modelo de projeto e build | — |
| Git (CLI) | A | Controle de versão | — |
| notify | A | Mudanças externas no filesystem | [ADR-0001](../adr/ADR-0001-notify-filesystem-watcher.md) |
| alacritty_terminal | A | Emulador VT do terminal integrado | [ADR-0004](../adr/ADR-0004-alacritty-terminal-emulator.md) |
| linuxdeploy (+ plugin Qt) | A | Empacotamento AppImage (build-time) | [ADR-0003](../adr/ADR-0003-linuxdeploy-appimage-packaging.md) |

Próxima integração recomendada: **EditorConfig** (L1), a primeira fatia pequena
depois de fechar a baseline A3.

## Referência profissional é obrigatória

Antes de criar ou alterar uma funcionalidade de IDE, estudar a implementação
oficial e atual pertinente — Code OSS, IntelliJ IDEA Community, Zed, Lapce,
NetBeans — conforme a seção 2.1 do roadmap de adaptação. Registrar revisão,
subsistema, licença/modo, lições e a adaptação no documento do domínio.

Isso serve para extrair invariantes, modos de falha, cancelamento, concorrência,
segurança e estratégia de teste. **Não** autoriza copiar função, classe ou
módulo, traduzir mecanicamente entre linguagens, nem transplantar runtime.

O ADR-0004 é o exemplo canônico: o emulador VT foi adotado inteiro (MODE-A,
mesmo motor do Alacritty/Zed) em vez de continuar mantendo um VT próprio raso —
e o contrato da IDE não mudou por causa disso.
