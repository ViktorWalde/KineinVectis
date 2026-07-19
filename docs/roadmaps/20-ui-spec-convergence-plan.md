# 20 — Convergência da UI atual para as specs (plano vinculante)

> **Status:** ativo e vinculante
> **Prioridade:** P0 para qualquer trabalho de UI
> **Fonte de verdade:** ESTE doc define COMO e QUANDO a UI converge;
> `docs/specs/` define PARA ONDE (inegociável); `docsprivate/diario/18` sequencia as
> fatias; `docs/arquitetura/19` registra a decisão (D12)
> **Ultima revisao:** 2026-07-15

## O problema, dito sem rodeio

A UI/UX implementada hoje **não reflete `docs/specs/`**. Ela nasceu como MVP
funcional (e cumpriu esse papel: M0 inteiro roda nela), mas paleta, layout,
dimensões e iconografia divergem da especificação canônica. Este documento
existe para que a convergência seja **explícita, ordenada e auditável** — em
nenhum momento "de entendimento vago".

## A decisão (2026-07-09, confirmada com o usuário)

**Convergência gradual (strangler fig), NÃO remake big-bang.** Cada fatia
substitui uma área da UI pela versão conforme spec, com a IDE utilizável o
tempo inteiro. Registrada como decisão D12 em
`docs/arquitetura/19-architecture-tradeoffs.md`, com gatilho objetivo de reavaliação
(seção "Gatilho de remake" abaixo).

**Por quê gradual e não remake total:**

```text
1. A arquitetura já separou visual de lógica (docs/arquitetura/17): componentes QML
   recebem property e emitem signal; estado vive em controllers; IPC em
   roteadores. Trocar a PELE de uma área não toca controller nem core —
   o custo de um remake visual incremental é baixo AQUI, porque o
   trabalho estrutural que o torna barato já foi feito e validado.
2. Big-bang congela entregas por semanas num projeto de uma pessoa cujo
   objetivo declarado é virar daily driver o quanto antes (docsprivate/diario/18).
   Gradual mantém o dogfooding vivo — e dogfooding é o que valida spec.
3. Não existe teste visual automatizado (docs/arquitetura/19, D9): um big-bang
   entregaria a superfície inteira de uma vez sem rede de segurança.
   Fatias pequenas mantêm o risco de regressão visual proporcional.
4. Precedente interno: Main.qml 4.6k → 336 linhas foi feito exatamente
   assim (fatias com gate verde) e não quebrou comportamento.
5. Parte do alvo das specs é FUNCIONALIDADE nova (Main Toolbar, KV
   Context, Start Screen, breadcrumbs). Isso não é "refazer o que
   existe", é construir o que falta — e o que falta já nasce conforme
   spec pela regra R1, convergindo naturalmente.
```

**O que o remake total daria e estamos abrindo mão conscientemente:** um
"corte limpo" sem estados intermediários visualmente híbridos. Durante a
convergência a IDE terá áreas já-conformes convivendo com áreas antigas.
Aceito: é uma IDE de uso próprio; consistência final > pureza intermediária.

## Regras vinculantes

```text
R1. UI NOVA nasce conforme spec, sempre. Antes de codar qualquer superfície
    nova, ler a spec da área (tabela "Specs por área" abaixo). Se a spec
    for omissa no detalhe, seguir o componente mais próximo JÁ conforme e
    registrar a interpretação na fatia (docsprivate/diario/18).
R2. Fatia de convergência (C*) é dedicada: não se mistura com feature no
    mesmo diff. Feature que precisa de área ainda não convergida usa a
    área como está (funcional primeiro), e a convergência daquela área
    entra na fila C.
R3. Cada marco de docsprivate/diario/18 carrega as fatias C mapeadas abaixo. Fatia C
    atrasada bloqueia a PRÓXIMA fatia de UI do marco seguinte (gate).
R4. "Conforme" é objetivo, não estético: valores de token idênticos aos da
    spec, dimensões idênticas, regiões presentes com o conteúdo listado.
    "Ficou parecido" NÃO é conforme.
R5. Divergência descoberta durante qualquer trabalho vira linha na tabela
    de inventário deste doc (com data), nunca correção silenciosa nem
    omissão.
R6. Todo valor visual passa por Theme.qml (single source). Convergir
    paleta/tipografia/dimensão = trocar tokens e consumidores dos tokens;
    cor solta em componente é bug de review (qmllint não pega isso — é
    revisão humana).
R7. Enquanto não houver teste visual automatizado, fatia C fecha com:
    gate completo verde + smoke offscreen + verificação visual do usuário
    contra a spec da área (screenshot lado a lado quando possível).
```

