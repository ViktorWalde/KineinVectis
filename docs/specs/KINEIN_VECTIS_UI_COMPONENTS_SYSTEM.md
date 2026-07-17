# Kinein Vectis — Parte 3: Sistema de Componentes da Interface

> Documento de especificação visual e funcional para os componentes principais da interface da IDE **Kinein Vectis**.
>
> Nome oficial: **Kinein Vectis**  
> Nome de uso diário: **Kinein**  
> Sigla visual: **KV**  
> Foco técnico: **C, C++ e Rust**  
> Direção de produto: **IDE Linux-first para sistemas, toolchains, CMake, embarcados, simulação e desenvolvimento de engenharia**.

---

## 1. Objetivo deste documento

Este documento define a **biblioteca de componentes visuais e interativos** da Kinein Vectis.

A Parte 1 tratou da iconografia.  
A Parte 2 tratou do layout principal da IDE.  
Esta Parte 3 define **como cada peça da interface deve se comportar, parecer e escalar**.

O objetivo é permitir que uma IA CLI, um desenvolvedor humano ou um futuro time do projeto consiga implementar a UI de forma consistente, sem cair em improvisações visuais.

Este documento deve ser usado para orientar:

- componentes Qt/QML;
- design tokens;
- estados visuais;
- botões;
- inputs;
- abas;
- painéis;
- cards;
- listas;
- trees;
- toolbars;
- status indicators;
- diálogos;
- menus;
- notificações;
- Assistente;
- integração visual com CMake, toolchains, targets, debug e simulação.

---

## 2. Princípio central

A interface da Kinein deve parecer profissional, familiar e confortável como uma IDE madura, mas com identidade própria.

A referência psicológica é:

```text
JetBrains-like na sensação de controle, familiaridade e polimento.
Kinein-like na identidade visual, iconografia, foco em sistemas e linguagem vetorial.
```

Isso significa:

- não copiar visualmente produtos existentes;
- manter densidade profissional;
- priorizar editor no centro;
- reduzir ansiedade visual;
- deixar CMake, toolchain, debug e target visíveis sem parecerem assustadores;
- fazer o usuário sentir que a IDE é poderosa, mas não caótica;
- não transformar a interface em dashboard futurista decorativo;
- manter todos os componentes funcionais, discretos e previsíveis.

---

## 3. Não objetivos

A Kinein **não deve** ter estes comportamentos visuais:

- UI com aparência de game launcher;
- excesso de neon;
- excesso de brilho;
- excesso de textura;
- cards enormes para ações simples;
- botões grandes demais em fluxo de código;
- ícones fofos/cartoon;
- excesso de animação;
- painel de IA invasivo;
- terminal com baixa legibilidade;
- mistura entre UI real e mockup de marketing;
- uso de engrenagens como símbolo principal para tudo;
- aparência de tema experimental de VS Code sem polimento.

A IDE deve parecer uma ferramenta de engenharia para uso diário.

---

## 4. Camadas do sistema visual

A interface deve ser pensada em cinco camadas:

```text
1. Tokens
   Cores, espaçamentos, raios, tipografia, sombras, opacidade.

2. Primitivos
   Texto, ícone, linha, borda, fundo, separador, superfície.

3. Componentes base
   Botão, input, select, tab, tree item, list item, badge, chip.

4. Componentes compostos
   Toolbar, sidebar, editor tab bar, status bar, tool window, Assistente card.

5. Fluxos
   Coding, build, debug, configure, embedded target, simulation, onboarding.
```

Nenhuma tela deve pular direto para o nível 5 sem respeitar os níveis 1 a 4.

---

## 5. Tokens de design

### 5.1 Cores base

```text
App background:        #0B0D10
Editor background:     #0F1216
Panel background:      #111418
Panel elevated:        #171B21
Panel sunken:          #0A0C0F
Border subtle:         #252A32
Border visible:        #343A44
Divider:               #20242B
Hover surface:         #1A1F27
Selected surface:      #222833
Current line:          #1A2028
```

### 5.2 Texto

```text
Text primary:          #E7E2D8
Text secondary:        #AAA39A
Text muted:            #737A84
Text disabled:         #4F5661
Text inverse:          #0B0D10
```

### 5.3 Acentos

```text
Amber primary:         #FFB000
Amber active:          #FFC93D
Amber soft:            #C98D19
Amber dark:            #8A5A0A

Blue technical:        #5C8DFF
Purple orbital:        #8A5CFF
Green success:         #7CCF6A
Red error:             #D45F5F
Orange warning:        #E69B32
Cyan info:             #57C7D4
```

### 5.4 Uso correto do âmbar

O âmbar é a identidade da Kinein, mas deve ser usado com disciplina.

Usar âmbar para:

- ação primária;
- aba ativa;
- ícone ativo;
- foco de componente;
- progresso de build;
- botão Run;
- estado selecionado;
- highlights mínimos;
- detalhe de marca KV.

Não usar âmbar para:

- todo texto importante;
- todos os ícones ao mesmo tempo;
- grandes áreas de fundo;
- bordas exageradas;
- warnings comuns se isso confundir com ação primária;
- decoração sem função.

A sensação desejada é:

```text
O fundo acalma. O âmbar orienta.
```

---

## 6. Espaçamento

### 6.1 Escala base

