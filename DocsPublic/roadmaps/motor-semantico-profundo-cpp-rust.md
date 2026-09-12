# Kinein Vectis — Motor Semântico Profundo e Fluxo Moderno para C/C++ e Rust

**Status:** especificação arquitetural complementar  
**Projeto:** Kinein Vectis  
**Data:** 14 de julho de 2026  
**Escopo:** `clangd`, `rust-analyzer`, Tree-sitter, `lldb-dap`, build systems, análise estática, testes e coordenação semântica do workspace  
**Prioridades:** C, C++, Rust, Linux, sistemas embarcados, cross-compilation, desempenho, privacidade e operação local-first

---

> **Estado em 2026-09-12.** Esta especificação continua sendo ALVO, não
> implementação — com uma exceção nascida da exigência do autor de que *"a
> IDE deve ler o projeto inteiro"*: a §2 ("entender o projeto inteiro") tem
> agora a primeira forma concreta no domínio `index` do core
> ([`40`](40-estado-e-continuidade.md) §7.17): todas as pastas, arquivos e
> declarações de C/C++/Rust (Python quando a gramática entrar), com as
> gramáticas Tree-sitter do editor, em job, com busca por nome sem LSP. O
> passo seguinte é o *contexto de compilador por arquivo* (§6: CMake File
> API/CDB, Cargo Metadata) — é o Project Graph desta página começando a
> existir, fatia a fatia, pelo `roadmaps/42` P0. Scheduler, brokers e RAM
> budget continuam só aqui.

## 1. Decisão executiva

É tecnicamente viável fazer o Kinein Vectis oferecer um fluxo mais coerente, transparente e moderno que o fluxo típico de VS Code/VSCodium e, em áreas específicas, mais conveniente que o CLion.

A meta inicial, entretanto, não deve ser reproduzir integralmente a profundidade do motor proprietário do CLion.

O CLion 2026.1 Nova utiliza o motor C++ do ReSharper/Rider para recursos centrais de inteligência, e não apenas `clangd`. A própria documentação da JetBrains esclarece que o Nova não usa `clangd` para recursos centrais como highlighting e completion.[^1] Isso significa que simplesmente conectar o Kinein ao `clangd` não produz automaticamente equivalência completa com o CLion.

A estratégia correta é:

```text
Não construir outro compilador C++.
Não reimplementar rust-analyzer.
Não limitar a IDE a ser uma interface genérica de LSP.

Construir uma camada própria de inteligência do workspace
que coordena motores especializados, build systems, perfis,
targets, diagnósticos, testes e depuradores.
```

O Kinein deve usar:

- `clangd` como motor semântico principal para C e C++;
- `rust-analyzer` como motor semântico principal para Rust;
- Tree-sitter como motor sintático incremental e sempre disponível;
- `lldb-dap` como ponte principal para depuração LLDB;
- GDB através de um adaptador compatível para toolchains e targets que exijam GDB;
- CMake File API, compilation database, Cargo Metadata e Meson introspection como fontes do modelo de projeto;
- `clang-tidy`, Clang Static Analyzer, Clippy e o compilador real como camadas adicionais;
- um **Kinein Semantic Workspace Engine**, pertencente à IDE, para coordenar tudo isso.

---

## 2. O que significa “entender o projeto inteiro”

“Entender o projeto inteiro” não significa manter uma AST completa de todos os arquivos aberta simultaneamente.

Essa abordagem seria cara, duplicaria trabalho dos motores e não corresponde à arquitetura do `clangd`.

Para C e C++, existem pelo menos três níveis diferentes:

```text
Nível 1 — árvore sintática do arquivo
Nível 2 — AST e preâmbulo dos arquivos ativos
Nível 3 — índice semântico de todo o projeto
```

O `clangd` mantém um índice dinâmico dos arquivos abertos e um índice em segundo plano de todas as unidades de tradução encontradas na compilation database. Esse índice global registra símbolos, referências e relações do codebase.[^2]

O `clangd` não precisa manter a AST completa de cada arquivo permanentemente para oferecer navegação global. Ele guarda AST e preâmbulo para arquivos ativos e usa índices persistentes para o restante.

Para Rust, o `rust-analyzer` trabalha com um grafo de crates, árvores sintáticas por arquivo, HIR, resolução de nomes, expansão de macros, inferência de tipos e uma base incremental baseada em Salsa. O estado relevante do projeto é mantido em memória e os resultados derivados são recomputados incrementalmente.[^3]

Portanto, a definição correta para o Kinein é:

```text
Projeto totalmente compreendido =
  modelo correto de targets e dependências
+ todos os arquivos relevantes descobertos
+ todos os contextos de compilação conhecidos
+ índices semânticos concluídos
+ dependências e macros carregadas
+ diagnósticos do compilador disponíveis
+ mecanismos para consultar símbolos e relações globalmente
```

---

## 3. Onde usar a RAM economizada

A economia de memória do núcleo da IDE pode ser usada de forma produtiva, mas não deve ser desperdiçada mantendo estruturas redundantes.

A prioridade deve ser:

1. índices completos;
2. caches quentes;
3. múltiplos contextos de compilação;
4. múltiplos targets;
5. múltiplos feature sets em Rust;
6. análise estática em lote;
7. histórico de diagnósticos;
8. cache de resultados de navegação;
9. pré-carregamento inteligente dos arquivos mais importantes;
10. processos semânticos isolados e reiniciáveis.

A RAM adicional é especialmente útil para executar:

```text
clangd: Debug host
clangd: Release host
clangd: ARM embedded
rust-analyzer: host/default features
rust-analyzer: embedded/no_std
rust-analyzer: tests/all targets
```

Não é necessário manter todos esses contextos com prioridade máxima o tempo inteiro. O Kinein pode ter um contexto ativo e outros contextos em prioridade reduzida.

---

# Parte I — Kinein Semantic Workspace Engine

## 4. Nova camada central

Este documento propõe o seguinte subsistema:

```text
Kinein Semantic Workspace Engine
abreviação: KSWE
```

O KSWE não é um parser de C++ ou Rust. Ele é o coordenador que compreende:

- workspaces;
- projetos;
- targets;
- toolchains;
- build profiles;
- source sets;
- compilation commands;
- target triples;
- sysroots;
- feature sets;
- dependências;
- language servers;
- analisadores;
- formatadores;
- testes;
- executáveis;
- configurações de execução;
- configurações de depuração;
- dispositivos;
- estado de indexação.

### 4.1 Visão geral

```text
┌─────────────────────────────────────────────────────────────┐
│                       Kinein UI                             │
│ Editor · Project View · Search · Diagnostics · Debug · Test │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│             Kinein Semantic Workspace Engine               │
│                                                             │
│ Project Graph · Context Matrix · Request Scheduler          │
│ Diagnostic Broker · Symbol Broker · Capability Registry     │
│ Build Coordinator · Test Coordinator · Debug Coordinator    │
└──────────────┬─────────────────┬─────────────────┬──────────┘
               │                 │                 │
       ┌───────▼───────┐ ┌──────▼────────┐ ┌─────▼─────────┐
       │ Syntax Layer  │ │ Semantic Layer │ │ Execution     │
       │ Tree-sitter   │ │ clangd         │ │ CMake/Cargo   │
       │ local index   │ │ rust-analyzer  │ │ Ninja/Meson   │
       └───────────────┘ └────────────────┘ └─────┬─────────┘
                                                  │
                                  ┌───────────────▼──────────┐
                                  │ Debug and Analysis       │
                                  │ lldb-dap · GDB · tidy    │
                                  │ Clippy · Static Analyzer │
                                  └──────────────────────────┘
```

