# Kinein Vectis — Parte 6

# Sistemas Embarcados, Targets, Flash, Serial, Remote SSH e QEMU

> **Nome oficial:** Kinein Vectis  
> **Nome curto:** Kinein  
> **Sigla visual:** KV  
> **Foco:** C, C++ e Rust  
> **Posicionamento:** IDE Linux-first para sistemas, toolchains, CMake, Rust/Cargo, software embarcado, Linux embarcado, targets remotos e simulação futura.  
> **Objetivo desta parte:** especificar como a IDE deve tratar ambientes embarcados de forma profissional, previsível e implementável por IA CLI, sem depender inicialmente de hardware físico obrigatório.

---

## 0. Nota importante sobre Tree-sitter

A Parte 5 descreveu editor, LSP, `clangd`, `rust-analyzer`, diagnósticos e refatoração, mas **Tree-sitter ainda não estava explícito**. Para a Kinein, ele deve entrar como uma camada própria do editor.

A decisão recomendada é:

```text
Tree-sitter: análise local incremental, rápida e estrutural do texto.
LSP: inteligência semântica profunda via clangd/rust-analyzer.
```

Tree-sitter **não substitui** `clangd` nem `rust-analyzer`. Ele complementa.

### 0.1 Onde Tree-sitter entra

```text
Editor Buffer
  ↓
Tree-sitter Parser Layer
  ↓
Syntax Tree local e incremental
  ↓
Recursos imediatos do editor
  ├── syntax highlighting estrutural
  ├── folding
  ├── bracket matching
  ├── seleção expandida por escopo
  ├── outline local rápido
  ├── breadcrumbs simples
  ├── detecção básica de função/classe/bloco
  └── navegação estrutural leve

LSP Layer
  ├── clangd
  ├── rust-analyzer
  └── diagnósticos/refatoração/semântica profunda
```

### 0.2 Responsabilidades do Tree-sitter na Kinein

Tree-sitter deve cuidar de:

- parsing incremental por arquivo aberto;
- realce estrutural quando o LSP ainda não carregou;
- folding de blocos;
- seleção por escopo;
- outline local rápido;
- breadcrumbs locais;
- detecção de símbolos simples;
- suporte futuro para queries customizadas por linguagem;
- melhor UX em arquivos grandes;
- fallback quando `clangd` ou `rust-analyzer` não estiverem disponíveis.

Tree-sitter **não deve** cuidar de:

- resolução completa de tipos;
- includes complexos;
- macros C/C++ profundas;
- template instantiation;
- borrow checker;
- refatoração semântica pesada;
- análise de projeto inteiro;
- erros reais de compilação;
- decisões de build.

### 0.3 Linguagens iniciais recomendadas

Para o MVP da Kinein:

```text
Obrigatórias:
- C
- C++
- Rust
- CMake
- TOML
- JSON
- YAML
- Shell
- Markdown

Futuras:
- GLSL
- QML
- Python auxiliar
- Linker scripts
- Device tree
- Kconfig
```

### 0.4 Decisão arquitetural

A Kinein deve ter um serviço explícito:

```text
SyntaxTreeService
```

Esse serviço não é o mesmo que `LanguageService`.

```text
LanguageService
  ├── LSP Client
  ├── clangd
  ├── rust-analyzer
  └── semantic operations

SyntaxTreeService
  ├── Tree-sitter parsers
  ├── syntax queries
  ├── local AST cache
  └── structural editor features
```

Essa separação evita misturar análise local rápida com inteligência semântica pesada.

---

## 1. Propósito da Parte 6

A Parte 4 definiu fluxos de **Toolchain, CMake, Build, Run e Debug**. A Parte 5 definiu **Editor e Language Intelligence**. A Parte 6 define o diferencial mais estratégico da Kinein: tratar software embarcado como fluxo de produto, e não como um conjunto solto de scripts.

A Kinein deve permitir que um programador de C, C++ ou Rust trabalhe com:

- projeto local comum;
- projeto CMake para desktop;
- projeto CMake cross-compilado;
- projeto Rust/Cargo local;
- Rust com targets específicos;
- Linux embarcado remoto via SSH;
- placa física com flash/debug;
- monitor serial;
- QEMU/emulação;
- sysroot e SDK;
- toolchain file;
- build profiles;
- deploy remoto;
- logs e telemetria básica.

A frase-guia desta etapa é:

```text
A IDE deve transformar ambiente embarcado em algo configurável visualmente, rastreável e reversível.
```