## Specs por área (o que ler antes de tocar em quê)

| Área | Spec canônica |
| --- | --- |
| Layout/regiões/dimensões | `KINEIN_VECTIS_LAYOUT_SYSTEM.md` |
| Paleta/tipografia/ícones | `KINEIN_VECTIS_VISUAL_SYSTEM_ICONS.md` |
| Componentes (botões, tabs, listas...) | `KINEIN_VECTIS_UI_COMPONENTS_SYSTEM.md` |
| Fluxos build/run/debug | `KINEIN_VECTIS_PRODUCT_FLOWS_BUILD_RUN_DEBUG.md` |
| Editor/inteligência | `KINEIN_VECTIS_EDITOR_LANGUAGE_INTELLIGENCE.md` |
| Onboarding/start/wizard | `KINEIN_VECTIS_ONBOARDING_*.md` |
| Índice geral | `KINEIN_VECTIS_SPEC_INDEX.md` |

## Inventário de divergências — AUDITORIA C0 (executada em 2026-07-09)

Auditoria formal por área, valores atuais medidos no código
(`Theme.qml` e componentes) contra os valores normativos das specs, com
referência exata (arquivo§seção). `LAYOUT` =
`KINEIN_VECTIS_LAYOUT_SYSTEM.md`; `VISUAL` =
`KINEIN_VECTIS_VISUAL_SYSTEM_ICONS.md`; `COMP` =
`KINEIN_VECTIS_UI_COMPONENTS_SYSTEM.md`.

### A. Paleta (LAYOUT §7.1, VISUAL §3.1) — diverge em todos os tokens

| Token (Theme.qml hoje) | Spec | Ref |
| --- | --- | --- |
| background0 `#0d0e0e` | `--kv-bg-app #0B0D10` | LAYOUT §7.1 |
| background1 `#121313` | `--kv-bg-surface #111418` | LAYOUT §7.1 |
| background2 `#191a18` | `--kv-bg-elevated #171B21` | LAYOUT §7.1 |
| (sem token de editor) | `--kv-bg-editor #0F1216` | LAYOUT §7.1 |
| (sem token de linha atual) | `--kv-bg-current-line #1A1F26` | LAYOUT §7.1 |
| borderSoft `#2a2922` | `--kv-border-subtle #2A2F37` (+ `strong #3A414A` ausente) | LAYOUT §7.1 |
| textPrimary `#eae6e1` | `#E7E2D8` | LAYOUT §7.1 |
| textSecondary `#b9b3a5` | `#A9A39A` | LAYOUT §7.1 |
| textMuted `#8f8a7c` | `#6F737A` (+ disabled `#4E535A` ausente) | LAYOUT §7.1 |
| accent `#ffbb00` | `--kv-amber #FFB000` (+ `active #FFC93D`, `dark #B97900` ausentes) | LAYOUT §7.1, VISUAL §3.1 |
| successSoft `#7fbf7f` | `#7CCF6A` | LAYOUT §7.1 |
| errorSoft `#d16d6d` | `#D45F5F` | LAYOUT §7.1 |
| infoSoft `#7aa2d8` | `#5C8DFF` | LAYOUT §7.1 |
| warningSoft `#ffbb00` | `#E6B84A` (amarelo distinto do âmbar) | LAYOUT §7.1 |
| (ausente) | roxo orbital `#8A5CFF` | LAYOUT §7.1 |

### B. Regiões de layout (LAYOUT §4.1, §9.1)