---

## 5. Grafo unificado do projeto

O KSWE deve normalizar CMake, Cargo, Meson e compilation databases em um modelo comum.

### 5.1 Estrutura conceitual

```rust
struct WorkspaceModel {
    roots: Vec<ProjectRoot>,
    projects: Vec<ProjectModel>,
    targets: Vec<TargetModel>,
    contexts: Vec<AnalysisContext>,
    toolchains: Vec<ToolchainModel>,
    source_files: Vec<SourceFile>,
    dependencies: Vec<DependencyEdge>,
}

struct AnalysisContext {
    id: ContextId,
    language: Language,
    project: ProjectId,
    target: TargetId,
    build_profile: BuildProfile,
    toolchain: ToolchainId,
    target_triple: Option<String>,
    sysroot: Option<PathBuf>,
    defines: Vec<String>,
    include_paths: Vec<PathBuf>,
    compiler_flags: Vec<String>,
    rust_features: Vec<String>,
    environment: EnvironmentMap,
}

struct TargetModel {
    id: TargetId,
    name: String,
    kind: TargetKind,
    sources: Vec<FileId>,
    generated_sources: Vec<FileId>,
    dependencies: Vec<TargetId>,
    outputs: Vec<PathBuf>,
    test_entries: Vec<TestId>,
}
```

### 5.2 Motivo

O language server não deve ser a única fonte de informação da IDE.

O `clangd` conhece a semântica C/C++, mas não deve ser responsável por responder sozinho:

- quais targets existem;
- qual target gera determinado binário;
- quais perfis estão configurados;
- como executar um teste;
- qual dispositivo deve receber o firmware;
- qual build directory está ativo;
- quais variáveis CMake existem;
- qual target Rust produz determinado executável;
- quais ambientes de execução estão disponíveis.

O KSWE deve conhecer isso através dos build systems.

---

## 6. Fontes do modelo de projeto

### 6.1 CMake File API

A CMake File API fornece informações semânticas do buildsystem gerado, incluindo:

- codemodel;
- directories;
- targets;
- cache;
- toolchains;
- arquivos CMake;
- backtrace graph;
- configure log.

Ela é uma API versionada, baseada em arquivos JSON dentro da build tree.[^4]

O Kinein deve usar a CMake File API como fonte principal do modelo CMake.

```text
CMakeLists.txt
      │
      ▼
CMake configure
      │
      ├── compile_commands.json
      └── .cmake/api/v1/reply/
              ├── codemodel
              ├── targets
              ├── toolchains
              ├── cache
              └── cmakeFiles
```

A division of responsibility deve ser:

```text
CMake File API
→ estrutura do projeto e dos targets

compile_commands.json
→ comandos semânticos exatos para clangd

Ninja/Make
→ execução real do build
```

### 6.2 Compilation database

A compilation database descreve os comandos de compilação por unidade de tradução.

Para C/C++, ela é essencial porque o significado do código depende de:

- `-I`;
- `-isystem`;
- `-D`;
- standard selecionado;
- arquitetura;
- target triple;
- sysroot;
- compilador;
- working directory;
- flags específicas do target.

A documentação do `clangd` enfatiza que comandos inadequados produzem erros de parsing, includes ausentes e interpretações incorretas do código.[^5]

### 6.3 Cargo Metadata

`cargo metadata` fornece JSON com:

- membros do workspace;
- packages;
- targets;
- features;
- dependências;
- grafo resolvido;
- workspace root;
- target directory;
- informações de plataforma.

A documentação do Cargo afirma explicitamente que a saída inclui o grafo de dependências resolvido para o workspace inteiro.[^6]

### 6.4 `rust-project.json`

Projetos Rust que não são modelados diretamente por Cargo podem fornecer um `rust-project.json`.

O Kinein deve suportar:

- Cargo;
- `rust-project.json`;
- múltiplos projetos Rust ligados;
- crates geradas;
- projetos `no_std`;
- toolchains customizadas.

### 6.5 Meson introspection

Meson gera informações de introspecção em `meson-info`, incluindo:

- targets;
- sources;
- generated sources;
- build options;
- dependencies;
- tests;
- benchmarks;
- arquivos do buildsystem.

A documentação do Meson descreve essa API como mecanismo para uma experiência de IDE comparável a uma integração nativa.[^7]

---

# Parte II — `clangd` em profundidade

## 7. O que o `clangd` já oferece

O `clangd` usa o frontend do Clang para analisar C e C++ e oferece, entre outros:

- completion;
- signature help;
- hover;
- diagnostics;
- fixes;
- definition;
- declaration;
- references;
- workspace symbols;
- semantic tokens;
- inlay hints;
- type hierarchy;
- call hierarchy;
- rename;
- code actions;
- include insertion;
- Include Cleaner;
- `clang-tidy`;
- índice de todo o projeto.

O índice do `clangd` contém:

- símbolos;
- locais de declaração e definição;
- documentação;
- referências;
- relações entre símbolos.

O `BackgroundIndex` analisa os arquivos da compilation database e persiste shards `.idx` para evitar reindexação completa a cada inicialização.[^2]

---

## 8. Arquitetura de índice do `clangd`

```text
MergedIndex
├── FileIndex
│   ├── arquivos abertos
│   ├── headers incluídos
│   └── dados atualizados durante edição
│
├── BackgroundIndex
│   ├── todas as unidades de tradução
│   ├── símbolos
│   ├── referências
│   ├── relações
│   └── shards persistentes
│
└── External Index, opcional
    ├── índice monolítico local
    └── índice remoto
```

### 8.1 FileIndex

É a camada mais atualizada.

Ela contém dados dos arquivos abertos e seus headers, garantindo que alterações recentes sejam visíveis antes que o índice em segundo plano seja atualizado.

### 8.2 BackgroundIndex

É responsável por cobertura global.

Ao encontrar uma compilation database, os comandos de compilação são colocados em uma fila e processados em paralelo.[^2]

### 8.3 Índice estático local

O `clangd-indexer` pode gerar um índice monolítico, que pode ser carregado com um arquivo externo.

O Kinein pode futuramente oferecer:

```text
Build Local Semantic Index
```

Uso:

- CI local;
- workspaces enormes;
- cache pré-construído;
- abertura inicial mais rápida;
- distribuição interna de índice dentro de equipes.

O formato interno do índice não deve ser tratado como banco de dados público estável pelo Kinein. A IDE deve deixar o `clangd` carregar e consultar o índice.

---

## 9. O modo profundo correto para C/C++

O modo profundo não deve abrir todos os arquivos artificialmente.

Ele deve:

1. validar toda a compilation database;
2. iniciar o background index;
3. aguardar a indexação integral;
4. indexar a standard library;
5. habilitar referências sem limite artificial;
6. manter preâmbulos dos arquivos mais usados;
7. executar análise estática em lote;
8. manter contextos múltiplos quando necessário;
9. expor o consumo de memória;
10. exibir o progresso real.