```text
2px   micro ajuste
4px   espaçamento interno mínimo
6px   compact controls
8px   espaçamento padrão pequeno
12px  espaçamento padrão médio
16px  blocos e grupos
20px  painéis compactos
24px  grupos grandes
32px  áreas de respiro
```

### 6.2 Densidade por área

```text
Toolbar:          6px a 8px entre itens
Sidebar:          4px vertical, 8px horizontal
Tree view:        2px a 4px vertical
Editor tabs:      0px entre abas, bordas sutis
Cards:            12px interno
Dialogs:          20px a 24px interno
Status bar:       6px horizontal
Assistente:       12px entre cards
```

A interface deve ser compacta, mas não sufocante.

---

## 7. Raios de borda

```text
Radius XS:        3px   itens de tree, pequenas tags
Radius SM:        5px   botões compactos, inputs pequenos
Radius MD:        8px   cards, selects, campos de texto
Radius LG:        12px  dialogs, popovers grandes
Radius XL:        18px  telas de onboarding e app icon preview
```

Não usar cantos muito arredondados na IDE real.  
O visual deve ser técnico e maduro.

---

## 8. Tipografia

### 8.1 Fontes recomendadas

```text
UI:       Inter, Segoe UI, Noto Sans, system sans
Editor:   JetBrains Mono, Cascadia Code, Fira Code, IBM Plex Mono
Terminal: JetBrains Mono, Cascadia Mono, Fira Mono
```

### 8.2 Tamanhos

```text
UI micro:          10px
UI small:          11px
UI default:        12px
UI comfortable:    13px
Panel title:       13px / medium
Dialog title:      18px / semibold
Editor default:    13px ou 14px
Terminal default:  12px ou 13px
Status bar:        11px
```

### 8.3 Regras

- O editor deve permitir escala independente da UI.
- Terminal deve permitir escala independente da UI.
- Status bar deve continuar legível em 100% zoom.
- Texto secundário não pode ficar invisível em monitores de baixo contraste.
- Não usar all caps em excesso. Usar all caps apenas em rótulos muito curtos ou grupos de sidebar.

---

## 9. Elevação e separação visual

Como a IDE é escura, a separação deve vir mais de diferença de superfície e bordas sutis do que de sombras fortes.

```text
Level 0 — App background
Level 1 — Editor e painéis principais
Level 2 — Cards, abas ativas, selects
Level 3 — Menus, popovers, command palette
Level 4 — Dialogs modais
```

### 9.1 Sombras

Usar sombras com moderação:

```text
Popover:    0 8px 24px rgba(0,0,0,0.35)
Dialog:     0 18px 48px rgba(0,0,0,0.45)
Card hover: 0 4px 16px rgba(0,0,0,0.22)
```

Em Qt/QML, se sombra pesar performance, preferir borda + superfície elevada.

---

## 10. Estados visuais globais

Todos os componentes interativos devem suportar estes estados:

```text
Default
Hover
Pressed
Focused
Selected
Active
Disabled
Loading
Error
Warning
Success
```

### 10.1 Default

Estado neutro, discreto, sem chamar atenção.

```text
Background: transparente ou painel base
Border: subtle ou none
Text: secondary/primary conforme importância
Icon: secondary
```

### 10.2 Hover

Hover deve ser visível, mas suave.

```text
Background: #1A1F27
Border: #343A44 se necessário
Text: primary
Icon: primary ou amber se for ação principal
```

### 10.3 Focused

Foco é essencial para teclado.

```text
Border: Amber primary com 60% a 85% opacidade
Outline: 1px ou 2px
Nunca depender apenas de cor se possível.
```

### 10.4 Active/Selected

Selecionado deve mostrar estado persistente.

```text
Background: #222833
Accent line: Amber primary
Text: primary
Icon: amber ou primary
```

### 10.5 Disabled

```text
Opacity: 0.38 a 0.45
Text: disabled
Icon: disabled
Sem hover
Sem tooltip invasivo, exceto quando explicar por que está desabilitado.
```

### 10.6 Loading

Usar loading discreto:

- spinner pequeno;
- barra de progresso fina;
- skeleton apenas em Assistente ou listas longas;
- nunca bloquear editor por operações de fundo.

### 10.7 Error

Erro deve ser claro, mas não alarmista.

```text
Red error: #D45F5F
Background error soft: rgba(212,95,95,0.10)
Border error: rgba(212,95,95,0.45)
```

### 10.8 Warning

```text
Orange warning: #E69B32
Background warning soft: rgba(230,155,50,0.10)
```

### 10.9 Success

```text
Green success: #7CCF6A
Background success soft: rgba(124,207,106,0.10)
```

---

# 11. Componentes base

---

## 11.1 KVButton

Botão padrão da Kinein.

### Tipos

```text
Primary
Secondary
Ghost
Danger
Success
Toolbar
IconOnly
SplitButton
```

### Primary

Usado apenas para ação principal de uma tela ou diálogo.

Exemplos:

- Open Workspace
- Configure Project
- Apply Suggestion
- Create Preset
- Start Debug

Visual:

```text
Background: Amber primary
Text: Text inverse
Border: Amber active
Radius: 6px
Height: 30px a 34px
Padding: 12px horizontal
Font weight: 500 ou 600
```

Nunca colocar vários botões primários na mesma área.

### Secondary

Usado para ações comuns.

```text
Background: #171B21
Border: #343A44
Text: primary
Hover: #1E242D
```

