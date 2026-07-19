# Kinein Vectis — Sistema Visual da IDE

**Parte 2 — Layout principal da IDE**  
**Escopo:** arquitetura visual, organização de painéis, estados de interface, padrões de layout real e separação explícita do layout de divulgação/marketing.  
**Produto:** Kinein Vectis  
**Nome de uso diário:** Kinein  
**Sigla visual:** KV  
**Foco técnico:** C, C++, Rust, CMake, toolchains, sistemas embarcados, Linux embarcado, software embarcado, debug, flash, simulação, OpenGL e análise física/matemática.

---

## 1. Objetivo deste documento

Este documento define o layout principal da IDE Kinein Vectis para orientar implementação, design, geração de mockups e uso por IA CLI.

A meta não é criar uma tela promocional, mas sim uma interface real, utilizável, confortável e escalável para desenvolvimento profissional em C, C++ e Rust.

O layout deve transmitir a sensação de:

- familiaridade imediata para usuários de IDEs profissionais;
- conforto visual para sessões longas de código;
- controle sobre CMake, compiladores, targets e toolchains;
- produtividade sem poluição visual;
- engenharia séria, sem estética lúdica/cartoon;
- identidade própria baseada em KV, âmbar industrial, vetores, circuitos e malhas técnicas.

A referência mental é:

```text
JetBrains-like na sensação de organização e conforto.
Kinein-like na identidade visual, iconografia, foco em sistemas e linguagem técnica.
```

Isto significa: não copiar ícones, formas, cores ou layout específico de nenhum produto. A inspiração está na experiência psicológica: previsibilidade, densidade controlada, painéis consistentes, atalhos familiares e foco no editor.

---

## 2. Premissas de produto

### 2.0 Referência de UI/UX: IntelliJ IDEA Community (invariante, 2026-07-18)

Estas specs têm como premissa **memória muscular JetBrains**. Junto delas,
vale como referência viva o **IntelliJ IDEA Community**: comportamento,
idioma visual, disposição das regiões, atalhos e aproveitamento das bordas
da janela são estudados nele antes de qualquer decisão de layout aqui.

Limites desta invariante, já contratuais (`ARCHITECTURE.md` §2.1):

- importa-se **invariante, decisão, modo de falha e estratégia de teste**;
  nunca código, Swing, IntelliJ Platform ou modelo interno;
- o que se vê lá é **redesenhado** no fluxo nativo Qt/QML, com o Theme e a
  iconografia da Kinein;
- onde o contexto diverge (C/C++/Rust, offline-first, sem host de
  extensões, chrome frameless próprio), **vence o contexto da Kinein**;
- a UI permanece **burra**: representa estado e emite intenção; regra de
  negócio mora no core e nos controllers, nunca no QML visual.

A Kinein Vectis não deve ser apenas um editor de código.

Ela deve ser uma IDE de engenharia de sistemas, capaz de ajudar o programador a lidar com:

- criação de projetos C/C++/Rust;
- configuração de CMake;
- escolha de compilador;
- seleção de target;
- presets de build;
- Debug/Release/RelWithDebInfo;
- cross-compilation;
- toolchain files;
- sysroot;
- SDKs;
- debug local/remoto;
- serial monitor;
- flash de firmware;
- execução em Linux embarcado;
- simulação e visualização futuras com OpenGL.

A promessa de UX é:

```text
Focus on code, not toolchains.
```

Ou em português:

```text
Foque no código, não no labirinto de toolchains.
```

---

## 3. Separação obrigatória: layout real vs layout de divulgação

A Kinein deve ter dois tipos de visual, com objetivos diferentes.

### 3.1 Layout real da IDE

É a interface que o usuário usa no dia a dia.

Características:

- sem textos promocionais ao redor;
- sem callouts explicativos fora da janela;
- sem linhas amarelas apontando recursos;
- sem slogan dentro da área de trabalho;
- sem cards decorativos;
- sem excesso de brilho;
- sem elementos flutuando fora da janela;
- foco total em editor, projeto, build, debug, terminal e contexto.

O layout real deve parecer uma aplicação instalada, não uma arte de apresentação.

### 3.2 Layout de divulgação/marketing

É usado em README, landing page, banners, posts, apresentação e pitch visual.

Pode conter:

- callouts laterais;
- slogan;
- textos como “Linux-first”, “feito para engenheiros”, “C/C++ e Rust”;
- setas ou linhas apontando recursos;
- cards de destaque;
- mockup central da IDE;
- app icon grande;
- selo de projeto open source.

Regra fundamental:

```text
Marketing explica o produto.
A IDE executa o trabalho.
```

Não misturar os dois dentro da interface real.

---

## 4. Arquitetura visual geral

A janela principal da Kinein é dividida em **sete** regiões. [Eram oito até
2026-07-18: o autor aprovou a fusão de Title/App Bar + Main Toolbar numa App
Bar única de 46px, no idioma do IntelliJ New UI — menus atrás do hambúrguer,
toolbar e controles de janela embutidos na mesma linha.]

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ 1. App Bar (única): ☰ menus · marca · workspace · toolbar · janela          │
├────┬─────────────────────┬──────────────────────────────────┬───────────────┤
│ 2  │ 3. Left Tool Window │ 4. Editor Area                   │ 5. Assistente │
│Rail│                     │                                  │               │
├────┴─────────────────────┴──────────────────────────────────┴───────────────┤
│ 6. Bottom Tool Window                                                        │
├──────────────────────────────────────────────────────────────────────────────┤
│ 7. Status Bar                                                               │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 4.1 Regiões

