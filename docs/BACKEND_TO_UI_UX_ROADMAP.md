# Backend to UI/UX Roadmap

> **Status:** active
> **Prioridade:** P0
> **Fonte de verdade:** nao; ponte operacional entre backend real e specs UI/UX
> **Ultima revisao:** 2026-07-05
> **Substituido por:** n/a

## Objetivo

Este documento amarra cada etapa de backend da Kinein Vectis ao impacto futuro
na UI/UX. Ele existe para permitir a decisao atual:

```text
1. consolidar backend e contratos primeiro;
2. manter os docs ativos sincronizados;
3. refatorar/reformular a UI depois, usando contratos estaveis;
4. nao perder as exigencias inegociaveis de UI/UX dos specs.
```

Este arquivo nao substitui `docs/specs/`. Quando houver duvida de produto,
layout, interacao ou sistema visual, a fonte alvo continua sendo:

```text
docs/specs/KINEIN_VECTIS_SPEC_INDEX.md
docs/specs/KINEIN_VECTIS_LAYOUT_SYSTEM.md
docs/specs/KINEIN_VECTIS_UI_COMPONENTS_SYSTEM.md
docs/specs/KINEIN_VECTIS_VISUAL_SYSTEM_ICONS.md
docs/specs/KINEIN_VECTIS_PRODUCT_FLOWS_BUILD_RUN_DEBUG.md
docs/specs/KINEIN_VECTIS_EDITOR_LANGUAGE_INTELLIGENCE.md
docs/specs/KINEIN_VECTIS_INTERNAL_ARCHITECTURE_CORE_IPC_JOBS.md
docs/specs/KINEIN_VECTIS_RESOURCE_ON_DEMAND_PERFORMANCE_STRATEGY.md
```

## Regra de uso

Antes de implementar uma etapa de backend:

```text
1. confirmar o dominio correto no core;
2. definir tipos em kinein-protocol quando houver contrato IPC novo;
3. implementar handler fino + servico de dominio;
4. transformar operacao longa em job;
5. escrever testes de comportamento;
6. atualizar docs/03-ipc-protocol.md se IPC mudar;
7. atualizar ContextoIA.md com estado real;
8. atualizar este roadmap com impacto UI futuro.
```

Antes de refatorar UI:

```text
1. usar este documento como mapa de capacidades backend ja estaveis;
2. abrir os specs UI/UX citados na etapa;
3. nao criar chamada direta a ferramenta externa na UI;
4. nao duplicar cliente IPC;
5. transformar Main.qml em componentes por dominio;
6. manter UI visual: sem regra de negocio, sem parsing de ferramenta.
```

## Estado atual resumido

```text
Backend Rust:
- workspace/fs/tool detection/LSP/run/terminal/build/test/quality funcionais.
- build.run, quality.run e test.run sao jobs assincronos/cancelaveis.
- job.list e job.cancel expostos.
- job.cancel aceito muda o status para cancelRequested antes do terminal
  cancelled.
- JobManager retem ate 100 jobs em memoria, preservando ativos e removendo
  finalizados mais antigos.
- Rust gate rapido verde em 2026-07-05: fmt, test, clippy.

UI:
- funcional, mas monolitica.
- Main.qml tem mais de 4k linhas.
- CoreClient ainda centraliza muitos dominios.
- UI ainda precisa se adaptar ao contrato novo de jobs: build/test/quality
  retornam { jobId }, resultado final vem por event.*.finished.
```

## Prioridades de backend antes da grande UI

### P0. Contrato de jobs e compatibilidade minima

**Status:** feito, incluindo cancelamento pela UI (2026-07-05, Fable) —
`CoreClient` trata `{ jobId }` como aceite e finaliza build/test/quality via
`event.<dominio>.finished`; `environment.scan` alimenta a aba Ferramentas via
`toolsListed`; a status bar tem cancelar (×) para build/test/quality/scan de
ambiente. `event.job.created/progress/output/finished` continuam reemitidos
como sinais Qt sem consumidor dedicado (nenhum painel de jobs generico existe
ainda — so os quatro cancelamentos pontuais). Ver `ContextoIA.md`.

Backend atual:

```text
job.list
job.cancel
event.job.created
event.job.progress
event.job.output
event.job.finished
status intermediario: cancelRequested
build.run    -> { jobId } + event.build.* + event.job.output para saida bruta
quality.run  -> { jobId } + event.quality.* + event.job.output para saida bruta
test.run     -> { jobId } + event.test.* + event.job.output para saida bruta
```

Pendencias backend:

```text
- revisar se todo erro de spawn/ferramenta ausente em job emite evento final
  rico o suficiente para Problems/Tool Window;
```

Impacto UI futuro:

```text
- Status bar mostra job ativo;
- tool window/lista de jobs mostra historico;
- historico generico usa event.job.output; paineis especificos continuam usando
  event.build/test/quality.*;
- build/test/quality ficam ativos ate event.<dominio>.finished;
- cancelamento pode mostrar status intermediario cancelRequested;
- botao Cancel chama job.cancel;
- eventos event.job.* nao abrem popup automatico.
```

Patch minimo UI em andamento (antes da grande refatoracao):

```text
Arquivos provaveis:
- ui/src/core_client.h
- ui/src/core_client.cpp
- ui/qml/Main.qml

Implementar:
- respostas { jobId } de build.run/quality.run/test.run significam "job aceito";
- resultado final vem de event.build.finished/event.quality.finished/event.test.finished;
- event.job.created/progress/output/finished alimenta estado/log minimo de jobs;
- event.environment.started/tool/finished alimenta scan de ambiente/tooling;
- tools.detect/status continuam funcionando para compatibilidade.

Nao implementar agora:
- redesign final;
- quebra massiva do Main.qml;
- executor/parsers na UI;
- paineis definitivos de Toolchains/Project Health/Settings.
```

Componentes UI provaveis:

```text
StatusBar.qml
JobsPopover.qml
TaskStatusChip.qml
BuildPanel.qml
TestsPanel.qml
QualityPanel.qml
```

Specs relacionadas:

```text
KINEIN_VECTIS_INTERNAL_ARCHITECTURE_CORE_IPC_JOBS.md
KINEIN_VECTIS_PRODUCT_FLOWS_BUILD_RUN_DEBUG.md
KINEIN_VECTIS_LAYOUT_SYSTEM.md
KINEIN_VECTIS_UI_COMPONENTS_SYSTEM.md
```

Nao fazer:

```text
- nao reintroduzir streaming sincrono de build/test/quality;
- nao deixar UI inferir sucesso pela resposta de start;
- nao criar executor paralelo na UI.
```

### P1. Process Runner interno unificado

**Status:** parcial; inventario claro, unificacao ainda nao feita.

Processos existentes hoje:

```text
build/test/quality -> JobManager + process::stream_command_lines_cancelable
run.start          -> RunManager com sh -c, pipes e stdin
terminal.open      -> TerminalManager com script(1), PTY e sanitizacao ANSI
lsp                -> LspManager/ServerHandle por linguagem
tools.detect       -> ToolDetector sincrono com --version
workspace.new Rust -> cargo new sincrono
```

Backend desejado:

```text
- criar uma camada interna de processo/task sem big-bang;
- manter run e terminal semanticamente separados;
- expor logs, cancelamento e status de forma consistente;
- preparar multi-processos futuros: git, cmake configure, cargo metadata,
  clang-tidy, formatters, AI CLI bridge.
```

Impacto UI futuro:

```text
- a UI mostra processos ativos de forma uniforme;
- logs sao clicaveis por job/task;
- "Background Services" consegue explicar o que esta vivo;
- Run/Terminal/Build continuam visualmente separados, mas com lifecycle comum.
```

Componentes UI provaveis:

```text
BackgroundServicesPanel.qml
ProcessLogView.qml
RunPanel.qml
TerminalPanel.qml
```

Specs relacionadas:

```text
KINEIN_VECTIS_RESOURCE_ON_DEMAND_PERFORMANCE_STRATEGY.md
KINEIN_VECTIS_INTERNAL_ARCHITECTURE_CORE_IPC_JOBS.md
KINEIN_VECTIS_PRODUCT_FLOWS_BUILD_RUN_DEBUG.md
```

Nao fazer:

```text
- nao misturar PTY interativo com run controlado;
- nao fazer UI spawnar comandos;
- nao esconder processos longos fora do Job/Event system.
```

### P1. Diagnostics unificados

**Status:** parcial.

Backend atual:

```text
event.build.diagnostic
event.quality.diagnostic
event.lsp.diagnostics
event.test.case
Diagnostic comum em kinein-protocol
source/category incremental em build/quality/lsp diagnostics
```

Limitacoes atuais:

```text
- diagnosticos ainda sao emitidos por eventos de origem, embora usem shape comum;
- Problems na UI agrupa parcialmente;
- nao ha id estavel de diagnostico;
- nao ha actions/next steps;
- errors de toolchain/config ainda nao entram num modelo unico.
- ainda nao ha range completo/actions/logRef preenchidos por produtores reais.
```

Backend desejado:

```text
Diagnostic {
  id,
  source,
  severity,
  category,
  message,
  file,
  line/column ou range,
  command,
  target,
  jobId,
  actions,
  logRef
}
```

Impacto UI futuro:

```text
- Problems Panel 2.0 com filtros por source/severity/current file;
- erro de build, LSP, quality, CMake, Cargo e toolchain aparecem no mesmo modelo;
- cada erro pode oferecer acoes: abrir log, configurar toolchain, rodar configure,
  reiniciar LSP, abrir AI Terminal manualmente.
```

Componentes UI provaveis:

```text
ProblemsPanel.qml
ProblemRow.qml
ProblemDetailsPane.qml
DiagnosticActionsMenu.qml
```

Specs relacionadas:

```text
KINEIN_VECTIS_EDITOR_LANGUAGE_INTELLIGENCE.md
KINEIN_VECTIS_PRODUCT_FLOWS_BUILD_RUN_DEBUG.md
KINEIN_VECTIS_UI_COMPONENTS_SYSTEM.md
```

Nao fazer:

```text
- nao parsear saida bruta na UI quando o core pode emitir evento estruturado;
- nao criar Problems paralelo por feature;
- nao misturar diagnostico de projeto com log interno da IDE.
```

### P1. Toolchain e environment scan

**Status:** tool detection basica implementada; `environment.scan` como job
implementado; toolchain real pendente.

Backend atual:

```text
tools.detect
tools.status
environment.scan -> { jobId } + event.environment.*
ToolDetector busca no PATH e roda --version.
Ferramentas cobertas: cargo, rustc, rustup, rust-analyzer, cmake, ninja, git,
clangd, clang, clang++ (`clangxx`), gcc, g++ (`gxx`), gdb, lldb, rg,
fd/fdfind.
```

Backend desejado:

```text
environment.fingerprint
toolchain.scan
toolchain.list
toolchain.healthCheck
toolchain.setDefault
```

Escopo inicial recomendado:

```text
- validar PATH + --version alem do conjunto local atual;
- opcionalmente compilar arquivo minimo C/C++/Rust como job;
- persistir toolchains com schema versionado;
- nunca instalar ferramenta automaticamente.
```

Impacto UI futuro:

```text
- First Run mostra ambiente local;
- Toolchain Settings permite ver/corrigir caminhos;
- top/status bar mostra toolchain ativa;
- projeto abre em modo degradado quando falta ferramenta.
```

Componentes UI provaveis:

```text
FirstRunScreen.qml
ToolchainSettingsPage.qml
ToolchainHealthCard.qml
ToolStatusBadge.qml
```

Specs relacionadas:

```text
KINEIN_VECTIS_ONBOARDING_PROJECT_WIZARD_SETTINGS.md
KINEIN_VECTIS_ONBOARDING_SETUP_OPTIMIZATION_LAYER.md
KINEIN_VECTIS_PRODUCT_FLOWS_BUILD_RUN_DEBUG.md
```

Nao fazer:

```text
- nao instalar pacotes sozinho;
- nao bloquear abertura de workspace por ferramenta ausente;
- nao esconder comando real testado.
```

### P1. CMake service incremental

**Status:** build CMake minimo implementado dentro de build.run.

Backend atual:

```text
ProjectKind::Cmake detectado por CMakeLists.txt.
build.run configura .kinein/build se nao houver CMakeCache.txt e roda cmake --build.
```

Limitacoes atuais:

```text
- nao ha cmake.configure separado;
- nao ha leitura de CMakePresets.json;
- nao ha profiles/targets;
- run.start tenta descobrir unico executavel em .kinein/build;
- parser CMake ainda e basico.
```

Backend desejado:

```text
cmake.presets.list
cmake.configure
cmake.build
cmake.cache.inspect
cmake.cache.clear
cmake.targets.list
```

Impacto UI futuro:

```text
- toolbar mostra profile/preset/target;
- Project Health explica "Needs Configure";
- CMake tool window mostra configure/build/logs;
- usuario ve comando real antes de primeira configuracao;
- compile_commands.json alimenta clangd com analise mais confiavel.
```

Componentes UI provaveis:

```text
CMakePanel.qml
CMakeConfigureDialog.qml
BuildProfileSelector.qml
TargetSelector.qml
ProjectHealthBanner.qml
```

Specs relacionadas:

```text
KINEIN_VECTIS_PRODUCT_FLOWS_BUILD_RUN_DEBUG.md
KINEIN_VECTIS_EDITOR_LANGUAGE_INTELLIGENCE.md
KINEIN_VECTIS_SCOPED_CONFIGURATION_ACTIONS_DOC_LINKS.md
```

Nao fazer:

```text
- nao sobrescrever CMakePresets.json sem preview/confirmacao;
- nao criar parser universal de CMake no MVP;
- nao esconder CMake real do usuario.
```

### P1. Cargo/Rust service incremental

**Status:** build/test/quality Rust implementados via cargo.

Backend atual:

```text
build.run    -> cargo build --message-format=json
test.run     -> cargo test
quality.run  -> cargo clippy --all-targets --message-format=json
LSP Rust     -> rust-analyzer sob demanda
```

Limitacoes atuais:

```text
- nao ha cargo metadata;
- nao ha features/profile/target triple;
- nao ha cargo check separado;
- rust-analyzer nao recebe configuracao do projeto ainda.
```

Backend desejado:

```text
cargo.metadata
cargo.check
cargo.build
cargo.test
cargo.features.list
rust.toolchain.status
```

Impacto UI futuro:

```text
- toolbar mostra profile/features/target;
- painel Rust/Cargo mostra crates, features e estado;
- Problems diferencia cargo build, cargo check, clippy e rust-analyzer;
- Project Health detecta cargo metadata quebrado.
```

Componentes UI provaveis:

```text
CargoPanel.qml
RustFeaturesSelector.qml
BuildProfileSelector.qml
ProjectHealthBanner.qml
```

Specs relacionadas:

```text
KINEIN_VECTIS_PRODUCT_FLOWS_BUILD_RUN_DEBUG.md
KINEIN_VECTIS_EDITOR_LANGUAGE_INTELLIGENCE.md
KINEIN_VECTIS_SCOPED_CONFIGURATION_ACTIONS_DOC_LINKS.md
```

Nao fazer:

```text
- nao tratar Rust como extensao secundaria;
- nao editar Cargo.toml automaticamente sem preview;
- nao criar parser proprio de Cargo quando cargo metadata resolve.
```

### P1. Project Health

**Status:** pendente.

Backend desejado:

```text
project.detect
project.fingerprint
project.health
```

Sinais iniciais:

```text
- workspace kind;
- ferramentas ausentes;
- CMake sem configure;
- compile_commands.json ausente em C/C++;
- cargo metadata falhou;
- LSP em modo degradado;
- build dir stale;
- run config ausente.
```

Impacto UI futuro:

```text
- banners discretos e acionaveis;
- status bar explica modo degradado;
- First Run/Project Wizard guia sem ser invasivo;
- usuario entende por que Build/Run/Debug estao indisponiveis.
```

Componentes UI provaveis:

```text
ProjectHealthBanner.qml
ProjectHealthPanel.qml
SetupActionButton.qml
```

Specs relacionadas:

```text
KINEIN_VECTIS_ONBOARDING_SETUP_OPTIMIZATION_LAYER.md
KINEIN_VECTIS_RESOURCE_ON_DEMAND_PERFORMANCE_STRATEGY.md
KINEIN_VECTIS_PRODUCT_FLOWS_BUILD_RUN_DEBUG.md
```

Nao fazer:

```text
- nao bloquear edicao em projeto quebrado;
- nao abrir popups automaticos agressivos;
- nao fingir que a IDE entende o projeto quando faltam dados.
```

### P1. Settings, storage e migracoes

**Status:** modelo strict em `kinein-config`; IPC/settings pendente.

Backend atual:

```text
kinein-config define ProjectSettings strict-by-default.
workspace.open persiste .kinein/workspace.json com schemaVersion.
```

Backend desejado:

```text
settings.get
settings.set
settings.schema
settings.search
settings.reset
storage com schemaVersion/migracoes/backups
```

Impacto UI futuro:

```text
- Settings tem paginas reais, lazy, por categoria;
- Strict/Balanced/Relaxed aparecem como escolha explicita;
- mudancas perigosas mostram preview/impacto;
- configs quebradas geram erro humano e reset seguro.
```

Componentes UI provaveis:

```text
SettingsWindow.qml
SettingsSidebar.qml
SettingsPage.qml
StrictnessSelector.qml
SchemaErrorDialog.qml
```

Specs relacionadas:

```text
KINEIN_VECTIS_ONBOARDING_PROJECT_WIZARD_SETTINGS.md
KINEIN_VECTIS_DUAL_WORKFLOW_CONFIGURATION_ACTIONS.md
KINEIN_VECTIS_SCOPED_CONFIGURATION_ACTIONS_DOC_LINKS.md
```

Nao fazer:

```text
- nao criar formato sem schema;
- nao salvar segredo em config/log;
- nao relaxar strict mode sem escolha explicita.
```

### P1. Risk engine e confirmacoes

**Status:** `JobRisk` existe; enforcement geral pendente.

Backend atual:

```text
JobRisk: low, medium, high, dangerous.
Jobs carregam risk no JobInfo.
```

Backend desejado:

```text
risk.classify(command/action)
confirmation.required events/responses
bloqueio de dangerous por padrao
preview/diff para mutacoes automaticas
```

Impacto UI futuro:

```text
- dialogos de confirmacao consistentes;
- acoes high/dangerous visivelmente diferentes;
- IA nunca aplica alteracao automaticamente;
- usuario ve diff/arquivos afetados antes de mudancas amplas.
```

Componentes UI provaveis:

```text
RiskConfirmationDialog.qml
PreviewDiffDialog.qml
DangerousActionBanner.qml
```

Specs relacionadas:

```text
KINEIN_VECTIS_INTERNAL_ARCHITECTURE_CORE_IPC_JOBS.md
KINEIN_VECTIS_KV_CONTEXT_AI_ASSISTANCE.md
KINEIN_VECTIS_AI_CLI_BRIDGE_EXTERNAL_TERMINAL.md
```

Nao fazer:

```text
- nao executar comando destrutivo sem confirmacao explicita;
- nao aplicar patch de IA automaticamente;
- nao permitir UI contornar risk engine.
```

### P2. Run configurations

**Status:** `run.start` MVP implementado; configs pendentes.

Backend atual:

```text
run.start { command? }
run.stdin
run.stop
um processo por vez
default: cargo run ou unico executavel CMake em .kinein/build
```

Backend desejado:

```text
runConfig.list
runConfig.create
runConfig.update
runConfig.delete
run.start { configId | command }
```

Impacto UI futuro:

```text
- toolbar tem Run Configuration selecionada;
- beforeRun fica visivel e desativavel;
- comando final auditavel;
- Run e Debug deixam de depender de heuristica simples.
```

Componentes UI provaveis:

```text
RunConfigSelector.qml
RunConfigDialog.qml
RunPanel.qml
```

Specs relacionadas:

```text
KINEIN_VECTIS_PRODUCT_FLOWS_BUILD_RUN_DEBUG.md
KINEIN_VECTIS_ONBOARDING_PROJECT_WIZARD_SETTINGS.md
```

### P2. Git MVP

**Status:** pendente.

Backend desejado:

```text
git.status
git.diff
git.branches
git.commitPreview
git.stage/unstage
```

Impacto UI futuro:

```text
- status bar mostra branch;
- Project tree mostra mudancas;
- painel Git separado;
- acoes destrutivas exigem risk/confirmacao.
```

Componentes UI provaveis:

```text
GitPanel.qml
GitStatusChip.qml
CommitDialog.qml
DiffViewer.qml
```

Specs relacionadas:

```text
KINEIN_VECTIS_LAYOUT_SYSTEM.md
KINEIN_VECTIS_UI_COMPONENTS_SYSTEM.md
KINEIN_VECTIS_INTERNAL_ARCHITECTURE_CORE_IPC_JOBS.md
```

### P2. AI CLI Bridge backend