### Ghost

Usado em toolbars, headers e cards.

```text
Background: transparent
Hover: #1A1F27
Pressed: #222833
```

### Toolbar button

```text
Size: 28x28 ou 30x30
Icon: 16px ou 18px
Radius: 5px
Hover background: #1A1F27
Active icon: Amber primary
```

### IconOnly

Usado para fechar aba, expandir painel, opções.

```text
Size: 24x24
Icon: 14px a 16px
Background hover discreto
```

### SplitButton

Usado para Run/Debug/Build com menu de configuração.

```text
Parte esquerda: ação direta
Parte direita: dropdown
Separador vertical sutil
Altura: 30px
```

Exemplo:

```text
[ Run ▶ ][ ▼ ]
[ Debug ◇ ][ ▼ ]
[ Build >_ ][ ▼ ]
```

---

## 11.2 KVInput

Campo de texto padrão.

Usos:

- busca;
- filtro;
- caminho de workspace;
- argumentos de execução;
- variáveis de ambiente;
- command palette.

Visual:

```text
Height: 30px a 34px
Radius: 6px
Background: #0F1216 ou #171B21
Border: #343A44
Text: primary
Placeholder: muted
Focus border: Amber primary
```

### Variações

```text
Compact: 26px
Default: 32px
Large: 38px
CommandPalette: 44px
```

### Regras

- Input não deve brilhar demais.
- Placeholder deve ser útil, não decorativo.
- Em erro, mostrar mensagem curta abaixo.
- Para caminhos, usar botão de browse à direita.

---

## 11.3 KVSelect

Select/dropdown para opções compactas.

Usos:

- target;
- profile;
- generator;
- compiler;
- kit;
- build type;
- encoding;
- line ending.

Visual:

```text
Height: 30px
Radius: 6px
Background: #171B21
Border: #343A44
Text: primary
Chevron: muted ou amber quando ativo
```

Exemplo de top toolbar:

```text
[ Target: x86_64-linux ▼ ]
[ CMake: Debug ▼ ]
[ clang++ 17 ▼ ]
```

### Dropdown menu

```text
Background: #171B21
Border: #343A44
Item height: 28px
Selected item: #222833 + amber left accent
Hover item: #1A1F27
```

---

## 11.4 KVCheckbox

Uso em settings, dialogs e configurações.

Visual:

```text
Size: 16x16
Radius: 3px
Border: #535B68
Checked fill: Amber primary
Check mark: #0B0D10
```

Regra:

- Não usar checkboxes em toolbar principal.
- Settings podem usar checkboxes, toggles e segmented controls.

---

## 11.5 KVToggle

Uso para ligar/desligar opções persistentes.

Exemplos:

- Auto configure CMake
- Auto detect toolchain
- Use Assistente
- Format on save
- Show minimap

Visual:

```text
Width: 36px
Height: 20px
Radius: 10px
Off: #343A44
On: Amber primary
Thumb: #E7E2D8 ou #0B0D10 conforme contraste
```

---

## 11.6 KVBadge

Badge pequeno para status.

Exemplos:

```text
Beta
Debug
Release
x86_64
arm-none-eabi
CMake
Cargo
Remote
Running
```

Visual:

```text
Height: 18px a 20px
Radius: 10px
Padding: 6px horizontal
Font: 10px ou 11px
Background: superfície elevada
Border: subtle
```

Cores:

```text
Default: gray
Active: amber
Success: green
Warning: orange
Error: red
Info: blue
```

---

## 11.7 KVChip

Chip é similar ao badge, mas clicável/removível.

Usos:

- filtros;
- tags de toolchain;
- include paths;
- defines;
- features Cargo;
- CMake options.

Visual:

```text
Height: 24px
Radius: 12px
Padding: 8px
Close icon opcional
```

---

## 11.8 KVTooltip

Tooltip deve ser útil e curto.

Visual:

```text
Background: #171B21
Border: #343A44
Text: primary
Secondary text: muted
Radius: 6px
Max width: 320px
Delay: 450ms a 650ms
```

Conteúdo recomendado:

```text
Título curto
Descrição de uma linha
Atalho, se existir
```

Exemplo:

```text
Build Project
Run CMake/Ninja build for the selected profile.
Shortcut: Ctrl+B
```

Não usar tooltip para explicar conceitos longos. Para isso, usar Assistente ou docs.

---

## 11.9 KVPopover

Popover para menus curtos, seletores, ações rápidas.

Visual:

```text
Background: #171B21
Border: #343A44
Radius: 8px
Shadow: leve
Padding: 6px
```

Usos:

- Run configuration quick menu;
- Build target menu;
- profile switcher;
- Git branch picker;
- target picker.

---

## 11.10 KVDialog

Dialog modal para decisões relevantes.

Visual:

```text
Width padrão: 520px a 720px
Background: #171B21
Border: #343A44
Radius: 12px
Header height: 52px
Footer height: 56px
Padding: 20px ou 24px
```

Estrutura:

```text
Header: título + descrição curta + botão fechar
Body: conteúdo
Footer: botões alinhados à direita
```

Regras:

- Botão primário à direita.
- Cancelamento à esquerda do primário.
- Não criar dialogs gigantes para fluxos complexos; usar wizard.

---

# 12. Componentes estruturais da IDE

---

## 12.1 KVAppShell

Componente raiz da IDE.

