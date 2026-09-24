# Kinein Vectis — Sistema Visual da IDE

> **Revisado em 2026-09-22.** A arquitetura consolidada está em
> `arquitetura-de-frontend-0.3-em-diante.md` e tem precedência. A Vectis não
> terá Assistente/Chat de IA embutido nem telemetria de produto/usuário. A área
> direita é uma Tool Window genérica; seu primeiro uso é Símbolos/Structure.

**Parte 2 — Layout principal da IDE**  
**Escopo:** arquitetura visual, organização de painéis, estados de interface, padrões de layout real e separação explícita do layout de divulgação/marketing.  
**Produto:** Kinein Vectis  
**Nome de uso diário:** Kinein  
**Sigla visual:** KV  
**Foco técnico:** C, C++, Rust, CMake, toolchains, sistemas embarcados, Linux embarcado, software embarcado, debug e flash.

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
- execução em Linux embarcado.

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

A janela principal da Kinein é dividida em regiões funcionais. Tool windows
laterais e inferior são opcionais; o editor permanece dominante.

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ 1. Title/App Bar                                                             │
├──────────────────────────────────────────────────────────────────────────────┤
│ 2. Main Toolbar                                                              │
├────┬─────────────────────┬──────────────────────────────────┬───────────────┤
│ 3  │ 4. Left Tool Window │ 5. Editor Area                   │ 6. Right Tool │
│Rail│                     │                                  │ Window opc.  │
├────┴─────────────────────┴──────────────────────────────────┴───────────────┤
│ 7. Bottom Tool Window                                                        │
├──────────────────────────────────────────────────────────────────────────────┤
│ 8. Status Bar                                                               │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 4.1 Regiões

| Região | Nome | Função |
|---|---|---|
| 1 | Title/App Bar | Identidade, menus, ações globais, janela |
| 2 | Main Toolbar | Projeto, Git e contexto de execução; ações essenciais |
| 3 | Tool Rail | Ícones verticais de janelas/ferramentas |
| 4 | Left Tool Window | Navegação e contexto: Project, Git, Embedded, Remote |
| 5 | Editor Area | Código, tabs, breadcrumbs, gutter, diagnósticos |
| 6 | Right Tool Window | Inspeção opcional: Símbolos/Structure, ambiente, registradores |
| 7 | Bottom Tool Window | Terminal, Problems, Jobs, Build, Debug, Testes, Search |
| 8 | Status Bar | Estado do projeto, branch, target, warnings, encoding, posição |

---

## 5. Princípios de UX

### 5.1 O editor é a área sagrada

A área central deve ser sempre dominante.

Regras:

- nenhum painel deve roubar atenção do editor sem ação explícita do usuário;
- tool windows não devem abrir sozinhas sem causa e gesto claros;
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
- inspeção contextual opcional à direita;
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
2px  — linhas finas, divisores internos
4px  — micro espaçamento
8px  — espaçamento padrão de componentes pequenos
12px — espaçamento entre blocos
16px — padding de painel
24px — separação de regiões
32px — seções grandes
```

### 6.3 Alturas recomendadas

| Elemento | Altura |
|---|---:|
| Title/App Bar | 40px |
| Main Toolbar | 44px |
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
| Right Tool Window | 320px | 240px | 480px |
| Rail direito opcional | 44px | 40px | 52px |
| Bottom Tool Window | 260px altura | 160px | 480px |

### 6.5 Bordas e cabeçalho mais naturais — compromisso da 0.3.x

**Pedido do autor em 2026-09-23; planejado, não implementado nesta revisão.**
Refinar o arredondamento da IDE, especialmente a região superior, tomando o
IntelliJ Community como referência de conforto. A expressão “rodapé superior”
foi interpretada provisoriamente como cabeçalho/cantos superiores; confirmar
essa região com o autor antes de fechar a geometria visual. Não substituir
essa intenção por um ajuste da status bar inferior.

Referência de organização:
[IntelliJ — New UI / Window header](https://www.jetbrains.com/help/idea/new-ui.html).
A documentação sustenta o objetivo de reduzir complexidade e integrar o
cabeçalho; não especifica um raio universal a copiar em Qt/Linux. Comparar
capturas no mesmo estado de janela/escala antes de escolher valores.

- Integrar contorno externo, header, fundos e separadores, sem recortes
  contrastantes ou várias bordas arredondadas sobrepostas na junção.
- Reutilizar `Theme.qml`, o header e o chrome existentes. Não espalhar raios
  novos por componente nem criar outra implementação de janela.
- Distinguir janela restaurada de maximizada, fullscreen e encaixada: cantos
  decorativos não devem abrir frestas na borda da tela ou cortar conteúdo.
- Preservar áreas clicáveis de minimizar/maximizar/fechar, arrasto da janela,
  duplo clique no header, resize nas bordas e foco de teclado.
- Verificar clipping, antialiasing, sombra e custo de composição em X11 e
  Wayland suportados, com escalas 100%, 125%, 150% e 200%; não exigir máscara
  ou transparência antes de medir sua necessidade e impacto.

**Aceite:** comparação visual antes/depois em 1024×700 e 1280×800, inclusive
restaurar/maximizar e mudar de escala, sem regressão de hit targets, resize,
primeiro frame ou legibilidade. A aprovação visual do autor continua necessária.
Proposta de encaixe: **0.3.4**, antecipável dentro da 0.3.x; não depende do
Grafana e não se declara entregue por apenas trocar um token de raio.

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
| Tool Window body | 13px |
| Título de painel | 13px semibold |

---

## 9. Layout real — estado padrão

Este é o layout inicial após abrir um projeto.

### 9.1 Estrutura padrão

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ KV | Kinein        File Edit View Navigate Code Build Run Tools Help         │
├──────────────────────────────────────────────────────────────────────────────┤
│ [Target: x86_64-linux ▼] [CMake: Debug ▼] [Configure] [Build] [Run] [Debug] │
├────┬──────────────────────┬─────────────────────────────────┬──────────────┤
│Rail│ Project              │ Editor                          │ Símbolos     │
│    │ Git                  │ Tabs + Breadcrumbs              │ Structure    │
│    │                      │ Code                            │ Busca        │
│    │                      │ Gutter + Diagnostics            │ contextual   │
├────┴──────────────────────┴─────────────────────────────────┴──────────────┤
│ Terminal | Problems | Build | CMake | Debug | Serial | Git                  │
├──────────────────────────────────────────────────────────────────────────────┤
│ KV main* | 0 errors | 2 warnings | CMake Debug | clang++ | x86_64 | UTF-8   │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 9.2 Painéis abertos por padrão

Para o primeiro uso:

- Project aberto à esquerda;
- Editor no centro;
- Right Tool Window recolhida por padrão e aberta por gesto/contexto;
- Terminal/Build inferior aberto apenas se houver processo ativo ou se o usuário abrir.

Sugestão de default:

```text
Primeiro projeto aberto: Project + Editor; Símbolos disponível à direita.
Sessões seguintes: restaurar layout salvo pelo usuário.
```

---

## 10. Title/App Bar

### 10.1 Função

A Title/App Bar contém:

- marca KV;
- nome curto Kinein;
- menus globais;
- ações de janela;
- indicadores globais discretos.

### 10.2 Conteúdo recomendado

```text
[KV] Kinein     File  Edit  View  Navigate  Code  Build  Run  Tools  Help
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

### 11.1 Ordem recomendada

