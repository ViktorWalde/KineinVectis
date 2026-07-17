# Kinein Vectis — Parte 8: Settings, Onboarding, Project Wizard e Setup Visual de CMake/Toolchain

> **Status:** especificação de produto, UX e arquitetura.  
> **Escopo:** primeira experiência do usuário, abertura/criação de projetos, configuração visual de CMake, compiladores, toolchains, targets, settings, presets e assistente de reparo.  
> **Prioridade:** altíssima. Esta parte define uma das promessas centrais da Kinein: **fazer o programador focar no código, não na luta contra ambiente, CMake, compilador e configuração.**

---

## 1. Objetivo da Parte 8

A Kinein Vectis precisa ser uma IDE para C, C++ e Rust que pareça poderosa, mas não intimidadora. O usuário deve abrir a IDE e sentir que consegue começar um projeto sem entrar em um labirinto de flags, caminhos, toolchains, generators, cache de CMake, `compile_commands.json`, presets e variáveis de ambiente.

A promessa desta parte é:

```text
A Kinein não esconde CMake, compiladores e toolchains.
Ela transforma esses sistemas em fluxos visuais, auditáveis e corrigíveis.
```

O objetivo não é “simplificar” de forma infantil. É **organizar a complexidade**.

---

## 2. Filosofia de UX

### 2.1 JetBrains-like na sensação

A experiência deve passar a sensação psicológica de IDE profissional:

```text
- fluxo guiado;
- escolhas claras;
- padrões seguros;
- estado visível;
- configurações editáveis;
- problemas explicados;
- automação controlada;
- aparência calma e polida;
- nada de wizard amador com dezenas de telas feias.
```

O usuário deve sentir:

```text
“Isso é profissional, mas eu não estou perdido.”
```

---

### 2.2 Kinein-like no foco

A Kinein tem identidade própria:

```text
- C, C++ e Rust como linguagens principais;
- CMake como sistema central para C/C++;
- Cargo como sistema central para Rust;
- toolchain como entidade de primeira classe;
- target como entidade de primeira classe;
- Linux-first;
- embarcados e Linux embarcado no horizonte desde o início;
- simulação/OpenGL no futuro sem poluir o MVP.
```

---

### 2.3 Configuração visível, não mágica

A IDE pode automatizar, mas não deve esconder.

Sempre que a Kinein gerar ou alterar algo, ela deve mostrar:

```text
- o que foi detectado;
- o que será criado;
- quais arquivos serão afetados;
- qual comando será executado;
- como desfazer;
- onde editar manualmente.
```

---

## 3. Regra de ouro da Parte 8

```text
Todo wizard deve terminar com arquivos reais, comandos reais e estado rastreável.
```

Exemplo:

```text
CMake project wizard
  ↓
gera CMakeLists.txt
gera CMakePresets.json
gera .kinein/workspace.json
detecta compilador
roda cmake configure
gera compile_commands.json
inicia clangd
mostra status
```

Nada deve ficar apenas “na memória da UI”.

---

## 4. Primeira abertura da IDE

### 4.1 Tela inicial

A tela inicial deve ser limpa, escura, confortável e direta.

Layout recomendado:

```text
┌──────────────────────────────────────────────────────────────┐
│ KV  Kinein                                                   │
│                                                              │
│ Start                                                       │
│                                                              │
│ [New C/C++ Project]  [New Rust Project]  [Open Workspace]    │
│ [Clone Repository]   [Open Folder]       [Settings]          │
│                                                              │
│ Recent Projects                                             │
│  • motor-firmware                                           │
│  • vision-sim                                               │
│  • rust-control                                             │
│                                                              │
│ Environment Status                                          │
│  clang++ 18 detected | cmake 3.30 | ninja | rustup | git     │
└──────────────────────────────────────────────────────────────┘
```

### 4.2 O que não colocar na tela inicial

Evitar:

```text
- banners enormes;
- marketing;
- cards demais;
- IA chamativa;
- simulação/OpenGL na primeira tela;
- botões genéricos sem contexto;
- excesso de texto.
```

A tela inicial deve convidar o usuário a criar ou abrir projeto.

---

## 5. First Run Setup

Na primeira execução, a Kinein deve rodar um setup rápido.

### 5.1 Etapas

