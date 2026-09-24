# Kinein Vectis — Parte 9: Arquitetura Interna da IDE

> **Revisão vinculante de 2026-09-22:** nenhuma camada, serviço, job ou boundary
> de Assistente/IA integra o produto. Não há telemetria de produto/usuário.
> Ferramentas externas escolhidas pelo usuário podem rodar no terminal sem
> participar da arquitetura da IDE. Ver
> `arquitetura-de-frontend-0.3-em-diante.md`.

> **Escopo:** Rust Core, Qt/QML Frontend, IPC, Jobs, Events, Storage, Services e Plugin Boundaries.  
> **Decisão de produto:** a Kinein deve **facilitar sem ser intrusiva/invasiva**.  
> **Decisão arquitetural:** a interface deve ser calma e responsiva; o core deve ser determinístico, auditável e comandado pelo usuário.

---

## 1. Objetivo desta parte

A Parte 9 define como a Kinein Vectis conversa por dentro.

Até aqui foram definidos:

```text
Parte 1  — Iconografia
Parte 2  — Layout principal
Parte 3  — Componentes UI
Parte 4  — Fluxos Build/Run/Debug
Parte 5  — Editor, LSP, Tree-sitter e diagnósticos
Parte 6  — Embedded, Targets, Flash, Serial, Remote SSH e QEMU
Parte 7  — Assistente original (cancelado)
Parte 7.1 — AI CLI Bridge externo (cancelado)
Parte 8  — Onboarding, Settings e Project Wizard
Parte 8.1 — Setup Intelligence Optimization Layer
```

Agora precisamos definir a espinha dorsal:

```text
UI Qt/QML
  ↕ IPC local
Rust Core
  ↕ Services internos
Jobs / Events / Storage / Tooling
```

A meta é evitar que a IDE vire um emaranhado de botões chamando comandos aleatórios.

---

## 2. Princípio central

A Kinein deve seguir esta regra:

```text
A UI apresenta, o Core decide, os Services executam, os Jobs registram, os Events notificam.
```

Isso significa:

```text
- QML não deve chamar cmake diretamente;
- QML não deve manipular CMakePresets sozinho;
- QML não deve iniciar clangd diretamente;
- QML não deve executar flash/debug diretamente;
- QML não deve rodar IA CLI diretamente;
- UI envia intenção;
- Core valida;
- Core cria job;
- Job executa;
- Events atualizam UI.
```

Essa separação é o que permite a IDE crescer sem virar bagunça.

---

## 3. Frase-guia de arquitetura

```text
Facilitar sem invadir:
a IDE observa o estado, sugere caminhos e executa ações explícitas,
mas não interrompe nem toma controle do usuário.
```

Implica:

```text
- sem pop-ups agressivos;
- sem auto-fix silencioso;
- sem build automático pesado sem pedido;
- sem IA embutida;
- sem modificação automática de projeto existente;
- sem terminal escondido executando comandos críticos;
- sem ações high-risk sem confirmação.
```

---

## 4. Modelo geral de processos

### 4.1 Opção recomendada

Para a Kinein, a arquitetura mais saudável é:

```text
Qt/QML Frontend como processo de interface
Rust Core como processo local/controlador
Comunicação por IPC local usando JSON-RPC + eventos
```

Fluxo:

```text
Kinein UI
  ↓ request JSON-RPC
Kinein Core
  ↓ cria job / chama service
Tooling externo
  ↓ logs/status
Kinein Core
  ↓ event stream
Kinein UI
```

---

### 4.2 Por que separar UI e Core

Benefícios:

```text
- UI não trava se build/debug/scan ficarem pesados;
- Core pode ser testado sem UI;
- IA CLI Bridge pode ser testado sem QML;
- comandos podem virar CLI no futuro;
- possível headless mode;
- melhor isolamento de responsabilidades;
- mais fácil integrar com Codex/Claude CLI externamente;
- mais fácil debugar IPC do que lógica escondida no QML.
```