### 9.1 Configuração conceitual

```yaml
# .clangd gerenciado ou sugerido pelo Kinein

Index:
  Background: Build
  StandardLibrary: true

Completion:
  AllScopes: Yes
  ArgumentLists: FullPlaceholders
  HeaderInsertion: IWYU
  CodePatterns: All

Diagnostics:
  UnusedIncludes: Strict
  MissingIncludes: Strict
  ClangTidy:
    Add:
      - bugprone-*
      - performance-*
      - modernize-*
      - readability-*
    Remove:
      - modernize-use-trailing-return-type
    FastCheckFilter: Loose

InlayHints:
  Enabled: true
  ParameterNames: true
  DeducedTypes: true
  Designators: true
  DefaultArguments: true
  BlockEnd: true

Hover:
  ShowAKA: true

Documentation:
  CommentFormat: Doxygen
```

Essas opções devem ser apresentadas em UI, e não impostas silenciosamente.

### 9.2 Argumentos de processo sugeridos

```text
clangd
--background-index
--clang-tidy
--completion-style=detailed
--header-insertion=iwyu
--limit-references=0
--query-driver=<allowlist-de-compiladores>
```

O `--limit-references=0` remove o limite padrão de resultados de referências. A documentação oficial recomenda esse parâmetro quando o usuário precisa receber mais de mil resultados.[^8]

`--query-driver` deve ser montado apenas com compiladores confiáveis, porque permite ao `clangd` executar o driver para descobrir includes e target. A documentação do `clangd` exige habilitação explícita exatamente por esse motivo.[^5]

---

## 10. Validação integral da compilation database

Antes de considerar o projeto “semanticamente pronto”, o Kinein deve executar uma auditoria.

### 10.1 Validações

```text
[ ] Todas as entradas têm arquivo existente?
[ ] O working directory existe?
[ ] O compilador existe?
[ ] O compilador é permitido pelo usuário?
[ ] Includes resolvem?
[ ] Sysroot existe?
[ ] Target triple é reconhecido?
[ ] Standard está definido?
[ ] Há arquivos duplicados com comandos incompatíveis?
[ ] Há fontes do target fora da compilation database?
[ ] Há headers sem includer conhecido?
[ ] Arquivos gerados já foram produzidos?
[ ] A build tree corresponde ao profile ativo?
```

### 10.2 Painel “Effective Compile Context”

Ao selecionar um arquivo:

```text
File: src/device/controller.cpp
Target: kinein-device-agent
Profile: Debug-ARM
Compiler: /opt/gcc-arm/bin/arm-none-eabi-g++
Standard: C++23
Target: arm-none-eabi
Sysroot: /opt/gcc-arm/arm-none-eabi
Defines: 34
Include paths: 18
Compilation database: build-arm/compile_commands.json
Command origin: exact
Semantic engine: clangd 22
Index state: complete
```

O usuário deve conseguir:

- copiar o comando;
- executar o comando;
- comparar com outro profile;
- abrir a entrada JSON;
- identificar flags adicionadas pelo `.clangd`;
- identificar flags removidas;
- trocar o contexto ativo.

Essa transparência pode ser melhor que o fluxo de IDEs tradicionais.

---

## 11. Headers e contextos ambíguos

Headers normalmente não possuem comandos próprios na compilation database.

O `clangd` pode:

- usar o comando de um arquivo que inclui o header;
- escolher uma unidade de tradução com nome semelhante;
- usar heurística de interpolação.

A documentação reconhece que essa heurística pode escolher um contexto inadequado.[^5]

O Kinein deve melhorar a experiência mostrando:

```text
Header: include/device/protocol.hpp

Contexto atual:
  emprestado de src/device/protocol.cpp

Outros contextos disponíveis:
  tests/protocol_tests.cpp
  src/simulator/protocol.cpp
  firmware/protocol.cpp
```

O usuário pode escolher:

```text
Usar automaticamente o includer mais relevante
Fixar contexto para este header
Fixar contexto por diretório
Comparar diagnósticos em múltiplos contextos
```

---

## 12. Matriz semântica C/C++

Um projeto pode interpretar o mesmo arquivo de formas diferentes.

Exemplo:

```text
Contexto A
Target: desktop
Compiler: Clang
Defines: KINEIN_DESKTOP, QT_WIDGETS

Contexto B
Target: embedded
Compiler: ARM GCC
Defines: KINEIN_EMBEDDED, NO_EXCEPTIONS

Contexto C
Target: tests
Compiler: Clang
Defines: KINEIN_TESTING, MOCK_HARDWARE
```

Uma única instância de `clangd` não representa simultaneamente todos os contextos contraditórios de um arquivo.

### 12.1 Solução proposta

```text
Semantic Context Matrix
├── Contexto ativo
│   └── prioridade interativa
├── Contextos secundários
│   └── prioridade de background
└── Contextos suspensos
    └── cache preservado
```

O Kinein pode executar uma instância de `clangd` por contexto incompatível.

### 12.2 Regras

- apenas o contexto ativo fornece completion;
- semantic tokens vêm do contexto ativo;
- contextos secundários podem produzir diagnósticos;
- resultados secundários são identificados pelo target;
- renames são aplicados apenas por uma autoridade semântica;
- comparações entre contexts são explícitas;
- caches e compilation databases devem ficar isolados.

### 12.3 Interface

```text
Context: Debug/Desktop ▼

Available:
✓ Debug/Desktop
  Release/Desktop
! Debug/ARM
  Tests/Desktop
```

Diagnóstico de múltiplos contexts:

```text
protocol.hpp:88

Desktop:
  OK

ARM:
  error: exceptions are disabled

Tests:
  warning: mock-only branch is unreachable
```

Esse recurso pode ser um diferencial real do Kinein.

---

## 13. Project-wide diagnostics para C/C++

O índice global do `clangd` não significa que ele execute todos os diagnósticos completos em todos os arquivos o tempo inteiro.

Os diagnósticos interativos são associados principalmente aos arquivos analisados pelo frontend durante a edição. O índice fornece navegação e referências globais, mas uma verificação completa deve usar processos de build e análise em lote.

O KSWE deve separar:

```text
Interactive Diagnostics
├── clang parser
├── clangd
├── clang-tidy rápido
└── Include Cleaner

Workspace Diagnostics
├── build real
├── clang-tidy batch
├── Clang Static Analyzer
├── CTU analysis
└── testes
```

### 13.1 `clang-tidy` em lote

`clang-tidy` usa a compilation database e pode executar:

- checks próprios;
- checks do Clang Static Analyzer;
- grupos `bugprone`;
- `performance`;
- `concurrency`;
- `cert`;
- `cppcoreguidelines`;
- `modernize`;
- `readability`.

A documentação oficial confirma que o `clang-tidy` é baseado em LibTooling e se beneficia da compilation database.[^9]

### 13.2 Clang Static Analyzer com CTU

A análise padrão normalmente opera dentro de uma única unidade de tradução.

