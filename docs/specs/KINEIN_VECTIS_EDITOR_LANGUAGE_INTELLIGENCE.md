# Kinein Vectis — Parte 5

# Editor, Language Intelligence, Indexação, Navegação, Diagnósticos e Refatoração

> **Nome oficial:** Kinein Vectis  
> **Nome curto:** Kinein  
> **Sigla visual:** KV  
> **Foco:** C, C++ e Rust  
> **Posicionamento:** IDE Linux-first para sistemas, toolchains, CMake, Rust/Cargo, baixo nível, alto nível e simulação futura.  
> **Objetivo desta parte:** especificar como o editor e a inteligência de linguagem devem funcionar de forma profissional, confortável, previsível e implementável por uma IA CLI.

---

## 1. Propósito desta etapa

A Parte 4 definiu os fluxos de produto para **Toolchain, CMake, Build, Run e Debug**. Esta Parte 5 define o coração da experiência diária da IDE: o **editor** e o sistema de **inteligência de linguagem**.

A Kinein não deve ser apenas um editor de texto com botões de build. Ela deve funcionar como uma IDE real para C, C++ e Rust, com:

- abertura confiável de arquivos grandes;
- realce de sintaxe consistente;
- navegação por símbolos;
- diagnósticos compreensíveis;
- autocompletar contextual;
- integração com `compile_commands.json`, CMake Presets e Cargo;
- suporte a toolchains locais e cross-compilers;
- suporte futuro a sistemas embarcados, Linux embarcado, simulação e OpenGL;
- UX confortável, inspirada na maturidade das IDEs profissionais, sem copiar visualmente nenhuma delas.

A frase-guia desta etapa:

```text
O usuário deve confiar que a Kinein entende o projeto antes de sugerir algo.
```

---

## 2. Escopo da Parte 5

Esta especificação cobre:

1. Modelo de editor.
2. Buffers, documentos e abas.
3. Realce de sintaxe.
4. LSP e servidores de linguagem.
5. Indexação de workspace.
6. Navegação por símbolos.
7. Autocomplete.
8. Diagnósticos.
9. Quick fixes e code actions.
10. Refatoração segura.
11. C/C++ com CMake.
12. Rust com Cargo.
13. Projetos mistos C/C++/Rust.
14. UI/UX do editor.
15. Painéis relacionados.
16. Contratos internos para IA CLI.
17. Critérios de aceite.

Esta etapa **não** cobre implementação profunda de renderização OpenGL, simulação física, debugger visual de hardware ou design do marketplace de plugins. Essas partes devem vir depois.

---

## 3. Princípios de experiência do editor

A experiência do editor deve seguir cinco princípios.

### 3.1 O editor é a área principal da IDE

Todo o layout deve proteger o editor. Painéis, assistente, terminal e tool windows existem para apoiar o código, não para disputar atenção.

Regras:

- o editor deve ocupar a maior área útil da tela;
- painéis laterais devem ser recolhíveis;
- o Assistente não deve abrir sozinho de forma agressiva;
- mensagens e diagnósticos devem ser úteis, não invasivos;
- a linha atual deve ser visível sem ser gritante;
- tooltips devem ajudar sem cobrir o código excessivamente.

### 3.2 A IDE deve parecer inteligente, mas não mágica

A Kinein deve explicar quando não consegue entender o projeto.

Exemplos:

- se `compile_commands.json` não existir, mostrar: “CMake ainda não foi configurado; a análise C/C++ pode estar incompleta.”
- se `rust-analyzer` não estiver disponível, mostrar: “Rust Analyzer não encontrado; instale ou configure um path.”
- se o target for cross-compilation e o sysroot estiver ausente, mostrar o erro de forma direta.

Nunca fingir que entende o projeto quando a base técnica está ausente.

### 3.3 Diagnóstico deve virar ação

Um erro não deve ser apenas uma linha vermelha. Sempre que possível, a IDE deve oferecer uma próxima ação:

- configurar CMake;
- selecionar toolchain;
- instalar servidor de linguagem;
- abrir erro no terminal;
- aplicar fix-it;
- explicar erro no Assistente;
- abrir documentação local ou externa;
- gerar configuração sugerida.

### 3.4 C/C++ e Rust devem ser cidadãos de primeira classe

A IDE não deve tratar Rust como extensão secundária nem CMake como “detalhe”. O produto nasce para essas três linguagens.

C/C++:

- CMake;
- compile commands;
- GCC/Clang;
- cross-compilers;
- includes;
- macros;
- linker;
- clangd;
- clang-tidy futuro.

Rust:

- Cargo;
- rust-analyzer;
- rustfmt;
- clippy;
- features;
- targets;
- workspaces;
- no_std futuro;
- crates e modules.

### 3.5 Conforto visual acima de efeito visual

O editor deve priorizar leitura prolongada.

Evitar:

- glow em texto de código;
- amarelo em excesso;
- contraste agressivo;
- ícones decorativos no gutter;
- animações em diagnóstico;
- tooltips gigantes.

Usar:

- contraste equilibrado;
- acento âmbar apenas para foco/ação;
- hierarquia visual clara;
- tipografia monoespaçada confortável;
- espaçamento respirável;
- bordas sutis.

---

## 4. Arquitetura conceitual do editor

A arquitetura deve separar claramente **texto**, **linguagem**, **projeto** e **interface**.

```text
Kinein App Shell
  └── Editor Area
        ├── Tab Manager
        ├── Text Editor Component
        ├── Gutter Component
        ├── Minimap Optional
        ├── Breadcrumbs
        ├── Inline Diagnostics
        └── Completion Popup

Kinein Core
  ├── Workspace Service
  ├── Document Service
  ├── Language Service
  ├── Index Service
  ├── Diagnostics Service
  ├── Refactor Service
  ├── CMake Service
  ├── Cargo Service
  └── Toolchain Service

External Tools
  ├── clangd
  ├── rust-analyzer
  ├── cmake
  ├── ninja/make
  ├── gcc/clang
  ├── cargo/rustc
  ├── gdb/lldb
  └── optional future analyzers
```

### 4.1 Separação obrigatória

A UI não deve executar diretamente comandos de linguagem. Ela deve chamar serviços do core.

Errado:

```text
Botão do editor chama clangd diretamente.
```

Certo:

```text
UI -> LanguageService -> LspClient -> clangd
```

Motivo:

- facilita testes;
- permite substituir implementação;
- prepara plugins;
- separa Qt/QML do core Rust/C++;
- permite logs, jobs e cancelamento.

---

## 5. Modelo de documento e buffer

### 5.1 Documento

Um documento representa o conteúdo aberto, salvo ou temporário.

Campos conceituais:

```json
{
  "uri": "file:///workspace/src/main.cpp",
  "language": "cpp",
  "encoding": "utf-8",
  "lineEnding": "LF",
  "version": 42,
  "isDirty": true,
  "isReadonly": false,
  "lastSavedHash": "...",
  "detectedFrom": "extension|modeline|manual"
}
```

### 5.2 Buffer

O buffer é o conteúdo editável em memória.

Requisitos:

- suportar arquivos médios e grandes sem travar a UI;
- rastrear versões para LSP;
- permitir undo/redo;
- permitir múltiplas cursors no futuro;
- aceitar edição incremental;
- gerar eventos para diagnóstico e indexação.

### 5.3 Estado dirty

A aba deve indicar claramente quando há alterações não salvas.

Visual:

- ponto pequeno ou marcador discreto na aba;
- não usar alerta vermelho;
- status bar deve mostrar “Modified” apenas quando útil.

### 5.4 Autosave

No MVP, autosave pode ser opcional e desligado por padrão.

Configurações futuras:

```json
{
  "editor.autosave.enabled": false,
  "editor.autosave.delayMs": 1500,
  "editor.autosave.onFocusLost": false
}
```

---

## 6. Abas, grupos e navegação entre arquivos

### 6.1 Abas

Cada arquivo aberto aparece como aba no topo do editor.

Informações da aba:

- ícone da linguagem;
- nome do arquivo;
- estado modificado;
- erro ou warning no arquivo;
- botão de fechar no hover;
- menu contextual.

Exemplo:

```text
main.cpp    motor_control.cpp ●    CMakeLists.txt    lib.rs
```

### 6.2 Grupos de editor

A IDE deve suportar split horizontal e vertical.

Ações:

- Split Right;
- Split Down;
- Move Tab to Other Group;
- Close Others;
- Close Unmodified;
- Reopen Closed Editor.

### 6.3 Preview tab

Inspirado em IDEs profissionais, ao clicar em um arquivo pelo Project Explorer, ele pode abrir em modo preview.

Regras:

- clique simples abre preview;
- duplo clique fixa aba;
- editar o arquivo fixa a aba automaticamente;
- preview deve ter estilo visual discreto.

---