| Região | Nome | Função |
|---|---|---|
| 1 | App Bar (única) | Identidade, menus (hambúrguer), target, perfil, build, run, debug, controles de janela |
| 2 | Tool Rail | Ícones verticais de janelas/ferramentas |
| 3 | Left Tool Window | Project, Structure, CMake, Toolchains, Targets |
| 4 | Editor Area | Código, tabs, breadcrumbs, gutter, diagnósticos |
| 5 | Assistente | Assistente contextual, explicação, correções e toolchain |
| 6 | Bottom Tool Window | Terminal, Problems, Build, CMake, Debug, Serial, Simulation, Git |
| 7 | Status Bar | Estado do projeto, branch, target, warnings, encoding, posição |

### 4.2 Modelo de superfície: regiões encostadas (2026-07-18)

Decisão do autor, após dogfooding: o modelo anterior — cada região como um
cartão com contorno próprio, flutuando sobre calhas de 8px — deixava a IDE
"espalhada" e "poluída", com margem morta nas bordas da janela. O modelo
atual segue o aproveitamento de tela do IntelliJ IDEA Community (§2.0):

```text
1. Regiões ENCOSTADAS umas nas outras e nas bordas da janela.
   Não existe margem entre o workspace e a janela; o rail esquerdo E' a
   borda (mesmo fundo da janela, ícones "no plano de fundo").
2. O divisor entre regiões é o fundo da janela (background0) aparecendo
   por 1px (`seamWidth`). Não existe Rectangle de borda para isso.
3. Nenhuma região tem contorno (border). Os PAINÉIS de conteúdo (Project,
   editor, Bottom Tool Window) têm raio (radiusLarge) sem contorno, sobre o
   fundo da janela — como as tool windows da JetBrains. A MOLDURA (rail,
   barras do topo, status bar) é plana: ela é a própria janela. [Corrigido
   em 2026-07-18: a 1ª versão desta seção zerou TODO raio e o autor apontou
   a regressão no aceite — "ficou layout do VS Code". Raio de região não era
   o problema; o problema era contorno + calha.] O arredondamento também
   mora DENTRO: chip de hover/seleção, aba, botão, popup, diálogo e banner.
4. A alça de redimensionamento é invisível: `splitterGrip` (7px) montado
   SOBRE o divisor; só a linha de hover pinta (âmbar, 2px).
5. Tons marcam as transições onde o divisor não aparece: rail e App Bar em
   background0; painéis em background1/background2; editor dominante.
```

O que continua sendo cartão, de propósito: diálogos, popups, notificações
(banners) e a tela inicial — são sobreposições, não regiões do layout.

Racional: eliminar ~10 contornos de 1px competindo e 16px de margem morta
por eixo; o olho passa a ler UMA superfície com divisões, não uma pilha de
cartões. Ver `ui/qml/Theme.qml` (`seamWidth`, `splitterGrip`).

---

## 5. Princípios de UX

### 5.1 O editor é a área sagrada

A área central deve ser sempre dominante.

Regras:

- nenhum painel deve roubar atenção do editor sem ação explícita do usuário;
- Assistente deve ajudar, não interromper;
- notificações devem ser discretas;
- erros críticos aparecem primeiro no editor e no Problems, não como popups agressivos;
- builds longos devem aparecer na barra inferior, não bloquear a tela.

### 5.2 Densidade controlada

A Kinein é uma IDE técnica e densa, mas a densidade precisa parecer controlável.

A interface deve evitar:

- toolbars enormes;
- ícones demais na mesma linha;
- texto pequeno demais;
- painéis abertos por padrão sem necessidade;
- excesso de abas simultâneas;
- listas sem agrupamento.

### 5.3 Familiaridade sem cópia

Usuários vindos de IDEs profissionais devem entender a Kinein no primeiro minuto.

Padrões familiares:

- projeto à esquerda;
- editor no centro;
- terminal/build/debug embaixo;
- contexto/assistente à direita;
- status bar no rodapé;
- toolbar superior com target e botões de execução.

Identidade própria:

- KV como assinatura visual;
- âmbar industrial como cor ativa;
- ícones baseados em vetores, chevrons, circuitos e malhas;
- foco explícito em C/C++/Rust, CMake, toolchains e sistemas.

### 5.4 Conforto psicológico

A interface deve reduzir ansiedade.

O usuário não deve sentir:

- “não sei onde configurar o compilador”;
- “não sei se estou buildando Debug ou Release”;
- “não sei qual target está ativo”;
- “não sei por que o CMake falhou”;
- “não sei onde ver serial/debug/logs”.

A Kinein deve deixar sempre visível:

- target ativo;
- perfil CMake;
- compilador/toolchain;
- estado do build;
- branch Git;
- quantidade de erros/warnings;
- posição no arquivo;
- linguagem/modo do arquivo.

---

## 6. Grid, dimensões e espaçamento

### 6.1 Janela base para mockups

Usar estes tamanhos para design e imagem de referência:

| Uso | Resolução |
|---|---:|
| Mockup padrão | 1920 × 1080 |
| Mockup 2x | 3840 × 2160 |
| Screenshot marketing | 2560 × 1440 |
| UI internal test | 1600 × 900 |

### 6.2 Escala de espaçamento

Usar escala de 4px.

```text
1px  — divisor entre regiões (seamWidth, §4.2) — a única separação entre elas
2px  — linhas finas, divisores internos
4px  — micro espaçamento
8px  — espaçamento padrão de componentes pequenos
12px — espaçamento entre blocos
16px — padding de painel
24px — separação de seções DENTRO de uma região (nunca entre regiões)
32px — seções grandes
```

### 6.3 Alturas recomendadas

| Elemento | Altura |
|---|---:|
| App Bar (única, desde 2026-07-18) | 46px |
| Tab Bar | 36px |
| Breadcrumb Bar | 26px |
| Status Bar | 28px |
| Bottom Tool Tabs | 34px |
| Linha de item no Project | 24px |
| Linha de código | 21–24px |
| Botão pequeno | 28px |
| Botão toolbar | 32px |

### 6.4 Larguras recomendadas

| Região | Largura padrão | Mínimo | Máximo |
|---|---:|---:|---:|
| Tool Rail esquerdo | 52px | 48px | 56px |
| Left Tool Window | 280px | 220px | 420px |
| Editor Area | flexível | 600px | ilimitado |
| Assistente | 360px | 300px | 480px |
| Rail direito opcional | 44px | 40px | 52px |
| Bottom Tool Window | 260px altura | 160px | 480px |

---

## 7. Paleta visual para layout

A paleta deve ser confortável, escura e com contraste equilibrado.

### 7.1 Tokens principais

```text
--kv-bg-app:              #0B0D10
--kv-bg-surface:          #111418
--kv-bg-elevated:         #171B21
--kv-bg-editor:           #0F1216
--kv-bg-current-line:     #1A1F26
--kv-border-subtle:       #2A2F37
--kv-border-strong:       #3A414A

--kv-text-primary:        #E7E2D8
--kv-text-secondary:      #A9A39A
--kv-text-muted:          #6F737A
--kv-text-disabled:       #4E535A

--kv-amber:               #FFB000
--kv-amber-active:        #FFC93D
--kv-amber-dark:          #B97900

--kv-blue-tech:           #5C8DFF
--kv-purple-orbital:      #8A5CFF
--kv-green-success:       #7CCF6A
--kv-red-error:           #D45F5F
--kv-yellow-warning:      #E6B84A
```

### 7.2 Regra de uso do âmbar

O âmbar é identidade, não decoração excessiva.

Usar âmbar em:

- aba ativa;
- botão Run/Build principal;
- ícone selecionado;
- progresso de build;
- foco de input;
- contorno de sugestão ativa;
- target conectado;
- highlight de recurso importante.

Evitar âmbar em:

- todo texto secundário;
- todas as bordas;
- todos os ícones ao mesmo tempo;
- blocos grandes de fundo;
- longos parágrafos;
- código comum.

---

## 8. Tipografia

### 8.1 Fonte de UI

Recomendação:

```text
Noto Sans, Inter, Segoe UI, system-ui
```

A fonte da UI deve ser neutra, legível e confortável.

### 8.2 Fonte de código

Recomendação:

```text
JetBrains Mono, Fira Code, Cascadia Code, Source Code Pro
```

A Kinein pode usar JetBrains Mono como sugestão opcional se a licença e a distribuição forem adequadas ao projeto, mas a IDE deve funcionar bem com qualquer fonte monoespaçada configurada pelo usuário.

### 8.3 Tamanhos

| Elemento | Tamanho |
|---|---:|
| Menu superior | 13px |
| Label de toolbar | 12–13px |
| Project tree | 13px |
| Editor | 14–15px |
| Terminal | 13px |
| Status bar | 12px |
| Assistente body | 13px |
| Título de painel | 13px semibold |

---

## 9. Layout real — estado padrão

Este é o layout inicial após abrir um projeto.

### 9.1 Estrutura padrão

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ ☰ Kinein · ws   [Target ▼] [Perfil ▼] [Build] [Run] [Debug]        — □ ×    │
├────┬──────────────────────┬─────────────────────────────────┬──────────────┤
│Rail│ Project              │ Editor                          │ Assistente   │
│    │ Structure            │ Tabs + Breadcrumbs              │ Context      │
│    │                      │ Code                            │ Explain      │
│    │                      │ Gutter + Diagnostics            │ Fix          │
├────┴──────────────────────┴─────────────────────────────────┴──────────────┤
│ Terminal | Problems | Build | CMake | Debug | Serial | Simulation | Git     │
├──────────────────────────────────────────────────────────────────────────────┤
│ KV main* | 0 errors | 2 warnings | CMake Debug | clang++ | x86_64 | UTF-8   │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 9.2 Painéis abertos por padrão

Para o primeiro uso:

- Project aberto à esquerda;
- Editor no centro;
- Assistente aberto à direita, mas colapsável;
- Terminal/Build inferior aberto apenas se houver processo ativo ou se o usuário abrir.

Sugestão de default:

```text
Primeiro projeto aberto: Project + Editor + Assistente.
Sessões seguintes: restaurar layout salvo pelo usuário.
```

---

## 10. App Bar (única desde 2026-07-18)

### 10.1 Função

A App Bar contém, numa única linha de 46px:

- hambúrguer (☰) que expande/recolhe os menus globais na própria barra,
  como no IntelliJ New UI — o modelo dos menus vive no `AppMenuModel.qml`;
- nome curto Kinein;
- nome do workspace aberto (some quando os menus expandem);
- região de arrasto da janela;
- cluster de toolbar (ver §11) ancorado à direita;
- controles de janela (minimizar/restaurar/fechar) EMBUTIDOS na barra.

### 10.2 Conteúdo recomendado

```text
☰  Kinein  meu-projeto        [Perfil ▼] ⚙ [Compilar] ✓ 🐞 ▶       — □ ×
```

Com o hambúrguer expandido:

```text
☰  Kinein  Arquivo Editar Exibir Navegar Código Build Executar ...  — □ ×
```

Evitar usar “Kinein Vectis” inteiro no topo diário. O nome completo aparece em:

- splash screen;
- About;
- README;
- site;
- documentação;
- tela de boas-vindas.

### 10.3 Regras visuais

- altura compacta;
- fundo quase igual ao app, levemente elevado;
- logo pequeno, não chamativo demais;
- texto claro, sem brilho;
- menu com hover sutil;
- nenhum slogan no topo.

---

## 11. Main Toolbar

A Main Toolbar é a região operacional mais importante para C/C++/Rust.

> Desde 2026-07-18 ela não é mais uma barra própria: é o **cluster de
> toolbar dentro da App Bar única** (§10), ancorado à direita antes dos
> controles de janela (`TopHeaderBar.qml`, agora um Item de largura
> implícita). A ordem e as regras abaixo continuam valendo para o cluster.

### 11.1 Ordem recomendada

```text
Target Selector
Build System / Profile
Configure
Build
Run
Debug
Flash
Simulate
Search
Settings
```

Exemplo:

```text
[Target: stm32f767zi ▼] [CMake: Debug ▼]  △  >_  >  ◇  ↯  ◎  Search  Settings
```

### 11.2 Elementos

#### Target Selector

Mostra onde o projeto vai rodar.

Exemplos:

- x86_64-linux-gnu
- aarch64-linux-gnu
- arm-none-eabi
- stm32f767zi
- qemu-aarch64
- remote-linux-board

#### Build Profile

Mostra o perfil de build ativo.

Exemplos:

- CMake: Debug
- CMake: Release
- CMake: RelWithDebInfo
- Cargo: debug
- Cargo: release

#### Configure

Ação para configurar CMake, presets, toolchain e cache.

#### Build

Ação principal de compilação.

#### Run

Executa binário/local target.

#### Debug

Inicia sessão GDB/LLDB.

#### Flash

Grava firmware em target embarcado.

#### Simulate

Abre execução/simulação quando disponível.

### 11.3 Regras de estado

| Estado | Visual |
|---|---|
| Idle | botões neutros |
| Build disponível | Build/Run âmbar suave |
| Build em andamento | progresso âmbar fino |
| Build falhou | indicador vermelho no Build e Problems |
| Debug ativo | Debug com acento roxo/azul |
| Target desconectado | Target com aviso discreto |
| Toolchain ausente | Target/Profile com warning |

---

## 12. Tool Rail esquerdo

### 12.1 Função

A Tool Rail contém apenas ícones das janelas principais.

Ordem padrão:

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

### 12.2 Regras

- ícones de 20–24px;
- rail de 52px;
- label opcional apenas se houver espaço;
- ativo em âmbar;
- hover com fundo elevado;
- rail nunca deve virar painel de propaganda.

### 12.3 Tool windows

| Ícone | Painel |
|---|---|
| Project | árvore de arquivos e estrutura |
| Search | busca global |
| Git | controle de versão |
| Build | tarefas de build |
| Debug | sessões e breakpoints |
| Targets | placas, remotos e toolchains |
| Simulate | simulação/visualização futura |
| Tools | CMake, SDK, plugins, ambientes |
| Extensions | extensões futuras |

---

## 13. Left Tool Window

### 13.1 Project

Deve ser o painel padrão.

Conteúdo:

- raiz do projeto;
- diretórios;
- arquivos;
- ícones de linguagem;
- CMakeLists.txt;
- CMakePresets.json;
- toolchain files;
- pastas build ignoradas ou minimizadas;
- markers de arquivo alterado.

Regras:

- tree indent consistente;
- nomes legíveis;
- ícones discretos;
- arquivo ativo com fundo suave e texto âmbar;
- nada de bordas fortes.

### 13.2 Structure

Abaixo do Project ou como aba alternativa.

Mostra:

- classes;
- funções;
- structs;
- enums;
- namespaces;
- métodos;
- campos;
- símbolos do arquivo atual.

### 13.3 Toolchains

Pode aparecer como painel alternativo.

Mostra:

- compiladores detectados;
- CMake detectado;
- Ninja/Make;
- GDB/LLDB;
- OpenOCD/probe-rs;
- sysroots;
- SDKs;
- status OK/Warn/Error.

---

## 14. Editor Area