Responsável por:

- barra superior;
- sidebar esquerda;
- área central;
- painéis direito e inferior;
- status bar;
- overlays globais;
- command palette;
- atalhos globais;
- persistência de layout.

Estrutura lógica:

```text
KVAppShell
├── KVTopBar
├── KVMainArea
│   ├── KVLeftActivityBar
│   ├── KVToolWindowAreaLeft
│   ├── KVEditorArea
│   ├── KVToolWindowAreaRight
│   └── KVBottomToolWindowArea
└── KVStatusBar
```

Regras:

- O editor deve ser a área dominante.
- Tool windows devem ser colapsáveis.
- O layout deve ser persistido por workspace.
- Deve haver modo Focus.
- Deve haver reset de layout.

---

## 12.2 KVTopBar

Barra superior principal.

Altura recomendada:

```text
40px a 44px
```

Conteúdo:

```text
[KV logo] [Kinein] [Project switcher]    [Target] [Profile] [Build] [Run] [Debug] [Flash] [Simulate]    [Search] [Settings] [Window controls]
```

### Regras de comportamento

- Deve ser compacta.
- Deve destacar Run/Build sem poluir.
- Target e Profile devem ser visíveis.
- Ações especializadas podem ficar em overflow.
- Não colocar texto longo na toolbar.

### Ordem recomendada das ações

```text
Target selector
Build profile selector
Configure
Build
Run
Debug
Flash
Simulate
Search
Assistente toggle
Settings
Overflow
```

---

## 12.3 KVActivityBar

Sidebar extrema esquerda com ícones principais.

Largura:

```text
48px a 52px
```

Itens iniciais:

```text
Project
Search
Git
Build
Debug
Targets
Simulate
Tools
Extensions
```

Estados:

```text
Default: ícone muted
Hover: superfície suave
Active: ícone âmbar + barra esquerda/indicador
Disabled: opacidade reduzida
```

Regra:

- Ícones devem ter tooltip.
- O item ativo deve ser evidente.
- Não usar texto sempre visível na activity bar em modo padrão.
- Texto pode aparecer no modo expanded/accessible.

---

## 12.4 KVToolWindow

Painel lateral ou inferior genérico.

Exemplos:

- Project;
- Structure;
- CMake;
- Build;
- Debug;
- Serial;
- Git;
- Assistente;
- Simulation.

Estrutura:

```text
Header
Toolbar compacta
Conteúdo
Footer opcional
```

### Header

```text
Height: 34px
Title: 12px ou 13px, medium
Actions: right aligned, 24x24
Border bottom: subtle
```

### Regras

- Todo tool window deve poder ser redimensionado.
- Todo tool window deve poder ser colapsado.
- Alguns tool windows podem ser movidos no futuro.
- Estado de largura/altura deve ser persistido.
- Se vazio, mostrar empty state útil.

---

## 12.5 KVPanelHeader

Header usado em tool windows.

Exemplo:

```text
Project                     [refresh] [collapse] [more]
```

Visual:

```text
Height: 34px
Padding: 8px a 10px
Title color: primary
Actions: muted -> hover primary
```

---

## 12.6 KVSplitter

Separador redimensionável entre painéis.

Visual:

```text
Width vertical: 1px normal, 4px hit area
Height horizontal: 1px normal, 4px hit area
Color normal: divider
Hover: border visible
Drag: amber subtle
```

Regra:

- Hit area deve ser maior que linha visual.
- Durante drag, mostrar feedback claro.
- Double click pode resetar tamanho.

---

# 13. Componentes do editor

---

## 13.1 KVEditorArea

Área central da IDE.

Composta por:

```text
Tab bar
Breadcrumbs
Editor viewport
Gutter
Minimap opcional
Inline diagnostics
```

Regras:

- O editor deve ter prioridade visual absoluta.
- Painéis nunca devem roubar foco sem ação do usuário.
- O editor deve continuar utilizável com painéis abertos.
- O fundo do editor deve ser levemente diferente dos painéis.

---

## 13.2 KVEditorTabBar

Abas dos arquivos abertos.

Altura:

```text
34px a 36px
```

Item de aba:

```text
Icone linguagem
Nome do arquivo
Indicador de modificação
Botão fechar no hover
```

Estados:

```text
Inactive: background app/panel, text secondary
Hover: background hover
Active: background editor, text primary, top/under accent amber
Modified: ponto pequeno ou texto em itálico discreto
Error in file: marcador vermelho pequeno
```

Regra:

- Não usar abas muito altas.
- Não usar bordas pesadas.
- Aba ativa deve ser clara.
- Fechar aba deve aparecer no hover ou quando ativa.

---

## 13.3 KVBreadcrumbs

Mostra caminho sem ocupar muito espaço.

Exemplo:

```text
src > drivers > motor > motor_control.cpp > MotorControl > update
```

Visual:

```text
Height: 24px
Font: 11px
Text muted
Último item: primary
Separadores: muted
```

Uso:

- navegação rápida;
- contexto do arquivo;
- símbolos C/C++/Rust.

---

## 13.4 KVGutter

Área à esquerda do código.

Contém:

- line numbers;
- breakpoints;
- diagnostics;
- execução atual;
- code actions;
- folding;
- coverage futura.

Visual:

```text
Background: editor background
Line number: muted
Current line number: primary/amber soft
Breakpoint: red/purple com borda
Warning: orange pequeno
Error: red pequeno
```