---

### 4.3 MVP possível

No MVP, para reduzir complexidade, a UI pode iniciar o Core como child process:

```text
kinein-ui
  └── kinein-core --stdio
```

Comunicação:

```text
stdin/stdout JSON-RPC
```

Futuro:

```text
Unix Domain Socket no Linux
Named Pipe no Windows, se houver suporte futuro
TCP local apenas para debugging
```

---

## 5. Camadas arquiteturais

A Kinein deve ter camadas explícitas.

```text
┌──────────────────────────────────────────────┐
│ Qt/QML Frontend                              │
│ views, components, layout, user interactions │
└───────────────────────┬──────────────────────┘
                        │ JSON-RPC + Events
┌───────────────────────▼──────────────────────┐
│ Rust Core                                    │
│ command routing, validation, state, events   │
└───────────────────────┬──────────────────────┘
                        │ internal service API
┌───────────────────────▼──────────────────────┐
│ Services                                     │
│ workspace, setup, cmake, cargo, lsp, etc.    │
└───────────────────────┬──────────────────────┘
                        │ job execution
┌───────────────────────▼──────────────────────┐
│ External Tools                               │
│ cmake, ninja, cargo, clangd, gdb, qemu...    │
└──────────────────────────────────────────────┘
```

---

## 6. Responsabilidades da UI Qt/QML

A UI deve cuidar de:

```text
- layout;
- navegação visual;
- componentes;
- renderização do editor;
- interação do usuário;
- exibição de estado;
- painéis;
- status bar;
- command palette;
- terminal visual;
- preview de contexto;
- preview de diff;
- feedback visual de jobs.
```

A UI não deve cuidar de:

```text
- regras de CMake;
- execução de comandos externos;
- validação profunda de toolchain;
- parser de logs;
- persistência principal;
- controle de jobs;
- decisão de risco;
- sanitização de contexto;
- mutações de arquivo sem passar pelo Core.
```

---

## 7. Responsabilidades do Rust Core

O Core deve cuidar de:

```text
- roteamento JSON-RPC;
- validação de comandos;
- gerenciamento de workspace;
- gerenciamento de estado;
- criação/cancelamento de jobs;
- emissão de eventos;
- execução de serviços;
- persistência;
- leitura/escrita de arquivos;
- health checks;
- integração com ferramentas externas;
- limites de segurança;
- confirmação de ações;
- logging estruturado.
```

O Core é a parte que deve ser mais testável.

---

## 8. Serviços principais do Core

### 8.1 Workspace Service

Responsável por:

```text
abrir workspace
fechar workspace
detectar tipo de projeto
manter workspace_id
resolver caminhos
carregar .kinein/workspace.json
expor árvore de arquivos
```

Métodos:

```text
workspace.open
workspace.close
workspace.getState
workspace.refresh
workspace.listFiles
```

---

### 8.2 Project Detection Service

Responsável por:

```text
detectar CMakeLists.txt
detectar CMakePresets.json
detectar Cargo.toml
detectar compile_commands.json
detectar .git
detectar DocsPublic/
gerar Project Fingerprint
```

Métodos:

```text
project.detect
project.fingerprint
project.health
```

---

### 8.3 Setup Intelligence Service

Responsável pela Parte 8.1:

```text
Environment Fingerprint
Project Fingerprint
Capability Matrix
Configuration Graph
Setup Plan
Setup Preview
Setup Apply
Repair Engine
Rollback
```

Métodos:

```text
environment.fingerprint
project.fingerprint
setup.plan
setup.preview
setup.apply
setup.repair
configGraph.get
```

---

### 8.4 Toolchain Service

Responsável por:

```text
detectar GCC/Clang
detectar CMake/Ninja/Make
detectar GDB/LLDB
detectar Rust/Cargo/Rust analyzer
detectar cross compilers
rodar health checks
calcular score de toolchain
persistir toolchains locais
```

