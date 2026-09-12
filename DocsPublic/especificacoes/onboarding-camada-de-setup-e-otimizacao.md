# Kinein Vectis — Parte 8.1: Optimization Layer para Onboarding, Setup Visual e Project Wizard

> **Tipo:** complemento/otimização da Parte 8.  
> **Objetivo:** transformar o onboarding e o setup visual em uma experiência ainda mais inteligente, robusta e profissional.  
> **Nome conceitual:** **Kinein Setup Intelligence Layer**.  
> **Resumo:** em vez de o wizard ser apenas uma sequência fixa de telas, ele deve ser alimentado por uma camada que entende ambiente, projeto, toolchains, targets, riscos e próximos passos.

---

## 1. Por que esta extensão existe

A Parte 8 definiu o fluxo principal: Start Screen, First Run Setup, Project Wizard, Toolchain Manager, CMake Setup, Project Health e Setup Assistant.

Esta Parte 8.1 melhora isso com uma ideia mais forte:

```text
O onboarding da Kinein não deve ser apenas um formulário.
Ele deve ser um sistema de diagnóstico, planejamento e reparo.
```

A diferença prática:

```text
Wizard comum:
Usuário escolhe opções → IDE cria arquivos.

Kinein Setup Intelligence:
IDE detecta ambiente → entende projeto → calcula opções seguras → mostra plano →
usuário revisa → IDE aplica → valida → se falhar, explica e repara.
```

Essa camada é o que pode fazer a Kinein parecer realmente profissional, próxima da sensação JetBrains-like: a IDE parece entender o projeto antes de pedir que o usuário decida tudo manualmente.

---

## 2. Problema real que a Kinein precisa resolver

C, C++ e Rust não são difíceis só pela linguagem. O atrito real costuma estar em:

```text
- compilador errado;
- CMake mal configurado;
- Ninja/Make ausente;
- CMake cache quebrado;
- build directory antigo;
- compile_commands.json ausente;
- clangd sem contexto;
- rust-analyzer não inicializando;
- múltiplas versões de compilador;
- toolchain cross sem sysroot;
- target remoto sem deploy path;
- debugger incompatível;
- projeto clonado de outro computador;
- variáveis de ambiente escondidas;
- SDK industrial em caminho local;
- permissões de serial/USB;
- scripts customizados de build.
```

A Kinein precisa transformar isso em algo visual, auditável e corrigível.

---

## 3. Ideia central: Setup Intelligence Layer

A camada deve ficar entre a UI e os serviços de build/toolchain.

```text
UI / Wizard / Settings
        ↓
Setup Intelligence Layer
        ↓
Environment Detector
Project Detector
Capability Matrix
Configuration Graph
Plan Generator
Risk Engine
Repair Engine
        ↓
Core Actions
CMake / Cargo / Toolchain / Target / LSP / Debug
```

Ela não substitui CMake, Cargo, clangd ou rust-analyzer.  
Ela organiza o caminho até eles.

---

## 4. Princípios da otimização

### 4.1 Menos perguntas, melhores defaults

A Kinein deve evitar perguntar algo que ela consegue detectar com segurança.

Exemplo:

```text
Se existe clang++ funcional, cmake, ninja e clangd:
sugerir Clang + Ninja + Debug/Release presets + compile_commands ON.
```

Mas deve mostrar:

```text
Detectado automaticamente:
- clang++ 18 em /usr/bin/clang++
- ninja em /usr/bin/ninja
- cmake 3.30 em /usr/bin/cmake
- clangd em /usr/bin/clangd
```

---

### 4.2 Todo default precisa de motivo

Não basta selecionar “Ninja” automaticamente.

A UI deve explicar:

```text
Ninja foi selecionado porque está instalado, é compatível com CMake
e costuma ser mais rápido para builds locais incrementais.
```

Isso dá confiança ao usuário.

---

### 4.3 Decisões devem ter confiança

Cada detecção deve ter um nível de confiança:

```text
High
Medium
Low
Unknown
```

Exemplo:

```text
Compiler: clang++ 18
Confidence: High
Reason: executable found, version parsed, test compile passed.
```

Exemplo ruim:

```text
Compiler: /opt/gcc-arm/bin/arm-none-eabi-g++
Confidence: Medium
Reason: executable found, but test compile was not executed yet.
```

---

### 4.4 Projeto existente nunca deve ser alterado automaticamente

