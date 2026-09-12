# Kinein Vectis — Parte 9.1: Dual Workflow, Experience Modes e Configuration Actions

> **Tipo:** complemento da Parte 9.  
> **Escopo:** modos de experiência, fluxo guiado vs fluxo avançado, Configuration Actions para CMake/Cargo, UX densa porém confortável, first-run choice e settings.  
> **Decisão central:** a Kinein deve facilitar sem invadir. O usuário escolhe quanto auxílio visual quer receber.

---

## 1. Objetivo desta parte

A Kinein Vectis deve atender dois perfis sem criar conflito:

```text
Usuário iniciante/intermediário:
quer configurar C/C++/Rust visualmente, sem memorizar CMake/Cargo/flags.

Usuário avançado/expert:
já tem CMakePresets, Cargo.toml, scripts da empresa, padrões próprios
e quer que a IDE não atrapalhe.
```

A solução não é criar duas IDEs.  
A solução é ter **modos de experiência** e uma camada opcional chamada:

```text
Configuration Actions
```

Essas ações são atalhos visuais para configurar projetos, sempre gerando alterações reais em arquivos reais, com preview e validação.

---

## 2. Princípio de produto

```text
Visual para quem quer.
Texto para quem prefere.
Controle para todos.
```

A Kinein deve oferecer uma camada visual confortável sobre CMake e Cargo, mas nunca deve prender o usuário nela.

O usuário pode:

```text
- usar Project Wizard;
- usar Configuration Actions;
- editar CMakeLists.txt manualmente;
- editar Cargo.toml manualmente;
- usar terminal;
- usar CMakePresets da empresa;
- desabilitar sugestões;
- manter só estado visual/status.
```

Tudo isso deve ser considerado uso legítimo da IDE.

---

## 3. Experience Modes

A Kinein deve oferecer modos de experiência.

```text
Guided
Balanced
Expert
```

Esses modos não mudam a capacidade da IDE.  
Eles mudam **o quanto a IDE aparece para ajudar**.

---

## 4. First-run choice

Na primeira abertura, a IDE deve perguntar uma vez:

```text
How do you want Kinein to assist you?
```

Ou em português, futuramente:

```text
Como você quer que a Kinein te auxilie?
```

Opções:

```text
Guided
Para quem quer configurar C/C++/Rust visualmente com mais ajuda.

Balanced
Ajuda quando há problema, mas sem interromper o fluxo.

Expert
Mostra estado e ferramentas, mas reduz sugestões e wizards automáticos.
```

Abaixo:

```text
Você pode mudar isso depois em Settings > General > Experience Mode.
```

Regra:

```text
Essa escolha aparece uma vez.
Não deve aparecer toda vez que abrir a IDE.
```

---

## 5. Guided Mode

### 5.1 Perfil

Para usuários que querem:

```text
- criar projeto visualmente;
- escolher compilador sem decorar paths;
- adicionar bibliotecas via UI;
- entender rapidamente CMake/Cargo;
- receber sugestões claras;
- usar preview de diff;
- corrigir setup com ajuda.
```

### 5.2 Comportamento

```text
- Project Wizard mais visível;
- Configuration Actions aparecem em locais estratégicos;
- Project Health mais explícito;
- Setup Assistant ativo;
- toolchain recommendations visíveis;
- botões de “Add dependency”, “Add include”, “Add library” mais acessíveis;
- explicações curtas nos cards.
```

### 5.3 O que evitar

Mesmo no Guided Mode:

```text
- nada de pop-up agressivo;
- nada de auto-fix silencioso;
- nada de edição sem preview;
- nada de tutorial forçado longo;
- nada de IA embutida.
```

---

## 6. Balanced Mode

### 6.1 Perfil

Modo recomendado como padrão geral depois da primeira escolha.

Para usuários que:

```text
- sabem um pouco;
- querem ajuda quando algo quebra;
- não querem uma IDE barulhenta;
- ainda gostam de atalhos visuais.
```

### 6.2 Comportamento

```text
- Project Wizard disponível;
- Configuration Actions disponíveis, mas não gritantes;
- Setup Assistant aparece em falhas claras;
- Project Health discreto;
- sugestões aparecem como ações secundárias;
- Command Palette sempre permite chamar ações.
```

---

## 7. Expert Mode

### 7.1 Perfil

Para usuários que:

```text
- já sabem CMake/Cargo;
- usam projetos empresariais;
- têm CMakePresets existentes;
- preferem terminal;
- não querem wizards interferindo;
- querem usar a IDE como editor/build/debug forte.
```