**Status:** futuro; nao implementar provider/chat embutido.

Backend desejado:

```text
aiBridge.profiles.list
aiBridge.context.create
aiBridge.context.preview
aiBridge.terminal.open
```

Impacto UI futuro:

```text
- AI Terminal separado do terminal comum;
- contexto e sanitizacao aparecem antes de abrir CLI;
- usuario escolhe Claude/Codex/GPT CLI;
- IDE nao conversa pelo usuario.
```

Componentes UI provaveis:

```text
AiTerminalPanel.qml
ContextPreviewDialog.qml
AiProfileSelector.qml
```

Specs relacionadas:

```text
KINEIN_VECTIS_AI_CLI_BRIDGE_EXTERNAL_TERMINAL.md
KINEIN_VECTIS_KV_CONTEXT_AI_ASSISTANCE.md
```

Nao fazer:

```text
- nao criar chat lateral/provider API no MVP;
- nao enviar codigo para IA externa sem acao explicita;
- nao guardar historico de conversa como feature principal.
```

## Sequencia recomendada (revisada 2026-07-05 — usuario pediu para nao atrasar o Qt)

A sequencia original empilhava ~10 itens de backend antes de qualquer
refatoracao de UI. O usuario decidiu explicitamente nao atrasar mais o
frontend Qt (projeto ja medio/medio-grande): cada entrega de backend deve vir
acompanhada de um ganho de UI visivel, em vez de mais uma fase de contrato
invisivel.

```text
1. [feito] Compatibilidade minima UI<->jobs (patch 2026-07-05).
2. [feito 2026-07-05] Cancelar build/test/quality/environment-scan pela UI:
   CoreClient::cancelBuild/cancelTests/cancelQuality/cancelEnvironmentScan
   chamam job.cancel usando o jobId aceito por dominio; controles "×" na
   status bar do Main.qml ao lado de cada indicador.
3. [proximo, sessao dedicada] Quebrar Main.qml em componentes por dominio
   (BuildPanel.qml, TestsPanel.qml, QualityPanel.qml, StatusBar/JobsPopover) —
   ver "Criterios para liberar a grande refatoracao UI" abaixo.
4. Os itens de backend abaixo ficam explicitamente ADIADOS — nenhum bloqueia
   os passos 2-3 acima. So retomar um deles quando uma fatia de UI concreta
   precisar dele:
   - Process Runner interno unificado;
   - JobManager: logRef persistente;
   - Diagnostic comum: ids estaveis, actions, logRef;
   - Environment scan: mais ferramentas, fingerprint, toolchain health;
   - CMake presets/configure/targets;
   - cargo metadata/check/features;
   - Project Health;
   - Settings/Storage com schema e migracao;
   - Risk Engine antes de Configuration Actions amplas.
```

## Criterios para liberar a grande refatoracao UI

Antes de quebrar `Main.qml` em componentes grandes, idealmente:

```text
[x] build/test/quality jobs estao consumidos corretamente pela UI atual
    (patch minimo concluido 2026-07-05; nao e a refatoracao visual final).
[x] Jobs tem contrato estavel para status/cancelamento (job.cancel no backend
    + botao de cancelar na UI, os dois desde 2026-07-05).
[x] Diagnostics tem modelo comum inicial; ainda faltam ids/actions/logRef.
[ ] CMake/Cargo tem contratos suficientes para toolbar/profile/target.
[ ] Project Health tem payload inicial.
[ ] Settings/storage tem schemaVersion.
[ ] Risk/confirmacao esta definido para acoes automaticas.
[ ] ContextoIA.md aponta a proxima etapa sem depender de memoria de sessao.
```

## Handoff rapido para agentes

Se voce e uma IA continuando o projeto:

```text
- Nao existe mais docs/archive/ (removido de proposito em 2026-07-05); nao
  recriar uma pasta de arquivo morto.
- Leia ContextoIA.md primeiro.
- Leia docs/ARCHITECTURE.md e docs/03-ipc-protocol.md antes de mudar backend.
- Use este arquivo para manter a ponte com a futura UI.
- Backend novo deve nascer no dominio certo e com testes.
- UI futura deve obedecer docs/specs, mas nao deve dirigir regra de negocio.
- Ao terminar uma entrega, atualize ContextoIA.md e este roadmap.
```