Métodos:

```text
toolchain.scan
toolchain.list
toolchain.get
toolchain.healthCheck
toolchain.setDefault
```

---

### 8.5 CMake Service

Responsável por:

```text
ler CMakePresets.json
criar presets seguros
rodar cmake configure
rodar cmake build
validar CMake cache
detectar build dirs stale
gerar compile_commands.json
parsear erros de configure
```

Métodos:

```text
cmake.presets.list
cmake.configure
cmake.build
cmake.cache.inspect
cmake.cache.clear
cmake.createPreset
```

---

### 8.6 Cargo/Rust Service

Responsável por:

```text
detectar Cargo.toml
rodar cargo metadata
rodar cargo check
rodar cargo build
rodar cargo test
integrar rust-analyzer
parsear erros básicos
```

Métodos:

```text
cargo.metadata
cargo.check
cargo.build
cargo.test
rust.toolchain.status
```

---

### 8.7 Language Service

Responsável por:

```text
gerenciar clangd
gerenciar rust-analyzer
gerenciar Tree-sitter
unificar diagnósticos
fornecer símbolos
hover
go to definition
rename
format
code actions
```

Métodos:

```text
language.status
language.diagnostics
language.symbols
language.hover
language.definition
language.references
language.rename
language.format
language.codeActions
```

---

### 8.8 Build Service

Responsável por:

```text
criar job de build
acompanhar progresso
coletar logs
parsear erro principal
emitir eventos
expor histórico
```

Métodos:

```text
build.run
build.cancel
build.status
build.logs
build.history
```

---

### 8.9 Run Service

Responsável por:

```text
listar executáveis
criar run configs
executar binário
gerenciar working directory
env vars
args
target local/remoto
```

Métodos:

```text
runConfig.list
runConfig.create
runConfig.update
run.start
run.stop
```

---

### 8.10 Debug Service

Responsável por:

```text
GDB/LLDB
debug configs
debug session
breakpoints
stack frames
variables
threads
console
embedded debug futuro
```

Métodos:

```text
debugConfig.list
debug.start
debug.stop
debug.pause
debug.resume
debug.breakpoints.set
debug.stackTrace
debug.variables
```

---

### 8.11 Terminal Service

Responsável por:

```text
terminal comum
processos PTY
shell profiles
session ids
logs
working directory
terminal kind
```

Tipos:

```text
terminal_kind: normal
terminal_kind: build
terminal_kind: debug
terminal_kind: serial
```

Métodos:

```text
terminal.open
terminal.write
terminal.resize
terminal.close
terminal.list
```

---

### 8.12 Reserva removida

O antigo `AI CLI Bridge Service` foi cancelado. Não reservar serviço, comandos
IPC, tipo de terminal, configuração ou estado persistido para essa finalidade.
Ferramentas que o usuário execute no terminal comum não recebem integração
privilegiada com o contexto da IDE.

---

### 8.13 Target Service

Responsável por:

```text
local target
remote SSH target
QEMU target
bare-metal target
serial ports
flash strategy
deploy path
```

Métodos:

```text
target.list
target.create
target.health
target.connect
target.disconnect
target.deploy
```

---

### 8.14 Settings Service

Responsável por:

```text
global settings
workspace settings
project settings
toolchains
targets
keymap
theme
AI bridge profiles
```

Métodos:

```text
settings.get
settings.set
settings.schema
settings.search
settings.reset
```

---

### 8.15 Storage Service

Responsável por:

```text
arquivos JSON
cache
logs
session state
recent projects
layout state
temporary context files
```

---

## 9. Job System

A Kinein precisa de um Job System forte.

### 9.1 Por quê

Operações longas não podem travar a UI:

```text
environment scan
project indexing
cmake configure
cmake build
cargo build
clangd indexing
qemu boot
flash
debug launch
remote deploy
AI context generation
```

