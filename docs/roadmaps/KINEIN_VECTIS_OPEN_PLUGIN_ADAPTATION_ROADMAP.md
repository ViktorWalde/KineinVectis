# Kinein Vectis — Roadmap de Adaptação de Plugins e Ferramentas Open Source

> **Data da pesquisa:** 11 de julho de 2026; política ampliada em 15 de julho de 2026
> **Objetivo:** construir o melhor ambiente possível para C, C++ e Rust, com conforto de IDE profissional, zero dependência de plugins proprietários do Visual Studio Code e sem usar Neovim/Vim como runtime.

---

# 1. Decisão arquitetural oficial

O **Kinein Vectis não executará plugins do Neovim nem extensões do VS Code/VSCodium diretamente**.

Plugins Neovim normalmente dependem de:

```text
vim.api
vim.lsp
vim.diagnostic
buffers/windows do Neovim
runtime Lua do Neovim
```

Extensões VSCodium/Code OSS normalmente dependem de:

```text
vscode.*
Extension Host
contribution points
commands API
workspace API
debug API
terminal API
webviews
storage e secrets APIs
```

Portanto, a estratégia correta é:

```text
1. Integrar diretamente protocolos e executáveis abertos.
2. Estudar plugins maduros como referência funcional e de UX.
3. Portar apenas módulos isolados e licenciados de forma compatível.
4. Não criar compatibilidade genérica com .vsix ou plugins Lua no início.
5. Não copiar comportamento proprietário ou dependências fechadas.
```

A arquitetura alvo é:

```text
Kinein Vectis
├── Qt/QML Native UI
├── Rust Core
├── LSP Client
├── DAP Client
├── Task Manager
├── Project Model
├── Toolchain Manager
├── Remote/Embedded Manager
└── Ferramentas locais e remotas abertas
```

---

# 2. Modos de reaproveitamento

## MODE-A — Integração direta

O Kinein executa ou se comunica com a ferramenta original.

Exemplos:

```text
clangd via LSP
rust-analyzer via LSP
CodeLLDB/lldb-dap via DAP
CMake via File API e Presets
Cargo via cargo metadata
ripgrep via CLI/JSON
fd via CLI
OpenSSH via CLI
probe-rs via CLI/API
```

Este é o modo preferido.

## MODE-B — Adaptação de comportamento e UX

O plugin serve como referência para reproduzir uma experiência equivalente em Qt/QML e Rust.

Exemplos:

```text
Telescope → Search Everywhere
Neo-tree → Project Explorer
Trouble → Problems/Symbols/Usages
Error Lens → diagnostics inline
Gitsigns → Git gutter
Which-key → descoberta de atalhos
Overseer → Task Manager
```

## MODE-C — Portabilidade seletiva

Portar código somente quando:

```text
a licença for compatível;
o módulo estiver bem isolado;
não depender profundamente do host original;
houver testes;
a manutenção do fork fizer sentido;
os avisos de copyright forem preservados.
```

## MODE-D — Referência apenas

Usar somente como estudo quando:

```text
a licença for copyleft forte;
o projeto depender profundamente do Neovim/VS Code;
houver telemetria;
o projeto estiver pouco maduro;
o código tiver dependências obscuras;
a adaptação custar mais do que reimplementar.
```

---

## 2.1 Referências profissionais obrigatórias para funcionalidades de IDE

Ao criar ou alterar uma funcionalidade de IDE, é obrigatório estudar código
oficial, atual e funcional de pelo menos uma referência pertinente abaixo. Em
decisões arquiteturais ou de segurança, comparar duas quando isso trouxer uma
segunda solução realmente relevante. O objetivo é aprender invariantes,
fronteiras, tratamento de erros, cancelamento, backpressure, concorrência,
segurança e estratégia de testes — não transportar a implementação.