## 7. Realce de sintaxe

### 7.1 Requisito mínimo

Suporte inicial:

- C;
- C++;
- Rust;
- CMake;
- JSON;
- TOML;
- Markdown;
- Shell;
- YAML.

### 7.2 Estratégia recomendada

A Kinein pode usar uma camada de realce baseada em parser incremental, mas deve manter fallback simples para arquivos desconhecidos.

Modelo:

```text
SyntaxHighlightService
  ├── grammar registry
  ├── token themes
  ├── semantic tokens from LSP
  └── fallback tokenizer
```

### 7.3 Ordem de prioridade visual

1. Semantic tokens do LSP, quando disponíveis.
2. Syntax tokens do parser local.
3. Regex/fallback para linguagens simples.

### 7.4 Tema de código

O tema não deve ser exageradamente colorido. O objetivo é leitura.

Sugestão:

```text
Keywords: roxo discreto
Types/classes: azul técnico suave
Functions: amarelo pálido ou verde suave
Strings: verde cinza
Numbers: laranja suave
Comments: cinza reduzido
Macros/preprocessor: âmbar discreto
Errors: vermelho controlado
Warnings: âmbar controlado
```

O amarelo principal da marca não deve dominar o código.

---

## 8. Language Service

O `LanguageService` é o orquestrador das funcionalidades de linguagem.

Responsabilidades:

- iniciar e parar servidores de linguagem;
- mapear linguagem para servidor;
- gerenciar configurações;
- abrir documentos no LSP;
- enviar mudanças incrementais;
- receber diagnósticos;
- pedir completion;
- pedir hover;
- pedir go to definition;
- pedir references;
- pedir rename;
- pedir code actions;
- fornecer estado para UI.

### 8.1 Servidores iniciais

C/C++:

```text
clangd
```

Rust:

```text
rust-analyzer
```

CMake:

```text
servidor CMake opcional no futuro
fallback inicial com highlighting + snippets + validação simples
```

### 8.2 Estados do servidor

Cada servidor deve ter estado claro.

```text
Not configured
Starting
Indexing
Ready
Degraded
Failed
Stopped
```

Visual na status bar:

```text
clangd: Ready
rust-analyzer: Indexing
CMake: Not configured
```

### 8.3 Modo degradado

Modo degradado significa: a linguagem abre, edita e realça sintaxe, mas a inteligência está incompleta.

Exemplos:

- `clangd` ativo, mas sem `compile_commands.json`;
- `rust-analyzer` ativo, mas `cargo metadata` falhou;
- projeto C++ sem toolchain selecionada;
- SDK embarcado sem include paths.

A UI deve mostrar isso sem assustar:

```text
Análise parcial: compile_commands.json não encontrado.
```

---

## 9. Integração C/C++ com CMake e compile_commands

### 9.1 Regra principal

Para C/C++, a fonte de verdade da análise deve ser o comando de compilação real.

Ordem de preferência:

1. `compile_commands.json` gerado pelo CMake.
2. CMake Presets conhecidos.
3. Configuração manual da toolchain.
4. Heurística limitada.

### 9.2 Fluxo esperado

```text
Abrir workspace
  ↓
Detectar CMakeLists.txt
  ↓
Verificar CMakePresets.json
  ↓
Verificar build directory
  ↓
Procurar compile_commands.json
  ↓
Se ausente: sugerir Configure
  ↓
Após configure: reiniciar/atualizar clangd
  ↓
Editor recebe diagnósticos confiáveis
```

### 9.3 UI para análise incompleta

No editor, evitar erro agressivo. Usar banner discreto:

```text
C/C++ analysis is partial. Configure CMake to enable accurate includes and diagnostics.
[Configure CMake] [Select Toolchain] [Ignore]
```

Em português, se o idioma da UI for pt-BR:

```text
A análise C/C++ está parcial. Configure o CMake para habilitar includes e diagnósticos precisos.
[Configurar CMake] [Selecionar Toolchain] [Ignorar]
```

### 9.4 Cross-compilation

Para sistemas embarcados, a IDE deve entender que o compilador alvo pode ser diferente do compilador host.

Campos relevantes:

```json
{
  "target": "arm-none-eabi",
  "compiler": "/opt/toolchains/gcc-arm/bin/arm-none-eabi-g++",
  "sysroot": "/opt/sdk/sysroot",
  "cmakeToolchainFile": "cmake/arm-none-eabi.cmake",
  "defines": ["STM32F767xx"],
  "includeDirs": ["include", "drivers/CMSIS"]
}
```