```text
1. Detectar sistema operacional.
2. Detectar shell.
3. Detectar Git.
4. Detectar CMake.
5. Detectar Ninja/Make.
6. Detectar GCC/Clang.
7. Detectar GDB/LLDB.
8. Detectar Rust/rustup/cargo/rust-analyzer.
9. Detectar clangd.
10. Detectar QEMU/OpenOCD/probe-rs opcionalmente.
11. Criar diretório de configurações da IDE.
12. Criar perfil local padrão.
```

### 5.2 Resultado visual

Mostrar um painel:

```text
Environment Check
├── C/C++ Toolchain
│   ├── clang++ 18.1.8   OK
│   ├── gcc 14.2         OK
│   ├── clangd           OK
│   └── gdb              OK
├── Build Tools
│   ├── cmake            OK
│   ├── ninja            OK
│   └── make             OK
├── Rust
│   ├── rustup           OK
│   ├── cargo            OK
│   └── rust-analyzer    Missing
└── Embedded Optional
    ├── qemu-system-arm  Missing
    ├── openocd          Missing
    └── probe-rs         Missing
```

### 5.3 Níveis de severidade

```text
Required:
- git
- cmake
- compiler for selected workflow

Recommended:
- ninja
- clangd
- rust-analyzer
- gdb/lldb

Optional:
- qemu
- openocd
- probe-rs
- serial tools
```

### 5.4 Reparos sugeridos

A IDE pode sugerir comandos, mas não executar automaticamente sem confirmação.

Exemplo:

```text
rust-analyzer is missing.

Suggested:
rustup component add rust-analyzer

[Copy command] [Run with confirmation] [Ignore]
```

---

## 6. Tipos de projeto no Project Wizard

O wizard deve começar por intenção, não por tecnologia crua.

### 6.1 Categorias principais

```text
C Project
C++ Project
Rust Project
Mixed C++/Rust Project
Existing CMake Project
Existing Cargo Project
Embedded Firmware Project
Linux Embedded / Remote Target Project
Library Project
Tool / CLI Project
```

### 6.2 MVP recomendado

Para o início, priorizar:

```text
1. C++ executable with CMake
2. C executable with CMake
3. Rust binary with Cargo
4. Existing CMake project
5. Existing Cargo project
```

Depois expandir:

```text
6. C++ library with CMake
7. Rust library
8. Mixed C++/Rust
9. Embedded firmware
10. Remote Linux target
```

---

## 7. Wizard: New C++ Project with CMake

### 7.1 Fluxo

```text
Start
  ↓
Project name/location
  ↓
Language standard
  ↓
Project type
  ↓
Toolchain selection
  ↓
Build profile
  ↓
Generated files preview
  ↓
Create project
  ↓
CMake configure
  ↓
Open editor
```

### 7.2 Tela 1 — Nome e localização

Campos:

```text
Project name: motor-control
Location: ~/Projects
Final path: ~/Projects/motor-control
```

Validações:

```text
- nome não vazio;
- caminho gravável;
- pasta vazia ou confirmação para usar existente;
- sem caracteres problemáticos;
- aviso se estiver dentro de diretório de sistema.
```

### 7.3 Tela 2 — Tipo de projeto

Opções:

```text
Executable
Static Library
Shared Library
Header-only Library
CLI Tool
```

MVP:

```text
Executable
Static Library
```

### 7.4 Tela 3 — Padrão da linguagem

C++:

```text
C++17
C++20
C++23
C++26 experimental
```

Padrão recomendado:

```text
C++23
```

C:

```text
C11
C17
C23
```

Padrão recomendado:

```text
C23 quando disponível, C17 como fallback.
```

### 7.5 Tela 4 — Toolchain

Mostrar toolchains detectadas:

```text
Clang 18 — /usr/bin/clang++
GCC 14 — /usr/bin/g++
Custom toolchain...
```

Cada toolchain deve mostrar:

```text
- compiler path;
- version;
- target triple;
- debugger compatível;
- CMake compatibility;
- status do clangd;
- se gera compile_commands.json.
```

### 7.6 Tela 5 — Build system

Para C/C++:

```text
CMake + Ninja
CMake + Make
```

Padrão recomendado:

```text
CMake + Ninja
```

A IDE deve explicar:

```text
Ninja é rápido e simples para builds locais.
Você ainda pode mudar depois em Settings > Build Tools.
```

### 7.7 Tela 6 — Preview de arquivos gerados