### 7.2 Comportamento padrão

```text
- sem wizard automático intrusivo;
- sem Setup Assistant abrindo sozinho;
- sem sugestões visuais grandes;
- Configuration Actions disponíveis, mas recolhidas;
- Project Health discreto;
- status bar continua informativa;
- Command Palette continua poderosa;
- Settings continuam completas.
```

### 7.3 Ponto importante

Expert Mode não deve remover recursos.  
Ele só reduz a presença deles.

O usuário expert ainda pode chamar:

```text
Configuration Actions
Project Wizard
Setup Assistant
Toolchain Manager
AI Terminal Bridge
```

Mas por intenção explícita.

---

## 8. Configuração granular

Além do modo geral, a IDE deve permitir ajustes finos.

```text
Settings > General > Experience Mode
```

Opções:

```text
Experience Mode: Guided / Balanced / Expert

Show Configuration Actions:
- Always
- When relevant
- Only from Command Palette
- Disabled

Show Setup Assistant:
- Automatically on setup issues
- Only as badge
- Manual only

Show Project Health:
- Expanded
- Compact
- Status bar only

Show explanations in action cards:
- Always
- Short
- Hidden by default
```

Isso evita que o modo vire algo rígido demais.

---

## 9. Configuration Actions

### 9.1 Definição

**Configuration Actions** são ações visuais que modificam configurações de projeto de forma segura.

Elas atuam sobre:

```text
CMakeLists.txt
CMakePresets.json
Cargo.toml
.kinein/*.json
run-configs
debug-configs
targets
toolchains locais
```

Elas não são “funções do compilador” literalmente.  
São receitas de configuração aplicáveis ao projeto.

---

## 10. Regra central das Configuration Actions

```text
Toda Configuration Action deve explicar, mostrar preview, aplicar com consentimento e validar.
```

Fluxo:

```text
usuário escolhe ação
  ↓
Kinein lê arquivo atual
  ↓
detecta estrutura
  ↓
gera plano
  ↓
mostra descrição curta + arquivos afetados
  ↓
mostra diff
  ↓
usuário confirma
  ↓
aplica
  ↓
valida
  ↓
atualiza Project Health
```

---

## 11. UI densa, mas confortável

A lista de ações será inevitavelmente densa.  
O objetivo não é esconder densidade; é organizá-la bem.

### 11.1 Regras visuais

```text
- grupos claros;
- busca no topo;
- filtros por linguagem/build system;
- cards compactos;
- descrições curtas;
- ícones discretos;
- badges de risco;
- favoritos;
- ações recentes;
- preview lateral;
- sem cards gigantes;
- sem excesso de cor;
- sem layout “loja de plugins”.
```

### 11.2 Estrutura recomendada

```text
Configuration Actions
├── Search
├── Filters
│   ├── CMake
│   ├── Cargo
│   ├── Run/Debug
│   ├── Targets
│   └── Advanced
├── Category list
├── Action list
└── Preview panel
```

### 11.3 Densidade ideal

Cada item da lista deve ter:

```text
nome
descrição de uma linha
arquivo afetado
badge de risco
compatibilidade
```

Exemplo:

```text
Add target_link_libraries
Vincula uma biblioteca existente ao target selecionado.
CMakeLists.txt · medium
```

---

## 12. Layout visual das Configuration Actions

```text
┌──────────────────────────────────────────────────────────────┐
│ Configuration Actions                         Search...      │
├──────────────────┬────────────────────────────┬──────────────┤
│ Categories       │ Actions                    │ Preview      │
│                  │                            │              │
│ Favorites        │ Add executable             │ O que faz    │
│ CMake            │ Add static library         │ Arquivos     │
│ Cargo            │ Add target_link_libraries  │ Diff         │
│ Run/Debug        │ Add include directory      │ Validação    │
│ Targets          │ Add dependency             │              │
│ Advanced         │ Add feature                │ [Apply]      │
└──────────────────┴────────────────────────────┴──────────────┘
```

Esse layout evita uma lista gigante solta.

---

## 13. Categorias iniciais

### 13.1 CMake — Basic

```text
Add executable
Add static library
Add shared library
Add source file to target
Add include directory
Add target_link_libraries
Add compile definition
Add compile feature
Enable compile_commands.json
Create Debug preset
Create Release preset
```

### 13.2 CMake — Intermediate