### 9.5 Diagnóstico de include

Quando um header não for encontrado, o Assistente deve sugerir:

- verificar `target_include_directories`;
- verificar `compile_commands.json`;
- verificar CMake configure;
- verificar sysroot;
- verificar SDK;
- verificar nome do arquivo/case-sensitive.

---

## 10. Integração Rust com Cargo

### 10.1 Fonte de verdade

Para Rust, a fonte de verdade é o Cargo.

Detectar:

- `Cargo.toml`;
- `Cargo.lock`;
- workspace Cargo;
- crates locais;
- features;
- target;
- toolchain channel;
- `.cargo/config.toml`.

### 10.2 Fluxo esperado

```text
Abrir workspace
  ↓
Detectar Cargo.toml
  ↓
Rodar cargo metadata
  ↓
Iniciar rust-analyzer
  ↓
Carregar crates, modules e features
  ↓
Exibir estado na status bar
```

### 10.3 Features

A IDE deve permitir escolher features de forma visual.

Exemplo:

```text
Rust Features: default, embedded, telemetry
```

UI:

- dropdown no painel Cargo/Rust;
- chips de features ativas;
- aviso se feature inválida;
- ação para editar `Cargo.toml`.

### 10.4 no_std futuro

Para projetos embarcados Rust, a IDE deve prever `no_std`, mas não precisa implementar tudo no MVP.

Sinais:

- presença de `#![no_std]`;
- target customizado;
- `.cargo/config.toml`;
- runner customizado;
- probe-rs futuro.

---

## 11. Projetos mistos C/C++/Rust

A Kinein deve aceitar projetos híbridos.

Exemplos:

- C++ app com biblioteca Rust via FFI;
- Rust app chamando C via `build.rs`;
- firmware com C, C++ e Rust experimental;
- backend C++ com módulos Rust;
- simulação C++ com Rust tooling.

### 11.1 Regra de UX

O usuário não deve precisar escolher “modo C++” ou “modo Rust” para o workspace inteiro. A IDE deve detectar por arquivo e por projeto.

### 11.2 Project Model

```text
Workspace
  ├── CMake Project Model
  ├── Cargo Project Model
  ├── File System Model
  ├── Toolchain Model
  └── Target Model
```

### 11.3 Barra de contexto do editor

No topo do editor, breadcrumbs e contexto podem mostrar:

```text
src > drivers > motor > motor_control.cpp > MotorControl > update
```

Para Rust:

```text
src > lib.rs > module sensor > impl SensorBus > read
```

---

## 12. Indexação

### 12.1 Objetivo

A indexação torna a IDE rápida para:

- buscar arquivos;
- buscar símbolos;
- montar outline;
- navegar por referências;
- gerar breadcrumbs;
- alimentar Assistente;
- acelerar command palette.

### 12.2 Tipos de índice

```text
File Index
  Arquivos, paths, extensões, tamanho, última modificação.

Symbol Index
  Classes, funções, structs, enums, traits, macros, modules.

Project Index
  Targets CMake, crates Cargo, toolchains, presets.

Diagnostics Index
  Últimos erros, warnings, severidade, origem.

Build Index
  Últimos builds, comandos, duração, status, artifacts.
```

### 12.3 Indexação incremental

A IDE não deve reindexar tudo a cada alteração.

Eventos:

- arquivo criado;
- arquivo removido;
- arquivo renomeado;
- arquivo salvo;
- CMake reconfigurado;
- Cargo metadata atualizado;
- toolchain alterada;
- build directory alterado.

### 12.4 Estados visuais

Status bar:

```text
Indexing… 42%
Index ready
Index degraded
Index failed
```

Não bloquear o editor enquanto indexa.

---

## 13. Navegação por símbolos

### 13.1 Go to Definition

Ação principal:

```text
Ctrl+B ou Ctrl+Click
```

Comportamento:

- se houver único destino, navegar direto;
- se houver múltiplos, abrir popup compacto;
- se o destino estiver em arquivo gerado ou externo, indicar claramente.

### 13.2 Go to Declaration

Importante em C/C++.

Exemplo:

- de `MotorControl::update` para declaração no `.hpp`.

### 13.3 Go to Implementation

Importante para:

- interfaces C++;
- traits Rust;
- métodos virtuais;
- abstrações.

### 13.4 Find References

Exibir em painel inferior ou popup dedicado.

Layout:

```text
References: MotorControl::update
  src/drivers/motor/motor_control.cpp:24
  src/core/app.cpp:78
  tests/motor_control_test.cpp:41
```

### 13.5 Symbol Search

Command palette:

```text
@ MotorControl
@ update
@ PID
```

### 13.6 Workspace Search

Separar busca textual de busca por símbolo.

- Search: texto bruto;
- Symbol Search: símbolos indexados;
- File Search: arquivos;
- Command Search: comandos.

---

## 14. Autocomplete

### 14.1 Fonte

Ordem de prioridade:

1. LSP completions.
2. Snippets da linguagem.
3. Palavras do buffer.
4. Símbolos do workspace.
5. Sugestões do Assistente somente quando explicitamente solicitadas.

### 14.2 UX do popup

O popup deve ser compacto.

Campos:

- ícone do tipo;
- nome;
- assinatura curta;
- origem;
- documentação resumida ao lado sob demanda.

Exemplo:

```text
compute(float target, float current, float dt)   method
current                                    variable
MotorDriver                                class
```

### 14.3 Não ser invasivo

Evitar:

- sugestões gigantes;
- completar automaticamente sem confirmação;
- inserir imports/includes sem avisar;
- IA sugerindo código grande no fluxo normal.

### 14.4 Includes automáticos

Quando uma completion exigir include, a IDE deve mostrar:

```text
PIDController — class
Add #include "control/pid_controller.hpp"
```

A aplicação do include deve ser explícita ou seguir configuração do usuário.

---

## 15. Hover e documentação inline

### 15.1 Hover básico

Ao passar o mouse sobre símbolo:

- tipo;
- assinatura;
- documentação curta;
- arquivo de origem;
- ações rápidas.

### 15.2 Hover de erro

Para diagnóstico:

```text
No matching function for call to compute
```

Deve mostrar:

- mensagem original;
- origem: clangd/build/rust-analyzer/cargo;
- severidade;
- sugestões;
- botão “Explain in Assistente”.

### 15.3 Não cobrir código demais

Hover deve ter largura máxima e não ocupar metade da tela.

---

## 16. Diagnósticos

### 16.1 Fontes de diagnóstico

```text
LSP diagnostics
Build diagnostics
CMake diagnostics
Cargo diagnostics
Static analysis future
Assistente explanations
```

### 16.2 Normalização

Todos os diagnósticos devem virar um modelo comum:

```json
{
  "id": "diag-123",
  "source": "clangd",
  "severity": "error",
  "file": "src/main.cpp",
  "range": { "startLine": 18, "startCol": 12, "endLine": 18, "endCol": 20 },
  "message": "use of undeclared identifier 'pid'",
  "code": "undeclared_var_use",
  "related": [],
  "actions": []
}
```

### 16.3 Severidades

```text
Error: vermelho controlado
Warning: âmbar
Info: azul técnico
Hint: cinza/verde suave
```

### 16.4 Gutter

No gutter:

- erro: marcador pequeno vermelho;
- warning: marcador âmbar;
- breakpoint: ponto vermelho/roxo futuro;
- execução atual: seta discreta.

Não usar ícones grandes ou poluição visual.

### 16.5 Problems Panel

Painel inferior deve agrupar por:

- arquivo;
- severidade;
- origem;
- target/build profile.

Exemplo:

```text
Problems
  Errors 2  Warnings 5

  src/drivers/motor/motor_control.cpp
    E 18:12 use of undeclared identifier 'pid'       clangd
    W 22:5  unused variable 'dt'                     clang-tidy future

  CMakeLists.txt
    E target_link_libraries references unknown target motor_core   cmake
```

### 16.6 Diagnóstico de build vs diagnóstico do editor

A IDE deve diferenciar:

- erro do LSP em tempo real;
- erro real do build;
- erro de configuração;
- erro do debugger;
- erro do target remoto.

Exemplo de UX:

```text
Build failed: 3 errors
Editor diagnostics may be stale. Last successful configure: 12 min ago.
```

---

## 17. Quick fixes e Code Actions

### 17.1 Ações básicas

C/C++:

- add missing include;
- qualify namespace;
- generate function definition;
- create declaration/definition pair;
- apply clangd fix-it;
- organize includes futuro.

Rust:

- add use statement;
- apply rust-analyzer assist;
- create function;
- implement missing trait items;
- convert match/if let;
- cargo fix futuro.

### 17.2 UX

Atalho:

```text
Alt+Enter
```

Popup:

```text
Quick Fixes
  Add #include "control/pid.hpp"
  Create local variable 'pid'
  Explain error in Assistente
  Ignore diagnostic
```

### 17.3 Aplicação segura

Toda alteração automática deve:

- mostrar preview quando for ampla;
- preservar formatação;
- permitir undo;
- não alterar múltiplos arquivos sem confirmação clara.

---

## 18. Refatoração

### 18.1 Filosofia

Refatoração é área sensível. A Kinein deve ser conservadora.

No MVP, suportar:

- Rename Symbol via LSP;
- Move/Rename File com atualização simples de references quando seguro;
- Generate Definition/Declaration para C++ quando LSP suportar;
- Apply Fix-it;
- Format Document;
- Format Selection.

Não tentar implementar refatorações complexas sem base sólida.

### 18.2 Rename Symbol

Fluxo:

```text
Usuário aciona Rename
  ↓
LanguageService valida símbolo
  ↓
LSP retorna workspace edits
  ↓
UI mostra preview resumido
  ↓
Usuário confirma
  ↓
DocumentService aplica edits
  ↓
Index atualiza
```

### 18.3 Preview obrigatório para múltiplos arquivos

Se a refatoração alterar mais de um arquivo, exibir preview:

```text
Rename MotorControl -> MotorController
  src/drivers/motor/motor_control.hpp     3 changes
  src/drivers/motor/motor_control.cpp     5 changes
  tests/motor_control_test.cpp            2 changes

[Apply Refactor] [Cancel]
```

### 18.4 Refatorações futuras

Depois do MVP:

- Extract Function;
- Change Signature;
- Move Symbol;
- Inline Function;
- Generate Tests;
- Convert C struct to C++ class helper;
- Rust module move;
- FFI helper generation.

---

## 19. Formatação

### 19.1 Format on save

Configuração:

```json
{
  "editor.formatOnSave": false,
  "cpp.formatter": "clang-format",
  "rust.formatter": "rustfmt"
}
```

Desligado por padrão no início para evitar surpresas.

### 19.2 Respeitar arquivos do projeto

C/C++:

- `.clang-format`;
- fallback interno apenas se não houver arquivo.

Rust:

- `rustfmt.toml`;
- fallback padrão do rustfmt.

### 19.3 UI

Status bar:

```text
Formatter: clang-format
Formatter: rustfmt
```

Se ausente:

```text
clang-format not found
[Configure]
```

---

## 20. Assistente integrado ao editor

O Assistente não deve ser um chatbot solto. Ele deve entender o contexto técnico da IDE.

### 20.1 Fontes de contexto permitidas

- arquivo atual;
- seleção atual;
- diagnósticos atuais;
- build log recente;
- CMake configure log;
- Cargo metadata;
- toolchain selecionada;
- target selecionado;
- symbols do arquivo;
- documentação configurada.

### 20.2 Ações do editor

Ações contextuais:

```text
Explain this function
Explain diagnostic
Suggest fix
Explain build error
Summarize file
Generate tests future
Explain CMake target
Explain Rust feature issue
```

### 20.3 Não atrapalhar

Assistente deve ser acionável, não invasivo.

Permitido:

- botão discreto no hover de erro;
- botão no Problems panel;
- menu contextual;
- shortcut.

Evitar:

- popups automáticos de IA;
- sugestões longas dentro do editor sem pedido;
- modificar código sem confirmação.

---

## 21. UI do editor

### 21.1 Elementos visuais

```text
Editor Header
  ├── Tabs
  ├── Breadcrumbs
  └── File state indicators

Editor Body
  ├── Gutter
  ├── Line numbers
  ├── Breakpoints future
  ├── Code area
  ├── Inline diagnostics
  ├── Completion popup
  └── Hover popup

Editor Footer/Status
  ├── Ln/Col
  ├── Encoding
  ├── Line Ending
  ├── Language Mode
  ├── LSP Status
  └── Target context
```

### 21.2 Gutter

Gutter deve conter:

- números de linha;
- fold markers;
- diagnostics markers;
- breakpoint markers futuro;
- current execution line futuro.

### 21.3 Linha atual

Visual:

```text
background: #1A1F26
border-left opcional: âmbar muito sutil apenas quando foco ativo
```

### 21.4 Seleção

Seleção deve ser confortável:

```text
selection background: azul/cinza escuro
não usar amarelo forte para seleção de texto
```

### 21.5 Busca no arquivo

Find widget compacto no canto superior direito do editor.