Antes de criar:

```text
motor-control/
├── CMakeLists.txt
├── CMakePresets.json
├── src/
│   └── main.cpp
├── include/
├── tests/
├── .gitignore
└── .kinein/
    └── workspace.json
```

Mostrar também preview do `CMakeLists.txt`.

---

## 8. CMakeLists.txt gerado

### 8.1 C++ executable mínimo

```cmake
cmake_minimum_required(VERSION 3.24)

project(motor_control
    VERSION 0.1.0
    LANGUAGES CXX
)

add_executable(motor_control
    src/main.cpp
)

target_compile_features(motor_control
    PRIVATE
        cxx_std_23
)

target_include_directories(motor_control
    PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/include
)
```

### 8.2 Regras

O CMake gerado deve ser:

```text
- simples;
- moderno;
- target-based;
- sem variáveis globais desnecessárias;
- sem macros mágicas;
- compatível com clangd;
- pronto para expansão.
```

Evitar:

```cmake
include_directories(...)
add_definitions(...)
set(CMAKE_CXX_FLAGS ...)
```

---

## 9. CMakePresets.json gerado

### 9.1 Presets recomendados

```json
{
  "version": 6,
  "configurePresets": [
    {
      "name": "debug",
      "displayName": "Debug",
      "generator": "Ninja",
      "binaryDir": "${sourceDir}/build/debug",
      "cacheVariables": {
        "CMAKE_BUILD_TYPE": "Debug",
        "CMAKE_EXPORT_COMPILE_COMMANDS": "ON"
      }
    },
    {
      "name": "release",
      "displayName": "Release",
      "generator": "Ninja",
      "binaryDir": "${sourceDir}/build/release",
      "cacheVariables": {
        "CMAKE_BUILD_TYPE": "Release",
        "CMAKE_EXPORT_COMPILE_COMMANDS": "ON"
      }
    }
  ],
  "buildPresets": [
    {
      "name": "debug",
      "configurePreset": "debug"
    },
    {
      "name": "release",
      "configurePreset": "release"
    }
  ]
}
```

### 9.2 Regra da Kinein

A IDE deve preferir `CMakePresets.json` quando existir.  
Se não existir, deve oferecer criar um preset mínimo.

---

## 10. Wizard: New Rust Project

### 10.1 Fluxo

```text
Project name/location
  ↓
Binary or library
  ↓
Edition
  ↓
Toolchain
  ↓
Generated files preview
  ↓
cargo init
  ↓
rust-analyzer start
```

### 10.2 Opções

```text
Binary application
Library
Workspace
CLI tool
```

### 10.3 Edition

```text
2021
2024
```

Padrão recomendado:

```text
2024 quando toolchain suportar; 2021 como fallback.
```

### 10.4 Arquivos

```text
Cargo.toml
src/main.rs
.gitignore
.kinein/workspace.json
```

### 10.5 Regras

A IDE não deve reinventar Cargo.  
Ela deve chamar `cargo init` ou gerar estrutura compatível.

---

## 11. Wizard: Existing Project

Esse fluxo é crucial para a usabilidade real.

### 11.1 Abrir pasta existente

Ao abrir uma pasta, detectar:

```text
CMakeLists.txt
CMakePresets.json
Cargo.toml
compile_commands.json
Makefile
meson.build futuramente
.bazel futuramente
```

### 11.2 Resultado

Mostrar:

```text
Project detected:
- CMake project
- C++ language
- No CMakePresets.json found
- clangd available
- compile_commands.json missing
```

Ações:

```text
[Use as-is]
[Create CMakePresets.json]
[Configure now]
[Open settings]
```

### 11.3 Não forçar migração

A IDE não deve alterar projeto existente automaticamente.  
Ela deve sugerir melhorias e mostrar diffs.

---

## 12. Toolchain Manager

O Toolchain Manager é uma tela essencial.

### 12.1 Localização

```text
Settings > Toolchains
```

### 12.2 Entidade Toolchain

Uma toolchain deve conter:

```json
{
  "id": "clang-local",
  "name": "Clang Local",
  "kind": "native",
  "c_compiler": "/usr/bin/clang",
  "cpp_compiler": "/usr/bin/clang++",
  "debugger": "/usr/bin/lldb",
  "cmake": "/usr/bin/cmake",
  "ninja": "/usr/bin/ninja",
  "clangd": "/usr/bin/clangd",
  "target_triple": "x86_64-unknown-linux-gnu",
  "environment": {}
}
```

