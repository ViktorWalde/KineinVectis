# Kinein Vectis — Parte 10.1: Resource-on-Demand, Inspections e Refatoração Profunda

> **Tipo:** correção e evolução da Parte 10.  
> **Escopo:** performance sem limite artificial, arquitetura orientada à demanda por padrão, uso total do LSP quando necessário, comportamento visual inspirado em IDEs profissionais, inspeções em tempo real, contexto profundo de projeto e refatoração progressiva.  
> **Decisão central:** a Kinein não terá fronteira rígida de memória como “2 GB”. A IDE deve ser performática por arquitetura, usando recursos sob demanda e ativando profundidade quando houver valor real.

---

## 1. Correção oficial: sem fronteira rígida de memória

A Kinein Vectis não deve ter uma regra como:

```text
passou de 2 GB → limitar recurso
```

Isso não faz sentido para uma IDE profissional.

Projetos grandes, `clangd`, `rust-analyzer`, debug, indexação, terminais e logs podem exigir bastante memória. Se o usuário precisa de análise profunda, a IDE deve usar os recursos necessários.

A regra correta é:

```text
Não existe hard cap de memória.
Existe arquitetura orientada à demanda por padrão.
```

Ou:

```text
A Kinein deve usar poder quando há valor,
e reduzir desperdício quando o recurso não está em uso.
```

---

## 2. Resource-on-Demand como padrão da IDE

A Kinein deve nascer com esta natureza:

```text
recurso pesado só entra quando há contexto;
recurso ocioso reduz atividade;
painel fechado não deve continuar caro;
logs antigos não ficam inteiros na memória;
LSP pode usar força total quando necessário;
a UI permanece responsiva.
```

Isso não é “modo limitado”.  
É engenharia de performance.

A Kinein não deve ser leve porque faz pouco.  
Ela deve ser leve porque faz o necessário no momento certo.

---

## 3. Princípio final de performance

```text
Kinein não limita poder.
Kinein evita desperdício.
```

A IDE deve sempre se perguntar:

```text
este recurso está visível?
está sendo usado?
está ajudando agora?
pode ser carregado depois?
pode ser suspenso?
pode ser cacheado em disco?
pode ser recalculado sob demanda?
```

---

## 4. Trade-offs aceitos

Toda escolha tem custo.

A Kinein deve aceitar estes trade-offs de forma explícita:

```text
mais inteligência semântica → mais CPU/RAM;
mais contexto de projeto → mais cache/estado;
mais refatoração profunda → mais análise;
mais UX visual → mais componentes;
mais integração com build system → mais complexidade.
```

Mas isso não invalida a proposta.

A solução é:

```text
ativar profundidade conforme necessidade;
mostrar o que está acontecendo;
permitir controle do usuário;
manter UI responsiva;
evitar trabalho invisível sem valor.
```

---

## 5. Uso total do LSP quando fizer sentido

A Kinein deve permitir usar o poder total de:

```text
clangd
rust-analyzer
debug adapters
indexação semântica
símbolos
diagnósticos
refatorações
```

Quando o usuário está trabalhando intensamente em C/C++ ou Rust, o LSP deve trabalhar de verdade.

Exemplo:

```text
usuário abre projeto C++ grande
usuário navega por código
usuário usa referências
usuário usa rename
usuário usa refactor
```

Nesse caso, a Kinein não deve “economizar demais” e atrapalhar.  
Ela deve ativar a inteligência necessária.

---

## 6. O que significa “sob demanda” na prática

### 6.1 Abrir projeto

```text
abrir UI rapidamente
detectar build systems
mostrar editor
iniciar scan essencial
adiar serviços pesados
```

### 6.2 Abrir arquivo C/C++

```text
Tree-sitter imediato
clangd inicia se houver contexto suficiente
diagnósticos aparecem conforme chegam
```

### 6.3 Abrir arquivo Rust

```text
Tree-sitter imediato
rust-analyzer inicia se Cargo estiver ativo
cargo metadata roda como job
```

### 6.4 Usar refatoração

```text
carregar contexto semântico mais profundo
buscar referências
montar preview
validar alterações
aplicar com confirmação
```

### 6.5 Fechar projeto ou trocar contexto

```text
parar jobs irrelevantes
liberar contextos temporários
reduzir watchers
limpar previews antigos
manter cache útil em disco
```

---

## 7. Comportamento visual desejado

A Kinein deve buscar comportamento visual profissional semelhante ao que boas IDEs fazem:

```text
avisar em tempo real;
não gritar;
mostrar o erro no lugar certo;
oferecer ação contextual;
permitir aprofundar;
não interromper o fluxo;
não abrir modal para tudo;
não esconder o problema.
```

Inspiração de comportamento, não cópia visual:

```text
JetBrains-like na assistência contextual.
Kinein-like na identidade visual.
```

---

## 8. Sistema de Inspections

A Kinein deve ter um conceito central chamado:

```text
Inspections
```

Inspections são verificações contínuas e contextuais.

Exemplos:

```text
erro de compilação;
warning do compilador;
diagnóstico do clangd;
diagnóstico do rust-analyzer;
CMakePresets ausente;
compile_commands.json ausente;
toolchain incompleta;
target sem run config;
include não resolvido;
dependência Cargo ausente;
CMake cache stale;
debugger não configurado;
serial permission issue;
```

---

## 9. Níveis de severidade visual

A UI deve usar severidades claras:

```text
Info
Weak Warning
Warning
Error
Critical
```

Mas sem exagero visual.

### 9.1 Info

Uso:

```text
toolchain detectada;
LSP indexando;
Project Health atualizando;
```

Visual:

```text
texto discreto;
ícone pequeno;
status bar.
```

### 9.2 Warning

Uso:

```text
compile_commands ausente;
debugger não configurado;
include directory suspeito;
```

Visual:

```text
âmbar discreto;
Problems;
badge no Project Health.
```

### 9.3 Error

Uso:

```text
build falhou;
CMake configure falhou;
Cargo check falhou;
símbolo não resolvido;
```

Visual:

```text
linha no editor;
Problems;
Build panel;
status bar.
```

### 9.4 Critical

Uso:

```text
workspace inválido;
permissão negada;
falha de processo essencial;
configuração corrompida.
```

Visual:

```text
banner compacto ou painel claro;
ainda evitar modal se possível.
```

---

## 10. Feedback em tempo real sem intrusão

A Kinein deve avisar em tempo real, mas com camadas.

```text
nível 1: gutter do editor
nível 2: underline no código
nível 3: Problems panel
nível 4: status bar
nível 5: Project Health
nível 6: action popup sob demanda
```

Evitar:

```text
modal automático;
toast repetitivo;
painel abrindo sozinho toda hora;
som;
animação chamativa;
IA se oferecendo sem pedido.
```

---

## 11. Project Context Model

Para aprofundar o contexto do projeto, a Kinein precisa manter um modelo interno.

```text
Project Context Model
```

Ele deve representar:

```text
workspace;
build systems ativos;
targets;
toolchains;
presets;
run configs;
debug configs;
source files;
include paths;
dependencies;
diagnostics;
symbols;
build history;
test history;
language servers;
Configuration Actions disponíveis;
Project Health;
```

Esse modelo é o que permite a IDE ajudar visualmente sem ser aleatória.

---

## 12. Project Context não é IA

Esse contexto é determinístico.

```text
não é chat;
não é IA;
não é agente;
não é adivinhação.
```

É um grafo técnico do projeto.

Exemplo:

```text
target app
  usa src/main.cpp
  depende de motor_driver
  compila com clang++
  usa preset debug
  gera binário build/debug/app
  tem run config Local Debug
  tem clangd associado via compile_commands.json
```

---

## 13. Configuration Graph

A Parte 8.1 já definiu o Configuration Graph.  
Aqui ele ganha papel maior.

Ele deve responder perguntas como:

```text
por que clangd não tem contexto?
por que target não aparece em Run?
por que debug não inicia?
por que include não resolve?
qual compiler está ativo?
qual preset gerou esse build dir?
qual Cargo workspace está ativo?
essa action é segura neste projeto?
```

---

## 14. Refatoração: visão realista

Refatoração profunda é um objetivo importante, mas deve ser progressivo.

Não tentar criar tudo no início.

A Kinein deve começar usando:

```text
clangd
rust-analyzer
LSP rename
LSP references
LSP code actions
formatters
Tree-sitter para estrutura local
```

E com o tempo construir camadas próprias ao redor.

---

## 15. Níveis de refatoração

### 15.1 Nível 1 — LSP-native

Base inicial.

```text
rename symbol
go to definition
find references
organize includes/imports
format file
code actions
quick fixes
```

Fonte:

```text
clangd
rust-analyzer
```

---

### 15.2 Nível 2 — IDE-assisted

A Kinein usa LSP + Project Context.

```text
rename com preview melhor;
mover arquivo e atualizar CMake target;
adicionar source ao target;
criar header/source pair;
extrair run config;
linkar biblioteca ausente;
corrigir include path;
adicionar dependência Cargo;
```

Fonte:

```text
LSP + Configuration Actions + Project Context Model
```

---

### 15.3 Nível 3 — Build-aware refactoring

A Kinein considera o build system.

```text
mover classe C++ e atualizar CMakeLists;
renomear target;
separar library/executable;
extrair static library;
converter source em target separado;
ajustar include directories;
atualizar presets relacionados;
```

Fonte:

```text
CMake model + clangd + Project Graph
```

---

### 15.4 Nível 4 — Workspace-aware refactoring

Para projetos maiores.

```text
refatorar múltiplos módulos;
ajustar dependências entre targets;
propagar mudança entre CMake e Cargo;
atualizar run/debug configs;
validar build depois;
```

Fonte:

```text
Project Context Model + Build Graph + LSP + Jobs
```

---

## 16. Preview obrigatório para refatoração

Toda refatoração não trivial deve ter preview.

```text
arquivos afetados;
símbolos afetados;
diff;
risco;
comando/ação equivalente;
validação posterior;
rollback possível.
```

A Kinein nunca deve aplicar refatoração profunda no escuro.

---

## 17. Refactoring UI

A UI de refatoração deve ser parecida com o estilo das Configuration Actions:

```text
ação clara;
descrição curta;
impacto;
preview lateral;
lista de arquivos;
checkboxes;
risco;
botão Apply;
botão Cancel;
validação.
```

Exemplo:

```text
Refactor: Move source to new static library

O que faz:
Cria uma biblioteca estática e move arquivos selecionados para um novo target.

Afeta:
CMakeLists.txt
src/motor.cpp
include/motor.hpp

Validação:
cmake --preset debug
cmake --build --preset debug
```

---

## 18. Inspections + Refactoring

Inspections devem oferecer ações contextuais.

Exemplo:

```text
include not found
```

Ações:

```text
Open include path settings
Add include directory
Inspect target
Open CMakeLists
Open docs
```

Exemplo:

```text
undefined reference
```

Ações:

```text
Open linker error
Add target_link_libraries
Inspect target dependencies
Open AI Terminal
```

Exemplo:

```text
Cargo unresolved crate
```

Ações:

```text
Add dependency
Open Cargo.toml
Run cargo check
Open docs
```

---

## 19. Erros em tempo real

A Kinein deve mostrar erros em tempo real a partir de múltiplas fontes.

```text
LSP diagnostics
compiler diagnostics
CMake configure diagnostics
Cargo diagnostics
runtime/debug diagnostics
Project Health diagnostics
```

Mas precisa unificar isso.

---

## 20. Unified Diagnostics Model

Criar um modelo unificado:

```json
{
  "id": "diag_001",
  "source": "clangd",
  "severity": "error",
  "message": "Use of undeclared identifier",
  "file": "src/main.cpp",
  "range": {
    "start_line": 12,
    "start_col": 8,
    "end_line": 12,
    "end_col": 20
  },
  "actions": [
    "language.quickFix",
    "configActions.add_include_directory"
  ]
}
```

Fontes possíveis:

```text
clangd
rust-analyzer
cmake
cargo
compiler
linker
debugger
project-health
setup-intelligence
```

---

## 21. Problems panel

O painel Problems deve ser poderoso, mas limpo.

Agrupar por:

```text
file
severity
source
target
build profile
```

Filtros:

```text
Errors
Warnings
Info
Current file
Current target
Build only
LSP only
CMake only
Cargo only
```

Ações:

```text
Open
Explain
Fix
Open docs
Open Configuration Action
Open AI Terminal
```

A ação de IA continua secundária.

---

## 22. Status visual no editor

No editor:

```text
gutter icons;
underline discreto;
hover com descrição;
quick fix lightbulb;
breadcrumb com contexto;
status de LSP;
```

Não usar:

```text
pop-up gigante;
janela cobrindo código;
animações chamativas;
overlay permanente.
```

---

## 23. Deep Project Awareness

Com o tempo, a Kinein deve entender:

```text
este arquivo pertence a qual target?
este target usa qual compiler?
este arquivo participa de qual preset?
este erro veio de qual configuração?
qual include path deveria resolver isso?
qual dependency falta?
qual run config executa esse binário?
```

Isso é mais importante do que “IA”.

---

## 24. Arquitetura para inteligência profunda

Camadas:

```text
Tree-sitter Layer
  estrutura local rápida

LSP Layer
  semântica por linguagem

Build System Layer
  CMake/Cargo/Presets/Targets

Project Context Layer
  grafo do projeto

Inspection Layer
  problemas, severidade, ações

Refactoring Layer
  preview, diff, apply, rollback

UI Layer
  visual discreto e contextual
```

---

## 25. Performance desta inteligência

Essa inteligência não pode pesar sempre.

Estratégia:

```text
Project Context básico sempre;
Project Context profundo sob demanda;
LSP profundo quando arquivo/target está ativo;
refactoring analysis só ao chamar refatoração;
build graph atualizado por eventos;
diagnostics agregados;
UI renderiza só o visível.
```

---

## 26. JetBrains-like behavior, Kinein-like implementation

O objetivo não é copiar UI.