Regra:

- Não poluir gutter.
- Code action só aparece quando relevante.
- Breakpoint precisa ser fácil de clicar.

---

## 13.5 KVInlineDiagnostic

Diagnóstico inline no editor.

Tipos:

```text
Error
Warning
Info
Hint
```

Visual:

```text
Underline ondulado ou linha sutil
Tooltip no hover
Quick fix no gutter
```

Regras:

- Mensagem completa não deve aparecer inline por padrão se ocupar muito espaço.
- Hover mostra detalhes.
- Assistente pode explicar erro complexo.

---

## 13.6 KVCodeActionPopup

Popup de ações rápidas.

Exemplos:

- Include missing header;
- Add link library;
- Fix CMake target;
- Convert include path;
- Apply clang-tidy fix;
- Add Rust feature;
- Import symbol.

Visual:

```text
Popover compacto
Lista com ícone + título + descrição curta
Atalho se houver
```

Regra:

- Deve abrir por lâmpada/gutter ou atalho.
- Deve ser navegável por teclado.

---

# 14. Componentes de Project/Explorer

---

## 14.1 KVTreeView

Árvore usada em Project, Structure, CMake Targets e Symbols.

Linha:

```text
Height: 22px a 24px
Indent: 14px a 16px por nível
Icon: 16px
Text: 12px
```

Estados:

```text
Hover: #1A1F27
Selected: #222833 + text primary
Focused selected: accent lateral amber
Modified file: cor secundária ou marker M
Ignored file: muted
Generated file: muted/italic opcional
```

Regras:

- Tree deve ser rápida com muitos arquivos.
- Virtualização é desejável para projetos grandes.
- Filtro deve ser instantâneo.
- Pastas geradas como build/ podem ser agrupadas ou de menor destaque.

---

## 14.2 KVProjectTreeItem

Cada item do explorer.

Metadados visuais:

```text
C++ source: ícone C++
C header: ícone H
Rust: ícone Rust discreto
CMake: ícone delta/triângulo
Toolchain file: ícone toolchain
Folder: pasta angular
Generated: badge G ou opacidade menor
Modified: marker M
Error: indicador vermelho
Warning: indicador laranja
```

---

## 14.3 KVStructureView

Mostra símbolos do arquivo atual.

Para C++:

```text
Classes
Structs
Enums
Functions
Methods
Fields
Namespaces
Macros
```

Para Rust:

```text
Modules
Structs
Enums
Traits
Impl blocks
Functions
Constants
Macros
```

Visual:

- mesmo KVTreeView;
- símbolos com ícones discretos;
- item atual destacado conforme cursor.

---

# 15. Componentes de Build/CMake/Toolchain

---

## 15.1 KVBuildProfileSelector

Select da toolbar para perfil.

Exemplos:

```text
CMake: Debug
CMake: Release
CMake: RelWithDebInfo
Cargo: dev
Cargo: release
```

Visual:

```text
Compact select
Badge de sistema: CMake/Cargo
Estado de configure: OK/Warning/Error
```

---

## 15.2 KVTargetSelector

Select da toolbar para target.

Exemplos:

```text
x86_64-linux-gnu
arm-none-eabi
aarch64-linux-gnu
stm32f767zi
qemu-aarch64
remote: devbox
```

Visual:

```text
Icone do target
Nome curto
Dropdown com descrição
Status dot
```

Dropdown deve mostrar:

```text
Target name
Architecture
Compiler
Sysroot
Debugger
Deploy method
Status
```

---

## 15.3 KVCMakeConfigureCard

Card para status do CMake configure.

Estados:

```text
Not configured
Configuring
Configured
Cache warning
Configure failed
```

Visual:

```text
Header: CMake Configure
Status badge
Resumo: generator, build dir, compiler
Ações: Configure, Clear Cache, Open CMakeCache, Edit Preset
```

---

## 15.4 KVToolchainCard

Card para toolchain detectada.

Campos:

```text
Compiler C
Compiler C++
Rust toolchain
Linker
Debugger
CMake
Ninja
Sysroot
Environment
PATH status
```

Estados:

```text
Detected
Missing
Version mismatch
Unsupported
Needs configuration
```

Visual:

- ícone toolchain;
- status dot;
- ações rápidas;
- mensagens curtas.

---

## 15.5 KVBuildOutputParser

Componente visual para build output estruturado.

Deve separar:

```text
Configure
Build
Link
Warnings
Errors
Tests
Artifacts
```

O terminal pode continuar bruto, mas o painel Build deve apresentar estrutura.

Visual:

```text
Timeline vertical
Itens com status
Tempo por etapa
Arquivo/linha clicável
```

---

## 15.6 KVProblemsList

Lista de problemas.

Colunas mínimas:

```text
Severity
File
Line
Message
Source
```

Densidade:

```text
Linha: 24px a 28px
Ícone: 14px
Fonte: 12px
```

Regras:

- Clicar navega para o arquivo.
- Agrupar por arquivo opcional.
- Filtrar por error/warning/info.
- Mostrar origem: clangd, rust-analyzer, CMake, linker, test.

---

# 16. Componentes de Debug

---

## 16.1 KVDebugToolbar

Toolbar durante debug.

Ações:

```text
Continue
Pause
Step Over
Step Into
Step Out
Restart
Stop
```