O modo Cross Translation Unit permite importar definições de outras unidades durante a análise. A documentação oficial descreve suporte baseado em PCH ou análise on-demand e recomenda ferramentas como CodeChecker para automação.[^10]

O Kinein pode oferecer:

```text
Analyze Project
├── Fast
│   └── clang-tidy selecionado
├── Full
│   ├── clang-tidy completo
│   └── build diagnostics
└── Deep CTU
    ├── Clang Static Analyzer
    ├── cross-translation-unit
    └── relatório de caminhos
```

---

## 14. Include Cleaner e gráfico de includes

O Include Cleaner do `clangd` usa a AST para detectar includes não utilizados e leva em conta referências explícitas, macros, deduções e instanciações de templates.[^11]

O Kinein deve construir uma UI superior:

```text
Include Graph
├── direct includes
├── transitive includes
├── public/private headers
├── unused includes
├── missing includes
├── include cycles
└── estimated parse cost
```

Para cada include:

```text
#include <vector>

Status: required
Reason:
  std::vector used at controller.cpp:42
Provider:
  /usr/include/c++/...
Context:
  Debug/Desktop
```

Para include removível:

```text
#include "legacy/compat.hpp"

Status: unused
Confidence: high
Affected contexts:
  Debug/Desktop: unused
  Debug/ARM: required
```

O último caso mostra por que a matriz de contexts importa.

---

## 15. Extensões específicas do `clangd`

O Kinein não deve limitar o cliente ao LSP básico.

O `clangd` possui extensões oficiais para:

- trocar source/header;
- status do arquivo;
- compilation commands;
- forçar diagnósticos;
- categorias de diagnóstico;
- fixes inline;
- symbol info;
- type hierarchy;
- score de completion;
- AST;
- consumo de memória;
- container de referências;
- regiões inativas.

A documentação oficial lista essas extensões e alerta que elas podem evoluir.[^12]

### 15.1 Recursos prioritários

#### File Status

Mostrar:

```text
Parsing
Building preamble
Indexing
Idle
```

#### AST Viewer

A extensão `textDocument/ast` permite inspecionar a AST sem reimplementar o parser.

O Kinein pode oferecer:

```text
Semantic AST Inspector
├── declaration
├── statement
├── expression
├── implicit cast
├── macro expansion
└── source range
```

#### Memory Usage

O `$/memoryUsage` fornece estimativa hierárquica da memória do `clangd`.[^12]

Painel:

```text
clangd memory: 2.8 GiB
├── background index: 1.2 GiB
├── main.cpp preamble: 380 MiB
├── controller.cpp AST: 210 MiB
├── standard library: 420 MiB
└── other: 590 MiB
```

Isso permite usar RAM agressivamente sem perder observabilidade.

---

# Parte III — `rust-analyzer` em profundidade

## 16. O modelo semântico do Rust

O `rust-analyzer` não é apenas um parser conectado ao editor.

Sua arquitetura inclui:

- parsing tolerante a código incompleto;
- árvores sintáticas por arquivo;
- grafo de crates;
- expansão de macros;
- resolução de nomes;
- HIR;
- inferência de tipos;
- base incremental;
- caches on-demand;
- serviços de IDE;
- integração com Cargo;
- integração com build scripts;
- processo separado para procedural macros.

A documentação da arquitetura explica que:

- o grafo de crates representa roots, `cfg` e dependências;
- Salsa é usado para computação incremental e on-demand;
- resolução de nomes, expansão de macros e inferência de tipos ficam nas crates HIR;
- alterações no corpo de uma função não devem invalidar dados globais não relacionados.[^3]

---

## 17. O que significa carregar todo o projeto Rust

Quando corretamente configurado, o `rust-analyzer` conhece:

- todos os crates do workspace;
- dependências;
- roots;
- features ativas;
- targets;
- `cfg`;
- sysroot;
- standard library sources;
- macros declarativas;
- procedural macros;
- build script outputs;
- tipos;
- impls;
- referências.

`cargo metadata` fornece o grafo resolvido do workspace, enquanto o `rust-analyzer` transforma isso em seu `CrateGraph`.[^6]

A arquitetura do `rust-analyzer` mantém os dados de entrada em memória e deriva o restante incrementalmente.[^3]

---

## 18. Deep Mode para Rust

### 18.1 Configuração conceitual

```json
{
  "cachePriming": {
    "enable": true,
    "numThreads": "physical"
  },
  "cargo": {
    "allTargets": true,
    "autoreload": true,
    "buildScripts": {
      "enable": true,
      "rebuildOnSave": true,
      "useRustcWrapper": true
    },
    "noDeps": false,
    "targetDir": true
  },
  "check": {
    "workspace": true,
    "allTargets": true,
    "command": "check"
  },
  "procMacro": {
    "enable": true,
    "attributes": {
      "enable": true
    },
    "processes": 2
  },
  "completion": {
    "autoimport": {
      "enable": true
    },
    "limit": null
  },
  "diagnostics": {
    "enable": true,
    "experimental": {
      "enable": false
    }
  },
  "inlayHints": {
    "chainingHints": {
      "enable": true
    },
    "closingBraceHints": {
      "enable": true
    },
    "closureCaptureHints": {
      "enable": true
    },
    "implicitDrops": {
      "enable": true
    }
  },
  "lru": {
    "capacity": 512
  }
}
```

O valor do LRU deve ser escolhido dinamicamente. A documentação expõe `lru.capacity` e informa que o padrão mantém um conjunto limitado de syntax trees.[^13]

### 18.2 Cache Priming

`cachePriming.enable` aquece caches quando o projeto é carregado. O número de threads também é configurável.[^13]

O Kinein deve mostrar:

```text
Rust Semantic Warm-up
Crates: 81/81
Files parsed: 1,642/1,642
Dependencies loaded: 143/143
Macros ready: 98%
Caches primed: 92%
```

### 18.3 Build scripts e proc macros

Ativar build scripts e procedural macros aumenta a precisão, mas executa código do projeto.

O Kinein deve apresentar isso como permissão:

```text
Este workspace solicita:
✓ executar build.rs
✓ carregar procedural macros
✓ executar cargo metadata
✓ executar cargo check
```

Os proc macros são executados em um processo separado pelo próprio `rust-analyzer`, permitindo recuperação caso uma macro falhe ou cause crash.[^3]

---

## 19. Cargo Check, Clippy e diagnósticos

O `rust-analyzer` possui diagnósticos próprios, mas também executa um comando de check.

A configuração padrão pode construir uma invocação semelhante a:

```text
cargo check --quiet --workspace --message-format=json --all-targets --keep-going
```

A documentação oficial descreve `check.workspace`, `check.allTargets`, override commands e execução por workspace.[^13]

O Kinein deve separar:

```text
RA Native Diagnostics
→ rápidos, incrementais, sem build completo

Cargo Check
→ verdade do compilador

Clippy
→ linting profundo

Tests
→ comportamento real
```

### 19.1 Perfis

#### Interactive

```text
rust-analyzer native diagnostics
cargo check on save
default feature set
active target
```

#### Deep

```text
workspace check
all targets
Clippy
build scripts
proc macros
cache priming
```

#### Offline Restricted

```text
cargo.noDeps = true
build scripts = disabled or prompt
proc macros = disabled or prompt
network = denied
```

