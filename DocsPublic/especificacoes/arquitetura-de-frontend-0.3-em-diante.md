# Arquitetura de frontend — 0.3 em diante

> **Classe: ALVO / PLANO VINCULANTE.** Este documento consolida o conteúdo
> aproveitável de `DocsPrivate/arquiKinein/` depois de confrontá-lo com o código,
> os gates e os contratos públicos em 2026-09-22.
>
> Ele não afirma o que já existe. Para estado, o código e o
> [`roadmap 40`](../roadmaps/40-estado-e-continuidade.md) vencem. Para regras de
> engenharia, [`ARCHITECTURE.md`](../arquitetura/ARCHITECTURE.md) vence.
>
> Em conflito com as especificações antigas de frontend, este documento tem
> precedência. Nenhuma ideia ambígua foi descartada: ela aparece em
> **Decisões ainda necessárias** ou **Adiar até existir gatilho real**.

## 1. Decisões de produto que não se reabrem

- O editor continua sendo a região central e dominante.
- A UI apresenta e recebe intenção; o Rust Core decide fatos e orquestra
  ferramentas; operações longas continuam como Jobs.
- A Vectis continua Qt/QML nativa, Linux-first, local-first e sem instalação
  silenciosa.
- A Vectis não terá chat, painel de Assistente de IA nem ações dependentes de
  modelo de IA.
- A Vectis não coleta telemetria de produto, uso ou usuário.
- Logs locais, métricas locais de desempenho e dados produzidos pelo alvo não
  são telemetria do produto. Devem ser nomeados pelo domínio: **Logs da IDE**,
  **Desempenho**, **Saída do alvo**, **Observabilidade** ou **Diagnóstico**.
- O header enxuto da 0.2.x — Projeto, Git e Execução — é preservado. Ações
  secundárias ficam em menus, paleta, tool windows ou contexto.
- Referências JetBrains, Code OSS, Zed, Lapce e NetBeans fornecem invariantes de
  UX e estratégias de teste; não fornecem arquitetura para copiar.

## 2. O que deve ser preservado da implementação atual

Não há autorização para reescrever uma IDE que já funciona. Continuam sendo a
base:

- `Theme.qml` e os componentes `Kv*` existentes;
- `Main.qml` como composition root;
- controllers QML por responsabilidade;
- uma única fachada `CoreClient`, internamente separada por domínio;
- roteadores QML de request/event;
- o editor, seu renderer e os controllers de documento, texto, linguagem,
  realce, persistência, completion e busca;
- `CommandDispatcher.qml` e o catálogo `command.list` já existentes;
- Jobs, Run, Debug, Terminal, Problems, Project Health e Settings atuais;
- shell, rail compacto/expandido, painel inferior, status bar e hosts atuais;
- capacidades atuais de Git, embarcados, banco, containers, Grafana, toolchains
  e Remote SSH.

Generalizar significa retirar conhecimento nominal do shell quando isso reduz
acoplamento medido. Não significa trocar componentes, estado ou contratos por
uma arquitetura paralela.

## 3. Modelo visual alvo

```text
AppShell
├── Header / Main Toolbar
├── Workspace
│   ├── Tool Window Bar esquerda
│   ├── Tool Window esquerda, opcional
│   ├── Editor Workspace
│   ├── Tool Window direita, opcional
│   └── Bottom Tool Window, opcional
├── Status Bar
└── Overlay Layer
```

As áreas têm semântica de trabalho:

| Área | Responsabilidade |
| --- | --- |
| Esquerda | navegação e troca de contexto: Projeto, Git, Embarcados, Remote |
| Centro | edição e visualizadores que se comportam como documentos |
| Direita | inspeção do que está ativo: Símbolos, estrutura, ambiente, registradores |
| Inferior | atividade temporal: Terminal, Problems, Jobs, Run, Debug, Testes, Search |
| Status | contexto compacto e estado atual; nunca painel de configuração |

Uma feature escolhe a menor superfície que atende o fluxo: command, popup,
dialog, tool window, editor tab, status item, job ou notification. Painel novo
de tela inteira exige justificativa de fluxo, não apenas espaço disponível.