Visual:

- aparece quando sessão ativa;
- usa ícones compactos;
- Debug ativo pode usar roxo/orbital discreto;
- Stop usa vermelho discreto.

---

## 16.2 KVDebugSessionBadge

Badge no topo/status bar.

Exemplo:

```text
Debug: motor-control.elf
GDB attached
Paused at motor_control.cpp:18
```

Estados:

```text
Starting
Running
Paused
Stopped
Crashed
Disconnected
```

---

## 16.3 KVVariablesView

Lista de variáveis durante debug.

Regras:

- Tree virtualizada;
- valores alterados destacados suavemente;
- watch expressions separadas;
- tipos longos truncados com tooltip;
- hex/decimal toggle para sistemas embarcados.

---

## 16.4 KVCallStackView

Lista de frames.

Visual:

```text
Frame index
Function name
File:line
Thread badge
```

Regras:

- frame atual destacado;
- threads agrupáveis;
- navegar com clique.

---

# 17. Componentes de sistemas embarcados

---

## 17.1 KVTargetBoardCard

Card de placa/target.

Campos:

```text
Board name
MCU/CPU
Architecture
Flash size
RAM size
Probe
OpenOCD config
Serial port
Deploy command
Debug command
```

Estados:

```text
Connected
Disconnected
Unknown
Flashing
Running
Debugging
Error
```

---

## 17.2 KVFlashPanel

Painel para gravar firmware.

Conteúdo:

```text
Artifact selecionado
Target board
Probe
Flash address
Erase option
Verify option
Botão Flash
Log compacto
```

Regra:

- Deve pedir confirmação para ações destrutivas.
- Deve mostrar progresso claro.
- Deve permitir copiar comando.

---

## 17.3 KVSerialMonitor

Monitor serial embutido.

Controles:

```text
Port
Baud rate
Data bits
Parity
Stop bits
Connect/Disconnect
Clear
Save log
Filter
Timestamp toggle
```

Visual:

- terminal-like;
- texto monoespaçado;
- filtros no header;
- status de conexão evidente.

---

## 17.4 KVRemoteTargetCard

Para Linux embarcado/remote development.

Campos:

```text
Host
User
Architecture
SSH status
Remote path
Deploy strategy
Debugger server
Sync status
```

Ações:

```text
Connect
Deploy
Run remote
Debug remote
Open terminal
```

---

# 18. Assistente

---

## 18.1 Propósito

Assistente é o painel lateral inteligente da Kinein.

Ele não deve parecer chatbot genérico.  
Ele deve parecer um **painel contextual de engenharia**.

Funções:

- explicar arquivo atual;
- explicar erro de compilação;
- sugerir correções CMake;
- sugerir configuração de compiler/toolchain;
- explicar linker errors;
- sugerir target/debug config;
- apontar documentação local/projeto;
- gerar comandos, mas não executar sem confirmação.

---

## 18.2 Estrutura do painel

```text
Header: Assistente
Tabs: Context | Explain | Fix | Toolchain | Docs
Content cards
Prompt input
Action footer opcional
```

---

## 18.3 KVContextCard

Card padrão do Assistente.

Visual:

```text
Background: #171B21
Border: #343A44
Radius: 8px
Padding: 12px
Title: primary
Description: secondary
Actions: right/bottom
```

Tipos:

```text
Summary
Suggestion
Warning
Fix
Doc link
Toolchain issue
Build explanation
Symbol explanation
```

---

## 18.4 KVSuggestionCard

Sugestão aplicável.

Exemplo:

```text
Verificação de dt
Considere validar dt para evitar valores não positivos.
[Ver diff] [Aplicar sugestão]
```

Regras:

- Toda sugestão que altera arquivo deve mostrar diff antes.
- Nunca aplicar automaticamente.
- Deve indicar escopo: arquivo atual, CMake, workspace, toolchain.
- Deve indicar confiança se necessário.

---

## 18.5 KVContextInput

Input inferior do painel.

Placeholder:

```text
Pergunte ao KV sobre este arquivo, build ou toolchain...
```

Visual:

```text
Height: 36px a 42px
Background: #0F1216
Border: #343A44
Send icon: amber
```

Regras:

- Enter envia.
- Shift+Enter quebra linha.
- Deve haver modo de contexto visível: arquivo, seleção, build log, workspace.

---

# 19. Command Palette

---

## 19.1 KVCommandPalette

A command palette é essencial para experiência profissional.

Atalho sugerido:

```text
Ctrl+Shift+P
```

Visual:

```text
Overlay central superior
Width: 720px
Input: 44px
Lista: até 12 resultados visíveis
Background: #171B21
Border: #343A44
Radius: 12px
```

Conteúdo de item:

```text
Ícone
Nome do comando
Categoria
Atalho
```

Comandos iniciais:

```text
Open Workspace
Configure CMake
Build Project
Run Project
Debug Project
Open Terminal
Switch Target
Switch Build Profile
Detect Toolchains
Open Settings
Toggle Assistente
Focus Editor
```

---

# 20. Settings UI

---

## 20.1 KVSettingsWindow

Settings deve ser profissional e fácil de navegar.

Layout:

```text
Sidebar de categorias à esquerda
Conteúdo à direita
Busca no topo
Botões Apply/Cancel/OK no rodapé, se necessário
```

Categorias iniciais:

```text
General
Appearance
Editor
Keymap
C/C++
Rust
CMake
Toolchains
Build & Run
Debug
Embedded Targets
Terminal
Assistente
Plugins/Extensions
```

Regras:

- Settings devem ser pesquisáveis.
- Cada opção deve ter descrição curta.
- Configurações avançadas devem ser recolhíveis.
- Mudanças perigosas devem ter aviso.

---

# 21. Onboarding e empty states

---

## 21.1 Welcome Screen

Tela inicial quando nenhum projeto está aberto.

Conteúdo:

```text
KV logo
Kinein
Open Workspace
New CMake Project
New Rust Project
Clone Repository
Recent Projects
Detect Toolchains
Documentation
```

Visual:

- limpa;
- escura;
- poucos cards;
- destaque para abrir projeto;
- sem estética de marketing exagerada.

---

## 21.2 Empty Project State

Quando workspace abriu, mas não há projeto detectado.

Mensagem:

```text
Nenhum projeto CMake, Cargo ou build configurável foi detectado.
```

Ações:

```text
Create CMakeLists.txt
Create CMakePresets.json
Detect Toolchain
Open Folder Settings
```

---

## 21.3 Empty Toolchain State

Quando compilador/CMake não encontrados.

Mensagem:

```text
A Kinein não encontrou uma toolchain C/C++ configurada.
```

Mostrar:

```text
GCC: missing
Clang: found/not found
CMake: found/not found
Ninja: found/not found
Rust: found/not found
```

Ações:

```text
Detect Again
Open Toolchain Settings
Copy Install Commands
```

Regra:

- A IDE nunca deve instalar automaticamente sem confirmação explícita.

---

# 22. Notificações

---

## 22.1 KVNotification

Notificações não devem atrapalhar o código.

Posição recomendada:

```text
Canto inferior direito, acima da status bar
```

Tipos:

```text
Info
Success
Warning
Error
Progress
```

Visual:

```text
Width: 320px a 420px
Radius: 8px
Background: #171B21
Border conforme severidade
Ícone pequeno
Título + descrição curta
Ações opcionais
```

Exemplos:

```text
Build completed successfully.
CMake configure failed.
Toolchain detected: clang++ 17.
Flash finished in 3.2s.
```

---

## 22.2 Progress Notification

Para operações longas:

```text
Configuring CMake
Building target
Flashing firmware
Indexing project
Syncing remote target
```

Visual:

- barra fina;
- opção cancelar quando possível;
- link para abrir log.

---

# 23. Status Bar

---

## 23.1 KVStatusBar

Altura:

```text
24px a 26px
```

Itens recomendados:

```text
KV mark
Git branch
Dirty status
Problems count
Build status
CMake profile
Target
Compiler
Debug status
Cursor position
Encoding
Line ending
Language mode
Notifications
```

Visual:

```text
Background: #0B0D10 ou #111418
Border top: divider
Text: muted/secondary
Active items: primary/amber
```

Regra:

- Status bar deve informar, não decorar.
- Cada item clicável deve ter tooltip.
- Itens menos usados podem ir para overflow em telas pequenas.

---

# 24. Menus e contexto

---

## 24.1 KVMenu

Menu padrão para contexto e top menu.

Visual:

```text
Item height: 28px
Padding horizontal: 10px
Icon area: 22px
Shortcut aligned right
Separator subtle
Submenu arrow muted
```

Estados:

```text
Hover: #1A1F27
Selected keyboard: #222833
Disabled: muted + opacity
Danger item: red text/icon
```

---

## 24.2 Context menus importantes

### Editor context menu

```text
Go to Definition
Find Usages
Rename
Refactor
Format Selection
Run Tests
Explain with Assistente
Fix with Assistente
```

### Project tree context menu

```text
New File
New Folder
Rename
Delete
Reveal in Terminal
Add to CMake Target
Mark as Generated
Exclude from Index
```

### CMake file context menu

```text
Configure Project
Reload CMake
Open Presets
Explain CMake Target
```

---

# 25. Regras de animação

Animações devem ser discretas e funcionais.

```text
Hover transition:        80ms a 120ms
Panel open/close:        120ms a 180ms
Popover open:            80ms a 120ms
Dialog open:             120ms a 160ms
Loading spinner:         contínuo, discreto
Progress bar:            linear, sem exagero
```

Não usar:

- bounce;
- elastic;
- partículas;
- brilho animado constante;
- transições longas;
- animações que atrasem o fluxo.

A IDE deve parecer responsiva.

---

# 26. Acessibilidade e conforto

## 26.1 Contraste

- Texto principal deve ter contraste alto.
- Texto secundário deve ser legível por longas sessões.
- Não usar cinza escuro demais para informações importantes.
- Erro/sucesso não devem depender só de cor; usar ícone e texto.

## 26.2 Teclado

Todos os componentes críticos devem ser navegáveis por teclado:

- botões;
- menus;
- command palette;
- settings;
- abas;
- project tree;
- Assistente actions;
- dialogs.

## 26.3 Foco

Foco deve ser visível, especialmente para:

- inputs;
- botões;
- tree items;
- command palette;
- tabs;
- selects;
- menus.

## 26.4 Movimento reduzido

Settings deve permitir:

```text
Reduce animations
Disable glow effects
Increase contrast
Increase UI font size
```

---

# 27. Integração Qt/QML

## 27.1 Estrutura sugerida de componentes

