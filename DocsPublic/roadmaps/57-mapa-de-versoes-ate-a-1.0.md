# 57 — Mapa de versões até a 1.0

> **Classe: PLANO** (`DocsPublic/README.md`). Escrito em 2026-10-01, a pedido do
> autor: "estruturar todas as versões do projeto até a v1.0, de forma
> explícita". Este documento é um **índice com estado**, não um segundo plano.
> O escopo detalhado de cada versão tem **um dono** — o documento citado na
> coluna "Dono do detalhe" —, e aqui só mora o resumo, a ordem, o critério de
> pronto e o que ainda não foi decidido. Se este mapa e o dono divergirem, o
> dono vence e este mapa se corrige no mesmo gesto. **Onde cada versão começa
> no código** (arquivos, mecanismo a estender, primeira fatia) está no
> [`58`](58-onde-cada-versao-comeca-no-codigo.md).

## 0. Como ler: as quatro situações

Cada versão está em exatamente uma situação. A distinção é o ponto deste
documento: **ninguém (pessoa ou IA) pode tratar uma proposta como decisão.**

| Situação | Significa | Quem pode mudar |
| --- | --- | --- |
| **ENCERRADA** | lançada; o registro está no `CHANGELOG.md` e no [`40.7`](40.7-registro-das-entregas.md) | ninguém: é histórico |
| **EM CURSO** | decidida pelo autor e em implementação | o autor, com registro no dono |
| **PLANEJADA** | escopo e ordem decididos pelo autor, com documento dono | o autor, com registro no dono |
| **PROPOSTA** | agrupamento sugerido, **sem decisão do autor** | o autor decide; até lá não se implementa nada a partir dela |

## 1. A linha do tempo

```mermaid
flowchart LR
  v02["0.1–0.2<br/>MVP e teste fechado"]:::done --> v03["0.3.0–0.3.5<br/>série 0.3"]:::done
  v03 --> v036["0.3.6–0.3.9<br/>casca e base visual"]:::now
  v036 --> v04["0.4.x<br/>embarcados"]:::planned
  v04 --> v05["0.5.x<br/>ambiente e capacidades"]:::planned
  v05 --> v06["0.6<br/>motor do editor"]:::planned
  v06 --> v07["0.7<br/>paridade diária"]:::planned
  v07 --> v08["0.8<br/>extensão declarativa"]:::planned
  v08 --> v09["0.9<br/>distribuição e comunidade"]:::planned
  v09 --> v10["1.0<br/>contratos congelados"]:::planned
  classDef done fill:#d9d9d9,stroke:#555
  classDef now fill:#ffe08a,stroke:#b38600,stroke-width:2px
  classDef planned fill:#bfe3ff,stroke:#2a6fa8
  classDef proposed fill:#ffffff,stroke:#888,stroke-dasharray:5 4
```

**Lançamento da 0.3.6–0.3.9 (decisão do autor, 2026-10-02):** as quatro saem
juntas, num pacote só; o detalhe e o que ainda está em aberto (juntar às notas
da 0.4) estão no [`53`](53-arquitetura-executavel-da-0.3.6.md) §11.

**Revisão de continuidade em 2026-10-07:** foco Qt 6.10. A 0.3.9 encerra com
o pente fino como último passo (16 do 59 §2), após os passos menores do Banco
(7–15). Novo AppImage fica fora desse fechamento; o autor considera lançá-lo
após concluir a 0.4.0, ainda sem decisão de publicação (59 §8).
Uso cronometrado (40.7 §7.201) continua entre as provas do passo 16.

Cinza: encerrada. Amarelo: em curso. Azul: planejada. Tracejado: proposta
(nenhuma versão desde 2026-10-01; o que resta proposto é o critério da 1.0, §4).

## 2. Versão a versão