Tudo isso deve virar job.

---

### 9.2 Modelo de Job

```json
{
  "job_id": "job_123",
  "kind": "cmake.configure",
  "title": "Configure CMake: debug",
  "workspace_id": "workspace_001",
  "status": "running",
  "progress": 0.42,
  "started_at": "2026-07-04T18:30:00",
  "can_cancel": true,
  "risk": "low"
}
```

---

### 9.3 Estados

```text
queued
running
waiting_for_confirmation
success
warning
failed
cancelled
skipped
```

---

### 9.4 Regras

```text
- todo job tem id;
- todo job emite eventos;
- todo job pode ter logs;
- job cancelável deve expor cancel;
- job crítico deve ter confirmação antes de iniciar;
- UI nunca assume sucesso sem event do Core.
```

---

## 10. Event System

Eventos mantêm a UI atualizada sem polling excessivo.

### 10.1 Eventos principais

```text
workspace.opened
workspace.closed
project.detected
project.health.updated
environment.scan.started
environment.scan.finished
toolchain.updated
cmake.configure.started
cmake.configure.finished
build.started
build.progress
build.finished
diagnostics.updated
language.server.status
terminal.opened
terminal.output
terminal.closed
setup.plan.created
setup.apply.finished
aiBridge.context.created
target.status.changed
settings.changed
```

---

### 10.2 Exemplo de evento

```json
{
  "event": "build.finished",
  "payload": {
    "job_id": "job_123",
    "workspace_id": "workspace_001",
    "status": "failed",
    "errors": 3,
    "warnings": 5,
    "main_error": {
      "kind": "linker",
      "message": "undefined reference to MotorDriver::init()",
      "file": "build/debug/CMakeFiles/app.dir/main.cpp.o"
    }
  }
}
```

---

## 11. IPC: JSON-RPC local

### 11.1 Por que JSON-RPC

Vantagens:

```text
simples
debugável
testável
compatível com stdio
compatível com socket
boa separação UI/Core
fácil logar
fácil mockar
```

### 11.2 Formato de request

```json
{
  "id": "req_001",
  "method": "cmake.configure",
  "params": {
    "workspace_id": "workspace_001",
    "preset": "debug"
  }
}
```

### 11.3 Formato de response

```json
{
  "id": "req_001",
  "result": {
    "job_id": "job_123"
  }
}
```

### 11.4 Formato de erro

```json
{
  "id": "req_001",
  "error": {
    "code": "TOOLCHAIN_NOT_FOUND",
    "message": "No valid C++ toolchain found.",
    "details": {
      "missing": ["cxx_compiler"]
    }
  }
}
```

---

## 12. Fluxo de build completo

```text
Usuário clica Build
  ↓
UI envia build.run
  ↓
Core valida workspace/toolchain/preset
  ↓
Core cria job
  ↓
Build Service executa cmake --build
  ↓
Job stream coleta output
  ↓
Parser identifica erros/warnings
  ↓
Core emite build.progress/build.finished
  ↓
UI atualiza status bar, Problems e Build tool window
  ↓
Se falhar, Project Health e AI Bridge podem oferecer ações secundárias
```

Ponto importante:

```text
AI Terminal nunca é aberto automaticamente.
```

---

## 13. Fluxo de AI Terminal correto

```text
Usuário escolhe "Open in AI Terminal"
  ↓
UI envia aiBridge.context.create
  ↓
Core coleta contexto determinístico
  ↓
Core sanitiza
  ↓
Core retorna preview
  ↓
Usuário confirma
  ↓
UI envia aiBridge.terminal.open
  ↓
Core abre terminal_kind=ai com comando configurado
```

A IA é chamada por intenção explícita.

---

## 14. Fluxo de setup correto