| Referência oficial | Uso principal como referência | Modo padrão |
| --- | --- | --- |
| [Code OSS (`microsoft/vscode`)](https://github.com/microsoft/vscode) | terminal/PTY host, shell integration, ciclo de sessões, workbench assíncrono e editor | MODE-B |
| [IntelliJ IDEA Community](https://github.com/JetBrains/intellij-community) | project model, actions/run configurations, tool windows, indexação, cancelamento, editor e gutter | MODE-B |
| [Zed](https://github.com/zed-industries/zed) | arquitetura Rust, concorrência, responsividade, workspace, editor e terminal | MODE-D por padrão; auditar cada componente |
| [Lapce](https://github.com/lapce/lapce) | fluxos de comando/estado/eventos e arquitetura de editor em Rust | MODE-B |
| [Apache NetBeans](https://github.com/apache/netbeans) | sistema de projetos, ações, modularidade, tarefas longas e tool windows | MODE-B |

Neste documento, “VS Code como referência” significa exclusivamente o código
público **Code OSS** no repositório oficial. A distribuição Visual Studio Code,
o Marketplace, extensões proprietárias, serviços fechados e customizações não
presentes no repositório público ficam fora do escopo. A distinção oficial está
documentada em [Differences between the repository and Visual Studio
Code](https://github.com/microsoft/vscode/wiki/Differences-between-the-repository-and-Visual-Studio-Code).

Licenças e limites precisam ser revalidados na revisão consultada; a tabela não
é autorização permanente de reutilização. Em especial, código copyleft é
MODE-D por padrão. Mesmo quando a licença for permissiva, a inclusão desta IDE
na lista autoriza **estudo**, não cópia. MODE-C continua reservado a componente
isolado deliberadamente adotado após auditoria, registro e ADR; não serve para
copiar uma função pronta da IDE de referência.

### Procedimento obrigatório

```text
1. Definir o problema, os invariantes e os modos de falha da fatia atual.
2. Localizar no repositório oficial uma implementação mantida, seus testes e a
   revisão/tag examinada; não escolher código legado só por conveniência.
3. Registrar no documento do domínio: URL/revisão, arquivos ou subsistema
   estudado, licença/modo, lições e diferenças arquiteturais da Kinein.
4. Extrair comportamento, contratos, estratégias de erro/cancelamento,
   segurança, concorrência e testes — nunca texto de implementação.
5. Projetar a solução nativa no fluxo Qt/QML → CoreClient → protocolo tipado →
   Rust Core → serviço/job/ferramenta externa.
6. Escrever código novo e testes próprios, coerentes com strict mode e specs.
7. Comparar o comportamento e os modos de falha com a referência estudada.
```

É proibido:

```text
copiar função, classe, módulo ou teste pronto;
traduzir mecanicamente TypeScript/Kotlin/Java/Rust para outra linguagem;
transplantar Electron/Node, Extension Host, Swing/IntelliJ Platform, GPUI,
Floem ou outro runtime/arquitetura para dentro da Kinein;
usar revisão antiga, abandonada ou vulnerável quando existe caminho atual;
remover proveniência/licença de um componente que tenha sido adotado via MODE-C;
tratar semelhança de UX como autorização para depender do host original.
```

### Exemplo canônico: terminal e Assistente

Para terminal, estudar no Code OSS o backend/PTY host, o ciclo de vida da
sessão, shell integration, resize, reconexão, backpressure e a separação entre
entrada, processo e renderização. A Kinein preserva sua própria arquitetura:
`portable-pty` + parser VT no Rust Core, eventos IPC tipados e renderização
Qt/QML pelo `TerminalPanel`/`TerminalManager`. Não incorporar Node, xterm.js ou
código do Code OSS. A correção que tornou a grade VT e o cursor autoritativos
no Assistente é o modelo: mesma qualidade funcional, desenho próprio e nenhuma
cópia de implementação.

---

# 3. Gate obrigatório de auditoria

Antes de instalar, clonar, portar ou adaptar qualquer item, verificar:

```text
[ ] repositório oficial;
[ ] licença SPDX;
[ ] LICENSE presente;
[ ] autoria e copyright;
[ ] releases/tags;
[ ] último commit;
[ ] changelog;
[ ] security policy;
[ ] security advisories;
[ ] issues críticas;
[ ] telemetria;
[ ] chamadas de rede;
[ ] execução de shell;
[ ] acesso a segredos;
[ ] dependências transitivas;
[ ] compatibilidade Linux;
[ ] testes automatizados;
[ ] possibilidade de pin por commit/tag;
[ ] manutenção necessária após adaptação;
```

Preferência de licença:

```text
1. MIT
2. Apache-2.0
3. MIT OR Apache-2.0
4. BSD-2-Clause / BSD-3-Clause
5. MPL-2.0 após revisão
```

GPL/AGPL:

```text
usar como referência;
não copiar para um Core MIT/Apache sem análise jurídica;
considerar processo separado quando tecnicamente apropriado;
documentar qualquer interação.
```

---

# 4. Ordem geral de implementação

```text
P0 — fundação obrigatória
P1 — experiência profissional principal
P2 — conforto e produtividade
P3 — embedded/remote avançado
P4 — opcionais e experimentais
```

---

# PARTE I — P0: FUNDAÇÃO OBRIGATÓRIA

## 5. clangd + vscode-clangd

**Repositórios:**

```text
https://github.com/clangd/vscode-clangd
https://github.com/llvm/llvm-project/tree/main/clang-tools-extra/clangd
https://clangd.llvm.org/
```

**Licença:** MIT para vscode-clangd; LLVM Project para clangd.  
**Modo:** MODE-A para clangd; MODE-B para a extensão.  
**Prioridade:** P0.

### O que entrega

```text
completion;
diagnostics;
go to definition;
find references;
hover;
rename;
code actions;
inlay hints;
semantic tokens;
include management;
formatting;
refatorações simples.
```

### Implementação no Kinein

```text
Kinein LSP Client
↓
clangd
↓
compile_commands.json
↓
CMake/Toolchain Model
```

### Regra

```text
Não usar Microsoft C/C++ cpptools como dependência.
Usar clangd como motor principal de inteligência C/C++.
```

---

## 6. rust-analyzer

**Repositórios:**

```text
https://github.com/rust-lang/rust-analyzer
https://rust-analyzer.github.io/manual.html
```

**Licença:** MIT OR Apache-2.0.  
**Modo:** MODE-A.  
**Prioridade:** P0.

### O que entrega

```text
completion;
diagnostics;
go to definition;
find references;
rename;
code actions;
inlay hints;
trait/impl navigation;
Cargo integration;
semantic tokens;
refatorações Rust.
```

### Implementação

```text
Kinein LSP Client
↓
rust-analyzer
↓
cargo metadata
↓
Cargo Workspace Model
```

---

## 7. Tree-sitter

**Repositórios:**

```text
https://github.com/tree-sitter/tree-sitter
https://github.com/nvim-treesitter/nvim-treesitter
```

**Modo:** MODE-A para Tree-sitter Core; MODE-B/MODE-C para queries.  
**Prioridade:** P0.

### O que entrega

```text
parsing incremental;
syntax highlighting estrutural;
outline;
folding;
seleção estrutural;
text objects futuros;
injeção de linguagens;
base para refatorações locais;
navegação por AST.
```

### Regras

```text
usar Tree-sitter diretamente;
manter registry próprio de parsers;
fixar versão/commit;
auditar a licença de cada parser;
não depender do runtime nvim-treesitter.
```

---

## 8. CMake Tools como referência + CMake direto

**Repositórios:**

```text
https://github.com/microsoft/vscode-cmake-tools
https://cmake.org/cmake/help/latest/manual/cmake-file-api.7.html
https://cmake.org/cmake/help/latest/manual/cmake-presets.7.html
```

**Licença:** MIT.  
**Modo:** MODE-B para a extensão; MODE-A para CMake.  
**Prioridade:** P0.

### O que reproduzir

```text
CMake Presets;
configure/build;
seleção de toolchain;
targets;
CTest;
project outline;
status de configuração;
run configurations;
copy/locate compile_commands.json.
```

### Advertência de privacidade

A extensão CMake Tools contém suporte de telemetria.

Para o Kinein:

```text
não portar componentes de telemetria;
não portar Application Insights;
não enviar usage data;
integrar CMake diretamente.
```

### Implementação

```text
CMake File API
CMakePresets.json
CTest
Ninja
compile_commands.json
```

---

## 9. Cargo e cargo metadata

**Origem:**

```text
https://doc.rust-lang.org/cargo/
https://doc.rust-lang.org/cargo/commands/cargo-metadata.html
```

**Modo:** MODE-A.  
**Prioridade:** P0.

### O que entrega

```text
workspace;
crates;
targets;
features;
dependencies;
build/test/check;
metadata estruturada;
run configurations.
```

### Regra

```text
Não interpretar Cargo.toml apenas com regex.
Usar cargo metadata como fonte principal do Project Model.
```

---

## 10. CodeLLDB / lldb-dap

**Repositórios:**

```text
https://github.com/vadimcn/codelldb
https://lldb.llvm.org/
```

**Licença:** MIT para CodeLLDB.  
**Modo:** MODE-A para adapter/DAP; MODE-B para UI.  
**Prioridade:** P0.

### O que entrega

```text
debug C++ e Rust;
breakpoints;
conditional breakpoints;
logpoints;
watchpoints;
threads;
stack frames;
variables;
memory;
disassembly;
remote debugging;
visualizadores de STL e Rust.
```

### Implementação

```text
Kinein DAP Client
↓
CodeLLDB/lldb-dap
↓
LLDB
```

### Alternativa

```text
GDB/MI para GDB;
DAP quando adapter maduro estiver disponível.
```

---

## 11. EditorConfig

**Repositórios:**

```text
https://github.com/editorconfig/editorconfig-vscode
https://editorconfig.org/
```

**Licença:** MIT para extensão; usar core oficial ou parser compatível.  
**Modo:** MODE-A/MODE-B.  
**Prioridade:** P0.

### O que entrega

```text
indent_style;
indent_size;
tab_width;
end_of_line;
charset;
trim_trailing_whitespace;
insert_final_newline.
```

### Regra

```text
Aplicar .editorconfig antes das preferências globais,
respeitando escopo e root=true.
```

---

## 12. ripgrep

**Repositório:**

```text
https://github.com/BurntSushi/ripgrep
```

**Licença:** MIT OR Unlicense.  
**Modo:** MODE-A.  
**Prioridade:** P0.

### Uso no Kinein

```text
Search in Files;
regex;
streaming de resultados;
respeito a .gitignore;
TODO search;
replace preview futuro.
```

---

## 13. fd

**Repositório:**

```text
https://github.com/sharkdp/fd
```

**Licença:** MIT OR Apache-2.0.  
**Modo:** MODE-A.  
**Prioridade:** P0.

### Uso

```text
Quick Open;
file discovery;
filtros;
respeito a ignores;
Project Explorer bootstrap.
```

---

## 14. Git CLI + Gitsigns como referência

**Repositórios:**

```text
https://github.com/lewis6991/gitsigns.nvim
https://git-scm.com/
```

**Licença:** MIT para Gitsigns.  
**Modo:** MODE-A para Git; MODE-B para Gitsigns.  
**Prioridade:** P0/P1.

### O que reproduzir

```text
added/changed/deleted na gutter;
staged/unstaged;
preview hunk;
stage/unstage hunk;
reset hunk;
next/previous hunk;
blame da linha;
diff inline.
```

### Regra

```text
Git CLI inicialmente;
libgit2/git2-rs somente quando houver benefício claro.
```

---

# PARTE II — P1: NÚCLEO DE EXPERIÊNCIA PROFISSIONAL

## 15. Overseer.nvim → Kinein Task Manager

**Repositório:**

```text
https://github.com/stevearc/overseer.nvim
```

**Licença:** MIT.  
**Modo:** MODE-B/MODE-C seletivo.  
**Prioridade:** P1 alta.

### O que reproduzir

```text
task registry;
background jobs;
cancelamento;
restart;
histórico;
multi-stage workflows;
parsers de output;
task list;
.vscode/tasks.json como import opcional;
Cargo/Make/CMake tasks;
preLaunchTask.
```

### Resultado esperado

```text
Command
↓
Task Manager
↓
Process Runner
↓
Output Parser
↓
Diagnostics
↓
UI
```

---

## 16. Mason.nvim → Tool Registry e Installer

**Repositórios:**

```text
https://github.com/mason-org/mason.nvim
https://mason-registry.dev/
```

**Licença:** Apache-2.0.  
**Modo:** MODE-B/MODE-C seletivo.  
**Prioridade:** P1 alta.

### O que reproduzir

```text
registry de ferramentas;
LSP/DAP/formatter/linter catalog;
status installed/missing/outdated;
logs de instalação;
version pinning;
health checks;
cancelamento;
instalação em diretório controlado.
```

### Adaptação segura

```text
Kinein Tool Registry
Kinein Tool Installer
```

### Regra

```text
não instalar nada sem confirmação;
preferir package manager do sistema;
permitir ferramenta instalada pelo usuário;
não baixar binário sem checksum/proveniência.
```

---

## 17. Telescope.nvim → Search Everywhere

**Repositório:**

```text
https://github.com/nvim-telescope/telescope.nvim
```

**Licença:** MIT.  
**Modo:** MODE-B.  
**Prioridade:** P1.

### O que reproduzir

```text
fuzzy finder;
preview;
providers;
sorters;
actions;
files;
text;
symbols;
commands;
recent files;
Git;
history.
```

### Search Everywhere do Kinein

```text
Files
Symbols
Text
Actions
Settings
Targets
Tests
Recent
Quality Rules
Remote Targets
```

Backend:

```text
ripgrep
fd
LSP
Command Registry
Project Model
```

---

## 18. Neo-tree → Project Explorer

**Repositório:**

```text
https://github.com/nvim-neo-tree/neo-tree.nvim
```

**Licença:** MIT.  
**Modo:** MODE-B.  
**Prioridade:** P1.

### O que reproduzir

```text
filesystem tree;
follow active file;
file watcher;
Git status;
diagnostics;
respeito a gitignore;
rename/move/delete;
buffers/symbols como fontes alternativas;
comportamento estável de foco.
```

### Integrações obrigatórias

```text
LSP file operations;
CMake target update;
Rust module update;
preview/diff;
confirmação para delete.
```

---

## 19. nvim-cmp → Completion Engine

**Repositórios:**

```text
https://github.com/hrsh7th/nvim-cmp
https://github.com/hrsh7th/cmp-nvim-lsp
```

**Licença:** MIT.  
**Modo:** MODE-B.  
**Prioridade:** P1.

### O que reproduzir

```text
múltiplas fontes;
LSP completion;
fuzzy matching;
ranking;
documentation popup;
commit characters;
keyboard navigation;
snippet integration;
path completion;
sem flicker.
```

### Fontes do Kinein

```text
LSP;
snippets;
filesystem/path;
commands;
CMake targets;
Cargo crates/features;
recent symbols.
```

---

## 20. LuaSnip + Friendly Snippets

**Repositórios:**

```text
https://github.com/L3MON4D3/LuaSnip
https://github.com/rafamadriz/friendly-snippets
```

**Licenças:** Apache-2.0 e MIT.  
**Modo:** MODE-B para engine; MODE-C possível para dados.  
**Prioridade:** P1.

### O que reproduzir

```text
tabstops;
placeholders;
choices;
variables;
nested snippets;
snippet history;
LSP snippet syntax;
snippets por linguagem.
```

### Formato recomendado

```text
VS Code Snippet JSON / LSP Snippet Syntax
```

Assim o Kinein pode usar coleções abertas sem depender de VS Code.

---

## 21. Conform.nvim → Formatter Orchestrator

**Repositório:**

```text
https://github.com/stevearc/conform.nvim
```

**Licença:** MIT.  
**Modo:** MODE-B/MODE-C seletivo.  
**Prioridade:** P1.

### O que reproduzir

```text
formatters por linguagem;
minimal diff;
preservação de cursor;
range formatting;
format on save;
encadeamento;
fallback LSP;
timeout;
logs.
```

### Ferramentas alvo

```text
clang-format;
rustfmt;
qmlformat;
cmake-format opcional;
taplo opcional.
```

---

## 22. Error Lens → Diagnostics Inline

**Repositórios:**

```text
https://github.com/usernamehw/vscode-error-lens
https://open-vsx.org/extension/usernamehw/errorlens
```

**Licença:** MIT.  
**Modo:** MODE-B/MODE-C seletivo.  
**Prioridade:** P1.

### O que reproduzir

```text
mensagem inline;
highlight de linha;
gutter icons;
status bar;
filtro por severidade;
toggle;
delay;
modo current-line.
```

### Modos do Kinein

```text
Off
Current Line
Errors Only
Errors + Warnings
Compact
Expanded
```

---

## 23. Trouble.nvim → Problems, Usages e Hierarquias

**Repositório:**

```text
https://github.com/folke/trouble.nvim
```

**Modo:** MODE-B.  
**Prioridade:** P1.

### O que reproduzir

```text
Problems tree;
references;
implementations;
definitions;
document symbols;
call hierarchy;
quickfix results;
filtros;
agrupamentos.
```

---

## 24. Which-key → Descoberta de Atalhos

**Repositório:**

```text
https://github.com/folke/which-key.nvim
```

**Licença:** Apache-2.0.  
**Modo:** MODE-B.  
**Prioridade:** P1/P2.

### O que reproduzir

```text
atalhos contextuais;
descrição de comandos;
prefix menus;
modo de aprendizagem;
cheat sheet;
descoberta sem decorar tudo.
```

### Adaptação JetBrains-like

```text
Find Action;
tooltips de atalhos;
atalhos relacionados;
popup contextual após prefixo;
busca por ação.
```

---

## 25. Aerial.nvim → Outline e Breadcrumbs

**Repositório:**

```text
https://github.com/stevearc/aerial.nvim
```

**Licença:** MIT.  
**Modo:** MODE-B.  
**Prioridade:** P1/P2.

### O que reproduzir

```text
code outline;
symbol tree;
quick navigation;
LSP fallback;
Tree-sitter fallback;
breadcrumbs;
current symbol tracking.
```

---

## 26. Open Remote - SSH + OpenSSH direto

**Repositórios:**

```text
https://github.com/jeanp413/open-remote-ssh
https://open-vsx.org/extension/jeanp413/open-remote-ssh
https://www.openssh.com/
```

**Licença:** MIT para Open Remote - SSH.  
**Modo:** MODE-A para OpenSSH; MODE-B/MODE-C seletivo para a extensão.  
**Prioridade:** P1 alta.

### Por que usar como referência

```text
SSH config;
host selection;
remote server bootstrap;
port forwarding;
remote extension host;
status de conexão;
reconnect;
remote filesystem.
```

### Arquitetura recomendada

Primeiro:

```text
OpenSSH CLI
SFTP/SCP/rsync
remote commands
port forwarding
remote terminal
remote build/run/debug
```

Depois:

```text
Kinein Remote Agent opcional
LSP remoto
file watching remoto
remote task execution
```

### Importante

```text
Não usar Microsoft Remote-SSH proprietário.
Open Remote - SSH é a alternativa FOSS a estudar.
Não depender de um servidor VSCodium remoto no design final.
```

---

## 27. Dev Container CLI

**Repositórios:**

```text
https://github.com/devcontainers/cli
https://containers.dev/
```

**Modo:** MODE-A.  
**Prioridade:** P1/P2.

### O que entrega

```text
devcontainer.json;
build;
up;
exec;
lifecycle commands;
features;
templates;
lockfiles;
ambiente reproduzível;
uso local ou remoto.
```

### Integração no Kinein

```text
Detect Dev Container
Build
Start
Open Terminal
Execute Task
Stop
Rebuild
Inspect Configuration
```

---

# PARTE III — P2: CONFORTO E PRODUTIVIDADE

## 28. nvim-autopairs

**Repositório:**

```text
https://github.com/windwp/nvim-autopairs
```

**Licença:** MIT.  
**Modo:** MODE-B.  
**Prioridade:** P2.

### O que reproduzir

```text
(), [], {}, quotes;
skip de fechamento existente;
regras por linguagem;
integração com completion;
Enter/Backspace inteligentes;
desativação em contextos especiais.
```

---

## 29. Gitsigns UX adicional

Além da gutter, reproduzir:

```text
inline blame opcional;
preview hunk;
stage selected lines;
undo stage;
diff against index;
navigation entre hunks;
status staged/unstaged.
```

---

## 30. crates.nvim → Cargo Dependency Assistant

**Repositório:**

```text
https://github.com/saecki/crates.nvim
```

**Licença:** MIT.  
**Modo:** MODE-B/MODE-C seletivo.  
**Prioridade:** P2.

### O que reproduzir

```text
versão instalada;
latest stable;
incompatibilidade semver;
features;
documentation;
upgrade dependency;
dependency hover;
Cargo.toml inline hints.
```

### Backend recomendado

```text
cargo metadata;
crates.io sparse index/API;
cargo update;
cargo add.
```

---

## 31. Todo Comments

**Repositório:**

```text
https://github.com/folke/todo-comments.nvim
```

**Licença:** Apache-2.0.  
**Modo:** MODE-B.  
**Prioridade:** P2.

### O que reproduzir

```text
TODO;
FIXME;
BUG;
HACK;
WARN;
PERF;
NOTE;
TEST;
highlight;
busca global;
lista no Problems/Tasks panel.
```

---

## 32. GitUI como ferramenta opcional

**Repositório:**

```text
https://github.com/gitui-org/gitui
```

**Licença:** MIT.  
**Modo:** MODE-A opcional; MODE-B para UX.  
**Prioridade:** P2.

### Uso

```text
abrir no terminal integrado;
fallback para operações Git avançadas;
estudar staging por linha/hunk;
branches;
stash;
log;
diff.
```

### Regra

```text
Não substituir o Git Manager nativo;
usar como ferramenta complementar opcional.
```

---

# PARTE IV — P3: TESTES, DEBUG E EMBEDDED

## 33. Neotest como referência de Test Explorer

**Repositório:**

```text
https://github.com/nvim-neotest/neotest
```

**Licença:** MIT.  
**Modo:** MODE-B.  
**Prioridade:** P2/P3.

### Advertência

O próprio projeto ainda se descreve como early-stage.

Portanto:

```text
usar como referência arquitetural;
não adotar como núcleo da IDE;
usar runners reais diretamente.
```

### O que reproduzir

```text
test discovery;
test tree;
run nearest;
run file;
run suite;
output;
diagnostics;
status por teste;
adapters.
```

### Runners iniciais

```text
CTest;
Catch2;
GoogleTest;
cargo test;
cargo nextest futuro.
```

---

## 34. nvim-dap e nvim-dap-ui como referência

**Repositórios:**

```text
https://github.com/mfussenegger/nvim-dap
https://github.com/rcarriga/nvim-dap-ui
```

**Licenças:** GPL-3.0 para nvim-dap; MIT para nvim-dap-ui.  
**Modo:** MODE-D para nvim-dap; MODE-B para nvim-dap-ui.  
**Prioridade:** P2/P3.

### O que reproduzir

```text
DAP lifecycle;
breakpoints;
threads;
stack;
scopes;
variables;
watch expressions;
REPL;
console;
session events;
layout de debug.
```

### Regra jurídica

```text
Não copiar código GPL-3.0 para Core MIT/Apache sem decisão consciente.
Implementar DAP Client próprio com base na especificação.
```

---

## 35. Cortex-Debug como referência de bare metal

**Repositório:**

```text
https://github.com/Marus/cortex-debug
```

**Licença:** MIT.  
**Modo:** MODE-B/MODE-C seletivo.  
**Prioridade:** P3.

### O que reproduzir

```text
GDB/MI;
OpenOCD;
J-Link;
ST-Link;
pyOCD;
Black Magic Probe;
multi-core;
SVD registers;
SWO;
disassembly;
launch profiles.
```

### Regra

```text
Usar como referência madura de arquitetura.
Preferir integração direta com DAP/GDB/probe tools.
```

---

## 36. probe-rs

**Repositório:**

```text
https://github.com/probe-rs/probe-rs
```

**Licença:** MIT OR Apache-2.0.  
**Modo:** MODE-A.  
**Prioridade:** P3.

### O que entrega

```text
flash;
debug;
ARM;
RISC-V;
RTT;
embedded Rust;
chip database;
GDB/MI/debug adapters;
CLI e bibliotecas Rust.
```

---

## 37. serialport-rs

**Repositório:**

```text
https://github.com/serialport/serialport-rs
```

**Licença:** MPL-2.0.  
**Modo:** MODE-A após revisão de licença.  
**Prioridade:** P3.

### O que entrega

```text
enumeração de portas;
USB metadata;
baud rate;
data bits;
parity;
stop bits;
I/O serial;
base para Serial Monitor.
```

### Alternativas

```text
Qt Serial Port;
tokio-serial;
CLI picocom/minicom como fallback.
```

---

## 38. Dev tools embarcadas diretas

Adicionar ao Tool Registry:

```text
OpenOCD
pyOCD
probe-rs
QEMU
gdb-multiarch
arm-none-eabi-gcc
riscv64-unknown-elf-gcc
Zephyr west
CMake toolchain files
Yocto SDK environment
Buildroot SDK
```

Estas ferramentas devem ser integradas diretamente, não via plugin genérico.

---

# 5. Itens a evitar

## 39. Microsoft Remote - SSH

```text
Evitar.
É uma extensão proprietária do produto Visual Studio Code.
Usar Open Remote - SSH apenas como referência FOSS.
Implementar OpenSSH diretamente.
```

## 40. Microsoft C/C++ cpptools

```text
Evitar como motor central.
Usar clangd.
Não depender de componentes binários/proprietários.
```

## 41. GitLens

```text
Evitar como referência principal.
Possui modelo open-core/comercial e funcionalidades proprietárias.
Usar Git CLI + Gitsigns + UI própria.
```

## 42. Copilot, Codeium e extensões semelhantes

```text
Não fazer parte da fundação.
IA deve ser provider externo opcional e sob demanda.
```

## 43. CMake Tools executado sem alterações

```text
Não embutir como-is por causa de dependências do Extension Host e telemetria.
Estudar UX e integrar CMake diretamente.
```

## 44. nvim-lint e toggleterm como código portado

```text
Ambos são GPL-3.0.
Podem ser estudados, mas não copiados automaticamente para Core permissivo.
```

---

# 6. Manifesto de componentes

Criar:

```text
docs/tooling/OPEN_COMPONENT_REGISTRY.json
```

Exemplo:

```json
{
  "schemaVersion": 1,
  "components": [
    {
      "id": "clangd",
      "category": "language-server",
      "priority": "P0",
      "integrationMode": "MODE-A",
      "repository": "https://github.com/llvm/llvm-project",
      "license": "Apache-2.0 WITH LLVM-exception",
      "telemetry": false,
      "networkAtRuntime": false,
      "pin": null,
      "status": "approved-for-research"
    },
    {
      "id": "open-remote-ssh",
      "category": "remote-reference",
      "priority": "P1",
      "integrationMode": "MODE-B",
      "repository": "https://github.com/jeanp413/open-remote-ssh",
      "license": "MIT",
      "telemetry": "audit-required",
      "networkAtRuntime": true,
      "pin": null,
      "status": "approved-for-audit"
    }
  ]
}
```

Estados:

```text
discovered
under-review
approved-for-research
approved-for-adaptation
approved-for-direct-integration
rejected
deprecated
```

---

# 7. Procedimento de varredura de um candidato

A varredura é feita item por item, sem paralelizar candidatos.

## Etapa 1 — Descoberta

```text
abrir repositório oficial;
registrar URL;
registrar owner;
registrar licença;
registrar última release;
registrar último commit;
registrar linguagem;
registrar dependências.
```

## Etapa 2 — Auditoria

```text
buscar telemetry;
buscar analytics;
buscar applicationinsights;
buscar sentry;
buscar posthog;
buscar segment;
buscar network requests;
buscar child_process/exec/spawn;
buscar shell scripts;
buscar download automático;
buscar secrets/env access.
```

## Etapa 3 — Classificação

```text
MODE-A
MODE-B
MODE-C
MODE-D
```

## Etapa 4 — ADR

Criar:

```text
docs/adr/ADR-XXXX-<component>.md
```

Conteúdo:

```text
contexto;
decisão;
alternativas;
licença;
riscos;
integração;
teste;
rollback;
manutenção.
```

## Etapa 5 — Pin

Nunca usar HEAD sem controle.

```text
tag estável;
commit hash;
checksum;
source URL;
data de auditoria.
```

## Etapa 6 — Proof of Concept

```text
branch isolada;
feature flag;
sem acesso a secrets;
sem instalação global;
testes;
benchmark;
logs locais.
```

## Etapa 7 — Aprovação

Somente integrar quando:

```text
licença aprovada;
telemetria removida/inexistente;
testes passam;
performance aceitável;
fallback existe;
documentação existe.
```

---

# 8. Ordem prática de execução

> A lista estendida de 53 candidatos solicitada em 2026-07-15, com decisões
> item a item e dependências L0–L10, está em `PONTO_ATUAL.md` A5.1–A5.4. Essa
> sequência complementa as fases históricas abaixo e prevalece para novas
> adoções: primeiro a capacidade arquitetural, depois a ferramenta. O primeiro
> recorte continua sendo EditorConfig após A3, nunca um Extension Host vazio.

## Fase 0 — Registro e auditoria

```text
1. Criar OPEN_COMPONENT_REGISTRY.json.
2. Criar template ADR.
3. Criar license scanner.
4. Criar telemetry/network scanner.
5. Criar dependency/SBOM report.
```

## Fase 1 — Linguagens

```text
6. clangd.
7. rust-analyzer.
8. Tree-sitter C/C++/Rust/CMake/TOML/QML.
9. EditorConfig.
```

## Fase 2 — Projeto e build

```text
10. CMake File API/Presets.
11. Cargo metadata.
12. Task Manager inspirado em Overseer.
13. Tool Registry inspirado em Mason.
```

## Fase 3 — Editor profissional

```text
14. Completion inspirado em nvim-cmp.
15. Snippets.
16. Formatter orchestrator.
17. Diagnostics inline.
18. Problems/Usages panel.
19. Outline/Breadcrumbs.
20. Autopairs.
```

## Fase 4 — Navegação e Git

```text
21. Search Everywhere.
22. Project Explorer.
23. Git gutter/hunks/blame.
24. TODO navigation.
```

## Fase 5 — Debug/testes

```text
25. DAP Client.
26. CodeLLDB.
27. GDB/MI.
28. Test Explorer.
29. CTest/Catch2/GoogleTest.
30. cargo test/cargo nextest.
```

## Fase 6 — Remote

```text
31. OpenSSH host profiles.
32. Remote terminal.
33. rsync/scp/sftp.
34. remote build/run.
35. port forwarding.
36. gdbserver.
37. remote agent opcional.
38. Dev Container CLI.
```

## Fase 7 — Embedded

```text
39. Serial Monitor.
40. QEMU.
41. OpenOCD.
42. pyOCD.
43. probe-rs.
44. Cortex-Debug UX reference.
45. Zephyr west.
46. Yocto/Buildroot SDK.
```

---

# 9. Lista final resumida por prioridade

## P0

```text
clangd
rust-analyzer
Tree-sitter
CMake File API/Presets
Cargo metadata
CodeLLDB/lldb-dap
EditorConfig
ripgrep
fd
Git CLI
```

## P1

```text
Overseer concepts
Mason registry concepts
Telescope concepts
Neo-tree concepts
nvim-cmp concepts
LuaSnip/Friendly Snippets
Conform concepts
Error Lens concepts
Trouble concepts
Which-key concepts
Aerial concepts
OpenSSH/Open Remote SSH reference
Dev Container CLI
```

## P2

```text
nvim-autopairs concepts
Gitsigns advanced UX
crates.nvim concepts
Todo Comments
GitUI optional
Neotest architecture reference
nvim-dap-ui UX reference
```

## P3

```text
Cortex-Debug reference
probe-rs
serialport-rs / Qt Serial Port
OpenOCD
pyOCD
QEMU
Zephyr west
Yocto SDK
Buildroot SDK
```

---

# 10. Regra final

O objetivo não é “instalar plugins do Neovim ou VSCodium dentro do Kinein”.

O objetivo é:

```text
reaproveitar protocolos;
integrar ferramentas abertas;
estudar arquiteturas maduras;
portar apenas módulos seguros;
construir UI/UX nativa;
preservar licenças;
eliminar telemetria;
manter controle total.
```

O resultado esperado é:

```text
Kinein Vectis
=
IDE C/C++/Rust profissional
+
ferramentas abertas maduras
+
UX visual própria
+
remote/embedded forte
+
zero dependência proprietária
+
zero telemetria obrigatória
```
