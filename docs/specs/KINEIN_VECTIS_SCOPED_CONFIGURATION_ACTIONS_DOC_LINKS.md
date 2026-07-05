# Kinein Vectis — Parte 9.2: Scoped Configuration Actions e Documentation Links

> **Tipo:** complemento direto da Parte 9.1.  
> **Escopo:** melhorar a UX das Configuration Actions para não mostrar ações irrelevantes, filtrar por sistema de build/projeto selecionado e preparar links de documentação por ação.  
> **Decisão central:** a lista de Configuration Actions deve ser **contextual**, não uma lista global despejada na tela.

---

## 1. Correção importante de conceito

Na conversa informal pode parecer que “o compilador selecionado” define se aparecem ações CMake ou Cargo. Tecnicamente, o que define a lista principal de ações é o **tipo de projeto / sistema de build ativo**.

```text
CMake project  → ações CMake
Cargo project  → ações Cargo
Mixed project  → ações CMake + Cargo
```

O compilador/toolchain ainda importa, mas em outro nível:

```text
CMake + Clang/GCC → ações CMake disponíveis com validação do compilador selecionado.
Cargo + Rust     → ações Cargo disponíveis com validação do cargo/rustc/rust-analyzer.
Mixed            → ações separadas para CMake e Cargo, respeitando cada toolchain.
```

Então a regra correta é:

```text
A lista de Configuration Actions é filtrada pelo Build System ativo.
A disponibilidade de cada ação é validada pela Toolchain selecionada.
```

---

## 2. Regra principal de UX

```text
Nunca mostrar ao usuário uma lista gigante de ações irrelevantes.
```

Se o projeto é CMake, o usuário não deve ver ações Cargo ocupando espaço.  
Se o projeto é Cargo, o usuário não deve ver ações CMake.  
Se o projeto é misto, aí sim a UI mostra CMake e Cargo juntos, bem separados.

---

## 3. Modos de escopo

### 3.1 CMake-only

Ativado quando:

```text
- projeto criado como C/C++ com CMake;
- pasta contém CMakeLists.txt;
- CMakePresets.json detectado;
- usuário escolheu CMake no Project Wizard;
- usuário marcou apenas CMake em Project Settings.
```

Mostrar:

```text
Configuration Actions
├── CMake Basic
├── CMake Intermediate
├── CMake Advanced
├── Run/Debug
├── Targets
└── Toolchain/CMake Setup
```

Não mostrar por padrão:

```text
Cargo
Rust workspace
Cargo features
Cargo dependencies
```

---

### 3.2 Cargo-only

Ativado quando:

```text
- projeto criado como Rust com Cargo;
- pasta contém Cargo.toml;
- cargo metadata funciona;
- usuário escolheu Cargo no Project Wizard;
- usuário marcou apenas Cargo em Project Settings.
```

Mostrar:

```text
Configuration Actions
├── Cargo Basic
├── Cargo Workspace
├── Cargo Features
├── Run/Debug
├── Targets
└── Rust Toolchain Setup
```

Não mostrar por padrão:

```text
CMake
CMakePresets
target_link_libraries
include directories
CMake cache
```

---

### 3.3 Mixed CMake + Cargo

Ativado quando:

```text
- projeto contém CMakeLists.txt e Cargo.toml;
- usuário escolheu Mixed C++/Rust;
- workspace tem subprojetos CMake e Cargo;
- usuário ativou ambos em Project Settings.
```

Mostrar o layout completo:

```text
Configuration Actions
├── CMake
│   ├── Basic
│   ├── Intermediate
│   └── Advanced
├── Cargo
│   ├── Basic
│   ├── Workspace
│   └── Features
├── Run/Debug
├── Targets
└── Advanced
```

Esse é o caso em que faz sentido o layout denso com CMake e Cargo juntos.

---

## 4. Seleção explícita no Project Wizard

No Project Wizard, o usuário deve escolher o tipo de projeto.

```text
Project Type:
[ C/C++ with CMake ]
[ Rust with Cargo ]
[ Mixed C++/Rust ]
[ Existing Project ]
```

Essa escolha define:

```text
- quais Configuration Actions aparecem;
- quais health checks são priorizados;
- quais services são ativados;
- quais toolchains serão validadas;
- quais arquivos serão gerados;
- quais docs serão sugeridas.
```

---

## 5. Detecção em projeto existente

Ao abrir uma pasta existente:

```text
CMakeLists.txt encontrado       → CMake detected
Cargo.toml encontrado           → Cargo detected
ambos encontrados               → Mixed detected
nenhum encontrado               → Unknown / Plain folder
```