Campos:

- termo;
- match count;
- next/previous;
- case sensitive;
- regex;
- whole word;
- replace toggle.

---

## 22. Painéis relacionados

### 22.1 Structure

Mostra símbolos do arquivo atual.

Para C++:

```text
MotorControl
  MotorControl(MotorDriver*)
  init(): bool
  update(float dt): void
  setTarget(float): void
```

Para Rust:

```text
SensorBus
  trait SensorBus
  struct I2cSensorBus
  impl SensorBus for I2cSensorBus
  fn read_temperature()
```

### 22.2 Problems

Consolidado de diagnósticos.

### 22.3 Search

Busca textual.

### 22.4 Symbols

Pode ser integrado ao Search ou Command Palette.

### 22.5 Assistente

Painel direito contextual.

---

## 23. Contratos internos para IA CLI

A IA CLI deve implementar por serviços, não por telas soltas.

### 23.1 Serviços sugeridos

```text
DocumentService
LanguageService
LspClient
IndexService
DiagnosticsService
SymbolService
CompletionService
RefactorService
FormatService
EditorStateService
```

### 23.2 Comandos internos sugeridos

```json
{ "cmd": "document.open", "uri": "file:///workspace/src/main.cpp" }
{ "cmd": "document.save", "uri": "file:///workspace/src/main.cpp" }
{ "cmd": "language.start", "language": "cpp" }
{ "cmd": "language.status", "language": "cpp" }
{ "cmd": "index.refresh", "scope": "workspace" }
{ "cmd": "symbols.search", "query": "MotorControl" }
{ "cmd": "diagnostics.list", "scope": "workspace" }
{ "cmd": "completion.request", "uri": "...", "line": 18, "character": 12 }
{ "cmd": "hover.request", "uri": "...", "line": 18, "character": 12 }
{ "cmd": "definition.goto", "uri": "...", "line": 18, "character": 12 }
{ "cmd": "references.find", "uri": "...", "line": 18, "character": 12 }
{ "cmd": "refactor.rename", "uri": "...", "line": 18, "character": 12, "newName": "MotorController" }
{ "cmd": "format.document", "uri": "..." }
```

### 23.3 Eventos sugeridos

```json
{ "event": "document.changed", "uri": "...", "version": 43 }
{ "event": "diagnostics.updated", "uri": "...", "count": 3 }
{ "event": "language.statusChanged", "language": "cpp", "status": "Indexing" }
{ "event": "index.progress", "percent": 42 }
{ "event": "symbols.updated", "uri": "..." }
{ "event": "editor.activeFileChanged", "uri": "..." }
```

---

## 24. Persistência

### 24.1 Estado de editor

Arquivo conceitual:

```text
.kinein/editor-state.json
```

Conteúdo:

```json
{
  "openTabs": [
    "src/main.cpp",
    "src/drivers/motor/motor_control.cpp",
    "CMakeLists.txt"
  ],
  "activeTab": "src/drivers/motor/motor_control.cpp",
  "cursorPositions": {
    "src/main.cpp": { "line": 42, "column": 8 }
  },
  "splits": [],
  "recentFiles": []
}
```

### 24.2 Estado de linguagem

```text
.kinein/language-state.json
```

Conteúdo:

```json
{
  "cpp": {
    "server": "clangd",
    "status": "ready",
    "compileCommands": "build/compile_commands.json"
  },
  "rust": {
    "server": "rust-analyzer",
    "status": "ready",
    "cargoMetadataStatus": "ok"
  }
}
```

### 24.3 Cache

A IDE pode manter cache interno, mas não deve exigir que o usuário edite manualmente.

```text
.kinein/cache/
```

---

## 25. Roadmap por fases

### Fase 1 — Editor básico confiável

- abrir/salvar arquivos;
- abas;
- line numbers;
- syntax highlighting básico;
- find in file;
- status bar de arquivo;
- Project Explorer integrado.

### Fase 2 — LSP mínimo

- iniciar clangd;
- iniciar rust-analyzer;
- diagnósticos no editor;
- hover;
- go to definition;
- autocomplete básico.

### Fase 3 — Integração com projeto real

- CMake configure alimenta clangd;
- detectar compile commands;
- cargo metadata alimenta Rust;
- Problems panel unificado;
- status de linguagem.

### Fase 4 — Navegação profissional

- find references;
- symbol search;
- structure panel;
- breadcrumbs semânticos;
- outline por arquivo.

