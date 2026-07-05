# Kinein Vectis — Parte 10: Fechamento Geral, Performance, MVP, Roadmap e Checklist de Polimento

> **Tipo:** etapa final de consolidação.  
> **Objetivo:** fechar a visão macro do projeto antes de entrar em polimento fino.  
> **Decisão central:** a Kinein deve ser uma IDE determinística, confortável, local-first e não invasiva para C/C++/Rust, com setup visual, Configuration Actions e fluxo expert opcional.

---

## 1. Resposta direta: LSP/clangd deixaria a IDE pesada?

Pode deixar pesada **se for mal gerenciado**.  
Mas não precisa deixar a Kinein pesada se a arquitetura tratar LSP, indexação e análise como serviços controlados, assíncronos e visíveis.

A regra correta é:

```text
clangd/rust-analyzer não devem ser “o coração pesado da IDE”.
Eles devem ser serviços auxiliares gerenciados pelo Core.
```

A Kinein não deve depender de clangd para tudo.

---

## 2. O que realmente depende de clangd/rust-analyzer

### 2.1 Depende fortemente de LSP

```text
go to definition
go to references
rename seguro
diagnósticos semânticos
hover com tipos
autocomplete semântico
signature help
code actions da linguagem
```

### 2.2 Não depende obrigatoriamente de LSP

```text
Configuration Actions
Project Wizard
Toolchain detection
CMakePresets generation
Cargo.toml editing
Environment Fingerprint
Project Health básico
Build/Run/Debug configs
AI CLI Bridge
Project Settings
Target Manager
```

Isso é importante: **Configuration Actions não devem depender de clangd** para existir.

Elas podem usar:

```text
Project Fingerprint
Build System Model
CMake scanner simples
Cargo TOML parser
Toolchain Health Check
Configuration Graph
```

O LSP ajuda, mas não sustenta tudo.

---

## 3. Tree-sitter, LSP e Configuration Actions

### 3.1 Tree-sitter

Uso ideal:

```text
syntax highlighting
folding
outline local
seleção por escopo
estrutura rápida
fallback antes do LSP
```

Peso:

```text
baixo a moderado
```

### 3.2 clangd/rust-analyzer

Uso ideal:

```text
semântica profunda
diagnósticos reais
rename
references
tipos
autocomplete
```

Peso:

```text
moderado a alto, dependendo do projeto
```

### 3.3 Configuration Actions

Uso ideal:

```text
editar CMake/Cargo com preview
adicionar dependência
adicionar target
ativar compile_commands
criar presets
criar run/debug config
```

Peso:

```text
baixo a moderado
```

Conclusão:

```text
A Kinein pode ser rápida porque a maioria do setup visual não precisa esperar LSP.
```

---

## 4. Estratégia para não pesar a IDE

### 4.1 Lazy start

Não iniciar tudo ao abrir a IDE.

```text
abrir workspace
  ↓
detectar projeto
  ↓
iniciar Tree-sitter rápido
  ↓
iniciar LSP só quando necessário
```

### 4.2 LSP sob demanda

```text
clangd só inicia para projeto C/C++ ativo
rust-analyzer só inicia para projeto Cargo ativo
em projeto CMake-only não iniciar rust-analyzer
em projeto Cargo-only não iniciar clangd
em projeto Mixed iniciar ambos, mas com controle
```

### 4.3 Indexação controlada

```text
não indexar diretórios build enormes sem necessidade
ignorar target/
ignorar .git/
ignorar node_modules/
ignorar logs/
ignorar cache/
respeitar .gitignore
```

### 4.4 Eventos agregados

Não atualizar UI para cada microevento.

```text
agregar diagnostics
debounce file watcher
debounce project scan
streamar logs grandes
```

### 4.5 Project size modes

A IDE pode detectar projeto grande:

```text
Small
Medium
Large
Huge
```

E ajustar comportamento:

```text
Huge project:
- LSP inicia mais tarde;
- menos sugestões automáticas;
- file tree lazy;
- Project Health compacto;
- indexação incremental.
```

---

## 5. Performance budget inicial

### 5.1 Abertura da IDE sem projeto

Meta:

```text
< 2 segundos perceptivos em máquina razoável
```

### 5.2 Abrir projeto pequeno

Meta:

```text
< 5 segundos até editor utilizável
```

Mesmo que LSP continue indexando.

### 5.3 Abrir projeto médio

Meta:

```text
editor utilizável rápido;
Project Health em background;
LSP com status visível.
```

### 5.4 Build

Build não deve travar UI.

```text
sempre job assíncrono
logs streamados
cancelamento quando possível
```

### 5.5 Configuration Actions

Meta:

```text
abrir lista instantaneamente usando registry local;
validar disponibilidade em background;
preview de diff rápido para ações simples.
```

---

## 6. UX para serviços pesados

A Kinein deve mostrar status, sem assustar.

Exemplo:

```text
clangd indexing...
rust-analyzer loading workspace...
CMake configure running...
Project Health updating...
```

Mas sem modal invasivo.

Local ideal:

```text
Status Bar
Project Health compact
Language Service badge
Background Jobs menu
```

---

## 7. Regra de ouro de performance

```text
A IDE deve ficar utilizável antes de ficar 100% analisada.
```

O usuário deve poder abrir arquivo, ler código e usar terminal mesmo se clangd ainda estiver trabalhando.

---

## 8. Pontos para polimento futuro

Antes de polir visualmente, precisamos saber quais pontos vão exigir revisão depois.

### 8.1 Polimento de UX

```text
densidade das listas;
hierarquia visual;
animações discretas;
transições de painel;
estados vazios;
onboarding curto;
botões principais/secundários;
tooltips;
atalhos;
Command Palette.
```

### 8.2 Polimento de texto

```text
nomes das ações;
descrições curtas;
mensagens de erro;
mensagens de Project Health;
explicações de CMake/Cargo;
rótulos de risco;
nomes de modos;
documentação curta.
```

### 8.3 Polimento técnico

```text
limites do parser CMake;
validação de Cargo.toml;
diff pequeno;
rollback;
logs sanitizados;
jobs canceláveis;
eventos bem nomeados;
protocolos JSON-RPC estáveis.
```

### 8.4 Polimento de performance

```text
lazy loading;
debounce;
cache;
file watcher;
limites de indexação;
status de LSP;
grandes projetos;
terminal PTY.
```

### 8.5 Polimento de produto

```text
MVP claro;
escopo reduzido;
sem plugin system cedo;
sem IA embutida;
sem simulador cedo;
sem embedded avançado cedo;
foco em CMake/Cargo/Toolchain/Editor.
```

---

## 9. Última etapa necessária antes do polimento

Sim: esta Parte 10 serve como última etapa macro.

Depois dela, o projeto já tem uma visão completa:

```text
visual
UX
componentes
build/run/debug
editor/LSP/Tree-sitter
embedded/targets
AI CLI Bridge externo
onboarding
setup intelligence
arquitetura interna
Configuration Actions
escopo por build system
performance
MVP
roadmap
```

A partir daqui, o ideal não é inventar novas grandes partes.  
O ideal é **revisar, cortar, organizar e transformar em plano implementável**.

---

## 10. Ordem final recomendada dos documentos

```text
01 — Visual System / Iconografia
02 — Layout principal
03 — UI Components
04 — Build, Run, Debug
05 — Editor, LSP, Tree-sitter
06 — Embedded Targets, Flash, Serial, QEMU
07.1 — AI CLI Bridge externo
08 — Onboarding, Project Wizard, Settings
08.1 — Setup Intelligence
09 — Arquitetura Interna
09.1 — Dual Workflow e Configuration Actions
09.2 — Scoped Configuration Actions e Docs
10 — Fechamento, Performance, MVP e Polimento
```

A Parte 7 original deve ser tratada como substituída pela Parte 7.1 ou revisada com essa nova decisão.

---

## 11. MVP final recomendado

O MVP não deve tentar entregar tudo.

### 11.1 MVP essencial

```text
Qt/QML UI básica
Rust Core via JSON-RPC stdio
Workspace open
Project detection CMake/Cargo
Toolchain scan local
CMake configure/build
Cargo metadata/check/build
Editor básico
Tree-sitter highlighting
clangd/rust-analyzer status
Diagnostics básicos
Terminal comum
AI Terminal externo simples
Project Wizard C++ CMake
Project Wizard Rust Cargo
Settings básicas
Configuration Actions MVP
Project Health básico
Job System
Event System
Storage básico
```

