# Kinein Vectis — Parte 4: Fluxos de Produto para Toolchain, CMake, Build, Run e Debug

> **Nome oficial:** Kinein Vectis  
> **Nome de uso diário:** Kinein  
> **Sigla visual:** KV  
> **Foco técnico:** C, C++ e Rust  
> **Foco inicial do produto:** sistemas embarcados, Linux embarcado, software embarcado e aplicações de alto nível em C++/Rust  
> **Direção de longo prazo:** toolchains, CMake, Cargo, Python, debug, deploy, targets, Linux embarcado e engenharia de sistemas

Este documento define a **Parte 4** do sistema de produto da Kinein Vectis: os fluxos centrais que fazem a IDE ser útil na prática.

A filosofia desta etapa é simples:

```text
O usuário deve focar no código.
A IDE deve tornar toolchain, CMake, compilador, build, run e debug previsíveis, visuais e recuperáveis.
```

A Kinein não deve tentar esconder completamente a complexidade de C/C++/Rust. Isso seria artificial. Ela deve **organizar a complexidade**, mostrar o que está acontecendo e oferecer caminhos claros quando algo falha.

---

## 1. Objetivo da Parte 4

A Parte 4 descreve como a IDE deve se comportar quando o usuário:

1. Abre um projeto.
2. Detecta linguagens e ferramentas.
3. Configura toolchain.
4. Configura CMake ou Cargo.
5. Executa configure/generate.
6. Compila.
7. Executa.
8. Depura.
9. Entende erros.
10. Ajusta ambiente sem sair da IDE.

O foco aqui não é aparência isolada de componentes. Isso já foi coberto nas Partes 1, 2 e 3. O foco agora é **comportamento de produto**.

A IDE precisa parecer visualmente confortável, mas seu valor real será percebido quando o usuário pensar:

```text
"Eu não precisei brigar com CMake, PATH, compilador, preset, build directory e debugger."
```

---

## 2. Princípio principal: fluxo guiado, não mágico

A Kinein deve evitar dois extremos.

### 2.1. Extremo ruim 1 — IDE mágica demais

Não fazer:

```text
- instalar coisas sem pedir;
- alterar CMakeLists.txt automaticamente sem revisão;
- esconder comandos reais;
- criar arquivos confusos;
- inventar configuração sem explicar;
- assumir target errado;
- substituir conhecimento técnico por automação opaca.
```

### 2.2. Extremo ruim 2 — editor cru demais

Também não fazer:

```text
- jogar tudo no terminal e deixar o usuário resolver;
- depender só de JSON manual;
- exigir que o usuário conheça todos os flags;
- tratar CMake, Cargo e debugger como plugins soltos;
- mostrar erro bruto sem resumo;
- deixar estados da IDE ambíguos.
```

### 2.3. Direção correta

A Kinein deve ser:

```text
explícita;
visual;
reversível;
auditável;
confortável;
profissional;
orientada a engenharia;
familiar para quem vem de CLion/JetBrains;
mais simples para quem ainda está aprendendo.
```

A UI deve sempre permitir ver:

```text
- qual projeto está aberto;
- qual toolchain está ativa;
- qual profile está ativo;
- qual build system está ativo;
- qual target será compilado;
- qual executável será rodado;
- qual debugger será usado;
- qual comando real será executado;
- onde ficam os artefatos;
- o que falhou;
- como corrigir.
```

---

## 3. Mental model do usuário

O usuário não deve ser obrigado a pensar em termos internos como `JSON-RPC`, `daemon`, `scanner`, `adapter`, `job`, `session` ou `state machine`.

O modelo mental que a UI deve apresentar é:

```text
Projeto
  ↓
Ambiente
  ↓
Toolchain
  ↓
Configuração
  ↓
Build
  ↓
Run / Debug
```

Para C/C++ com CMake:

```text
Projeto CMake
  ↓
Kit / Toolchain
  ↓
Preset ou Profile
  ↓
Configure
  ↓
Build
  ↓
Run / Debug
```

Para Rust com Cargo:

```text
Projeto Cargo
  ↓
Rust toolchain
  ↓
Target / Features / Profile
  ↓
Build
  ↓
Run / Debug
```

Para embarcados no futuro:

```text
Projeto
  ↓
Cross toolchain
  ↓
Target board / sysroot / toolchain file
  ↓
Build firmware ou aplicação
  ↓
Flash / Deploy / Run remoto
  ↓
Debug / Serial / Logs
```

---

## 4. Níveis de maturidade do fluxo

A implementação deve ser feita por camadas. Não tentar implementar tudo de uma vez.

### 4.1. MVP 0 — Execução local básica

Objetivo:

```text
Abrir projeto, detectar ferramentas, escolher profile simples, buildar e rodar localmente.
```

Inclui:

```text
- detectar CMake;
- detectar Ninja ou Make;
- detectar GCC/Clang;
- detectar Cargo/rustc;
- detectar GDB/LLDB;
- abrir projeto;
- listar presets básicos;
- executar cmake configure;
- executar cmake --build;
- executar cargo build;
- rodar executável local;
- mostrar saída no terminal integrado;
- mostrar status claro.
```

Não inclui ainda:

```text
- cross-compilation complexa;
- flash em placa;
- debug remoto;
- QEMU completo;
- análise profunda de linker script;
- auto-correção de CMakeLists.txt.
```

### 4.2. MVP 1 — Toolchain visual e presets

Objetivo:

```text
Permitir configurar toolchains e profiles sem editar JSON manualmente.
```

Inclui:

```text
- tela de Toolchains;
- tela de CMake Profiles;
- validação de compilador;
- validação de debugger;
- cache de ferramentas detectadas;
- leitura de CMakePresets.json;
- criação opcional de profile local da Kinein;
- suporte básico a compile_commands.json;
- Problem Matcher para erros de GCC/Clang/Rust.
```

### 4.3. MVP 2 — Debug profissional local

Objetivo:

```text
Depurar aplicações C/C++/Rust localmente com experiência visual previsível.
```

Inclui:

```text
- breakpoints;
- start/stop/restart;
- step over;
- step into;
- step out;
- continue;
- threads;
- call stack;
- variables;
- watches;
- debug console;
- integração com GDB/LLDB via adapter;
- escolha de executável e argumentos;
- environment variables por configuração.
```

### 4.4. MVP 3 — Embedded/local remoto controlado

Objetivo:

```text
Começar a tratar Linux embarcado e software embarcado sem transformar a IDE em ferramenta de placa específica.
```

Inclui:

```text
- target remoto via SSH;
- deploy de binário;
- run remoto;
- logs remotos;
- serial monitor básico;
- configuração de sysroot;
- toolchain file CMake;
- validação de cross compiler;
- debug remoto como fluxo separado.
```

### 4.5. MVP 4 — Emulação (QEMU)

Objetivo:

```text
Adicionar emulação como workbench próprio, sem poluir o fluxo padrão.
```

Inclui futuramente:

```text
- QEMU targets;
- viewer de logs;
- painéis de telemetria;
- plots.
```

---

## 5. Arquitetura de produto para os fluxos

A Kinein deve manter separação rígida entre UI e core.

```text
Qt/QML UI
  - apresenta estado;
  - coleta intenção do usuário;
  - mostra logs e feedback;
  - nunca executa ferramentas diretamente.

Rust Core
  - detecta ferramentas;
  - valida toolchains;
  - executa processos;
  - parseia saída;
  - gerencia jobs;
  - publica eventos;
  - persiste estado seguro.

External Tools
  - cmake;
  - ninja/make;
  - gcc/clang;
  - cargo/rustc;
  - gdb/lldb;
  - ssh/scp;
  - qemu/openocd no futuro.
```

A regra é:

```text
UI nunca chama cmake diretamente.
UI pede ao core: "configure este workspace com este profile".
Core executa, captura, parseia e emite eventos.
```

Isso evita acoplamento, facilita testes e permite criar CLI depois usando o mesmo core.

---

## 6. Entidades principais do domínio da IDE

A Kinein precisa ter modelos claros. A IA CLI deve implementar esses modelos cedo, mesmo que alguns campos fiquem vazios no MVP.

### 6.1. Workspace

Representa o projeto aberto.

Campos mínimos:

```json
{
  "id": "workspace-id",
  "rootPath": "/home/user/project",
  "name": "motor-control-firmware",
  "kind": "cmake | cargo | mixed | unknown",
  "openedAt": "timestamp",
  "lastActiveProfileId": "debug-x86_64",
  "lastActiveTargetId": "motor-control"
}
```

Regras:

```text
- um workspace sempre tem uma raiz;
- a IDE não deve acessar arquivos fora da raiz sem permissão;
- workspace unknown ainda deve abrir;
- workspace mixed pode ter CMake e Cargo juntos;
- o estado da UI não deve ser misturado com o modelo técnico do projeto.
```

### 6.2. Toolchain

Representa um conjunto coerente de ferramentas para compilar e depurar.

Campos recomendados:

```json
{
  "id": "clang-system",
  "name": "Clang System",
  "kind": "native | cross | remote | custom",
  "languageFamily": "c-cpp | rust | mixed",
  "cCompiler": "/usr/bin/clang",
  "cppCompiler": "/usr/bin/clang++",
  "rustc": "/home/user/.cargo/bin/rustc",
  "cargo": "/home/user/.cargo/bin/cargo",
  "debugger": "/usr/bin/lldb",
  "cmake": "/usr/bin/cmake",
  "generator": "Ninja",
  "sysroot": null,
  "targetTriple": "x86_64-unknown-linux-gnu",
  "environment": {}
}
```

Regras:

```text
- toolchain não é só compilador;
- toolchain inclui debugger quando possível;
- toolchain pode ser incompleta;
- toolchain incompleta deve aparecer como "Needs attention";
- usuário pode corrigir manualmente;
- core deve validar caminhos antes de salvar;
- core nunca deve apagar toolchain sem confirmação.
```

### 6.3. Build Profile

Representa uma configuração de build.

Para CMake:

```json
{
  "id": "cmake-debug",
  "name": "Debug",
  "buildSystem": "cmake",
  "toolchainId": "clang-system",
  "presetName": "debug",
  "generator": "Ninja",
  "buildDir": "build/debug",
  "buildType": "Debug",
  "cacheVariables": {
    "CMAKE_EXPORT_COMPILE_COMMANDS": "ON"
  }
}
```

Para Rust:

```json
{
  "id": "cargo-dev",
  "name": "dev",
  "buildSystem": "cargo",
  "toolchainId": "rust-stable",
  "profile": "dev",
  "features": [],
  "targetTriple": "x86_64-unknown-linux-gnu"
}
```

Regras:

```text
- profile deve ser explícito;
- profile ativo deve aparecer na top toolbar;
- trocar profile não deve recompilar automaticamente sem consentimento;
- profile deve saber se está configurado, desatualizado ou quebrado;
- profile pode ser project-local ou user-local.
```

### 6.4. Build Target

Representa o alvo a ser compilado.

```json
{
  "id": "motor-control.elf",
  "name": "motor-control.elf",
  "kind": "executable | library | test | firmware | custom",
  "buildSystem": "cmake",
  "profileId": "cmake-debug",
  "outputPath": "build/debug/motor-control.elf"
}
```

Regras:

```text
- a IDE deve listar targets reais quando possível;
- target ativo deve aparecer na toolbar;
- se não houver target detectado, a IDE deve oferecer criar Run Configuration;
- targets de biblioteca não devem aparecer como Run por padrão;
- firmware pode ser buildável mas não executável localmente.
```

### 6.5. Run Configuration

Representa como executar algo.

```json
{
  "id": "run-motor-control-local",
  "name": "Run motor-control",
  "kind": "local-executable | cargo-bin | cargo-test | remote-ssh | qemu | custom",
  "targetId": "motor-control.elf",
  "executable": "build/debug/motor-control.elf",
  "workingDirectory": "${workspaceRoot}",
  "arguments": [],
  "environment": {},
  "beforeRun": ["build-active-target"],
  "terminalMode": "integrated"
}
```