| Região | Hoje | Spec | Status |
| --- | --- | --- | --- |
| 1 Title/App Bar | AppMenuBar 40px, identidade + 9 menus; decoração server-side ainda separada | menus File/Edit/View/... 40px + ações de janela | **parcial**: conteúdo conforme; integração das ações/chrome pendente |
| 2 Main Toolbar | 44px, target/profile/configure/build/test/quality/run/debug | Target/Profile/Configure/Build/Run/Debug, 44px | **conforme em código; validação visual pendente** |
| 3 Tool Rail | 42px | 52px (48–56) | **diverge** (abaixo do mínimo) |
| 4 Left Tool Window | Project, 280px automático (220–420), redimensionável/persistido | 280px (220–420) redimensionável; Project/Structure/CMake/Toolchains/Targets | **parcial** (dimensionamento conforme; outras tool views futuras) |
| 5 Editor Area | tabs+texto+popups | + breadcrumbs (COMP §13.3), gutter (COMP §13.4), linha atual | **parcial** |
| 6 Assistente | seletor 360px; sessão terminal ajustável/persistida 300–720px + maximização; Project independente; Claude/Codex sobre PTY existente | AI CLI Bridge externo, terminal-first e separável do Terminal comum | **correções 0.51/0.52 em código; validação funcional/visual pendente** |
| 7 Bottom Tool Window | 260px automático (160–480), redimensionável/persistido | 260px (160–480) redimensionável | **conforme em código; validação visual pendente** |
| 8 Status Bar | 26px; indicadores de jobs | 28px; branch, erros/avisos, profile, compiler, target, Ln/Col, encoding, linguagem (LAYOUT §17.2–17.3) | **parcial** |

### C. Dimensões normativas (LAYOUT §6.3–6.4)

| Elemento | Hoje | Spec |
| --- | --- | --- |
| Tab bar do editor | 30px (abas 26px) | 36px |
| Bottom tool tabs | 20px | 34px |
| Status bar | 26px | 28px |
| Item da project tree | 24px | 24px ✓ **conforme** |
| Botão pequeno | 28px | 28px ✓ **conforme** |
| Breadcrumb bar | ausente | 26px |

### D. Escalas (LAYOUT §6.2; COMP §6–7)

| Escala | Hoje | Spec |
| --- | --- | --- |
| Espaçamento | 6/12/20 (+gap 8) | escala de 4px: 2/4/8/12/16/24/32 (6 e 20 fora) |
| Raios | 6 e 10 | XS 3 / SM 5 / MD 8 / LG 12 / XL 18 (6 e 10 fora) |

### E. Tipografia (LAYOUT §8)

| Elemento | Hoje | Spec |
| --- | --- | --- |
| Famílias UI/código | Inter+Noto / JetBrains Mono | idem ✓ **conforme** (LAYOUT §8.1–8.2) |
| Editor | 13px (a auditoria inicial mediu 14px no texto de estado vazio, não no TextEdit — corrigido na C1) | 14–15px |
| Terminal | 11px | 13px |
| Project tree | 12px | 13px |
| Status bar | 10px | 12px |
| Título de painel | 11–12px bold | 13px semibold |

### F. Iconografia (VISUAL §4–5) — conforme em código (C2; R7 pendente)

`KvIcon.qml` centraliza desenhos vetoriais lineares, grid 24×24 e stroke
1.75px; `KvIconButton` aplica estados default/hover/ativo/desabilitado e
acessibilidade. Rail, toolbar, tabs, painéis, diálogos e controles de ação
deixaram de depender de glifos Unicode/fontes. A árvore é a exceção deliberada:
os cinco SVGs autorais fornecidos pelo usuário vivem em `ui/assets/icons/tree/`
com bytes idênticos aos masters e são dimensionados pelo mesmo `KvIcon`, sem
redesenho ou mapa paralelo.

### G. Componentes catalogados (COMP §11–13)

Formalizados e em uso: `KvButton`, `KvIconButton`, `KvTooltip`,
`PanelSplitter`, breadcrumbs e gutter. `SettingsToggleRow` é o toggle formal
da Settings atual. `KVSelect` e `KVBadge/KVChip` genéricos ainda não existem;
os seletores/chips específicos permanecem locais até haver um segundo caso
real que justifique extração (política anti-duplicação/abstração prematura).

### H. Onboarding (ONBOARDING_*) — implementação material em C5