## 4. Sistema mínimo de Tool Windows

O problema real é nominal: `SideRail.qml`, hosts e shell ainda conhecem vários
domínios pelo nome. O primeiro contrato deve ser deliberadamente pequeno:

```text
ToolWindowEntry
├── id estável
├── title
├── icon
├── area: left | right | bottom
├── order
├── available
├── active
└── component/factory interna
```

Primeira sequência:

1. representar Projeto e Git sem alterar aparência;
2. representar Símbolos/Structure na direita;
3. representar Remote SSH no shell diário;
4. migrar os demais domínios somente quando houver paridade;
5. remover caminhos nominais apenas depois dos testes e da paridade visual.

Critério: adicionar uma tool window interna não deve exigir branching nominal em
`SideRail.qml`, `ShellWorkspaceHost.qml` e `Main.qml` ao mesmo tempo.

Não entram nesta primeira geração: API pública de plugins, detach, floating,
dock graph arbitrário, multi-monitor, nested docking, auto-hide complexo ou
drag-and-drop de áreas.

## 5. Commands: convergir, não recriar

A Vectis já possui `command.list`, IDs estáveis, paleta, atalhos e
`CommandDispatcher.qml`. Portanto:

- não criar segundo catálogo de comandos;
- evoluir o dispatcher atual para ser o caminho comum de paleta, atalhos,
  menus, Project Health e ações de tool windows;
- comando desconhecido deve ser recusado e observável;
- a habilitação pode começar com condições simples já medidas;
- uma linguagem genérica de context keys só nasce quando as condições atuais
  estiverem duplicadas em consumidores reais.

O dispatcher encaminha intenção. Regra de negócio permanece no core ou no dono
do estado já existente.

### 5.1 Extender antes de criar

O princípio aberto/fechado é critério de aceite da série 0.3. Antes de criar
classe, controller, popup, catálogo, estado ou rota, a implementação deve
localizar o dono e os pontos de extensão já existentes. A ordem é:

1. reutilizar o comportamento existente sem copiá-lo;
2. estender seu contrato quando a responsabilidade continua sendo a mesma;
3. compor um adaptador de domínio quando só metadados e ações são específicos;
4. criar outra abstração apenas quando houver responsabilidade distinta,
   consumidor real e teste que não cabem honestamente no dono atual.

Um adaptador novo não é um subsistema novo: ele pode declarar as ações do seu
domínio, mas encaminha menu, clipboard, comando, estado e execução aos donos
comuns. Duas fontes de verdade para a mesma ação, dois dispatchers, dois
buffers ou dois ciclos de sessão reprovam a revisão mesmo que os gates
mecânicos permaneçam verdes.

Aplicação na primeira fatia: `TerminalContextMenu` só configura ações e compõe
`AppMenuPopup`; entrada e seleção continuam em `TerminalInputController` e
`TerminalSelectionController`; sessão e scrollback continuam no core e em
`alacritty_terminal`; a ponte amplia `CoreClient`/router existentes. Não há
segundo terminal, clipboard, popup renderer, dispatcher ou buffer QML.

## 6. Editor Workspace

O renderer atual é preservado. A evolução estrutural é incremental:

1. `ListView` horizontal com overflow — **já entregue em 2026-09-22**;
2. identidade estável de documento/aba, deixando índice como detalhe da view;
3. preview tab;
4. pin/unpin;
5. Símbolos/Structure fora do `EditorPane`, na primeira tool window direita;
6. um primeiro split com dois grupos;
7. persistência versionada do layout de grupos;
8. splits adicionais apenas após uso real do primeiro.

Não entram antes de consumidor concreto: `EditorProviderRegistry`, tipo
universal de documento, decoration registry genérico, árvore arbitrária de
splits, buffer v2 ou framework universal de markers.

## 7. Design System e componentes

O Design System é uma API de produto. Toda área reutiliza tokens e componentes
para não inventar hover, foco, erro, loading ou densidade próprios.