---

## 20. Feature Matrix em Rust

Executar sempre com `--all-features` não é necessariamente correto.

Features podem ser mutuamente exclusivas.

Exemplo:

```toml
[features]
backend-opcua = []
backend-modbus = []
runtime-std = []
runtime-embedded = []
```

Combinar todas pode produzir um estado inválido.

### 20.1 Contextos nomeados

```text
Rust Contexts
├── Default
│   ├── default features
│   └── host target
├── Industrial
│   ├── backend-opcua
│   ├── runtime-std
│   └── x86_64-unknown-linux-gnu
├── Embedded
│   ├── backend-modbus
│   ├── runtime-embedded
│   └── thumbv7em-none-eabihf
└── Tests
    ├── all test targets
    └── host target
```

Assim como em C++, o Kinein pode executar instâncias separadas do `rust-analyzer` quando os contexts forem incompatíveis.

### 20.2 Autoridade ativa

Apenas o context ativo deve:

- fornecer completion;
- fornecer rename;
- decidir imports;
- fornecer semantic tokens principais.

Contexts secundários podem:

- verificar compilação;
- produzir diagnósticos identificados;
- manter índice;
- validar compatibilidade.

---

## 21. Funcionalidades específicas do `rust-analyzer`

O cliente do Kinein deve expor mais que o LSP mínimo.

A documentação do `rust-analyzer` descreve recursos como:

- annotations;
- auto import;
- completion com imports;
- expand macro recursively;
- find all references;
- go to implementation;
- inlay hints;
- memory layout no hover;
- runnables;
- related tests;
- rename com tratamento semântico;
- semantic highlighting;
- memory usage;
- structural selection;
- item tree.

O Find All References inclui referências em expansões de macros, patterns, type contexts e referências através de borrow/deref.[^14]

### 21.1 Macro Expansion Viewer

```text
Original
derive(MyTrait)

Expanded
impl MyTrait for Device {
    ...
}
```

Recursos:

- expansão por nível;
- diferenças;
- origem da macro;
- tokens gerados;
- navegação da expansão para o source;
- avisos sobre proc macro não confiável.

### 21.2 Memory Layout Hover

O `rust-analyzer` pode apresentar:

- size;
- alignment;
- padding;
- niches;
- field offsets.

O Kinein pode transformar isso em uma visualização:

```text
DeviceState — 24 bytes, alignment 8

0..8    id
8..9    status
9..16   padding
16..24  timestamp
```

### 21.3 Related Tests

A IDE pode mostrar os testes relacionados a um item e permitir:

- run;
- debug;
- run with coverage;
- run selected;
- compare last result.

---

# Parte IV — Tree-sitter como camada instantânea

## 22. Função correta do Tree-sitter

Tree-sitter é uma biblioteca de parsing incremental que atualiza a árvore sintática eficientemente enquanto o arquivo é editado.[^15]

Ele deve ser a camada sempre disponível, mesmo quando:

- o language server ainda não iniciou;
- o projeto não compila;
- a compilation database está ausente;
- Cargo está indisponível;
- o engine semântico reiniciou;
- o arquivo não pertence a um target.

### 22.1 Responsabilidades

- syntax highlighting inicial;
- folding;
- outline sintático;
- breadcrumbs;
- seleção estrutural;
- text objects;
- matching de blocos;
- indentação assistida;
- detecção de funções/classes;
- linguagem injetada;
- índice lexical;
- navegação preliminar;
- edição estrutural.

### 22.2 Não é responsabilidade

- resolução semântica completa;
- tipos reais;
- overload resolution;
- templates C++;
- borrow checking;
- trait solving;
- macros expandidas semanticamente;
- refactoring global seguro;
- compilação.

---

## 23. Três índices no Kinein

```text
Syntax Index
├── Tree-sitter
├── todos os arquivos reconhecidos
├── rápido
└── impreciso semanticamente

Semantic Index
├── clangd
├── rust-analyzer
├── símbolos e referências reais
└── dependente do contexto

Build/Analysis Index
├── compilador
├── clang-tidy
├── Clippy
├── static analyzer
└── diagnósticos completos
```

### 23.1 Fallback progressivo

```text
T0 — arquivo abriu
Tree-sitter responde imediatamente

T1 — language server analisou arquivo
semantic tokens substituem/augmentam syntax tokens

T2 — índice global disponível
referências e workspace symbols completos

T3 — build/análise profunda concluída
diagnósticos completos
```

A interface não deve piscar ou apagar informações. Cada camada deve atualizar somente o que sabe melhorar.

---

## 24. Índice sintático do workspace

O Kinein pode usar Tree-sitter para analisar todos os arquivos reconhecidos em background e gerar um índice leve.

Tree-sitter possui queries de tags para capturar definições e referências sintáticas. A documentação define convenções como:

- `@definition.class`;
- `@definition.function`;
- `@definition.method`;
- `@reference.call`;
- `@reference.class`.[^16]

Esse índice serve para:

- busca instantânea antes do LSP;
- outline global;
- símbolos em arquivos não pertencentes ao build;
- detecção de código gerado ou abandonado;
- navegação em linguagens sem LSP;
- restauração de sessão;
- priorização do aquecimento semântico.

Ele não deve ser apresentado como verdade semântica.

### 24.1 Confiança do resultado

```text
Symbol result:
  source: Tree-sitter
  confidence: syntactic
  semantic confirmation: pending
```

Depois:

```text
Symbol result:
  source: clangd
  confidence: semantic
  context: Debug/Desktop
```

---

## 25. Queries e language injection

Tree-sitter suporta queries para:

- highlights;
- locals;
- injections.

A documentação descreve queries locais com:

- `@local.scope`;
- `@local.definition`;
- `@local.reference`.

Também descreve language injections para código incorporado dentro de outra linguagem.[^17]

Uso no Kinein:

```text
C++ raw string contendo SQL
Markdown contendo C++
QML contendo JavaScript
Rust doc comment contendo Rust
CMake contendo generator expressions
```

---

# Parte V — Cliente LSP profissional

## 26. Não basta “implementar LSP”

Um cliente LSP simples pode oferecer completion e diagnostics.

Uma IDE profissional precisa implementar:

- ciclo de vida;
- capabilities;
- dynamic registration;
- incremental synchronization;
- position encoding;
- cancellation;
- progress;
- partial results;
- request ordering;
- snapshots;
- stale result rejection;
- workspace edits;
- transactional edits;
- file operations;
- semantic token deltas;
- lazy resolve;
- multiple servers;
- custom extensions;
- observabilidade.

A especificação LSP 3.17 inclui cancellation, progress genérico, partial results, semantic tokens, inlay hints, call hierarchy, type hierarchy e diagnostic pull.[^18]

---

## 27. Scheduler de requisições

Prioridades:

```text
P0 — edição e sincronização
P1 — completion e signature help
P2 — hover e navigation
P3 — diagnostics do arquivo ativo
P4 — references e workspace symbols
P5 — indexação e warming
P6 — análises profundas
```

### 27.1 Cancelamento

Quando o usuário digita, resultados de uma versão anterior podem ficar obsoletos.

O cliente deve:

- enviar `$/cancelRequest`;
- descartar respostas de snapshot antigo;
- preservar partial results quando úteis;
- evitar bloquear a UI;
- reagendar apenas o necessário.

O `clangd` também usa filas por arquivo, descarta operações obsoletas e separa AST/preamble para manter responsividade.[^19]

O `rust-analyzer` utiliza revisão global e cancelamento de computações obsoletas em sua base incremental.[^3]

---

## 28. Semantic tokens sem flicker

Pipeline:

```text
Tree-sitter highlight
       │
       ▼
semanticTokens/range para viewport
       │
       ▼
semanticTokens/full
       │
       ▼
semanticTokens/full/delta nas edições seguintes
```

O LSP 3.17 suporta semantic tokens completos, por range e por delta.[^18]

Regras:

- Tree-sitter nunca deve desaparecer antes do semantic layer chegar;
- semantic tokens podem complementar syntax tokens;
- tokens devem ser versionados por documento;
- delta inválido deve cair para full request;
- viewport tem prioridade sobre minimap;
- arquivos invisíveis não devem competir com completion.

---

## 29. Inlay hints interativos

O LSP 3.17 permite labels compostos, tooltips, locations clicáveis e resolução lazy.[^18]

O Kinein deve permitir:

- clicar no tipo inferido;
- navegar do hint para a definição;
- ativar por categoria;
- limitar por viewport;
- mostrar hints adicionais no modo aprofundado;
- reduzir ruído automaticamente.

### 29.1 C++

```text
auto value = create();     value: DeviceState
send(data, size);          data: ..., size: ...
```

### 29.2 Rust

```text
let state = load();        state: DeviceState
iterator.map(...);         Item = SensorReading
```

---

## 30. Workspace edits transacionais

Refactorings e fixes podem alterar muitos arquivos.

O Kinein deve:

1. validar versões;
2. construir preview;
3. detectar arquivos modificados externamente;
4. aplicar em uma transação;
5. salvar atomicamente;
6. permitir rollback;
7. executar formatador seletivamente;
8. revalidar o projeto.

```text
Rename DeviceManager → DeviceController

Files: 18
Edits: 74
Contexts checked:
✓ Debug/Desktop
✓ Tests/Desktop
! Debug/ARM — 2 ambiguous references
```

Esse fluxo pode ser mais seguro que simplesmente aplicar o `WorkspaceEdit`.

---

# Parte VI — Depuração com `lldb-dap`

## 31. Arquitetura

`lldb-dap` expõe o LLDB através do Debug Adapter Protocol para qualquer IDE que implemente DAP.[^20]

```text
Kinein Debug UI
       │
       ▼
Kinein DAP Client
       │
       ▼
lldb-dap
       │
       ▼
LLDB
       │
       ├── local process
       ├── core dump
       ├── gdb-remote
       └── remote target
```

O LLDB oferece mecanismos de extensão como:

- data formatters;
- frame recognizers;
- Python scripting;
- custom commands.

Esses mecanismos também ficam disponíveis através do `lldb-dap`.[^20]

---

## 32. Capability-driven UI

O Kinein não deve presumir que todo adapter implementa tudo.

Ao inicializar a sessão, deve ler as capabilities e habilitar:

- conditional breakpoints;
- function breakpoints;
- hit conditions;
- logpoints;
- data breakpoints;
- instruction breakpoints;
- read memory;
- write memory;
- disassembly;
- restart frame;
- set expression;
- cancel;
- stepping granularity.

A documentação oficial do `lldb-dap` publica sua matriz de capabilities.[^20]

A UI deve ser adaptativa.

---

## 33. Modelo de configuração sem `launch.json`

O Kinein pode importar `launch.json`, mas seu modelo interno deve ser tipado.

```rust
struct DebugProfile {
    name: String,
    target: TargetId,
    context: ContextId,
    adapter: DebugAdapter,
    program: PathBuf,
    arguments: Vec<String>,
    working_directory: PathBuf,
    environment: EnvironmentMap,
    source_maps: Vec<SourceMap>,
    attach: Option<AttachConfig>,
    remote: Option<RemoteDebugConfig>,
}
```

A configuração deve ser derivada do target sempre que possível.

```text
Target: kinein-agent
Output: build/debug/bin/kinein-agent
Debugger: lldb-dap
Working directory: project root
Environment: Debug/Desktop
```

O usuário não deve digitar novamente caminhos que a IDE já conhece.

---

## 34. Fluxo de depuração moderno

### 34.1 One-click debug

```text
cursor dentro de teste
→ Kinein identifica target
→ constrói se necessário
→ resolve executável
→ inicia lldb-dap
→ adiciona breakpoint temporário
→ executa teste específico
```

### 34.2 Inline values

O LSP 3.17 possui suporte a inline values associado a um frame DAP.[^18]

O Kinein pode mostrar durante pause:

```cpp
int retries = 3;        retries = 2
auto status = read();   status = Timeout
```

### 34.3 Memory and disassembly

Painéis:

- memory viewer;
- register viewer;
- disassembly;
- source/assembly mixed view;
- peripheral/register descriptions futuramente;
- watch expressions;
- data breakpoints.

### 34.4 Remote debugging

O LLDB usa arquitetura client-server para remote debugging através do protocolo gdb-remote.[^21]

O Kinein deve modelar:

```text
Local build
Remote deploy
Remote process
Source mapping
Sysroot mapping
Shared libraries
Connection transport
```

---

## 35. C++ e Rust mistos

Projetos podem ter:

- core Rust;
- frontend C++;
- FFI C;
- bibliotecas estáticas;
- `cbindgen`;
- `bindgen`;
- CMake chamando Cargo;
- Cargo chamando CMake.

O KSWE deve registrar:

```text
FFI Boundary
├── exported C symbol
├── Rust implementation
├── C/C++ declaration
├── generated header
├── ABI
└── producing target
```

Depuração:

- uma única sessão LLDB quando o binário contém símbolos compatíveis;
- formatters adequados;
- transição de stack C++ → C ABI → Rust;
- source maps;
- nomes demangled.

Esse fluxo integrado pode ser um diferencial do Kinein.

---

# Parte VII — Broker de diagnósticos

## 36. Fontes

```text
Tree-sitter
clangd
clang compiler
GCC
clang-tidy
Clang Static Analyzer
rust-analyzer
rustc
Clippy
CMake
Cargo
Meson
Ninja
tests
debugger
```

### 36.1 Estrutura

```rust
struct UnifiedDiagnostic {
    source: DiagnosticSource,
    context: ContextId,
    file: FileId,
    range: TextRange,
    severity: Severity,
    code: Option<String>,
    message: String,
    related: Vec<RelatedLocation>,
    fixes: Vec<FixAction>,
    snapshot: DocumentVersion,
    confidence: Confidence,
}
```

---

## 37. Deduplicação

O mesmo erro pode aparecer em:

- `clangd`;
- build;
- `clang-tidy`;
- compiler.

O broker deve agrupar por:

```text
arquivo
range
código
mensagem normalizada
contexto
```

UI:

```text
error: use of undeclared identifier 'device'

Sources:
  clangd
  Clang build

Context:
  Debug/Desktop
```

---

## 38. Diagnósticos por context

