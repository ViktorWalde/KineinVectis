# 06 — Onde mexer (o mapa por área)

Complementa a tabela de [`../contribuindo.md`](../contribuindo.md) §3 com
o que a Etapa 2 e a Etapa 3 mudaram (2026-09-18/19).

## O repositório em uma olhada

```text
crates/kinein-protocol/   os tipos do IPC (um arquivo por domínio) + PROTOCOL_VERSION
crates/kinein-core/src/   o core: <dominio>/ (lógica) + handlers/<dominio>.rs (fino)
  runtime/, runtime.rs    o laço, defer_work/defer_then/continuations
  lsp/, lang/, index/     language servers, tree-sitter, o índice do projeto
  tests/                  testes de despacho por domínio; scripts/fake_*.py
ui/src/                   a ponte C++ (core_client_<dominio>.cpp), main.cpp (hooks de medição)
ui/qml/                   app/ (composição), ipc/ (roteadores), shell/ (trilho, hosts, atalhos),
                          editor/, git/, datasource/, container/, grafana/, embedded/, components/ (Kv*)
scripts/                  os gates (verificar-*.sh / verificar_*.py), qml-harness/tst_*.qml
DocsPublic/               arquitetura/ (03 = contrato), roadmaps/ (40 = estado), manual.md
DocsPrivate/Codex/        registros datados, evidências, PROMPT-proxima-sessao.md (privado)
```

## Por área

| Quero mudar… | Onde |
| --- | --- |
| Um método/evento do IPC | `kinein-protocol/src/<dominio>.rs` → `arquitetura/03` → `handlers/<dominio>.rs` → `ui/src/core_client_<dominio>.cpp` → `ui/qml/ipc/<Dominio>*Router.qml` |
| O laço do core / algo que trava a UI | `runtime/services.rs` (`defer_work`, `defer_then`), `runtime.rs` (`LoopEvent`), `jobs/` |
| Editor (texto, completion, navegação) | `ui/qml/editor/` (`EditorController` está em débito de catraca — divida antes de crescer), `lang/` e `lsp/` no core |
| A aba Símbolos | `editor/SymbolsController.qml` (filho do `index/IndexController.qml`), `EditorOutlinePanel.qml`, `SymbolResultsList.qml` |
| Git | `ui/qml/git/` (`GitController` ≤ 400; `GitWindow` em pé; `GitViewerPane` no editor), `core/src/git/` |
| O slot esquerdo (explorer/Git) | `shell/ShellController.qml` (`leftWindow`), `shell/ShellLeftWindowHost.qml` |
| O trilho | `shell/SideRail.qml` (ícones em `components/KvIcon.qml` e `KvIconGlyphs.js`) |
| Painel de baixo e abas | `panels/bottom/BottomPanelHost.qml`, `shell/BottomTabBar.qml` |
| Banco de dados | `ui/qml/datasource/` (`DataSourceDiscoveryController` = descobrir/criar/remover), `core/src/datasource/` |
| Containers | `ui/qml/container/` (grade com seleção + barra de ações), `core/src/container/` |
| Embarcados | `ui/qml/embedded/` (painel em abas; `EmbeddedController.tab`), `core/src/{serial,flash,probe,toolchain}` |
| Grafana | `ui/qml/grafana/` (a E3-7 do `44` está desenhada e pendente), `core/src/grafana/` |
| Peças comuns de UI | `ui/qml/components/Kv*.qml` (`KvPanelFrame`, `KvPanelHeader`, `KvVerdict`, `KvDataGrid` com seleção, `KvToggleChip`, `KvButton` com tooltip/danger) |
| Atalhos e menu | `commands/*.rs` (catálogo), `shell/GlobalShortcuts.qml` (`// comando:`), `shell/AppMenuItems.qml` + `ShellHeaderHost.qml` |
| Comandos da paleta / dispatcher | `command/CommandDispatcher.qml` (`<id>=<arg>` só para a medição) |
| Hooks de medição | `ui/src/main.cpp` (screenshot), `app/StartupCommands.qml`, `ui/src/typing_perf_harness.cpp` |
| Configurações persistidas | `core/src/settings.rs` (`SettingsValues` é contrato), `settings/SettingsController.qml` |
| Um gate novo | `scripts/verificar_<x>.py` + `verificar-<x>.sh`, e a linha no `verificar.sh` |