```text
Usuário cria novo projeto
  ↓
UI envia setup.plan
  ↓
Core gera plano
  ↓
UI mostra preview
  ↓
Usuário confirma
  ↓
Core aplica plano como jobs
  ↓
Core valida resultado
  ↓
UI abre projeto ou mostra reparo
```

---

## 15. Storage e diretórios

### 15.1 Configuração global

```text
~/.config/kinein/
  settings.json
  toolchains.json
  ai-bridge.json
  keymap.json
  recent-projects.json
```

### 15.2 Cache global

```text
~/.cache/kinein/
  toolchain-scan.json
  DocsPublic/
  indexes/
  context/
  logs/
```

### 15.3 Dados locais de projeto

```text
.kinein/
  workspace.json
  targets.json
  run-configs.json
  debug-configs.json
  layout.json
```

### 15.4 Dados locais não versionáveis

```text
.kinein/session.json
.kinein/toolchains.local.json
.kinein/secrets.json
.kinein/cache/
```

---

## 16. Regras de versionamento

Pode versionar:

```text
.kinein/workspace.json
.kinein/targets.example.json
CMakePresets.json
README.md
DocsPublic/
```

Não versionar:

```text
.kinein/session.json
.kinein/secrets.json
.kinein/toolchains.local.json
context temp files
logs locais
```

---

## 17. Crate structure sugerida

```text
kinein/
├── crates/
│   ├── kinein-core/
│   ├── kinein-protocol/
│   ├── kinein-ipc/
│   ├── kinein-events/
│   ├── kinein-jobs/
│   ├── kinein-storage/
│   ├── kinein-workspace/
│   ├── kinein-projects/
│   ├── kinein-setup/
│   ├── kinein-toolchains/
│   ├── kinein-cmake/
│   ├── kinein-cargo/
│   ├── kinein-language/
│   ├── kinein-terminal/
│   ├── kinein-ai-bridge/
│   ├── kinein-targets/
│   ├── kinein-debug/
│   └── kinein-cli/
├── apps/
│   ├── kinein-core-daemon/
│   └── kinein-ui/
├── ui/
│   └── qml/
└── DocsPublic/
```

---

## 18. `kinein-protocol`

Este crate deve conter os contratos entre UI e Core.

Conteúdo:

```text
request types
response types
event types
error codes
job types
workspace ids
path types
risk levels
terminal kinds
```

Regra:

```text
UI e Core não devem inventar JSON solto.
Tudo deve passar por tipos versionados.
```

---

## 19. Error model

Erros devem ser estruturados.

### 19.1 Categorias

```text
WORKSPACE_ERROR
PROJECT_DETECTION_ERROR
TOOLCHAIN_ERROR
CMAKE_ERROR
CARGO_ERROR
LANGUAGE_SERVICE_ERROR
TERMINAL_ERROR
AI_BRIDGE_ERROR
TARGET_ERROR
STORAGE_ERROR
PERMISSION_ERROR
USER_CANCELLED
```

### 19.2 Exemplo

```json
{
  "code": "CMAKE_CONFIGURE_FAILED",
  "message": "CMake configure failed.",
  "severity": "error",
  "user_action": "Open CMake output",
  "can_repair": true,
  "repair_method": "setup.repair",
  "evidence": {
    "job_id": "job_123",
    "log_range": [120, 147]
  }
}
```

---

## 20. Risk Engine

A arquitetura deve incluir um sistema simples de risco.

### 20.1 Níveis

```text
low
medium
high
dangerous
```

### 20.2 Exemplos low

```text
ler versão de ferramenta
abrir arquivo
rodar cmake --version
rodar cargo metadata
copiar contexto
abrir AI Terminal sem contexto
```

### 20.3 Exemplos medium

```text
criar CMakePresets.json
criar .kinein/workspace.json
criar run config
rodar cmake configure
rodar build
incluir arquivo atual no contexto IA
```

### 20.4 Exemplos high

