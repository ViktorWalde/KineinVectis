# 05 — Design System

## Identidade

Nome: **Kernwerk Studio**

Símbolo: **KW**

Estilo: técnico, escuro, minimalista, Linux-first, inspirado em estética de distros Arch Linux, porém com identidade própria.

## Referências visuais

Assets de referência aprovados em `imagens/`:

- `imagens/layout-mockup.png` — mockup do layout completo da IDE: tema escuro
  quase-preto, acento âmbar `#ffbb00`, explorer à esquerda, editor central com
  abas, painel assistente à direita, terminal/build embaixo, status bar com
  branch e tagline "Linux-first, feito para engenheiros".
- `imagens/app-icon.png` — ícone oficial: monograma KW (K branco/prata, W
  âmbar/dourado) sobre fundo escuro arredondado com borda dourada sutil.

A UI Qt/QML (MVP 0.4) deve seguir estes assets e a paleta abaixo.

## Paleta principal

```text
Accent escuro:    #6e5c01
Texto claro:      #eae6e1
Accent forte:     #ffbb00
Neutro oliva:     #6e6c58
```

## Paleta auxiliar sugerida

```text
Background 0:     #0d0e0e
Background 1:     #121313
Background 2:     #191a18
Surface 1:        #1f201d
Surface 2:        #25261f
Border soft:      #2a2922
Text primary:     #eae6e1
Text secondary:   #b9b3a5
Text muted:       #8f8a7c
Accent:           #ffbb00
Accent dim:       #6e5c01
Neutral olive:    #6e6c58
Error soft:       #d16d6d
Warning soft:     #ffbb00
Success soft:     #7fbf7f
Info soft:        #7aa2d8
```

## Tema padrão

O tema padrão é escuro, confortável para longas horas.

Princípios:

- baixo brilho geral;
- contraste suficiente sem agredir os olhos;
- amarelo usado como destaque, não como cor dominante;
- painéis com bordas sutis;
- código no centro com distrações reduzidas;
- árvore de projeto leve, aberta e escaneável, com conforto visual próximo ao
  Project View das IDEs JetBrains;
- elementos previsíveis para memória muscular.

## Densidade

Padrão:

```text
Confortável
```

Opções futuras:

```text
Compacta
Confortável
Espaçosa
```

## Tipografia

Sugestões:

- Editor: JetBrains Mono, Fira Code, Cascadia Code ou monospace do sistema.
- UI: Inter, Noto Sans, ou fonte do sistema KDE.
- Terminal: JetBrains Mono ou monospace do sistema.

## Layout principal

```text
┌──────────────────────────────────────────────────────────────┐
│ Top Bar: Projeto | Branch | Run Config | Build | Debug | IA  │
├──────┬───────────────────────────────┬───────────────────────┤
│ Left │ Editor Tabs                   │ Right Tool Window     │
│ Bar  │                               │ AI / Docs / Inspector │
│      │ Código                        │                       │
├──────┴───────────────────────────────┴───────────────────────┤
│ Bottom Tool Window: Terminal | Problems | Build | Git | Debug│
├──────────────────────────────────────────────────────────────┤
│ Status Bar: Git | CMake | clangd | Java | Python | Encoding  │
└──────────────────────────────────────────────────────────────┘
```

## Árvore de projeto

A árvore de projeto deve priorizar conforto e legibilidade em sessões longas.
Referência de sensação: Project View das IDEs JetBrains, sem copiar assets ou
identidade visual proprietária.

Direção visual:

- linhas densas, mas com respiro suficiente para leitura;
- fundo do painel discreto, sem parecer uma caixa pesada ou fechada;
- seleção e hover sutis, usando contraste baixo e acento apenas quando útil;
- ícones pequenos e funcionais para pasta, arquivo, expansão e ações;
- indentação clara, sem conectores visuais excessivos;
- cabeçalho simples, com nome do projeto e ações compactas;
- botões de ação preferencialmente por ícone com tooltip futuro, não botões
  textuais grandes.

Evitar:

- painel de explorer com aparência pesada, saturada ou excessivamente
  contrastada;
- excesso de bordas internas, cards ou caixas dentro da árvore;
- comportamento de navegação parecido com gerenciador de arquivos;
- trocar workspace ao clicar em diretório dentro da árvore.

## Memória muscular

Atalhos e localização de painéis devem ser familiares para usuários JetBrains:

```text
Alt+1        Project
Alt+4        Run/Build
Alt+5        Debug
Alt+9        Git
Alt+Enter    Quick Fix
Shift+F6     Rename
Ctrl+B       Go to Definition
Ctrl+Alt+L   Format Code
Shift Shift  Search Everywhere
```

## Regra visual

Não mover automaticamente painéis de lugar. A IDE pode sugerir, mas nunca reorganizar o workspace sem ação explícita do usuário.