---

## 2. Princípio central: target é entidade de primeira classe

Em IDEs comuns, o usuário normalmente configura:

```text
Build → Run → Debug
```

Na Kinein, o fluxo deve ser:

```text
Target → Toolchain → Build Profile → Deploy/Run/Debug/Observe
```

Isso muda tudo.

Um projeto embarcado não é apenas um código-fonte. Ele tem um alvo.

Exemplos de alvos:

```text
Local Linux x86_64
Local Linux ARM via sysroot
Remote Linux SBC via SSH
Bare-metal ARM Cortex-M
QEMU ARM Linux
QEMU RISC-V
Industrial PC remoto
Emulador customizado
```

A IDE deve sempre responder claramente:

```text
Estou compilando para quem?
Estou rodando onde?
Estou depurando como?
Estou observando quais logs?
```

---

## 3. Modelo conceitual de Target

A entidade `Target` deve ser persistida no workspace.

### 3.1 Campos mínimos

```json
{
  "id": "local-x86_64-debug",
  "name": "Local Linux x86_64",
  "kind": "local_linux",
  "architecture": "x86_64",
  "os": "linux",
  "toolchainId": "gcc-system",
  "buildProfileId": "cmake-debug",
  "runnerId": "local-process",
  "debuggerId": "gdb-local",
  "serialId": null,
  "deployId": null,
  "qemuId": null,
  "enabled": true
}
```

### 3.2 Tipos de Target

```text
local_linux
remote_linux_ssh
bare_metal_mcu
qemu_linux
qemu_bare_metal
container_future
custom_external
```

### 3.3 Campos por categoria

#### Local Linux

```text
architecture
compiler
cmake preset
cargo target
runner
working directory
environment variables
```

#### Remote Linux via SSH

```text
host
port
user
authentication mode
remote workspace path
remote deploy path
remote run command
remote environment
rsync/scp mode
remote debugger mode
```

#### Bare-metal MCU

```text
architecture
vendor family
flash tool
debug probe
openocd config
probe-rs config futuro
memory layout
firmware artifact
serial port
reset strategy
```

#### QEMU

```text
architecture
machine
cpu
kernel image
rootfs image
dtb
initrd
extra args
network mode
serial mode
gdb stub port
```

---

## 4. Target Selector no layout

A toolbar superior deve ter um seletor de target visível.

Exemplo:

```text
[ Target: Local x86_64 Linux ▼ ] [ Profile: CMake Debug ▼ ] [ Configure ] [ Build ] [ Run ] [ Debug ]
```

Para projeto embarcado:

```text
[ Target: STM32F4 OpenOCD ▼ ] [ Profile: Debug ▼ ] [ Build ] [ Flash ] [ Debug ] [ Serial ]
```

Para Linux embarcado remoto:

```text
[ Target: RPi Remote SSH ▼ ] [ Profile: Release ARM64 ▼ ] [ Build ] [ Deploy ] [ Run ] [ Debug ] [ Logs ]
```

Para QEMU:

```text
[ Target: QEMU ARMv7 ▼ ] [ Profile: Debug ▼ ] [ Build ] [ Boot QEMU ] [ Debug ] [ Serial ]
```

### 4.1 Regra psicológica

O usuário não deve precisar lembrar qual terminal ou script usar.

A IDE deve deixar evidente:

```text
Target selecionado
Toolchain usada
Profile de build
Artefato gerado
Ação principal disponível
Estado atual
```

---

## 5. Tool Window: Targets

A Kinein deve ter uma Tool Window própria chamada:

```text
Targets
```

Ela fica no painel esquerdo, junto de Project, Search, Git, Build, Debug e Simulation futura.

### 5.1 Estrutura visual

```text
Targets
├── Local
│   └── Local Linux x86_64
├── Remote Linux
│   └── RPi Lab
├── Bare-metal
│   ├── STM32F4 Discovery
│   └── RP2040 Probe
├── QEMU
│   ├── ARMv7 Linux
│   └── RISC-V Linux
└── Add Target...
```

### 5.2 Card de target

Cada target deve aparecer como um card compacto:

```text
┌──────────────────────────────────────────┐
│ ● Local Linux x86_64                     │
│ gcc 15 · CMake Debug · Ready             │
│ [Select] [Edit] [Duplicate]              │
└──────────────────────────────────────────┘
```

Estados:

```text
Ready
Missing toolchain
Missing sysroot
Connection failed
Needs configure
Build failed
Flash ready
Running
Debugging
Serial open
```