```text
Add tests with CTest
Add install rule
Add option()
Add configure_file()
Add subdirectory
Add FetchContent dependency
Add package with find_package
Add toolchain file
Add sysroot
Add custom command
Add custom target
```

### 13.3 CMake — Advanced

```text
Create interface library
Add generator expression
Add imported target
Add export set
Add package config
Add cross-compilation preset
Add sanitizer profile
Add warnings profile
Add LTO profile
Inspect CMake cache
Repair stale build directory
```

### 13.4 Cargo — Basic

```text
Add dependency
Add dev-dependency
Add build-dependency
Add feature
Add binary target
Add library target
Run cargo check
Run cargo test
Set edition
```

### 13.5 Cargo — Workspace

```text
Create workspace
Add workspace member
Remove workspace member
Add shared dependency
Configure workspace package
Configure workspace lints
```

### 13.6 Run/Debug

```text
Create run configuration
Create debug configuration
Add program arguments
Add environment variable
Set working directory
Enable build before run
Stop at main
Select debugger
```

### 13.7 Targets

```text
Create local target
Create remote SSH target
Create QEMU target
Create bare-metal target
Add serial port
Add deploy path
Add flash command
Add GDB server
```

---

## 14. Action Card

Cada ação deve ter uma definição clara.

```json
{
  "id": "cmake.add_target_link_libraries",
  "title": "Add target_link_libraries",
  "category": "CMake",
  "level": "basic",
  "description": "Vincula uma biblioteca existente a um target CMake.",
  "affects": ["CMakeLists.txt"],
  "risk": "medium",
  "requires": ["cmake_project", "existing_target"],
  "preview_supported": true,
  "rollback_supported": true
}
```

---

## 15. Action UI item

Visual compacto:

```text
┌────────────────────────────────────────────────────┐
│ Add target_link_libraries                 medium   │
│ Vincula uma biblioteca existente ao target.         │
│ CMakeLists.txt · requer target existente            │
└────────────────────────────────────────────────────┘
```

Para não ficar pesado:

```text
- título em destaque;
- descrição curta;
- metadados pequenos;
- badge discreto;
- hover revela mais detalhes;
- preview lateral mostra o resto.
```

---

## 16. Preview panel

Ao selecionar uma ação:

```text
O que faz
Arquivos afetados
Pré-requisitos
Campos necessários
Diff proposto
Validação após aplicar
Risco
Rollback
```

Exemplo:

```text
Add target_link_libraries

O que faz:
Vincula uma biblioteca ao target selecionado usando CMake moderno.

Afeta:
CMakeLists.txt

Campos:
Target: motor_control
Library: motor_driver
Visibility: PRIVATE

Diff:
+ target_link_libraries(motor_control
+     PRIVATE
+         motor_driver
+ )

Validação:
cmake --preset debug
```

---

## 17. Busca e filtros

Como haverá muitas ações, busca precisa ser excelente.

### 17.1 Busca por termos técnicos

```text
link
library
dependency
include
preset
debug
target
feature
workspace
serial
qemu
flash
```

### 17.2 Busca por intenção

```text
quero adicionar uma biblioteca
quero linkar algo
quero adicionar dependency Rust
quero criar preset
quero ativar compile_commands
```

No MVP, busca pode ser keyword.  
Depois pode ser fuzzy.

### 17.3 Filtros

```text
Language: C / C++ / Rust
Build system: CMake / Cargo
Level: Basic / Intermediate / Advanced
Risk: Low / Medium / High
Available only
Favorites
Recently used
```

---

## 18. Compatibilidade

Cada ação deve indicar se está disponível.

Estados:

```text
Available
Partially available
Unavailable
Unsupported for this project
Requires configuration
```

Exemplo:

```text
Add target_link_libraries
Available
Reason: CMake project with target motor_control detected.
```

Exemplo:

```text
Add Rust dependency
Unavailable
Reason: Cargo.toml not found.
```

Em Expert Mode, ações indisponíveis podem ficar ocultas por padrão.

---

## 19. Modos e Configuration Actions

### 19.1 Guided

```text
Configuration Actions visíveis em:
- Project view;
- CMake panel;
- Cargo panel;
- Project Health;
- Command Palette;
- right-click em CMakeLists/Cargo.toml.
```

### 19.2 Balanced

```text
Configuration Actions visíveis em:
- Command Palette;
- CMake/Cargo panel;
- Project Health quando relevante.
```

### 19.3 Expert

```text
Configuration Actions:
- ocultas por padrão;
- acessíveis via Command Palette;
- acessíveis por Settings;
- opção para mostrar compactamente na UI.
```