Para projetos existentes, a Kinein deve agir como um médico:

```text
examinar
diagnosticar
propor tratamento
mostrar riscos
pedir autorização
aplicar
validar
```

Nunca:

```text
abrir pasta → alterar CMakeLists automaticamente.
```

---

## 5. Environment Fingerprint

Antes de criar ou abrir projeto, a IDE deve montar uma “impressão digital” do ambiente.

### 5.1 Dados coletados

```json
{
  "os": "linux",
  "distro": "cachyos",
  "desktop": "kde",
  "shell": "fish",
  "arch": "x86_64",
  "home": "/home/vitor",
  "path_entries": [],
  "package_managers": ["pacman", "flatpak"],
  "tools": {
    "git": {},
    "cmake": {},
    "ninja": {},
    "make": {},
    "gcc": {},
    "clang": {},
    "clangd": {},
    "gdb": {},
    "lldb": {},
    "rustup": {},
    "cargo": {},
    "rust_analyzer": {},
    "qemu": {},
    "openocd": {},
    "probe_rs": {}
  }
}
```

### 5.2 O que validar

Para cada ferramenta:

```text
path
version
is_executable
is_compatible
test_command
confidence
notes
```

### 5.3 Cache

O scan pode ser cacheado, mas precisa invalidar quando:

```text
PATH mudar;
arquivo executável sumir;
versão mudar;
usuário pedir rescan;
projeto exigir ferramenta não verificada;
erro de build indicar ferramenta quebrada.
```

---

## 6. Project Fingerprint

Ao abrir uma pasta, a IDE deve montar uma impressão digital do projeto.

### 6.1 Detectores

```text
CMakeLists.txt
CMakePresets.json
Cargo.toml
compile_commands.json
Makefile
build/
.git/
DocsPublic/
README.md
.kinein/
```

### 6.2 Resultado

Exemplo:

```json
{
  "kind": "cmake",
  "languages": ["cpp"],
  "has_cmake_presets": false,
  "has_compile_commands": false,
  "has_existing_build_dirs": true,
  "has_git": true,
  "detected_targets": ["motor_control"],
  "detected_docs": ["README.md", "DocsPublic/build.md"],
  "risk_flags": [
    "missing_presets",
    "missing_compile_commands",
    "stale_build_directory"
  ]
}
```

### 6.3 Uso

Esse fingerprint alimenta:

```text
Project Health
Setup Assistant
Assistente
CMake Setup
Run Config Wizard
Toolchain recommendation
LSP initialization
```

---

## 7. Capability Matrix

A IDE deve cruzar ambiente + projeto + intenção do usuário.

### 7.1 Exemplo

Usuário quer criar projeto C++ local.

```text
Necessário:
- CMake
- C++ compiler

Recomendado:
- Ninja
- clangd
- debugger

Opcional:
- Git
- formatter
- static analyzer
```

### 7.2 Matriz

```text
Capability              Status       Source        Confidence
C++ compile             Ready        clang++       High
CMake configure         Ready        cmake+ninja   High
LSP C++                 Ready        clangd        High
Debug native            Partial      lldb          Medium
Rust project            Partial      cargo         Medium
Embedded ARM            Missing      arm gcc       Low
QEMU target             Missing      qemu          Low
```

### 7.3 Benefício

O wizard pode esconder opções inviáveis por padrão, mas com “Show unavailable” para usuários avançados.

---

## 8. Configuration Graph

A maior melhoria conceitual é criar um grafo de configuração.

### 8.1 Entidades

```text
Workspace
Project
Language
Build System
Toolchain
Compiler
Build Profile
Target
Run Configuration
Debug Configuration
LSP Context
Documentation Index
Assistente
```

### 8.2 Relações

```text
Workspace uses Toolchain
Toolchain provides Compiler
Project uses Build System
Build Profile uses Configure Preset
Configure Preset produces compile_commands.json
clangd consumes compile_commands.json
Run Config uses Target
Debug Config uses Debugger
Assistente consumes Diagnostics, Logs and Docs
```

### 8.3 Por que isso importa

Sem grafo, a IDE vira um conjunto de telas soltas.  
Com grafo, a IDE sabe explicar:

```text
clangd está quebrado porque compile_commands.json não existe.
compile_commands.json não existe porque CMake configure ainda não rodou.
CMake configure não rodou porque não há preset selecionado.
Não há preset porque o projeto existente não possui CMakePresets.json.
```