### 14.1 Função

O editor é o centro da experiência.

Componentes:

- tabs;
- breadcrumbs;
- gutter;
- line numbers;
- code editor;
- diagnostics;
- minimap opcional;
- inline hints opcionais;
- quick fixes.

### 14.2 Tab bar

Regras:

- altura 36px;
- aba ativa com linha âmbar superior ou inferior;
- arquivo modificado com ponto discreto;
- fechar aba no hover;
- ícone de linguagem pequeno;
- evitar abas muito altas.

### 14.3 Breadcrumbs

Exemplo:

```text
src > drivers > motor > motor_control.cpp > MotorControl > update
```

Regras:

- texto pequeno;
- separado por chevrons discretos;
- último item com destaque suave;
- clicável para navegação.

### 14.4 Gutter

Conteúdo:

- números de linha;
- breakpoints;
- warnings;
- erros;
- sugestões;
- folding;
- run marker para testes/funções.

Regras:

- largura mínima;
- ícones pequenos;
- erro/warning claros mas não agressivos;
- linha atual com fundo suave.

### 14.5 Syntax highlight

O tema de código deve priorizar legibilidade.

Sugestão:

- keywords: roxo/azul discreto;
- tipos/classes: azul claro;
- strings: verde suave;
- números: laranja controlado;
- funções: amarelo pálido ou azul;
- comentários: cinza esverdeado;
- erros: sublinhado vermelho discreto;
- warnings: sublinhado âmbar.

Evitar:

- cores neon saturadas;
- excesso de amarelo;
- contraste alto demais em todo o código.

---

## 15. Assistente — painel direito

### 15.1 Função

Assistente é o painel de assistência contextual.

Ele deve ajudar com:

- resumo do arquivo;
- explicação de símbolos;
- sugestões de correção;
- erros de compilação;
- linker errors;
- problemas de CMake;
- toolchain;
- documentação relacionada;
- sugestões de refatoração;
- perguntas do usuário.

### 15.2 Nome

Usar:

```text
Assistente
```

Evitar nomes genéricos como:

```text
AI Assistant
Copilot
Chatbot
```

### 15.3 Abas

```text
Context
Explain
Fix
Toolchain
Docs
```

Versão PT-BR, se desejado:

```text
Contexto
Explicar
Corrigir
Toolchain
Docs
```

### 15.4 Conteúdo padrão

Para arquivo aberto:

- card de resumo;
- símbolos principais;
- sugestões de melhoria;
- problemas detectados;
- input “Pergunte ao KV...”.

### 15.5 Regras de comportamento

- não abrir sozinho em toda ação;
- não sobrepor editor;
- sugestões devem ser aplicáveis com revisão;
- nenhuma alteração automática sem confirmação;
- cartões devem ser curtos;
- ações primárias em âmbar;
- erros críticos com vermelho discreto.

### 15.6 Estados

| Estado | Conteúdo |
|---|---|
| Sem arquivo | dicas de projeto e configuração |
| Arquivo aberto | resumo e símbolos |
| Build falhou | explicar erro e sugerir correção |
| Toolchain ausente | guia de instalação/configuração |
| Debug ativo | variáveis, stack, breakpoints, explicações |
| CMake erro | cache, presets, generator e toolchain file |

---

## 16. Bottom Tool Window

### 16.1 Abas padrão

```text
Terminal
Problems
Build
CMake
Debug
Serial
Telemetry
Simulation
Git
```

### 16.2 Terminal

Terminal embutido.

Regras:

- fonte monoespaçada;
- contraste moderado;
- prompt legível;
- saída de build com cores controladas;
- links de arquivo clicáveis.

### 16.3 Problems

Lista de erros e warnings.

Colunas:

- severidade;
- mensagem;
- arquivo;
- linha;
- origem: clangd, rust-analyzer, CMake, linker, linter.

### 16.4 Build

Mostra pipeline de build.

Exemplo:

```text
Configure ✓
Generate ✓
Build 62%
Link pending
```

### 16.5 CMake

Painel dedicado para:

- configure;
- generate;
- cache;
- presets;
- generator;
- toolchain file;
- build directory;
- logs filtráveis.

### 16.6 Debug

Painel para:

- threads;
- call stack;
- variables;
- watches;
- registers futuramente;
- memory view futuramente.

### 16.7 Serial

Painel para embedded:

- portas seriais;
- baud rate;
- logs;
- filtros;
- timestamps;
- enviar comandos.

### 16.8 Simulation

Inicialmente pode estar vazio ou experimental.

No futuro:

- logs de simulação;
- parâmetros;
- stepping;
- telemetria;
- plots;
- viewport OpenGL em layout específico.

---

## 17. Status Bar

### 17.1 Função

A Status Bar deve responder rapidamente:

```text
onde estou, o que está ativo, o que está errado, qual target estou usando.
```

### 17.2 Conteúdo recomendado

```text
KV | branch | errors | warnings | build state | CMake profile | compiler | target | line/col | encoding | language
```

Exemplo:

```text
KV  main*  0 errors  2 warnings  CMake: Debug  clang++ 17  x86_64-linux-gnu  Ln 18, Col 41  UTF-8  C++
```

### 17.3 Regras