Start Screen aparece sem workspace, mostra saúde de ferramentas e oferece
Abrir/Novo C++/Novo Rust/Settings sem instalar nada. A criação reutiliza o
FolderPicker existente e mostra preview de arquivos/comandos. O scaffold C++
é target-based C++23 com presets Debug/Release; Rust delega ao Cargo. Um First
Run Setup modal separado não foi criado: a detecção passiva da Start Screen
cobre o único fluxo atual sem forçar wizard/login; reavaliar quando houver
toolchain manager configurável.

### Conformidades já existentes (registro honesto)

Famílias tipográficas; editor 14px; item de tree 24px; botão pequeno 28px;
disciplina de uso do âmbar (accent contido em ativo/foco/primário, alinhado
a LAYOUT §7.2); popups escuros com borda sutil; densidade geral próxima do
alvo.

## Ordem de convergência (fatias C, amarradas aos marcos de docsprivate/diario/18)

```text
C0 [FEITA 2026-07-09] Auditoria formal executada; resultado é a tabela
   acima (áreas A–H, com referências arquivo§seção e conformidades
   registradas). O M1 está fechado com esta fatia.
C1 [FEITA 2026-07-09; validação visual do usuário pendente — R7]
   Executada conforme a seção "Execução da C1" abaixo: (a) tokens da spec
   no Theme.qml (paleta completa da área A, escalas da área D, tamanhos
   tipográficos da área E nos elementos auditados); (b) dimensões das
   regiões da área B/C (rail 52, status 28, bottom tabs 34, tab bar 36,
   left 280, bottom 260, Assistente 360); (c) PanelSplitter para
   redimensionar left/bottom/context nos min/max da spec; (d) correção
   semântica: fundo de seleção agora é surfaceSelected #222833 (âmbar só
   como acento), conforme COMP §10.4. Após validação visual do usuário,
   atualizar as linhas das áreas A/C/D do inventário para "conforme".
C2 [FEITA 2026-07-14; validação visual R7 pendente] Iconografia: `KvIcon`
   vetorial + botões/tooltip centrais substituíram glifos de texto nas ações.
   Em 2026-07-15, pasta fechada/aberta e arquivos C/C++/Rust passaram a usar
   fielmente os SVGs entregues pelo usuário via o mesmo contrato.
C3 [FEITA 2026-07-14; validação visual R7 pendente] Main Toolbar (região 2)
   nasceu
   conforme spec — aqui feature e convergência coincidem por natureza
   (target/profile/build/run/debug são funcionalidade nova).
C4 [durante M3] Editor Area: breadcrumbs, gutter de diagnósticos (depois
   breakpoints no M2/M3 do debugger), linha atual, conforme
   EDITOR_LANGUAGE_INTELLIGENCE + LAYOUT_SYSTEM.
   [parcial 2026-07-09, fatias M2.5b/M2.5c] A gutter NASCEU no editor:
   números de linha (janela visível, sem custo em
   arquivo longo), breakpoints por clique (bolinha errorSoft) e linha de
   execução do debugger destacada (accentDim 35%). M2.5c somou a linha
   do cursor (surfaceSelected, some na seleção) e breadcrumbs do caminho
   relativo acima do editor (segmento de símbolo LSP fica pós-M2).
   [C4 fechada em 2026-07-11, fatia T6] Gutter de diagnósticos ENTREGUE:
   o store por arquivo virou o DiagnosticsController (consome o mesmo
   lspDiagnostics que a aba Problemas); o editor sublinha o range
   (ondulado por severidade, no EditorHighlighter) e marca a linha na
   gutter com tooltip da mensagem; navegação F2/Shift+F2. O sinal
   lspDiagnostics deixou de ser não-consumido. Pendente só a validação
   visual do usuário (R7). Underline aponta para os semantic tokens/
   layout de EDITOR_LANGUAGE_INTELLIGENCE; refino fino (marker bar à
   direita, contadores) fica para o Problems 2.0/C6.
   [correção de dogfooding 2026-07-15] O layout saiu de
   `EditorTextSurface.qml` para `EditorGutter.qml`. Folding/breakpoint,
   diagnóstico, blame e número agora ocupam faixas independentes; a largura
   numérica usa `FontMetrics`, eliminando a sobreposição do breakpoint em
   arquivos com mais dígitos ou fonte ampliada.
C5 [FEITA 2026-07-14; remediações funcionais 0.50–0.52; validação R7 pendente]
   App Bar com menus em overlay global e ações ligadas, Assistente como AI CLI
   Bridge Claude/Codex sobre PTY, Start Screen e criação com preview conforme
   ONBOARDING_*. O dogfooding abriu uma correção vinculada em 0.51: Codex em
   modo inline oficial para scrollback, barra persistente, largura terminal
   responsiva, maximização e input/resize com paridade do Terminal integrado.
   A largura 300–480 continua normativa para o seletor/estado compacto; a CLI
   ativa pode ocupar até 720px ou a área de trabalho porque passa a ser um
   layout operacional de terminal, conforme a Parte 7.1 que supera o KV
   Context-chat original. A tentativa intermediária de guia da linha ativa foi
   removida em 2026-07-15: conteúdo, ordem, input e cursor pertencem somente ao
   grid VT da CLI, sem composer, moldura ou parsing paralelo.
   O terceiro feedback foi tratado no 0.52: a sessão ativa mantém o splitter,
   persiste sua largura própria em 300–720px, não fecha `Project`, preserva o
   scroll durante nova saída e impede somente `CSI 3 J` nas sessões do bridge
   de apagar o transcript inline. O Terminal comum continua honrando `clear`.
C6 [PENDENTE: validação visual do usuário] Auditoria de fechamento: tabela do inventário inteira "conforme";
   o que sobrar vira decisão registrada (spec ajustada OU fatia extra).
```