Cada componente interativo deve cobrir, quando aplicável:

```text
default · hover · pressed · focused · selected · disabled
loading · empty · warning · error · success
```

Regras consolidadas:

- densidade compacta por padrão, com área clicável suficiente;
- foco de teclado sempre visível;
- tooltip explica ação curta; documentação ou texto inline explica conceito;
- cor nunca é o único portador de significado;
- motion curta, funcional e respeitando reduced motion;
- o âmbar orienta ação/foco/seleção; não cobre superfícies grandes;
- forms alinham rótulos, mostram origem/efeito e mantêm ações perigosas
  distintas;
- listas e grids usam identidade estável, não índice como identidade de
  domínio;
- loading preserva contexto; erro oferece próximo passo quando ele é conhecido;
- empty state explica por que está vazio e qual gesto pode mudar isso.

Não renomear mecanicamente todos os componentes para uma nova família. Componentes
`Kv*` atuais são a base; novos primitivos só entram quando eliminam duplicação
real e ganham teste.

## 8. Estado, modelos e bridge Qt

```text
Rust Core       fatos, validação, execução e contratos
Qt/C++ bridge   transporte, threading, lifetime e adaptação de modelos
QML controller  estado de apresentação e coordenação de gesto
QML visual      composição e interação
```

QML pode possuir busca temporária, seleção visual, painel aberto, splitter e
foco. Não pode ser fonte autoritativa de compilador ativo, branch Git, placa,
alvo remoto, sessão de debug, resultado de build ou toolchain efetiva.

`QAbstractItemModel` é indicado quando volume, atualização incremental,
identidade ou ordenação tornam `QVariantList`/`ListModel` um problema medido.
Não é migração obrigatória para todo modelo pequeno.

Não criar agora `FrontendApplication`, `ApplicationStore`, `WorkspaceStore`,
`EnvironmentStore`, `SessionStore`, `ServiceRegistry` ou uma camada C++ por
simetria. Cada classe nova exige responsabilidade distinta, teste isolado e
consumidor real. A revisão consolidada de `arquiKinein` explicitamente revogou
a criação antecipada desses universais.

## 9. Sessões, Jobs, status e notificações

Jobs já são a abstração comum de operação longa. Build, Run, Debug, Test, Flash,
Deploy, Serial e Terminal mantêm seus donos atuais.

Uma `Session` genérica só nasce quando houver requisito compartilhado concreto,
como múltiplas execuções simultâneas, histórico, pinning, compound run ou tabs de
sessão. Até lá, não duplicar lifecycle.

A status bar tem orçamento visual. Prioridade:

1. operação em curso e cancelamento;
2. contexto que altera o significado do gesto atual;
3. saúde/erro que exige atenção;
4. informação acessória no tooltip/overflow.

Evento não abre painel automaticamente. Atualiza status, Problems, Jobs ou a
tool window correspondente; notificação transitória só quando perder o evento
causaria erro ou espera sem explicação.

## 10. Mapa de superfícies por domínio

| Domínio | Superfície principal | Complementos |
| --- | --- | --- |
| Project | Tool window esquerda | comandos, menus de contexto |
| Git | Tool window esquerda | editor para diff/commit, header resumido |
| Símbolos/Structure | Tool window direita | Search Everywhere, navegação |
| Build/Test/Quality | Bottom tool window | Problems, status, editor |
| Run/Debug | Bottom tool window | header, editor, status |
| Terminal | Bottom tool window | múltiplas sessões existentes |
| Embedded | Tool window esquerda | status, Run/Debug, terminal |
| Remote SSH | Tool window + HUD de status | Run/Debug, terminal, Jobs |
| Toolchains/Environment | Tool window ou Settings | status compacto |
| Banco/Containers/Grafana | Tool windows de domínio | Jobs/terminal quando necessário |
| Settings | janela própria | comandos e busca |

O inventário é alvo inicial, não justificativa para migrar tudo no mesmo ciclo.

## 11. Migração segura

Toda fatia segue:

```text
medir baseline
→ introduzir contrato mínimo
→ adaptar a implementação atual
→ provar paridade
→ tornar o caminho novo padrão
→ remover legacy
```

Regras:

- sem big-bang rewrite;
- infraestrutura vem antes da migração visual que depende dela;
- adapter temporário é aceitável, dupla fonte de verdade não;
- toda fase deixa a `main` utilizável;
- legacy só sai depois de paridade funcional, teclado e layout;
- mudança estrutural e redesign visual não precisam ocorrer no mesmo PR;
- feature nova usa os componentes/commands atuais e não cria “legacy novo”;
- screenshots, harness QML, qmllint, catracas e binário abrindo acompanham a
  migração;
- mudanças de protocolo só entram quando melhoram a fronteira real.

## 12. Aplicação à 0.3

A 0.3 usa esta arquitetura como direção, não como obrigação de concluí-la.

Entram quando necessários às fatias reais da 0.3:

- convergência do `CommandDispatcher` atual;
- contrato mínimo de Tool Windows;
- Remote SSH utilizável diariamente;
- identidade estável de abas;
- Structure/Símbolos como prova da área direita;
- componentes/tokens tocados pelas mudanças;
- baseline visual, de teclado, desempenho e acessibilidade.

Não bloqueiam a 0.3:

- migração de todos os domínios;
- Settings reescrito;
- stores universais;
- plugin API pública;
- docking avançado;
- split arbitrário;
- modelo universal de sessions;
- modularização completa dos módulos QML.

A Etapa 4 de LSP, edição e compiladores continua válida. Este plano é uma frente
horizontal: cada fatia da Etapa 4 deve usar os contratos consolidados quando os
tocar, sem parar o backend para reescrever todo o frontend.

## 13. Adiar até existir gatilho real

| Ideia | Gatilho mínimo |
| --- | --- |
| Session model comum | múltiplas sessões, histórico ou pinning em dois domínios |
| Resolution trace genérico | dois fluxos não conseguem explicar override com os campos atuais |
| Context-key language | condições duplicadas e divergentes em múltiplos consumidores |
| Decoration registry | múltiplos providers competem pela mesma camada do editor |
| Plugin contributions | Commands e Tool Windows internas estabilizadas |
| Dock graph | left/right/bottom deixam de atender fluxo real |
| Modelos C++ adicionais | volume/latência/identidade tornam o modelo QML insuficiente |

## 14. Decisões ainda necessárias — não descartar sem perguntar

- Se a primeira tool window direita deve chamar **Símbolos**, **Structure** ou
  expor os dois modos com esses nomes.
- Se a escolha Projeto/Git/Remote da área esquerda deve persistir por workspace
  ou globalmente.
- Se preview tab e pin entram na 0.3 ou imediatamente depois.
- Qual política de split é suficiente após o primeiro split de dois grupos.
- Quais dados técnicos do dispositivo devem usar o termo “telemetria”. O termo
  está proibido para coleta do produto; dados do alvo foram preservados como
  **Saída do alvo/Observabilidade** até decisão de nomenclatura.
- O limite exato entre Environment, Toolchains e Settings depois que houver uso
  diário suficiente para medir sobreposição.

## 15. Fonte extraída e partes não promovidas

Foram promovidos de `DocsPrivate/arquiKinein/`: shell centrado no editor, áreas
left/right/bottom, Tool Windows mínimas, commands por ID, design system,
estados obrigatórios, acessibilidade, tabs robustas, split incremental,
fronteiras Rust/C++/QML, migração por estrangulamento e gatilhos contra
overengineering.

Não foram promovidos como decisão imediata: stores universais, registry para
tudo, reescrita ampla do bridge em C++, plugin API, dock graph completo,
Session universal, nova série de Partes e a premissa de que Standard Make define
sozinho a fronteira da 0.3. Essas ideias permanecem no material privado para
consulta e só avançam pelos gatilhos acima.

O alvo específico de Remote SSH está em
[`remote-ssh-ui-hud.md`](remote-ssh-ui-hud.md).