---

## 20. Não atrapalhar projetos empresariais

Projetos empresariais podem ter:

```text
CMake complexo
presets próprios
toolchain da empresa
scripts customizados
monorepo
políticas internas
targets múltiplos
dependências privadas
```

A Kinein deve reconhecer isso.

### 20.1 Regra

```text
Em projeto existente, Configuration Actions nunca devem aplicar mudanças sem preview.
```

### 20.2 Modo seguro

Para projetos existentes:

```text
Preferir criar arquivos locais .kinein/
Evitar editar CMakeLists.txt automaticamente
Respeitar CMakePresets existentes
Mostrar "Use existing configuration"
Permitir "Do not suggest setup for this project"
```

---

## 21. Parsing de CMake

CMake é linguagem flexível.  
Não dá para prometer entender 100% no MVP.

### 21.1 Estratégia realista

MVP:

```text
suportar CMakeLists simples;
suportar projetos gerados pela Kinein;
detectar targets básicos;
detectar add_executable;
detectar add_library;
detectar target_include_directories;
detectar target_link_libraries;
gerar patches localizados;
avisar quando arquivo é complexo demais.
```

### 21.2 Quando não entender

A IDE deve dizer:

```text
Este CMakeLists.txt parece complexo.
A Kinein pode abrir o arquivo e sugerir um patch manual,
mas não vai aplicar automaticamente.
```

### 21.3 Regras de patch

```text
- preferir inserir perto do target relevante;
- preservar formatação quando possível;
- não reformatar arquivo inteiro;
- não reorganizar CMake existente;
- não remover conteúdo;
- gerar diff pequeno.
```

---

## 22. Parsing de Cargo

Cargo.toml é mais estruturado.

A Kinein pode usar parser TOML.

Ações mais seguras:

```text
adicionar dependency
adicionar dev-dependency
adicionar feature
alterar edition
adicionar workspace member
```

Ainda assim:

```text
preview obrigatório;
preservar comentários quando possível;
não reformatar arquivo inteiro no MVP;
```

---

## 23. Configuration Actions e terminal

Toda ação visual deve poder mostrar o equivalente textual.

Exemplo:

```text
Visual action:
Add dependency serde

Equivalent:
cargo add serde --features derive
```

Ou se for editar TOML diretamente:

```text
Cargo.toml:
serde = { version = "1", features = ["derive"] }
```

Para CMake:

```text
CMakeLists.txt:
target_link_libraries(app PRIVATE motor_driver)
```

Isso ajuda usuários intermediários a aprenderem sem obrigar leitura de documentação.

---

## 24. Command Palette

Configuration Actions devem aparecer na Command Palette.

Exemplos:

```text
CMake: Add target_link_libraries
CMake: Add include directory
CMake: Enable compile_commands.json
Cargo: Add dependency
Cargo: Add feature
Run: Create run configuration
Debug: Create debug configuration
Target: Create remote SSH target
```

No Expert Mode, esse pode ser o principal caminho.

---

## 25. Context menu

Em arquivos específicos:

### 25.1 CMakeLists.txt

```text
Add executable
Add library
Add include directory
Add linked library
Create preset
Inspect target
```

### 25.2 Cargo.toml

```text
Add dependency
Add dev-dependency
Add feature
Add workspace member
Run cargo check
```

### 25.3 Project tree

```text
Add source to target
Create run config
Create debug config
Open Configuration Actions
```

---

## 26. “Library of actions”, não “library of magic”

A lista de ações deve parecer uma caixa de ferramentas profissional.

Ela não deve parecer:

```text
store
marketplace
plugin gallery
AI suggestion feed
tutorial infantil
```

Ela deve parecer:

```text
painel técnico
compacto
pesquisável
organizado
confiável
```

---

## 27. Settings sugeridas

```text
Settings > General > Experience Mode
Settings > Build > Configuration Actions
```

Configurações:

```text
Enable Configuration Actions: true/false
Default visibility: always / relevant / command_palette_only
Show descriptions: full / short / hidden
Show risk badges: true/false
Require preview before apply: always
Show unavailable actions: true/false
Enable CMake actions: true/false
Enable Cargo actions: true/false
Enable advanced actions: true/false
```

Regra:

```text
Require preview before apply deve ser always.
Não permitir desligar preview em ações que editam arquivo.
```

---

## 28. JSON-RPC sugerido

### 28.1 Listar ações