## Execução da C1 (design fechado em 2026-07-09)

**Mapeamento de tokens (Theme.qml), atual → spec:**

```text
background0 #0d0e0e → #0B0D10 (bg-app)         accent      #ffbb00 → #FFB000
background1 #121313 → #111418 (bg-surface)     accentDim   #6e5c01 → #B97900 (amber-dark;
background2 #191a18 → #171B21 (bg-elevated)                  pressed/seleção de texto)
surface1    #1f201d → #171B21 (elevated)       errorSoft   #d16d6d → #D45F5F
surface2    #25261f → #1A1F27 (hover, COMP     warningSoft #ffbb00 → #E6B84A
             §10.2)                            successSoft #7fbf7f → #7CCF6A
borderSoft  #2a2922 → #2A2F37                  infoSoft    #7aa2d8 → #5C8DFF
textPrimary #eae6e1 → #E7E2D8                  neutralOlive: REMOVIDO (sem uso)
textSecondary #b9b3a5 → #A9A39A
textMuted   #8f8a7c → #6F737A

NOVOS: backgroundEditor #0F1216, currentLine #1A1F26, surfaceSelected
#222833 (COMP §10.4 — fundo selecionado NÃO é âmbar; âmbar é a linha/
detalhe), borderStrong #3A414A, textDisabled #4E535A, accentActive
#FFC93D, purpleOrbital #8A5CFF.

Escalas: spacingXSmall 4 (novo), spacingSmall 6→8, spacingMedium 12 (=),
spacingLarge 20→16, spacingRegion 24 (novo); radiusXSmall 3 (novo),
radius 6→5, radiusLarge 10→8, radiusDialog 12 (novo); panelGap 8 (=).
[panelGap REMOVIDO em 2026-07-18: o modelo de cartoes sobre calhas caiu;
regioes encostadas com divisor de 1px (seamWidth) + alca invisivel
(splitterGrip). Ver LAYOUT_SYSTEM §4.2.]
Tipografia (tokens novos): fontSizeStatus 12, fontSizeTree 13,
fontSizeTerminal 13, fontSizePanelTitle 13, fontSizeEditor 14 — aplicados
nos elementos auditados na área E (status bar 10→12, tree 12→13,
terminal 11→13, títulos de painel →13).
```

**Correção semântica incluída:** os usos de `accentDim` como FUNDO de item
selecionado (tree, seletor de pastas, Search Everywhere, completion)
migram para `surfaceSelected` — a spec manda fundo `#222833` com âmbar só
em linha/acento (COMP §10.4, LAYOUT §7.2). `accentDim` (agora amber-dark)
permanece em pressed e na seleção de texto do editor/inputs.

**Dimensões:** rail 42→52 (botões 30→32); status bar 26→28; editor tab bar
30→36 (abas 26→30, botão salvar 24→28); tabs do painel inferior 20→26
(strip fecha em 34 com a margem nova de 8); explorer 260→280; painel
inferior 170→260; Assistente 300→360.