### 5.3 Cores de estado

```text
Ready: verde discreto
Needs configure: âmbar
Missing: vermelho discreto
Running: azul técnico
Debugging: roxo técnico
Serial open: âmbar suave
```

---

## 6. Wizard: Add Target

A criação de target deve ser guiada, não manual.

### 6.1 Primeira tela

```text
What do you want to target?

[ Local Linux ]
[ Remote Linux over SSH ]
[ Bare-metal MCU ]
[ QEMU / Emulator ]
[ Custom external runner ]
```

### 6.2 Local Linux

Campos:

```text
Name
Architecture
Compiler: GCC/Clang/Custom
CMake preset
Cargo target
Debugger: GDB/LLDB
Environment variables
```

### 6.3 Remote Linux via SSH

Campos:

```text
Name
Host
Port
User
Authentication method
Remote path
Deploy method: rsync/scp/custom
Remote run command
Remote debugger
Remote environment
```

Ações no wizard:

```text
Test connection
Detect remote architecture
Detect remote tools
Create deploy profile
Create run config
```

### 6.4 Bare-metal MCU

Campos:

```text
Name
Architecture
Family
Toolchain
Flash tool
Debug probe
OpenOCD config path
Firmware artifact
Serial port
Reset strategy
```

Ações:

```text
Detect probes
Validate OpenOCD
Validate firmware path
Test flash command dry-run
Open serial after flash
```

### 6.5 QEMU

Campos:

```text
Name
Architecture
Machine
CPU
Kernel image
Rootfs image
DTB
Initrd
QEMU binary
GDB stub port
Serial mode
Network mode
Extra args
```

Ações:

```text
Validate qemu-system binary
Validate images
Generate boot command
Start paused for debugger
Open serial console
```

---

## 7. Remote SSH como fluxo de produto

Remote SSH é essencial para Linux embarcado e industrial PCs.

A Kinein deve permitir:

```text
conectar
validar ferramentas remotas
deployar artefato
executar remotamente
depurar remotamente
abrir logs remotos
sincronizar arquivos selecionados
```

### 7.1 Remote SSH não deve ser editor remoto completo no MVP

Para proteger qualidade, o MVP não precisa implementar edição remota total estilo VS Code Remote.

Recomendação pragmática:

```text
MVP:
- projeto local
- build local/cross local
- deploy para host remoto
- run remoto
- logs remotos
- debug remoto básico

Futuro:
- workspace remoto completo
- indexação remota
- LSP remoto
- file explorer remoto completo
```

### 7.2 Fluxo recomendado

```text
1. User selects Remote Linux target.
2. Kinein tests SSH connection.
3. Kinein detects remote architecture.
4. Kinein checks remote folder.
5. Build happens locally or remotely depending on profile.
6. Artifact is deployed.
7. Remote process starts.
8. Logs stream back to the IDE.
9. Optional debugger attaches.
```

### 7.3 UI do Remote Target

```text
Remote Target: RPi Lab
Status: Connected
Host: 192.168.1.40
Arch: aarch64
Deploy path: /home/vitor/kinein/app
Run command: ./motor_control

[ Test Connection ] [ Deploy ] [ Run ] [ Debug ] [ Open Logs ]
```

### 7.4 Erros comuns que devem ser explicados

```text
SSH host unreachable
Permission denied
Remote path missing
Architecture mismatch
Missing runtime library
Binary not executable
Port already in use
Debugger not installed remotely
```

Cada erro deve ter ação sugerida.

Exemplo:

```text
Binary not executable.
Action: chmod +x remote artifact
Button: Apply fix remotely
```

---

## 8. Flash para bare-metal

Flash é uma ação crítica. A IDE deve tratá-la com cuidado.

### 8.1 Fluxo de flash

```text
Build firmware
  ↓
Validate artifact
  ↓
Detect probe
  ↓
Validate flash tool
  ↓
Optional erase
  ↓
Flash
  ↓
Verify
  ↓
Optional reset
  ↓
Optional open serial
```

### 8.2 UI de flash

Toolbar:

```text
[ Build ] [ Flash ] [ Debug ] [ Serial ]
```

Painel de flash:

```text
Flash Target: STM32F4 Discovery
Artifact: build/firmware.elf
Probe: ST-Link detected
Tool: OpenOCD
Config: board/stm32f4discovery.cfg

[ Flash ] [ Flash + Reset ] [ Flash + Serial ] [ Dry Run ]
```

