# Kinein Vectis / Kernwerk Studio — Implementation Tasks

> **Tipo:** plano pragmático de tarefas para os próximos meses.  
> **Base:** documentos atuais do repositório Kernwerk Studio + nova direção Kinein Vectis.  
> **Stack confirmada:** Rust no backend/core e Qt6/QML no frontend.  
> **Restrição:** este documento não altera código. Ele organiza execução, prioridades, dependências e critérios de aceite.

---

## 1. Nota de alinhamento de nome

Os documentos atuais do repositório ainda usam:

```text
Kernwerk Studio
KW
kernwerk-core
kernwerk-studio
```

A visão nova usa:

```text
Kinein Vectis
KV
Kinein
```

Decisão pragmática:

```text
Não renomear código, crates, binários, schemas e paths agora.
```

Motivo:

```text
renomear cedo demais gera ruído;
quebra scripts;
atrapalha a estabilidade;
não entrega valor funcional imediato.
```

Regra:

```text
Tratar Kinein Vectis como direção de produto/identidade futura.
Tratar Kernwerk Studio como estado real atual do repositório.
```

A migração de nome deve ser uma milestone própria e futura, somente depois de:

```text
core estável;
UI estável;
gate verde;
release local funcionando;
documentação consolidada.
```

---

## 2. Premissas técnicas vigentes

A arquitetura permanece:

```text
Qt6/QML Frontend
  ↕ JSON-RPC local / stdio
Rust Core
  ↕ ferramentas externas
```

Regras inegociáveis:

```text
UI não chama ferramentas externas diretamente.
Core orquestra ferramentas.
Operações longas viram tarefas/jobs.
Tudo que altera arquivo precisa de preview quando não for edição direta.
IA é externa via terminal dedicado.
Sem provider/chat embutido no MVP.
Sem hard cap de memória.
Resource-on-Demand é padrão da arquitetura.
```

---

## 3. Estado real usado como ponto de partida

O repositório já possui uma base significativa.

Estado atual resumido:

```text
core.ping
command.list
tools.detect / tools.status
workspace.browse/open/status/close
fs.list/read/write/createFile/createDirectory/rename/delete/findFiles/search
build.run
test.run
quality.run
run.start/stdin/stop
terminal.open/input/close
lsp.didChange/definition/hover/completion/references/rename/semanticTokens
```

O protocolo atual registrado nos documentos é:

```text
0.19.0
```

Também já existe:

```text
Qt/QML UI subindo core como processo filho;
workspace selector próprio via core;
Project tree funcional;
abas/salvamento;
build/test/quality básicos;
terminal com PTY real;
run panel;
diagnósticos LSP no Problems;
go to definition;
hover;
completion;
rename;
find usages;
semantic tokens;
rename/delete/move no explorer;
gate único scripts/verificar.sh;
reorganização parcial do core.
```

---

## 4. Filosofia da execução

A partir daqui, a prioridade não é criar mais visão.

A prioridade é:

```text
estabilizar;
reorganizar;
unificar;
medir;
entregar pequenos blocos funcionais;
dogfooding;
não duplicar sistemas.
```

Regra:

```text
Toda nova feature deve reutilizar o sistema existente.
```

Não criar:

```text
novo cliente IPC;
novo protocolo paralelo;
novo painel Problems;
novo executor de processo;
novo LSP manager;
novo terminal paralelo;
nova política de IA embutida.
```

---

## 5. Prioridades por horizonte

### 5.1 Agora — estabilização e arquitetura

```text
P0 — não quebrar o que já funciona;
P0 — modularizar dívida técnica restante;
P0 — manter gate verde;
P0 — separar UI grande em componentes;
P0 — preparar Task Manager/Process Runner sem big bang.
```

### 5.2 Próximas semanas — dogfooding real

```text
P1 — Build misto Cargo + CMake;
P1 — run configurations;
P1 — Git MVP;
P1 — code actions LSP;
P1 — C++ quality via clang-tidy/clang-format;
P1 — polishing do terminal.
```

### 5.3 Próximos meses — experiência profissional