**KVSplitter mínimo:** `shell/PanelSplitter.qml` como alça de arraste em
OVERLAY nas bordas (não muda a estrutura do ShellLayout): explorer|editor,
editor|assistente e editor|painel-inferior. Tamanhos viram propriedades do
`ShellController` (`explorerWidth`, `contextWidth`, `bottomPanelHeight`)
com clamp nos min/max da spec (220–420, 300–480, 160–480). Persistência de
layout fica para Settings/M4 (por ora, por sessão de execução).

**Fora do escopo da C1** (ficam para C2+): iconografia, componentes KV*
formais, breadcrumbs/gutter, Main Toolbar, Title Bar, demais tamanhos de
fonte não auditados na área E.

**Validação (R7):** gate completo + smoke offscreen + verificação visual
do usuário contra LAYOUT §6–7/§9 e VISUAL §3 (a paleta muda a IDE inteira;
regressões visuais só o olho pega).

## Conforto imediato e reformulação futura da Title/App Bar (2026-07-15)

**Evidência de dogfooding:** `docsprivate/imagens/bugs/ReformularBarra.png` mostra o nome
do workspace e do produto em branco puro na decoração nativa, acima da App Bar
já tematizada. O contraste e a duplicação causam fadiga visual.

**Correção mínima entregue:** `Main.qml` mantém a decoração server-side, mas
deixa seu título vazio. A identidade continua na App Bar e o workspace passa a
aparecer ali com `Theme.textMuted`, elide central e sem cor literal. Isso
remove o texto branco duplicado sem introduzir ainda uma decoração client-side.

**Correção do aceite pelo dogfooding (bloqueante antes de novo código):** no
AppImage aberto em Fedora/Wayland, a barra resultante não apresenta as ações
visíveis **Minimizar**, **Maximizar** e **Restaurar**. Portanto, a afirmação de
que a mitigação preservava todos os controles nativos não está aceita. Registrar
como regressão P2 do shell, anterior ao polimento P3 da barra completa.

Critérios obrigatórios da próxima correção, antes de remover qualquer moldura:

- exibir Minimizar e um único controle alternável Maximizar/Restaurar, além de
  Fechar, com ícones, tooltip e nomes acessíveis Kinein;
- Maximizar deve aparecer no estado normal; Restaurar deve substituí-lo apenas
  quando a janela estiver maximizada e recuperar a geometria normal anterior;
- o estado autoritativo vem da janela/Qt e do compositor, sem booleano visual
  paralelo em QML;
- duplo clique e arraste da região livre não podem competir com os botões;
- validar clique, teclado, foco, Wayland e X11 em janela normal, maximizada e
  restaurada; smoke offscreen sozinho não serve como aceite desses controles;
- nenhuma nova funcionalidade da barra é considerada pronta enquanto os três
  comandos relatados pelo usuário não estiverem visíveis e funcionais.