A UI deve mostrar:

```text
Detected build systems:
[✓] CMake
[ ] Cargo
```

Ou:

```text
Detected build systems:
[✓] CMake
[✓] Cargo
```

O usuário pode corrigir manualmente:

```text
Project Settings > Build Systems
```

Exemplo:

```text
Detected: CMake + Cargo
Active in Kinein:
[✓] CMake
[✓] Cargo
```

Ou se o Cargo.toml for só de ferramenta auxiliar:

```text
[✓] CMake
[ ] Cargo
```

---

## 6. UI: Build System Scope Switcher

O painel de Configuration Actions deve ter um seletor de escopo no topo.

### 6.1 CMake-only

```text
Configuration Actions
Scope: CMake
```

Sem tabs de Cargo.

### 6.2 Cargo-only

```text
Configuration Actions
Scope: Cargo
```

Sem tabs de CMake.

### 6.3 Mixed

```text
Configuration Actions
Scope: [ All ] [ CMake ] [ Cargo ] [ Run/Debug ] [ Targets ]
```

O usuário pode filtrar rapidamente.

---

## 7. Contexto do arquivo ativo

Além do escopo global, a IDE deve usar contexto local.

### 7.1 Se arquivo ativo é `CMakeLists.txt`

Priorizar:

```text
CMake actions
```

### 7.2 Se arquivo ativo é `Cargo.toml`

Priorizar:

```text
Cargo actions
```

### 7.3 Se arquivo ativo é `.cpp`, `.hpp`, `.c`, `.h`

Priorizar:

```text
Add source to target
Add include directory
Add linked library
Create run config
Create debug config
```

### 7.4 Se arquivo ativo é `.rs`

Priorizar:

```text
Add dependency
Add feature
Run cargo check
Create run config
```

Isso não muda o escopo do projeto, apenas ordena a lista.

---

## 8. Estados de ação por escopo

Cada ação deve ter estado:

```text
Visible
Hidden by scope
Unavailable
Partially available
Available
Recommended
```

Exemplo CMake-only:

```text
Cargo: Add dependency
state: hidden_by_scope
reason: Cargo is not active for this workspace.
```

Exemplo Mixed:

```text
Cargo: Add dependency
state: available
reason: Cargo.toml detected and Cargo toolchain is valid.
```

Exemplo CMake com toolchain quebrada:

```text
CMake: Add executable
state: partially_available
reason: CMake project detected, but no valid C++ compiler selected.
```

---

## 9. Configuration Actions não são “menu global”

A lista não deve parecer um menu universal com tudo.  
Ela deve parecer uma caixa de ferramentas contextual.

```text
Projeto CMake:
mostre ferramentas CMake.

Projeto Cargo:
mostre ferramentas Cargo.

Projeto misto:
mostre as duas caixas de ferramentas.
```

Essa regra reduz densidade sem sacrificar poder.

---

## 10. Documentation Links por ação

Futuramente, cada Configuration Action deve ter link de documentação.

### 10.1 Objetivo

O usuário deve conseguir entender a ação sem sair procurando manualmente.

Exemplo:

```text
Add target_link_libraries

O que faz:
Vincula uma biblioteca a um target CMake.

Docs:
[Documentação oficial: target_link_libraries]
```

---

## 11. Como documentar sem poluir a UI

O link não deve virar um elemento gigante na lista.

### 11.1 Na lista

Mostrar discretamente:

```text
Add target_link_libraries                         docs
Vincula uma biblioteca existente ao target.
CMakeLists.txt · medium
```

### 11.2 No preview lateral

Mostrar melhor:

```text
Documentation
- Official CMake: target_link_libraries
- Kinein guide: Linking libraries in CMake
```

### 11.3 Tooltip

Ao passar o mouse em `docs`:

```text
Open official documentation for this CMake command.
```

---

## 12. Tipos de documentação

Cada ação pode ter múltiplas fontes.

```text
official_doc
kinein_guide
project_doc
local_man_page
example
```

Exemplo:

```json
{
  "docs": [
    {
      "kind": "official_doc",
      "title": "CMake: target_link_libraries",
      "url": "official-doc-placeholder",
      "priority": 1
    },
    {
      "kind": "kinein_guide",
      "title": "Como linkar bibliotecas em CMake moderno",
      "path": "docs/kinein/cmake/linking.md",
      "priority": 2
    }
  ]
}
```

No MVP, o campo pode existir mesmo que o link real ainda não esteja preenchido.

---

## 13. Documentação oficial vs documentação local

Ordem recomendada:

```text
1. documentação do próprio projeto;
2. guia curto da Kinein;
3. documentação oficial;
4. exemplos locais;
5. internet, se habilitado futuramente.
```

Como a Kinein é local-first, o ideal é permitir cache local de docs depois.

---

## 14. Documentation Registry

Para não espalhar links no código, criar um registry.

```text
docs/
  registry/
    cmake-actions.json
    cargo-actions.json
    run-debug-actions.json
```

Exemplo:

```json
{
  "cmake.add_target_link_libraries": {
    "official": {
      "title": "target_link_libraries",
      "provider": "CMake",
      "url_key": "cmake.target_link_libraries"
    },
    "kinein": {
      "title": "Linking libraries in modern CMake",
      "path": "docs/kinein/cmake/linking.md"
    }
  }
}
```

O `url_key` pode ser resolvido por uma camada de documentação.

---

## 15. Evitar dependência de internet

A Kinein não precisa abrir internet automaticamente.

Opções:

```text
Open online docs
Open cached docs
Open Kinein guide
Copy command name
```

Se offline:

```text
Documentation unavailable offline.
[Open Kinein guide] [Copy command name]
```

---

## 16. Action metadata revisado

A action definition deve ganhar campos de escopo e docs.

```json
{
  "id": "cmake.add_target_link_libraries",
  "title": "Add target_link_libraries",
  "scope": ["cmake"],
  "category": "CMake Basic",
  "level": "basic",
  "description": "Vincula uma biblioteca existente a um target CMake.",
  "affects": ["CMakeLists.txt"],
  "risk": "medium",
  "requires": ["cmake_project", "existing_target"],
  "preview_supported": true,
  "rollback_supported": true,
  "docs": [
    {
      "kind": "official_doc",
      "title": "CMake target_link_libraries",
      "url_key": "cmake.target_link_libraries"
    }
  ]
}
```

Cargo:

```json
{
  "id": "cargo.add_dependency",
  "title": "Add dependency",
  "scope": ["cargo"],
  "category": "Cargo Basic",
  "description": "Adiciona uma crate em [dependencies] no Cargo.toml.",
  "affects": ["Cargo.toml"],
  "risk": "medium",
  "requires": ["cargo_project"],
  "docs": [
    {
      "kind": "official_doc",
      "title": "Cargo dependencies",
      "url_key": "cargo.dependencies"
    }
  ]
}
```

---

## 17. Active Build System Model

Adicionar entidade no Core:

```json
{
  "workspace_id": "workspace_001",
  "detected_build_systems": ["cmake", "cargo"],
  "active_build_systems": ["cmake"],
  "primary_build_system": "cmake"
}
```

Exemplo Mixed:

```json
{
  "workspace_id": "workspace_002",
  "detected_build_systems": ["cmake", "cargo"],
  "active_build_systems": ["cmake", "cargo"],
  "primary_build_system": "cmake"
}
```

---

## 18. Project Settings: Build Systems

Tela:

```text
Project Settings > Build Systems

Detected:
[✓] CMake     CMakeLists.txt
[✓] Cargo     Cargo.toml

Active in Kinein:
[✓] CMake
[ ] Cargo

Primary:
CMake
```

Isso resolve projetos que têm Cargo apenas para ferramenta auxiliar ou CMake apenas para wrapper.

---

## 19. JSON-RPC adicional

### 19.1 Obter escopo ativo

```json
{
  "method": "buildSystems.getActive",
  "params": {
    "workspace_id": "workspace_001"
  }
}
```

### 19.2 Atualizar escopo ativo

```json
{
  "method": "buildSystems.setActive",
  "params": {
    "workspace_id": "workspace_001",
    "active": ["cmake"],
    "primary": "cmake"
  }
}
```

### 19.3 Listar ações com escopo

```json
{
  "method": "configActions.list",
  "params": {
    "workspace_id": "workspace_001",
    "scope": "active_build_systems",
    "include_hidden_by_scope": false
  }
}
```

### 19.4 Abrir documentação

```json
{
  "method": "docs.openForAction",
  "params": {
    "action_id": "cmake.add_target_link_libraries",
    "preferred_source": "official_doc"
  }
}
```

---

## 20. UX para lista densa

Mesmo filtrada por escopo, a lista ainda será densa.  
Então aplicar estas regras:

```text
altura de item compacta;
descrição de uma linha;
badges pequenos;
preview lateral para detalhe;
busca sempre visível;
atalhos por teclado;
categorias colapsáveis;
recentes/favoritos;
sem ícones grandes;
sem cards enormes;
sem banners.
```

---