| Versão | Situação | Tema | O que entrega (resumo) | Dono do detalhe | Critério de pronto |
| --- | --- | --- | --- | --- | --- |
| 0.1–0.2 | ENCERRADA | MVP e teste fechado | o núcleo: core Rust, UI Qt/QML, editor, LSP, terminal, build | [`30`](30-caminho-para-o-mvp.md), [`34`](34-depois-do-mvp.md), `CHANGELOG.md` | — |
| 0.3.0–0.3.5 | ENCERRADA | série 0.3 | shell, Remote SSH (escritor único), terminal, editor, Grafana, AppImage | [`47`](47-estrutura-da-v0.3.md), [`48`](48-arquitetura-executavel-da-serie-0.3.md), [`51`](51-plano-fechamento-0.3.5.md) | AppImage 0.3.5 com smokes (40.7 §7.149) |
| 0.3.6 | FEITA NO CHECKOUT (2026-10-03) | limpeza e base | G0 (gates antes do código — feito), terminal mudo, passeio por superfícies, inventário F0, layout versionado | [`53`](53-arquitetura-executavel-da-0.3.6.md) §11, [`49`](49-frontend-0.3.6-e-sequencia-0.5.md) §4 | gate completo, passeio sem aviso no AppImage, telas nas três larguras (53 §11) |
| 0.3.7 | FEITA NO CHECKOUT (2026-10-03) | navegação | trilho por áreas, painel de baixo contextual | 53 §5.4–§5.5 | idem |
| 0.3.8 | FEITA NO CHECKOUT (2026-10-03) | centro e contexto | host de superfície central, header e status com contexto efetivo | 53 §5.6–§5.7 | idem |
| 0.3.9 | EM CURSO (2026-10-07): passos 1–6 feitos; base do Banco e D1a.1–D1a.3 aceitas (40.7 §7.220–§7.237). Atual: passo 7/D1a.4. Restante dividido em passos 8–15; pente fino no 16, último. AppImage adiado | teclado, fluidez e prova | foco, teclado, densidade, modo Foco, carga sob demanda e Banco; provas no Qt 6.10 | [`59`](59-fechamento-da-0.3.9.md) §2/§2.1/§7; 53 §5.8–§5.9 | aceites dos passos 7–16 e medida antes/depois; sem novo pacote como requisito |
| 0.4.0–0.4.4 | PLANEJADA | embarcados | contexto efetivo, diagnóstico ao salvar, cross ponta a ponta, tamanho por símbolo, cache, gravar e depurar com SVD; **PlatformIO como cidadão de tier 1** nas cinco jornadas e **emuladores como alvo** (QEMU, Renode, QEMU da Espressif) — decisões de 2026-10-01 | [`52`](52-arquitetura-executavel-da-0.4.md) §11 | as cinco jornadas J1–J5 provadas (52 §1, §10) |
| 0.5.x | PLANEJADA | ambiente e capacidades | Environment Center, Library por capacidades/providers, resultados normalizados, monitoramento de processo | [`49`](49-frontend-0.3.6-e-sequencia-0.5.md) §6, [`50`](50-biblioteca-e-providers-0.5.md) | por fatia; corte de release pela evidência (49 §6) |
| 0.6 | PLANEJADA | motor do editor | a decisão M5.4 (o `TextEdit` do QtQuick bloqueia split, minimap, multi-cursor) e a migração | [`21`](21-roadmap-de-longo-prazo.md) §M5.4 | ADR da decisão + editor novo sem regressão de latência (régua do 45) |
| 0.7 | PLANEJADA | paridade diária | o restante de M5: navegação pesada, git avançado (conflitos, histórico), multi-cursor sobre o motor novo | 21 §M5.1–§M5.3 | o autor usa a IDE o dia inteiro sem sair para outra (régua do 34, TR1) |
| 0.8 | PLANEJADA | extensão declarativa | LS e adaptadores DAP configuráveis, task runner, tema/atalhos/layout **em dados** | 21 §M6.1–§M6.3, [`47`](47-estrutura-da-v0.3.md) §10.2 | um servidor/adaptador novo entra por configuração, sem código no core |
| 0.9 | PLANEJADA | distribuição e comunidade | CI pública, documentação pública, diagnóstico de falha sem telemetria, processo de release | 21 §M7 | "outra pessoa instala e contribui sem mim" (21 §M7) |
| 1.0 | PLANEJADA | contratos congelados | protocolo IPC e formatos persistidos versionados com migração; nada quebra sem versão major | este documento §4 | §4 — **ainda PROPOSTA** |

### 2.1 O que o código já entrega dos marcos M5–M7 (medido em 2026-10-01)