### 8.3 Segurança

A IDE deve evitar ações destrutivas sem clareza.

Regras:

```text
Nunca executar erase completo sem indicar.
Nunca rodar comando de flash invisível.
Sempre mostrar o comando expandido em modo avançado.
Sempre registrar logs.
Permitir dry-run quando possível.
Permitir copiar comando para terminal.
```

### 8.4 Estados de flash

```text
Idle
Building
Waiting for probe
Flashing
Verifying
Resetting
Done
Failed
Cancelled
```

### 8.5 Logs de flash

O painel deve separar:

```text
Command
Stdout
Stderr
Parsed summary
Suggested fix
```

Exemplo:

```text
OpenOCD failed: target not halted.
Suggested actions:
- Check BOOT0/RESET state.
- Try connect-under-reset.
- Verify board config.
```

---

## 9. Serial Monitor

Serial Monitor deve ser tratado como ferramenta de engenharia, não só terminal bruto.

### 9.1 Funcionalidades MVP

```text
listar portas seriais
definir baud rate
abrir/fechar conexão
mostrar logs em tempo real
enviar texto
limpar console
salvar log
pausar scroll
filtro simples
```

### 9.2 Configuração

```json
{
  "id": "stm32-uart",
  "name": "STM32 UART",
  "port": "/dev/ttyACM0",
  "baudRate": 115200,
  "dataBits": 8,
  "stopBits": 1,
  "parity": "none",
  "lineEnding": "lf",
  "autoOpenAfterFlash": true
}
```

### 9.3 UI

```text
Serial Monitor
[ Port: /dev/ttyACM0 ▼ ] [ Baud: 115200 ▼ ] [ Open ] [ Clear ] [ Save ]

[Filter logs...] [Auto-scroll ✓] [Timestamps ✓]

12:01:03.122 boot: init
12:01:03.230 motor: pwm ready
12:01:03.481 sensor: ok
```

### 9.4 Recursos futuros

```text
plot a partir de valores seriais
telemetria key=value
regex filters
binary mode
protocol decoder
CSV export
trigger alerts
```

---

## 10. QEMU e emulação

QEMU é estratégico porque permite desenvolver sem hardware físico.

### 10.1 Princípio

A Kinein deve tratar QEMU como target selecionável, não como comando solto.

```text
Target: QEMU ARMv7 Linux
Build: cross compile
Run: boot emulator
Debug: attach GDB to QEMU stub
Observe: serial console
```

### 10.2 Fluxo QEMU Linux

```text
Validate qemu binary
  ↓
Validate kernel/rootfs/dtb
  ↓
Generate command
  ↓
Start QEMU
  ↓
Open serial console
  ↓
Optional attach debugger
  ↓
Stop/restart instance
```

### 10.3 UI do QEMU Target

```text
QEMU Target: ARMv7 Linux
Machine: virt
CPU: cortex-a15
Kernel: images/zImage
Rootfs: images/rootfs.ext4
DTB: images/virt.dtb
GDB Stub: :1234
Serial: IDE Console

[ Boot ] [ Boot Paused ] [ Attach Debugger ] [ Stop ] [ Open Console ]
```

### 10.4 QEMU Bare-metal futuro

Para bare-metal:

```text
firmware.elf
machine
cpu
gdb stub
semihosting
serial console
```

O MVP pode apenas preparar a arquitetura; não precisa suportar todos os modos.

### 10.5 Erros comuns

```text
QEMU binary missing
Unsupported machine
Kernel image missing
Rootfs missing
Port already in use
GDB failed to attach
QEMU exited immediately
```

A IDE deve explicar e sugerir correção.

---

## 11. Sysroot e SDK

Projetos de Linux embarcado frequentemente dependem de `sysroot` e SDK.

A Kinein deve ter um painel:

```text
SDKs & Sysroots
```

### 11.1 Modelo

```json
{
  "id": "arm64-yocto-sdk",
  "name": "Yocto ARM64 SDK",
  "kind": "sysroot_sdk",
  "path": "/opt/poky/4.0/sysroots/aarch64-poky-linux",
  "environmentScript": "/opt/poky/environment-setup-aarch64-poky-linux",
  "compilerPrefix": "aarch64-poky-linux-",
  "cmakeToolchainFile": "toolchains/aarch64-yocto.cmake",
  "valid": true
}
```

### 11.2 UX

O usuário deve poder:

```text
Add SDK
Validate SDK
Run environment script probe
Detect compiler
Detect sysroot path
Generate CMake toolchain file suggestion
Associate SDK with target
```

### 11.3 Integração com CMake

Ao selecionar um SDK, a IDE pode sugerir:

```text
-DCMAKE_TOOLCHAIN_FILE=toolchains/aarch64-yocto.cmake
-DCMAKE_SYSROOT=/opt/poky/...
-DCMAKE_C_COMPILER=aarch64-poky-linux-gcc
-DCMAKE_CXX_COMPILER=aarch64-poky-linux-g++
```

Mas deve manter tudo editável e visível.

---

## 12. Debug embarcado

Debug deve ser separado por categoria.

### 12.1 Debug local

```text
binary local
gdb/lldb local
breakpoints
variables
call stack
threads
```

### 12.2 Debug remoto Linux

```text
local gdb + gdbserver remoto
ou gdb remoto via SSH
source path mapping
remote executable
remote args
remote environment
```

### 12.3 Debug bare-metal

```text
OpenOCD/probe starts server
GDB connects
firmware symbols loaded
reset/halt
breakpoints
continue/step
```

### 12.4 Debug QEMU

```text
QEMU starts with gdb stub
GDB connects to localhost:port
symbols from ELF
serial console visible
```

### 12.5 UI de debug

```text
Debug
├── Sessions
├── Call Stack
├── Variables
├── Registers futuro
├── Memory View futuro
├── Breakpoints
└── Console
```

Para MVP:

```text
breakpoints
continue
pause
step over
step into
step out
stop
call stack
variables básicas
console
```

Futuro:

```text
register view
memory view
disassembly
RTOS threads
peripheral view
trace
```

---

## 13. Jobs assíncronos

Todas as ações de target devem ser jobs.

```text
Detect Tools
Test SSH
Configure CMake
Build
Deploy
Flash
Start QEMU
Open Serial
Run Remote
Attach Debugger
```

### 13.1 Modelo de job

```json
{
  "jobId": "job-2026-001",
  "kind": "flash",
  "targetId": "stm32f4-debug",
  "state": "running",
  "progress": 0.42,
  "title": "Flashing STM32F4",
  "startedAt": "2026-07-04T18:22:10Z",
  "canCancel": true
}
```

### 13.2 Logs estruturados

Todo job deve emitir eventos:

```text
job.started
job.output
job.warning
job.error
job.progress
job.completed
job.failed
job.cancelled
```

### 13.3 Cancelamento

Cancelar deve tentar encerrar o processo externo com segurança.

Regras:

```text
não deixar QEMU órfão
não deixar SSH preso
não interromper flash sem avisar quando perigoso
não matar processo sem log
```

---

## 14. Assistente para embarcados

Assistente deve ajudar especialmente em erros difíceis de ambiente.

### 14.1 O que ele deve explicar

```text
erro de compilador cruzado
erro de sysroot
erro de linker
erro de QEMU
erro de SSH
erro de flash
erro de OpenOCD
erro de porta serial
erro de permissão Linux
```

### 14.2 Formato de resposta recomendado

```text
Problema detectado
Causa provável
Como verificar
Correção sugerida
Comando equivalente
Arquivos envolvidos
```

Exemplo:

```text
Problema: compilador aarch64 não encontrado.
Causa provável: SDK não carregado ou PATH incompleto.
Como verificar: which aarch64-linux-gnu-g++
Correção: selecione um SDK ou configure o path do compilador.
Ação: Open Toolchain Settings
```

### 14.3 Ações rápidas

```text
Open Target Settings
Open Toolchain Settings
Re-run Detection
Copy Command
Explain Error
Create Issue Note
Open Docs
```

---

## 15. Persistência no workspace

A Kinein deve persistir configurações em `.kinein/`.

Estrutura recomendada:

```text
.kinein/
├── workspace.json
├── targets.json
├── toolchains.json
├── sdks.json
├── run-configs.json
├── debug-configs.json
├── serial.json
├── qemu.json
└── logs/
```

### 15.1 targets.json

```json
{
  "version": 1,
  "activeTargetId": "local-x86_64-debug",
  "targets": []
}
```

### 15.2 serial.json

```json
{
  "version": 1,
  "profiles": []
}
```

### 15.3 qemu.json

```json
{
  "version": 1,
  "profiles": []
}
```

---

## 16. JSON-RPC sugerido

A UI Qt/QML deve falar com o core por comandos claros.

### 16.1 Targets