```text
P2 — Task Manager visual;
P2 — Unified Diagnostics;
P2 — First Run / Toolchain Setup;
P2 — Project Health;
P2 — Configuration Actions MVP;
P2 — Resource Manager / Background Services;
P2 — EditorDocument mais robusto.
```

### 5.4 Pós-V1

```text
P3 — Quality Center completo;
P3 — Configuration Actions avançadas;
P3 — refatoração build-aware;
P3 — debug visual DAP;
P3 — embedded targets;
P3 — rename/migração oficial para Kinein Vectis, se mantida a decisão.
```

---

# 6. Milestone 0 — Consolidação antes de novas features

## Objetivo

Garantir que os documentos e o estado real do repositório não se contradizem antes de crescer.

## Tarefas

### M0.1 — Atualizar índice de precedência

**Tipo:** docs  
**Prioridade:** P0  
**Dependências:** nenhuma

Ações:

```text
confirmar que ContextoIA.md é fonte de verdade de estado real;
confirmar que docs numerados são contratos vigentes;
marcar pacotes planning/quality/subsystems como referência;
registrar que Kinein Vectis é direção futura, não rename imediato.
```

Critério de aceite:

```text
um agente novo sabe o que vale mais quando documentos discordam.
```

---

### M0.2 — Corrigir política de IA

**Tipo:** docs/produto  
**Prioridade:** P0

Ações:

```text
atualizar docs/12-ai-policy.md;
substituir "painel chat/provider" por "AI Terminal Bridge";
registrar que Claude/Codex/GPT CLI rodam em terminal separado;
garantir que nada é enviado sem ação explícita.
```

Critério de aceite:

```text
não existe ambiguidade sobre IA embutida.
```

---

### M0.3 — Separar V1 / Pós-V1 / Futuro

**Tipo:** docs/produto  
**Prioridade:** P0

Ações:

```text
marcar docs/16 compiler modes/function store como pós-V1;
marcar Configuration Actions avançadas como pós-MVP;
manter C/C++ e Rust como foco curto prazo;
Java/Python apenas pos-V1 ou retomada futura.
```

Critério de aceite:

```text
nenhum agente tenta implementar loja de funções antes da IDE ficar funcional.
```

---

# 7. Milestone 1 — Reorganização de engenharia sem mudança de comportamento

## Objetivo

Reduzir dívida técnica sem adicionar features grandes.

## Tarefas

### M1.1 — Modularizar `lsp.rs`

**Tipo:** arquitetura/core  
**Prioridade:** P0  
**Dependências:** gate verde atual

Ações conceituais:

```text
separar lifecycle do servidor LSP;
separar transporte LSP;
separar parsing/conversão de mensagens;
separar diagnostics;
separar semantic tokens;
separar requests correlacionadas;
manter API pública do core estável.
```

Critério de aceite:

```text
comportamento externo igual;
testes verdes;
nenhum novo LSP manager paralelo;
nenhuma mudança de protocolo desnecessária.
```

---

### M1.2 — Modularizar `CoreClient`

**Tipo:** arquitetura/UI  
**Prioridade:** P0

Ações conceituais:

```text
separar chamadas por domínio:
workspace;
fs;
build/test/quality;
lsp;
run;
terminal;
tools;
settings futuro.
```

Critério de aceite:

```text
Main.qml reduz acoplamento;
CoreClient deixa de parecer God object;
sem quebrar sinais existentes.
```

---

### M1.3 — Quebrar `Main.qml` em componentes

**Tipo:** UI/UX  
**Prioridade:** P0

Componentes sugeridos:

```text
AppShell.qml
TopBar.qml
ActivityRail.qml
ProjectPanel.qml
EditorArea.qml
EditorTabs.qml
BottomToolWindow.qml
ProblemsPanel.qml
BuildPanel.qml
RunPanel.qml
TerminalPanel.qml
ToolsPanel.qml
RightAssistantShell.qml
StatusBar.qml
CommandPalette.qml
```

Critério de aceite:

```text
nenhuma regra de negócio vai para QML;
componentes continuam visuais;
IPC continua no client;
layout fica mais fácil de manter.
```

---

### M1.4 — Garantir gate único como contrato

**Tipo:** qualidade/processo  
**Prioridade:** P0

Ações:

```text
usar scripts/verificar.sh como gate oficial;
documentar rápido vs completo;
não atualizar binários release se gate falhar;
rodar gate antes de finalizar entregas de desenvolvimento.
```

Critério de aceite:

```text
não existe validação manual solta como fonte primária.
```

---

# 8. Milestone 2 — Task Manager e Process Runner incremental

## Objetivo

Unificar execução de processos sem reescrever tudo de uma vez.

## Tarefas

### M2.1 — Inventariar executores existentes

**Tipo:** arquitetura/core  
**Prioridade:** P0

Mapear:

```text
build.run;
test.run;
quality.run;
run.start;
terminal.open;
lsp process management;
tools.detect;
future git/status.
```

Critério de aceite:

```text
documento curto mostra onde cada processo nasce hoje.
```

---

### M2.2 — Definir `Task` como contrato interno

**Tipo:** arquitetura/protocolo  
**Prioridade:** P1

Campos mínimos:

```text
task_id;
command_id;
title;
category;
status;
workspace;
can_cancel;
safety;
started_at;
logs_ref;
```

Estados:

```text
queued;
running;
waiting_for_confirmation;
succeeded;
failed;
cancel_requested;
cancelled;
skipped.
```

Critério de aceite:

```text
build/test/quality podem ser representados como Task sem mudar UI ainda.
```

---

### M2.3 — Criar eventos `task.*` como camada futura

**Tipo:** IPC/arquitetura  
**Prioridade:** P1

Eventos:

```text
task.queued;
task.started;
task.progress;
task.output;
task.warning;
task.error;
task.confirmationRequired;
task.cancelRequested;
task.cancelled;
task.failed;
task.succeeded;
task.finished.
```

Critério de aceite:

```text
eventos antigos podem coexistir;
não quebrar event.build/test/quality atuais de uma vez.
```

---

### M2.4 — UI: Task popup discreto

**Tipo:** UI/UX  
**Prioridade:** P1

Ações:

```text
status bar mostra tarefas ativas;
popup lista tarefas;
botão cancel quando possível;
logs clicáveis;
sem modal automático.
```

Critério de aceite:

```text
usuário entende o que está rodando sem poluição visual.
```

---

# 9. Milestone 3 — Resource-on-Demand real

## Objetivo

Implementar a estratégia de performance como comportamento natural.

## Tarefas

### M3.1 — Background Services view

**Tipo:** UI/UX  
**Prioridade:** P1

Mostrar discretamente:

```text
core connected;
Tree-sitter;
clangd;
rust-analyzer;
Project Health;
CMake/Cargo status;
terminal;
tasks.
```

Critério de aceite:

```text
usuário consegue ver o que está ativo sem abrir logs brutos.
```

---

### M3.2 — Lifecycle de recursos

**Tipo:** arquitetura/core  
**Prioridade:** P1

Estados:

```text
not_loaded;
warm;
active;
background;
sleeping;
evicted;
failed.
```

Aplicar primeiro em:

```text
LSP;
Configuration Actions registry;
Project Health;
docs cache futuro;
AI context files;
terminal sessions.
```

Critério de aceite:

```text
serviços pesados têm estado explicável.
```

---

### M3.3 — Performance profiles

**Tipo:** settings/UI  
**Prioridade:** P2

Perfis:

```text
Responsive;
Balanced;
Full Intelligence;
Large Workspace.
```

Critério de aceite:

```text
perfil muda quando ativar profundidade, não bloqueia recursos.
```

---

# 10. Milestone 4 — UI/UX base profissional

## Objetivo

Fazer a UI parecer uma IDE coesa, não uma coleção de telas.

## Tarefas

### M4.1 — Tokens visuais únicos

**Tipo:** UI/design system  
**Prioridade:** P0

Ações:

```text
centralizar cores;
centralizar spacing;
centralizar radius;
centralizar tipografia;
remover cores soltas.
```