```text
Target Selector
Build System / Profile
Configure
Build
Run
Debug
Flash
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
- largura natural limitada e nomes longos truncados com reticências;
- quando as abas excederem a área do editor, comprimi-las uniformemente até
  uma largura mínima utilizável;
- depois da largura mínima, usar rolagem horizontal e manter a aba ativa
  sempre visível;
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

## 15. Right Tool Window — inspeção contextual

### 15.1 Função

A área direita mostra informação auxiliar sobre o contexto ativo sem competir
com o editor. Não é painel de chat nem superfície permanente obrigatória.

Primeiro uso:

- Símbolos/Structure do arquivo;
- busca de símbolos do projeto;
- navegação para a declaração escolhida.

Usos posteriores, quando houver fluxo medido:

- ambiente/toolchain efetivos;
- registradores e periféricos durante debug;
- inspeção de banco, containers ou observabilidade.

### 15.2 Regras de comportamento

- nasce recolhida;
- abre por comando, atalho ou gesto explícito;
- preserva foco e largura quando o layout for restaurado;
- reutiliza models/controllers existentes;
- não interpreta logs nem duplica fatos do core;
- erro mostra causa e próximo passo determinístico quando conhecidos;
- documentação é link/contexto, não resposta gerada por IA.

### 15.3 Estados

| Estado | Conteúdo |
|---|---|
| Sem workspace | alça indisponível ou empty state curto |
| Workspace sem arquivo | busca de símbolos do projeto |
| Arquivo aberto | Structure e busca de símbolos |
| Sem resultados | termo e escopo pesquisados, limpar busca |
| Backend indisponível | estado local disponível e limitação explícita |

---

## 16. Bottom Tool Window

### 16.1 Abas padrão

```text
Terminal
Problems
Jobs
Build
CMake
Debug
Serial
Search
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
- Símbolos/Structure disponível à direita.

Painéis:

- Project aberto;
- Right Tool Window recolhida por padrão;
- Bottom oculto ou terminal pequeno.

### 18.2 Focus Editor

Uso:

- codar por longos períodos;
- leitura profunda;
- refatoração.

Painéis:

- Tool Rail visível;
- Left Tool Window colapsado;
- Right Tool Window colapsada;
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
- Toolchain/Environment aberto quando necessário;
- Bottom em CMake/Build.

### 18.4 Debug Active

Uso:

- debug local/remoto.

Painéis:

- Debug tool window à esquerda ou inferior;
- Editor no centro;
- inspeção de Debug opcional à direita;
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
- Toolchain/Environment ou registradores à direita;
- Bottom com Serial/Debug/Build.


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

