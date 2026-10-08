# 01 — Mapa de módulos: quem fala com quem, por quê e como

> **GERADO** por `scripts/module_map.py` a partir do código — não edite à mão.
> O porquê de cada aresta e de cada contexto mora nas tabelas `EDGES` e `CONTEXTS`
> do script; o resto (controllers, métodos, handlers, módulos, ciclos) é **medido**.
> O gate `scripts/module_map.py --check` reprova se este documento divergir do código
> ou se surgir um ciclo novo entre módulos do core.

## Como ler

- **Nível 1** mostra as pastas do repositório e as únicas formas pelas quais elas se
  comunicam. Tudo o que não está ali **não acontece** (a QML não fala com o core; o
  core não conhece a UI).
- **O caminho de um pedido** é o mesmo em todos os contextos: o que muda é o domínio.
- **Nível 2** é o grafo interno do `kinein-core`, domínio a domínio.
- **Contextos** repetem o caminho para cada área da IDE, com os arquivos reais de cada
  etapa e o que cada um diz de si (o comentário de cabeçalho dele).

## Nível 1 — as pastas do repositório

```mermaid
flowchart LR
  n_ui_qml["<b>ui/qml</b><br/>a interface: telas, controllers e roteadores de IPC (QML)"]
  n_ui_src["<b>ui/src</b><br/>a ponte C++/Qt: CoreClient (processo, JSON-RPC, despacho), janela, CLI"]
  n_crates_kinein_core["<b>crates/kinein-core</b><br/>o core Rust: todo o trabalho (projeto, build, LSP, git, debug, terminal...)"]
  n_crates_kinein_protocol["<b>crates/kinein-protocol</b><br/>os tipos de mensagem do IPC (contrato unico, serde)"]
  n_crates_kinein_config["<b>crates/kinein-config</b><br/>leitura e validacao das configuracoes do usuario"]
  n_crates_kinein_cli["<b>crates/kinein-cli</b><br/>gera UM pedido JSON-RPC por invocacao, para scripts e smokes"]
  n_schemas["<b>schemas</b><br/>o formato dos arquivos persistidos (JSON Schema)"]
  n_scripts["<b>scripts</b><br/>os gates e as provas; ninguem importa estes arquivos"]
  n_packaging["<b>packaging</b><br/>o AppImage: junta ui + core num pacote portavel"]
  n_DocsPublic["<b>DocsPublic</b><br/>a documentacao; o 03 e' lido por gate como lista canonica do IPC"]
  n_ui_qml -->|"coreClient (QML_ELEMENT)"| n_ui_src
  n_ui_src -->|"processo + JSON-RPC (stdio)"| n_crates_kinein_core
  n_crates_kinein_core -->|"Cargo"| n_crates_kinein_protocol
  n_crates_kinein_core -->|"Cargo"| n_crates_kinein_config
  n_crates_kinein_cli -->|"Cargo"| n_crates_kinein_protocol
  n_crates_kinein_core -->|"contrato do arquivo"| n_schemas
  n_scripts -->|"le o 03 (lista canonica)"| n_DocsPublic
  n_scripts -->|"sobe o core real"| n_crates_kinein_core
  n_packaging -->|"compila a UI release"| n_ui_src
  n_packaging -->|"compila o core release"| n_crates_kinein_core
```

| De | Para | Como | Por quê |
| --- | --- | --- | --- |
| `ui/qml` | `ui/src` | propriedade `coreClient` (QML_ELEMENT) chamada pelos roteadores de `ui/qml/ipc` | a QML nunca fala processo nem JSON: so' o CoreClient sabe que o core existe |
| `ui/src` | `crates/kinein-core` | processo filho (QProcess); JSON-RPC 2.0 por linha em stdin/stdout | a UI nao linka Rust: um crash do core nao derruba a janela, e o core reinicia |
| `crates/kinein-core` | `crates/kinein-protocol` | dependencia Cargo | o formato de cada mensagem tem um dono so', compartilhado com a CLI |
| `crates/kinein-core` | `crates/kinein-config` | dependencia Cargo | ler e validar configuracao nao e' trabalho de cada dominio |
| `crates/kinein-cli` | `crates/kinein-protocol` | dependencia Cargo | a CLI monta pedidos com os MESMOS tipos que o core le |
| `crates/kinein-core` | `schemas` | o formato persistido e' documentado pelo schema | arquivo que o usuario guarda precisa de contrato fora do codigo |
| `scripts` | `DocsPublic` | o verificar_fiacao_ipc.py le o 03-protocolo-ipc.md | o documento do protocolo nao pode divergir dos metodos roteados |
| `scripts` | `crates/kinein-core` | os gates sobem o binario do core e falam JSON-RPC com ele | prova contra o core REAL, nao contra um falso |
| `packaging` | `ui/src` | o empacotador compila a UI release e a poe no AppDir | o pacote e' o que o usuario roda; tem de ser o mesmo codigo |
| `packaging` | `crates/kinein-core` | o empacotador compila o core release e o poe ao lado da UI | UI e core viajam juntos e na mesma versao |

## O caminho de um pedido (todos os contextos)

```mermaid
sequenceDiagram
  participant C as ui/qml/(domínio)/XController
  participant R as ui/qml/ipc/XRequestRouter
  participant K as ui/src/CoreClient
  participant H as kinein-core handlers/(x).rs
  participant M as kinein-core (módulos)
  participant E as ui/qml/ipc/XEventRouter
  C->>R: sinal xRequested (intenção)
  R->>K: coreClient.metodo(args)
  K->>H: JSON-RPC 'dominio.metodo' por stdin
  H->>M: chamada Rust
  H-->>K: resposta ou event.* por stdout
  K-->>E: sinal C++ (core_client_dispatch*.cpp)
  E-->>C: atualiza o estado do controller
```

Por que assim: o **controller** guarda estado e intenção e não sabe que existe IPC;
o **RequestRouter** é o único que chama o `CoreClient` naquele domínio (uma guarda
cross-domain, quando existe, mora nele e só nele); o **EventRouter** é o espelho da
volta. Trocar o transporte muda o `CoreClient`, e nenhuma tela.

Medido: 183 métodos IPC roteados pelo core.

## Nível 2 — os domínios do `kinein-core`

```mermaid
flowchart LR
  n_core_build[build]
  n_core_cargo[cargo]
  n_core_cdb[cdb]
  n_core_cmake[cmake]
  n_core_commands[commands]
  n_core_configaction[configaction]:::cycle
  n_core_container[container]
  n_core_coverage[coverage]
  n_core_dap[dap]
  n_core_datasource[datasource]
  n_core_db[db]
  n_core_flash[flash]
  n_core_format[format]
  n_core_fsops[fsops]
  n_core_fswatch[fswatch]
  n_core_git[git]
  n_core_grafana[grafana]
  n_core_handlers[handlers]
  n_core_index[index]
  n_core_jobs[jobs]
  n_core_lang[lang]
  n_core_library[library]:::cycle
  n_core_lsp[lsp]
  n_core_outcome[outcome]
  n_core_owned_child[owned_child]
  n_core_probe[probe]
  n_core_process[process]
  n_core_project[project]
  n_core_python[python]:::cycle
  n_core_remote[remote]
  n_core_rpc[rpc]
  n_core_run[run]:::cycle
  n_core_runconfig[runconfig]
  n_core_runtime[runtime]
  n_core_serial[serial]
  n_core_settings[settings]
  n_core_setup[setup]
  n_core_size[size]
  n_core_stderr_tail[stderr_tail]
  n_core_terminal[terminal]
  n_core_test[test]
  n_core_toolchain[toolchain]
  n_core_tools[tools]
  n_core_workspace[workspace]
  n_core_build --> n_core_cdb
  n_core_build --> n_core_cmake
  n_core_build --> n_core_process
  n_core_build --> n_core_toolchain
  n_core_cargo --> n_core_toolchain
  n_core_cmake --> n_core_toolchain
  n_core_configaction --> n_core_cdb
  n_core_configaction --> n_core_cmake
  n_core_configaction --> n_core_fsops
  n_core_configaction --> n_core_library
  n_core_configaction --> n_core_runconfig
  n_core_container --> n_core_tools
  n_core_coverage --> n_core_process
  n_core_coverage --> n_core_python
  n_core_dap --> n_core_lsp
  n_core_dap --> n_core_python
  n_core_dap --> n_core_run
  n_core_dap --> n_core_stderr_tail
  n_core_datasource --> n_core_container
  n_core_datasource --> n_core_db
  n_core_datasource --> n_core_fsops
  n_core_datasource --> n_core_jobs
  n_core_datasource --> n_core_owned_child
  n_core_datasource --> n_core_stderr_tail
  n_core_datasource --> n_core_tools
  n_core_flash --> n_core_build
  n_core_fswatch --> n_core_lsp
  n_core_grafana --> n_core_datasource
  n_core_handlers --> n_core_build
  n_core_handlers --> n_core_cdb
  n_core_handlers --> n_core_cmake
  n_core_handlers --> n_core_configaction
  n_core_handlers --> n_core_container
  n_core_handlers --> n_core_coverage
  n_core_handlers --> n_core_dap
  n_core_handlers --> n_core_datasource
  n_core_handlers --> n_core_flash
  n_core_handlers --> n_core_format
  n_core_handlers --> n_core_fsops
  n_core_handlers --> n_core_grafana
  n_core_handlers --> n_core_index
  n_core_handlers --> n_core_jobs
  n_core_handlers --> n_core_library
  n_core_handlers --> n_core_lsp
  n_core_handlers --> n_core_probe
  n_core_handlers --> n_core_process
  n_core_handlers --> n_core_project
  n_core_handlers --> n_core_python
  n_core_handlers --> n_core_remote
  n_core_handlers --> n_core_rpc
  n_core_handlers --> n_core_run
  n_core_handlers --> n_core_runconfig
  n_core_handlers --> n_core_serial
  n_core_handlers --> n_core_settings
  n_core_handlers --> n_core_setup
  n_core_handlers --> n_core_terminal
  n_core_handlers --> n_core_toolchain
  n_core_handlers --> n_core_tools
  n_core_index --> n_core_cdb
  n_core_index --> n_core_cmake
  n_core_index --> n_core_fswatch
  n_core_index --> n_core_lang
  n_core_index --> n_core_python
  n_core_jobs --> n_core_lsp
  n_core_library --> n_core_cmake
  n_core_library --> n_core_configaction
  n_core_lsp --> n_core_cmake
  n_core_lsp --> n_core_fsops
  n_core_lsp --> n_core_stderr_tail
  n_core_owned_child --> n_core_stderr_tail
  n_core_project --> n_core_tools
  n_core_python --> n_core_run
  n_core_rpc --> n_core_dap
  n_core_run --> n_core_python
  n_core_runtime --> n_core_rpc
  n_core_runtime --> n_core_settings
  n_core_serial --> n_core_build
  n_core_serial --> n_core_toolchain
  n_core_setup --> n_core_tools
  n_core_terminal --> n_core_lsp
  n_core_test --> n_core_build
  n_core_test --> n_core_process
  n_core_test --> n_core_python
  n_core_toolchain --> n_core_tools
  n_core_workspace --> n_core_fsops
  n_core_workspace --> n_core_python
  classDef cycle stroke:#d33,stroke-width:3px
```

44 módulos, 86 dependências (`crate::<módulo>` fora de testes). Em vermelho, os que estão num ciclo.

### Ciclos

- **configaction ↔ library** — dívida medida em 2026-10-01: a biblioteca propoe acoes, e as acoes aplicam bibliotecas
- **python ↔ run** — dívida medida em 2026-10-01: o run pede o interpretador ao python, e o python executa pelo run

### O que cada domínio diz de si