- altura 28px;
- pequenos ícones;
- texto secundário;
- alertas discretos;
- build progress fino;
- branch visível;
- target sempre visível.

---

## 18. Layouts operacionais oficiais

A IDE deve ter alguns layouts pré-definidos, mas todos derivados do layout principal.

### 18.1 Default Coding

Uso:

- escrever código;
- navegar projeto;
- Assistente disponível.

Painéis:

- Project aberto;
- Assistente aberto;
- Bottom oculto ou terminal pequeno.

### 18.2 Focus Editor

Uso:

- codar por longos períodos;
- leitura profunda;
- refatoração.

Painéis:

- Tool Rail visível;
- Left Tool Window colapsado;
- Assistente colapsado;
- Bottom oculto.

Atalho sugerido:

```text
Ctrl+Shift+F12 ou Distraction Free Mode equivalente
```

### 18.3 Build & CMake

Uso:

- configurar projeto;
- corrigir CMake;
- entender toolchain.

Painéis:

- Project ou CMake à esquerda;
- Editor no centro;
- Assistente em Toolchain/Fix;
- Bottom em CMake/Build.

### 18.4 Debug Active

Uso:

- debug local/remoto.

Painéis:

- Debug tool window à esquerda ou inferior;
- Editor no centro;
- Assistente opcional;
- Bottom com Debug, Variables, Stack.

### 18.5 Embedded Target

Uso:

- flash;
- serial;
- debug remoto;
- target board.

Painéis:

- Targets à esquerda;
- Editor no centro;
- Assistente em Toolchain;
- Bottom com Serial/Debug/Build.

### 18.6 Simulation Workbench

Uso futuro:

- OpenGL;
- simulação física;
- plots;
- visualização.

Importante:

```text
Não misturar Simulation Workbench com layout padrão no primeiro MVP.
```

Este layout é uma evolução futura.

Painéis:

- Editor e viewport lado a lado;
- Simulation/Telemetry inferior;
- Assistente opcional;
- controles de simulação dedicados.

---

## 19. Fluxos principais

### 19.1 Primeiro uso

Tela de boas-vindas:

```text
Kinein Vectis
[Open Project]
[New CMake Project]
[New Rust Project]
[Import Existing Project]
[Configure Toolchains]
```

Deve mostrar status:

- CMake detectado?
- Ninja detectado?
- GCC/Clang detectado?
- Rust/Cargo detectado?
- GDB/LLDB detectado?

### 19.2 Abrir projeto existente

Fluxo:

```text
Open Folder
Detect project kind
Detect build system
Detect toolchains
Suggest profile
Open layout default
```

### 19.3 Novo projeto CMake

Assistente visual:

```text
Project name
Language: C / C++ / C + C++
Standard: C17/C23, C++17/C++20/C++23
Executable / Library / Embedded Firmware
Compiler: auto / GCC / Clang / Cross
Generator: Ninja / Make
Create CMakePresets.json: yes
```

### 19.4 Configurar toolchain

A IDE deve explicar visualmente:

```text
Compiler
Debugger
Build tool
CMake
Sysroot
Target triple
Environment PATH
```

O usuário deve conseguir ver exatamente o que está faltando.

### 19.5 Build falhou

Fluxo:

```text
Build fails
Problems recebe erro
Linha do editor marca erro
Build panel mostra etapa que falhou
Assistente oferece explicação
Usuário aplica correção ou abre docs
```

---

## 20. Regras para não poluir a interface

Evitar:

- painéis abertos demais por padrão;
- brilho forte em toda borda;
- animações constantes;
- notificações grandes;
- tooltips longos demais;
- cards enormes no Assistente;
- ícones coloridos demais;
- fundo com texturas dentro da IDE real;
- decoração matemática no layout real.

Importante:

```text
Matemática, circuitos e malhas 3D pertencem ao app icon, splash, marketing e alguns painéis especializados.
A IDE real deve ser limpa e funcional.
```

---

## 21. Diferença entre tema visual e marca

A marca usa:

- KV;
- âmbar;
- grafite;
- vetores;
- circuitos;
- malha 3D;
- física/matemática.

A interface diária usa:

- grafite limpo;
- divisores sutis;
- ícones simples;
- âmbar em ações/estado;
- pouco ruído;
- muita legibilidade.

Regra:

```text
Marca pode ser expressiva.
Interface deve ser calma.
```

---

## 22. Componentes essenciais

### 22.1 Button

Variantes:

- Primary: âmbar;
- Secondary: grafite elevado;
- Ghost: transparente;
- Danger: vermelho discreto;
- Success: verde discreto.

Estados:

- default;
- hover;
- pressed;
- focused;
- disabled;
- loading.

### 22.2 Dropdown

Usado para:

- target;
- build profile;
- compiler;
- generator;
- serial port;
- run configuration.

### 22.3 Tabs

Tipos:

- editor tabs;
- panel tabs;
- Assistente tabs;
- bottom tabs.

Regras:

- aba ativa clara;
- hover discreto;
- fechar no hover;
- modificado com ponto;
- erro com pequeno marcador.

### 22.4 Cards

Usados apenas em:

- Assistente;
- tela de boas-vindas;
- toolchain diagnostics;
- sugestões aplicáveis.

Evitar cards no editor principal.

### 22.5 Toast/Notification

Local:

- canto inferior direito;
- ou integrado à status bar.

Tipos:

- info;
- success;
- warning;
- error;
- action required.

Tempo:

- sucesso: curto;
- erro: persistente até ação;
- build: status bar/painel, não toast constante.

---

## 23. Estados de projeto

### 23.1 Projeto sem toolchain

Visual:

- Target selector com warning;
- Assistente abre aba Toolchain;
- card “Configure Toolchain”;
- Problems mostra “compiler not configured”;
- Run/Debug desabilitados.

### 23.2 Projeto configurado

Visual:

- Target OK;
- profile OK;
- Build habilitado;
- Run habilitado se houver artefato executável;
- Debug habilitado se debugger disponível.

### 23.3 Build em andamento

Visual:

- barra fina de progresso na toolbar ou bottom;
- Build icon ativo;
- status bar mostra percentual/etapa;
- output no painel Build.

### 23.4 Debug ativo

Visual:

- Debug icon ativo;
- botão Stop visível;
- gutter com linha atual;
- Bottom em Debug;
- status bar mostra processo/sessão.

### 23.5 Target embarcado conectado

Visual:

- Target selector com indicador verde/âmbar;
- Flash habilitado;
- Serial habilitado;
- Debug remoto habilitado se probe disponível.

---

## 24. Menus principais

### 24.1 File

- New Project
- Open Project
- Recent Projects
- Save
- Save All
- Settings
- Exit

### 24.2 Edit

- Undo
- Redo
- Cut/Copy/Paste
- Find
- Replace

### 24.3 View

- Appearance
- Tool Windows
- Focus Editor
- Split Editor
- Toggle Assistente
- Toggle Terminal

### 24.4 Navigate

- Go to File
- Go to Symbol
- Go to Definition
- Go to Declaration
- Find Usages

### 24.5 Code

- Format
- Refactor
- Rename
- Generate
- Inspect

### 24.6 Build

- Configure
- Build
- Rebuild
- Clean
- CMake Settings
- Toolchains

### 24.7 Run

- Run
- Debug
- Stop
- Run Configurations
- Attach Debugger

### 24.8 Tools

- CMake
- Toolchains
- Targets
- Serial Monitor
- QEMU
- OpenOCD
- Extensions

### 24.9 Help

- Documentation
- Keyboard Shortcuts
- About Kinein Vectis

---

## 25. Command Palette

A Kinein deve ter uma Command Palette para reduzir dependência de menus.

Atalho sugerido:

```text
Ctrl+Shift+P
```

Comandos:

- Open Project
- Configure CMake
- Select Target
- Select Toolchain
- Build Project
- Run Project
- Debug Project
- Flash Target
- Open Serial Monitor
- Toggle Assistente
- Open Settings

A Command Palette deve mostrar atalhos e contexto.

---

## 26. Configurações visuais

### 26.1 Appearance

Opções iniciais:

- Theme: Kinein Dark
- Accent: Amber
- Font Size
- Editor Font
- UI Density: Comfortable / Compact
- Show Tool Labels
- Show Minimap
- Restore Layout on Startup

### 26.2 UI Density

Comfortable:

- mais espaçamento;
- melhor para longas sessões.

Compact:

- mais informação;
- melhor para telas menores.

---

## 27. Acessibilidade e conforto

Regras:

- foco visível por teclado;
- contraste suficiente;
- sem dependência exclusiva de cor;
- ícones com tooltip;
- target/build/debug com texto, não só ícone;
- animações reduzíveis;
- tema sem brilho excessivo;
- suporte a escala de fonte.

Cuidado com o âmbar:

- não usar como texto longo;
- evitar glow forte;
- evitar fundo amarelo grande;
- usar como sinal, não como tinta dominante.

---

## 28. Layout de divulgação/marketing

O layout de marketing pode mostrar a IDE com callouts.

Estrutura recomendada:

```text
Logo KV + Kinein Vectis
Slogan curto
Mockup da IDE central
Callouts laterais
Cards inferiores
App icon
```

Callouts possíveis:

- C/C++ e Rust first;
- CMake sem fricção;
- Linux-first;
- toolchains visuais;
- Assistente;
- embedded targets;
- debug e serial;
- simulação futura.

Mas esses elementos não devem aparecer dentro da UI real.

---

## 29. Implementação futura em Qt/QML

### 29.1 Estrutura sugerida

```text
ui/
  qml/
    App.qml
    theme/
      Tokens.qml
      Colors.qml
      Typography.qml
      Metrics.qml
    components/
      KvButton.qml
      KvDropdown.qml
      KvTabBar.qml
      KvPanel.qml
      KvToolRail.qml
      KvStatusBar.qml
      KvToolbar.qml
      KvToast.qml
    layout/
      MainWindow.qml
      EditorArea.qml
      LeftToolWindow.qml
      RightContextPanel.qml
      BottomToolWindow.qml
    icons/
      svg/
      generated/
```

### 29.2 Tokens em QML

Centralizar:

- cores;
- tamanhos;
- fontes;
- espaçamento;
- raios;
- opacidades;
- duração de animações.

### 29.3 Persistência de layout

Salvar:

- painéis abertos;
- largura dos painéis;
- altura do bottom;
- abas abertas;
- layout operacional ativo;
- posição de split;
- tema e densidade.