Essa cadeia é exatamente o tipo de clareza que diferencia uma IDE profissional.

---

## 9. Setup Plan

A camada deve gerar um plano antes de aplicar mudanças.

### 9.1 Exemplo: criar C++ project

```json
{
  "plan_id": "setup_plan_001",
  "title": "Create C++ CMake project",
  "steps": [
    {
      "id": "create_files",
      "kind": "write_files",
      "risk": "medium",
      "files": [
        "CMakeLists.txt",
        "CMakePresets.json",
        "src/main.cpp",
        ".kinein/workspace.json"
      ]
    },
    {
      "id": "run_cmake_configure",
      "kind": "command",
      "risk": "low",
      "command": "cmake --preset debug"
    },
    {
      "id": "start_clangd",
      "kind": "service",
      "risk": "low"
    }
  ]
}
```

### 9.2 A UI deve mostrar

```text
Arquivos a criar
Comandos a executar
Configurações escolhidas
Riscos
Como desfazer
```

---

## 10. Setup Plan para projeto existente

### 10.1 Exemplo

Projeto CMake existente sem presets.

Plano sugerido:

```text
Não alterar CMakeLists.txt.
Criar CMakePresets.json mínimo.
Configurar build/debug.
Gerar compile_commands.json.
Criar .kinein/workspace.json.
Rodar cmake --preset debug.
Iniciar clangd.
```

### 10.2 Regra

Alterações em arquivos existentes devem ser separadas de arquivos novos.

```text
Safe:
- criar .kinein/workspace.json
- criar CMakePresets.json se não existir

Higher risk:
- editar CMakeLists.txt
- editar scripts existentes
```

---

## 11. Repair Engine

Quando algo falhar, a Kinein deve gerar um plano de reparo.

### 11.1 CMake configure falhou

Possíveis causas:

```text
compiler not found
generator missing
invalid toolchain file
dependency missing
cache stale
source/build dir mismatch
CMake version too old
```

Ações:

```text
Clear CMake cache
Select different generator
Select different compiler
Create local preset
Open error in Assistente
Show exact command
```

### 11.2 clangd sem contexto

Causa provável:

```text
compile_commands.json missing
```

Plano:

```text
1. Enable CMAKE_EXPORT_COMPILE_COMMANDS.
2. Run CMake configure.
3. Restart clangd.
```

### 11.3 Rust analyzer ausente

Plano:

```text
1. Check rustup.
2. Check rust-analyzer component.
3. Suggest: rustup component add rust-analyzer.
4. Restart language service.
```

### 11.4 Build directory quebrado

Sinais:

```text
CMakeCache.txt points to another source dir
compiler path no longer exists
old generator mismatch
```

Plano:

```text
1. Explain stale cache.
2. Offer safe clean of build/debug only.
3. Reconfigure.
```

---

## 12. Score de recomendação

Toolchains e configurações devem receber score.

### 12.1 Critérios

```text
tool exists
version parsed
test compile passed
debugger available
LSP available
matches project language
matches target architecture
matches user default
works with CMake generator
```

### 12.2 Exemplo

```text
Clang Local
Score: 94/100
Reason:
- clang++ found
- cmake found
- ninja found
- clangd found
- lldb found
- test compile passed
```

```text
ARM GCC
Score: 52/100
Reason:
- compiler found
- debugger missing
- sysroot unknown
- no test compile yet
```

---

## 13. Quick Start Profiles

Para reduzir atrito, a IDE pode oferecer perfis.

### 13.1 Perfis

```text
Quick Start
Professional C++
Rust Standard
Embedded-ready
Custom
```

### 13.2 Quick Start

```text
C++23
CMake + Ninja
Debug/Release
compile_commands ON
clangd
Run config automática
Git opcional
```

### 13.3 Professional C++

```text
C++23
CMake + Ninja
Debug/Release/RelWithDebInfo
warnings profile
tests folder
clangd
formatter placeholder
```

### 13.4 Rust Standard

```text
cargo init
edition 2024 se suportado
rust-analyzer
cargo check
```

### 13.5 Embedded-ready

Não deve ser padrão do MVP simples.

```text
CMake toolchain file
cross compiler
sysroot
target board
flash/debug placeholders
serial config
```

---

## 14. Smart Defaults sem aprisionar o usuário

A IDE deve sugerir, não prender.