Regras:

```text
- Run Configuration não deve ser confundida com Build Profile;
- várias Run Configurations podem usar o mesmo Build Target;
- beforeRun deve ser visível e desativável;
- usuário deve poder ver o comando final.
```

### 6.6. Debug Configuration

Representa como depurar algo.

```json
{
  "id": "debug-motor-control-local",
  "name": "Debug motor-control",
  "kind": "gdb | lldb | cppdbg | rust-gdb | remote-gdb",
  "runConfigurationId": "run-motor-control-local",
  "debuggerPath": "/usr/bin/gdb",
  "program": "build/debug/motor-control.elf",
  "arguments": [],
  "workingDirectory": "${workspaceRoot}",
  "environment": {},
  "setupCommands": [],
  "stopAtEntry": false
}
```

Regras:

```text
- Debug Configuration pode herdar de Run Configuration;
- debugger deve ser validado;
- se binário não existir, UI sugere Build antes de Debug;
- se símbolos de debug faltarem, UI deve avisar;
- breakpoints devem persistir por arquivo e linha.
```

---

## 7. Estado global do workspace

A UI sempre deve saber em que estado o workspace está.

Estados recomendados:

```text
NoWorkspace
OpeningWorkspace
ScanningWorkspace
DetectingTools
NeedsToolchain
NeedsConfigure
Ready
Configuring
Building
Running
Debugging
Stopping
Failed
```

### 7.1. Regras de transição

```text
NoWorkspace -> OpeningWorkspace
OpeningWorkspace -> ScanningWorkspace
ScanningWorkspace -> DetectingTools
DetectingTools -> NeedsToolchain | NeedsConfigure | Ready
NeedsToolchain -> DetectingTools | NeedsConfigure
NeedsConfigure -> Configuring
Configuring -> Ready | Failed
Ready -> Building | Running | Debugging
Building -> Ready | Failed
Running -> Ready | Failed
Debugging -> Ready | Failed
Failed -> NeedsToolchain | NeedsConfigure | Ready
```

### 7.2. Apresentação visual na UI

A UI deve refletir estado em três lugares:

```text
1. Top toolbar
   Mostra target, profile, status de build/debug.

2. Status bar
   Mostra estado compacto e persistente.

3. Tool window relevante
   Mostra detalhes completos, logs e ações.
```

Exemplos:

```text
Status bar: CMake: Ready
Status bar: Build: Running 62%
Status bar: Toolchain: Missing debugger
Status bar: Debug: attached
Status bar: Configure failed — see CMake
```

---

## 8. Fluxo 1 — Abrir workspace

Este é o primeiro fluxo crítico.

### 8.1. Objetivo

Abrir qualquer pasta sem assustar o usuário.

A IDE deve conseguir abrir:

```text
- projeto CMake válido;
- projeto Cargo válido;
- projeto misto;
- pasta com código C/C++ sem build system;
- pasta desconhecida;
- projeto quebrado;
- projeto ainda não configurado.
```

### 8.2. Passos internos

```text
1. Usuário seleciona pasta.
2. UI envia workspace.open(rootPath).
3. Core valida se a pasta existe.
4. Core verifica permissões.
5. Core identifica marcadores.
6. Core classifica tipo do projeto.
7. Core procura configurações locais da Kinein.
8. Core procura CMakePresets.json.
9. Core procura Cargo.toml.
10. Core procura compile_commands.json existente.
11. Core emite eventos de progresso.
12. UI mostra Project tool window.
13. UI mostra banner apenas se houver ação necessária.
```

### 8.3. Marcadores de projeto

Para CMake:

```text
CMakeLists.txt
CMakePresets.json
cmake/
build/
compile_commands.json
```

Para Rust:

```text
Cargo.toml
Cargo.lock
src/main.rs
src/lib.rs
rust-toolchain.toml
```

Para embarcados:

```text
*.ld
linker.ld
openocd.cfg
*.svd
memory.x
.cargo/config.toml
cmake/toolchain-*.cmake
```

Para C/C++ genérico:

```text
src/*.c
src/*.cpp
include/*.h
include/*.hpp
Makefile
meson.build
```

### 8.4. UI esperada

Se tudo estiver pronto:

```text
Project aberto.
Toolbar preenchida.
Build disponível.
Run disponível se target executável existir.
Debug disponível se debugger válido existir.
```

Se faltar toolchain:

```text
Banner discreto:
"Nenhuma toolchain válida encontrada para este projeto."
Botões: [Configurar toolchain] [Detectar novamente] [Ignorar por enquanto]
```

Se faltar configure:

```text
Banner discreto:
"Este projeto CMake ainda não foi configurado."
Botões: [Configurar Debug] [Escolher preset] [Abrir CMake]
```

Se for pasta desconhecida:

```text
Banner discreto:
"Projeto aberto como pasta simples."
Botões: [Criar CMake mínimo] [Criar Cargo project] [Somente editar arquivos]
```

---

## 9. Fluxo 2 — Detecção de ferramentas

### 9.1. Objetivo

Detectar ferramentas sem travar a UI e sem instalar nada sozinho.

Ferramentas iniciais:

```text
cmake
ninja
make
gcc
g++
clang
clang++
cc
c++
cargo
rustc
rustup
gdb
lldb
git
python, opcional para scripts
ssh, opcional para remoto futuro
```

### 9.2. Regra de execução

A detecção deve rodar em background.

```text
UI -> core: tools.detect
core -> UI: tools.detect.progress
core -> UI: tools.detect.result
```

A UI não deve congelar.

### 9.3. Estados de ferramenta

```text
Missing
Found
Valid
Invalid
UnsupportedVersion
NeedsConfiguration
```

Exemplo:

```json
{
  "tool": "cmake",
  "path": "/usr/bin/cmake",
  "version": "3.29.6",
  "status": "Valid",
  "source": "PATH"
}
```

### 9.4. Validação mínima

Para cada ferramenta:

```text
cmake --version
ninja --version
make --version
gcc --version
g++ --version
clang --version
clang++ --version
cargo --version
rustc --version
gdb --version
lldb --version
git --version
```

Para compiladores C/C++:

```text
- detectar versão;
- detectar target padrão;
- compilar um arquivo mínimo opcionalmente;
- detectar se C++ funciona, não apenas C;
- detectar suporte básico ao standard solicitado se possível.
```

Arquivo mínimo C++ para validação:

```cpp
#include <iostream>
int main() { std::cout << "kinein-toolchain-ok\n"; }
```

Arquivo mínimo C:

```c
#include <stdio.h>
int main(void) { puts("kinein-toolchain-ok"); return 0; }
```

Arquivo mínimo Rust:

```rust
fn main() {
    println!("kinein-toolchain-ok");
}
```

### 9.5. UI esperada

Toolchain Settings deve mostrar:

```text
Toolchain: Clang System
C compiler: /usr/bin/clang      Valid
C++ compiler: /usr/bin/clang++  Valid
CMake: /usr/bin/cmake           Valid
Generator: Ninja                Valid
Debugger: /usr/bin/lldb         Valid
Rust: stable                    Valid
```

Com botões:

```text
[Detectar novamente]
[Editar caminhos]
[Testar toolchain]
[Criar toolchain custom]
```

### 9.6. Erro bem tratado

Erro ruim:

```text
spawn ENOENT
```

Erro Kinein:

```text
CMake não foi encontrado no PATH.
Instale CMake ou selecione o caminho manualmente em Toolchains.
Comando testado: cmake --version
```

---

## 10. Fluxo 3 — Toolchain Setup

### 10.1. Objetivo

Dar ao usuário uma tela simples para resolver ambiente.

A tela de Toolchains deve ser uma das telas mais importantes da IDE.

Ela deve permitir:

```text
- ver toolchains detectadas;
- criar toolchain manual;
- validar compilador;
- validar CMake;
- validar debugger;
- configurar sysroot;
- configurar target triple;
- configurar environment variables;
- selecionar generator padrão;
- ver comando de teste;
- ver resultado do teste.
```

### 10.2. Toolchains padrão

A IDE pode sugerir automaticamente:

```text
GCC System
Clang System
Rust Stable
Rust Nightly
Cross GCC Custom
Remote Linux Target
```

### 10.3. Visual da tela

Layout recomendado:

```text
Settings / Toolchains

[Lista de toolchains]     [Detalhes da toolchain selecionada]
- GCC System              Name: GCC System
- Clang System            Kind: Native
- Rust Stable             C compiler: /usr/bin/gcc
- Custom ARM              C++ compiler: /usr/bin/g++
                           Debugger: /usr/bin/gdb
                           CMake: /usr/bin/cmake
                           Generator: Ninja
                           Sysroot: none
                           Target triple: x86_64-linux-gnu

                           [Testar] [Aplicar] [Duplicar]
```

### 10.4. Validação em camadas

A validação deve ser dividida:

```text
Layer 1 — Path existe.
Layer 2 — Executável responde --version.
Layer 3 — Compilador compila arquivo mínimo.
Layer 4 — Debugger consegue iniciar sessão mínima.
Layer 5 — CMake consegue configurar projeto mínimo.
Layer 6 — Toolchain funciona com projeto real.
```

MVP pode implementar até Layer 3.

### 10.5. Resultado visual

```text
✓ Path válido
✓ Versão detectada
✓ C++ mínimo compilou
✓ CMake mínimo configurou
! Debugger não encontrado
```

Botão sugerido:

```text
[Corrigir debugger]
```

O Assistente deve explicar:

```text
"Sua toolchain compila C/C++, mas ainda não possui debugger configurado. Você poderá buildar e rodar, mas o botão Debug ficará indisponível até configurar GDB ou LLDB."
```

---

## 11. Fluxo 4 — CMake Configure

### 11.1. Objetivo

Transformar CMake em um fluxo visual previsível.

O usuário não deve precisar lembrar sempre:

```bash
cmake -S . -B build/debug -G Ninja -DCMAKE_BUILD_TYPE=Debug
```

Mas deve poder ver esse comando quando quiser.

### 11.2. Estados de CMake

```text
NotDetected
Detected
PresetAvailable
NeedsConfigure
Configuring
Configured
Outdated
Failed
```

### 11.3. CMake Configure flow

```text
1. Workspace aberto.
2. Core detecta CMakeLists.txt.
3. Core procura CMakePresets.json.
4. Core lista presets se existirem.
5. Se não houver preset, IDE sugere profile padrão.
6. Usuário escolhe profile.
7. Core monta comando.
8. UI mostra resumo antes da primeira execução.
9. Usuário confirma.
10. Core executa configure.
11. Core parseia saída.
12. Core salva estado do profile.
13. UI mostra sucesso ou falha.
```

### 11.4. Tela de primeira configuração

Quando abrir projeto CMake sem configuração:

```text
Card central discreto no editor ou painel CMake:

Projeto CMake detectado
Este projeto ainda não foi configurado na Kinein.

Profile: [Debug ▼]
Generator: [Ninja ▼]
Toolchain: [Clang System ▼]
Build dir: build/debug
Export compile_commands.json: [x]

Comando:
cmake -S . -B build/debug -G Ninja -DCMAKE_BUILD_TYPE=Debug -DCMAKE_EXPORT_COMPILE_COMMANDS=ON

[Configurar agora]
[Escolher preset]
[Abrir configurações]
```

### 11.5. CMakePresets.json

Se existir `CMakePresets.json`, a IDE deve respeitar.

Regras:

```text
- presets do projeto têm prioridade;
- Kinein não deve sobrescrever CMakePresets.json automaticamente;
- Kinein pode sugerir criar presets, mas só com confirmação;
- presets inválidos devem aparecer como erro compreensível;
- presets ocultos não devem ser destacados como escolha principal;
- inherited presets devem ser resolvidos pelo core.
```

### 11.6. Profiles locais da Kinein