Arquivo sugerido:

```text
.kinein/layout.json
```

Ou configuração global:

```text
~/.config/kinein/layout.json
```

---

## 30. Instruções para IA CLI

Ao usar este documento com IA CLI, priorizar:

1. Implementar layout real, não marketing.
2. Criar componentes reutilizáveis antes de telas grandes.
3. Implementar tokens de tema primeiro.
4. Criar shell da janela principal.
5. Criar Tool Rail.
6. Criar Left Tool Window com Project placeholder.
7. Criar Editor Area placeholder.
8. Criar Assistente placeholder.
9. Criar Bottom Tool Window.
10. Criar Status Bar.
11. Só depois integrar dados reais.

Ordem de implementação sugerida:

```text
MVP UI Layout 0.1: tokens + janela + regiões vazias
MVP UI Layout 0.2: toolbar + rail + status bar
MVP UI Layout 0.3: project tree fake + editor fake
MVP UI Layout 0.4: bottom tool window fake
MVP UI Layout 0.5: Assistente fake
MVP UI Layout 0.6: conectar ao core real
```

### 30.1 Prompt para IA CLI implementar layout shell

```text
Implemente o shell visual principal da IDE Kinein Vectis em Qt/QML seguindo o documento KINEIN_VECTIS_LAYOUT_SYSTEM.md. Não implemente lógica real ainda. Crie componentes reutilizáveis para App Bar, Main Toolbar, Tool Rail, Left Tool Window, Editor Area, Assistente, Bottom Tool Window e Status Bar. Use tokens de tema centralizados. O layout deve ser real de IDE, não marketing: sem callouts, sem slogans, sem textos promocionais ao redor. Priorize visual limpo, escuro, JetBrains-like na sensação, Kinein-like na identidade.
```

### 30.2 Prompt para IA CLI criar tokens

```text
Crie os tokens visuais da Kinein Vectis em QML usando as cores, espaçamentos, fontes, raios e métricas descritos em KINEIN_VECTIS_LAYOUT_SYSTEM.md. Todos os componentes devem importar esses tokens, sem cores hardcoded espalhadas.
```

### 30.3 Prompt para IA CLI criar estados de layout

```text
Implemente estados de layout para Default Coding, Focus Editor, Build & CMake, Debug Active e Embedded Target. Cada estado deve apenas reorganizar/mostrar/ocultar painéis, mantendo a mesma arquitetura visual principal.
```

---

## 31. Instruções para geração de imagem de layout

Para gerar imagem de referência, usar prompt:

```text
Criar mockup de alta resolução de uma IDE profissional chamada Kinein Vectis, nome curto Kinein, sigla KV. A imagem deve mostrar apenas a interface real da IDE, sem callouts de marketing, sem textos promocionais externos e sem elementos fora da janela. Tema escuro grafite confortável, estilo visual polido e profissional, inspirado na sensação de IDEs modernas como JetBrains, mas com identidade própria. Layout: app bar superior com KV | Kinein, menu File/Edit/View/Navigate/Code/Build/Run/Tools/Help; toolbar com Target selector, CMake profile, Configure, Build, Run, Debug, Flash, Simulate; tool rail esquerdo com Project, Search, Git, Build, Debug, Targets, Simulate, Tools; painel esquerdo com Project e Structure; editor central com tabs, breadcrumbs, código C++ legível, gutter e diagnostics; painel direito Assistente com abas Context, Explain, Fix, Toolchain, Docs; painel inferior com Terminal, Problems, Build, CMake, Debug, Serial, Simulation, Git; status bar inferior com branch, errors, warnings, compiler, target, line/column, encoding. Usar acento âmbar industrial com moderação, azul/roxo apenas como acentos técnicos. Interface limpa, densa mas confortável, sem excesso de brilho, sem decoração matemática no layout real.
```

---

## 32. Checklist de qualidade

Antes de aceitar um layout, verificar:

- [ ] A interface parece utilizável, não apenas bonita?
- [ ] O editor é a área dominante?
- [ ] Target e build profile estão claros?
- [ ] CMake/Build/Debug/Serial têm locais óbvios?
- [ ] Assistente ajuda sem dominar?
- [ ] O âmbar está usado com moderação?
- [ ] O layout não tem callouts de marketing?
- [ ] O visual é confortável para 6+ horas de uso?
- [ ] Os painéis são previsíveis?
- [ ] A status bar responde “onde estou e o que está ativo?”
- [ ] Existe caminho claro para toolchain ausente?
- [ ] O layout funciona em 1920×1080?
- [ ] Existe modo Focus Editor?
- [ ] Existe separação futura para Simulation Workbench?

---

## 33. Decisão final desta parte

A Kinein Vectis deve usar um layout real de IDE profissional, limpo e denso de forma controlada.

A interface principal deve seguir esta fórmula:

```text
Project/Tools à esquerda
Editor no centro
Assistente à direita
Terminal/Build/Debug embaixo
Target/CMake/Run no topo
Estado do sistema no rodapé
```

O layout de divulgação deve ficar separado e pode usar callouts, slogans e cards.

A marca pode ser expressiva.  
A interface deve ser calma.  
A IDE deve parecer familiar no primeiro minuto e poderosa depois de uma semana.