```text
Recommended setup
[Use recommended] [Customize]
```

Ao clicar Customize, mostrar todos os detalhes.

---

## 15. Setup Timeline

Adicionar uma timeline visual melhora muito a confiança.

```text
Detect environment
Choose template
Generate files
Configure CMake
Start language services
Create run config
Open editor
```

Cada etapa:

```text
Pending
Running
Done
Warning
Failed
Skipped
```

---

## 16. CMake Cache Hygiene

CMake cache é fonte comum de frustração. A Kinein deve tratar isso muito bem.

### 16.1 Detectar cache inválido

Verificar:

```text
CMAKE_HOME_DIRECTORY
CMAKE_CXX_COMPILER path
CMAKE_GENERATOR
CMAKE_BUILD_TYPE
CMAKE_TOOLCHAIN_FILE
source dir atual
build dir atual
```

### 16.2 Mostrar problema

Exemplo:

```text
Este build directory foi criado para outro caminho:
Old: /home/user/old-project
Now: /home/vitor/motor-control
```

### 16.3 Ação segura

```text
[Create new build directory]
[Clear this cache]
[Open CMakeCache.txt]
[Ignore]
```

Preferir criar novo build directory em vez de apagar.

---

## 17. Local Overrides

Projetos compartilhados não devem carregar caminhos locais do usuário.

### 17.1 Arquivos

Versionável:

```text
CMakePresets.json
.kinein/workspace.json
```

Local:

```text
CMakeUserPresets.json
.kinein/toolchains.local.json
.kinein/session.json
```

### 17.2 Regra

Se a configuração contém `/home/vitor`, `/opt/sdk-local`, paths pessoais ou credenciais, sugerir arquivo local.

---

## 18. Visual Diff para configurações

Não usar diff só para código. Usar para config também.

Exemplo:

```diff
+ {
+   "name": "debug",
+   "generator": "Ninja",
+   "binaryDir": "${sourceDir}/build/debug",
+   "cacheVariables": {
+     "CMAKE_EXPORT_COMPILE_COMMANDS": "ON"
+   }
+ }
```

---

## 19. Setup Assistant como painel persistente

O Setup Assistant não deve ser modal obrigatório.  
Ele deve ser um painel persistente e revisável.

```text
Project Setup
├── Environment
├── Toolchain
├── CMake
├── LSP
├── Run/Debug
└── Targets
```

Cada item mostra:

```text
status
evidência
ação sugerida
risco
```

---

## 20. Melhorias na tela de Settings

### 20.1 “Why is this disabled?”

Toda opção indisponível deve explicar por quê.

Exemplo:

```text
Debug with LLDB está indisponível.
Motivo: lldb não foi encontrado.
Ação: configurar debugger em Settings > Toolchains.
```

### 20.2 “Use this as default”

Em Toolchains:

```text
[Use as default for C++ projects]
[Use as default for this workspace]
[Use only for this target]
```

### 20.3 “Open generated file”

Depois que a IDE gerar config, permitir abrir imediatamente:

```text
Open CMakePresets.json
Open .kinein/workspace.json
Open run-configs.json
```

---

## 21. Telemetria local de setup

Não é telemetria remota. É histórico local para melhorar UX.

Guardar:

```text
última toolchain usada
último template usado
falhas recorrentes
setup plans aplicados
comandos executados
arquivos gerados
```

Uso:

```text
sugerir defaults melhores
mostrar histórico
permitir rollback
```

---

## 22. Rollback

Cada setup plan deve ter estratégia de rollback.

### 22.1 Para arquivos criados

```text
remover arquivos criados
```

### 22.2 Para arquivos editados

```text
restaurar backup temporário
ou aplicar patch reverso
```

### 22.3 Para comandos

Nem todo comando tem rollback. A UI deve dizer.

Exemplo:

```text
cmake configure não altera código-fonte, mas cria arquivos em build/debug.
Rollback: remover build/debug.
```

---

## 23. JSON-RPC complementar

### 23.1 Environment fingerprint

```json
{
  "method": "environment.fingerprint",
  "params": {
    "refresh": true
  }
}
```

### 23.2 Project fingerprint

```json
{
  "method": "project.fingerprint",
  "params": {
    "path": "/home/vitor/Projects/motor-control"
  }
}
```

### 23.3 Generate setup plan