```text
editar CMakeLists.txt
limpar build directory
rodar script externo
alterar permissões
passar múltiplos arquivos para IA CLI
```

### 20.5 Exemplos dangerous

```text
apagar arquivos fora do workspace
instalar pacote automaticamente
alterar PATH global
executar comando shell arbitrário vindo de IA
mandar secrets para contexto
```

Regras:

```text
high exige confirmação;
dangerous deve ser bloqueado ou exigir confirmação muito explícita;
ações vindas de IA nunca são aplicadas automaticamente.
```

---

## 21. Non-intrusive UX rules conectadas à arquitetura

A arquitetura deve reforçar o comportamento visual.

### 21.1 Events não devem virar pop-up automaticamente

Exemplo:

```text
build.failed
```

Não deve abrir modal gigante.

Deve atualizar:

```text
status bar
Problems
Build tool window
Project Health badge
```

Só abrir painel se o usuário pedir ou se for primeira experiência configurável.

---

### 21.2 Sugestões são discretas

```text
[Open error]
[Open CMake]
[Repair setup]
[Open in AI Terminal]
```

A ação de IA deve ser secundária.

---

### 21.3 Estado visível reduz ansiedade

Status bar deve mostrar:

```text
workspace
branch
build status
diagnostics
toolchain
target
language server
```

Sem precisar abrir 5 telas.

---

### 21.4 Sem automação surpresa

A Kinein pode recomendar:

```text
Run CMake configure?
```

Mas não deve rodar silenciosamente em projeto existente.

---

## 22. Plugin Boundaries

Plugins são importantes no longo prazo, mas perigosos cedo demais.

### 22.1 Decisão para MVP

```text
Sem sistema público de plugins no MVP.
```

Motivo:

```text
- aumenta superfície de segurança;
- complica arquitetura;
- dificulta estabilidade;
- atrapalha foco no core;
- exige API pública madura.
```

### 22.2 Boundaries internos desde cedo

Mesmo sem plugins externos, organizar serviços como se fossem módulos.

```text
workspace service
cmake service
cargo service
language service
terminal service
ai bridge service
target service
```

Isso permite criar plugin API depois.

---

### 22.3 Futuro Plugin API

Possíveis tipos:

```text
language plugin
build system plugin
target plugin
debug adapter plugin
tool window plugin
template plugin
documentation provider plugin
```

### 22.4 Regras futuras

Plugins não devem poder:

```text
executar comando sem permissão;
ler secrets;
alterar arquivos sem preview;
abrir IA externa automaticamente;
bloquear UI;
injetar painel invasivo;
```

---

## 23. Logging

Logs precisam existir desde o início.

### 23.1 Logs do Core

```text
~/.cache/kinein/logs/core.log
```

Conteúdo:

```text
requests
responses
events
jobs
tool invocations
errors
duration
```

Sem secrets.

### 23.2 Logs de Job

```text
~/.cache/kinein/logs/jobs/job_123.log
```

### 23.3 Logs para Support Bundle

Futuro:

```text
support bundle sanitizado
```

---

## 24. Testabilidade

A arquitetura precisa ser testável.

### 24.1 Testes unitários

```text
path resolver
CMake preset parser
toolchain detector
risk engine
context sanitizer
setup plan generator
error parser
```

### 24.2 Testes de integração

```text
abrir projeto CMake fake
rodar cmake configure fake
parsear build log
gerar compile_commands
abrir terminal mock
gerar contexto IA mock
```

### 24.3 Testes de IPC

```text
request válido
request inválido
evento emitido
job lifecycle
erro estruturado
```

---

## 25. CLI complementar

O Core deve permitir uma CLI futura:

```bash
kinein doctor
kinein doctor --workspace .
kinein setup plan
kinein setup apply
kinein context export --build-error
kinein toolchains list
kinein cmake configure
```

Isso ajuda:

```text
debug
CI
usuários avançados
testes
documentação
```