Se o projeto não tiver presets, a IDE pode usar profiles locais em `.kinein/` ou estado de usuário.

Recomendação:

```text
.kinein/profiles.json        project-local, se o usuário aceitar salvar no projeto
~/.config/kinein/workspaces  user-local, se for configuração pessoal
```

Regra:

```text
A IDE sempre deve perguntar antes de criar arquivos no projeto.
```

### 11.7. Output parser

Durante configure, a IDE deve identificar:

```text
- compilador selecionado;
- generator usado;
- build directory;
- erro de cache;
- variável CMake inválida;
- compilador ausente;
- ABI detectada;
- toolchain file quebrado;
- package não encontrado;
- erro em CMakeLists.txt;
- warning relevante.
```

Exemplo de erro bruto:

```text
CMake Error at CMakeLists.txt:14 (find_package):
  Could not find a package configuration file provided by "Foo".
```

Erro Kinein:

```text
Pacote CMake não encontrado: Foo
Arquivo: CMakeLists.txt:14
Causa provável: dependência não instalada ou CMAKE_PREFIX_PATH ausente.
Ações: [Abrir linha] [Adicionar CMAKE_PREFIX_PATH] [Ver comando]
```

---

## 12. Fluxo 5 — Build

### 12.1. Objetivo

Compilar com feedback claro e sem poluir o editor.

O Build deve ser acionável por:

```text
- botão Build na toolbar;
- atalho;
- menu Build;
- beforeRun;
- command palette;
- painel CMake/Cargo.
```

### 12.2. Estados de Build

```text
Idle
Queued
Preparing
Running
ParsingOutput
Succeeded
SucceededWithWarnings
Failed
Cancelled
```

### 12.3. Build flow CMake

```text
1. Usuário clica Build.
2. UI envia build.run(profileId, targetId).
3. Core verifica se profile está configurado.
4. Se não estiver, pede configure primeiro.
5. Core monta comando cmake --build.
6. Core inicia job.
7. Core emite eventos de stdout/stderr.
8. Core parseia warnings/errors.
9. UI atualiza Build tool window.
10. UI atualiza Problems.
11. UI atualiza status bar.
12. Core registra artifact final.
```

Comando típico:

```bash
cmake --build build/debug --target motor-control -j 8
```

### 12.4. Build flow Cargo

```text
1. Usuário clica Build.
2. Core detecta Cargo.toml.
3. Core monta comando cargo build.
4. Se houver target triple, adiciona --target.
5. Se houver features, adiciona --features.
6. Core parseia saída rustc/cargo.
7. UI mostra mensagens em Problems.
```

Comando típico:

```bash
cargo build --profile dev
```

Com target:

```bash
cargo build --target thumbv7em-none-eabihf
```

### 12.5. Build tool window

A aba Build deve ter três modos:

```text
1. Summary
2. Output
3. Problems
```

Summary:

```text
Build: Failed
Profile: Debug
Target: motor-control
Duration: 2.31s
Errors: 1
Warnings: 3
Command: cmake --build build/debug --target motor-control -j 8
```

Output:

```text
Saída bruta, colorida, pesquisável, copiável.
```

Problems:

```text
Lista estruturada:
File | Line | Severity | Message | Tool
```

### 12.6. Problem matcher

A Kinein deve converter erros em diagnostics.

Campos:

```json
{
  "file": "src/drivers/motor/motor_control.cpp",
  "line": 42,
  "column": 17,
  "severity": "error",
  "message": "use of undeclared identifier 'pidController'",
  "tool": "clang++",
  "raw": "..."
}
```

Para GCC/Clang:

```text
file:line:column: error: message
file:line:column: warning: message
```

Para MSVC futuro:

```text
file(line,column): error Cxxxx: message
```

Para Rust:

```text
Preferir JSON output se possível:
cargo build --message-format=json
```

### 12.7. UX de falha

Quando build falhar:

```text
- não abrir popup agressivo;
- destacar aba Problems;
- mostrar erro principal no Build Summary;
- marcar arquivos no Project;
- mostrar badge na status bar;
- Assistente oferece resumo.
```

Exemplo Assistente:

```text
O build falhou por erro de compilação em motor_control.cpp:42.
O símbolo pidController não foi declarado no escopo atual.
Possíveis causas:
1. include ausente;
2. nome incorreto;
3. variável movida para outro namespace.
```

---

## 13. Fluxo 6 — Run

### 13.1. Objetivo

Executar binários e projetos sem confundir build target com run configuration.

### 13.2. Estados de Run

```text
Idle
Preparing
BuildingBeforeRun
Launching
Running
ExitedSuccessfully
ExitedWithError
Killed
FailedToLaunch
```

### 13.3. Run local C/C++

```text
1. Usuário seleciona Run Configuration.
2. Clica Run.
3. Core verifica beforeRun.
4. Se beforeRun ativo, executa Build.
5. Core verifica se executável existe.
6. Core inicia processo.
7. UI abre Run/Terminal tool window.
8. Stdout/stderr aparecem em tempo real.
9. Status bar mostra processo ativo.
10. Usuário pode Stop/Restart.
```

### 13.4. Run Cargo

Opções:

```text
- cargo run;
- cargo run --bin name;
- cargo test;
- executar binário já compilado.
```

MVP recomendado:

```text
Cargo Run Configuration usa cargo run no início.
Depois evolui para executar binários target/debug quando fizer sentido.
```

### 13.5. Run Configuration UI

Tela:

```text
Run Configurations

Name: Run motor-control
Kind: Local executable
Build before run: [x]
Executable: build/debug/motor-control.elf
Working directory: ${workspaceRoot}
Arguments: --log-level debug
Environment:
  MOTOR_PORT=/dev/ttyUSB0

[Run] [Debug] [Apply]
```

### 13.6. Falhas comuns

Executável não existe:

```text
O executável não foi encontrado.
Target esperado: build/debug/motor-control.elf
Ações: [Build agora] [Editar Run Configuration] [Ver build directory]
```

Permissão negada:

```text
O arquivo existe, mas não possui permissão de execução.
Ações: [chmod +x] [Abrir terminal] [Editar configuração]
```

Processo retorna código não-zero:

```text
Processo finalizado com código 1.
Ações: [Ver saída] [Rodar novamente] [Debug]
```

---

## 14. Fluxo 7 — Debug

### 14.1. Objetivo

Oferecer debug visual confiável sem reinventar GDB/LLDB.

A Kinein deve usar adapters/protocolos, não parse manual frágil no longo prazo.

Opções de arquitetura:

```text
- GDB/MI adapter próprio mínimo;
- LLDB adapter;
- Debug Adapter Protocol como camada;
- integração gradual por backend.
```

Recomendação pragmática:

```text
MVP: adapter mínimo controlado pelo core.
Longo prazo: camada de debug abstraída para GDB, LLDB e remote debug.
```

### 14.2. Estados de Debug

```text
Idle
Preparing
BuildingBeforeDebug
LaunchingDebugger
Attaching
Running
Paused
Stepping
Stopped
Terminated
Failed
```

### 14.3. Debug local flow

```text
1. Usuário clica Debug.
2. Core verifica Debug Configuration.
3. Core verifica binário.
4. Core verifica símbolos de debug se possível.
5. Core executa build before debug se ativo.
6. Core inicia debugger adapter.
7. Core registra breakpoints.
8. Core inicia programa.
9. UI muda para layout Debug.
10. Painéis Variables/Call Stack/Watches aparecem.
11. Usuário controla execução.
12. Encerramento limpa sessão.
```

### 14.4. Layout Debug

Quando Debug inicia, a IDE deve manter memória muscular, mas destacar painéis úteis.

```text
Centro: editor
Esquerda: Project ou Debug Sessions
Direita: Variables/Watches opcional
Inferior: Debug Console, Call Stack, Breakpoints, Threads
Toolbar: Continue, Step Over, Step Into, Step Out, Restart, Stop
```

Não mudar tudo agressivamente. A sensação deve ser:

```text
"Estou na mesma IDE, agora com ferramentas de debug visíveis."
```

### 14.5. Breakpoints

Modelo:

```json
{
  "id": "bp-1",
  "file": "src/main.cpp",
  "line": 42,
  "enabled": true,
  "condition": null,
  "hitCondition": null,
  "logMessage": null
}
```

MVP:

```text
- breakpoint simples;
- enable/disable;
- remover;
- persistir por workspace;
- mostrar estado: pending/resolved/invalid.
```

Longo prazo:

```text
- conditional breakpoint;
- logpoint;
- hit count;
- data breakpoint quando suportado;
- function breakpoint.
```

### 14.6. Falhas de Debug

Debugger ausente:

```text
Debugger não configurado para a toolchain ativa.
Ações: [Configurar debugger] [Rodar sem debug] [Detectar novamente]
```

Binário sem símbolo:

```text
O binário parece não conter símbolos de debug.
Profile atual: Release
Sugestão: usar profile Debug ou RelWithDebInfo.
Ações: [Trocar para Debug] [Continuar mesmo assim]
```

Breakpoint não resolvido:

```text
Breakpoint pendente: este arquivo ainda não foi carregado pelo debugger.
Ele será ativado quando o módulo correspondente for carregado.
```

---

## 15. Integração com Assistente

O Assistente não deve ser um chat genérico nesta etapa. Ele deve ser um painel contextual para reduzir ansiedade.

### 15.1. Durante Toolchain Setup

Mostrar:

```text
- resumo da toolchain;
- ferramenta ausente;
- por que isso importa;
- ação recomendada;
- comando testado.
```

### 15.2. Durante CMake Configure

Mostrar:

```text
- profile ativo;
- comando configurado;
- variáveis relevantes;
- erro principal;
- onde clicar;
- sugestões seguras.
```

### 15.3. Durante Build

Mostrar:

```text
- erro principal;
- arquivo/linha;
- se parece erro de sintaxe, include, linker ou CMake;
- sugestão de investigação;
- links para Problems e Output.
```

### 15.4. Durante Run

Mostrar:

```text
- processo em execução;
- argumentos;
- cwd;
- exit code;
- stdout/stderr relevante.
```

### 15.5. Durante Debug

Mostrar:

```text
- estado atual: running/paused;
- função atual;
- motivo da pausa;
- breakpoint atingido;
- variável suspeita;
- sugestão de próximo passo.
```

Regra crítica:

```text
Assistente sugere. Não altera código automaticamente sem ação explícita do usuário.
```

---

## 16. Comandos internos JSON-RPC sugeridos

A lista abaixo não é uma API final. É um contrato inicial para orientar a IA CLI.

### 16.1. Workspace

```text
workspace.open
workspace.close
workspace.status
workspace.scan
workspace.kind.detect
```

### 16.2. Tools

```text
tools.detect
tools.status
tools.validate
tools.version
```

### 16.3. Toolchain

```text
toolchain.list
toolchain.create
toolchain.update
toolchain.delete
toolchain.validate
toolchain.setActive
toolchain.testCompile
```

### 16.4. CMake

```text
cmake.detect
cmake.presets.list
cmake.profiles.list
cmake.profile.create
cmake.profile.update
cmake.configure
cmake.cache.read
cmake.targets.list
cmake.clean
```

### 16.5. Cargo

```text
cargo.detect
cargo.metadata
cargo.targets.list
cargo.build
cargo.test
cargo.run
```

### 16.6. Build

```text
build.run
build.cancel
build.status
build.history
build.artifacts.list
```

### 16.7. Run

```text
run.configurations.list
run.configuration.create
run.configuration.update
run.start
run.stop
run.restart
run.status
```

### 16.8. Debug

```text
debug.configurations.list
debug.configuration.create
debug.start
debug.stop
debug.restart
debug.continue
debug.pause
debug.stepOver
debug.stepInto
debug.stepOut
debug.breakpoints.list
debug.breakpoints.set
debug.breakpoints.remove
debug.variables.list
debug.stackTrace
debug.threads
```

### 16.9. Jobs/Event stream