### 11.2 Não-MVP

```text
plugin system público
marketplace
IA embutida
simulador OpenGL
QEMU completo
remote development completo
debugger avançado completo
embedded flash completo
CMake parser universal
refatorações próprias profundas
instalação automática de toolchains
```

---

## 12. Primeiras Configuration Actions do MVP

### 12.1 CMake

```text
Enable compile_commands.json
Create Debug preset
Create Release preset
Add executable
Add static library
Add source file to target
Add include directory
Add target_link_libraries
Inspect CMake cache
Repair stale build directory
```

### 12.2 Cargo

```text
Add dependency
Add dev-dependency
Add feature
Set edition
Run cargo check
Create run config
```

Essas ações já validam a proposta do produto sem expandir demais.

---

## 13. Primeiro fluxo de usuário ideal

### 13.1 Usuário iniciante/intermediário

```text
Abre Kinein
Escolhe Guided ou Balanced
New C++ Project
Escolhe Clang/GCC detectado
Kinein gera CMakeLists/CMakePresets
Kinein roda configure
Editor abre main.cpp
Usuário usa Configuration Actions para adicionar biblioteca
Preview diff
Apply
Build
```

### 13.2 Usuário avançado

```text
Abre Kinein
Escolhe Expert
Open Workspace
Kinein detecta CMakePresets existentes
Não altera nada
Mostra status discreto
Usuário usa terminal/editor normalmente
Se quiser, chama Configuration Actions via Command Palette
```

---

## 14. Filosofia final do Kinein

```text
Kinein não é uma IDE que força um caminho.
É uma IDE que torna caminhos técnicos visíveis.
```

Ela deve:

```text
mostrar opções reais;
mostrar estado real;
mostrar comandos reais;
mostrar arquivos reais;
mostrar riscos reais;
permitir preview;
permitir cancelar;
permitir modo expert;
permitir terminal;
permitir edição manual;
facilitar sem invadir.
```

---

## 15. Decisões congeladas até o polimento

Para evitar voltar em círculos, congelar estas decisões:

```text
Nome: Kinein Vectis
Sigla: KV
Stack visual: Qt/QML
Core: Rust
IPC: JSON-RPC local
IA: externa via AI CLI Bridge
Sem IA embutida
Configuration Actions como camada visual
Guided/Balanced/Expert Modes
CMake e Cargo filtrados por Build System ativo
Preview obrigatório para alteração de arquivo
Project Health determinístico
Toolchain/Build/Target como entidades visuais
```

---

## 16. Próximo passo depois desta etapa

Depois desta Parte 10, o próximo passo recomendado é:

```text
Criar um SPEC_INDEX.md consolidando todos os documentos,
corrigir contradições,
marcar prioridades,
e separar MVP / Pós-MVP / Futuro.
```

Depois:

```text
criar README do projeto
criar árvore de pastas inicial
criar issues/tarefas
criar primeiro milestone
começar implementação do core mínimo
```

---

## 17. Critérios de aceite para fechar a fase de definição

```text
[ ] Existe visão visual.
[ ] Existe visão de UX.
[ ] Existe arquitetura interna.
[ ] Existe fluxo build/run/debug.
[ ] Existe decisão sobre IA externa.
[ ] Existe onboarding/setup.
[ ] Existe Configuration Actions.
[ ] Existe estratégia de performance.
[ ] Existe MVP reduzido.
[ ] Existe lista clara de não-MVP.
[ ] Existe roadmap de implementação.
```

Se tudo isso está documentado, a fase de definição macro está completa.

---

## 18. Resumo executivo

Sim, clangd/rust-analyzer podem pesar.  
Mas a Kinein não precisa ser pesada se:

```text
LSP for lazy;
Tree-sitter for fast local structure;
Configuration Actions for deterministic config;
Jobs for long operations;
Events for UI updates;
Project Health for state;
Expert Mode for low-noise workflow.
```

A fase macro está praticamente fechada.

A próxima fase não deve ser criar mais ideias grandes.  
Deve ser:

```text
consolidar
corrigir
priorizar
cortar excesso
organizar MVP
e depois implementar.
```

Frase final:

```text
Kinein Vectis deve ser leve na presença,
forte na estrutura
e cuidadosa na forma como ajuda.
```