Critério de aceite:

```text
novos componentes não usam cor hardcoded fora de token.
```

---

### M4.2 — Project tree polish

**Tipo:** UI/UX  
**Prioridade:** P1

Ações:

```text
reduzir peso visual;
hover sutil;
seleção clara;
indentação confortável;
ícones pequenos;
context menu limpo;
expandir/recolher diretório sem trocar workspace.
```

Critério de aceite:

```text
parece Project View de IDE, não gerenciador de arquivos pesado.
```

---

### M4.3 — Top bar e status bar

**Tipo:** UI/UX  
**Prioridade:** P1

Top bar deve mostrar:

```text
workspace;
build target/profile;
run button;
build/test/quality;
terminal;
git branch futuro;
task status.
```

Status bar deve mostrar:

```text
core;
LSP;
diagnostics;
line/column;
encoding/line ending;
active task;
toolchain/target futuro.
```

Critério de aceite:

```text
estado principal visível sem abrir painéis.
```

---

# 11. Milestone 5 — Editor MVP hardening

## Objetivo

Deixar o editor seguro e confiável antes de recursos avançados.

## Tarefas

### M5.1 — Formalizar `EditorDocument`

**Tipo:** editor/UI model  
**Prioridade:** P1

Campos:

```text
id;
path;
language;
encoding;
lineEnding;
isDirty;
isReadOnly;
version;
lastSavedVersion.
```

Critério de aceite:

```text
LSP usa versionamento consistente;
dirty state é confiável;
salvamento não fica implícito.
```

---

### M5.2 — Alteração externa e salvamento seguro

**Tipo:** editor/filesystem  
**Prioridade:** P1

Ações:

```text
detectar arquivo alterado fora da IDE;
avisar antes de sobrescrever;
oferecer reload;
oferecer compare futuro;
preservar line endings.
```

Critério de aceite:

```text
IDE nunca sobrescreve mudança externa silenciosamente.
```

---

### M5.3 — Large File Mode

**Tipo:** editor/performance  
**Prioridade:** P2

Ações:

```text
detectar arquivo grande;
desativar recursos pesados;
avisar discretamente;
manter edição básica.
```

Critério de aceite:

```text
arquivo grande não trava UI.
```

---

# 12. Milestone 6 — Unified Diagnostics e Error UX

## Objetivo

Unificar build, test, quality, LSP e tool errors em uma UX profissional.

## Tarefas

### M6.1 — Modelo unificado de diagnóstico

**Tipo:** protocolo/core  
**Prioridade:** P1

Campos:

```text
id;
severity;
source;
category;
message;
file;
range;
command;
target;
actions;
log_ref.
```

Critério de aceite:

```text
Problems consegue agrupar diagnósticos por origem e severidade.
```

---

### M6.2 — Problems panel 2.0

**Tipo:** UI/UX  
**Prioridade:** P1

Filtros:

```text
Errors;
Warnings;
Info;
Current file;
Build;
LSP;
Quality;
CMake;
Cargo.
```

Ações rápidas:

```text
open file;
open log;
run configure;
restart LSP;
open toolchain setup;
open AI Terminal.
```

Critério de aceite:

```text
erro bom ensina o usuário a resolver.
```

---

### M6.3 — Tool missing UX

**Tipo:** UI/UX/tooling  
**Prioridade:** P1

Exemplo:

```text
clangd não encontrado.
Impacto: C/C++ terá autocomplete e diagnostics limitados.
Ações: selecionar binário, abrir setup, continuar sem clangd.
```

Critério de aceite:

```text
ferramenta ausente nunca vira erro obscuro.
```

---

# 13. Milestone 7 — First Run e Toolchain Setup

## Objetivo

Permitir que a IDE explique o ambiente local sem instalar nada silenciosamente.

## Tarefas

### M7.1 — First Run screen

**Tipo:** UI/UX  
**Prioridade:** P2

Mostrar:

```text
C/C++ tools;
Rust tools;
Git;
optional tools;
missing tools;
impacto;
ações.
```