```text
job.list
job.status
job.cancel
event.subscribe
event.unsubscribe
```

Eventos:

```text
workspace.scanned
tools.detected
toolchain.validation.updated
cmake.configure.started
cmake.configure.output
cmake.configure.finished
build.started
build.output
build.problem.detected
build.finished
run.started
run.output
run.finished
debug.started
debug.paused
debug.continued
debug.stopped
```

---

## 17. Jobs e concorrência

A Kinein deve tratar operações longas como jobs.

Jobs:

```text
- tool detection;
- toolchain validation;
- cmake configure;
- build;
- cargo metadata;
- run process;
- debug session;
- indexação futura.
```

Cada job deve ter:

```json
{
  "id": "job-123",
  "kind": "build",
  "title": "Build motor-control",
  "state": "running",
  "progress": 62,
  "startedAt": "timestamp",
  "canCancel": true
}
```

Regras:

```text
- um build pode ser cancelado;
- configure pode ser cancelado;
- run pode ser parado;
- debug pode ser parado;
- dois builds do mesmo profile não devem rodar ao mesmo tempo sem decisão explícita;
- a UI deve mostrar jobs ativos na status bar;
- logs devem permanecer consultáveis após término.
```

---

## 18. Persistência

Separar estado técnico, configuração do usuário e artefatos gerados.

### 18.1. User-global

Local sugerido:

```text
~/.config/kinein/
```

Conteúdo:

```text
toolchains.json
recent-workspaces.json
ui-state.json
settings.json
```

### 18.2. Workspace-local privado

Local:

```text
<workspace>/.kinein/
```

Conteúdo opcional:

```text
workspace.json
profiles.json
run-configurations.json
debug-configurations.json
breakpoints.json
```

Regra:

```text
Criar .kinein/ no projeto só com confirmação ou configuração explícita.
```

### 18.3. Build artifacts

Para CMake:

```text
build/debug
build/release
build/relwithdebinfo
```

Para Cargo:

```text
target/debug
target/release
```

Regra:

```text
A IDE não deve apagar diretórios de build sem confirmação explícita.
```

---

## 19. UX das ações principais

### 19.1. Toolbar principal

Ordem recomendada:

```text
[Target Selector] [Profile Selector] [Configure] [Build] [Run] [Debug] [Stop] [More]
```

Exemplo:

```text
Target: motor-control.elf  |  CMake: Debug  |  Configure  Build  Run  Debug
```

### 19.2. Estados dos botões

Configure:

```text
enabled quando projeto CMake existe;
highlight quando configure necessário;
disabled em projeto Cargo puro.
```

Build:

```text
enabled quando profile válido existe;
disabled se toolchain ausente;
loading durante build.
```

Run:

```text
enabled quando Run Configuration válida existe;
se executável ausente, oferece Build;
loading durante processo.
```

Debug:

```text
enabled quando Debug Configuration e debugger válidos existem;
warning se profile sem símbolos;
disabled se debugger ausente.
```

Stop:

```text
visível quando run/debug/build ativo;
vermelho discreto;
nunca usar popup para parar, a menos que haja risco de perda.
```

---

## 20. Painéis necessários para esta etapa

### 20.1. Toolchains tool window ou Settings page

Função:

```text
Resolver ambiente.
```

Elementos:

```text
- lista de toolchains;
- validação;
- caminho de ferramentas;
- generator;
- debugger;
- environment;
- botão testar.
```

### 20.2. CMake tool window

Função:

```text
Resolver configuração CMake.
```

Elementos:

```text
- profile ativo;
- presets;
- cache variables importantes;
- configure output;
- targets;
- comando real;
- botão configure.
```

### 20.3. Build tool window

Função:

```text
Entender compilação.
```

Elementos:

```text
- resumo;
- output;
- problems;
- duration;
- comando;
- artefatos.
```

### 20.4. Run tool window

Função:

```text
Interagir com processo.
```

Elementos:

```text
- stdout;
- stderr;
- input quando terminal interativo;
- exit code;
- restart;
- stop.
```

### 20.5. Debug tool window

Função:

```text
Depurar.
```

Elementos:

```text
- call stack;
- variables;
- watches;
- breakpoints;
- threads;
- debug console;
- controls.
```

### 20.6. Problems tool window

Função:

```text
Agregador central de erros.
```

Elementos:

```text
- build errors;
- configure errors;
- static diagnostics;
- linker errors;
- rustc errors;
- filtros;
- agrupamento por arquivo.
```

---

## 21. Fluxos de erro que precisam ser excelentes

Uma IDE para C/C++/Rust será julgada pela qualidade dos erros.

### 21.1. CMake cache quebrado

Mensagem:

```text
O cache CMake parece incompatível com a configuração atual.
Isso pode ocorrer após trocar compilador, generator ou build directory.
```

Ações:

```text
[Reconfigurar]
[Limpar build dir]
[Ver cache]
[Cancelar]
```

### 21.2. Compilador não encontrado

Mensagem:

```text
O compilador C++ configurado não existe ou não pode ser executado.
```

Ações:

```text
[Selecionar compilador]
[Detectar novamente]
[Ver Toolchain]
```

### 21.3. Linker error

Mensagem:

```text
A compilação terminou, mas a linkedição falhou.
Causa provável: símbolo não definido, biblioteca ausente ou ordem de link.
```

Ações:

```text
[Ver erro principal]
[Abrir CMakeLists.txt]
[Ver comando de link]
```

### 21.4. Rust target ausente

Mensagem:

```text
O target Rust configurado não está instalado.
```

Ações:

```text
[Mostrar comando rustup]
[Alterar target]
[Configurar toolchain]
```

A IDE pode mostrar o comando, mas não executar sem confirmação:

```bash
rustup target add thumbv7em-none-eabihf
```

### 21.5. Debugger incompatível

Mensagem:

```text
O debugger selecionado não parece compatível com este binário ou target.
```

Ações:

```text
[Escolher outro debugger]
[Rodar sem debug]
[Ver detalhes]
```

---