Os marcos do [`21`](21-roadmap-de-longo-prazo.md) não marcam estado; isto foi
medido nos métodos IPC roteados pelo core:

```text
M5.1 refatoracao por LSP     EXISTE   lsp.rename, lsp.codeActions, lsp.applyCodeAction,
                                      lsp.workspaceEdit.apply/cancel
M5.2 git avancado            PARCIAL  pull, push, stash, branchCreate, checkout, blame;
                                      sem fluxo de conflito
M5.3 navegacao pesada        PARCIAL  definition, references, workspaceSymbols,
                                      switchSourceHeader
M5.4 motor do editor         ABERTO   "a MAIOR decisao tecnica pendente" (21 §M5.4)
M6.1/M6.2 LS e DAP config.   ABERTO
M6.3 task runner             ABERTO   nenhum metodo task.*
M7.1 empacotamento           EXISTE   AppImage portavel com smokes (packaging/appimage)
M7.2 CI publica              AUSENTE  nao ha .github/ no repositorio
```

## 3. Pedidos do autor e para onde foram

Os três pedidos de 2026-10-01 já têm dono:

1. **PlatformIO como cidadão de tier 1** → **0.4** (decisão do autor,
   2026-10-01). O que existe e o que falta por jornada, medido no código, está
   no [`58`](58-onde-cada-versao-comeca-no-codigo.md) §4.4; o requisito está no
   [`52`](52-arquitetura-executavel-da-0.4.md) §5.10.
2. **Emuladores na IDE** → **0.4** (decisão do autor, 2026-10-01): QEMU (já no
   gate), Renode, QEMU da Espressif — como **alvo** onde o firmware roda,
   pelo `debugServer` do kit. **Não é a simulação física/matemática que saiu do
   produto** (40 §5); a matriz "jornada × emulador × NÃO PROVADO" é o 52 §10.1.
3. **Direção visual da 0.3.6–0.3.9** → 53 §13.1, com a linha de base medida
   no 58 §4.3; a task dedicada de UX/HUD a detalha.

## 4. Proposta de critério para a 1.0

**PROPOSTA, sem decisão do autor.** A 1.0 é a promessa de que um contrato não
quebra sem aviso:

- o protocolo IPC tem versão (já tem, `0.147.0` em 2026-10-02) e a 1.0 congela a
  major: método removido ou mudado só numa 2.0;
- todo arquivo que o usuário guarda (`schemas/`) tem versão e migração provada;
- a 0.9 entregou CI pública rodando `scripts/verificar.sh --estrito` num
  ambiente declarado;
- nenhuma decisão aberta de §5 bloqueia o uso diário.

## 5. O que vale em TODAS as versões (não se reabre)

Não está copiado aqui — está no dono, e é lá que se lê:

- as camadas e os anti-padrões: [`ARCHITECTURE.md`](../arquitetura/ARCHITECTURE.md) §2, §8;
- as decisões do produto que não se reabrem: [`40`](40-estado-e-continuidade.md) §5
  (IA fora da IDE, simulação fora do produto, Python nativo, licenças proibidas…);
- 0.4 em diante: sem Lua antes nem depois da 1.0, alvo Linux nativo, o modelo
  semântico detecta deriva e não resolve versão: [`47`](47-estrutura-da-v0.3.md) §10.2.

## 6. Decisões deste mapa

| # | Decisão | Situação | Onde está |
| --- | --- | --- | --- |
| 1 | o agrupamento da 0.6 à 1.0 (§2) | **decidida (2026-10-01):** aceito como proposto; as cinco versões passam a PLANEJADA | este documento |
| 2 | o critério da 1.0 (§4) | **aberta** | este documento |
| 3 | PlatformIO tier 1 e emuladores | **decidida (2026-10-01):** os dois na 0.4 | 52 §5.10, §10.1; 58 §4.4 |
| 4 | versão mínima do gdb | **decidida (2026-10-01):** vale o ≥ 16; abaixo, degradação explicada (protocolo 0.145.0) | [`contribuindo/04`](../contribuindo/04-os-gates-que-dizem-nao.md) |
| 5 | qual qmllint vale | **decidida (2026-10-01):** vale o ≥ 6.5; no 6.4, NÃO PROVADO | `contribuindo/04` |