Wizard determinístico de projeto:

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
Project Health/Problems mostra causa e próximo passo quando o core conhece
Usuário aplica ação explícita ou abre a documentação
```

---

## 20. Regras para não poluir a interface

Evitar:

- painéis abertos demais por padrão;
- brilho forte em toda borda;
- animações constantes;
- notificações grandes;
- tooltips longos demais;
- cards enormes em tool windows;
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
- tool window tabs;
- bottom tabs.

Regras:

- aba ativa clara;
- hover discreto;
- fechar no hover;
- modificado com ponto;
- erro com pequeno marcador.

### 22.4 Cards

Usados apenas em:

- tela de boas-vindas;
- toolchain diagnostics;
- Project Health e ações de configuração com preview.

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
- Toolchain/Environment abre no ponto relevante;
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
- Toggle Right Tool Window
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
- Toggle Symbols/Structure
- Focus Remote
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
- embedded targets;
- Remote SSH;
- debug e serial.

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

## 30. Instruções de implementação do shell

Ao implementar este documento, com ou sem ferramenta de apoio, priorizar:

1. Implementar layout real, não marketing.
2. Criar componentes reutilizáveis antes de telas grandes.
3. Implementar tokens de tema primeiro.
4. Criar shell da janela principal.
5. Criar Tool Rail.
6. Criar Left Tool Window com Project placeholder.
7. Criar Editor Area placeholder.
8. Criar Right Tool Window opcional com Símbolos/Structure.
9. Criar Bottom Tool Window.
10. Criar Status Bar.
11. Só depois integrar dados reais.

Ordem de implementação sugerida:

```text
MVP UI Layout 0.1: tokens + janela + regiões vazias
MVP UI Layout 0.2: toolbar + rail + status bar
MVP UI Layout 0.3: project tree fake + editor fake
MVP UI Layout 0.4: bottom tool window fake
MVP UI Layout 0.5: Right Tool Window com modelo de símbolos existente
MVP UI Layout 0.6: conectar ao core real
```

### 30.1 Brief de implementação do layout shell

```text
Implementar o shell visual principal da IDE Kinein Vectis em Qt/QML seguindo sistema-de-layout.md e arquitetura-de-frontend-0.3-em-diante.md. Reutilizar os componentes atuais para App Bar, Main Toolbar, Tool Rail, Left/Right/Bottom Tool Windows, Editor Area e Status Bar. Usar tokens de tema centralizados. O layout deve ser real de IDE, não marketing: sem callouts, slogans ou textos promocionais na área de trabalho. Priorizar visual limpo, escuro, denso e confortável, com referência de comportamento profissional e identidade própria.
```

### 30.2 Prompt para IA CLI criar tokens

```text
Crie os tokens visuais da Kinein Vectis em QML usando as cores, espaçamentos, fontes, raios e métricas descritos em sistema-de-layout.md. Todos os componentes devem importar esses tokens, sem cores hardcoded espalhadas.
```

### 30.3 Prompt para IA CLI criar estados de layout

```text
Implemente estados de layout para Default Coding, Focus Editor, Build & CMake, Debug Active e Embedded Target. Cada estado deve apenas reorganizar/mostrar/ocultar painéis, mantendo a mesma arquitetura visual principal.
```

---

## 31. Instruções para geração de imagem de layout

Para gerar imagem de referência, usar prompt:

```text
Criar mockup de alta resolução de uma IDE profissional chamada Kinein Vectis, nome curto Kinein, sigla KV. Mostrar apenas a interface real da IDE, sem callouts de marketing ou elementos fora da janela. Tema escuro grafite confortável e identidade própria. Layout: header enxuto com Projeto, Git e Execução; tool rail esquerdo; Project/Git à esquerda; editor central com tabs, breadcrumbs, código, gutter e diagnostics; Tool Window direita opcional com Símbolos/Structure; painel inferior com Terminal, Problems, Jobs, Build, Debug e Search; status bar com workspace, job, toolchain, contexto remoto quando houver, line/column e LSP. Usar acento âmbar com moderação. Interface limpa, densa e confortável, sem chat, Assistente de IA ou Telemetry.
```

---

## 32. Checklist de qualidade

Antes de aceitar um layout, verificar:

- [ ] A interface parece utilizável, não apenas bonita?
- [ ] O editor é a área dominante?
- [ ] Target e build profile estão claros?
- [ ] CMake/Build/Debug/Serial têm locais óbvios?
- [ ] A Tool Window direita ajuda sem disputar espaço com o editor?
- [ ] O âmbar está usado com moderação?
- [ ] O layout não tem callouts de marketing?
- [ ] O visual é confortável para 6+ horas de uso?
- [ ] Os painéis são previsíveis?
- [ ] A status bar responde “onde estou e o que está ativo?”
- [ ] Existe caminho claro para toolchain ausente?
- [ ] O layout funciona em 1920×1080?
- [ ] Existe modo Focus Editor?

---

## 33. Decisão final desta parte

A Kinein Vectis deve usar um layout real de IDE profissional, limpo e denso de forma controlada.

A interface principal deve seguir esta fórmula:

```text
Project/Tools à esquerda
Editor no centro
Inspeção opcional à direita
Terminal/Build/Debug embaixo
Projeto/Git/Execução no topo
Estado do sistema no rodapé
```

O layout de divulgação deve ficar separado e pode usar callouts, slogans e cards.

A marca pode ser expressiva.  
A interface deve ser calma.  
A IDE deve parecer familiar no primeiro minuto e poderosa depois de uma semana.
