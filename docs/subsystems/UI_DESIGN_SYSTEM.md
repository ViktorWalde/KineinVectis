# Kernwerk Studio — UI Design System

> Este documento define o sistema visual do Kernwerk Studio. A interface deve ser rígida, previsível, confortável, JetBrains-like em memória muscular, mas sem copiar assets, marca, layout proprietário ou identidade visual de qualquer empresa.

---

## 1. Objetivo

O Kernwerk precisa de uma UI consistente.

Uma IDE profissional não deve ter telas improvisadas. Todos os painéis, botões, badges, menus, toolbars e diálogos devem seguir um Design System.

Regra:

```text
Nenhuma tela deve criar componente visual improvisado fora do Design System.
```

---

## 2. Princípios visuais

```text
consistência;
baixa fadiga visual;
alta previsibilidade;
densidade ajustável;
atalhos fortes;
painéis estáveis;
pouca animação;
sem layout mágico;
tema escuro confortável;
suporte a tela pequena;
sem telemetria visual/distrações.
```

O usuário deve sentir:

```text
"Eu sei onde as coisas ficam."
"Eu consigo usar por horas."
"Eu consigo operar por teclado."
"Eu entendo o que a IDE está fazendo."
```

---

## 3. Tokens visuais

Tokens são valores oficiais do design.

### 3.1 Cores

A paleta deve ser definida por tokens, não valores soltos.

```text
kw.bg.0          fundo principal
kw.bg.1          painéis
kw.bg.2          superfície elevada
kw.border.0      borda suave
kw.text.0        texto principal
kw.text.1        texto secundário
kw.text.disabled texto desabilitado
kw.accent        cor de destaque
kw.warning       aviso
kw.error         erro
kw.success       sucesso
kw.info          informação
```

Nenhuma tela deve usar cor direta sem passar por token.

---

## 4. Tipografia

Definir:

```text
fonte da UI;
fonte do editor;
tamanho padrão;
tamanho compacto;
altura de linha;
peso de fonte;
tamanho mínimo.
```

Sugestões:

```text
UI: Inter, Noto Sans ou system font.
Editor: JetBrains Mono, Fira Code, Cascadia Code ou monospace do sistema.
```

Regra:

```text
A fonte do editor deve ser configurável.
```

---

## 5. Espaçamento

Tokens:

```text
space.1 = 4px
space.2 = 8px
space.3 = 12px
space.4 = 16px
space.5 = 24px
```

Tamanhos importantes:

```text
top bar: 40px
status bar: 24px
sidebar compacta: 44px
painel mínimo: 240px
altura de item em lista compacta: 28px
altura de item confortável: 34px
```

---

## 6. Componentes oficiais

Componentes básicos:

```text
KButton
KIconButton
KToggle
KCheckbox
KRadio
KTextField
KSelect
KTooltip
KDialog
KMenu
KContextMenu
KNotification
KBadge
KStatusBadge
KProgressBar
KSpinner
```

Componentes de IDE:

```text
KToolWindow
KPanel
KPanelHeader
KTabBar
KEditorTab
KProjectTree
KProblemItem
KBuildOutput
KTerminalPanel
KCommandPalette
KSettingsRow
KToolStatusCard
KTaskItem
KRunConfigSelector
KQualityRuleCard
```

---

## 7. Estados dos componentes

Todo componente interativo deve ter estados:

```text
default
hover
active
focused
disabled
loading
danger
selected
error
warning
success
```

Regra:

```text
Estado de foco deve ser visível para navegação por teclado.
```

---

## 8. Layout base

Layout desktop:

```text
┌──────────────────────────────────────────────────────────────┐
│ Top Bar                                                      │
├────┬───────────────────────────────┬─────────────────────────┤
│    │ Editor Tabs                   │ Right Panel             │
│Bar │ Editor                        │ Inspector / AI / Docs   │
│    │                               │                         │
├────┴───────────────────────────────┴─────────────────────────┤
│ Terminal | Problems | Build | Git | Debug | Tasks | Tests     │
├──────────────────────────────────────────────────────────────┤
│ Status Bar                                                   │
└──────────────────────────────────────────────────────────────┘
```

Tela pequena:

```text
Right Panel fechado por padrão.
Bottom panel com altura reduzida.
Sidebar compacta.
Command Palette como acesso principal.
```

---

## 9. Painéis fixos

Painéis principais:

```text
Project
Search
Git
Run
Debug
Embedded & Remote
Quality
Tasks
Settings
```

Regra:

```text
A IDE não deve mover painéis automaticamente sem confirmação.
```

Ela pode sugerir:

```text
Detectamos tela pequena. Deseja ativar layout compacto?
```

---

## 10. Atalhos e memória muscular

Atalhos padrão:

```text
Shift Shift       Search Everywhere
Ctrl+Shift+A     Find Action
Alt+Enter        Quick Fix
Ctrl+B           Go to Definition
Ctrl+Alt+B       Go to Implementation
Ctrl+Alt+L       Format Code
Shift+F6         Rename
Ctrl+Shift+F     Search in Files
Ctrl+E           Recent Files
Alt+1            Project
Alt+4            Run/Build
Alt+5            Debug
Alt+9            Git
```

Regra:

```text
Todos os atalhos devem chamar comandos registrados no Command System.
```

---

## 11. Tema escuro confortável

Regras:

```text
não usar preto puro em tudo;
não usar branco puro em texto longo;
evitar amarelo forte em grandes áreas;
usar contraste suficiente;
não depender só de cor para erro;
usar ícone + texto + cor;
permitir ajuste de fonte;
permitir reduzir animações.
```

---

## 12. Animações

Animações devem ser discretas.

Permitido:

```text
fade suave;
expansão de painel curta;
hover leve.
```

Evitar:

```text
animações longas;
efeitos chamativos;
movimento constante;
transições que atrasam trabalho.
```

Configuração:

```text
Reduce Motion: on/off
```

---

## 13. Quality Center visual

Regras visuais:

```text
cada regra tem card;
cada regra mostra Trust Level;
cada regra tem explicação;
cada regra mostra impacto;
cada regra mostra arquivos gerados;
cada regra mostra quando evitar.
```

Exemplo:

```text
[✓] Pedantic errors
Trust: Compiler official
Impact: High strictness
Generated: -Wpedantic -pedantic-errors
```

---

## 14. Embedded & Remote visual

Painel deve separar:

```text
Targets
Toolchains
Deploy
Debug
Serial
QEMU
SDKs
Logs
```

Ações perigosas devem usar estilo danger e confirmação.

---

## 15. Critérios de aceite

O Design System está pronto para MVP quando:

```text
tokens existem;
componentes básicos existem;
layout base existe;
tema escuro existe;
status bar existe;
task item existe;
problem item existe;
settings row existe;
nenhuma tela usa cor hardcoded sem token;
atalhos passam pelo command system.
```

---

## 16. Decisão final

A UI do Kernwerk deve ser rígida como o Core.

Uma interface bonita sem sistema vira bagunça.  
Uma interface com Design System consegue crescer com consistência.