```text
controller.cpp:120

Debug/Desktop:
  no issue

Release/Desktop:
  warning: variable unused

Debug/ARM:
  error: type is not available for this target
```

O editor pode mostrar apenas o context ativo e deixar os demais em uma aba.

---

# Parte VIII — Fluxo completo ao abrir um projeto

## 39. Sequência

```text
1. Descobrir roots
2. Detectar build systems
3. Carregar configuração do workspace
4. Construir modelo preliminar com Tree-sitter/filesystem
5. Consultar CMake File API/Cargo Metadata/Meson
6. Construir Unified Project Graph
7. Identificar contexts
8. Validar toolchains
9. Iniciar clangd/rust-analyzer
10. Sincronizar arquivos abertos
11. Iniciar background index/cache priming
12. Descobrir tests e run targets
13. Atualizar UI progressivamente
14. Marcar workspace semanticamente pronto
```

---

## 40. Estados de prontidão

```text
Opening
Syntax Ready
Project Model Ready
Active File Semantic Ready
Navigation Ready
Workspace Indexed
Deep Analysis Ready
```

A UI pode apresentar:

```text
Workspace Intelligence

Syntax                Ready
CMake model           Ready
C++ active context    Ready
C++ global index      78%
Rust crate graph      Ready
Rust cache priming    93%
Tests                 124 discovered
Deep diagnostics      Not run
```

---

## 41. O editor nunca deve bloquear

A abertura do projeto não deve esperar a indexação completa.

```text
0–100 ms
Tree-sitter e arquivo visível

100 ms–2 s
project model inicial e language server

segundos/minutos
background indexing e cache priming

sob demanda
deep analysis
```

Esses tempos são metas internas, não garantias universais.

---

# Parte IX — Modos de uso de recursos

## 42. Eco

```text
um context ativo
background index com baixa prioridade
sem análise profunda automática
cache reduzido
secondary contexts suspensos
```

## 43. Balanced

```text
um context ativo
índice completo
cache priming
clang-tidy rápido
cargo check
secondary contexts sob demanda
```

## 44. Deep

```text
índice completo
standard library index
reference limit removido
cache ampliado
clang-tidy mais amplo
workspace cargo check
all targets válidos
proc macros e build scripts habilitados
Tree-sitter workspace index
```

## 45. Matrix

```text
múltiplas instâncias de clangd
múltiplas instâncias de rust-analyzer
comparação entre targets
diagnósticos por context
caches isolados
alta utilização de RAM e CPU
```

---

## 46. Orçamento adaptativo

O KSWE deve possuir um `ResourceGovernor`.

```text
ResourceGovernor
├── system memory
├── available memory
├── engine memory
├── UI latency
├── CPU load
├── battery state
└── thermal state
```

Políticas:

- não matar engines agressivamente;
- suspender contexts secundários antes do ativo;
- manter índices persistentes;
- reduzir threads antes de descartar caches;
- nunca comprometer salvamento;
- permitir override do usuário;
- mostrar exatamente quem consome memória.

### 46.1 Para o notebook de 24 GB

Uma política inicial possível:

```text
Reserva do sistema e outros apps: 7–9 GB
Kinein core/UI: medido dinamicamente
Motores semânticos: até 8–12 GB em Deep/Matrix
Build/debug/test: reserva temporária
```

Esses valores são um ponto de partida de produto, não uma garantia de consumo real.

---

# Parte X — Plugins e adaptadores

## 47. Não transformar cada integração em plugin pesado

Separação:

```text
Core protocol client
├── LSP
├── DAP
├── build protocol
└── test protocol

Engine adapter
├── clangd capabilities
├── rust-analyzer capabilities
├── lldb-dap capabilities
└── GDB adapter capabilities

Language contribution
├── Tree-sitter grammar
├── queries
├── snippets
├── file types
└── settings schema
```

---

## 48. Adaptador `clangd`

Responsabilidades:

- descoberta do binário;
- validação de versão;
- argumentos;
- `.clangd`;
- compilation database;
- query-driver allowlist;
- custom extensions;
- memory usage;
- crash recovery;
- index state;
- context isolation;
- logs.

Não deve:

- implementar parsing C++;
- modificar o binário silenciosamente;
- baixar uma versão sem consentimento;
- esconder o comando executado.

---

## 49. Adaptador `rust-analyzer`

Responsabilidades:

- toolchain;
- Cargo discovery;
- linked projects;
- Cargo metadata;
- feature contexts;
- target contexts;
- build script permissions;
- proc macro permissions;
- cache priming;
- runnables;
- related tests;
- macro expansion;
- memory metrics;
- logs.

---

## 50. Adaptador Tree-sitter

Responsabilidades:

- grammar version;
- ABI;
- highlights;
- locals;
- injections;
- tags;
- folding;
- indent rules;
- query tests;
- incremental updates.

---

## 51. Adaptador `lldb-dap`

Responsabilidades:

- localizar LLDB compatível;
- iniciar adapter;
- negociar capabilities;
- traduzir DebugProfile;
- source maps;
- pretty-printers;
- remote debugging;
- logs;
- crash recovery.

---

# Parte XI — O que pode ser melhor que CLion

## 52. Melhorias realistas desde cedo

### 52.1 Transparência total

Mostrar:

- engine;
- versão;
- contexto;
- comando;
- target;
- toolchain;
- memória;
- indexação;
- processos;
- rede;
- logs.

### 52.2 Context Matrix

Comparar o mesmo código em:

- host;
- embedded;
- tests;
- diferentes toolchains;
- diferentes feature sets.

### 52.3 C++ e Rust como primeira classe

Não tratar Rust como complemento isolado.

### 52.4 Local-first

- sem telemetria;
- ferramentas locais;
- downloads explícitos;
- rede controlada.

### 52.5 Fluxo target-centric

O usuário escolhe um target, e a IDE deriva:

- context;
- build;
- run;
- test;
- debug;
- deploy;
- flash.

### 52.6 Diagnósticos unificados

Uma única interface para:

- LSP;
- compilador;
- linter;
- static analyzer;
- tests.

### 52.7 Uso consciente da RAM

O usuário vê por que a RAM está sendo usada e pode escolher profundidade.

---

## 53. Onde o CLion continuará superior inicialmente

- refactorings proprietários complexos;
- décadas de tratamento de edge cases;
- análise C++ própria;
- inspeções avançadas;
- compreensão profunda de CMake script;
- integração madura de profiling e coverage;
- debuggers em diferentes plataformas;
- refactorings estruturais de build files;
- plugins e integrações comerciais;
- suporte consolidado a projetos gigantes.

O Kinein deve ser honesto sobre esses limites.

---

# Parte XII — MVP técnico

## 54. Primeiro nível utilizável

### Projeto

- abrir CMake;
- abrir Cargo;
- abrir compilation database;
- detectar Meson;
- construir target graph;
- profiles;
- toolchains.

### C/C++

- `clangd`;
- full background index;
- completion;
- hover;
- diagnostics;
- references;
- workspace symbols;
- semantic tokens;
- inlay hints;
- rename preview;
- source/header switch;
- compilation command viewer;
- memory viewer.

### Rust