### 12.3 Tipos de toolchain

```text
Native GCC
Native Clang
Rust stable
Rust nightly
Cross GCC
Clang cross
Custom CMake toolchain file
Remote SSH toolchain
SDK/sysroot toolchain
```

### 12.4 Health Check

Cada toolchain deve ter botão:

```text
[Run Health Check]
```

Checagens:

```text
compiler --version
cmake --version
ninja --version
debugger --version
test compile hello world
test CMake configure
test compile_commands generation
```

### 12.5 Estado visual

```text
Ready
Needs attention
Broken
Missing debugger
Missing clangd
CMake incompatible
```

---

## 13. CMake Setup Visual

A Kinein deve ter uma tela visual para CMake.

### 13.1 Localização

```text
Settings > Build > CMake
```

Ou em Project Settings:

```text
Project > CMake
```

### 13.2 Elementos

```text
Configure Preset
Build Preset
Generator
Build directory
Install directory
Toolchain file
Cache variables
Environment variables
Export compile_commands.json
```

### 13.3 Visual recomendado

```text
CMake
├── Active preset: debug
├── Generator: Ninja
├── Build dir: build/debug
├── Compiler: clang++ 18
├── Export compile commands: ON
├── Configure status: OK
└── Last configure command:
    cmake --preset debug
```

### 13.4 Cache Variables Editor

Tabela:

```text
Name                     Value                 Type
CMAKE_BUILD_TYPE          Debug                 STRING
CMAKE_EXPORT_COMPILE...   ON                    BOOL
CMAKE_TOOLCHAIN_FILE      /path/to/file.cmake   FILEPATH
```

Regras:

```text
- editar com validação;
- marcar variáveis modificadas;
- permitir reset;
- mostrar origem: preset, cache, usuário ou IDE;
- não editar cache sem aviso.
```

---

## 14. Run Configuration Wizard

### 14.1 Campos

```text
Name
Executable
Working directory
Arguments
Environment
Target
Before launch
Debugger
```

### 14.2 Geração automática

Após CMake configure, a IDE deve detectar executáveis:

```text
build/debug/motor_control
```

Criar sugestão:

```text
Run motor_control
Debug motor_control
```

### 14.3 Regras

```text
- nunca assumir binário inexistente;
- se houver múltiplos executáveis, pedir escolha;
- se binário não foi buildado, oferecer Build before Run;
- se Debug, verificar símbolos.
```

---

## 15. Debug Configuration Wizard

### 15.1 Native Debug

Campos:

```text
Executable
Debugger: gdb/lldb
Arguments
Working directory
Environment
Stop at main
Prelaunch build
```

### 15.2 Embedded Debug futuro

Campos:

```text
Probe
OpenOCD config
GDB server
ELF file
Reset strategy
Flash before debug
```

### 15.3 Remote Debug futuro

Campos:

```text
SSH target
Remote executable path
Local source mapping
Remote working directory
GDB server port
```

---

## 16. Target Manager

Targets não são apenas “run configs”. Eles representam onde o código roda.

### 16.1 Tipos

```text
Local Machine
Remote SSH Linux
QEMU Virtual Target
Bare-metal Board
Custom Target
```

### 16.2 Target local

```json
{
  "id": "local-linux",
  "name": "Local Linux",
  "kind": "local",
  "os": "linux",
  "arch": "x86_64"
}
```

### 16.3 Target remoto

```json
{
  "id": "rpi-dev",
  "name": "Raspberry Pi Dev",
  "kind": "remote-ssh",
  "host": "192.168.1.20",
  "user": "vitor",
  "deploy_path": "/home/vitor/app"
}
```

### 16.4 Target embarcado

```json
{
  "id": "stm32-dev",
  "name": "STM32 Dev Board",
  "kind": "bare-metal",
  "probe": "stlink",
  "gdb_server": "openocd",
  "flash": true
}
```

---

## 17. Settings Architecture

### 17.1 Níveis de configuração

A Kinein precisa separar:

```text
Global Settings
User Settings
Workspace Settings
Project Settings
Run/Debug Configurations
Target Settings
Toolchain Settings
```

### 17.2 Prioridade

```text
Project overrides Workspace
Workspace overrides User
User overrides Defaults
```