**Referência profissional atual:** Zed oficial na revisão
`1e22d1a83f8b1b7acc528d15cfab0644852380c0` (2026-07-15) separa decoração
`client`/`server` e oferece tokens próprios para fundo ativo/inativo da title
bar. A documentação oficial do [Qt 6 Window](https://doc.qt.io/qt-6/qml-qtquick-window.html)
confirma que `title` é apenas o texto entregue ao sistema; a documentação de
[window flags](https://doc.qt.io/qt-6/qt.html#WindowType-enum) alerta que
`FramelessWindowHint` pode remover a manipulação nativa de move/resize. Foram
extraídos somente invariantes e modos de falha; nenhum código, runtime ou
arquitetura do Zed foi copiado.

**Implementação entregue (2026-07-15, a pedido explícito do usuário):** os
controles foram integrados à barra da própria IDE, no estilo de acabamento das
IDEs JetBrains, sem copiar sua composição. Por decisão do usuário, a decoração
client-side foi antecipada — `Main.qml` passa a `Qt.Window |
Qt.FramelessWindowHint`. Mapa da fatia:

- `ui/src/window_chrome_controller.{h,cpp}` — `WindowChromeController`
  (`QML_ELEMENT` do módulo `KineinVectis`) guarda um `QPointer<QQuickWindow>` e
  expõe o estado autoritativo `maximized` (de `QWindow::windowStates`) e as ações
  `minimize`/`toggleMaximized`/`closeWindow`/`startSystemMove`/
  `startSystemResize`. `toggleMaximized` faz `showNormal`/`showMaximized`; o
  restaurar recupera a geometria normal via Qt/compositor, sem geometria manual.
- `ui/qml/shell/WindowControls.qml` — Minimizar, um único alternável
  Maximizar/Restaurar e Fechar como `KvIconButton` (tooltip e `Accessible.name`
  Kinein). O ícone/rótulo do alternável deriva de `maximized`; não há booleano
  visual paralelo.
- `ui/qml/shell/AppMenuBar.qml` — hospeda os controles à direita; a região livre
  (`dragRegion`) arrasta a janela via `startSystemMove` (limiar de 6px) e o duplo
  clique alterna maximizar, sem competir com os botões. A marca `Kinein` some
  abaixo de 850px para preservar legibilidade.
- `ui/qml/shell/WindowResizeHandles.qml` — oito bordas com `startSystemResize`,
  desabilitadas quando a janela está maximizada ou em fullscreen.
- `ui/qml/components/KvIcon.qml` — ícones vetoriais `minimize`, `maximize` e
  `restore`.

**Referência profissional desta fatia:** IntelliJ IDEA Community oficial
`e3b4dba36d013fc221b8471b3a4a8bd5336c24cc` (2026-07-15), em
`platform/platform-impl/src/com/intellij/openapi/wm/impl/customFrameDecorations`
e `WindowButtonsConfiguration.kt`, Apache-2.0, Mode-D. Invariantes adotados:
controles no header; ordem minimizar → maximizar/restaurar → fechar; alternável
guiado pelo estado real da janela; atualização quando o estado externo muda.
Adaptação nativa em Qt/QML — nenhum código, Swing/JBR ou classe do IntelliJ foi
copiado ou portado.

**Estado de aceite — ACEITO (2026-07-15):** gates automatizados verdes
(`scripts/verificar.sh` integral + smoke offscreen `exit 124`) e, sobretudo, o
usuário testou em Fedora/Wayland e confirmou que Minimizar, Maximizar/Restaurar,
Fechar, arraste, duplo clique e resize das oito bordas funcionam. A regressão P2
está encerrada. Um AppImage 0.1.0 novo foi gerado e testado com este código. O
polimento restante (snap, escala fracionária, multimonitor auditados) segue como
fatia P3 própria abaixo.

**Fatia futura, sem big-bang:** transformar a App Bar existente na decoração
client-side original da Kinein, suave e compacta, inspirada no nível de
acabamento das IDEs JetBrains sem copiar sua composição. Critérios de aceite:

- título e estado ativo/inativo usam apenas tokens Kinein, sem branco puro;
- marca curta `Kinein`, workspace discreto, menus e ações de janela convivem
  sem perder legibilidade em 800px;
- arraste, duplo clique, snap, minimizar, maximizar/restaurar, fechar e resize
  nas oito bordas funcionam em X11 e Wayland;
- foco, teclado, nomes acessíveis, escala fracionária e múltiplos monitores
  permanecem corretos;
- a decoração nativa só é removida depois de smoke visual real nos dois
  backends; até lá, não usar `FramelessWindowHint`.

## Gatilho de remake (quando esta decisão seria revertida)

Reavaliar big-bang com o usuário — não silenciosamente — se, e somente se:

```text
- na C1, trocar os tokens exigir reescrever mais da metade dos componentes
  visuais (sinal de que o acoplamento visual é pior que o mapeado); OU
- duas fatias C consecutivas custarem mais que o dobro do estimado; OU
- o híbrido intermediário começar a atrapalhar o dogfooding diário
  (relato do usuário, não impressão do agente).
```

## Definition of done de uma fatia C

```text
[ ] Área listada no inventário com referência de spec.
[ ] Valores/estruturas idênticos aos da spec (R4), via tokens (R6).
[ ] Comportamento funcional preservado (mesmos sinais/controllers).
[ ] Gate completo verde + smoke offscreen.
[ ] Verificação visual do usuário contra a spec (R7).
[ ] Inventário atualizado para "conforme" + ContextoIA sincronizado.
```