- `rust-analyzer`;
- Cargo workspace;
- cache priming;
- build scripts;
- proc macros;
- completion;
- references;
- implementations;
- macro expansion;
- runnables;
- related tests;
- Cargo check.

### Tree-sitter

- highlighting;
- folding;
- outline;
- structural selection;
- syntax index.

### Debug

- `lldb-dap`;
- launch;
- attach;
- breakpoints;
- variables;
- stack;
- threads;
- evaluate;
- memory;
- disassembly.

---

## 55. Segundo nível

- multiple contexts;
- clang-tidy batch;
- Clippy profiles;
- Include Graph;
- workspace diagnostic broker;
- test explorer;
- CTest;
- Cargo tests;
- remote debugging;
- embedded target profiles;
- source maps;
- GDB adapter.

---

## 56. Terceiro nível

- Clang Static Analyzer CTU;
- unified C++/Rust FFI graph;
- project-wide safe edits;
- include cost analysis;
- custom clang-tidy checks;
- coverage;
- profiling;
- dependency visualization;
- semantic snapshots;
- static index generation;
- distributed/local index import.

---

# Parte XIII — Critérios de aceitação

## 57. Correção semântica

```text
[ ] Todos os targets reconhecidos
[ ] Todos os source files atribuídos
[ ] Todas as compilation databases validadas
[ ] Todos os Cargo workspaces carregados
[ ] Feature context visível
[ ] Target triple visível
[ ] Sysroot visível
[ ] Index progress real
[ ] References completas após warm-up
[ ] Rename com preview e rollback
```

---

## 58. Responsividade

Metas iniciais:

```text
Tree-sitter na viewport: sem bloqueio perceptível
UI thread: nunca aguardar LSP
Completion: cancelável
Hover: cancelável
Workspace search: streaming
References: partial results quando possível
Indexing: sempre background
```

---

## 59. Resiliência

```text
[ ] clangd pode reiniciar sem reiniciar a IDE
[ ] rust-analyzer pode reiniciar sem perder arquivos
[ ] lldb-dap pode falhar sem derrubar a IDE
[ ] plugin host isolado
[ ] logs disponíveis
[ ] loops de crash detectados
[ ] safe mode disponível
[ ] caches podem ser limpos seletivamente
```

---

## 60. Observabilidade

```text
[ ] memória por engine
[ ] CPU por engine
[ ] estado de indexação
[ ] requests lentas
[ ] context ativo
[ ] comandos executados
[ ] origem do diagnóstico
[ ] rede por processo
[ ] versão de ferramenta
```

---

# Parte XIV — Roadmap recomendado

## Fase A — Fundamentos do cliente

- LSP 3.17;
- DAP;
- process supervisor;
- cancellation;
- progress;
- partial results;
- semantic token delta;
- transactional workspace edits;
- logs estruturados.

## Fase B — Project Model

- CMake File API;
- compile commands;
- Cargo Metadata;
- Meson introspection;
- target graph;
- toolchains;
- contexts.

## Fase C — Deep C/C++

- background index completo;
- `clangd` extensions;
- compile context viewer;
- memory viewer;
- Include Cleaner;
- clang-tidy;
- context matrix.

## Fase D — Deep Rust

- crate graph;
- cache priming;
- feature contexts;
- targets;
- proc macro permissions;
- macro viewer;
- related tests;
- Clippy.

## Fase E — Debug

- lldb-dap;
- target-derived profiles;
- remote;
- memory;
- disassembly;
- mixed C++/Rust.

## Fase F — Workspace Intelligence

- diagnostic broker;
- syntax index;
- semantic graph;
- test graph;
- dependency graph;
- resource governor.

## Fase G — Análise profunda

- CTU;
- CodeChecker integration;
- advanced refactor previews;
- multi-context compatibility analysis;
- embedded workflows.

---

# Parte XV — Conclusão

O Kinein Vectis pode alcançar um fluxo profissional profundo sem criar um compilador ou um language server próprio.

O caminho correto é:

```text
Tree-sitter
→ resposta sintática imediata

clangd / rust-analyzer
→ semântica de linguagem profunda

CMake / Cargo / Meson
→ verdade estrutural do projeto

compiler / clang-tidy / Clippy / static analyzer
→ verdade de build e qualidade

lldb-dap / GDB
→ execução e depuração

Kinein Semantic Workspace Engine
→ coordenação, contexto, UX, segurança e transparência
```

A economia de RAM do núcleo do Kinein deve ser usada para:

- indexar integralmente;
- aquecer caches;
- manter contextos múltiplos;
- executar análises profundas;
- preservar resultados;
- reduzir latência.

Ela não deve ser usada para duplicar a AST interna dos motores.

A meta de produto recomendada é:

> O Kinein não precisa inicialmente ser tão profundo quanto o CLion em todos os refactorings. Ele deve ser mais claro, previsível, integrado e moderno na forma como C++, Rust, targets, toolchains, build, análise e depuração trabalham juntos.

Esse objetivo é tecnicamente viável e cria uma identidade própria para a IDE.

---

# Referências oficiais

[^1]: [JetBrains — CLion language engines, 2026.1](https://www.jetbrains.com/help/clion/clion-language-engines.html)

[^2]: [LLVM — The clangd index](https://clangd.llvm.org/design/indexing)

[^3]: [rust-analyzer — Architecture](https://rust-analyzer.github.io/book/contributing/architecture.html)

[^4]: [CMake — File API](https://cmake.org/cmake/help/latest/manual/cmake-file-api.7.html)

[^5]: [LLVM — clangd compile commands](https://clangd.llvm.org/design/compile-commands)

[^6]: [Cargo Book — cargo metadata](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html)

[^7]: [Meson — IDE integration](https://mesonbuild.com/IDE-integration.html)

[^8]: [LLVM — clangd FAQ](https://clangd.llvm.org/faq)

[^9]: [LLVM — clang-tidy](https://clang.llvm.org/extra/clang-tidy/)

[^10]: [LLVM — Clang Static Analyzer Cross Translation Unit](https://clang.llvm.org/docs/analyzer/user-docs/CrossTranslationUnit.html)

[^11]: [LLVM — Include Cleaner](https://clangd.llvm.org/design/include-cleaner)

[^12]: [LLVM — clangd protocol extensions](https://clangd.llvm.org/extensions)

[^13]: [rust-analyzer — Configuration](https://rust-analyzer.github.io/book/configuration.html)

[^14]: [rust-analyzer — Features](https://rust-analyzer.github.io/book/features.html)

[^15]: [Tree-sitter — Introduction](https://tree-sitter.github.io/)

[^16]: [Tree-sitter — Code Navigation Systems](https://tree-sitter.github.io/tree-sitter/4-code-navigation.html)

[^17]: [Tree-sitter — Syntax Highlighting, locals and injections](https://tree-sitter.github.io/tree-sitter/3-syntax-highlighting.html)

[^18]: [Language Server Protocol Specification 3.17](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/)

[^19]: [LLVM — clangd threads and request handling](https://clangd.llvm.org/design/threads)

[^20]: [LLVM — Getting started with lldb-dap](https://lldb.llvm.org/use/lldbdap.html)

[^21]: [LLVM — LLDB remote debugging](https://lldb.llvm.org/use/remote.html)