### 17.3 Onde armazenar

Linux-first:

```text
Global/user:
~/.config/kinein/settings.json
~/.config/kinein/toolchains.json
~/.config/kinein/providers.json

Workspace/project:
.kinein/workspace.json
.kinein/toolchains.local.json
.kinein/targets.json
.kinein/run-configs.json
.kinein/debug-configs.json
```

### 17.4 O que versionar

Pode versionar:

```text
.kinein/workspace.json
.kinein/targets.example.json
CMakePresets.json
```

Não versionar:

```text
.kinein/toolchains.local.json
.kinein/secrets.json
.kinein/session.json
```

---

## 18. Settings UI

### 18.1 Estrutura

```text
Settings
├── General
│   ├── Appearance
│   ├── Editor
│   ├── Keymap
│   └── Updates
├── Build
│   ├── CMake
│   ├── Generators
│   ├── Build Profiles
│   └── Jobs
├── Toolchains
│   ├── C/C++
│   ├── Rust
│   ├── Debuggers
│   └── Cross Compilation
├── Targets
│   ├── Local
│   ├── Remote SSH
│   ├── QEMU
│   └── Boards
├── Languages
│   ├── C/C++
│   ├── Rust
│   ├── Tree-sitter
│   └── LSP
├── Assistente
│   ├── Mode
│   ├── Local Provider
│   ├── External Provider
│   ├── Privacy
│   └── Prompt Transparency
└── Advanced
    ├── Logs
    ├── JSON-RPC
    ├── Diagnostics
    └── Experimental
```

### 18.2 Design da tela

A tela de Settings deve ser dividida:

```text
Sidebar de categorias à esquerda.
Conteúdo principal à direita.
Busca no topo.
Status no rodapé.
```

---

## 19. Busca em Settings

A busca deve ser excelente.

Exemplos:

```text
"compiler"
"clangd"
"cmake"
"ninja"
"rust analyzer"
"serial"
"qemu"
"theme"
"font"
"keymap"
```

Resultado deve mostrar caminho:

```text
Toolchains > C/C++ > Compiler path
Build > CMake > Generator
Languages > C/C++ > clangd
```

---

## 20. Project Health Dashboard

Ao abrir projeto, a Kinein deve ter um painel de saúde.

### 20.1 Campos

```text
Project type
Languages
Build system
Toolchain
LSP status
Tree-sitter status
CMake configure status
compile_commands.json status
Run configurations
Debug configurations
Targets
Docs index
```

### 20.2 Exemplo

```text
Project Health
├── CMake project             OK
├── C++23                     OK
├── clang++                   OK
├── clangd                    OK
├── compile_commands.json     OK
├── Run config                Missing
├── Debug config              Missing
└── Docs index                OK
```

Ações:

```text
[Create Run Config]
[Create Debug Config]
[Open CMake Settings]
[Explain]
```

---

## 21. Setup Assistant

O Setup Assistant é um fluxo guiado para corrigir problemas.

### 21.1 Quando aparece

```text
- primeira abertura;
- projeto sem toolchain;
- CMake configure falhou;
- clangd sem compile_commands;
- rust-analyzer ausente;
- debugger ausente;
- target inválido;
```

### 21.2 Formato

Não deve ser modal agressivo. Melhor:

```text
card lateral
banner discreto
tool window
```

### 21.3 Exemplo

```text
CMake is not configured yet.

Recommended setup:
1. Select compiler
2. Select generator
3. Create debug preset
4. Run configure

[Start Setup] [Use terminal manually] [Ignore]
```

---

## 22. Fluxo “abrir projeto quebrado”

Esse fluxo é extremamente importante.

### 22.1 Situação

Usuário clona um projeto C++ real e abre na IDE.

Problemas:

```text
- sem presets;
- CMake antigo;
- dependências ausentes;
- compiler path quebrado;
- cache antigo;
- build directory de outro PC;
- clangd sem compile_commands.
```

### 22.2 Comportamento ideal

```text
Kinein detecta projeto
  ↓
não altera nada
  ↓
mostra Project Health
  ↓
sugere configuração local
  ↓
cria preset local se usuário aceitar
  ↓
roda configure
  ↓
se falhar, Assistente explica
```

### 22.3 Regra

Nunca “consertar” projeto de outra pessoa sem consentimento.