| Domínio | Depende de | Cabeçalho |
| --- | --- | --- |
| `build` | cdb, cmake, process, toolchain | Build execution with streamed output and structured diagnostics. |
| `cargo` | toolchain | Servico Cargo: |
| `cdb` | — | Descoberta e diagnóstico da compilation database do C/C++. |
| `cmake` | toolchain | Servico CMake: |
| `commands` | — | Command descriptors advertised to the UI (command palette, menus, shortcuts). |
| `configaction` | cdb, cmake, fsops, library, runconfig | Configuration Actions: |
| `container` | tools | Containers como dominio NATIVO: |
| `coverage` | process, python | Cobertura de linhas dos testes (D8 do roadmaps/41, P5 do 40 §4.1, 2026-09-17), com o LCOV como lingua comum. |
| `dap` | lsp, python, run, stderr_tail | Subsistema de debug: |
| `datasource` | container, db, fsops, jobs, owned_child, stderr_tail, tools | Fontes de dados: |
| `db` | — | Persistência local em SQLite — rede de segurança de dados (DocsPublic/seguranca/23). |
| `flash` | build | Gravar como CONFIGURACAO DE EXECUCAO (E4 do integracoes/38 §6; decisao do autor em 2026-09-11: |
| `format` | — | Buffer formatting by orchestrating the project's own formatters. |
| `fsops` | — | File system operations confined to the open workspace root. |
| `fswatch` | lsp | Debounced, workspace-confined observation of external file-system changes. |
| `git` | — | Git orquestrado sobre o binario git. |
| `grafana` | datasource | Observabilidade: |
| `handlers` | build, cdb, cmake, configaction, container, coverage, dap, datasource, flash, format, fsops, grafana, index, jobs, library, lsp, probe, process, project, python, remote, rpc, run, runconfig, serial, settings, setup, terminal, toolchain, tools | Handlers for the build / quality / test runners, all async cancelable jobs. |
| `index` | cdb, cmake, fswatch, lang, python | O indice proprio do projeto INTEIRO: |
| `jobs` | lsp | Job system: |
| `lang` | — | Incremental local syntax intelligence backed by Tree-sitter. |
| `library` | cmake, configaction | Bibliotecas C/C++ curadas: |
| `lsp` | cmake, fsops, stderr_tail | Subsistema LSP: |
| `outcome` | — | O desfecho de um pedido (RequestOutcome) e o erro dos lacos de IO (CoreError). |
| `owned_child` | stderr_tail | A base prova o que diz: |
| `probe` | — | Sondas de debug conectadas: |
| `process` | — | Synchronous line streaming for child processes. |
| `project` | tools | O MODELO do projeto embarcado — pilar 0 do roadmaps/42 (2026-09-12). |
| `python` | run | O dominio python: |
| `remote` | — | O alvo Linux por SSH como recurso do projeto (P6 do roadmaps/42, fatia 1, 2026-09-17): |
| `rpc` | dap | JSON-RPC helpers shared by the request handlers: |
| `run` | python | O que "Executar" roda, e como se mostra. |
| `runconfig` | — | Run configurations por workspace (.kinein/runconfigs.json). |
| `runtime` | rpc, settings | Habilitacao dos servicos externos e configuracao dos servidores LSP do Core. |
| `serial` | build, toolchain | Portas seriais USB: |
| `settings` | — | Settings persistidos em dois níveis (settings.*). |
| `setup` | tools | Como instalar o que falta — passo a passo OFICIAL, para a distro detectada. |
| `size` | — | O tamanho de um ELF: |
| `stderr_tail` | — | O stderr de um processo filho de LONGA VIDA (adaptador DAP, servidor de debug, servidor LSP): |
| `terminal` | lsp | Terminal profissional: |
| `test` | build, process, python | Test execution with streamed, per-case results. |
| `toolchain` | tools | Toolchain: |
| `tools` | — | External tool detection. |
| `workspace` | fsops, python | Workspace opening, project kind detection, and metadata persistence. |

## Cobertura: todo método IPC tem um lugar

Dos 183 métodos roteados pelo core, 153 seguem o caminho padrão
e estão num contexto abaixo. Os outros 30 estão aqui,
nomeados, para nada ficar invisível:

**Chamados direto da QML, sem RequestRouter** (fora do padrão — dívida medida em
2026-10-01: o composition root fala com o `CoreClient` no lugar do roteador do domínio):

| Método | Quem chama |
| --- | --- |
| `build.run` | `ui/qml/app/AppDomains.qml` |
| `cargo.check` | `ui/qml/command/CommandDispatcher.qml` |
| `cargo.metadata` | `ui/qml/Main.qml`, `ui/qml/command/CommandDispatcher.qml` |
| `coverage.run` | `ui/qml/app/AppDomains.qml` |
| `environment.scan` | `ui/qml/Main.qml` |
| `format.capabilities` | `ui/qml/Main.qml` |
| `lsp.restart` | `ui/qml/command/CommandDispatcher.qml`, `ui/qml/shell/ShellHeaderHost.qml` |
| `quality.run` | `ui/qml/app/AppDomains.qml` |
| `run.capabilities` | `ui/qml/Main.qml` |
| `settings.get` | `ui/qml/Main.qml`, `ui/qml/app/AppDomains.qml` |
| `settings.set` | `ui/qml/app/AppDomains.qml` |
| `test.discover` | `ui/qml/app/AppDomains.qml` |
| `test.run` | `ui/qml/app/AppDomains.qml` |
| `tools.detect` | `ui/qml/Main.qml`, `ui/qml/app/AppDomains.qml`, `ui/qml/command/CommandDispatcher.qml`, `ui/qml/shell/ShellHeaderHost.qml` |
| `workspace.close` | `ui/qml/Main.qml`, `ui/qml/command/CommandDispatcher.qml`, `ui/qml/shell/ShellHeaderHost.qml` |
| `workspace.createFolder` | `ui/qml/Main.qml` |
| `workspace.createProject` | `ui/qml/Main.qml` |
| `workspace.recent.clear` | `ui/qml/app/AppDomains.qml` |
| `workspace.recent.list` | `ui/qml/app/AppDomains.qml` |
| `workspace.recent.pin` | `ui/qml/app/AppDomains.qml` |
| `workspace.recent.remove` | `ui/qml/app/AppDomains.qml` |

**Enviados só pela ponte C++** (ciclo de vida do processo, respostas encadeadas): `cmake.presets.list`, `cmake.status`, `core.ping`, `remote.status`, `runConfig.list`, `tools.status`.

**Sem cliente na UI** (a CLI, os gates ou nenhum): `core.shutdown`, `job.list`, `workspace.status`.

## Contextos

Cada contexto é o caminho de um pedido aplicado a uma área da IDE, com os arquivos
reais de cada etapa. Ida em linha cheia, volta (resposta e eventos) tracejada.

### Workspace e arvore do projeto

Abrir uma pasta e mexer em arquivos e' o unico caminho de escrita em disco fora do editor: passa pelo core para que a vigilancia de arquivos, os rascunhos e o indice vejam a mesma mudanca.

```mermaid
flowchart LR
  subgraph QML["ui/qml"]
    n_ui_qml_search_SearchEverywhereController_qml["SearchEverywhereController"]
    n_ui_qml_workspace_ProjectHealthController_qml["ProjectHealthController"]
    n_ui_qml_workspace_RecentWorkspacesController_qml["RecentWorkspacesController"]
    n_ui_qml_workspace_WorkspaceController_qml["WorkspaceController"]
    n_ui_qml_ipc_ProjectTreeRequestRouter_qml["ipc/ProjectTreeRequestRouter"]
    n_ui_qml_ipc_WorkspaceEventRouter_qml["ipc/WorkspaceEventRouter"]
  end
  subgraph CPP["ui/src"]
    n_ui_src_core_client_dispatch_cpp["core_client_dispatch.cpp"]
    n_ui_src_core_client_dispatch_cmake_cpp["core_client_dispatch_cmake.cpp"]
    n_ui_src_core_client_log_cpp["core_client_log.cpp"]
    n_ui_src_core_client_notifications_cpp["core_client_notifications.cpp"]
    n_ui_src_core_client_requests_cpp["core_client_requests.cpp"]
  end
  subgraph IPC["JSON-RPC"]
    n_ipc_cmake(["cmake.* · 1"])
    n_ipc_command(["command.* · 1"])
    n_ipc_fs(["fs.* · 9"])
    n_ipc_workspace(["workspace.* · 1"])
  end
  subgraph CORE["crates/kinein-core"]
    n_crates_kinein_core_src_handlers_cmake_rs["handlers/cmake.rs"]
    n_crates_kinein_core_src_handlers_fs_rs["handlers/fs.rs"]
    n_crates_kinein_core_src_lib_rs["lib.rs"]
    n_core_cdb[cdb]
    n_core_jobs[jobs]
    n_core_rpc[rpc]
    n_core_settings[settings]
    n_core_toolchain[toolchain]
    n_core_tools[tools]
  end
  n_tools_workspace[/"sistema de arquivos · lixeira do desktop"/]
  n_ui_qml_ipc_ProjectTreeRequestRouter_qml --> n_ui_src_core_client_requests_cpp
  n_ui_qml_ipc_WorkspaceEventRouter_qml -.-> n_ui_qml_search_SearchEverywhereController_qml
  n_ui_qml_ipc_WorkspaceEventRouter_qml -.-> n_ui_qml_workspace_ProjectHealthController_qml
  n_ui_qml_ipc_WorkspaceEventRouter_qml -.-> n_ui_qml_workspace_RecentWorkspacesController_qml
  n_ui_qml_ipc_WorkspaceEventRouter_qml -.-> n_ui_qml_workspace_WorkspaceController_qml
  n_ui_qml_ipc_WorkspaceEventRouter_qml --> n_ui_src_core_client_requests_cpp
  n_ui_src_core_client_dispatch_cpp -.-> n_ui_qml_ipc_WorkspaceEventRouter_qml
  n_ui_src_core_client_dispatch_cmake_cpp -.-> n_ui_qml_ipc_WorkspaceEventRouter_qml
  n_ui_src_core_client_log_cpp -.-> n_ui_qml_ipc_WorkspaceEventRouter_qml
  n_ui_src_core_client_notifications_cpp -.-> n_ui_qml_ipc_WorkspaceEventRouter_qml
  n_ui_src_core_client_requests_cpp --> n_ipc_cmake
  n_ui_src_core_client_requests_cpp --> n_ipc_command
  n_ui_src_core_client_requests_cpp --> n_ipc_fs
  n_ui_src_core_client_requests_cpp --> n_ipc_workspace
  n_ipc_cmake --> n_crates_kinein_core_src_handlers_cmake_rs
  n_crates_kinein_core_src_handlers_cmake_rs --> n_core_cdb
  n_crates_kinein_core_src_handlers_cmake_rs --> n_core_rpc
  n_crates_kinein_core_src_handlers_cmake_rs --> n_core_settings
  n_crates_kinein_core_src_handlers_cmake_rs --> n_core_toolchain
  n_ipc_fs --> n_crates_kinein_core_src_handlers_fs_rs
  n_crates_kinein_core_src_handlers_fs_rs --> n_core_jobs
  n_crates_kinein_core_src_handlers_fs_rs --> n_core_rpc
  n_ipc_command --> n_crates_kinein_core_src_lib_rs
  n_ipc_workspace --> n_crates_kinein_core_src_lib_rs
  n_crates_kinein_core_src_lib_rs --> n_core_tools
  CORE -.->|processos| n_tools_workspace
```

| Etapa | Arquivo | O que ele diz de si |
| --- | --- | --- |
| controller | `ui/qml/search/SearchEverywhereController.qml` | SEARCH EVERYWHERE: |
| controller | `ui/qml/workspace/ProjectHealthController.qml` |  |
| controller | `ui/qml/workspace/RecentWorkspacesController.qml` | A lista de recentes como a tela inicial a mostra (Etapa 2 F7, 2026-09-18): |
| controller | `ui/qml/workspace/WorkspaceController.qml` |  |
| roteador | `ui/qml/ipc/ProjectTreeRequestRouter.qml` | Leva ao core o que a arvore de projeto pede: |
| roteador | `ui/qml/ipc/WorkspaceEventRouter.qml` |  |
| ponte C++ | `ui/src/core_client_dispatch.cpp` |  |
| ponte C++ | `ui/src/core_client_dispatch_cmake.cpp` | Dispatch do dominio CMake/build (ARCHITECTURE.md §5). |
| ponte C++ | `ui/src/core_client_log.cpp` |  |
| ponte C++ | `ui/src/core_client_notifications.cpp` | O que o core manda SEM SER PERGUNTADO: |
| ponte C++ | `ui/src/core_client_requests.cpp` |  |
| handler Rust | `crates/kinein-core/src/handlers/cmake.rs` | Handlers for cmake.* requests (impl Core): |
| handler Rust | `crates/kinein-core/src/handlers/fs.rs` | Filesystem request router and mutation handlers. |
| handler Rust | `crates/kinein-core/src/lib.rs` | Rust core for Kinein Vectis. |

Métodos IPC (12): `cmake.configure`, `command.list`, `fs.copy`, `fs.createDirectory`, `fs.createFile`, `fs.delete`, `fs.list`, `fs.read`, `fs.rename`, `fs.transferBatch`, `fs.trash`, `workspace.browse`.

### Editor e inteligencia de linguagem (LSP)

O editor nao conhece servidor de linguagem: pede ao core, que sobe um servidor por linguagem, traduz LSP para o protocolo da IDE e devolve diagnosticos e simbolos.

```mermaid
flowchart LR
  subgraph QML["ui/qml"]
    n_ui_qml_editor_EditorAppendController_qml["EditorAppendController"]
    n_ui_qml_editor_EditorController_qml["EditorController"]
    n_ui_qml_workspace_LspStatusController_qml["LspStatusController"]
    n_ui_qml_ipc_EditorRequestRouter_qml["ipc/EditorRequestRouter"]
    n_ui_qml_ipc_EditorEventRouter_qml["ipc/EditorEventRouter"]
  end
  subgraph CPP["ui/src"]
    n_ui_src_core_client_dispatch_cpp["core_client_dispatch.cpp"]
    n_ui_src_core_client_dispatch_lsp_cpp["core_client_dispatch_lsp.cpp"]
    n_ui_src_core_client_notifications_cpp["core_client_notifications.cpp"]
    n_ui_src_core_client_requests_cpp["core_client_requests.cpp"]
  end
  subgraph IPC["JSON-RPC"]
    n_ipc_draft(["draft.* · 2"])
    n_ipc_format(["format.* · 1"])
    n_ipc_fs(["fs.* · 3"])
    n_ipc_lsp(["lsp.* · 12"])
    n_ipc_syntaxTree(["syntaxTree.* · 2"])
    n_ipc_workspace(["workspace.* · 1"])
  end
  subgraph CORE["crates/kinein-core"]
    n_crates_kinein_core_src_handlers_draft_rs["handlers/draft.rs"]
    n_crates_kinein_core_src_handlers_format_rs["handlers/format.rs"]
    n_crates_kinein_core_src_handlers_fs_rs["handlers/fs.rs"]
    n_crates_kinein_core_src_handlers_lsp_mod_rs["handlers/lsp/mod.rs"]
    n_crates_kinein_core_src_handlers_syntax_rs["handlers/syntax.rs"]
    n_crates_kinein_core_src_lib_rs["lib.rs"]
    n_core_format[format]
    n_core_jobs[jobs]
    n_core_lsp[lsp]
    n_core_rpc[rpc]
    n_core_tools[tools]
  end
  n_tools_editor[/"clangd · rust-analyzer · pyright/basedpyright"/]
  n_ui_qml_editor_EditorController_qml --> n_ui_qml_ipc_EditorRequestRouter_qml
  n_ui_qml_ipc_EditorEventRouter_qml -.-> n_ui_qml_editor_EditorAppendController_qml
  n_ui_qml_ipc_EditorEventRouter_qml -.-> n_ui_qml_editor_EditorController_qml
  n_ui_qml_ipc_EditorEventRouter_qml -.-> n_ui_qml_workspace_LspStatusController_qml
  n_ui_qml_ipc_EditorRequestRouter_qml --> n_ui_src_core_client_requests_cpp
  n_ui_src_core_client_dispatch_cpp -.-> n_ui_qml_ipc_EditorEventRouter_qml
  n_ui_src_core_client_dispatch_lsp_cpp -.-> n_ui_qml_ipc_EditorEventRouter_qml
  n_ui_src_core_client_notifications_cpp -.-> n_ui_qml_ipc_EditorEventRouter_qml
  n_ui_src_core_client_requests_cpp --> n_ipc_draft
  n_ui_src_core_client_requests_cpp --> n_ipc_format
  n_ui_src_core_client_requests_cpp --> n_ipc_fs
  n_ui_src_core_client_requests_cpp --> n_ipc_lsp
  n_ui_src_core_client_requests_cpp --> n_ipc_syntaxTree
  n_ui_src_core_client_requests_cpp --> n_ipc_workspace
  n_ipc_draft --> n_crates_kinein_core_src_handlers_draft_rs
  n_crates_kinein_core_src_handlers_draft_rs --> n_core_rpc
  n_ipc_format --> n_crates_kinein_core_src_handlers_format_rs
  n_crates_kinein_core_src_handlers_format_rs --> n_core_format
  n_crates_kinein_core_src_handlers_format_rs --> n_core_rpc
  n_ipc_fs --> n_crates_kinein_core_src_handlers_fs_rs
  n_crates_kinein_core_src_handlers_fs_rs --> n_core_jobs
  n_crates_kinein_core_src_handlers_fs_rs --> n_core_rpc
  n_ipc_lsp --> n_crates_kinein_core_src_handlers_lsp_mod_rs
  n_crates_kinein_core_src_handlers_lsp_mod_rs --> n_core_lsp
  n_ipc_syntaxTree --> n_crates_kinein_core_src_handlers_syntax_rs
  n_ipc_workspace --> n_crates_kinein_core_src_lib_rs
  n_crates_kinein_core_src_lib_rs --> n_core_tools
  CORE -.->|processos| n_tools_editor
```

| Etapa | Arquivo | O que ele diz de si |
| --- | --- | --- |
| controller | `ui/qml/editor/EditorAppendController.qml` | Append a prepared instruction using native editing, after opening its file. |
| controller | `ui/qml/editor/EditorController.qml` |  |
| controller | `ui/qml/workspace/LspStatusController.qml` | O ESTADO DOS SERVIDORES DE LINGUAGEM para a barra de status (Etapa 2, F2). |
| roteador | `ui/qml/ipc/EditorRequestRouter.qml` | Espelho do EditorEventRouter. |
| roteador | `ui/qml/ipc/EditorEventRouter.qml` |  |
| ponte C++ | `ui/src/core_client_dispatch.cpp` |  |
| ponte C++ | `ui/src/core_client_dispatch_lsp.cpp` | Dispatch do dominio LSP (ARCHITECTURE.md §5): |
| ponte C++ | `ui/src/core_client_notifications.cpp` | O que o core manda SEM SER PERGUNTADO: |
| ponte C++ | `ui/src/core_client_requests.cpp` |  |
| handler Rust | `crates/kinein-core/src/handlers/draft.rs` | Handlers de draft.* (autosave/rascunhos) — rede de segurança (DocsPublic/seguranca/23). |
| handler Rust | `crates/kinein-core/src/handlers/format.rs` | Handlers for format.* requests (impl Core): |
| handler Rust | `crates/kinein-core/src/handlers/fs.rs` | Filesystem request router and mutation handlers. |
| handler Rust | `crates/kinein-core/src/handlers/lsp/mod.rs` | Handlers for lsp.* requests (impl Core). |
| handler Rust | `crates/kinein-core/src/handlers/syntax.rs` | Handlers for local incremental syntax intelligence (syntaxTree.*). |
| handler Rust | `crates/kinein-core/src/lib.rs` | Rust core for Kinein Vectis. |

Métodos IPC (21): `draft.clear`, `draft.save`, `format.text`, `fs.read`, `fs.readExternal`, `fs.write`, `lsp.applyCodeAction`, `lsp.codeActions`, `lsp.completion`, `lsp.definition`, `lsp.didChange`, `lsp.hover`, `lsp.references`, `lsp.rename`, `lsp.semanticTokens`, `lsp.switchSourceHeader`, `lsp.workspaceEdit.apply`, `lsp.workspaceEdit.cancel`, `syntaxTree.indent`, `syntaxTree.update`, `workspace.saveSession`.

### Busca, indice e simbolos

Buscar e substituir mexe em disco; o indice e' do core para responder sem reler a arvore. O roteador da busca conhece o editor: substituir com arquivo sujo perderia edicao.

```mermaid
flowchart LR
  subgraph QML["ui/qml"]
    n_ui_qml_editor_EditorController_qml["EditorController"]
    n_ui_qml_index_IndexController_qml["IndexController"]
    n_ui_qml_search_SearchController_qml["SearchController"]
    n_ui_qml_search_SearchEverywhereController_qml["SearchEverywhereController"]
    n_ui_qml_ipc_SearchRequestRouter_qml["ipc/SearchRequestRouter"]
    n_ui_qml_ipc_SearchEventRouter_qml["ipc/SearchEventRouter"]
    n_ui_qml_ipc_IndexRequestRouter_qml["ipc/IndexRequestRouter"]
    n_ui_qml_ipc_IndexEventRouter_qml["ipc/IndexEventRouter"]
  end
  subgraph CPP["ui/src"]
    n_ui_src_core_client_dispatch_cpp["core_client_dispatch.cpp"]
    n_ui_src_core_client_dispatch_lsp_cpp["core_client_dispatch_lsp.cpp"]
    n_ui_src_core_client_index_cpp["core_client_index.cpp"]
    n_ui_src_core_client_notifications_cpp["core_client_notifications.cpp"]
    n_ui_src_core_client_requests_cpp["core_client_requests.cpp"]
  end
  subgraph IPC["JSON-RPC"]
    n_ipc_command(["command.* · 1"])
    n_ipc_fs(["fs.* · 4"])
    n_ipc_index(["index.* · 3"])
    n_ipc_lsp(["lsp.* · 2"])
  end
  subgraph CORE["crates/kinein-core"]
    n_crates_kinein_core_src_handlers_fs_rs["handlers/fs.rs"]
    n_crates_kinein_core_src_handlers_index_rs["handlers/index.rs"]
    n_crates_kinein_core_src_handlers_lsp_mod_rs["handlers/lsp/mod.rs"]
    n_crates_kinein_core_src_lib_rs["lib.rs"]
    n_core_index[index]
    n_core_jobs[jobs]
    n_core_lsp[lsp]
    n_core_python[python]
    n_core_rpc[rpc]
    n_core_toolchain[toolchain]
    n_core_tools[tools]
  end
  n_tools_search[/"ripgrep · tree-sitter (embutido)"/]
  n_ui_qml_editor_EditorController_qml --> n_ui_qml_ipc_IndexRequestRouter_qml
  n_ui_qml_editor_EditorController_qml --> n_ui_qml_ipc_SearchRequestRouter_qml
  n_ui_qml_index_IndexController_qml --> n_ui_qml_ipc_IndexRequestRouter_qml
  n_ui_qml_ipc_IndexEventRouter_qml -.-> n_ui_qml_index_IndexController_qml
  n_ui_qml_ipc_IndexEventRouter_qml -.-> n_ui_qml_search_SearchEverywhereController_qml
  n_ui_qml_ipc_IndexRequestRouter_qml --> n_ui_src_core_client_index_cpp
  n_ui_qml_ipc_IndexRequestRouter_qml --> n_ui_src_core_client_requests_cpp
  n_ui_qml_ipc_SearchEventRouter_qml -.-> n_ui_qml_search_SearchController_qml
  n_ui_qml_ipc_SearchEventRouter_qml -.-> n_ui_qml_search_SearchEverywhereController_qml
  n_ui_qml_ipc_SearchRequestRouter_qml --> n_ui_src_core_client_requests_cpp
  n_ui_qml_search_SearchController_qml --> n_ui_qml_ipc_SearchRequestRouter_qml
  n_ui_qml_search_SearchEverywhereController_qml --> n_ui_qml_ipc_IndexRequestRouter_qml
  n_ui_qml_search_SearchEverywhereController_qml --> n_ui_qml_ipc_SearchRequestRouter_qml
  n_ui_src_core_client_dispatch_cpp -.-> n_ui_qml_ipc_SearchEventRouter_qml
  n_ui_src_core_client_dispatch_lsp_cpp -.-> n_ui_qml_ipc_IndexEventRouter_qml
  n_ui_src_core_client_dispatch_lsp_cpp -.-> n_ui_qml_ipc_SearchEventRouter_qml
  n_ui_src_core_client_index_cpp --> n_ipc_index
  n_ui_src_core_client_index_cpp -.-> n_ui_qml_ipc_IndexEventRouter_qml
  n_ui_src_core_client_notifications_cpp -.-> n_ui_qml_ipc_IndexEventRouter_qml
  n_ui_src_core_client_requests_cpp --> n_ipc_command
  n_ui_src_core_client_requests_cpp --> n_ipc_fs
  n_ui_src_core_client_requests_cpp --> n_ipc_lsp
  n_ipc_fs --> n_crates_kinein_core_src_handlers_fs_rs
  n_crates_kinein_core_src_handlers_fs_rs --> n_core_jobs
  n_crates_kinein_core_src_handlers_fs_rs --> n_core_rpc
  n_ipc_index --> n_crates_kinein_core_src_handlers_index_rs
  n_crates_kinein_core_src_handlers_index_rs --> n_core_index
  n_crates_kinein_core_src_handlers_index_rs --> n_core_jobs
  n_crates_kinein_core_src_handlers_index_rs --> n_core_python
  n_crates_kinein_core_src_handlers_index_rs --> n_core_rpc
  n_crates_kinein_core_src_handlers_index_rs --> n_core_toolchain
  n_ipc_lsp --> n_crates_kinein_core_src_handlers_lsp_mod_rs
  n_crates_kinein_core_src_handlers_lsp_mod_rs --> n_core_lsp
  n_ipc_command --> n_crates_kinein_core_src_lib_rs
  n_crates_kinein_core_src_lib_rs --> n_core_tools
  CORE -.->|processos| n_tools_search
```

| Etapa | Arquivo | O que ele diz de si |
| --- | --- | --- |
| controller | `ui/qml/editor/EditorController.qml` |  |
| controller | `ui/qml/index/IndexController.qml` | Estado do indice do projeto INTEIRO (pilar 0 do roadmaps/42, decisao do autor em 2026-09-12: |
| controller | `ui/qml/search/SearchController.qml` | BUSCA E SUBSTITUIÇÃO NO PROJETO — o painel de baixo. |
| controller | `ui/qml/search/SearchEverywhereController.qml` | SEARCH EVERYWHERE: |
| roteador | `ui/qml/ipc/SearchRequestRouter.qml` | Espelho do SearchEventRouter: |
| roteador | `ui/qml/ipc/SearchEventRouter.qml` | Resultados de busca, substituicao e Search Everywhere -> os DOIS donos. |
| roteador | `ui/qml/ipc/IndexRequestRouter.qml` | Espelho do IndexEventRouter: |
| roteador | `ui/qml/ipc/IndexEventRouter.qml` | O que o core responde/emite de index.* -> IndexController (os totais) e SearchEverywhereController (os simbolos do #nome, antes do LSP responder). |
| ponte C++ | `ui/src/core_client_dispatch.cpp` |  |
| ponte C++ | `ui/src/core_client_dispatch_lsp.cpp` | Dispatch do dominio LSP (ARCHITECTURE.md §5): |
| ponte C++ | `ui/src/core_client_index.cpp` | O indice do projeto INTEIRO no lado da UI (pilar 0 do roadmaps/42, decisao do autor em 2026-09-12): |
| ponte C++ | `ui/src/core_client_notifications.cpp` | O que o core manda SEM SER PERGUNTADO: |
| ponte C++ | `ui/src/core_client_requests.cpp` |  |
| handler Rust | `crates/kinein-core/src/handlers/fs.rs` | Filesystem request router and mutation handlers. |
| handler Rust | `crates/kinein-core/src/handlers/index.rs` | Handler for index.* requests (impl Core) — o indice do projeto inteiro (pilar 0 do roadmaps/42, decisao do autor em 2026-09-12). |
| handler Rust | `crates/kinein-core/src/handlers/lsp/mod.rs` | Handlers for lsp.* requests (impl Core). |
| handler Rust | `crates/kinein-core/src/lib.rs` | Rust core for Kinein Vectis. |

Métodos IPC (10): `command.list`, `fs.findFiles`, `fs.read`, `fs.replace`, `fs.search`, `index.context`, `index.status`, `index.symbols`, `lsp.documentSymbols`, `lsp.workspaceSymbols`.

### Git

O core executa o git do sistema. O roteador e' a unica aresta do git para o editor: checkout, branch, pull e stash nao rodam com arquivo modificado.

```mermaid
flowchart LR
  subgraph QML["ui/qml"]
    n_ui_qml_editor_EditorController_qml["EditorController"]
    n_ui_qml_git_GitController_qml["GitController"]
    n_ui_qml_ipc_GitRequestRouter_qml["ipc/GitRequestRouter"]
    n_ui_qml_ipc_GitEventRouter_qml["ipc/GitEventRouter"]
  end
  subgraph CPP["ui/src"]
    n_ui_src_core_client_dispatch_cpp["core_client_dispatch.cpp"]
    n_ui_src_core_client_dispatch_debug_cpp["core_client_dispatch_debug.cpp"]
    n_ui_src_core_client_notifications_cpp["core_client_notifications.cpp"]
    n_ui_src_core_client_requests_git_cpp["core_client_requests_git.cpp"]
  end
  subgraph IPC["JSON-RPC"]
    n_ipc_git(["git.* · 15"])
  end
  subgraph CORE["crates/kinein-core"]
    n_crates_kinein_core_src_handlers_git_rs["handlers/git.rs"]
    n_core_jobs[jobs]
    n_core_rpc[rpc]
  end
  n_tools_git[/"git"/]
  n_ui_qml_editor_EditorController_qml --> n_ui_qml_ipc_GitRequestRouter_qml
  n_ui_qml_git_GitController_qml --> n_ui_qml_ipc_GitRequestRouter_qml
  n_ui_qml_ipc_GitEventRouter_qml -.-> n_ui_qml_git_GitController_qml
  n_ui_qml_ipc_GitRequestRouter_qml --> n_ui_src_core_client_requests_git_cpp
  n_ui_src_core_client_dispatch_cpp -.-> n_ui_qml_ipc_GitEventRouter_qml
  n_ui_src_core_client_dispatch_debug_cpp -.-> n_ui_qml_ipc_GitEventRouter_qml
  n_ui_src_core_client_notifications_cpp -.-> n_ui_qml_ipc_GitEventRouter_qml
  n_ui_src_core_client_requests_git_cpp --> n_ipc_git
  n_ipc_git --> n_crates_kinein_core_src_handlers_git_rs
  n_crates_kinein_core_src_handlers_git_rs --> n_core_jobs
  n_crates_kinein_core_src_handlers_git_rs --> n_core_rpc
  CORE -.->|processos| n_tools_git
```

| Etapa | Arquivo | O que ele diz de si |
| --- | --- | --- |
| controller | `ui/qml/editor/EditorController.qml` |  |
| controller | `ui/qml/git/GitController.qml` | Estado git da UI (fatia M3.1): |
| roteador | `ui/qml/ipc/GitRequestRouter.qml` | Espelho do GitEventRouter: |
| roteador | `ui/qml/ipc/GitEventRouter.qml` | Resultado do git.status → GitController, e os momentos que a UI já conhece como "o status pode ter mudado" (save e fs-ops) → refresh. |
| ponte C++ | `ui/src/core_client_dispatch.cpp` |  |
| ponte C++ | `ui/src/core_client_dispatch_debug.cpp` | Dispatch do dominio DEBUG (ARCHITECTURE.md §5): |
| ponte C++ | `ui/src/core_client_notifications.cpp` | O que o core manda SEM SER PERGUNTADO: |
| ponte C++ | `ui/src/core_client_requests_git.cpp` | Dominio GIT no lado da UI: |
| handler Rust | `crates/kinein-core/src/handlers/git.rs` | Handlers for git.* requests (impl Core). |

Métodos IPC (15): `git.blame`, `git.branchCreate`, `git.branches`, `git.checkout`, `git.commit`, `git.commitDiff`, `git.discard`, `git.fileDiff`, `git.log`, `git.pull`, `git.push`, `git.stage`, `git.stash`, `git.status`, `git.unstage`.

### Executar, terminal, build e testes

Toda execucao e' uma sessao de terminal do core (PTY), e todo trabalho longo e' um Job: a UI so' recebe o render e os eventos, e cancelar e' um pedido como outro qualquer.

```mermaid
flowchart LR
  subgraph QML["ui/qml"]
    n_ui_qml_coverage_CoverageController_qml["CoverageController"]
    n_ui_qml_diagnostics_DiagnosticsController_qml["DiagnosticsController"]
    n_ui_qml_jobs_JobsController_qml["JobsController"]
    n_ui_qml_runtime_RunConfigController_qml["RunConfigController"]
    n_ui_qml_runtime_RuntimeController_qml["RuntimeController"]
    n_ui_qml_ipc_RuntimeRequestRouter_qml["ipc/RuntimeRequestRouter"]
    n_ui_qml_ipc_RuntimeEventRouter_qml["ipc/RuntimeEventRouter"]
    n_ui_qml_ipc_JobsEventRouter_qml["ipc/JobsEventRouter"]
    n_ui_qml_ipc_CoverageRequestRouter_qml["ipc/CoverageRequestRouter"]
    n_ui_qml_ipc_CoverageEventRouter_qml["ipc/CoverageEventRouter"]
  end
  subgraph CPP["ui/src"]
    n_ui_src_core_client_coverage_cpp["core_client_coverage.cpp"]
    n_ui_src_core_client_dispatch_cpp["core_client_dispatch.cpp"]
    n_ui_src_core_client_dispatch_cmake_cpp["core_client_dispatch_cmake.cpp"]
    n_ui_src_core_client_dispatch_lsp_cpp["core_client_dispatch_lsp.cpp"]
    n_ui_src_core_client_log_cpp["core_client_log.cpp"]
    n_ui_src_core_client_notifications_cpp["core_client_notifications.cpp"]
    n_ui_src_core_client_requests_cpp["core_client_requests.cpp"]
    n_ui_src_core_client_requests_run_cpp["core_client_requests_run.cpp"]
  end
  subgraph IPC["JSON-RPC"]
    n_ipc_coverage(["coverage.* · 1"])
    n_ipc_run(["run.* · 3"])
    n_ipc_runConfig(["runConfig.* · 3"])
    n_ipc_terminal(["terminal.* · 9"])
  end
  subgraph CORE["crates/kinein-core"]
    n_crates_kinein_core_src_handlers_coverage_rs["handlers/coverage.rs"]
    n_crates_kinein_core_src_handlers_run_rs["handlers/run.rs"]
    n_crates_kinein_core_src_handlers_runconfig_rs["handlers/runconfig.rs"]
    n_crates_kinein_core_src_handlers_terminal_rs["handlers/terminal.rs"]
    n_core_build[build]
    n_core_coverage[coverage]
    n_core_fsops[fsops]
    n_core_jobs[jobs]
    n_core_project[project]
    n_core_python[python]
    n_core_rpc[rpc]
    n_core_run[run]
    n_core_runconfig[runconfig]
    n_core_terminal[terminal]
    n_core_toolchain[toolchain]
  end
  n_tools_runtime[/"cargo · cmake/ninja · ctest · pytest · sh -lc"/]
  n_ui_qml_coverage_CoverageController_qml --> n_ui_qml_ipc_CoverageRequestRouter_qml
  n_ui_qml_ipc_CoverageEventRouter_qml -.-> n_ui_qml_coverage_CoverageController_qml
  n_ui_qml_ipc_CoverageRequestRouter_qml --> n_ui_src_core_client_coverage_cpp
  n_ui_qml_ipc_JobsEventRouter_qml -.-> n_ui_qml_diagnostics_DiagnosticsController_qml
  n_ui_qml_ipc_JobsEventRouter_qml -.-> n_ui_qml_jobs_JobsController_qml
  n_ui_qml_ipc_RuntimeEventRouter_qml -.-> n_ui_qml_runtime_RunConfigController_qml
  n_ui_qml_ipc_RuntimeEventRouter_qml -.-> n_ui_qml_runtime_RuntimeController_qml
  n_ui_qml_ipc_RuntimeRequestRouter_qml --> n_ui_src_core_client_requests_cpp
  n_ui_qml_ipc_RuntimeRequestRouter_qml --> n_ui_src_core_client_requests_run_cpp
  n_ui_qml_runtime_RunConfigController_qml --> n_ui_qml_ipc_RuntimeRequestRouter_qml
  n_ui_qml_runtime_RuntimeController_qml --> n_ui_qml_ipc_RuntimeRequestRouter_qml
  n_ui_src_core_client_coverage_cpp --> n_ipc_coverage
  n_ui_src_core_client_coverage_cpp -.-> n_ui_qml_ipc_CoverageEventRouter_qml
  n_ui_src_core_client_dispatch_cpp -.-> n_ui_qml_ipc_RuntimeEventRouter_qml
  n_ui_src_core_client_dispatch_cmake_cpp -.-> n_ui_qml_ipc_RuntimeEventRouter_qml
  n_ui_src_core_client_dispatch_lsp_cpp -.-> n_ui_qml_ipc_JobsEventRouter_qml
  n_ui_src_core_client_log_cpp -.-> n_ui_qml_ipc_RuntimeEventRouter_qml
  n_ui_src_core_client_notifications_cpp -.-> n_ui_qml_ipc_CoverageEventRouter_qml
  n_ui_src_core_client_notifications_cpp -.-> n_ui_qml_ipc_JobsEventRouter_qml
  n_ui_src_core_client_notifications_cpp -.-> n_ui_qml_ipc_RuntimeEventRouter_qml
  n_ui_src_core_client_requests_cpp --> n_ipc_terminal
  n_ui_src_core_client_requests_run_cpp --> n_ipc_run
  n_ui_src_core_client_requests_run_cpp --> n_ipc_runConfig
  n_ui_src_core_client_requests_run_cpp -.-> n_ui_qml_ipc_RuntimeEventRouter_qml
  n_ipc_coverage --> n_crates_kinein_core_src_handlers_coverage_rs
  n_crates_kinein_core_src_handlers_coverage_rs --> n_core_coverage
  n_crates_kinein_core_src_handlers_coverage_rs --> n_core_jobs
  n_crates_kinein_core_src_handlers_coverage_rs --> n_core_rpc
  n_crates_kinein_core_src_handlers_coverage_rs --> n_core_toolchain
  n_ipc_run --> n_crates_kinein_core_src_handlers_run_rs
  n_crates_kinein_core_src_handlers_run_rs --> n_core_fsops
  n_crates_kinein_core_src_handlers_run_rs --> n_core_project
  n_crates_kinein_core_src_handlers_run_rs --> n_core_python
  n_crates_kinein_core_src_handlers_run_rs --> n_core_rpc
  n_crates_kinein_core_src_handlers_run_rs --> n_core_run
  n_crates_kinein_core_src_handlers_run_rs --> n_core_runconfig
  n_ipc_runConfig --> n_crates_kinein_core_src_handlers_runconfig_rs
  n_crates_kinein_core_src_handlers_runconfig_rs --> n_core_build
  n_crates_kinein_core_src_handlers_runconfig_rs --> n_core_rpc
  n_ipc_terminal --> n_crates_kinein_core_src_handlers_terminal_rs
  n_crates_kinein_core_src_handlers_terminal_rs --> n_core_rpc
  n_crates_kinein_core_src_handlers_terminal_rs --> n_core_terminal
  CORE -.->|processos| n_tools_runtime
```

| Etapa | Arquivo | O que ele diz de si |
| --- | --- | --- |
| controller | `ui/qml/coverage/CoverageController.qml` | A COBERTURA dos testes na tela (D8 do roadmaps/41, 2026-09-17): |
| controller | `ui/qml/diagnostics/DiagnosticsController.qml` | Store de diagnósticos por arquivo (fatia T6). |
| controller | `ui/qml/jobs/JobsController.qml` |  |
| controller | `ui/qml/runtime/RunConfigController.qml` | Configuracoes de execucao salvas: |
| controller | `ui/qml/runtime/RuntimeController.qml` |  |
| roteador | `ui/qml/ipc/RuntimeRequestRouter.qml` | Espelho do RuntimeEventRouter: |
| roteador | `ui/qml/ipc/RuntimeEventRouter.qml` |  |
| roteador | `ui/qml/ipc/JobsEventRouter.qml` |  |
| roteador | `ui/qml/ipc/CoverageRequestRouter.qml` | Espelho do CoverageEventRouter: |
| roteador | `ui/qml/ipc/CoverageEventRouter.qml` | O que o core responde/emite de coverage.* -> CoverageController. |
| ponte C++ | `ui/src/core_client_coverage.cpp` | A COBERTURA dos testes no lado da UI (D8 do roadmaps/41, 2026-09-17): |
| ponte C++ | `ui/src/core_client_dispatch.cpp` |  |
| ponte C++ | `ui/src/core_client_dispatch_cmake.cpp` | Dispatch do dominio CMake/build (ARCHITECTURE.md §5). |
| ponte C++ | `ui/src/core_client_dispatch_lsp.cpp` | Dispatch do dominio LSP (ARCHITECTURE.md §5): |
| ponte C++ | `ui/src/core_client_log.cpp` |  |
| ponte C++ | `ui/src/core_client_notifications.cpp` | O que o core manda SEM SER PERGUNTADO: |
| ponte C++ | `ui/src/core_client_requests.cpp` |  |
| ponte C++ | `ui/src/core_client_requests_run.cpp` | Dominio RUN no lado da UI: |
| handler Rust | `crates/kinein-core/src/handlers/coverage.rs` | Handlers de coverage.* (impl Core). |
| handler Rust | `crates/kinein-core/src/handlers/run.rs` | Handlers for run.* requests (impl Core). |
| handler Rust | `crates/kinein-core/src/handlers/runconfig.rs` | Handlers for runConfig.* requests (impl Core): |
| handler Rust | `crates/kinein-core/src/handlers/terminal.rs` | Handlers for terminal.* requests (impl Core). |

Métodos IPC (16): `coverage.lines`, `run.script`, `run.start`, `run.stop`, `runConfig.delete`, `runConfig.save`, `runConfig.setActive`, `terminal.clearScrollback`, `terminal.close`, `terminal.copySelection`, `terminal.input`, `terminal.mouse`, `terminal.open`, `terminal.resize`, `terminal.scroll`, `terminal.selectAll`.

### Depuracao (DAP)

O core e' o cliente DAP: sobe o adaptador, guarda a sessao e manda para a UI frames, variaveis e paradas ja' no formato da IDE.

```mermaid
flowchart LR
  subgraph QML["ui/qml"]
    n_ui_qml_debug_DebugController_qml["DebugController"]
    n_ui_qml_ipc_DebugRequestRouter_qml["ipc/DebugRequestRouter"]
    n_ui_qml_ipc_DebugEventRouter_qml["ipc/DebugEventRouter"]
  end
  subgraph CPP["ui/src"]
    n_ui_src_core_client_dispatch_cpp["core_client_dispatch.cpp"]
    n_ui_src_core_client_dispatch_debug_cpp["core_client_dispatch_debug.cpp"]
    n_ui_src_core_client_notifications_cpp["core_client_notifications.cpp"]
    n_ui_src_core_client_requests_debug_cpp["core_client_requests_debug.cpp"]
  end
  subgraph IPC["JSON-RPC"]
    n_ipc_debug(["debug.* · 14"])
  end
  subgraph CORE["crates/kinein-core"]
    n_crates_kinein_core_src_handlers_debug_rs["handlers/debug.rs"]
    n_core_dap[dap]
    n_core_python[python]
    n_core_rpc[rpc]
    n_core_toolchain[toolchain]
  end
  n_tools_debug[/"gdb -i dap · lldb-dap · debugpy"/]
  n_ui_qml_debug_DebugController_qml --> n_ui_qml_ipc_DebugRequestRouter_qml
  n_ui_qml_ipc_DebugEventRouter_qml -.-> n_ui_qml_debug_DebugController_qml
  n_ui_qml_ipc_DebugRequestRouter_qml --> n_ui_src_core_client_requests_debug_cpp
  n_ui_src_core_client_dispatch_cpp -.-> n_ui_qml_ipc_DebugEventRouter_qml
  n_ui_src_core_client_dispatch_debug_cpp -.-> n_ui_qml_ipc_DebugEventRouter_qml
  n_ui_src_core_client_notifications_cpp -.-> n_ui_qml_ipc_DebugEventRouter_qml
  n_ui_src_core_client_requests_debug_cpp --> n_ipc_debug
  n_ipc_debug --> n_crates_kinein_core_src_handlers_debug_rs
  n_crates_kinein_core_src_handlers_debug_rs --> n_core_dap
  n_crates_kinein_core_src_handlers_debug_rs --> n_core_python
  n_crates_kinein_core_src_handlers_debug_rs --> n_core_rpc
  n_crates_kinein_core_src_handlers_debug_rs --> n_core_toolchain
  CORE -.->|processos| n_tools_debug
```

| Etapa | Arquivo | O que ele diz de si |
| --- | --- | --- |
| controller | `ui/qml/debug/DebugController.qml` | Estado da sessao de debug e dos breakpoints na UI (fatia M2.5b). |
| roteador | `ui/qml/ipc/DebugRequestRouter.qml` | Espelho do DebugEventRouter: |
| roteador | `ui/qml/ipc/DebugEventRouter.qml` | Roteia eventos de debug do CoreClient para o DebugController. |
| ponte C++ | `ui/src/core_client_dispatch.cpp` |  |
| ponte C++ | `ui/src/core_client_dispatch_debug.cpp` | Dispatch do dominio DEBUG (ARCHITECTURE.md §5): |
| ponte C++ | `ui/src/core_client_notifications.cpp` | O que o core manda SEM SER PERGUNTADO: |
| ponte C++ | `ui/src/core_client_requests_debug.cpp` | Dominio de DEBUG no lado da UI: |
| handler Rust | `crates/kinein-core/src/handlers/debug.rs` | Handlers for debug.* requests (impl Core). |

Métodos IPC (14): `debug.continue`, `debug.disassemble`, `debug.evaluate`, `debug.next`, `debug.pause`, `debug.readMemory`, `debug.scopes`, `debug.setBreakpoints`, `debug.stackTrace`, `debug.start`, `debug.stepIn`, `debug.stepOut`, `debug.stop`, `debug.variables`.

### Embarcados: sonda, serial, gravacao

Gravar e monitorar passam pelo mesmo caminho de Executar (configuracao de execucao): a IDE compoe a linha da ferramenta oficial e nunca grava sem pedido.

```mermaid
flowchart LR
  subgraph QML["ui/qml"]
    n_ui_qml_embedded_EmbeddedController_qml["EmbeddedController"]
    n_ui_qml_runtime_RuntimeController_qml["RuntimeController"]
    n_ui_qml_ipc_EmbeddedRequestRouter_qml["ipc/EmbeddedRequestRouter"]
    n_ui_qml_ipc_EmbeddedEventRouter_qml["ipc/EmbeddedEventRouter"]
  end
  subgraph CPP["ui/src"]
    n_ui_src_core_client_dispatch_cpp["core_client_dispatch.cpp"]
    n_ui_src_core_client_dispatch_cmake_cpp["core_client_dispatch_cmake.cpp"]
    n_ui_src_core_client_notifications_cpp["core_client_notifications.cpp"]
    n_ui_src_core_client_probe_cpp["core_client_probe.cpp"]
    n_ui_src_core_client_requests_run_cpp["core_client_requests_run.cpp"]
  end
  subgraph IPC["JSON-RPC"]
    n_ipc_build(["build.* · 1"])
    n_ipc_probe(["probe.* · 1"])
    n_ipc_project(["project.* · 1"])
    n_ipc_runConfig(["runConfig.* · 1"])
    n_ipc_serial(["serial.* · 5"])
  end
  subgraph CORE["crates/kinein-core"]
    n_crates_kinein_core_src_handlers_probe_rs["handlers/probe.rs"]
    n_crates_kinein_core_src_handlers_project_rs["handlers/project.rs"]
    n_crates_kinein_core_src_handlers_runconfig_rs["handlers/runconfig.rs"]
    n_crates_kinein_core_src_handlers_serial_rs["handlers/serial.rs"]
    n_crates_kinein_core_src_lib_rs["lib.rs"]
    n_core_build[build]
    n_core_dap[dap]
    n_core_probe[probe]
    n_core_project[project]
    n_core_rpc[rpc]
    n_core_serial[serial]
    n_core_toolchain[toolchain]
    n_core_tools[tools]
  end
  n_tools_embedded[/"esptool · OpenOCD · probe-rs · mpremote · tio/picocom · QEMU"/]
  n_ui_qml_embedded_EmbeddedController_qml --> n_ui_qml_ipc_EmbeddedRequestRouter_qml
  n_ui_qml_ipc_EmbeddedEventRouter_qml -.-> n_ui_qml_embedded_EmbeddedController_qml
  n_ui_qml_ipc_EmbeddedEventRouter_qml -.-> n_ui_qml_runtime_RuntimeController_qml
  n_ui_qml_ipc_EmbeddedRequestRouter_qml --> n_ui_src_core_client_probe_cpp
  n_ui_qml_ipc_EmbeddedRequestRouter_qml --> n_ui_src_core_client_requests_run_cpp
  n_ui_src_core_client_dispatch_cpp -.-> n_ui_qml_ipc_EmbeddedEventRouter_qml
  n_ui_src_core_client_dispatch_cmake_cpp -.-> n_ui_qml_ipc_EmbeddedEventRouter_qml
  n_ui_src_core_client_notifications_cpp -.-> n_ui_qml_ipc_EmbeddedEventRouter_qml
  n_ui_src_core_client_probe_cpp --> n_ipc_build
  n_ui_src_core_client_probe_cpp --> n_ipc_probe
  n_ui_src_core_client_probe_cpp --> n_ipc_project
  n_ui_src_core_client_probe_cpp --> n_ipc_serial
  n_ui_src_core_client_probe_cpp -.-> n_ui_qml_ipc_EmbeddedEventRouter_qml
  n_ui_src_core_client_requests_run_cpp --> n_ipc_runConfig
  n_ipc_probe --> n_crates_kinein_core_src_handlers_probe_rs
  n_crates_kinein_core_src_handlers_probe_rs --> n_core_probe
  n_crates_kinein_core_src_handlers_probe_rs --> n_core_rpc
  n_crates_kinein_core_src_handlers_probe_rs --> n_core_toolchain
  n_ipc_project --> n_crates_kinein_core_src_handlers_project_rs
  n_crates_kinein_core_src_handlers_project_rs --> n_core_build
  n_crates_kinein_core_src_handlers_project_rs --> n_core_project
  n_crates_kinein_core_src_handlers_project_rs --> n_core_rpc
  n_crates_kinein_core_src_handlers_project_rs --> n_core_toolchain
  n_ipc_runConfig --> n_crates_kinein_core_src_handlers_runconfig_rs
  n_crates_kinein_core_src_handlers_runconfig_rs --> n_core_build
  n_crates_kinein_core_src_handlers_runconfig_rs --> n_core_rpc
  n_ipc_serial --> n_crates_kinein_core_src_handlers_serial_rs
  n_crates_kinein_core_src_handlers_serial_rs --> n_core_dap
  n_crates_kinein_core_src_handlers_serial_rs --> n_core_project
  n_crates_kinein_core_src_handlers_serial_rs --> n_core_rpc
  n_crates_kinein_core_src_handlers_serial_rs --> n_core_serial
  n_crates_kinein_core_src_handlers_serial_rs --> n_core_toolchain
  n_ipc_build --> n_crates_kinein_core_src_lib_rs
  n_crates_kinein_core_src_lib_rs --> n_core_tools
  CORE -.->|processos| n_tools_embedded
```

| Etapa | Arquivo | O que ele diz de si |
| --- | --- | --- |
| controller | `ui/qml/embedded/EmbeddedController.qml` | Estado do painel de embarcados (roadmaps/35 §5.7, fatia 1 — o FIO). |
| controller | `ui/qml/runtime/RuntimeController.qml` |  |
| roteador | `ui/qml/ipc/EmbeddedRequestRouter.qml` | Espelho do EmbeddedEventRouter: |
| roteador | `ui/qml/ipc/EmbeddedEventRouter.qml` | O que o core responde de probe.*, build.size e serial.* -> EmbeddedController. |
| ponte C++ | `ui/src/core_client_dispatch.cpp` |  |
| ponte C++ | `ui/src/core_client_dispatch_cmake.cpp` | Dispatch do dominio CMake/build (ARCHITECTURE.md §5). |
| ponte C++ | `ui/src/core_client_notifications.cpp` | O que o core manda SEM SER PERGUNTADO: |
| ponte C++ | `ui/src/core_client_probe.cpp` | Dominio de embarcados no lado da UI: |
| ponte C++ | `ui/src/core_client_requests_run.cpp` | Dominio RUN no lado da UI: |
| handler Rust | `crates/kinein-core/src/handlers/probe.rs` | Handler for probe.* requests (impl Core). |
| handler Rust | `crates/kinein-core/src/handlers/project.rs` | Handler for project.* requests (impl Core) — o MODELO do projeto embarcado (pilar 0 do roadmaps/42). |
| handler Rust | `crates/kinein-core/src/handlers/runconfig.rs` | Handlers for runConfig.* requests (impl Core): |
| handler Rust | `crates/kinein-core/src/handlers/serial.rs` | Handler for serial.* requests (impl Core). |
| handler Rust | `crates/kinein-core/src/lib.rs` | Rust core for Kinein Vectis. |

Métodos IPC (9): `build.size`, `probe.list`, `project.model`, `runConfig.flashProposal`, `serial.access`, `serial.files`, `serial.identify`, `serial.list`, `serial.monitor`.

### Toolchains e kits

O kit diz qual compilador, sysroot e alvo valem; o core detecta, explica a origem e nunca 'corrige' sozinho.

```mermaid
flowchart LR
  subgraph QML["ui/qml"]
    n_ui_qml_toolchain_ToolchainController_qml["ToolchainController"]
    n_ui_qml_ipc_ToolchainRequestRouter_qml["ipc/ToolchainRequestRouter"]
    n_ui_qml_ipc_ToolchainEventRouter_qml["ipc/ToolchainEventRouter"]
  end
  subgraph CPP["ui/src"]
    n_ui_src_core_client_dispatch_cpp["core_client_dispatch.cpp"]
    n_ui_src_core_client_dispatch_cmake_cpp["core_client_dispatch_cmake.cpp"]
    n_ui_src_core_client_notifications_cpp["core_client_notifications.cpp"]
    n_ui_src_core_client_toolchain_cpp["core_client_toolchain.cpp"]
  end
  subgraph IPC["JSON-RPC"]
    n_ipc_toolchain(["toolchain.* · 7"])
  end
  subgraph CORE["crates/kinein-core"]
    n_crates_kinein_core_src_handlers_toolchain_rs["handlers/toolchain.rs"]
    n_crates_kinein_core_src_handlers_toolchain_import_rs["handlers/toolchain_import.rs"]
    n_crates_kinein_core_src_handlers_toolchain_install_rs["handlers/toolchain_install.rs"]
    n_core_flash[flash]
    n_core_jobs[jobs]
    n_core_project[project]
    n_core_rpc[rpc]
    n_core_toolchain[toolchain]
  end
  n_tools_toolchain[/"compiladores cross · rustup"/]
  n_ui_qml_ipc_ToolchainEventRouter_qml -.-> n_ui_qml_toolchain_ToolchainController_qml
  n_ui_qml_ipc_ToolchainRequestRouter_qml --> n_ui_src_core_client_toolchain_cpp
  n_ui_qml_toolchain_ToolchainController_qml --> n_ui_qml_ipc_ToolchainRequestRouter_qml
  n_ui_src_core_client_dispatch_cpp -.-> n_ui_qml_ipc_ToolchainEventRouter_qml
  n_ui_src_core_client_dispatch_cmake_cpp -.-> n_ui_qml_ipc_ToolchainEventRouter_qml
  n_ui_src_core_client_notifications_cpp -.-> n_ui_qml_ipc_ToolchainEventRouter_qml
  n_ui_src_core_client_toolchain_cpp --> n_ipc_toolchain
  n_ui_src_core_client_toolchain_cpp -.-> n_ui_qml_ipc_ToolchainEventRouter_qml
  n_ipc_toolchain --> n_crates_kinein_core_src_handlers_toolchain_rs
  n_crates_kinein_core_src_handlers_toolchain_rs --> n_core_rpc
  n_crates_kinein_core_src_handlers_toolchain_rs --> n_core_toolchain
  n_ipc_toolchain --> n_crates_kinein_core_src_handlers_toolchain_import_rs
  n_crates_kinein_core_src_handlers_toolchain_import_rs --> n_core_rpc
  n_ipc_toolchain --> n_crates_kinein_core_src_handlers_toolchain_install_rs
  n_crates_kinein_core_src_handlers_toolchain_install_rs --> n_core_flash
  n_crates_kinein_core_src_handlers_toolchain_install_rs --> n_core_jobs
  n_crates_kinein_core_src_handlers_toolchain_install_rs --> n_core_project
  n_crates_kinein_core_src_handlers_toolchain_install_rs --> n_core_rpc
  n_crates_kinein_core_src_handlers_toolchain_install_rs --> n_core_toolchain
  CORE -.->|processos| n_tools_toolchain
```

| Etapa | Arquivo | O que ele diz de si |
| --- | --- | --- |
| controller | `ui/qml/toolchain/ToolchainController.qml` | Estado da toolchain do workspace (roadmap 30, etapa 5). |
| roteador | `ui/qml/ipc/ToolchainRequestRouter.qml` | Espelho do ToolchainEventRouter: |
| roteador | `ui/qml/ipc/ToolchainEventRouter.qml` | O que o core responde de toolchain.* -> ToolchainController. |
| ponte C++ | `ui/src/core_client_dispatch.cpp` |  |
| ponte C++ | `ui/src/core_client_dispatch_cmake.cpp` | Dispatch do dominio CMake/build (ARCHITECTURE.md §5). |
| ponte C++ | `ui/src/core_client_notifications.cpp` | O que o core manda SEM SER PERGUNTADO: |
| ponte C++ | `ui/src/core_client_toolchain.cpp` | Dominio da toolchain no lado da UI: |
| handler Rust | `crates/kinein-core/src/handlers/toolchain.rs` | Handlers for toolchain.* requests (impl Core): |
| handler Rust | `crates/kinein-core/src/handlers/toolchain_import.rs` | Handlers do gerenciador de toolchain que LE o disco (impl Core). |
| handler Rust | `crates/kinein-core/src/handlers/toolchain_install.rs` | Handlers do provedor de instalacao de toolchain (impl Core). |

Métodos IPC (7): `toolchain.get`, `toolchain.importKit`, `toolchain.inspectSysroot`, `toolchain.install`, `toolchain.installable`, `toolchain.set`, `toolchain.setKit`.

### Remote SSH

O alvo remoto e' espelhado e comandado pelo core; o resultado de `remote.command` vai para o dono certo (configuracao de execucao, kit ou terminal).

```mermaid
flowchart LR
  subgraph QML["ui/qml"]
    n_ui_qml_remote_RemoteController_qml["RemoteController"]
    n_ui_qml_runtime_RuntimeController_qml["RuntimeController"]
    n_ui_qml_ipc_RemoteRequestRouter_qml["ipc/RemoteRequestRouter"]
    n_ui_qml_ipc_RemoteEventRouter_qml["ipc/RemoteEventRouter"]
  end
  subgraph CPP["ui/src"]
    n_ui_src_core_client_dispatch_cpp["core_client_dispatch.cpp"]
    n_ui_src_core_client_notifications_cpp["core_client_notifications.cpp"]
    n_ui_src_core_client_remote_cpp["core_client_remote.cpp"]
    n_ui_src_core_client_requests_cpp["core_client_requests.cpp"]
    n_ui_src_core_client_requests_run_cpp["core_client_requests_run.cpp"]
  end
  subgraph IPC["JSON-RPC"]
    n_ipc_remote(["remote.* · 14"])
    n_ipc_runConfig(["runConfig.* · 1"])
    n_ipc_toolchain(["toolchain.* · 1"])
    n_ipc_workspace(["workspace.* · 1"])
  end
  subgraph CORE["crates/kinein-core"]
    n_crates_kinein_core_src_handlers_remote_rs["handlers/remote.rs"]
    n_crates_kinein_core_src_handlers_remote_command_rs["handlers/remote_command.rs"]
    n_crates_kinein_core_src_handlers_remote_directories_rs["handlers/remote_directories.rs"]
    n_crates_kinein_core_src_handlers_remote_discover_rs["handlers/remote_discover.rs"]
    n_crates_kinein_core_src_handlers_remote_mirror_rs["handlers/remote_mirror.rs"]
    n_crates_kinein_core_src_handlers_remote_trust_rs["handlers/remote_trust.rs"]
    n_crates_kinein_core_src_handlers_runconfig_rs["handlers/runconfig.rs"]
    n_crates_kinein_core_src_handlers_toolchain_rs["handlers/toolchain.rs"]
    n_crates_kinein_core_src_lib_rs["lib.rs"]
    n_core_build[build]
    n_core_jobs[jobs]
    n_core_process[process]
    n_core_remote[remote]
    n_core_rpc[rpc]
    n_core_toolchain[toolchain]
    n_core_tools[tools]
  end
  n_tools_remote[/"ssh · rsync"/]
  n_ui_qml_ipc_RemoteEventRouter_qml -.-> n_ui_qml_remote_RemoteController_qml
  n_ui_qml_ipc_RemoteRequestRouter_qml --> n_ui_src_core_client_remote_cpp
  n_ui_qml_ipc_RemoteRequestRouter_qml --> n_ui_src_core_client_requests_cpp
  n_ui_qml_ipc_RemoteRequestRouter_qml --> n_ui_src_core_client_requests_run_cpp
  n_ui_qml_remote_RemoteController_qml --> n_ui_qml_ipc_RemoteRequestRouter_qml
  n_ui_qml_runtime_RuntimeController_qml --> n_ui_qml_ipc_RemoteRequestRouter_qml
  n_ui_src_core_client_dispatch_cpp -.-> n_ui_qml_ipc_RemoteEventRouter_qml
  n_ui_src_core_client_notifications_cpp -.-> n_ui_qml_ipc_RemoteEventRouter_qml
  n_ui_src_core_client_remote_cpp --> n_ipc_remote
  n_ui_src_core_client_remote_cpp --> n_ipc_toolchain
  n_ui_src_core_client_remote_cpp -.-> n_ui_qml_ipc_RemoteEventRouter_qml
  n_ui_src_core_client_requests_cpp --> n_ipc_workspace
  n_ui_src_core_client_requests_run_cpp --> n_ipc_runConfig
  n_ipc_remote --> n_crates_kinein_core_src_handlers_remote_rs
  n_crates_kinein_core_src_handlers_remote_rs --> n_core_jobs
  n_crates_kinein_core_src_handlers_remote_rs --> n_core_process
  n_crates_kinein_core_src_handlers_remote_rs --> n_core_remote
  n_crates_kinein_core_src_handlers_remote_rs --> n_core_rpc
  n_ipc_remote --> n_crates_kinein_core_src_handlers_remote_command_rs
  n_crates_kinein_core_src_handlers_remote_command_rs --> n_core_remote
  n_crates_kinein_core_src_handlers_remote_command_rs --> n_core_rpc
  n_ipc_remote --> n_crates_kinein_core_src_handlers_remote_directories_rs
  n_crates_kinein_core_src_handlers_remote_directories_rs --> n_core_jobs
  n_crates_kinein_core_src_handlers_remote_directories_rs --> n_core_process
  n_crates_kinein_core_src_handlers_remote_directories_rs --> n_core_remote
  n_crates_kinein_core_src_handlers_remote_directories_rs --> n_core_rpc
  n_ipc_remote --> n_crates_kinein_core_src_handlers_remote_discover_rs
  n_crates_kinein_core_src_handlers_remote_discover_rs --> n_core_remote
  n_crates_kinein_core_src_handlers_remote_discover_rs --> n_core_rpc
  n_ipc_remote --> n_crates_kinein_core_src_handlers_remote_mirror_rs
  n_crates_kinein_core_src_handlers_remote_mirror_rs --> n_core_jobs
  n_crates_kinein_core_src_handlers_remote_mirror_rs --> n_core_process
  n_crates_kinein_core_src_handlers_remote_mirror_rs --> n_core_remote
  n_crates_kinein_core_src_handlers_remote_mirror_rs --> n_core_rpc
  n_ipc_remote --> n_crates_kinein_core_src_handlers_remote_trust_rs
  n_crates_kinein_core_src_handlers_remote_trust_rs --> n_core_remote
  n_crates_kinein_core_src_handlers_remote_trust_rs --> n_core_rpc
  n_ipc_runConfig --> n_crates_kinein_core_src_handlers_runconfig_rs
  n_crates_kinein_core_src_handlers_runconfig_rs --> n_core_build
  n_crates_kinein_core_src_handlers_runconfig_rs --> n_core_rpc
  n_ipc_toolchain --> n_crates_kinein_core_src_handlers_toolchain_rs
  n_crates_kinein_core_src_handlers_toolchain_rs --> n_core_rpc
  n_crates_kinein_core_src_handlers_toolchain_rs --> n_core_toolchain
  n_ipc_workspace --> n_crates_kinein_core_src_lib_rs
  n_crates_kinein_core_src_lib_rs --> n_core_tools
  CORE -.->|processos| n_tools_remote
```

| Etapa | Arquivo | O que ele diz de si |
| --- | --- | --- |
| controller | `ui/qml/remote/RemoteController.qml` | Estado do ALVO LINUX POR SSH (P6 fatia 1 do roadmaps/42, 2026-09-17): |
| controller | `ui/qml/runtime/RuntimeController.qml` |  |
| roteador | `ui/qml/ipc/RemoteRequestRouter.qml` | Espelho do RemoteEventRouter: |
| roteador | `ui/qml/ipc/RemoteEventRouter.qml` | O que o core responde/emite de remote.* -> RemoteController. |
| ponte C++ | `ui/src/core_client_dispatch.cpp` |  |
| ponte C++ | `ui/src/core_client_notifications.cpp` | O que o core manda SEM SER PERGUNTADO: |
| ponte C++ | `ui/src/core_client_remote.cpp` | O ALVO LINUX POR SSH no lado da UI (P6 fatia 1 do roadmaps/42, 2026-09-17): |
| ponte C++ | `ui/src/core_client_requests.cpp` |  |
| ponte C++ | `ui/src/core_client_requests_run.cpp` | Dominio RUN no lado da UI: |
| handler Rust | `crates/kinein-core/src/handlers/remote.rs` | Handlers de remote.* (impl Core) — P6 fatia 1 do roadmaps/42, 2026-09-17. |
| handler Rust | `crates/kinein-core/src/handlers/remote_command.rs` | remote.command (impl Core) — compor a linha, sem rodar nada. |
| handler Rust | `crates/kinein-core/src/handlers/remote_directories.rs` | One-level remote directory browsing for the existing SSH mirror flow. |
| handler Rust | `crates/kinein-core/src/handlers/remote_discover.rs` | remote.discover / remote.resolve (impl Core) — a DESCOBERTA e a EXPLICACAO do SSH que a maquina ja' tem (0.132.0, fatia R0.5 de especificacoes/remote-ssh-ui-hud… |
| handler Rust | `crates/kinein-core/src/handlers/remote_mirror.rs` | remote.open / remote.sync / remote.status (impl Core) — o workspace ESPELHADO (P6 fatia 2 do roadmaps/42, 2026-09-18) — e o empurrao automatico depois de um fs.… |
| handler Rust | `crates/kinein-core/src/handlers/remote_trust.rs` | remote.hostKey / remote.trustHost (impl Core, 0.153.0): |
| handler Rust | `crates/kinein-core/src/handlers/runconfig.rs` | Handlers for runConfig.* requests (impl Core): |
| handler Rust | `crates/kinein-core/src/handlers/toolchain.rs` | Handlers for toolchain.* requests (impl Core): |
| handler Rust | `crates/kinein-core/src/lib.rs` | Rust core for Kinein Vectis. |

Métodos IPC (17): `remote.command`, `remote.deploy`, `remote.directories`, `remote.discover`, `remote.hostKey`, `remote.list`, `remote.open`, `remote.parseCommand`, `remote.probe`, `remote.remove`, `remote.resolve`, `remote.save`, `remote.sync`, `remote.trustHost`, `runConfig.save`, `toolchain.setKit`, `workspace.open`.

### Banco de dados e observabilidade

Credenciais atravessam o roteador uma vez e nao ficam: a senha vai no `datasource.test`, o token no `grafana.probe`, e o cliente os redige do log.

```mermaid
flowchart LR
  subgraph QML["ui/qml"]
    n_ui_qml_datasource_DataSourceController_qml["DataSourceController"]
    n_ui_qml_editor_EditorAppendController_qml["EditorAppendController"]
    n_ui_qml_grafana_GrafanaController_qml["GrafanaController"]
    n_ui_qml_ipc_DataSourceRequestRouter_qml["ipc/DataSourceRequestRouter"]
    n_ui_qml_ipc_DataSourceEventRouter_qml["ipc/DataSourceEventRouter"]
    n_ui_qml_ipc_GrafanaRequestRouter_qml["ipc/GrafanaRequestRouter"]
    n_ui_qml_ipc_GrafanaEventRouter_qml["ipc/GrafanaEventRouter"]
  end
  subgraph CPP["ui/src"]
    n_ui_src_core_client_datasource_cpp["core_client_datasource.cpp"]
    n_ui_src_core_client_dispatch_cpp["core_client_dispatch.cpp"]
    n_ui_src_core_client_grafana_cpp["core_client_grafana.cpp"]
    n_ui_src_core_client_notifications_cpp["core_client_notifications.cpp"]
    n_ui_src_core_client_requests_cpp["core_client_requests.cpp"]
  end
  subgraph IPC["JSON-RPC"]
    n_ipc_datasource(["datasource.* · 16"])
    n_ipc_grafana(["grafana.* · 4"])
    n_ipc_job(["job.* · 1"])
  end
  subgraph CORE["crates/kinein-core"]
    n_crates_kinein_core_src_handlers_datasource_rs["handlers/datasource.rs"]
    n_crates_kinein_core_src_handlers_grafana_rs["handlers/grafana.rs"]
    n_crates_kinein_core_src_handlers_jobs_rs["handlers/jobs.rs"]
    n_crates_kinein_core_src_lib_rs["lib.rs"]
    n_core_datasource[datasource]
    n_core_grafana[grafana]
    n_core_jobs[jobs]
    n_core_rpc[rpc]
    n_core_tools[tools]
  end
  n_tools_data[/"PostgreSQL/SQLite/... · Grafana"/]
  n_ui_qml_datasource_DataSourceController_qml --> n_ui_qml_ipc_DataSourceRequestRouter_qml
  n_ui_qml_editor_EditorAppendController_qml --> n_ui_qml_ipc_DataSourceRequestRouter_qml
  n_ui_qml_grafana_GrafanaController_qml --> n_ui_qml_ipc_GrafanaRequestRouter_qml
  n_ui_qml_ipc_DataSourceEventRouter_qml -.-> n_ui_qml_datasource_DataSourceController_qml
  n_ui_qml_ipc_DataSourceRequestRouter_qml --> n_ui_src_core_client_datasource_cpp
  n_ui_qml_ipc_DataSourceRequestRouter_qml --> n_ui_src_core_client_requests_cpp
  n_ui_qml_ipc_GrafanaEventRouter_qml -.-> n_ui_qml_grafana_GrafanaController_qml
  n_ui_qml_ipc_GrafanaRequestRouter_qml --> n_ui_src_core_client_grafana_cpp
  n_ui_src_core_client_datasource_cpp --> n_ipc_datasource
  n_ui_src_core_client_datasource_cpp -.-> n_ui_qml_ipc_DataSourceEventRouter_qml
  n_ui_src_core_client_dispatch_cpp -.-> n_ui_qml_ipc_DataSourceEventRouter_qml
  n_ui_src_core_client_dispatch_cpp -.-> n_ui_qml_ipc_GrafanaEventRouter_qml
  n_ui_src_core_client_grafana_cpp --> n_ipc_grafana
  n_ui_src_core_client_grafana_cpp -.-> n_ui_qml_ipc_GrafanaEventRouter_qml
  n_ui_src_core_client_notifications_cpp -.-> n_ui_qml_ipc_DataSourceEventRouter_qml
  n_ui_src_core_client_notifications_cpp -.-> n_ui_qml_ipc_GrafanaEventRouter_qml
  n_ui_src_core_client_requests_cpp --> n_ipc_job
  n_ipc_datasource --> n_crates_kinein_core_src_handlers_datasource_rs
  n_crates_kinein_core_src_handlers_datasource_rs --> n_core_datasource
  n_crates_kinein_core_src_handlers_datasource_rs --> n_core_jobs
  n_crates_kinein_core_src_handlers_datasource_rs --> n_core_rpc
  n_ipc_grafana --> n_crates_kinein_core_src_handlers_grafana_rs
  n_crates_kinein_core_src_handlers_grafana_rs --> n_core_datasource
  n_crates_kinein_core_src_handlers_grafana_rs --> n_core_grafana
  n_crates_kinein_core_src_handlers_grafana_rs --> n_core_jobs
  n_crates_kinein_core_src_handlers_grafana_rs --> n_core_rpc
  n_ipc_job --> n_crates_kinein_core_src_handlers_jobs_rs
  n_crates_kinein_core_src_handlers_jobs_rs --> n_core_jobs
  n_crates_kinein_core_src_handlers_jobs_rs --> n_core_rpc
  n_ipc_datasource --> n_crates_kinein_core_src_lib_rs
  n_crates_kinein_core_src_lib_rs --> n_core_tools
  CORE -.->|processos| n_tools_data
```

| Etapa | Arquivo | O que ele diz de si |
| --- | --- | --- |
| controller | `ui/qml/datasource/DataSourceController.qml` | Estado de fontes de dados; regras/validacao sao do core. |
| controller | `ui/qml/editor/EditorAppendController.qml` | Append a prepared instruction using native editing, after opening its file. |
| controller | `ui/qml/grafana/GrafanaController.qml` | Estado da OBSERVABILIDADE (etapa 27 do roadmaps/35). |
| roteador | `ui/qml/ipc/DataSourceRequestRouter.qml` | Espelho do DataSourceEventRouter: |
| roteador | `ui/qml/ipc/DataSourceEventRouter.qml` | Roteia as respostas de datasource.* do CoreClient para o controller. |
| roteador | `ui/qml/ipc/GrafanaRequestRouter.qml` | Leva os pedidos do GrafanaController ao CoreClient. |
| roteador | `ui/qml/ipc/GrafanaEventRouter.qml` | Roteia as respostas e o evento de grafana.* para o GrafanaController. |
| ponte C++ | `ui/src/core_client_datasource.cpp` | Dominio de FONTES DE DADOS no lado da UI: |
| ponte C++ | `ui/src/core_client_dispatch.cpp` |  |
| ponte C++ | `ui/src/core_client_grafana.cpp` | Dominio de OBSERVABILIDADE no lado da UI: |
| ponte C++ | `ui/src/core_client_notifications.cpp` | O que o core manda SEM SER PERGUNTADO: |
| ponte C++ | `ui/src/core_client_requests.cpp` |  |
| handler Rust | `crates/kinein-core/src/handlers/datasource.rs` | Handler dos pedidos datasource.* (impl Core). |
| handler Rust | `crates/kinein-core/src/handlers/grafana.rs` | Handler dos pedidos grafana.* (impl Core). |
| handler Rust | `crates/kinein-core/src/handlers/jobs.rs` | Handlers for job.* requests (impl Core). |
| handler Rust | `crates/kinein-core/src/lib.rs` | Rust core for Kinein Vectis. |

Métodos IPC (21): `datasource.console`, `datasource.console.statement`, `datasource.create`, `datasource.destroy`, `datasource.disconnect`, `datasource.discover`, `datasource.impact`, `datasource.introspect`, `datasource.list`, `datasource.odbc.authorize`, `datasource.odbc.sources`, `datasource.preview.decide`, `datasource.query`, `datasource.remove`, `datasource.save`, `datasource.test`, `grafana.forget`, `grafana.get`, `grafana.probe`, `grafana.save`, `job.cancel`.

### Containers

A UI nunca chama docker/podman: o roteador e' a unica ponte, e o core fala com o motor.

```mermaid
flowchart LR
  subgraph QML["ui/qml"]
    n_ui_qml_container_ContainerController_qml["ContainerController"]
    n_ui_qml_runtime_RuntimeController_qml["RuntimeController"]
    n_ui_qml_ipc_ContainerRequestRouter_qml["ipc/ContainerRequestRouter"]
    n_ui_qml_ipc_ContainerEventRouter_qml["ipc/ContainerEventRouter"]
  end
  subgraph CPP["ui/src"]
    n_ui_src_core_client_container_cpp["core_client_container.cpp"]
    n_ui_src_core_client_dispatch_cpp["core_client_dispatch.cpp"]
    n_ui_src_core_client_notifications_cpp["core_client_notifications.cpp"]
  end
  subgraph IPC["JSON-RPC"]
    n_ipc_container(["container.* · 6"])
  end
  subgraph CORE["crates/kinein-core"]
    n_crates_kinein_core_src_handlers_container_rs["handlers/container.rs"]
    n_core_container[container]
    n_core_jobs[jobs]
    n_core_process[process]
    n_core_rpc[rpc]
  end
  n_tools_container[/"docker · podman"/]
  n_ui_qml_container_ContainerController_qml --> n_ui_qml_ipc_ContainerRequestRouter_qml
  n_ui_qml_ipc_ContainerEventRouter_qml -.-> n_ui_qml_container_ContainerController_qml
  n_ui_qml_ipc_ContainerEventRouter_qml -.-> n_ui_qml_runtime_RuntimeController_qml
  n_ui_qml_ipc_ContainerRequestRouter_qml --> n_ui_src_core_client_container_cpp
  n_ui_src_core_client_container_cpp --> n_ipc_container
  n_ui_src_core_client_container_cpp -.-> n_ui_qml_ipc_ContainerEventRouter_qml
  n_ui_src_core_client_dispatch_cpp -.-> n_ui_qml_ipc_ContainerEventRouter_qml
  n_ui_src_core_client_notifications_cpp -.-> n_ui_qml_ipc_ContainerEventRouter_qml
  n_ipc_container --> n_crates_kinein_core_src_handlers_container_rs
  n_crates_kinein_core_src_handlers_container_rs --> n_core_container
  n_crates_kinein_core_src_handlers_container_rs --> n_core_jobs
  n_crates_kinein_core_src_handlers_container_rs --> n_core_process
  n_crates_kinein_core_src_handlers_container_rs --> n_core_rpc
  CORE -.->|processos| n_tools_container
```

| Etapa | Arquivo | O que ele diz de si |
| --- | --- | --- |
| controller | `ui/qml/container/ContainerController.qml` | Estado do painel de containers (roadmaps/28 §0: |
| controller | `ui/qml/runtime/RuntimeController.qml` |  |
| roteador | `ui/qml/ipc/ContainerRequestRouter.qml` | Espelho do ContainerEventRouter: |
| roteador | `ui/qml/ipc/ContainerEventRouter.qml` | O que o core responde de container.* -> ContainerController; a aba de terminal que container.open cria -> RuntimeController, como qualquer outra. |
| ponte C++ | `ui/src/core_client_container.cpp` | Dominio de containers no lado da UI: |
| ponte C++ | `ui/src/core_client_dispatch.cpp` |  |
| ponte C++ | `ui/src/core_client_notifications.cpp` | O que o core manda SEM SER PERGUNTADO: |
| handler Rust | `crates/kinein-core/src/handlers/container.rs` | Handler for container.* requests (impl Core). |

Métodos IPC (6): `container.action`, `container.compose`, `container.images`, `container.list`, `container.open`, `container.status`.

### Python

O interpretador do projeto (venv, uv) e' decidido no core e usado por executar, testar, depurar e formatar — um dono so'.

```mermaid
flowchart LR
  subgraph QML["ui/qml"]
    n_ui_qml_python_PythonController_qml["PythonController"]
    n_ui_qml_ipc_PythonRequestRouter_qml["ipc/PythonRequestRouter"]
    n_ui_qml_ipc_PythonEventRouter_qml["ipc/PythonEventRouter"]
  end
  subgraph CPP["ui/src"]
    n_ui_src_core_client_notifications_cpp["core_client_notifications.cpp"]
    n_ui_src_core_client_python_cpp["core_client_python.cpp"]
  end
  subgraph IPC["JSON-RPC"]
    n_ipc_python(["python.* · 3"])
  end
  subgraph CORE["crates/kinein-core"]
    n_crates_kinein_core_src_handlers_python_rs["handlers/python.rs"]
    n_core_jobs[jobs]
    n_core_process[process]
    n_core_project[project]
    n_core_python[python]
    n_core_rpc[rpc]
  end
  n_tools_python[/"python · uv · ruff"/]
  n_ui_qml_ipc_PythonEventRouter_qml -.-> n_ui_qml_python_PythonController_qml
  n_ui_qml_ipc_PythonRequestRouter_qml --> n_ui_src_core_client_python_cpp
  n_ui_qml_python_PythonController_qml --> n_ui_qml_ipc_PythonRequestRouter_qml
  n_ui_src_core_client_notifications_cpp -.-> n_ui_qml_ipc_PythonEventRouter_qml
  n_ui_src_core_client_python_cpp --> n_ipc_python
  n_ui_src_core_client_python_cpp -.-> n_ui_qml_ipc_PythonEventRouter_qml
  n_ipc_python --> n_crates_kinein_core_src_handlers_python_rs
  n_crates_kinein_core_src_handlers_python_rs --> n_core_jobs
  n_crates_kinein_core_src_handlers_python_rs --> n_core_process
  n_crates_kinein_core_src_handlers_python_rs --> n_core_project
  n_crates_kinein_core_src_handlers_python_rs --> n_core_python
  n_crates_kinein_core_src_handlers_python_rs --> n_core_rpc
  CORE -.->|processos| n_tools_python
```

| Etapa | Arquivo | O que ele diz de si |
| --- | --- | --- |
| controller | `ui/qml/python/PythonController.qml` | O AMBIENTE Python do projeto (bloco B do roadmaps/41, fatia 1 em 2026-09-12). |
| roteador | `ui/qml/ipc/PythonRequestRouter.qml` | Espelho do PythonEventRouter: |
| roteador | `ui/qml/ipc/PythonEventRouter.qml` | O que o core responde/emite de python.* -> PythonController. |
| ponte C++ | `ui/src/core_client_notifications.cpp` | O que o core manda SEM SER PERGUNTADO: |
| ponte C++ | `ui/src/core_client_python.cpp` | O AMBIENTE Python do projeto no lado da UI (bloco B do roadmaps/41, fatia 1 em 2026-09-12): |
| handler Rust | `crates/kinein-core/src/handlers/python.rs` | Handlers for python.* requests (impl Core) — o AMBIENTE do projeto Python (bloco B do roadmaps/41, cadeia iniciada em 2026-09-12). |

Métodos IPC (3): `python.createEnvironment`, `python.status`, `python.stubs`.

### Configuracao: acoes, biblioteca, setup e ajustes

Mudanca de configuracao do projeto e' sempre list -> preview -> apply, com escopo e documentacao; o controller nunca escreve arquivo de configuracao.

```mermaid
flowchart LR
  subgraph QML["ui/qml"]
    n_ui_qml_configaction_ConfigActionController_qml["ConfigActionController"]
    n_ui_qml_library_LibraryController_qml["LibraryController"]
    n_ui_qml_settings_SettingsController_qml["SettingsController"]
    n_ui_qml_setup_SetupController_qml["SetupController"]
    n_ui_qml_ipc_ConfigActionRequestRouter_qml["ipc/ConfigActionRequestRouter"]
    n_ui_qml_ipc_ConfigActionEventRouter_qml["ipc/ConfigActionEventRouter"]
    n_ui_qml_ipc_LibraryRequestRouter_qml["ipc/LibraryRequestRouter"]
    n_ui_qml_ipc_LibraryEventRouter_qml["ipc/LibraryEventRouter"]
    n_ui_qml_ipc_SetupRequestRouter_qml["ipc/SetupRequestRouter"]
    n_ui_qml_ipc_SetupEventRouter_qml["ipc/SetupEventRouter"]
    n_ui_qml_ipc_SettingsEventRouter_qml["ipc/SettingsEventRouter"]
  end
  subgraph CPP["ui/src"]
    n_ui_src_core_client_configaction_cpp["core_client_configaction.cpp"]
    n_ui_src_core_client_datasource_cpp["core_client_datasource.cpp"]
    n_ui_src_core_client_dispatch_cpp["core_client_dispatch.cpp"]
    n_ui_src_core_client_dispatch_cmake_cpp["core_client_dispatch_cmake.cpp"]
    n_ui_src_core_client_dispatch_debug_cpp["core_client_dispatch_debug.cpp"]
    n_ui_src_core_client_library_cpp["core_client_library.cpp"]
    n_ui_src_core_client_requests_cpp["core_client_requests.cpp"]
  end
  subgraph IPC["JSON-RPC"]
    n_ipc_cmake(["cmake.* · 1"])
    n_ipc_configAction(["configAction.* · 3"])
    n_ipc_library(["library.* · 2"])
    n_ipc_setup(["setup.* · 1"])
  end
  subgraph CORE["crates/kinein-core"]
    n_crates_kinein_core_src_handlers_cmake_rs["handlers/cmake.rs"]
    n_crates_kinein_core_src_handlers_configaction_rs["handlers/configaction.rs"]
    n_crates_kinein_core_src_handlers_library_rs["handlers/library.rs"]
    n_crates_kinein_core_src_handlers_setup_rs["handlers/setup.rs"]
    n_core_cdb[cdb]
    n_core_configaction[configaction]
    n_core_library[library]
    n_core_rpc[rpc]
    n_core_settings[settings]
    n_core_setup[setup]
    n_core_toolchain[toolchain]
  end
  n_tools_config[/"os gerenciadores oficiais de cada ecossistema"/]
  n_ui_qml_configaction_ConfigActionController_qml --> n_ui_qml_ipc_ConfigActionRequestRouter_qml
  n_ui_qml_ipc_ConfigActionEventRouter_qml -.-> n_ui_qml_configaction_ConfigActionController_qml
  n_ui_qml_ipc_ConfigActionRequestRouter_qml --> n_ui_src_core_client_configaction_cpp
  n_ui_qml_ipc_LibraryEventRouter_qml -.-> n_ui_qml_library_LibraryController_qml
  n_ui_qml_ipc_LibraryRequestRouter_qml --> n_ui_src_core_client_library_cpp
  n_ui_qml_ipc_LibraryRequestRouter_qml --> n_ui_src_core_client_requests_cpp
  n_ui_qml_ipc_SettingsEventRouter_qml -.-> n_ui_qml_settings_SettingsController_qml
  n_ui_qml_ipc_SetupEventRouter_qml -.-> n_ui_qml_setup_SetupController_qml
  n_ui_qml_ipc_SetupRequestRouter_qml --> n_ui_src_core_client_datasource_cpp
  n_ui_qml_library_LibraryController_qml --> n_ui_qml_ipc_LibraryRequestRouter_qml
  n_ui_qml_setup_SetupController_qml --> n_ui_qml_ipc_SetupRequestRouter_qml
  n_ui_src_core_client_configaction_cpp --> n_ipc_configAction
  n_ui_src_core_client_configaction_cpp -.-> n_ui_qml_ipc_ConfigActionEventRouter_qml
  n_ui_src_core_client_datasource_cpp --> n_ipc_setup
  n_ui_src_core_client_datasource_cpp -.-> n_ui_qml_ipc_SetupEventRouter_qml
  n_ui_src_core_client_dispatch_cpp -.-> n_ui_qml_ipc_ConfigActionEventRouter_qml
  n_ui_src_core_client_dispatch_cpp -.-> n_ui_qml_ipc_LibraryEventRouter_qml
  n_ui_src_core_client_dispatch_cpp -.-> n_ui_qml_ipc_SetupEventRouter_qml
  n_ui_src_core_client_dispatch_cmake_cpp -.-> n_ui_qml_ipc_LibraryEventRouter_qml
  n_ui_src_core_client_dispatch_debug_cpp -.-> n_ui_qml_ipc_SettingsEventRouter_qml
  n_ui_src_core_client_library_cpp --> n_ipc_library
  n_ui_src_core_client_library_cpp -.-> n_ui_qml_ipc_LibraryEventRouter_qml
  n_ui_src_core_client_requests_cpp --> n_ipc_cmake
  n_ipc_cmake --> n_crates_kinein_core_src_handlers_cmake_rs
  n_crates_kinein_core_src_handlers_cmake_rs --> n_core_cdb
  n_crates_kinein_core_src_handlers_cmake_rs --> n_core_rpc
  n_crates_kinein_core_src_handlers_cmake_rs --> n_core_settings
  n_crates_kinein_core_src_handlers_cmake_rs --> n_core_toolchain
  n_ipc_configAction --> n_crates_kinein_core_src_handlers_configaction_rs
  n_crates_kinein_core_src_handlers_configaction_rs --> n_core_configaction
  n_crates_kinein_core_src_handlers_configaction_rs --> n_core_rpc
  n_ipc_library --> n_crates_kinein_core_src_handlers_library_rs
  n_crates_kinein_core_src_handlers_library_rs --> n_core_library
  n_crates_kinein_core_src_handlers_library_rs --> n_core_rpc
  n_ipc_setup --> n_crates_kinein_core_src_handlers_setup_rs
  n_crates_kinein_core_src_handlers_setup_rs --> n_core_rpc
  n_crates_kinein_core_src_handlers_setup_rs --> n_core_setup
  CORE -.->|processos| n_tools_config
```

| Etapa | Arquivo | O que ele diz de si |
| --- | --- | --- |
| controller | `ui/qml/configaction/ConfigActionController.qml` | Estado das Configuration Actions (roadmap 30, etapa 2). |
| controller | `ui/qml/library/LibraryController.qml` | Estado do catalogo de bibliotecas (etapa 20 do roadmaps/35). |
| controller | `ui/qml/settings/SettingsController.qml` | Estado de configuracoes (fatia M4.1). |
| controller | `ui/qml/setup/SetupController.qml` | O passo a passo OFICIAL para instalar o que falta, na distro detectada. |
| roteador | `ui/qml/ipc/ConfigActionRequestRouter.qml` | Espelho do ConfigActionEventRouter: |
| roteador | `ui/qml/ipc/ConfigActionEventRouter.qml` | O que o core devolve das Configuration Actions -> ConfigActionController. |
| roteador | `ui/qml/ipc/LibraryRequestRouter.qml` | Espelho do LibraryEventRouter: |
| roteador | `ui/qml/ipc/LibraryEventRouter.qml` | Roteia as respostas de library.* do CoreClient para o LibraryController. |
| roteador | `ui/qml/ipc/SetupRequestRouter.qml` | Espelho do SetupEventRouter: |
| roteador | `ui/qml/ipc/SetupEventRouter.qml` | Roteia a resposta de setup.list do CoreClient para o SetupController. |
| roteador | `ui/qml/ipc/SettingsEventRouter.qml` | Resultado de settings.get/set → SettingsController (fatia M4.1). |
| ponte C++ | `ui/src/core_client_configaction.cpp` | Dominio das Configuration Actions no lado da UI: |
| ponte C++ | `ui/src/core_client_datasource.cpp` | Dominio de FONTES DE DADOS no lado da UI: |
| ponte C++ | `ui/src/core_client_dispatch.cpp` |  |
| ponte C++ | `ui/src/core_client_dispatch_cmake.cpp` | Dispatch do dominio CMake/build (ARCHITECTURE.md §5). |
| ponte C++ | `ui/src/core_client_dispatch_debug.cpp` | Dispatch do dominio DEBUG (ARCHITECTURE.md §5): |
| ponte C++ | `ui/src/core_client_library.cpp` | Dominio de BIBLIOTECAS no lado da UI: |
| ponte C++ | `ui/src/core_client_requests.cpp` |  |
| handler Rust | `crates/kinein-core/src/handlers/cmake.rs` | Handlers for cmake.* requests (impl Core): |
| handler Rust | `crates/kinein-core/src/handlers/configaction.rs` | Handlers for configAction.* requests (impl Core): |
| handler Rust | `crates/kinein-core/src/handlers/library.rs` | Handlers for library.* requests (impl Core). |
| handler Rust | `crates/kinein-core/src/handlers/setup.rs` | Handler dos pedidos setup.* (impl Core). |

Métodos IPC (7): `cmake.targets.list`, `configAction.apply`, `configAction.list`, `configAction.preview`, `library.list`, `library.plan`, `setup.list`.