```json
{
  "method": "setup.plan",
  "params": {
    "intent": "new_project",
    "template": "cpp-cmake-executable",
    "profile": "quick_start",
    "path": "/home/vitor/Projects/motor-control"
  }
}
```

### 23.4 Preview setup plan

```json
{
  "method": "setup.preview",
  "params": {
    "plan_id": "setup_plan_001"
  }
}
```

### 23.5 Apply setup plan

```json
{
  "method": "setup.apply",
  "params": {
    "plan_id": "setup_plan_001",
    "confirmation_token": "user_confirmed"
  }
}
```

### 23.6 Repair setup

```json
{
  "method": "setup.repair",
  "params": {
    "source": "cmake_configure_failure",
    "job_id": "job_123"
  }
}
```

### 23.7 Config graph

```json
{
  "method": "configGraph.get",
  "params": {
    "workspace_id": "workspace_001"
  }
}
```

---

## 24. Rust modules sugeridos

```text
crates/
  kinein-setup/
    environment_fingerprint.rs
    project_fingerprint.rs
    capability_matrix.rs
    config_graph.rs
    setup_plan.rs
    setup_preview.rs
    setup_apply.rs
    repair_engine.rs
    scoring.rs
    rollback.rs

  kinein-toolchains/
    detector.rs
    health_check.rs
    compiler_probe.rs
    debugger_probe.rs

  kinein-projects/
    templates.rs
    cmake_template.rs
    rust_template.rs
    existing_project.rs
```

---

## 25. QML components sugeridos

```text
ui/components/setup/
  StartScreen.qml
  EnvironmentCheckPanel.qml
  ProjectWizard.qml
  ProjectTemplateList.qml
  SetupTimeline.qml
  SetupPlanPreview.qml
  GeneratedFilesPreview.qml
  ToolchainRecommendationCard.qml
  ProjectHealthDashboard.qml
  RepairSuggestionCard.qml
  ConfigGraphView.qml
```

---

## 26. Critérios de aceite para esta extensão

```text
[ ] Environment Fingerprint estruturado.
[ ] Project Fingerprint estruturado.
[ ] Capability Matrix com status e confiança.
[ ] Setup Plan antes de aplicar mudanças.
[ ] Preview de arquivos/comandos.
[ ] Score de toolchain.
[ ] Project Health explica causa de problemas.
[ ] Setup Assistant sugere reparos.
[ ] CMake cache stale detectado.
[ ] compile_commands ausente gera ação clara.
[ ] clangd/rust-analyzer status conectado ao setup.
[ ] Projetos existentes não são alterados sem confirmação.
[ ] Rollback para arquivos criados/editados.
[ ] JSON-RPC separado para plan/preview/apply.
```

---

## 27. MVP realista da Parte 8.1

Para não exagerar no início:

```text
1. Environment Fingerprint básico.
2. Project Fingerprint para CMake/Cargo.
3. Toolchain score simples.
4. Setup Plan para novo C++/Rust.
5. Preview antes de criar arquivos.
6. CMake configure com timeline.
7. Project Health básico.
8. Reparos para:
   - missing compiler;
   - missing cmake;
   - missing ninja;
   - missing compile_commands;
   - missing rust-analyzer;
   - stale CMake cache.
```

---

## 28. O que deixar para depois

```text
- ConfigGraph visual completo;
- rollback avançado de múltiplos arquivos;
- suporte a múltiplos build systems;
- download/instalação automática de SDK;
- embedded wizard completo;
- remote SSH setup completo;
- QEMU profiles;
- heurísticas muito complexas;
- IA autônoma de setup.
```

---

## 29. Como isso melhora a Parte 8

A Parte 8 cria o fluxo.  
A Parte 8.1 cria a inteligência por trás do fluxo.

Sem a Parte 8.1:

```text
A Kinein tem um bom wizard.
```

Com a Parte 8.1:

```text
A Kinein entende o ambiente, recomenda o caminho, mostra o plano,
aplica com segurança, valida o resultado e ajuda a reparar falhas.
```

Essa é a diferença entre uma UI bonita e uma IDE realmente confiável.

---

## 30. Frase-guia

```text
A Kinein deve fazer setup como uma IDE profissional:
detectar, explicar, planejar, aplicar com consentimento, validar e reparar.
```

O usuário não deve sentir que está preenchendo formulário.  
Ele deve sentir que a IDE está montando um ambiente técnico sólido com ele.