---

## 26. Performance

### 26.1 Regras

```text
UI nunca deve bloquear em scan/build/index.
Jobs pesados são assíncronos.
Eventos devem ser agregados quando muito frequentes.
Logs grandes devem ser streamados.
Árvore de arquivos deve ser lazy.
Indexação deve ter debounce.
```

### 26.2 Cuidado com LSP

```text
clangd e rust-analyzer podem consumir recursos.
A IDE deve mostrar status, mas não reiniciar em loop.
```

---

## 27. Segurança operacional

### 27.1 Princípios

```text
não executar comando sem origem clara;
não passar secrets para contexto;
não editar projeto existente sem confirmação;
não baixar ferramentas automaticamente;
não instalar pacotes automaticamente;
não aceitar comando vindo de IA como ação confiável;
não esconder processo rodando.
```

### 27.2 Terminal

Terminal normal e AI Terminal são processos explícitos.

```text
Toda sessão tem:
- session_id;
- kind;
- command;
- cwd;
- started_at;
- user_initiated: true/false.
```

Ações não iniciadas pelo usuário devem ser raras e visíveis.

---

## 28. Roadmap de implementação

### 28.1 MVP 0.1 — Core mínimo

```text
kinein-protocol
kinein-core
JSON-RPC stdio
workspace.open
core.ping
event stream básico
storage básico
```

### 28.2 MVP 0.2 — Workspace e Toolchain

```text
project.detect
environment.fingerprint
toolchain.scan
settings get/set
Project Health básico
```

### 28.3 MVP 0.3 — CMake/Cargo Jobs

```text
cmake.configure
cmake.build
cargo.metadata
cargo.check
job system
logs
events
```

### 28.4 MVP 0.4 — Language Services

```text
Tree-sitter layer
clangd status
rust-analyzer status
diagnostics
symbols básico
```

### 28.5 MVP 0.5 — Terminal e AI Bridge

```text
terminal.open normal
terminal.open ai
aiBridge.context.create
aiBridge.context.preview
aiBridge.terminal.open
```

### 28.6 MVP 0.6 — Setup Intelligence

```text
setup.plan
setup.preview
setup.apply
repair básico
CMake cache validation
compile_commands validation
```

---

## 29. O que não implementar agora

Para manter o projeto saudável:

```text
- plugin system público;
- IA embutida;
- agente autônomo;
- remote development completo;
- debugger visual completo avançado;
- simulador OpenGL;
- suporte a todos os build systems;
- marketplace;
- instalação automática de toolchains;
- cloud sync;
- multi-user collaboration.
```

---

## 30. Critérios de aceite da Parte 9

A arquitetura estará boa quando:

```text
[ ] UI não executa ferramentas externas diretamente.
[ ] Core expõe JSON-RPC tipado.
[ ] Jobs têm lifecycle claro.
[ ] Events atualizam UI.
[ ] Logs são rastreáveis.
[ ] Storage separa global/workspace/session.
[ ] Toolchain/CMake/Cargo são services independentes.
[ ] AI Bridge não é chat embutido.
[ ] Terminal IA é separado do terminal comum.
[ ] Ações high-risk exigem confirmação.
[ ] Project Health nasce de dados determinísticos.
[ ] Setup Plan é previewável antes de aplicar.
[ ] Arquitetura permite CLI futura.
```

---

## 31. Resumo executivo

A Kinein Vectis deve ser construída como:

```text
Qt/QML Frontend
  ↕ JSON-RPC + Events
Rust Core
  ↕ Services
Jobs + Storage + External Tools
```

A IDE deve facilitar o trabalho sem invadir o fluxo do usuário.

A regra final:

```text
A Kinein não deve ser uma IDE que tenta fazer tudo sozinha.
Ela deve ser uma IDE que mostra o estado real, reduz atrito,
executa ações explícitas e mantém o usuário no controle.
```