## 21. CMake-only layout

```text
Configuration Actions
Scope: CMake

Search...

Categories:
- Favorites
- Basic
- Intermediate
- Advanced
- Run/Debug
- Targets

Actions:
- Add executable
- Add static library
- Add target_link_libraries
- Add include directory
- Enable compile_commands.json
```

---

## 22. Cargo-only layout

```text
Configuration Actions
Scope: Cargo

Search...

Categories:
- Favorites
- Dependencies
- Features
- Workspace
- Run/Debug
- Targets

Actions:
- Add dependency
- Add dev-dependency
- Add feature
- Add workspace member
- Run cargo check
```

---

## 23. Mixed layout

```text
Configuration Actions
Scope: [All] [CMake] [Cargo] [Run/Debug] [Targets]

Categories:
- Favorites
- CMake
- Cargo
- Run/Debug
- Targets
- Advanced

Actions:
- CMake: Add target_link_libraries
- CMake: Enable compile_commands.json
- Cargo: Add dependency
- Cargo: Add feature
```

O layout do SVG anterior é adequado para esse modo Mixed.

---

## 24. Integração com Experience Modes

### 24.1 Guided

```text
Mostra ações do escopo ativo automaticamente.
Mostra descrições curtas.
Mostra docs de forma mais visível no preview.
```

### 24.2 Balanced

```text
Mostra ações quando relevante.
Docs ficam no preview.
```

### 24.3 Expert

```text
Oculta painel por padrão.
Ações acessíveis por Command Palette.
Docs acessíveis por hover/preview quando o usuário chamar.
```

---

## 25. Command Palette também deve respeitar escopo

Em CMake-only:

```text
CMake: Add executable
CMake: Add target_link_libraries
CMake: Create Debug preset
```

Não listar:

```text
Cargo: Add dependency
```

A menos que o usuário habilite:

```text
Show actions outside active scope
```

---

## 26. Context menu também deve respeitar escopo

Em CMake-only:

```text
CMakeLists.txt context menu:
- Add executable
- Add library
- Add link library
- Open CMake docs
```

Em Cargo-only:

```text
Cargo.toml context menu:
- Add dependency
- Add feature
- Open Cargo docs
```

Em Mixed:

```text
CMakeLists.txt → CMake actions
Cargo.toml     → Cargo actions
```

---

## 27. MVP desta melhoria

Implementar primeiro:

```text
[ ] Active Build System Model.
[ ] Project Settings > Build Systems.
[ ] Configuration Actions filtradas por CMake/Cargo/Mixed.
[ ] CMake-only mostra apenas CMake.
[ ] Cargo-only mostra apenas Cargo.
[ ] Mixed mostra CMake + Cargo com tabs/filtros.
[ ] Action metadata com campo scope.
[ ] Action metadata com campo docs.
[ ] Preview lateral com seção Documentation.
[ ] Command Palette respeita escopo.
```

---

## 28. Pós-MVP

```text
[ ] Documentation Registry completo.
[ ] Cache local de docs.
[ ] Kinein Guides curtos.
[ ] Links oficiais por comando CMake/Cargo.
[ ] Busca por intenção usando documentação.
[ ] Favoritos por escopo.
[ ] Recentes por escopo.
[ ] Team recommended actions.
```

---

## 29. Critérios de aceite

```text
[ ] Projeto CMake não mostra ações Cargo por padrão.
[ ] Projeto Cargo não mostra ações CMake por padrão.
[ ] Projeto Mixed mostra CMake e Cargo juntos de forma organizada.
[ ] Usuário pode alterar sistemas ativos em Project Settings.
[ ] Command Palette respeita o escopo ativo.
[ ] Context menus respeitam arquivo ativo e escopo.
[ ] Cada ação pode ter link de documentação.
[ ] Documentação aparece no preview, não polui a lista.
[ ] A lista continua densa, mas confortável.
[ ] Expert Mode mantém ações ocultas por padrão, mas acessíveis.
```

---

## 30. Resumo executivo

A melhoria principal é:

```text
Configuration Actions não devem ser globais.
Elas devem ser filtradas pelo sistema de build ativo do projeto.
```

Fluxos:

```text
CMake selecionado → só CMake.
Cargo selecionado → só Cargo.
CMake + Cargo selecionados → ambos, organizados.
```

E cada ação deve evoluir para ter:

```text
nome claro;
descrição curta;
arquivo afetado;
risco;
preview;
validação;
link de documentação.
```

Frase-guia:

```text
A Kinein mostra só o que faz sentido agora,
mas deixa o resto acessível quando o usuário quiser.
```