```json
{
  "method": "configActions.list",
  "params": {
    "workspace_id": "workspace_001",
    "filters": {
      "language": ["cpp"],
      "build_system": ["cmake"],
      "level": ["basic", "intermediate"],
      "available_only": true
    }
  }
}
```

### 28.2 Obter detalhes

```json
{
  "method": "configActions.get",
  "params": {
    "action_id": "cmake.add_target_link_libraries"
  }
}
```

### 28.3 Criar plano

```json
{
  "method": "configActions.plan",
  "params": {
    "workspace_id": "workspace_001",
    "action_id": "cmake.add_target_link_libraries",
    "inputs": {
      "target": "motor_control",
      "library": "motor_driver",
      "visibility": "PRIVATE"
    }
  }
}
```

### 28.4 Preview

```json
{
  "method": "configActions.preview",
  "params": {
    "plan_id": "plan_001"
  }
}
```

### 28.5 Aplicar

```json
{
  "method": "configActions.apply",
  "params": {
    "plan_id": "plan_001",
    "confirmation_token": "user_confirmed"
  }
}
```

---

## 29. Core modules sugeridos

```text
crates/
  kinein-config-actions/
    action_registry.rs
    action_definition.rs
    action_filter.rs
    action_planner.rs
    action_preview.rs
    action_apply.rs
    action_validation.rs
    action_rollback.rs

  kinein-cmake-edit/
    cmake_scanner.rs
    cmake_target_model.rs
    cmake_patch.rs
    cmake_actions.rs

  kinein-cargo-edit/
    cargo_toml_model.rs
    cargo_patch.rs
    cargo_actions.rs

  kinein-experience/
    experience_mode.rs
    suggestion_policy.rs
    ui_visibility_policy.rs
```

---

## 30. QML components sugeridos

```text
ui/components/config_actions/
  ConfigurationActionsPanel.qml
  ConfigurationActionList.qml
  ConfigurationActionItem.qml
  ConfigurationActionPreview.qml
  ConfigurationActionFilters.qml
  ConfigurationActionSearch.qml
  ConfigurationActionBadge.qml
  ConfigurationActionInputForm.qml
```

---

## 31. MVP realista

Para o MVP, implementar poucas ações com qualidade.

### 31.1 CMake MVP

```text
Enable compile_commands.json
Add executable
Add static library
Add source file to target
Add include directory
Add target_link_libraries
Create Debug preset
Create Release preset
Inspect CMake cache
```

### 31.2 Cargo MVP

```text
Add dependency
Add dev-dependency
Add feature
Run cargo check
Set edition
```

### 31.3 Experience MVP

```text
First-run Experience Mode choice
Guided/Balanced/Expert setting
Configuration Actions panel
Command Palette integration
Preview diff obrigatório
```

---

## 32. Pós-MVP

```text
CMake advanced actions
Cargo workspace actions
Targets actions
Run/Debug actions
Favorites
Recent actions
Fuzzy search
Import from existing configs
Action recipes
Team-shared recommended actions
```

---

## 33. Critérios de aceite

A parte estará correta quando:

```text
[ ] Usuário escolhe Guided/Balanced/Expert no primeiro uso.
[ ] Modo pode ser alterado depois nas Settings.
[ ] Expert Mode reduz UI guiada sem remover recursos.
[ ] Configuration Actions tem busca e filtros.
[ ] Lista é densa, mas visualmente confortável.
[ ] Cada ação tem nome claro e descrição curta.
[ ] Cada ação mostra arquivo afetado e risco.
[ ] Toda alteração tem preview de diff.
[ ] Projetos existentes não são alterados sem consentimento.
[ ] CMake complexo é tratado com cautela.
[ ] Cargo.toml é editado por parser TOML.
[ ] Command Palette expõe ações para usuários avançados.
[ ] Usuário pode desabilitar a camada visual guiada.
```

---

## 34. Resumo executivo

A Kinein deve ter dois fluxos naturais:

```text
Guided visual workflow
para quem quer configurar C/C++/Rust sem decorar CMake/Cargo.

Direct expert workflow
para quem já sabe e quer velocidade, terminal e controle.
```

As **Configuration Actions** são a ponte entre esses mundos.

Elas permitem:

```text
- escolher ações visualmente;
- entender o que a ação faz;
- aplicar mudanças reais em CMake/Cargo;
- ver diff antes;
- validar depois;
- aprender sem ser forçado;
- ignorar tudo isso se preferir editar manualmente.
```

Frase-guia:

```text
A Kinein deve tornar configuração visível e confortável,
mas nunca obrigatória.
```