---

## 23. Fluxo “quero só programar”

A Kinein deve ter um caminho rápido.

```text
New C++ Project
  ↓
Name: app
  ↓
Use recommended setup
  ↓
Create
```

A IDE escolhe:

```text
C++23
CMake + Ninja
Debug/Release presets
clang++ ou g++ disponível
compile_commands ON
Run config automática
Git init opcional
```

Tempo ideal:

```text
menos de 30 segundos até abrir main.cpp com projeto configurado.
```

---

## 24. Fluxo “sou avançado”

Usuário avançado deve poder controlar tudo.

Opções avançadas:

```text
custom compiler path
custom CMake generator
custom toolchain file
custom sysroot
custom cache variables
custom environment
custom build directory
custom debug adapter
custom target
custom launch command
```

A UI deve usar progressive disclosure:

```text
básico primeiro
advanced collapsible depois
```

---

## 25. Progressive Disclosure

Não mostrar tudo de uma vez.

### 25.1 Exemplo

Tela simples:

```text
Compiler: Clang 18
Generator: Ninja
Build profile: Debug
```

Botão:

```text
[Advanced]
```

Mostra:

```text
C compiler path
C++ compiler path
CMAKE_C_COMPILER
CMAKE_CXX_COMPILER
CMAKE_TOOLCHAIN_FILE
CMAKE_SYSROOT
environment variables
cache variables
```

---

## 26. Templates de projeto

### 26.1 C++ executable

```text
CMakeLists.txt
CMakePresets.json
src/main.cpp
include/
tests/
.gitignore
.kinein/workspace.json
```

### 26.2 C executable

```text
CMakeLists.txt
CMakePresets.json
src/main.c
include/
.gitignore
.kinein/workspace.json
```

### 26.3 Rust binary

```text
Cargo.toml
src/main.rs
.gitignore
.kinein/workspace.json
```

### 26.4 Mixed C++/Rust futuro

```text
CMakeLists.txt
Cargo.toml
crates/
cpp/
include/
bindings/
.kinein/workspace.json
```

Essa opção deve ser avançada, não MVP inicial.

---

## 27. Git no Wizard

Perguntar:

```text
Initialize Git repository?
```

Opções:

```text
Yes
No
Use existing
```

Se sim:

```text
git init
gerar .gitignore
opcionalmente primeiro commit
```

Não criar remoto automaticamente.

---

## 28. Keymap e atalhos no onboarding

A Kinein pode oferecer:

```text
Kinein Default
JetBrains-like
VS Code-like
Vim mode futuramente
```

Padrão recomendado:

```text
Kinein Default com familiaridade JetBrains.
```

Mas não deve travar usuário.

---

## 29. Aparência no onboarding

Pedir pouco:

```text
Theme: Kinein Dark
Accent: Amber
Font: JetBrains Mono or system monospace
```

Não gastar muitas telas com aparência.

---

## 30. Assistente no onboarding

O Assistente deve aparecer como recurso, mas não como centro da experiência.

Mensagem discreta:

```text
Assistente can explain build errors, CMake issues and toolchain problems.
It works locally when configured and can be disabled.
```

Não forçar login ou API key.

---

## 31. Estados visuais do Wizard

### 31.1 Ready

Tudo OK.

```text
verde discreto
```

### 31.2 Warning

Algo recomendado ausente.

```text
âmbar
```

### 31.3 Error

Algo obrigatório ausente.

```text
vermelho discreto
```

### 31.4 Detecting

Scan em andamento.

```text
spinner minimalista
```

### 31.5 Manual Required

Usuário precisa escolher caminho.

```text
azul técnico
```

---

## 32. Design do Project Wizard

### 32.1 Estrutura visual

```text
┌──────────────────────────────────────────────────────────────┐
│ New Project                                                  │
├─────────────────┬────────────────────────────────────────────┤
│ Templates       │ Main form                                  │
│                 │                                            │
│ C++             │ Project name                               │
│ C               │ Location                                   │
│ Rust            │ Type                                       │
│ Existing        │ Toolchain                                  │
│ Embedded        │ Build system                               │
│                 │                                            │
│                 │ [Advanced]                                 │
├─────────────────┴────────────────────────────────────────────┤
│ Files preview                                      [Create]   │
└──────────────────────────────────────────────────────────────┘
```

### 32.2 Painel de preview