```text
target.list
target.create
target.update
target.delete
target.select
target.status
target.validate
```

### 16.2 Remote SSH

```text
remote.testConnection
remote.detectInfo
remote.deploy
remote.run
remote.stop
remote.logs
```

### 16.3 Flash

```text
flash.detectProbes
flash.validate
flash.run
flash.cancel
flash.status
```

### 16.4 Serial

```text
serial.listPorts
serial.open
serial.close
serial.write
serial.clear
serial.saveLog
serial.status
```

### 16.5 QEMU

```text
qemu.validate
qemu.start
qemu.startPaused
qemu.stop
qemu.attachDebugger
qemu.status
qemu.console
```

### 16.6 SDK/Sysroot

```text
sdk.list
sdk.add
sdk.validate
sdk.detect
sdk.remove
sdk.associateTarget
```

---

## 17. Roadmap de implementação

### 17.1 MVP 6.1 — Target Model

Implementar:

```text
Target data model
target.list
target.create
target.select
target.status
UI básica de Targets
persistência em .kinein/targets.json
```

Critério de aceite:

```text
Usuário consegue criar e selecionar Local Linux target.
Toolbar mostra target ativo.
Estado persiste ao fechar/reabrir.
```

### 17.2 MVP 6.2 — Serial Monitor

Implementar:

```text
serial.listPorts
serial.open
serial.close
serial.write
serial logs
UI Serial básica
```

Critério de aceite:

```text
Usuário consegue abrir /dev/ttyUSB0 ou /dev/ttyACM0, ver logs e fechar sem travar a IDE.
```

### 17.3 MVP 6.3 — Remote SSH Deploy/Run

Implementar:

```text
remote.testConnection
remote.detectInfo
remote.deploy
remote.run
remote logs
```

Critério de aceite:

```text
Usuário consegue compilar localmente, copiar artefato para host remoto e executar com logs na IDE.
```

### 17.4 MVP 6.4 — QEMU Target Básico

Implementar:

```text
qemu.validate
qemu.start
qemu.stop
console serial
```

Critério de aceite:

```text
Usuário consegue cadastrar comando QEMU controlado pela IDE e ver console/logs.
```

### 17.5 MVP 6.5 — Flash Básico

Implementar:

```text
flash profile
flash.run via comando configurável
logs estruturados
dry-run
```

Critério de aceite:

```text
Usuário consegue executar comando de flash configurado, ver logs, copiar comando e diagnosticar falhas.
```

---

## 18. O que não implementar agora

Para proteger qualidade, não implementar na primeira versão:

```text
marketplace de placas
suporte automático a todo MCU existente
peripheral viewer completo
trace SWO/ETM
RTOS awareness profundo
edição remota total
Yocto integration profunda
Buildroot wizard completo
simulação OpenGL integrada pesada
análise elétrica/circuital
```

A Kinein deve começar sólida, não ampla demais.

---

## 19. Checklist para IA CLI

Ao implementar esta parte, a IA CLI deve seguir esta ordem:

1. Criar modelos de dados para Target.
2. Criar persistência `.kinein/targets.json`.
3. Criar serviço `TargetService`.
4. Criar comandos JSON-RPC de target.
5. Exibir target ativo na toolbar.
6. Criar Tool Window Targets.
7. Implementar Serial Monitor básico.
8. Implementar Remote SSH test/deploy/run básico.
9. Implementar QEMU profile básico.
10. Implementar Flash profile configurável.
11. Integrar logs com Jobs.
12. Integrar erros com Assistente.
13. Adicionar testes unitários para modelos e validação.
14. Adicionar testes de integração para comandos simulados.

---

## 20. Critérios gerais de aceite

Esta Parte 6 estará bem implementada quando:

```text
Target é entidade visível e persistida.
Build/Run/Debug respeitam o target ativo.
Serial Monitor funciona sem travar UI.
Remote SSH consegue testar conexão e executar comando remoto.
QEMU pode ser iniciado/parado como job.
Flash pode rodar como comando configurável com logs.
Erros têm mensagens compreensíveis e ações sugeridas.
Usuário consegue copiar comandos equivalentes.
A UI nunca esconde comandos críticos.
A IDE continua confortável e previsível.
```

---

## 21. Frase-guia final

```text
A Kinein não deve tentar esconder a complexidade dos sistemas embarcados.
Ela deve organizar essa complexidade em fluxos visuais seguros, rastreáveis e fáceis de repetir.
```