O objetivo é capturar comportamento:

```text
saber o que está errado;
mostrar no local certo;
dar ação útil;
permitir aprofundar;
não atrapalhar digitação;
não transformar ajuda em barulho;
ter refatorações confiáveis;
mostrar preview;
integrar build system com editor.
```

A identidade visual continua Kinein:

```text
grafite escuro;
âmbar como guia;
painéis calmos;
densidade confortável;
sem marketing dentro da IDE.
```

---

## 27. Performance Profiles revisados

Os perfis não devem ser sobre “limitar poder”.  
Devem ser sobre quando ativar poder.

```text
Responsive
Balanced
Full Intelligence
Large Workspace
```

### 27.1 Responsive

```text
prioriza UI rápida;
LSP mais sob demanda;
menos indexação inicial;
bom para hardware fraco.
```

### 27.2 Balanced

```text
equilíbrio padrão;
LSP inicia quando contexto aparece;
Project Health incremental;
Configuration Actions sob demanda.
```

### 27.3 Full Intelligence

```text
usa mais análise;
inicia serviços semânticos mais cedo;
melhor para máquinas fortes;
bom para refatoração e navegação intensiva.
```

### 27.4 Large Workspace

```text
respeita projetos enormes;
file tree lazy;
watchers reduzidos;
indexação controlada;
diagnósticos agregados.
```

---

## 28. O usuário deve entender o que está ativo

A Kinein pode mostrar um menu discreto:

```text
Background Services
├── clangd: running
├── rust-analyzer: sleeping
├── Project Health: updated
├── CMake cache: stale
├── Tree-sitter: active
└── Jobs: 1 running
```

Isso gera confiança.

---

## 29. Quando usar muita memória é aceitável

É aceitável usar muita memória quando:

```text
usuário abriu projeto grande;
LSP está indexando;
refatoração profunda foi chamada;
múltiplos terminais estão abertos;
build logs estão ativos;
debug session está rodando;
Full Intelligence está ligado.
```

O importante é:

```text
ser explicável;
ser observável;
ser liberável;
não travar UI.
```

---

## 30. Quando usar muita memória é desperdício

É desperdício quando:

```text
painel fechado continua renderizando;
log antigo continua inteiro em RAM;
LSP de linguagem não usada está ativo;
contextos antigos de IA ficam vivos;
preview descartado continua em memória;
file tree carrega diretórios ignorados;
diagnósticos obsoletos acumulam.
```

Esses casos devem ser corrigidos pela arquitetura.

---

## 31. MVP desta estratégia

No MVP:

```text
Tree-sitter rápido
clangd/rust-analyzer status
LSP por escopo ativo
Problems básico
Unified Diagnostics básico
Project Health básico
Configuration Actions com ações contextuais
Preview obrigatório
Background Jobs menu simples
Experience Mode
Performance Profile simples: Responsive / Balanced / Full Intelligence
```

---

## 32. Pós-MVP

```text
Inspections mais ricas
Quick fixes conectadas a Configuration Actions
Refactor preview melhor
Build-aware refactoring inicial
Deep Project Context
Large Workspace mode
Background Services detalhado
docs conectadas às actions
```

---

## 33. Futuro

```text
refatorações profundas multi-target
CMake graph avançado
workspace-aware refactoring
análise de dependências entre targets
rename target/build-aware
extract library
move module
integration tests após refactor
rollback completo
```

---

## 34. Critérios de aceite

```text
[ ] Não existe hard cap de RAM.
[ ] Resource-on-Demand é padrão da IDE, não modo especial.
[ ] LSP pode usar força total quando necessário.
[ ] LSP não inicia fora de contexto sem motivo.
[ ] UI permanece responsiva durante indexação.
[ ] Problems mostra erros em tempo real.
[ ] Inspections têm severidade e ações.
[ ] Project Context Model está definido.
[ ] Refatorações começam pelo LSP-native.
[ ] Refatorações profundas têm preview.
[ ] Configuration Actions ajudam a resolver diagnósticos.
[ ] Expert Mode continua não invasivo.
[ ] Full Intelligence existe para máquinas/projetos que precisam.
```

---

## 35. Resumo executivo

A Kinein deve seguir esta regra:

```text
não limitar poder;
ativar poder sob demanda;
explicar o que está ativo;
liberar o que não agrega;
manter a UI responsiva.
```

Ela pode, com o tempo, se aproximar do comportamento de IDEs profissionais:

```text
erro em tempo real;
ação contextual;
refatoração confiável;
contexto profundo;
build system integrado;
sem ser invasiva.
```

Frase final:

```text
A Kinein não deve ser leve porque faz pouco.
Ela deve ser leve porque só faz o necessário,
e poderosa porque sabe ativar profundidade quando o usuário precisa.
```