Sempre mostrar:

```text
arquivos gerados
comandos que serão executados
configurações escolhidas
avisos
```

---

## 33. Design do Toolchain Manager

```text
┌──────────────────────────────────────────────────────────────┐
│ Toolchains                                      [Add] [Scan]  │
├──────────────────────┬───────────────────────────────────────┤
│ Clang Local       OK │ Name: Clang Local                      │
│ GCC Local         OK │ C compiler: /usr/bin/clang             │
│ Rust Stable       OK │ C++ compiler: /usr/bin/clang++         │
│ ARM GCC     Missing │ Debugger: /usr/bin/lldb                │
│                      │ CMake: /usr/bin/cmake                  │
│                      │ Ninja: /usr/bin/ninja                  │
│                      │                                       │
│                      │ [Run Health Check] [Use as Default]    │
└──────────────────────┴───────────────────────────────────────┘
```

---

## 34. Design do CMake Setup

```text
┌──────────────────────────────────────────────────────────────┐
│ CMake                                                        │
├──────────────────────────────────────────────────────────────┤
│ Active preset: Debug                                         │
│ Generator: Ninja                                             │
│ Build dir: build/debug                                       │
│ Toolchain: Clang Local                                       │
│ Export compile_commands.json: ON                             │
│                                                              │
│ Cache Variables                                              │
│ CMAKE_BUILD_TYPE          Debug                              │
│ CMAKE_EXPORT_COMPILE...   ON                                 │
│                                                              │
│ Last command: cmake --preset debug                           │
│                                                              │
│ [Configure] [Clear Cache] [Open CMakePresets.json]           │
└──────────────────────────────────────────────────────────────┘
```

---

## 35. JSON-RPC sugerido

### 35.1 Environment scan

```json
{
  "method": "environment.scan",
  "params": {
    "include_optional": true
  }
}
```

### 35.2 Create project

```json
{
  "method": "project.create",
  "params": {
    "template": "cpp-cmake-executable",
    "name": "motor-control",
    "location": "/home/vitor/Projects",
    "language_standard": "cxx23",
    "toolchain_id": "clang-local",
    "generator": "Ninja",
    "create_git": true
  }
}
```

### 35.3 Detect project

```json
{
  "method": "project.detect",
  "params": {
    "path": "/home/vitor/Projects/motor-control"
  }
}
```

### 35.4 Toolchain health

```json
{
  "method": "toolchain.healthCheck",
  "params": {
    "toolchain_id": "clang-local"
  }
}
```

### 35.5 CMake configure

```json
{
  "method": "cmake.configure",
  "params": {
    "workspace_id": "workspace_001",
    "preset": "debug"
  }
}
```

### 35.6 Create run config

```json
{
  "method": "runConfig.create",
  "params": {
    "name": "Run motor_control",
    "executable": "${workspaceRoot}/build/debug/motor_control",
    "working_directory": "${workspaceRoot}",
    "target_id": "local-linux",
    "before_launch": ["build:debug"]
  }
}
```

---

## 36. Schemas internos

### 36.1 `.kinein/workspace.json`

```json
{
  "version": 1,
  "name": "motor-control",
  "kind": "cmake",
  "languages": ["cpp"],
  "default_toolchain": "clang-local",
  "default_target": "local-linux",
  "default_build_profile": "debug",
  "created_by": "kinein"
}
```

### 36.2 `.kinein/run-configs.json`

```json
{
  "version": 1,
  "configs": [
    {
      "id": "run_motor_control",
      "name": "Run motor_control",
      "kind": "native",
      "executable": "${workspaceRoot}/build/debug/motor_control",
      "working_directory": "${workspaceRoot}",
      "args": [],
      "env": {},
      "target": "local-linux",
      "before_launch": ["build:debug"]
    }
  ]
}
```

### 36.3 `.kinein/targets.json`

```json
{
  "version": 1,
  "targets": [
    {
      "id": "local-linux",
      "name": "Local Linux",
      "kind": "local",
      "arch": "x86_64",
      "os": "linux"
    }
  ]
}
```

---

## 37. Integração com Assistente

O Assistente deve atuar como explicador e reparador do onboarding/setup.

Exemplos:

```text
Por que meu CMake configure falhou?
Qual compilador devo escolher?
O que é compile_commands.json?
Por que clangd precisa disso?
Qual a diferença entre preset Debug e Release?
Por que Ninja é recomendado?
O que significa target local?
```