```text
ui/
  qml/
    App/
      KVAppShell.qml
      KVTopBar.qml
      KVStatusBar.qml
    Components/
      KVButton.qml
      KVIconButton.qml
      KVInput.qml
      KVSelect.qml
      KVCheckbox.qml
      KVToggle.qml
      KVBadge.qml
      KVChip.qml
      KVTooltip.qml
      KVPopover.qml
      KVDialog.qml
      KVMenu.qml
      KVTreeView.qml
      KVTreeItem.qml
      KVSplitter.qml
      KVPanelHeader.qml
      KVNotification.qml
    Editor/
      KVEditorArea.qml
      KVEditorTabBar.qml
      KVEditorTab.qml
      KVBreadcrumbs.qml
      KVGutter.qml
    ToolWindows/
      KVToolWindow.qml
      ProjectToolWindow.qml
      BuildToolWindow.qml
      CMakeToolWindow.qml
      DebugToolWindow.qml
      TargetsToolWindow.qml
      SerialToolWindow.qml
      KVContextToolWindow.qml
    Theme/
      KVTheme.qml
      KVColors.qml
      KVSpacing.qml
      KVTypography.qml
      KVIcons.qml
```

---

## 27.2 Tokens centralizados

Nunca hardcodar cores diretamente nos componentes.

Usar algo como:

```qml
KVTheme.colors.background
KVTheme.colors.panel
KVTheme.colors.panelElevated
KVTheme.colors.borderSubtle
KVTheme.colors.textPrimary
KVTheme.colors.textSecondary
KVTheme.colors.amberPrimary
```

Espaçamentos:

```qml
KVTheme.spacing.xs
KVTheme.spacing.sm
KVTheme.spacing.md
KVTheme.spacing.lg
```

Raios:

```qml
KVTheme.radius.sm
KVTheme.radius.md
KVTheme.radius.lg
```

---

## 27.3 Contratos de componentes

Cada componente QML deve ter propriedades claras.

Exemplo conceitual:

```qml
KVButton {
    text: "Configure"
    iconName: "cmake-configure"
    variant: "primary"
    size: "default"
    enabled: true
    loading: false
    onClicked: core.configureCMake()
}
```

Exemplo:

```qml
KVSelect {
    label: "Target"
    value: "x86_64-linux-gnu"
    status: "ok"
    model: targetModel
}
```

---

# 28. Checklist de implementação para IA CLI

Ao implementar componentes, seguir esta ordem:

```text
1. Criar KVTheme com tokens.
2. Criar KVIcon wrapper.
3. Criar KVButton e KVIconButton.
4. Criar KVInput e KVSelect.
5. Criar KVPanelHeader.
6. Criar KVToolWindow base.
7. Criar KVActivityBar.
8. Criar KVTopBar.
9. Criar KVStatusBar.
10. Criar KVTreeView e KVTreeItem.
11. Criar KVEditorTabBar.
12. Criar KVNotification.
13. Criar KVDialog.
14. Integrar tudo no KVAppShell.
```

Não começar por telas avançadas antes de ter os componentes base.

---

# 29. Critérios de qualidade visual

Antes de aceitar uma implementação, verificar:

```text
A UI parece profissional em 1920x1080?
A UI ainda é legível em 1366x768?
O editor continua sendo o foco?
A toolbar está compacta?
Os painéis não brigam por atenção?
O âmbar está sendo usado com moderação?
O estado ativo é claro?
O hover é perceptível?
O foco por teclado é visível?
O terminal é legível?
A status bar informa sem poluir?
Os componentes parecem pertencer ao mesmo sistema?
A IDE parece confortável para 6 horas de uso?
```

---

# 30. Anti-padrões

Evitar:

```text
Botões primários demais
Bordas amarelas em todos os elementos
Textos pequenos demais
Cinza com contraste baixo demais
Sidebar com ícones sem tooltip
Cards com sombras exageradas
Terminal com fonte ruim
Settings sem busca
Dialogs gigantes
Painéis que abrem sozinhos
Assistente interrompendo fluxo
Animações decorativas
Ícones inconsistentes
Layout que parece landing page
```

---

# 31. Direção final

A biblioteca de componentes da Kinein deve sustentar a IDE por muitos anos.

A meta não é apenas deixar bonito.  
A meta é criar uma base visual que permita escalar de:

```text
MVP com editor + explorer + terminal
```

para:

```text
IDE completa para C/C++/Rust, CMake, debug, toolchains, sistemas embarcados, Linux embarcado, targets remotos, QEMU e simulação física/matemática.
```

Cada componente deve passar a sensação de:

```text
controle
clareza
força técnica
conforto visual
engenharia real
```

A frase-guia continua sendo:

```text
Focus on code, not toolchains.
```

E a regra de produto:

```text
A Kinein deve deixar projetos C, C++ e Rust menos intimidadores sem esconder o poder real das ferramentas.
```

---

## 32. Próximo documento recomendado

A próxima parte deve ser:

```text
Parte 4 — Fluxos de Produto: CMake, Toolchain, Build, Run e Debug
```

Esse documento deve detalhar:

- fluxo de abrir workspace;
- detecção de projeto;
- configuração CMake;
- seleção de compiler;
- build profiles;
- run configurations;
- debug local;
- erros e recuperação;
- como Assistente deve ajudar sem atrapalhar.