### Fase 5 — Refatoração segura

- rename symbol;
- apply code actions;
- format document;
- preview de alterações multi-file.

### Fase 6 — Inteligência assistida

- Assistente explica diagnóstico;
- Assistente explica build error;
- Assistente sugere ajustes de CMake/toolchain;
- ações sempre confirmadas.

---

## 26. O que não implementar agora

Para proteger qualidade, não implementar no início:

- refatorações complexas próprias sem LSP;
- IA escrevendo grandes blocos automaticamente;
- debugger visual embutido avançado;
- OpenGL viewport dentro do editor;
- simulação física integrada;
- marketplace de plugins;
- terminal remoto complexo;
- suporte completo a todos os build systems;
- parser C++ próprio;
- indexador semântico próprio completo.

A Kinein deve começar sólida, não gigante.

---

## 27. Critérios de aceite

A Parte 5 pode ser considerada bem implementada quando:

- abrir um projeto CMake simples e editar C++ for confortável;
- configurar CMake gerar `compile_commands.json` e melhorar diagnósticos automaticamente;
- `clangd` mostrar hover, completion e go to definition;
- abrir um projeto Rust/Cargo iniciar `rust-analyzer` corretamente;
- diagnostics aparecerem no gutter, editor e Problems panel;
- erros de build forem navegáveis;
- arquivos modificados forem claros;
- a IDE não travar durante indexação;
- Assistente conseguir explicar um erro selecionado sem atrapalhar;
- Rename Symbol funcionar com preview quando alterar múltiplos arquivos;
- o usuário sempre souber se a análise está pronta, parcial ou falhou.

---

## 28. Checklist para IA CLI

A IA CLI deve seguir esta ordem:

```text
1. Criar modelos de Document, Buffer e EditorTab.
2. Criar DocumentService com open/save/change events.
3. Criar Editor UI básica com tabs, gutter e status.
4. Criar LanguageService sem LSP real ainda, com estados mockados.
5. Integrar LspClient para clangd.
6. Integrar LspClient para rust-analyzer.
7. Normalizar diagnostics.
8. Exibir diagnostics no editor e Problems panel.
9. Implementar hover e completion.
10. Implementar go to definition.
11. Implementar indexação de arquivos/símbolos simples.
12. Implementar symbol search.
13. Implementar code actions básicas.
14. Implementar rename com preview.
15. Integrar Assistente com seleção/diagnóstico.
```

Regra final:

```text
Não avançar para recursos chamativos antes do editor ficar confiável.
```

---

## 29. Prompt de implementação para IA CLI

```text
Implemente a Parte 5 da Kinein Vectis: Editor, Language Intelligence, Indexação, Diagnósticos e Refatoração.

Siga a arquitetura por serviços: DocumentService, LanguageService, LspClient, IndexService, DiagnosticsService, SymbolService, CompletionService, RefactorService e EditorStateService.

Priorize C/C++ com clangd e Rust com rust-analyzer. Para C/C++, use compile_commands.json gerado pelo CMake como fonte principal de análise. Para Rust, use Cargo.toml/cargo metadata como base para rust-analyzer. A UI deve mostrar claramente estados Ready, Indexing, Degraded e Failed.

Construa a experiência do editor com abas, gutter, line numbers, syntax highlighting, diagnostics inline, hover, completion, go to definition, find references, structure panel e Problems panel. O visual deve seguir o sistema Kinein: dark confortável, acento âmbar com moderação, tipografia legível e foco absoluto no editor.

Não implemente refatorações complexas próprias. Use LSP para rename/code actions e sempre mostre preview quando múltiplos arquivos forem alterados. Assistente deve explicar erros e contexto apenas quando acionado pelo usuário, sem popups invasivos.

Implemente em fases, com testes e critérios de aceite claros. Não avance para simulação/OpenGL antes do editor e da inteligência de linguagem estarem confiáveis.
```

---

## 30. Resumo executivo

A Parte 5 define a Kinein como uma IDE real para C, C++ e Rust.

O editor deve ser:

- rápido;
- confortável;
- confiável;
- integrado ao projeto;
- honesto sobre o que entende;
- excelente em CMake/Cargo;
- preparado para sistemas embarcados;
- extensível para simulação futura;
- familiar para usuários de IDEs profissionais;
- autoral na identidade visual e na experiência de toolchain.

A Kinein vence quando o programador sente:

```text
“Eu posso focar no código. A IDE me ajuda a entender o projeto e o ambiente.”
```