Em caso de falha, o Setup Assistant chama o Assistente com evidências.

---

## 38. Integração com Tree-sitter e LSP

Após criação/abertura:

```text
Tree-sitter deve funcionar imediatamente para estrutura local.
LSP deve iniciar depois que o projeto tiver contexto suficiente.
```

Para C/C++:

```text
clangd deve receber compile_commands.json.
```

Para Rust:

```text
rust-analyzer deve receber Cargo.toml/cargo metadata.
```

Se faltar:

```text
Project Health deve avisar.
```

---

## 39. Integração com sistemas embarcados

Mesmo que não seja MVP inicial completo, o design deve prever:

```text
Cross compiler selection
CMake toolchain file
Sysroot
SDK path
OpenOCD config
QEMU target
Serial port
Remote SSH target
```

Essas opções devem ficar em:

```text
Advanced
Targets
Toolchains > Cross Compilation
```

Não devem poluir o fluxo simples de C++ local.

---

## 40. Segurança operacional

### 40.1 Comandos que exigem confirmação

```text
instalar pacotes
executar shell arbitrário
apagar build directory
alterar PATH global
alterar arquivo fora do workspace
alterar permissões de porta serial
alterar grupos do usuário
enviar contexto para IA externa
```

### 40.2 Comandos seguros

```text
cmake --preset debug
cmake --build --preset debug
cargo metadata
cargo check
compiler --version
cmake --version
```

Mesmo assim, mostrar o comando.

---

## 41. MVP da Parte 8

Prioridade inicial:

```text
[ ] Tela inicial com Open Folder, New Project e Recent Projects.
[ ] Environment scan básico.
[ ] New C++ CMake executable wizard.
[ ] New C CMake executable wizard.
[ ] New Rust binary wizard.
[ ] Existing CMake project detection.
[ ] Existing Cargo project detection.
[ ] Toolchain Manager básico.
[ ] CMakePresets generation.
[ ] compile_commands.json ON por padrão.
[ ] Run config automática para executável único.
[ ] Project Health Dashboard básico.
[ ] Settings > Toolchains.
[ ] Settings > Build > CMake.
[ ] Integração com Assistente para falhas.
```

---

## 42. Pós-MVP

```text
[ ] Templates de biblioteca.
[ ] Mixed C++/Rust.
[ ] Remote SSH target.
[ ] QEMU target.
[ ] Embedded firmware wizard.
[ ] OpenOCD/probe-rs setup.
[ ] Setup Assistant avançado.
[ ] Busca em Settings.
[ ] Importar presets existentes.
[ ] Comparar toolchains.
[ ] Diagnóstico visual de CMake cache.
[ ] Profiles compartilháveis.
```

---

## 43. O que não implementar agora

Para preservar qualidade:

```text
- wizard de embarcados complexo no MVP;
- marketplace de templates;
- download automático de toolchains;
- instalação automática de dependências;
- suporte completo a todos os build systems;
- abstração que esconda CMake completamente;
- geração de CMake excessivamente opinativa;
- UI lotada de opções avançadas;
- auto-fix sem diff;
- provider de IA obrigatório no onboarding.
```

---

## 44. Critérios de aceite de UX

O fluxo será bom quando:

```text
- usuário iniciante consegue criar C++/Rust sem entender tudo de CMake;
- usuário avançado consegue ver e editar tudo;
- o projeto gerado é limpo e profissional;
- CMakePresets é claro;
- compile_commands.json funciona;
- clangd/rust-analyzer inicializam;
- erro de ambiente vira ação clara;
- nenhum arquivo é alterado sem preview;
- nenhuma dependência externa é obrigatória sem explicação;
- a IDE parece polida, calma e confiável.
```

---

## 45. Resumo executivo

A Parte 8 é uma das bases do produto.

Ela define como a Kinein Vectis transforma:

```text
CMake
compilador
toolchain
target
debugger
presets
run configs
project setup
```

em uma experiência visual, profissional e segura.

A promessa final:

```text
Kinein Vectis deixa o programador começar certo,
entender o ambiente,
corrigir problemas com segurança
e voltar rapidamente para o código.
```

A IDE não deve esconder a engenharia.  
Ela deve **organizar a engenharia para que o usuário não se perca nela**.