## 22. Como manter a sensação JetBrains-like

A Kinein deve ser familiar, mas não clone.

### 22.1. O que copiar como princípio

```text
- toolbar compacta;
- tool windows previsíveis;
- editor no centro;
- actions sempre acessíveis;
- status bar informativa;
- configurações por projeto;
- debug layout estável;
- problemas agrupados;
- command palette/search everywhere futuramente.
```

### 22.2. O que não copiar literalmente

```text
- ícones específicos;
- layout exato;
- nomes proprietários;
- fluxos proprietários;
- estilo gráfico idêntico;
- dependência de convenções internas JetBrains.
```

### 22.3. Identidade Kinein

```text
- âmbar industrial;
- Assistente;
- foco em toolchain visível;
- CMake como cidadão de primeira classe;
- Rust como cidadão de primeira classe;
- sistemas e embedded desde o design inicial.
```

---

## 23. Critérios de aceite para IA CLI

A IA CLI deve implementar por etapas e marcar progresso.

### 23.1. Etapa A — Modelos

Concluído quando existir:

```text
WorkspaceInfo
ToolInfo
ToolchainInfo
BuildProfile
BuildTarget
RunConfiguration
DebugConfiguration
JobInfo
ProblemDiagnostic
```

### 23.2. Etapa B — Tool detection

Concluído quando:

```text
- tools.detect lista ferramentas;
- tools.status retorna cache;
- UI mostra ferramenta válida/inválida;
- nenhum comando bloqueia UI.
```

### 23.3. Etapa C — CMake configure

Concluído quando:

```text
- projeto CMake é detectado;
- CMakePresets.json é listado se existir;
- profile Debug default funciona sem preset;
- configure roda pelo core;
- output aparece na UI;
- falha aparece no Problems.
```

### 23.4. Etapa D — Build

Concluído quando:

```text
- Build roda para CMake;
- Build roda para Cargo;
- output é capturado;
- errors/warnings são parseados;
- Problems abre arquivo/linha;
- status bar atualiza.
```

### 23.5. Etapa E — Run

Concluído quando:

```text
- Run Configuration local existe;
- executável é iniciado pelo core;
- stdout/stderr aparecem;
- stop funciona;
- exit code aparece;
- beforeRun opcional funciona.
```

### 23.6. Etapa F — Debug MVP

Concluído quando:

```text
- Debug Configuration existe;
- GDB ou LLDB inicia;
- breakpoint simples funciona;
- continue/step/stop funcionam;
- call stack básico aparece;
- variables básico aparece.
```

---

## 24. Não fazer agora

Para proteger qualidade, não implementar nesta etapa:

```text
- marketplace de plugins;
- QEMU completo;
- flash em placa real;
- suporte profundo a Make/Meson/Bazel;
- refatoração avançada;
- indexador próprio;
- auto-instalação de dependências;
- UI complexa de dashboard;
- IA alterando código automaticamente.
```

Esses itens podem vir depois. O objetivo agora é o núcleo emocional da IDE:

```text
abrir projeto → configurar ambiente → buildar → rodar → depurar.
```

---

## 25. Checklist final desta parte

A Parte 4 está bem implementada quando o usuário consegue dizer:

```text
- Entendi qual toolchain estou usando.
- Entendi qual profile estou compilando.
- Entendi qual comando a IDE executou.
- Entendi por que o configure falhou.
- Entendi por que o build falhou.
- Consegui rodar meu programa.
- Consegui parar meu programa.
- Consegui iniciar debug.
- Consegui ver breakpoints, stack e variáveis.
- Não precisei sair da IDE para resolver o básico.
```

A promessa da Kinein nesta fase:

```text
Focus on code, not toolchains.
```

Essa frase não é só slogan. É o requisito de produto.

---

## 26. Ordem recomendada para implementação real

A ordem mais segura:

```text
1. Consolidar modelos no core.
2. Implementar Job system.
3. Implementar tools.detect assíncrono.
4. Implementar Toolchains UI simples.
5. Implementar CMake configure.
6. Implementar Build CMake.
7. Implementar Problems parser básico GCC/Clang.
8. Implementar Cargo metadata/build.
9. Implementar Run Configuration local.
10. Implementar Debug Configuration MVP.
11. Integrar Assistente com mensagens estruturadas.
12. Refinar UI/UX, atalhos e estados.
```

A ordem errada seria começar pelo debug visual antes de ter build e run sólidos.

---

## 27. Prompt de trabalho para IA CLI

Use este prompt quando for pedir à IA CLI para implementar esta parte:

```text
Implemente a Parte 4 da Kinein Vectis com foco em fluxos de produto para Toolchain, CMake, Build, Run e Debug. Siga o documento KINEIN_VECTIS_PRODUCT_FLOWS_BUILD_RUN_DEBUG.md como fonte de verdade.

Priorize a implementação em camadas: modelos, job system, tool detection, toolchain validation, CMake configure, build, problems parser, run local e debug MVP. A UI Qt/QML deve apenas solicitar ações e renderizar estado; o Rust core deve executar processos, validar ferramentas, gerenciar jobs, parsear saída e emitir eventos.

Não implemente QEMU completo, flash real, plugin marketplace ou IA que altera código automaticamente nesta etapa. Mantenha a experiência JetBrains-like na previsibilidade: editor no centro, toolbar compacta, tool windows claras, status bar informativa e feedback discreto.

Ao implementar, sempre exponha: comando real executado, estado do job, logs, problemas estruturados, ações de correção e mensagens compreensíveis para o usuário.
```

---

## 28. Resumo executivo

A Parte 4 transforma a Kinein de uma interface bonita em uma IDE de verdade.

Ela define o coração operacional:

```text
Toolchain
CMake
Cargo
Build
Run
Debug
Problems
Jobs
Assistente
```

O usuário pode começar com C/C++ local, migrar para Rust ou Python, e avançar para Linux embarcado e bare metal sem que a IDE precise mudar de filosofia.

A Kinein deve ser uma IDE para engenharia de sistemas: confortável, explícita, visual, rigorosa e útil.
