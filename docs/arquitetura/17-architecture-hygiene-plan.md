# 17 — Plano de Higiene Arquitetural Sem Dívida Nova

> **Status:** concluído para o estado atual em 2026-07-06; os guardrails
> abaixo continuam obrigatórios para impedir regressão.
> **Objetivo:** eliminar pontos conhecidos de concentração antes de novas
> features grandes. Pequena concentração conhecida hoje vira refatoração grande
> amanhã; portanto, não há "dívida aceitável" nesta fase.

## Regra da fase

Enquanto este plano estivesse aberto, ficaram congeladas features como Git, AI
Bridge, debug avançado, run configurations avançadas, Project Health e Settings
completos. A fase terminou depois da remoção das concentrações conhecidas; a
partir daqui, qualquer feature nova deve respeitar os mesmos limites antes de
ser considerada pronta.

Critério central:

```text
UI visual -> controller/store -> CoreClient facade -> handler IPC interno -> Rust core
```

Nenhuma etapa pode criar caminho paralelo a esse fluxo.

## Critérios de saída

A fase terminou porque todos os itens abaixo ficaram verdadeiros:

- `Main.qml` é apenas composition root: janela, instâncias e wiring.
- `Main.qml` não contém `ListModel`, `Connections`, timers, estado de domínio
  ou helpers de domínio.
- Controllers QML grandes foram quebrados antes de virarem god controllers.
- `CoreClient` continua sendo a única fachada QML, mas sua implementação C++
  está dividida por responsabilidade interna.
- Eventos IPC novos ou existentes passam por roteadores por domínio.
- Componentes visuais só recebem `property` e emitem `signal`.
- Docs canônicos registram as regras para impedir regressão.
- Gate rápido, builds debug/release e smokes offscreen passam.

## Resultado 2026-07-06 — ⚠ OS NÚMEROS ABAIXO NÃO VALEM MAIS

> **Medido em 2026-07-17: dois deles regrediram 2–3x e um foi refeito.**
>
> ```text
> Main.qml                336 -> 270    (refeito: AppDomains, 0686213)
> ShellWorkspaceHost.qml  248 -> 576    2.3x — REGREDIU
> EditorController.qml    318 -> 1070   3.4x — REGREDIU, e' o maior debito da UI
> ```
>
> Este bloco e' **registro de 2026-07-06**, nao o estado de hoje. O `Status` no
> topo diz "concluido para o estado atual" e isso deixou de ser verdade dez dias
> depois — exatamente o que a `ARCHITECTURE.md` §1.1 documenta sobre si mesma.
> **A fonte viva do tamanho de cada arquivo e' `scripts/arquitetura-baseline.txt`,
> mantido pela catraca; nao este documento.** Os guardrails de responsabilidade
> abaixo continuam valendo — o que apodreceu foi o inventario, nao a regra.

- `Main.qml`: **336 linhas**, composition root puro. Não contém `ListModel`,
  `Connections`, `Shortcut`, `Timer`, helper de domínio nem componente visual
  pesado embutido.
- `ui/qml/shell/ShellWorkspaceHost.qml`: **248 linhas**, host visual central
  do workspace; agrega rail, explorer, editor, painel inferior e assistente
  via `property`/`signal` e não acessa `CoreClient` diretamente.
- `EditorController.qml`: **318 linhas**, fachada do domínio editor; documentos,
  texto e completion foram separados em `EditorDocumentController.qml`,
  `EditorTextController.qml` e `EditorCompletionController.qml`.
- `CoreClient`: fachada QML única preservada. Implementação C++ dividida em
  `core_client_process.cpp`, `core_client_requests.cpp`,
  `core_client_dispatch.cpp`, `core_client_state.cpp` e
  `core_client_log.cpp`.
- `ui/CMakeLists.txt`: registra os novos QML por subpasta e preserva aliases
  estáveis. O prefixo QML usado pelo runtime foi declarado explicitamente e a
  política Qt para `qmldir` extra foi tratada quando disponível.
- Validação feita: `cmake --build --preset dev-local`, smoke offscreen debug,
  `cmake --build --preset dev-local-release`, smoke offscreen release, smoke
  offscreen via `scripts/kinein-vectis` e `scripts/verificar.sh --rapido`
  verde.

## Sequência de execução

1. **Preservar o estado atual em blocos lógicos.**
   Separar commits quando o usuário pedir commit/push, ou pelo menos manter o
   working tree auditável por grupos claros: QML, C++/Qt, docs e tooling.

2. **[feito] Esvaziar o restante do `Main.qml`.**
   Extrair atalhos globais, estado visual de shell e helpers restantes para
   componentes/controllers de `ui/qml/shell/`. Meta: `Main.qml` sem função que
   conheça domínio e preferencialmente abaixo de 500 linhas.

3. **[feito] Impedir `EditorController.qml` de virar o próximo god controller.**
   Separar abas/documentos, LSP, completion, hover, usages/rename e semantic
   tokens em subcontrollers ou stores do domínio editor. O controller principal
   deve coordenar subcontrollers, não concentrar todas as regras.

4. **[feito] Refatorar `CoreClient` internamente.**
   Preservar a API pública QML, mas dividir transporte, requests, dispatch,
   workspace/fs, jobs/build/test/quality, LSP, search, run/terminal, tools e
   logs em arquivos C++ por responsabilidade.

5. **[feito] Registrar guardrails nos docs.**
   Atualizar `docs/arquitetura/ARCHITECTURE.md`, `docs/arquitetura/15-engineering-debt-and-refactor.md`
   e `docsprivate/ContextoIA.md` sempre que uma concentração for removida ou uma regra
   anti-regressão mudar.

6. **[feito] Validar e atualizar o launcher local.**
   Rodar build debug/release, smoke offscreen debug/release, smoke pelo launcher
   e `scripts/verificar.sh --rapido`. Rodar o gate completo quando a mudança
   tocar core/protocolo ou antes de release.

## Limites objetivos

Estes limites não são estética; são gatilhos de split:

- arquivo QML visual acima de 300 linhas exige revisão;
- controller/store QML acima de 400 linhas exige split por subresponsabilidade;
- arquivo C++ acima de 500 linhas exige split ou justificativa registrada;
- arquivo Rust fora de testes acima de 400-500 linhas segue a regra de split de
  `docs/arquitetura/ARCHITECTURE.md`;
- qualquer arquivo que combine renderização, estado e IPC deve ser quebrado
  mesmo que esteja abaixo do limite de linhas.

## Não fazer daqui para frente

- Não adicionar feature funcional grande que viole os limites deste documento.
- Não criar `CoreClient2` nem cliente QML paralelo.
- Não mover regra de negócio para QML para reduzir C++/Rust.
- Não criar componente visual que chame ferramenta externa, filesystem ou core
  diretamente.
- Não aceitar "só um helper pequeno" em `Main.qml` se ele tiver domínio.
- Não deixar doc novo fora de `docs/README.md`.