Critério de aceite:

```text
usuário sabe o que falta e pode continuar com suporte limitado.
```

---

### M7.2 — Toolchain profiles

**Tipo:** core/config/UI  
**Prioridade:** P2

Gerar/editar:

```text
.kernwerk/toolchains.json
```

Suportar:

```text
System Clang;
System GCC;
Rust stable;
custom binary paths.
```

Critério de aceite:

```text
toolchain ativa é explícita e auditável.
```

---

# 14. Milestone 8 — CMake/Cargo dogfooding profissional

## Objetivo

Tornar a IDE boa o suficiente para uso diário em C/C++ e Rust.

## Tarefas

### M8.1 — Build misto Cargo + CMake

**Tipo:** build/core/UI  
**Prioridade:** P1

Ações:

```text
detectar workspace com Cargo.toml e CMakeLists.txt;
permitir escolher build system ativo;
Ctrl+F9 roda build correto;
mostrar ambos quando configurado como mixed.
```

Critério de aceite:

```text
projeto mixed não é tratado como apenas Cargo ou apenas CMake por acidente.
```

---

### M8.2 — Run configurations

**Tipo:** run/core/UI  
**Prioridade:** P1

Campos:

```text
name;
kind;
command/program;
args;
cwd;
env;
build_before_run;
target.
```

Critério de aceite:

```text
usuário não depende de botão mágico para todo projeto.
```

---

### M8.3 — C++ Quality MVP

**Tipo:** quality/core/UI  
**Prioridade:** P1

Ações:

```text
clang-format check;
clang-tidy check;
diagnósticos no Problems;
logs locais;
mensagens claras.
```

Critério de aceite:

```text
quality.run não é só RustCargo.
```

---

# 15. Milestone 9 — Git MVP

## Objetivo

Adicionar Git sem transformar a IDE em cliente Git gigante.

## Tarefas

### M9.1 — Git status

**Tipo:** core/tooling/UI  
**Prioridade:** P1

Ações:

```text
detectar repo;
branch atual;
arquivos modificados;
untracked;
staged/unstaged;
debounce.
```

Critério de aceite:

```text
status aparece sem travar UI.
```

---

### M9.2 — Stage/unstage e commit básico

**Tipo:** UI/core  
**Prioridade:** P2

Ações:

```text
stage file;
unstage file;
commit message;
commit;
mostrar erro claro.
```

Critério de aceite:

```text
fluxo básico de commit funciona sem terminal, mas terminal continua disponível.
```

---

# 16. Milestone 10 — AI Terminal Bridge

## Objetivo

Substituir a ideia de painel de IA embutida por terminal externo dedicado.

## Tarefas

### M10.1 — AI Terminal separado

**Tipo:** UI/terminal  
**Prioridade:** P1

Ações:

```text
aba separada do Terminal comum;
perfil de comando configurável;
Claude/Codex/GPT CLI por comando externo;
badge "external";
sem provider interno.
```

Critério de aceite:

```text
usuário sabe que a IA é ferramenta externa.
```

---

### M10.2 — Context Builder básico

**Tipo:** core/UI/security  
**Prioridade:** P2

Ações:

```text
contexto de seleção;
contexto de erro de build;
preview antes de enviar;
sanitização de secrets;
copiar para clipboard ou passar arquivo.
```

Critério de aceite:

```text
nada é enviado para IA externa sem ação explícita.
```

---

# 17. Milestone 11 — Configuration Actions MVP

## Objetivo

Criar a primeira versão da camada visual de configuração sem cair em loja gigante.

## Tarefas

### M11.1 — Registry de actions

**Tipo:** core/config  
**Prioridade:** P2

Campos:

```text
id;
title;
scope;
category;
level;
description;
affects;
risk;
requires;
docs;
```

Critério de aceite:

```text
lista carrega rápido sem calcular preview de tudo.
```

---

### M11.2 — Scoped UI

**Tipo:** UI/UX  
**Prioridade:** P2

Escopos:

```text
CMake-only;
Cargo-only;
Mixed.
```

Critério de aceite:

```text
projeto CMake não mostra Cargo por padrão;
projeto Cargo não mostra CMake por padrão;
Mixed mostra ambos.
```

---

### M11.3 — Primeiras actions

**Tipo:** core/UI  
**Prioridade:** P2

CMake:

```text
Enable compile_commands.json;
Create Debug preset;
Create Release preset;
Add include directory;
Add target_link_libraries.
```

Cargo:

```text
Add dependency;
Add dev-dependency;
Add feature;
Run cargo check.
```

Critério de aceite:

```text
toda alteração mostra preview/diff.
```

---

# 18. Milestone 12 — LSP Code Actions e Refatoração inicial

## Objetivo

Aproximar a IDE do comportamento de IDE profissional sem tentar refatoração profunda cedo demais.

## Tarefas

### M12.1 — Code actions LSP

**Tipo:** LSP/editor/UI  
**Prioridade:** P1

Ações:

```text
lsp.codeActions;
quick fix popup;
aplicar workspace edit com preview quando alterar arquivo;
organize imports/includes quando disponível.
```

Critério de aceite:

```text
quick fixes reais aparecem no editor sem UI invasiva.
```

---

### M12.2 — Refactor preview simples

**Tipo:** editor/refactoring  
**Prioridade:** P2

Ações:

```text
mostrar arquivos afetados;
mostrar diff textual;
permitir aplicar/cancelar;
rollback simples quando possível.
```

Critério de aceite:

```text
nenhuma refatoração não trivial aplica mudança no escuro.
```

---

### M12.3 — Build-aware refactoring futuro

**Tipo:** pós-V1  
**Prioridade:** P3

Exemplos:

```text
mover source e atualizar CMake;
criar library target;
renomear target;
adicionar source ao target;
atualizar Cargo workspace.
```

Critério de aceite:

```text
não bloquear V1 com isso.
```

---

# 19. O que não fazer agora

Não implementar agora:

```text
renomear todo o projeto para Kinein;
plugin system público;
marketplace;
IA provider interno;
chat lateral embutido;
debug visual completo;
DAP completo;
embedded completo;
QEMU visual;
loja de funções avançada;
CMake parser universal;
indexador semântico próprio;
multi-root workspace avançado;
SQLite index persistente antes da V1;
Java/Python curto prazo.
```

---

# 20. Ordem recomendada de execução

Se for trabalhar nos próximos meses, seguir esta sequência:

```text
1. M0 — consolidar docs e política de IA.
2. M1 — modularizar lsp.rs, CoreClient e Main.qml.
3. M2 — Task Manager/Process Runner incremental.
4. M4 — UI shell/components/tokens.
5. M6 — Unified Diagnostics/Problems.
6. M8 — Build mixed, run configs e C++ quality.
7. M9 — Git MVP.
8. M12.1 — Code actions LSP.
9. M10 — AI Terminal Bridge.
10. M7 — First Run/Toolchain Setup.
11. M11 — Configuration Actions MVP.
12. M5 — Editor hardening contínuo em paralelo.
```

Observação:

```text
M5 Editor hardening deve acontecer continuamente,
mas sem travar features de dogfooding essenciais.
```

---

# 21. Critério de “pronto para V1”

A V1 pode ser considerada realista quando:

```text
[ ] projeto abre/fecha sem perder estado;
[ ] editor é confiável para arquivos comuns;
[ ] build/test/quality funcionam para Rust e CMake;
[ ] run configurations existem;
[ ] terminal real funciona;
[ ] LSP C/C++ e Rust são úteis;
[ ] Problems mostra diagnósticos unificados;
[ ] Git básico funciona;
[ ] toolchain setup explica ambiente;
[ ] UI é consistente e confortável;
[ ] AI Terminal é externo e opcional;
[ ] gate completo passa;
[ ] a IDE consegue ser usada no próprio repositório.
```

---

# 22. Frase final

```text
A próxima fase não é imaginar mais a IDE.
É fazer o Kernwerk atual ficar sólido o suficiente
para virar Kinein Vectis com segurança.
```
