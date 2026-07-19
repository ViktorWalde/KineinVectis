# PONTO ATUAL — fila viva do Kinein Vectis (2026-07-15)

> Este arquivo contém somente trabalho presente ou futuro, em ordem de
> execução. Trabalho concluído deve ser registrado no documento do domínio e
> em `ContextoIA.md`, e então removido daqui.
>
> **Estado implementado: o CÓDIGO — mede-se, não se lê.** O `ContextoIA.md` é
> LOG datado e não responde "o que existe hoje" (rebaixado em 2026-07-17; ver
> `docs/README.md`). Mapa: `GUIAIA.md`. Histórico: Git. Protocolo `0.61.0`.
>
> Não alterar a UI fora das specs. Commits locais de checkpoint após marco
> crítico/teste verde foram autorizados em 2026-07-15; push e publicação não
> foram. **Dogfooding/self-hosting ativo desde 2026-07-14:** o usuário já
> está na Kinein, e cada bloqueio ou saída para outra IDE passa a ordenar o
> backlog antes de funcionalidade nova.

## TRILHA — leia isto primeiro (medido em 2026-07-17)

> **Classe deste arquivo: ESTADO** (`docs/README.md`). Tem que ser verdade HOJE.
> Se divergir do código, **o código vence** e este arquivo se corrige no mesmo
> gesto. Não é log — log é o `ContextoIA.md`, e ele não manda em nada.

### Regra zero, antes de qualquer item

```text
MEDIR.  Fila e' hipotese, nao estado.
        `ls` no artefato · `git log` na area · grep no gate · rodar.
```

Isto não é cerimônia: em 2026-07-17 esta fila listava como "design pronto" um
harness que a §A3 **deste mesmo arquivo** dava como entregue — e o arquivo
existia. Medir custa 30 segundos; reimplementar o que existe custa uma fatia.

### Onde o projeto está (tudo medido, nada herdado)

```text
HEAD          d57a96d          protocolo 0.61.0        gate completo: VERDE
L0            FECHADO          A3.1-A3.4; typing_perf_harness.cpp existe,
                               mediana 7,4 ms / p95 8,4 ms (orcamento 16/20)
cursor/TUI    APROVADO         pelo autor em 2026-07-17. Fecha R0-R3.
IA na IDE     FORA DE ESCOPO   0 ocorrencias em ui/qml. Nao reabrir.
debito        15 arquivos      catraca verde (conta so CODIGO desde
                               2026-07-17); 8 arquivos sairam nesta sessao
gates         8                verificar.sh: fmt/test/clippy, cpp, qml,
                               qml-fiacao, docs, presets, icone, arquitetura,
                               qml-logica (+presets +icone nesta sessao)
teste C++     2 alvos          ui/tests: 23 casos (highlighter 13,
                               auto-close regions 10); ctest no gate
harnesses QML 16               +tst_autoclose +tst_code_actions
handlers core 17               +integration (L1 v1 read-only)
binario atalho c18cad8         1 commit atras do HEAD: falta o integration.list
                               (core Rust, sem UI ainda — nao afeta dogfooding).
                               atualizar-tudo.sh --ide fecha o manifesto no HEAD.
```

### PARA A PRÓXIMA SESSÃO — comece aqui (handoff de 2026-07-17)

O que a sessão de 2026-07-17 fechou: 3 bugs (autocomplete/rehighlight, auto-close
type-over, flake do `tools::`), a mina de cache (gate de presets, ícone com fonte
única, catraca contando só código), infra de teste C++ do zero, 2 god-files
RESOLVIDOS (dispatch, highlighter) + EditorController começado, e o L1
`integration` v1 read-only. Tudo commitado, gate completo verde.

**DESVIO AUTORIZADO PELO AUTOR em 2026-07-18 — árvore de projeto, fatia 1 de N.**
Pedido direto: arrastar arquivo/pasta com o mouse, e a navegação do IntelliJ IDEA
Community ("memória muscular JetBrains"). Entregue nesta sessão, gate completo
verde: arraste (move via `fs.rename`, que JÁ era move — zero mudança de core ou
protocolo) + teclado (↑↓, ←→ colapsa/expande, Enter, Delete, Alt+1 foca a árvore,
Esc volta ao editor). Arquivos novos: `ProjectTreeGestures` (decide gesto),
`ProjectTreeRow` (linha: fonte de arraste e alvo de drop), `ProjectExplorerHost`
(fiação). `ShellWorkspaceHost` ENCOLHEU 537→530 para caber (regra "quem toca a
área paga a dela"); baseline atualizada no mesmo commit.
GESTO HUMANO FEITO E ACEITO em 2026-07-18 ("está funcionando arrastar uma
pasta para outra"). As regras seguem provadas por `tst_project_tree_dnd.qml`
(3 mutações reprovam). Commit `40d1f15`.
O QUE NÃO ENTROU, e é o que o autor pediu como referência: multi-seleção
(Ctrl/Shift+clique), `Ctrl+C/X/V` (exige um `fs.copy` NOVO no core — único item
que não é só UI), `F5` Copy…/`F6` Move…, speed search, autoscroll from source,
`Ctrl+Shift+C` copiar caminho. Ver a tabela de paridade decidida com o autor.

**DESVIO, PARTE 2 — layout/UX geral (aprovado em 2026-07-18, EM ANDAMENTO).**
Arraste aceito no gesto real ("está funcionando"). O autor então pediu o
layout como um todo na referência IntelliJ IDEA Community + specs, e apontou a
causa com precisão: ferramentas "espalhadas", visual "poluído", bordas da
janela mal aproveitadas — controles de janela "deslocados" em vez de
embutidos, ícones que deveriam ser "plano de fundo" nas bordas. A invariante
pedida (specs + referência IntelliJ Community, memória muscular) está
registrada na LAYOUT_SYSTEM §2.0; a regra "UI burra, zero regra de negócio"
está lá dentro. Print de referência: `imagens/prints/` (2026-07-16).

PLANO DE 4 FATIAS, ordem decidida com o autor (a observação dele reordenou —
o painel inferior era a 1ª e virou 3ª, porque é sintoma da calha, não causa):

```text
F1  Modelo de superfície do shell (§4.2 da LAYOUT_SYSTEM) — ACEITA pelo
    autor em 2026-07-18 ("bem melhor"), commit `aec1b06`, COM uma regressão
    apontada no aceite: os painéis perderam TODO o raio e ficou "layout do
    VS Code". O §4.2 saiu radical demais (flat total); o correto JetBrains é
    raio nas regiões SEM contorno, sobre o fundo da janela. Correção na F1b.
F1b Feedback do aceite da F1 (2026-07-18) — 4 de 5 FEITOS no mesmo dia:
    [x] raio de volta nos PAINÉIS (radiusLarge sem contorno; moldura — rail,
        barras, status — segue plana). §4.2 corrigida no mesmo commit.
    [x] ícone de terminal no rail (toggle JetBrains: ativo esconde, senão
        openTerminalPanel materializa sessão). Rail agora recebe estado cru
        bottomOpen/bottomTab e mapeia sozinho; ShellWorkspaceHost ENCOLHEU
        530→529 apesar do handler novo.
    [x] cabeçalho da árvore: chips de criação removidos (contexto + menu
        Arquivo cobrem); header por ÂNCORA com nome em elide — sobreposição
        com o chip do build system morta pela raiz.
    [x] ícones da árvore no AppImage — CAUSA-RAIZ: o binário TEM os SVGs
        (qrc), mas o linuxdeploy-plugin-qt não embarca libqsvg.so (plugin de
        runtime, invisível à análise ELF); dev usa o Qt do sistema, por isso
        só o AppImage quebra. Fix: EXTRA_QT_PLUGINS=svg + validação que
        REPROVA o empacotamento sem o plugin (empacotar-appimage.sh). NOTA:
        o dist/ atual (07-17) continua sem ícones até a próxima geração —
        que segue aguardando a decisão "AppImage só quando estabilizar".
    [x] modo "imagem" (markdown renderizado) — FEITO 2026-07-18, AGUARDA
        GESTO HUMANO. Chip "Imagem"/"Código" no canto do editor em arquivo
        .md; render idêntico ao Manual (TextEdit.MarkdownText); estado POR
        ARQUIVO no EditorMarkdownModeController (apresentação, não negócio;
        core nunca sabe), limpo ao fechar workspace. O débito foi PAGO por
        extração antes da feature: EditorBreadcrumbs saiu do EditorPane
        (485→469) e o breadcrumb-helper saiu do ShellWorkspaceHost
        (529→525). Provado por tst_markdown_mode.qml (2 mutações reprovam).
        Limitação registrada da v1: imagem com caminho relativo dentro do
        .md não resolve (sem baseUrl); texto/título/lista/tabela/código sim.

TRILHA APÓS A F1b — ordem fixada pelo autor em 2026-07-18:

```text
1. MARKDOWN ("modo imagem" p/ .md)   próxima fatia; paga débito do EditorPane
                                     por extração (outline sidebar) antes de
                                     adicionar o preview — movimento + feature
2. F2  barra única                   JÁ APROVADA; funde AppMenuBar+TopHeaderBar
                                     (~46px), hamburger, controles embutidos;
                                     edita §4/§9/§10 da LAYOUT_SYSTEM no MESMO
                                     commit (8 regiões → 7)
3. F3  painel inferior 1 linha       spec e IntelliJ concordam; 26→34px (§6.3)
4. F4  âmbar/Salvar                  editor sagrado (§5.1), âmbar contido (§7.2)
5. VOLTA À FILA PRINCIPAL            fatia 2.2 (config+event do integration) —
                                     o desvio de layout NÃO cancela o roadmap
                                     28; L2+ segue esperando o v1 completo
```

APPIMAGE/ÍCONES — respondido ao autor em 2026-07-18: gerar um AppImage novo
JÁ SAI CORRIGIDO. O fix é no script (EXTRA_QT_PLUGINS=svg) e a validação
reprova o empacotamento se libqsvg.so faltar. O autor gera quando quiser;
a decisão "AppImage quando estabilizar" é dele e continua valendo.
F2  Barra única — APROVADA pelo autor (fusão AppMenuBar+TopHeaderBar ~46px,
    menu hamburger, controles de janela embutidos). EXIGE editar §4/§9/§10
    da LAYOUT_SYSTEM no mesmo commit (8 regiões → 7): decisão já tomada,
    falta executar. É a fatia que mais aproxima do print.
F3  Painel inferior: 2 linhas de abas → 1 (ferramenta escolhida no rail,
    como no IntelliJ); BottomTabBar 26→34px fecha a §6.3. Spec e IntelliJ
    concordam; zero decisão pendente.
F4  Limpeza do editor: remover o "Salvar" âmbar flutuante (§5.1, o editor é
    sagrado) e conter o âmbar à regra da §7.2 (Compilar deixa de ser bloco
    âmbar cheio).
Follow-up SEM decisão: cantos arredondados da JANELA (frameless/Wayland,
    exige teste no compositor real — não entra em fatia visual sem rede).
```

ARMADILHA DESTA ÁREA: não há teste de pixel no repo — a rede de fatia visual
é gate (qmllint/fiação/harness) + build release-hardened + O OLHO DO AUTOR.
Nunca declarar fatia visual entregue sem o gesto humano.

**DEPOIS DESSE DESVIO, A PRÓXIMA FATIA É A 2.2** (config + event do `integration`). É a metade de
ESCRITA do contrato v1 — a de leitura (descriptor + health) já está de pé. NÃO
comece por outra coisa sem decisão do autor; o roadmap 28 diz que L2+ só começa
com o v1 completo. Passos concretos da 2.2:

```text
1. protocolo: IntegrationConfig (id, chave->valor, escopo global|workspace,
   default reversivel) + IntegrationEvent (health mudou / config mudou).
   Padrao: crates/kinein-protocol/src/integration.rs, serde camelCase, testes.
2. dominio: crates/kinein-core/src/integration/config.rs (ler/gravar por escopo,
   reversivel). Persistencia: ver db::DraftStore (docs/seguranca/23) para o
   padrao de store local por-workspace; NAO inventar store nova.
3. handler: integration.get/set/reset no handlers/integration.rs (ja existe).
4. DECISAO QUE SO' APARECE AQUI: EditorConfig e' FFI (editorconfig-rs) ou parser
   proprio? A auditoria de 2026-07-16 derrubou "lib madura" (§0.2e). Config
   generico (chave-valor por escopo) NAO precisa disso; so' decida se/quando a
   vertical EditorConfig entrar. Comece pelo config generico.
5. teste E2E via handle_request + mutacao, como no integration.list.
Saida do v1 completo: descriptor+health+config+event de pe; so' entao L2.
```

**Antes de codar qualquer coisa, MEDIR** (regra zero acima) e ler o roadmap 28 §2
+ ARCHITECTURE §2.1 (o "plugin" e' descritor tipado, nunca host de extensoes).

**Para o autor testar (rodar os .sh):** ver a secao "COMO TESTAR" no fim da
trilha.

### CRONOGRAMA EM FASES — o mapa linear (aprovado pelo autor, 2026-07-17)

Ordem linear e cirurgica: nenhuma fase abre antes de a anterior FECHAR, e cada
uma tem um criterio de saida BINARIO (nao "esta bom?", mas "X acontece? sim/nao").
E' a defesa contra se perder nas ramificacoes conforme o projeto cresce. Marque
`[x]` ao concluir; o detalhe de cada item vive nas secoes E1-E6 e no roadmap 28.

```text
FASE 0 — Estabilizar o loop de dogfooding (destrava o autor HOJE)
  [x] 0.1  Icone: fonte unica ui/assets/app-icon.png; atalho via tema hicolor
           (nome, nao caminho); atualizar-tudo reinstala o atalho; gate
           verificar-icone. imagens/ passa a ser so' imagens. FEITO 2026-07-17.
  [x] 0.2  E1 — flake do `tools::`: retry em ETXTBSY (raiz) + lock sem poison
           (cascata). NAO era falta de O_CLOEXEC — a std ja o usa; a raiz e' a
           janela fork->exec. FEITO 2026-07-17, provado por mutacao + 5x verde.
  Saida: FASE 0 fechada — o loop de dogfooding esta estavel.

FASE 1 — Pagar os god-files que BLOQUEIAM feature (curto prazo, contínuo)
  Metodo fixo (molde: E6): teste primeiro -> corte por RESPONSABILIDADE (nao
  linhas, §4 regra 9) -> GUIAIA da area re-medido no mesmo commit. 1 por fatia.
  [~] 1.1  EditorController.qml     878->842  code actions extraidos (fatia 1.1).
           NAO resolvido: e' coordenador (fachada em cadeia), ~10 fatias para 400
           e' o pior custo/beneficio. ADIADO: quem tocar feature de editor paga o
           resto (regra "quem toca a area paga a dela"). Medicao no ContextoIA.
  [x] 1.2  core_client_dispatch.cpp 757->384  RESOLVIDO 2026-07-17: dividido por
           dominio (§5) em _language/_debug/_build. Movimento puro (byte-identico
           ao HEAD), saiu do baseline. Pendencia: QTest de dispatch e' fatia
           propria (o movimento nao mudou comportamento; compilador+linker + smoke
           foram a rede).
  [ ] 1.3  GitPanel.qml             633/300  bloqueia feature de Git
  [x] 1.4  editor_highlighter.cpp   818->486  RESOLVIDO 2026-07-17: languageForPath
           + rebuildRules (regras regex por linguagem) -> _rules.cpp; cores
           compartilhadas -> _palette.h. Movimento puro (byte-identico), saiu do
           baseline. Teste languageForPath migrou junto e segue provado por mutacao.
  Saida: os que bloqueiam feature saem do baseline; GUIAIA §5 re-medido.

  Padrao aprendido (1.2 e 1.4 vs 1.1): god-file organizado por dominio/camada
  (roteador, regras) RESOLVE numa fatia por movimento puro; coordenador de
  fachada em cadeia (EditorController) rende pouco por fatia. Ataque os primeiros;
  os segundos, so' quando tocar a feature da area.

FASE 2 — L1: dominio `integration` v1 (o gargalo do medio prazo)
  [~] 2.1  E2 — `integration.list` READ-ONLY entregue 2026-07-17 (decisao da IA,
           opcao 3, autor delegou). Contrato IntegrationDescriptor/Health/Info/
           ListResult no protocolo; dominio integration/ (registry+health) que
           REUSA tools.rs (invariante 1, provado por mutacao); handler fino
           integration.rs na cadeia; teste E2E via handle_request. Zero
           dependencia nova.
  [ ] 2.2  config + event: IntegrationConfig (escopo global/workspace,
           reversivel) e IntegrationEvent (health/config mudou). E' a parte de
           ESCRITA — decidir FFI editorconfig-rs vs parser proprio so' aqui.
  Saida (v1 completo): descriptor+health (feito) + config+event; a aba
           informativa le integration.list. So' entao L2+ comeca.

FASE 3 — L2-L4: C/C++/Rust SOLIDOS (a profundidade vertical)  [roadmap 28]
  [ ] 3.1  L2  diagnostico/teste/cobertura num contrato so + Jobs cancelaveis
               (Cppcheck, Clang Analyzer, cargo-audit/deny, Valgrind, gcov/lcov)
  [ ] 3.2  L3  project graph, targets/perfis explicaveis, cache provenance
  [ ] 3.3  L4  DAP solido + profiling (Heaptrack, perf/Hotspot)
  Saida: rodar teste + ver cobertura de um projeto C/C++/Rust DENTRO da Kinein,
         sem terminal. So' entao Docker (L5), banco (L5.5), embarcados (L6).
```

### COMO TESTAR (os .sh que o autor roda)

```text
scripts/atualizar-tudo.sh --ide
    O comando principal. Apaga o cache de build, reconstroi UI+core do ZERO,
    roda o GATE COMPLETO (fmt/test/clippy Rust, C++ estrito, qmllint, os 8
    gates, ctest, 16 harnesses), REINSTALA o atalho (binario + icone no tema
    hicolor, cache invalidado) e grava o manifesto. Ao terminar, confira
    build/kinein-build-manifest.env: git_head TEM que bater com o HEAD atual —
    manifesto com HEAD velho = nao passou (a barreira do pipe engana: rode SEM
    | tee, ou confie so' no manifesto).
    -> Depois, abra o atalho "Kinein Vectis (Desenvolvimento)" no menu do GNOME
       e dogfoode: autocomplete nao volta ao 1o item, auto-close nao engole
       caractere, e o ICONE novo aparece.

scripts/verificar.sh            gate completo SEM reconstruir do zero (mais rapido)
scripts/verificar.sh --rapido   sem os builds debug/release (so' lint+testes)

O integration.list (L1) e' core Rust SEM UI ainda — nao da' para "ver" na tela.
Ele e' coberto pelo gate (cargo test). A aba informativa que o le e' fatia futura.

NAO rodar scripts/atualizar-tudo.sh --appimage: o AppImage fica para quando a
versao Desenvolvimento estabilizar (decisao do autor). O atalho de
Desenvolvimento e' o alvo de teste diario.
```

### A trilha, em ordem de DESBLOQUEIO

O que não exige decisão vem primeiro. O que exige tem recomendação e um "se
ninguém responder, siga por X" — nenhuma sessão para esperando.

```text
E1  flake do `tools::`        RESOLVIDO     2026-07-17 (ETXTBSY + poison)
E2  L1: dominio `integration` DECISAO SUA   recomendacao pronta (§0.2e)
E3  debito god-file           SEM decisao   pre-requisito por area
E4  resto (protocolo, AppImage)             P3, sem bloqueio
```

**E1 — flake do `tools::` (§0.2h). RESOLVIDO em 2026-07-17.** A hipótese registrada
(`O_CLOEXEC`, "fechar o descritor antes do exec") estava **errada**, e medir
mostrou: a std do Rust já abre com `O_CLOEXEC`, e `fs::write` já fecha antes do
exec. A raiz eram **dois** defeitos: (A) o exec do script recém-escrito dava
`ETXTBSY` porque um fork de outro módulo herda o descritor de escrita na janela
entre `fork` e `exec` (o `O_CLOEXEC` só fecha no `exec`, não no `fork`); (B)
`EXEC_LOCK.lock().unwrap()` num `Mutex<()>` propagava *poison*, então uma falha
virava cascata. Correção: retry direcionado a `ETXTBSY` em `run_version_command`
(transitório por definição) + `unwrap_or_else(PoisonError::into_inner)`. Teste
`probe_espera_um_etxtbsy_transitorio` reproduz o `ETXTBSY` de forma determinística
(segura um fd de escrita 30 ms) e cai sem o retry. Registro completo no
`ContextoIA.md`.

**E2 — L1: o domínio `integration` v1. [DECISÃO SUA, com saída]**
VERIFICADO ABERTO: não existe `handlers/integration.rs`. O que trava é o §0.2e —
a auditoria derrubou a premissa (não há biblioteca EditorConfig Rust madura).
RECOMENDAÇÃO: opção 3 — validar o `integration` v1 pelo **inventário das
ferramentas já detectadas** (clangd, rust-analyzer, CMake, Cargo, Git, rg, fd,
lldb-dap, Clippy), zero dependência nova, contrato de pé; só então decidir FFI vs
parser próprio para o EditorConfig.
**Se ninguém responder: seguir pela opção 3 e registrar como decisão da IA.**

**E3 — débito god-file (§0.2g). Não é fatia única: é pré-requisito por área.**
Quem for tocar uma área, paga a dela antes. **A fonte VIVA é
`scripts/arquitetura-baseline.txt`** (números de CÓDIGO, sem comentário/branco
desde 2026-07-17); a lista abaixo é um retrato dos maiores em 2026-07-17, não a
verdade permanente — meça o baseline ao retomar.
```text
RESOLVIDOS nesta sessao (movimento puro por dominio/camada):
  core_client_dispatch.cpp  757 -> 384   dividido por dominio (fatia 1.2)
  editor_highlighter.cpp    818 -> 486   regras/paleta separadas (fatia 1.4)

MAIORES RESTANTES (codigo/limite), retrato de 2026-07-17:
  EditorController.qml     842 (400)  coordenador de fachada: ADIADO (rende
                                      ~40 linhas/fatia; pague ao tocar editor)
  commands.rs              664 (500)  descriptors de command.list
  terminal.rs              657 (500)  bloqueia feature de terminal
  GitPanel.qml             633 (300)  VISUAL puro: ADIADO (componentizacao sem
                                      teste de pixel; pague ao tocar Git)
  handlers/lsp.rs          609 (500)
  lsp/manager.rs           588 (500)  bloqueia feature de LSP
  dap/session.rs           563 (500)  bloqueia feature de debug
  core_client_requests.cpp 551 (500)  irmao do dispatch: dividir por dominio
```
PADRAO APRENDIDO (fatias 1.2/1.4 vs 1.1): god-file organizado por DOMINIO/CAMADA
(roteador, regras) resolve numa fatia por MOVIMENTO PURO (prove byte-a-byte).
Coordenador de fachada (EditorController) ou visual (GitPanel) rende pouco/nao
tem rede — ADIE, pague ao tocar a feature da area. `core_client_requests.cpp`
e' o proximo alvo LIMPO (irmao do dispatch, mesma tecnica).

Quando a catraca disparar, há **três suspeitos nesta ordem: a sua mudança, a
categoria, o arquivo** (§4 regra 9). Não corte linha para caber e não suba o
baseline — os dois são trapaça. Contar linha de Rust exige cortar no
`#[cfg(test)]` (§4 regra 10), senão dá falso alarme.

**E4 — resto conhecido, P3, sem bloqueio.** `assistant_terminal_width` no
protocolo (detalhe no §0.2j); AppImage só depois de L1 e dos ícones (§0.2d-5).

### Depois do E2: para onde o projeto vai (decidido em 2026-07-17)

> **SUPERADA pelo CRONOGRAMA EM FASES no topo deste arquivo (2026-07-17 fim do
> dia).** Esta seção é o registro detalhado de manhã/tarde; os números de débito
> aqui (23 arquivos, contagem de TEXTO) são de 07-16/07-17 e NÃO valem mais —
> hoje são 15 arquivos, contagem de código, e dispatch + highlighter foram
> RESOLVIDOS. Fonte viva do débito: `scripts/arquitetura-baseline.txt`. Leia o
> cronograma no topo; esta seção fica pelo raciocínio, não pelos números.

Estruturado em **`docs/roadmaps/28-plataforma-de-plugins-e-verticais.md`**. Resumo,
para não haver dúvida ao retomar:

```text
E2 (L1)  a PLATAFORMA. Enquanto ela nao existir, plugin nenhum tem onde nascer
         e cada ferramenta nova vira mais um handler ad-hoc.
   |
L2-L4    C/C++/RUST SOLIDOS primeiro — diagnostico/teste/cobertura num contrato
         so, Project Graph, DAP e profiling. E' profundidade no que ja existe.
         Docker e banco sao superficie nova: IDE com Docker e sem cobertura de
         teste e' demo.
   |
L5       RemoteContext. **DOCKER entra aqui** — container e' contexto remoto
         (`container://` ao lado de `ssh://`), nao nivel proprio.
L5.5     BANCO DE DADOS. conexao -> schema browser -> query console -> result grid.
L6       EMBARCADOS.
```

**Decisão do autor: Docker e banco são NATIVOS, de primeira classe** — domínios do
core como `git`/`lsp`/`terminal`, não plugins de terceiro. Implementar do zero, e
o autor aceitou que é demorado. O `integration` v1 dá o descriptor/health/config
(como aparecem para o usuário); a funcionalidade é Rust nosso.

**FATO QUE MUDA A REFERÊNCIA, verificado em 2026-07-17:** o **IntelliJ IDEA
Community NÃO tem Database Tools** — é Ultimate. Não há UI de banco lá para
estudar. O idioma visual do Community continua sendo a referência; a referência
*funcional* de cliente de banco é o DBeaver (a auditar).

### UI/UX — dívida contínua e explícita, atravessa todos os níveis

Registro do autor (2026-07-17): **a UI/UX da IDE tem muito a ser otimizado e
polido**, tendo o **IntelliJ IDEA Community** como referência — *"essa UI/UX da
JetBrains é muito agradável/confortável para o desenvolvimento"*.

E a regra, que já é contrato (`ARCHITECTURE.md` §2.1): **não é copiar e colar.**
Importa-se invariante, decisão, modo de falha e estratégia de teste; nunca código,
runtime, IntelliJ Platform/Swing ou modelo interno. O que se vê lá é
**redesenhado** no fluxo nativo Qt/QML, com o Theme e a iconografia da Kinein. Onde
o contexto diverge (C/C++/Rust, offline-first, sem host de extensões, frameless com
chrome próprio), **vence o contexto da Kinein** — não a fidelidade ao IntelliJ.
Adaptação realista e pragmática. Detalhe em `docs/roadmaps/28` §7.

### Armadilhas que já custaram horas — leia antes de validar

```text
O atalho "Kinein Vectis (Desenvolvimento)" roda
build/linux-clang-release-hardened/, NAO o dev-local.
=> depois de mexer na UI:  cmake --build --preset release-hardened
   ANTES de pedir validacao. Em 2026-07-17 o autor passou horas com uma IDE
   quebrada porque esse binario estava 4 commits atras.

qmllint "limpo" + boot ate o primeiro frame NAO provam fiacao QML.
=> binding auto-referente `x: x` entrega null em silencio.
   scripts/verificar-qml-fiacao.sh pega; rode o gesto real assim mesmo.
```

## 0. Dogfooding ativo

O gatilho **“estou no Kinein”** já foi recebido. A primeira regressão concreta
é o Assistente não se comportar visualmente como um terminal profissional:
faltavam scrollback/barra perceptível no Codex, largura adequada e fluidez no
resize. Após o primeiro reinício, surgiu um segundo detalhe: a linha de
digitação inline ficava sem limite visual entre o aviso de usage e o status do
modelo. No segundo reinício foram reportados quatro sintomas adicionais: a
faixa não envolvia os glifos, o divisor livre desaparecia durante a sessão, a
árvore `Project` era fechada e o scroll se perdia em chats longos. A correção
0.52 tratou esses quatro sem criar input ou terminal paralelo. Após o aceite do
dimensionamento, o feedback restante ficou restrito à faixa da entrada
multilinha e à roda do mouse; a correção pós-0.52 de 2026-07-15 aguarda o gesto
real apenas nesses dois pontos, conforme `REENTRADA-KV`. O usuário pausou esse
gesto e autorizou expressamente a continuação do roadmap. A1 e A2 foram
entregues nos protocolos 0.53.0 e 0.55.0. A tentativa posterior de demarcar a
entrada foi removida: grade VT, spans e cursor voltaram a ser a única fonte
visual, no modelo terminal-first. Antes do início efetivo de A3, o dogfooding
revelou que o caret ficava muito depois do texto e que verde/bold do prompt era
visualmente agressivo. A correção 0.56 unifica spans e cursor pela largura VT,
usa família monoespaçada real e suaviza peso/paleta sem interpretar o prompt;
o usuário aprovou posição horizontal, Terminal comum e verdes suaves. Restou
somente um ajuste fino: no Assistente o caret de Claude/Codex não parecia
centralizado. O teste exclusivo com `-2` ficou perto, mas o usuário pediu valor
`0`, igual ao Terminal puro aprovado. A pesquisa seguinte mostrou que as TUIs
usam o cursor nativo do terminal e podem solicitar forma/piscagem por DECSCUSR,
estado que o render anterior descartava. A correção 0.57 preserva esse estado
genericamente. O usuário esclareceu em seguida que Assistente é somente outra
apresentação da mesma base de terminal, não um backend diferente; por isso todo
offset por superfície foi removido e `DefaultUserShape` também usa a célula VT
integral. O usuário abriu a Kinein por `scripts/kinein-vectis` e não percebeu
mudança relevante: o caret da TUI ainda parece desalinhado. Portanto 0.57 fecha
a lacuna de forma/piscagem, mas não a causa visual. Esse polimento foi adiado
sem novo offset; sua retomada começa por reprodução instrumentada e métricas de
célula/DPR em `docs/roadmaps/26-terminal-rendering-parity-roadmap.md`. Por decisão do
usuário, a experiência funcional do terminal do Code OSS é a base de paridade,
traduzida para Rust + IPC + Qt, sem Electron, Node, WebView ou xterm.js.

1. registrar cada problema observado pelo usuário ou por um testador com ação,
   esperado, resultado, reprodução, distro e log quando houver;
2. corrigir primeiro perda de dados, crash, corrupção, falha de abertura/build
   ou bloqueio que force a saída para outra IDE;
3. depois tratar regressões funcionais e atritos reproduzíveis de uso diário;
4. quando não houver feedback bloqueador e o usuário mandar prosseguir no
   roadmap, seguir a fila da seção 2; A1, A2 e **A3 foram entregues**;
5. continuar pelas etapas deste arquivo e pelos docs existentes, sem inventar
   outro remake, subsistema paralelo ou roadmap substituto.

O dogfooding não autoriza push, publicação, mudança de visibilidade, envio
externo ou implementação aleatória fora da fila. Commits locais são feitos
somente nos checkpoints verdes já autorizados. O repositório-fonte continua
privado.

### 0.1 Protocolo econômico de reentrada após reiniciar a própria Kinein

Reiniciar a IDE encerra a sessão de IA que está rodando dentro dela. Para não
gastar outra conversa reconstruindo contexto, o handoff deve ficar neste
arquivo antes de fechar. Na sessão nova, o usuário precisa escrever somente:

```text
Leia AGENTS.md e retome pelo marcador REENTRADA-KV em PONTO_ATUAL.md. Siga as
leituras obrigatórias silenciosamente, não resuma o histórico e não refaça o
que já está validado.
```

A sessão nova deve executar esta sequência:

1. ler `AGENTS.md` e a documentação obrigatória indicada nele, sem devolver
   uma recapitulação extensa ao usuário;
2. localizar primeiro o marcador `REENTRADA-KV` abaixo e confrontá-lo com
   `ContextoIA.md` + código apenas onde houver divergência;
3. rodar `git status --short` para preservar o worktree existente; arquivos
   presentes não autorizam limpeza/descarte. Commit local só depois do gate
   verde do marco; push continua sem autorização;
4. não repetir investigação, implementação, gate ou build já registrados como
   verdes, a menos que o código tenha mudado depois do marcador ou apareça
   evidência concreta de regressão;
5. retomar diretamente pela ação `PRÓXIMO GESTO`; responder inicialmente com
   uma frase curta de orientação, não com outro plano ou resumo;
6. depois do gesto real, registrar o resultado no marcador: se falhar,
   ação/esperado/observado/ambiente viram a prioridade do dogfooding; se passar,
   marcar o aceite e seguir a próxima fatia desta fila somente quando o usuário
   mandar prosseguir.

#### REENTRADA-KV — estado exato para a próxima reentrada

```text
ESTADO
- Protocolo atual 0.61.0. O `aiBridge` NAO existe: nao ha politica por
  programa no core, e uma CLI de IA e um programa como outro qualquer.
  `terminal.mouse` (0.60.0) decide o gesto no core; `format.capabilities`
  (0.61.0) publica o catalogo de formatters e a UI nao mantem lista.
- L0 fechado (A3.1-A3.4 em `docs/roadmaps/21`), com UMA excecao declarada:
  falta o harness Qt de digitacao tecla->frame. A rota do harness QML esta
  fechada com evidencia (qmldir aponta para qrc:); nao retentar.
- Cursor/TUI: RESOLVIDO e ACEITO. Causa era o `Column` do Qt Quick descartar
  linha vazia (largura zero); o texto subia e o cursor ficava certo. Ver
  `docs/roadmaps/26` §4.7.
- Codex abre com argumento fixo --no-alt-screen; Claude permanece sem argumento.
- KV ativo reutiliza TerminalPanel/TerminalManager, tem barra persistente,
  roda/arrasto, teclado/paste VT, largura livre persistida 300–720px e
  ampliar/restaurar; Project permanece independente fora da maximização.
- O bridge preserva o transcript contra CSI 3 J ainda emitido por versões do
  Codex; o Terminal comum continua honrando clear.
- Não existe guia, faixa ou input paralelo: grade VT, spans ANSI e cursor são
  a única representação da entrada, seguindo comportamento terminal-first.
- Cada span informa a largura autoritativa em células VT; fonte, resize,
  seleção e caret usam a mesma grade. ANSI bold usa peso médio e a paleta verde
  é suave; o shell continua dono do texto e dos atributos do prompt.
- Não existe mais propriedade de offset vertical no AssistantPanel,
  TerminalPanel ou TerminalViewport. Forma e piscagem solicitadas por DECSCUSR
  seguem no render; reset/default é resolvido no core para `bar`. Barra/bloco
  usam a célula VT integral, underline usa a base e `steady` não pisca. Não há
  regra por agente ou superfície.
- O teste humano pelo script de desenvolvimento não percebeu correção do
  alinhamento vertical. O problema continua aberto e foi adiado; 0.57 não tem
  aceite visual. A retomada está integralmente especificada em `docs/roadmaps/26` e
  começa por fixture PTY, overlay de baseline/célula e DPR, não por offset.
- Code OSS/xterm.js são a base de paridade comportamental autorizada para o
  terminal. A implementação continua nativa em `portable-pty` + Rust + IPC +
  Qt; Electron, Node, WebView, Extension Host e runtime xterm.js não entram.
- A roda aceita os dois formatos do Qt (`angleDelta` e `pixelDelta`) e segue o
  mesmo `terminal.scroll` do Terminal comum.
- Gutter usa faixas independentes para folding/breakpoint, diagnóstico, blame,
  diff e números medidos por FontMetrics; marcador não invade o número.
- Semantic tokens carregam path+version e respostas obsoletas são descartadas;
  Tree-sitter permanece fallback estrutural instantâneo.
- Os cinco SVGs de árvore fornecidos foram integrados sem alterar seus bytes.
- Scripts shell reconhecidos têm ação de execução na árvore; o core confina o
  caminho e usa argv explícito, sem interpolação.
- Packaging corrigido para cache host/container isolado, mounts Podman/SELinux
  e invocação por bash. A entrada direta delega ao builder Debian auditado; o
  worker não é um build nativo. `dist/` só recebe por staging o conjunto
  completo AppImage/checksum/instalador/Tutorial, preservando a entrega anterior
  em caso de falha.
- Barra da janela agora é client-side: `Main.qml` usa `FramelessWindowHint` e a
  App Bar hospeda Minimizar, alternável Maximizar/Restaurar e Fechar
  (`WindowControls.qml`), guiados pelo `QWindow` via `WindowChromeController`
  (`ui/src/window_chrome_controller.*`). Região livre arrasta/duplo-clica;
  `WindowResizeHandles.qml` cobre as oito bordas. ACEITO pelo usuário em
  Fedora/Wayland (2026-07-15); regressão P2 encerrada. Ver `docs/roadmaps/20` §barra.

VALIDAÇÃO JÁ FEITA — NÃO REPETIR SEM MUDANÇA DE CÓDIGO
- `scripts/verificar.sh` completo: verde; binários release do atalho de
  desenvolvimento atualizados.
- Testes Rust, clippy -D warnings, C++/QML estritos e harnesses QML: verdes.
- Build Clang debug strict e `scripts/verificar-cpp.sh`: verdes.
- `scripts/verificar-qml.sh`: verde usando o response file do build strict
  atualizado; o antigo `build/dev-local` não tem precedência.
- tst_assistant_layout cobre divisor ativo/largura/Project; tst_terminal_scroll
  cobre nova saída, snap, troca de sessão, roda tradicional e `pixelDelta`;
  Rust cobre CSI 3 J entre chunks.
- AppImage final (33.737.208 bytes; SHA256 `86b335b2b1ba8c81d958df4f1e45f7d9c0838fdad2a2567c84199f84b8dbdc0d`),
  teste host e Debian mínimo sem rede: verdes. Ambos validam também instalador
  executado fora da pasta, `.desktop`, PNG e `Tutorial.md` idêntico à fonte.
- Correção 0.56: gate integral verde com 335 testes Rust, Clippy, C++/QML
  estritos, 12 harnesses, builds Debug/Release e smoke offscreen de 8 s
  (`exit 124` esperado). Binários do atalho de desenvolvimento atualizados.
- O teste anterior `cursorVerticalOffset: -2` exclusivo do Assistente passou
  no gate, mas não recebeu aceite visual. O novo valor `0` passou em qmllint,
  12 harnesses, rebuild Release e smoke offscreen de 8 s; o gesto humano
  posterior não aprovou o cursor e `dist/` não foi regenerado.
- Primeiro recorte DECSCUSR 0.57: gate integral verde com 337 testes Rust,
  Clippy, C++/QML, 12 harnesses, builds e smoke. O feedback posterior removeu
  todo offset e unificou `DefaultUserShape`; o mesmo gate integral passou
  novamente e o smoke release ficou vivo por 8 s sem saída (`exit 124`).
  Binários de desenvolvimento atualizados; `dist/` continua intocado.
- Gesto humano posterior: aberta por `scripts/kinein-vectis`, a versão não
  apresentou mudança visual relevante para o usuário. O caret de Claude/Codex
  continua sem aceite. Automação verde não equivale a correção visual.
- A auditoria seguinte encontrou `target/release/kinein-core` anterior a
  `terminal.rs`; o launcher podia combinar UI nova com core antigo. O gate foi
  refeito com `debug-strict`/`release-hardened`: 337 testes Rust, Clippy,
  C++/QML, 12 harnesses e os builds passaram. Smoke real pelo
  `scripts/kinein-vectis` ficou vivo por 8 s sem saída (`exit 124`). UI/core
  release agora são os binários atuais; hashes e caminhos estão em
  `ContextoIA.md`. `dist/` permaneceu intocado.
- A baseline release A3 com `N=3` passou os orçamentos: 250 ms primeiro frame,
  103 MB UI, 3,4 ms workspace, 0,0 ms leitura 10k e 7 MB core. A expansão
  A3.1–A3.4 está detalhada abaixo; Code OSS/Zed e resultados estão em docs/roadmaps/21.
- Controles de janela (barra client-side): `scripts/verificar.sh` integral verde
  + smoke offscreen debug/release (`exit 124`); ACEITO pelo usuário em
  Fedora/Wayland. AppImage 0.1.0 regenerado e testado com este código.

PRÓXIMO GESTO — ordem explicita, montada em 2026-07-17

A ordem abaixo e' por DESBLOQUEIO, nao por tamanho. O que nao exige decisao vem
primeiro; o que exige esta marcado com [DECISAO SUA] e tem recomendacao pronta,
para a proxima sessao nunca parar esperando.

E0. FEITO — cursor/TUI APROVADO pelo autor em 2026-07-17. Nao reabrir.
    A causa-raiz era a linha vazia colapsando no positioner do Qt: o TEXTO
    escorregava para cima e o cursor, posicionado por `yForRow`, nunca esteve
    errado (roadmap 26 §4.7; corrigido em `ab3becc`, 07-16 16:53).
    O QUE ISSO APAGOU DO ROADMAP, e e' o ganho maior:
    - **R3 DECIDIDO por evidencia: opcao 1, fica o QML.** O criterio dele era
      "manter se R2 corrigir o visual e cumprir orcamento" — corrigiu e cumpre.
      Nao trocar de renderer; reabrir exige medicao nova em ADR.
    - **R2 vira CONDICIONAL.** A razao que o criou (o sintoma) acabou; o R1 ja
      media que em DPR 1 ele e' nulo (~0,11px). So executar se uma MEDICAO em
      DPR != 1 mostrar divergencia — nunca por impressao visual, que foi o que
      custou a fatia R1 inteira.

E1. Desentupir o gate: flake do `tools::` (§0.2h). SEM DECISAO, alto retorno.
    VERIFICADO EM ABERTO (2026-07-17): `crates/kinein-core/src/tools.rs` nao muda
    desde `4dacc1b` (07-16 22:53) e nao tem `O_CLOEXEC` nem fechamento de
    descritor; o `EXEC_LOCK` esta intacto e o comentario dele descreve o bug que
    continua la. Passar 3x seguidas nao prova nada: o §0.2h mediu "1 em 3", e na
    suite COMPLETA (`--workspace --all-features`), nao no `-p kinein-core --lib`.
    POR QUE: reprova ao acaso e ensina a reexecutar ate passar — a doenca que a
    catraca do §0.2g existe para impedir. Bateu 2x na sessao de 2026-07-17 e a
    IA se flagrou fazendo exatamente isso.
    CAUSA PROVAVEL, ja escrita no proprio arquivo (`crates/kinein-core/src/tools.rs:507`):
    o `EXEC_LOCK` serializa so os testes de `tools` entre si; outro teste da suite
    forkando no momento errado reproduz `ETXTBSY`, e a 2a falha e' cascata (mutex
    envenenado).
    CAMINHO: fechar o descritor de escrita ANTES do exec (ou `O_CLOEXEC`).
    NAO aumentar o escopo do lock — isso esconde, nao corrige.

E2. FEITO desde 2026-07-16 — A3.3 item 1, harness tecla->frame.
    `ui/src/typing_perf_harness.cpp`, atras de `KINEIN_PERF_TYPING`; mediana
    7,4 ms / p95 8,4 ms contra orcamento de 16/20 ms. **L0 esta fechado.**
    ERRO DE REGISTRO, corrigido em 2026-07-17: o `PRÓXIMO GESTO` listava isto
    como "design pronto, o autor retoma" enquanto a §A3 do MESMO arquivo dizia
    "A3.1–A3.4 estao fechadas... entregue em 2026-07-16". O documento se
    contradizia e a IA copiou a metade errada sem medir. Ao montar fila, MEDIR:
    `ls` no arquivo, `git log` na area, grep no gate — nunca herdar o item.

E3. [DECISAO SUA] Primeiro recorte do L1 (§0.2e).
    VERIFICADO EM ABERTO (2026-07-17): **nao existe `handlers/integration.rs`**.
    Os handlers sao build, cargo, cmake, debug, draft, format, fs, git, jobs,
    lsp, runconfig, run, settings, syntax, terminal, workspace. A entrega
    arquitetural do L1 e' o dominio `integration` v1 (ver a tabela L0–L10); o
    painel Ferramentas existente e' `tools.detect`, que NAO e' o `integration`.
    O QUE TRAVA: a auditoria derrubou a premissa do plano — nao existe biblioteca
    EditorConfig Rust madura. As 3 saidas estao no §0.2e com o trade-off medido.
    RECOMENDACAO (para nao travar): opcao 3 — validar `integration` v1 pelo
    INVENTARIO das ferramentas ja detectadas (clangd, rust-analyzer, CMake,
    Cargo, Git, rg, fd, lldb-dap, Clippy). Zero dependencia nova, contrato de pe,
    e so entao decidir FFI (`editorconfig-rs`) vs parser proprio.
    SE VOCE NAO RESPONDER: seguir pela opcao 3 e registrar como decisao da IA.

E4. Debito god-file (§0.2g) — 23 arquivos, MEDIDO em 2026-07-17. NAO e' fatia
    unica: e' pre-requisito de quem for tocar cada area, e a catraca ja cobrou 4x.

    Ja PAGO desde a medicao de 07-16 (5 de 10 da lista original):
    ```text
    Main.qml                 700 -> 270   AppDomains (0686213)
    BottomPanelHost.qml      541 -> 383   TerminalSessionTabs (696aa23)
    RuntimeController.qml    494 -> 309   RunConfig + remocao do Assistente
    AppMenuBar.qml           303 -> 292   icone saiu de dentro da IDE
    ShellWorkspaceHost.qml   582 -> 576   parcial; ainda 1.4x o limite
    ```
    Em ABERTO, e cada um bloqueia a sua area (limite entre parenteses):
    ```text
    EditorController.qml     1070 (400)  bloqueia QUALQUER feature de editor
    terminal.rs               955 (500)  bloqueia feature de terminal
    editor_highlighter.cpp    910 (500)  bloqueia realce
    core_client_dispatch.cpp  804 (500)  §5 ja manda dividir por dominio
    GitPanel.qml              764 (300)  bloqueia feature de Git
    lsp/manager.rs            732 (500)  bloqueia feature de LSP
    commands.rs               696 (500)
    dap/session.rs            672 (500)  bloqueia feature de debug
    core_client_requests.cpp  660 (500)
    ShellWorkspaceHost.qml    576 (400)  composition host; corte por area
    ```
    ARMADILHA DE MEDICAO: a catraca conta linhas **fora dos testes** (corte no
    `#[cfg(test)]`, §4 regra 10). Contar `wc -l` cru em arquivo Rust da numero
    inflado e falso alarme — aconteceu em 2026-07-17.
    REGRA (§4 regra 9): quando a catraca disparar, ha TRES suspeitos nesta ordem —
    a sua mudanca, a categoria, o arquivo. Nao corte linha para caber e nao suba
    o baseline: os dois sao trapaca.

E5. Resto conhecido, sem pressa e sem bloqueio:
    - **`assistant_terminal_width` — o que e':** um campo de CONFIGURACAO no
      contrato. `kinein-protocol/src/settings.rs:62` (`Option<u32>`) e `:91`
      (`u32`), serializado como `assistantTerminalWidth` no JSON-RPC. Guardava a
      LARGURA do painel Assistente lateral — aquele que o 0.59.0 removeu. Hoje:
      **a UI nao le** (a propriedade morta saiu do `SettingsController` em
      2026-07-17) e o core so o mescla/valida por inercia — 9 pontos em Rust,
      incluindo `handlers/settings.rs:113`, que valida a largura de um painel que
      nao existe. E' peso morto no contrato.
      **Por que nao saiu junto:** tirar mexe no `kinein-protocol` = mudanca de
      contrato, com decisao de VERSAO e atualizacao do `docs/arquitetura/03`.
      Nao se enfia isso numa remocao de UI. Fatia propria, P3.
    - AppImage para testadores: o plano do autor (§0.2d-5) manda fechar A3, L1 e
      os icones antes. O AppImage atual (07-15 23:08) e' ANTERIOR ao remake do
      icone, por isso o atalho do AppImage mostra o icone velho — o do
      Desenvolvimento ja mostra o novo.

ARMADILHA MEDIDA NESTA SESSAO, leia antes de validar qualquer coisa:
o atalho "Kinein Vectis (Desenvolvimento)" roda
`build/linux-clang-release-hardened/`, NAO o `dev-local`. Em 2026-07-17 o autor
passou horas com uma IDE quebrada porque esse binario era de 07-16 23:18 e nao
tinha 4 commits. **Depois de qualquer mudanca de UI, rodar
`cmake --build --preset release-hardened` antes de pedir validacao.**

RESULTADO PENDENTE
- Cursor/TUI: RESOLVIDO e aprovado pelo autor em 2026-07-17 (ver E0). O que
  resta do roadmap 26 e' R4–R7 (paridade VT, interacoes, desempenho, a11y);
  R2 ficou condicional e R3 foi decidido.
- R4–R7 detalhados em `docs/roadmaps/26-terminal-rendering-parity-roadmap.md`.
- O AppImage 0.1.0 de 2026-07-15 foi regenerado a pedido do usuário para
  embarcar a barra client-side aceita; ele carrega o estado atual do cursor/TUI,
  que o usuário optou por não deixar bloquear a entrega. Não regerar o AppImage
  *por causa do cursor* antes do aceite visual dele.
- Se qualquer gesto falhar, registrar ação/esperado/observado/ambiente e
  priorizar a regressão antes de A3.

LIMITES
- Commit local somente após checkpoint verde; não fazer push/publicação.
- Não reabrir a discussão de chat embutido: Assistente é terminal dedicado.
- Fidelidade ao Code OSS significa comportamento adaptado à arquitetura da
  Kinein; não incorporar Electron/Node/xterm.js nem copiar implementação.
```

### Loop de desenvolvimento a partir do dogfooding

```text
feedback real (autor ou testador)
        ↓
reprodução e causa-raiz
        ↓
teste/harness de regressão quando aplicável
        ↓
correção pequena na camada dona
        ↓
gate + gesto real
        ↓
docs sincronizadas e retorno ao uso
```

Se vários feedbacks chegarem juntos, usar esta prioridade:

```text
P0  perda/corrupção de dados, segurança, crash ou IDE não abre
P1  bloqueio de edição, build, run, debug, terminal, Git ou navegação
P2  comportamento incorreto/repetível que prejudica o fluxo diário
P3  conforto, polimento ou funcionalidade nova
```

Feedback de testador não vira feature automaticamente: reproduzir, conferir se
já existe solução no core/UI e encaixar no domínio/roadmap correto. Se for uma
ideia nova sem bloqueio, registrar atrás dos problemas reais e de A3.

### 0.2 Controles de janela integrados (P2 — ACEITO em 2026-07-15)

O dogfooding do AppImage em Fedora/Wayland havia revelado que **Minimizar**,
**Maximizar** e **Restaurar** não estavam visíveis. A pedido explícito do
usuário, os controles foram integrados à barra da própria IDE (estilo JetBrains,
sem copiar): `Main.qml` usa `Qt.Window | Qt.FramelessWindowHint`,
`WindowControls.qml` traz Minimizar + alternável Maximizar/Restaurar + Fechar
guiados pelo estado real do `QWindow` via `WindowChromeController`, a região
livre arrasta/duplo-clica e `WindowResizeHandles.qml` cobre as oito bordas.
Detalhes e referência profissional (IntelliJ IDEA Community `e3b4dba`) em
`docs/roadmaps/20` e `ContextoIA.md`.

**Aceito:** o usuário testou em Fedora/Wayland e confirmou que funciona. A
regressão P2 está encerrada. Um AppImage 0.1.0 novo foi gerado com este código e
entregue em `dist/`. O polimento P3 restante de `docs/roadmaps/20` (snap, escala
fracionária, multimonitor e acabamento visual) segue como fatia própria, não
bloqueante; a imagem de referência é `imagens/bugs/ReformularBarra.png`.

### 0.2b aiBridge REMOVIDO e Assistente desabilitado (2026-07-16, protocolo 0.59.0)

**O `aiBridge.*` era a interferência e foi removido.** Ele abria `claude`/`codex`
por um caminho especial: argumentos injetados pelo core (`--no-alt-screen`,
`--ax-screen-reader`) e um filtro que engolia `CSI 3 J` da própria aplicação.
Isso é a IDE se metendo entre o programa e o terminal — o que nenhuma IDE
profissional faz, e o que fazia o agente se comportar diferente dentro e fora
da Kinein.

Uma CLI de IA agora é **um programa como outro qualquer**: abrir o terminal e
rodar `claude`. Mesmo PTY, mesmo emulador, mesmo contrato de um `ls`.

Removidos: `handlers/ai.rs`, `protocol/ai.rs`, `tests/ai.rs`, a superfície
`ui/qml/assistant/*`, o `AiBridgeEventRouter`, o atalho do rail, o estado do
shell, as settings `aiCliProfile`/`aiCliFlatTranscript`, e — no `terminal.rs` —
o `ScrollbackPreserver` e o `open_command_with_policy`. Não existe mais política
por programa no terminal.

**Assistente volta como UI pura:** um atalho visual que abre uma sessão de
terminal comum, para desacoplar visualmente do uso padrão, **sem regra de
negócio no core**. A UI representa o backend; não o define. Depende do terminal
consolidado (ADR-0004 já trocou o emulador para o `alacritty_terminal`, motor do
Zed; falta encaminhamento de mouse ao app e a decisão de renderer).

O modelo de IA não mudou: externa, por CLI do usuário, sem chat embutido e sem
rede pela IDE. Caiu o **mecanismo**, não o princípio.

### 0.2c Assistente reativado como UI pura (2026-07-16)

A dependência registrada em §0.2b ("depende do terminal consolidado") caiu: a
roda ao aplicativo está feita (protocolo 0.60.0) e a grade foi corrigida e
aceita. O atalho voltou, e agora ele é o que sempre deveria ter sido.

`Exibir → Assistente` (`view.context`) abre uma sessão de terminal **comum**,
rotulada `Assistente N`, para a sessão do agente não se perder entre os terminais
de build. Se já existe uma viva, foca ela em vez de acumular aba.

**Não existe regra de negócio em camada nenhuma.** O core não sabe o que é "KV
Context": para ele é o mesmo `terminal.open` do Alt+F12, um `$SHELL` no PTY. O
rótulo é estado de UI (`pendingContext`/`contextSeq` no `RuntimeController`) e
não é parâmetro do protocolo — se um dia virar, a política por programa que o
0.59.0 removeu voltou. Quem roda `claude`/`codex` é o usuário, digitando.

Detalhe de robustez: `terminal.open` pode falhar (teto de 12 sessões) e o erro
vai para o handler genérico do `CoreClient`, sem chegar ao QML. Sem tratamento a
marca ficaria presa e a próxima aba comum nasceria rotulada "Assistente"; um
timeout de 4 s a solta.

O atalho tem **duas entradas**: `Exibir → Assistente` e o ícone dedicado no
`SideRail` (o `context` do `KvIcon` sobreviveu ao 0.59.0; só o botão tinha
sido arrancado junto com o painel do assistente). O ícone acende conforme a
aba ATIVA do terminal ser de contexto — não há painel próprio para alternar.

Cobertura: `tst_multi_terminal.qml` (rótulo, marca consumida, aba comum não
herda, estado aceso segue a aba ativa, foco em vez de acumular, numeração não
repete, inerte sem workspace).

Fio solto que isso fechou: o item de menu "Assistente" existia desde o 0.59.0
apontando para uma ação `view.context` que **não existia** — opção morta na barra.

Pendência aberta: `assistantTerminalWidth` sobreviveu à remoção do painel do
assistente em três camadas (`SettingsController.qml`, `settings.rs`, schema). É
setting órfã. Decidir: ou o Assistente passa a usar largura persistida, ou sai.

**Opção B (a UI digitar o comando do agente) segue em aberto** e é preocupação
válida do autor: hoje o atalho abre a aba e o usuário digita `claude`. Subir para
a B exige que o comando seja **configurável**, senão é a regra por programa
apenas migrando de camada — o core deixaria de conhecer "claude" e a UI passaria
a conhecer. Analisar em fatia própria.

### 0.2d Backlog levantado pelo autor em 2026-07-16 (pontuado, não implementado)

**1. Assistente: confirmar em tela.** O autor reportou que o item aparece em
`Exibir` e não funciona. Essa é exatamente a descrição do estado **anterior** aos
commits de hoje: até `034b773` o item apontava para uma ação `view.context`
inexistente. Depois de `034b773` (menu) e `19eef85` (ícone no rail) o wiring foi
verificado — `runtimeController` chega ao `ShellHeaderHost` e ao
`ShellWorkspaceHost`, e ambos chamam `openContext()`. **Falta o gesto humano no
build novo.** Se continuar morto, é regressão real e tem prioridade: registrar
ação/esperado/observado/ambiente.

**2. Renomear "KV Context" — FEITO em 2026-07-17 (`37d4bbc`).** Nome definido
pelo autor: **"Assistente"**. O rename foi ate os internos e os docs vivos —
`view.assistant`, `openAssistant()`, `assistantKind`, `AssistantController`/
`AssistantSelector`, icone `assistant` — porque o §0.2d exigia decidir de uma vez
"para nao ficar meio renomeado". NAO renomeados de proposito: `ContextoIA.md`,
`docs/diario/` e o ADR-0004, que sao LOG do que foi decidido quando o nome era
outro. Ressalva viva: o `ui/qml/assistant/` removido no 0.59.0 tinha
`AssistantPanel`/`AssistantController` — nome parecido, coisa diferente (aquele
era terminal PARALELO); o aviso esta no cabecalho do `AssistantSelector`.

**2b. Aba propria do Assistente — FEITO em 2026-07-17.** Pedido do autor: o
Assistente precisa de separacao VISUAL, nao ficar no meio de "Terminal 1, 2, 3".
Decisao do autor apos medicao: **aba no painel inferior**, e nao o painel direito
da spec §4.1.

O que decidiu: `terminalRenders[id] = render` **nao notifica binding** (medido).
A IDE desenha so a sessao ATIVA, entao um painel direito exigiria uma segunda
vista notificante no `RuntimeController` — codigo que funciona. Com uma aba por
vez, so ha uma sessao desenhada e o `RuntimeController` nao foi tocado. O core, o
`TerminalViewport`, a grade, a roda e o cursor tambem nao.

Como a aba sabe quem e' dela sem o RuntimeController aprender o conceito: ele so
sabe pedir a aba `"terminal"`; o `AssistantController.tabFor()` traduz para
`"assistant"` quando a sessao ativa (ou a pendente, via `pendingKind`) e' do
agente, e a traducao e aplicada no AppDomains.

**3. Autocomplete travado na primeira sugestão — CORRIGIDO em 2026-07-16.**
Confirmado e fechado. **A suspeita registrada estava errada**: a tecla não era
capturada por outra camada. A cadeia inteira (`EditorTextSurface.Keys.onPressed`
→ `EditorPane` → `ShellWorkspaceHost` → `moveCompletion` → `move()`) estava
íntegra, e o `move()` movia.

O índice era **zerado por baixo**. `refilter()` terminava em `index = 0`
incondicionalmente, e `handleResolved()` chama `refilter()` — então **toda
resposta do servidor desfazia a navegação do usuário**. A janela é enorme: a
A3.2 mediu a primeira `completion` do rust-analyzer em **2520 ms**, e nesse
intervalo o usuário já desceu na lista que o fallback local (Tree-sitter) abriu
instantaneamente. Sintoma exato do relato: preso no primeiro item.

`refilter(preservarSelecao)` agora separa as duas causas, que exigem
comportamentos opostos: resposta do servidor **preserva** (por identidade do
`insertText`, não por posição — a lista nova pode vir em outra ordem); usuário
digitando **zera** (o prefixo mudou, o ranking mudou junto, o topo volta a ser a
melhor aposta, como VS Code). Item que sumiu da lista nova cai para o primeiro:
seleção fantasma aceitaria um item que o usuário não está vendo.

Coberto por 3 checks novos no `tst_completion.qml` — o teste **falhou primeiro**
(bitmask 1024) e só então passou.

**4. Ícones no app (ver §6): 158 dos 163 SVGs não estão na IDE.** Pré-requisito
do AppImage "completo" que o autor quer distribuir.

**5. AppImage para testadores.** Plano do autor: fechar A3.1–A3.4 (L0), L1, a UI
do Assistente e os ícones, e então gerar um AppImage completo para distribuir.
Não gerar antes disso; `dist/` só recebe conjunto completo por staging.

**6. Código vai ser open source — codar pensando nisso.** Observação do autor de
que há muito "comentário de IA" no projeto. Isso é uma **varredura própria**, na
mesma família da varredura de camada: comentário que narra a sessão ("a IA deve",
"nesta fatia", "o usuário pediu") não é documentação técnica e não sobrevive à
publicação. A política de tom já existe em `PLANO_ORGANIZACAO_E_HANDOFF.md` §6,
mas ela cobria `.md` — falta aplicá-la a **comentário de código**. Regra a partir
de agora: comentário explica invariante e causa, não processo nem autoria. Fatia
própria, depois da trilha atual.

### 0.2g Arquivos "god" na UI — MEDIDO e travado por catraca (2026-07-16)

Observação do autor: "tem muito arquivo god na UI de novo… precisamos de um
plano de arquitetura atômica". Medi antes de escrever plano, e o resultado
muda a resposta:

**O projeto NÃO precisa de plano de arquitetura novo. Já tem um, é bom, e não
era aplicado.** A `ARCHITECTURE.md` §6 já define a regra de split (QML visual
~300 linhas, controller/store ~400, C++ ~500, e nunca misturar renderização +
estado + IPC). O que faltava era **dente**: a regra morava num `.md` que
ninguém relê, e o gate não a checava.

Medição de 2026-07-16 — **20 arquivos acima do limite da própria regra**:

```text
1070 (limite 400, 2.7x)  ui/qml/editor/EditorController.qml
 910 (limite 500, 1.8x)  ui/src/editor_highlighter.cpp
 804 (limite 500, 1.6x)  ui/src/core_client_dispatch.cpp
 764 (limite 300, 2.5x)  ui/qml/panels/bottom/GitPanel.qml
 700 (limite 400, 1.8x)  ui/qml/Main.qml
 660 (limite 500, 1.3x)  ui/src/core_client_requests.cpp
 582 (limite 400, 1.5x)  ui/qml/shell/ShellWorkspaceHost.qml
 574 · 548 · 541 · 538 · 504 · 494 · 464 · 420 · 370 · 329 · 325 · 319 · 303
```

O sintoma mais eloquente: a `ARCHITECTURE.md` afirmava *"Estado validado em
2026-07-06: `Main.qml` tem 336 linhas"*. Dez dias depois são **700**. A regra
não foi revogada — ela apodreceu em silêncio enquanto o gate ficava verde.

**Feito: catraca no gate** (`scripts/verificar-arquitetura.sh`, dentro do
`verificar.sh`). Não é limite duro — falhar nos 20 de uma vez só ensinaria a
desligar o script. O débito fica congelado em `scripts/arquitetura-baseline.txt`
e **só pode diminuir**: arquivo novo acima do limite reprova, arquivo em débito
que cresce reprova, encolher é sempre aceito. Testado: pegou o `Main.qml`
engordando 2 linhas.

**Mea culpa:** parte do débito é desta sessão. O `RuntimeController.qml` (494) e
o `TerminalViewport.qml` (319) cresceram por minha mão hoje. A catraca vale para
mim também — foi por isso que ela nasceu com baseline em vez de exceções.

**A catraca TRAVOU a camada de shell (achado em 2026-07-16).** Não é teoria: o
seletor do §0.2f foi bloqueado por ela. Os dois arquivos que qualquer feature de
shell precisa tocar estão em débito e congelados — `Main.qml` (700/400, é o
composition root: todo controller novo é instanciado ali) e `RuntimeController`
(494/400). Não dá para adicionar nada sem pagar antes. **A catraca está certa e
fez o trabalho dela**: a arquitetura tem que ceder antes da feature entrar.

**Em andamento — split do `Main.qml` por domínio de fiação (2026-07-16).**
`700 → 610`, gate verde a cada passo (build dev-local + strict, qmllint estrito,
catraca, lógica QML).

O padrão não foi inventado: o projeto já tinha `ui/qml/ipc/<X>EventRouter.qml`
para `coreClient → controller` (o que o core **manda**). Faltava a casa do
sentido inverso, `controller → coreClient` (o que a UI **pede**), e era ele que
morava solto no composition root. Agora existe `<X>RequestRouter.qml`, simétrico:

```text
EditorRequestRouter    72 -> 19 no Main.qml   (19 pedidos)
RuntimeRequestRouter   53 -> 20               (13 pedidos)
DebugRequestRouter     30 -> 12               (11 pedidos)
```

Critério do corte: **só pedido ao core entra no router**. Fiação de controller
para HOST (abrir diálogo, focar find bar) não é IPC e fica no `Main.qml`, onde os
dois se enxergam. O `RuntimeRequestRouter` não interpreta terminal: `terminalWheel`
é só transporte, quem decide o que a roda significa é o core (`wheel_action`,
0.60.0) — a UI voltar a decidir isso foi o bug do Claude não rolar.

**O padrão sozinho NÃO chega a 400.** Medido: o que resta são `GitController`
(58), `ShellWorkspaceHost` (50), `SearchController` (45), `ProjectTreeController`
(39), `ShellHeaderHost` (36). Extrair os três controllers restantes leva a ~515 —
ainda acima. O resto é binding de propriedade e bloco de host, que **é** trabalho
de composition root e não sai por router. Chegar abaixo de 400 exige um corte mais
fundo (módulos por domínio, cada um dono do seu controller + routers), e isso é
decisão de arquitetura — não deve ser improvisada no meio de uma fatia.

**Ordem do débito** (critério da §6: "quem MISTURA responsabilidade primeiro,
não quem é maior"):

1. `Main.qml` (700) — é o composition root; dobrou de tamanho e virou o lugar
   onde tudo se conecta. Quebrar por domínio de wiring é o de maior retorno.
2. `EditorController.qml` (1070) — o maior e o que mais mistura; já tem
   subcontrollers (`documents`, `text`, `completion`, `find`), então o caminho é
   continuar movendo, não inventar estrutura.
3. `GitPanel.qml` (764) e `EditorPane.qml` (538) — visuais gordos; provavelmente
   misturam apresentação e estado.
4. `core_client_dispatch.cpp` (804) / `requests.cpp` (660) — a §5 já manda:
   "`CoreClient` é fachada única… dividida internamente por domínio quando
   crescer". Já cresceu.

Cada um é fatia própria, com a catraca atualizada no mesmo commit.

### 0.2f Assistente: o seletor tem que voltar (P2, 2026-07-16)

**Correção de premissa, e o erro foi meu.** O registro do §0.2b dizia que a UI do
assistente saiu junto com o `aiBridge`. Feedback do autor no gesto: *"antes tinha
o problema da UI ter lógica, mas a UI era boa e bonita"* — e ele está certo sobre
a perda, mas a atribuição merece precisão, porque ela decide a fatia:

> **O problema nunca foi a UI ter lógica. Era o CORE ter política por programa** —
> injetar `--no-alt-screen`/`--ax-screen-reader` e filtrar `CSI 3 J`, fazendo o
> agente se comportar diferente dentro da IDE. **O seletor Claude/Codex nunca foi
> o problema.** Ele foi removido por associação, junto com o mecanismo ruim.
> Perda desnecessária.

Estado atual (§0.2c): o atalho abre um terminal comum e o usuário digita
`claude`. Funciona, mas é regressão de fluxo frente ao que existia.

**O que precisa voltar:** ao acionar Assistente, escolher entre as CLIs de IA
**instaladas**; a escolha abre a sessão com o agente rodando.

**A linha, e ela é fina:**

```text
DETECTAR   claude/codex existem no PATH?   → é CAPACIDADE. Pode ficar no core.
EXECUTAR   como claude é rodado?           → é POLÍTICA. NÃO pode voltar ao core.
```

O `tools.detect` já é a forma certa e já existe: reporta
`{ id, displayName, status, path, version }` para cargo, clangd, git…, e a UI já
recebe a lista (`Main.qml:56` → `workspaceController.toolsList`). Detectar
`claude` é o mesmo que detectar `cargo` — não muda como o programa roda. É o
precedente do `format.capabilities` (0.61.0): o core publica o catálogo, a UI
consome.

**Estado em 2026-07-16: passo 1 FEITO (core). Passos 2–4 (UI) em aberto.**

**Decisão do autor no caminho — a sugestão de instalação virou agnóstica.** O
`ToolSpec` tinha `pacman_package: &str` e o core sugeria `sudo pacman -S <pkg>`
só quando `pacman` existia. Na Fedora do autor isso **nunca disparava**: campo
morto. Distinção que decidiu o desenho:

```text
DETECTAR   le o PATH + bit de execucao   → ja era agnostico. Nao mudou.
INSTALAR   NAO se deduz do PATH          → a ferramenta ausente e justamente a
                                           que nao esta la. Adivinhar gerenciador
                                           (ou traduzir nome por distro: `g++` e
                                           `gcc-c++` na Fedora) e palpite
                                           disfarcado de instrucao.
```

`pacman_package` foi **removido**. Regra nova: `install_command: Option<&str>` —
o core só sugere quando o comando é canônico e independente de distro (npm, no
caso das CLIs de IA); para ferramenta de distro é `None` e o gerenciador de
pacotes é assunto do usuário. **Trade-off explícito:** quem usa Arch/CachyOS
perde a sugestão `pacman` que existia. Manter Arch atendido exigiria detectar o
gerenciador presente e manter matriz de nomes por distro — não foi feito.

**Passo 1 — core: FEITO.** `claude` e `codex` no `KNOWN_TOOLS`, só detecção.
Nenhum ramo por programa, nenhum `ProfileSpec`, `flat_args` ou filtro. Coberto
por `ai_clis_are_detected_exactly_like_any_other_tool`, que compara o
tratamento de `claude` com o de `cargo` (mesmo probe, mesma estrutura) e **cai
se alguém escrever `if spec.id == "claude"` no detector** — é a cobertura
mínima que esta seção exigia. `install_suggestion_does_not_depend_on_the_distribution`
prova que ter `pacman` no PATH não muda mais nada. 268 testes verdes.

**Passos 2–4 — UI: EM ABERTO.** O desenho abaixo continua valendo:
2. **UI:** o atalho abre um seletor com as CLIs **detectadas** (status
   `Available`); ausentes aparecem com `suggested_install`, sem executar nada —
   o `ToolInfo` já carrega esse campo e o contrato já diz que o core nunca roda
   a sugestão.
3. **UI:** escolhido o agente, abrir a sessão rotulada e mandar o comando por
   `terminal.input`, como se o usuário tivesse digitado. É a Opção B do §0.2c —
   e o comando deve vir do `path`/`id` **detectado**, não de string literal no
   QML, senão a política só migrou de camada.
4. **rail:** o ícone fica sempre visível (já está; `enabled` segue o workspace).

Isso resolve o §0.2c-B e o §0.2d-2 (renomear) de uma vez: o seletor é o lugar
natural de "Agente Auxiliar" aparecer.

**Cobertura mínima exigida:** um teste que falhe se o core ganhar ramo por
programa, e harness do seletor (o `RuntimeController` é QtQuick puro e já é
testado em `tst_multi_terminal.qml`).

### 0.2e L1 — auditoria do EditorConfig (2026-07-16). RESULTADO NEGATIVO.

O `PONTO_ATUAL` exige, antes de qualquer código: "confirmar biblioteca/licença,
contrato e conflito com settings". A auditoria foi feita e **contraria a
recomendação do plano**. EditorConfig continua sendo boa ideia; a premissa de
que existe biblioteca Rust madura para ele **não se sustenta**.

| Crate | Última publicação | Downloads recentes | Veredito |
| --- | --- | ---: | --- |
| `editorconfig-core` 0.1.3 | jul/2025 | 22 | passa a suíte oficial de conformidade, mas é nicho: 1 mantenedor, 534 linhas, 4 versões em 3 dias e nada depois |
| `editorconfig` 1.0.0 | **nov/2017** | 461 | abandonado há nove anos |
| `editorconfig-rs` 0.2.3 (+ `-sys`) | fev/2025 | 15 | mantido, e liga a **libeditorconfig oficial** — mas por FFI a uma lib C |

Todos MIT, então licença não é o problema. Manutenção e forma são.

**O conflito com a política do projeto.** O `AGENTS.md` proíbe "escolher revisão
antiga/abandonada apenas porque contém uma função conveniente", e o ADR-0004
escolheu o `alacritty_terminal` explicitamente por ser "Rust puro… sem Node,
Electron, WebView ou FFI". As três opções violam um desses:

- `editorconfig`: abandonado;
- `editorconfig-core`: vivo mas sem adoção (22 downloads) — adotar é assumir a
  manutenção de fato, sem o benefício de uma base testada por terceiros;
- `editorconfig-rs`: o único com pedigree (liga o core **oficial**), mas traz
  FFI + dependência nativa C, exatamente o que o projeto vem evitando, e
  complica o AppImage.

**Não decidido — decisão do autor, em fatia própria.** As saídas reais:

1. **`editorconfig-rs` + `-sys`**, aceitando FFI e a dependência C, com o
   benefício de ser o core oficial. Exige auditar o impacto no empacotamento.
2. **Parser próprio**, contrariando "orquestrar, nunca reimplementar" — mas a
   spec do EditorConfig é pequena (INI + globs + `root=true`) e existe uma
   **suíte de conformidade oficial** (`editorconfig-core-test`) que tornaria a
   decisão verificável em vez de arrogante. É o argumento mais forte a favor.
3. **Trocar o primeiro recorte do L1.** EditorConfig foi escolhido por ser
   "pequeno e auditável"; a auditoria mostra que o pequeno não é tão pequeno. O
   inventário das ferramentas já detectadas (clangd, rust-analyzer, CMake,
   Cargo, Git, rg, fd, lldb-dap, Clippy) valida o `integration` v1 **sem
   dependência nova nenhuma**, e talvez seja o primeiro recorte mais honesto.

**Conflito com settings, mapeado.** `SettingsValues` já tem `format_on_save`,
`auto_close_pairs` e `editor_font_size`; o EditorConfig traria `indent_style`,
`indent_size`, `tab_width`, `end_of_line`, `charset`,
`trim_trailing_whitespace` e `insert_final_newline` — **nenhum colide hoje**.
A regra de precedência do roadmap ("aplicar `.editorconfig` antes das
preferências globais") vale para os campos novos; se um dia um campo coincidir,
a precedência tem de ser explícita no contrato, não implícita no código.

Recomendação: **opção 3 primeiro** (inventário valida o `integration` v1 sem
dependência), depois decidir 1 vs 2 para o EditorConfig com o contrato já de pé.

### 0.2j Assistente/KV Context — CONSTRUÍDO E REMOVIDO (2026-07-17)

**Decisão do autor, depois de rodar: a linha inteira de IA na IDE está FORA DE
ESCOPO.** O §0.2f pedia o seletor de volta; ele voltou, funcionou, e o próprio uso
mostrou que não se paga:

> "do jeito que foi implementado não faz sentido usar esse atalho visual, pois o
> usuário pode usar o claude/codex e outros agentes via terminal naturalmente"

O atalho poupava digitar **uma palavra** (`claude`), e cobrava por isso um
seletor, um rótulo, uma numeração, uma aba e um ícone. Removido no mesmo dia.
Registro canônico e o obstáculo técnico medido: topo e §4 de
`docs/specs/KINEIN_VECTIS_ASSISTANT_AI_ASSISTANCE.md`. Fase M6.4 do roadmap 21
marcada fora de escopo.

**O que a fatia deixou de bom, e sobreviveu à remoção:**

```text
fix do coreClient null       15 roteadores mortos; a IDE nao lia pastas nem
                             criava projetos. Achado SO porque a feature obrigou
                             a subir o binario novo.
verificar-qml-fiacao.sh      trava do binding auto-referente `x: x`. Nenhum gate
                             pegava essa classe: build passa, qmllint diz limpo,
                             o boot vai ao primeiro frame e o Qt nao avisa.
ARCHITECTURE §4 regra 9      3 casos medidos da catraca + "quando ela dispara ha
                             tres suspeitos: a sua mudanca, a categoria, o
                             arquivo" + o teste do VOCABULARIO.
TerminalSessionTabs.qml      a barra de chips (183 linhas) saiu do BottomPanelHost,
                             que caiu 550 -> 383 e SAIU do debito.
RuntimeController 309        volta a fazer so uma coisa: manter sessoes. Era 494
                             ontem, 397 no f7f472b. Menor do que jamais foi.
```

**O que sobrou no core, de propósito:** `claude` e `codex` no `KNOWN_TOOLS`,
detectados como `cargo`/`clangd` — mesmo probe, zero ramo por programa, coberto
por `ai_clis_are_detected_exactly_like_any_other_tool`. Não é assistente: é o
painel Ferramentas dizendo se o binário está no PATH, e é mais útil agora que o
fluxo é o terminal direto.

**Resto conhecido, NÃO resolvido:** `assistant_terminal_width` ainda existe em
`kinein-config`/`settings.rs` e no protocolo, sobra do aiBridge (0.59.0). A UI
não lê mais (removida a propriedade morta do `SettingsController`). Tirar do core
é mudança de contrato e pede decisão de versão — **fatia própria**, não se
enfia numa remoção de UI.

### 0.2i Os testes de lógica QML não conseguiam reprovar (achado e CORRIGIDO em 2026-07-16)

Achado ao escrever o teste do autocomplete: o teste novo passava verde **com o
bug presente**. A causa não era o teste — era a suíte.

**Código de saída de processo tem 8 bits.** `Qt.exit(256)` sai como **0**. Como
todo harness fazia `Qt.exit(bitmask)` direto, qualquer check com bit ≥ 256 era
letra morta: falhava e o gate dizia `ok`. Medido em 2026-07-16, **7 dos 14
harnesses** tinham checks nessa faixa:

```text
tst_multi_terminal            32 checks, bits ate 2^31  (~24 mortos)
tst_format_capabilities       15 checks, bits ate 2^14  (~7 mortos)
tst_terminal_metrics          14 checks                 (~6 mortos)
tst_terminal_scroll           13 checks                 (~5 mortos)
tst_find                      12 checks                 (~4 mortos)
tst_terminal_input            11 checks                 (~3 mortos)
tst_completion / tst_terminal_geometry_overlay: 9 checks (1 morto cada)
```

A ironia é exata: esta suíte **existe** porque o bug D1 sobreviveu a dois ciclos
de correção com "sonda verde no backend, GUI quebrada" — e ela carregava o mesmo
vício. O `tst_shell_functional` já se defendia com `Math.min(failures, 255)`;
a lição nunca foi generalizada para os outros treze.

**Corrigido:** o bitmask vai para a SAÍDA (`console.error`, onde não trunca) e o
código de saída só diz passou/falhou. Nenhum diagnóstico se perde e nada mais
passa verde por estouro.

**O que os checks ressuscitados revelaram:** um só, e era o **teste** que estava
errado, não o produto. O `tst_multi_terminal` exigia que abrir um terminal comum
mantivesse o contexto ativo — contradizendo a própria linha 92 do arquivo, que
exige "abrir ativa". `handleTerminalOpened` termina em `selectTerminal(id)`, e
está certo. Expectativa corrigida; o produto não mudou.

### 0.2h Teste instável em `tools` (achado em 2026-07-16, P3)

Encontrado ao rodar o gate, **não é regressão de fatia nenhuma** — reproduz em
árvore que não toca Rust:

```text
tools::tests::detected_tool_reports_path_and_version  → status Failed, esperado Detected
tools::tests::fd_detection_accepts_fdfind_binary_name → PoisonError (cascata do anterior)
```

Bateu 1 vez em 3 execuções da suíte completa; isolado (`cargo test -p kinein-core
--lib tools::`) passa sempre. A causa provável está escrita no próprio arquivo: o
`EXEC_LOCK` (`crates/kinein-core/src/tools.rs:418`) existe porque "um teste pode
forkar enquanto outro ainda segura o descritor de escrita do script, e o exec
falha com `ETXTBSY`" — mas ele serializa só os testes de `tools` entre si. Outro
teste da suíte forkando no momento errado reproduz o mesmo `ETXTBSY`, e a segunda
falha é cascata (o primeiro morreu segurando o mutex, envenenando-o).

**Por que não é cosmético:** gate que reprova ao acaso ensina a reexecutar até
passar, que é a mesma doença que a catraca do §0.2g foi desenhada para evitar.
Fatia própria: fechar o descritor antes do exec (ou `O_CLOEXEC`), não aumentar o
escopo do lock.

### 0.3 Sessão de organização e feedback (2026-07-16)

Sessão de trabalho autônoma autorizada pelo autor (com backup; proibido git
destrutivo/histórico). Entregas e feedback:

- **Reorganização da documentação (concluída, sem commit):** os `.md` técnicos
  foram agrupados por assunto sob `docs/{arquitetura,build,seguranca,roadmaps}`
  via `git mv` (nomes preservados), 410 referências raiz-relativas reescritas em
  77 arquivos (docs + comentários de código + scripts + CMake), `docs/README.md`
  reescrito como índice e 0 links markdown quebrados. `cargo check` verde.
  Raiz intacta: `README/MANUAL/Tutorial/AGENTS` e os pessoais. Plano completo
  (faixa pessoal, não publicar): `PLANO_ORGANIZACAO_E_HANDOFF.md`.
- **Scroll do agente Claude no Assistente — SUPERADO. O texto abaixo descreve a
  `aiCliFlatTranscript`, que NAO existe mais** (removida com o aiBridge no
  0.59.0: injetar `--ax-screen-reader` era politica por programa no core). A
  correcao vigente e o encaminhamento da roda ao aplicativo (0.60.0,
  `docs/roadmaps/26` §11.4). Mantido como registro do que foi tentado:
  **RESOLVIDO como toggle (protocolo 0.58.0):** a causa-raiz era o Claude interativo usar **tela alternada** (sem
  scrollback por semântica VT), sem flag inline como o `--no-alt-screen` do
  Codex. O teste ao vivo confirmou: com `--ax-screen-reader` o scroll e o cursor
  funcionam, mas a TUI decorativa vira texto puro. Por isso virou preferência:
  setting **`aiCliFlatTranscript`** (default `false` = TUI decorativa), exposta
  em Configurações como "Assistente: histórico navegável", lida no
  `aiBridge.terminal.open` e válida na próxima sessão. `ProfileSpec` ganhou
  `flat_args`; Codex mantém `--no-alt-screen` nos dois modos (já tem TUI **e**
  histórico). Contrato/manual/schema atualizados; 3 testes novos de perfil.
  Detalhes em `docs/roadmaps/26` §11. Opção A (encaminhar roda ao app em
  alt-screen — corrige vim/htop também) segue no backlog §6.
- **"Modo imagem" para `.md` (backlog, ver seção 6):** preview de markdown
  renderizado no editor, estilo JetBrains, reusando `TextEdit.MarkdownText` já
  usado pelo `DocumentationDialog`.

## 1. TR0 — aceite funcional da rodada atual

Antes de ampliar o produto, validar a aplicação real em tela. Regressões
encontradas aqui têm prioridade e não autorizam outro remake visual.

### 1.1 Checklist de validação manual

- **Autocomplete:** a primeira sugestão estrutural aparece sem esperar o LSP;
  a resposta semântica de clangd/rust-analyzer substitui o fallback quando
  estiver pronta, sem popup duplicado ou salto de seleção.
- **Terminal sob rajada:** saída contínua acompanha a linha atual sem atrasos;
  a barra aparece assim que existe histórico; rolar ou arrastar preserva a
  leitura; nova entrada do usuário volta ao final; múltiplas abas permanecem
  independentes.
- **Menus Arquivo–Ajuda:** todas as opções ficam acima do editor, legíveis e
  acionáveis. `Ajuda → Manual da IDE` abre a documentação interna.
- **Criação no projeto:** menu Arquivo e clique direito oferecem adicionar
  arquivo/pasta e reutilizam o fluxo confinado ao workspace.
- **Assistente:** Claude/Codex instalados pelo usuário são descobertos; a
  escolha abre uma sessão própria sobre o terminal real; Codex preserva
  scrollback em modo inline; barra/roda/arrasto, teclado, seleção, copiar/colar
  e resize se comportam como no Terminal integrado, inclusive durante nova
  saída; a sessão tem largura livre/persistida, coexiste com `Project` e pode
  ser ampliada sem acoplar o painel ao terminal comum.
- **Estrutura e painéis:** a aba Estrutura redimensiona, recolhe e restaura;
  o layout inicial se adapta à janela sem cobrir editor ou menus.
- **Barras e tooltips:** editor, terminal e listas longas mostram posição e
  permitem arrastar; descrições de ícones nunca aparecem sob o editor.

### 1.2 Critério de saída do TR0

- checklist acima aceito em uma sessão real;
- qualquer falha reproduzível ganhou teste/harness quando aplicável;
- `bash scripts/verificar.sh` verde;
- aceite visual R7/C6 confrontado com as specs, sem mudança estética lateral.

O início do dogfooding não precisa esperar uma cerimônia separada de TR0: a
primeira sessão dentro da Kinein deve percorrer este checklist naturalmente.
Falhas encontradas nela interrompem a próxima fatia do roadmap até a regressão
correspondente ficar corrigida e protegida.

## 2. TR1 — distribuição e substituição de editores generalistas

### A3 — responsividade medida (ENTREGUE em 2026-07-16)

A3.1–A3.4 estão fechadas. O item 1 do A3.3 (digitação tecla→frame no editor
real) era o último em aberto e foi entregue em 2026-07-16: mediana 7,4 ms, p95
8,4 ms, orçamento 16/20 ms. **Nenhuma afirmação de responsividade do A3 depende
mais de impressão visual.**

Baseline versionada, orçamentos, método e as armadilhas que produzem número
falso estão em `docs/roadmaps/21` (A3.1–A3.4); o estado implementado, em
`ContextoIA.md`. A entrada única continua sendo `scripts/medir-performance.sh` +
`scripts/medir-core.py` — não criar segundo runner nem framework de benchmark.

Reação a regressão é gate, não sugestão: número acima do orçamento abre fatia de
causa-raiz antes de nova profundidade semântica, e antes de otimizar, perfilar.

A primeira integração pequena recomendada a seguir era **EditorConfig** — mas a
auditoria do §0.2e deu **resultado negativo** e a decisão está com o autor.
Ler o §0.2e antes de escrever qualquer código de L1.

### A4 — confortos que bloquearem o dogfooding

Prioridade inicial, ajustada pelos motivos reais de saída para outro editor:

1. split editor;
2. multicursor;
3. EditorConfig;
4. zoom do editor;
5. links e busca no scrollback do terminal;
6. duplo clique para selecionar palavra no terminal.

Aceite do TR1: uma semana de desenvolvimento C/C++ e Rust sem abrir editor
generalista auxiliar. Quando o usuário disser **“estou no Kinein”**, registrar
cada exceção e corrigir primeiro o bloqueio reproduzível. Feedback dos
testadores entra no mesmo funil, identificado pela origem e pelo ambiente, sem
substituir evidência de reprodução.

### A5 — candidatos explícitos para análise futura de integrações

Lista solicitada pelo usuário em 2026-07-15. **Não é decisão de adoção nem fila
de implementação imediata**: alguns itens já estão presentes ou registrados,
outros podem ser substituídos por opção mais adequada. A sessão própria deve
confrontar manutenção atual, licença, segurança, compatibilidade Linux-first,
duplicação do que existe e encaixe Qt/QML → IPC → Rust Core antes de escolher:

1. **Open Remote SSH (Open VSX/comunidade):** já consta no roadmap de adaptação
   como referência FOSS. Reavaliar UX de abrir pasta, arquivos e sessão remotos
   sem executar extensão VS Code/VSCodium dentro da Kinein nem depender de
   servidor proprietário.
2. **SSHFS / sshfs-win por CLI:** estudar montagem de Raspberry Pi, satélite ou
   host embarcado como árvore local. Comparar com OpenSSH direto e considerar
   latência, desconexão, watcher, escrita atômica e o fato de `sshfs-win` não
   ser a variante primária de uma IDE Linux-first.
3. **Bear (Build EAR):** avaliar geração de `compile_commands.json` ao
   interceptar builds C/C++ quando CMake File API/presets ou banco de compilação
   nativo do projeto não estiverem disponíveis; não substituir CMake nem
   clangd.
4. **CodeLLDB:** já é referência explícita no roadmap e a Kinein já possui
   fundação DAP com `lldb-dap`. Analisar apenas lacunas concretas de experiência
   e embarcados, sem incorporar o host de extensão.
5. **libssh / ssh2-rs:** candidatos nativos para sessão, túnel e SFTP. Comparar
   custo de dependência, superfície de segurança e manutenção com a política
   atual de orquestrar OpenSSH CLI antes de escolher biblioteca de binding.
6. **Valgrind / Memcheck:** avaliar integração de análise de vazamentos e
   acessos inválidos C/C++ como job cancelável, com parser no core e resultados
   navegáveis na UI.
7. **Heaptrack:** avaliar profiling de memória C/C++ e Rust e visualização de
   resultados sem criar profiler próprio; medir custo e disponibilidade nas
   distribuições suportadas.
8. **Clippy:** já está implementado no strict gate e em `quality.run` para
   Rust. A análise futura deve tratar somente lacunas de UX/diagnósticos, não
   adicionar outro linter equivalente.
9. **Sigrok / PulseView:** avaliar captura e decodificação de sinais como I2C,
   SPI e UART e uma visualização integrada para analisadores lógicos. Preferir
   orquestrar o motor/protocolo aberto e não recriar decoders ou osciloscópio.
10. **Serial Studio:** avaliar telemetria serial em tempo real e painéis de
    gráficos, mapas, bússolas e medidores. Comparar integração externa,
    formatos de dados e custo de uma UI nativa antes de qualquer adoção.
11. **Wokwi CLI / QEMU:** QEMU já pertence à trilha embarcada; acrescentar
    Wokwi como candidato para simulação sem placa de ESP32/STM32 e afins.
    Verificar licença, dependência de serviço/rede, reprodutibilidade e
    adequação à política local-first antes de escolher.
12. **Unity / Google Test (GTest) / Criterion:** estudar descoberta e consumo
    de relatórios de testes C/C++ no painel existente. Comparar com CTest já
    suportado; para Rust, preservar `cargo test` como fonte autoritativa.
13. **Doxygen:** candidato para geração acionável de documentação C/C++/Rust a
    partir do projeto, como job externo e cancelável, sem gerador próprio.
14. **Sphinx / Breathe:** avaliar composição de manuais sobre a saída do
    Doxygen e geração HTML/PDF, mantendo templates e configuração pertencentes
    ao projeto do usuário.
15. **Bloaty McBloatface:** avaliar análise de tamanho de binários e seções para
    firmware, com resultados navegáveis e visualizações de consumo de flash;
    não criar analisador binário interno.
16. **GitOxide (gix) / libgit2:** candidatos nativos para Git. A Kinein hoje já
    oferece status, diff, blame, histórico, branches, stash e operações remotas
    orquestrando Git no core; comparar segurança, paridade e custo antes de
    substituir uma CLI madura ou duplicar funcionalidades existentes.
17. **DAP / lldb-dap / gdb-dap:** DAP e `lldb-dap` já formam a fundação de
    debug da Kinein. Avaliar `gdb-dap` e lacunas de hardware sem criar outro
    protocolo nem interpretar a saída humana de GDB/LLDB.
18. **LSIF:** avaliar índices persistentes para navegação em bases C/C++ muito
    grandes e comparar com formatos/ecossistemas atuais antes de escolher. Não
    duplicar clangd nem iniciar indexador global sem orçamento medido.
19. **OpenOCD:** candidato principal para flash e debug JTAG/SWD, orquestrado
    como processo confinado e observável. Separar transporte GDB, comandos de
    controle e diagnóstico; Telnet/RPC entram na mesma análise, sem comandos
    montados por shell.
20. **pyOCD:** candidato Cortex-M complementar ao OpenOCD, especialmente para
    automação e probes CMSIS-DAP; comparar cobertura de targets, distribuição,
    dependência Python e paridade antes de ativar por placa.
21. **CMSIS-DAP:** protocolo/firmware de probe ARM, não plugin de UI. Modelar
    como capacidade detectada do adaptador físico e fonte para OpenOCD/pyOCD.
22. **avrdude / esptool / stlink:** candidatos específicos para flash AVR,
    ESP e STM32. Cada ferramenta precisa de adapter tipado, detecção de versão,
    preview do comando, cancelamento e logs; nada de executor shell genérico.
23. **Cppcheck / LLVM Clang Static Analyzer:** candidatos C/C++ para
    `quality.run`, comparados com clang-tidy já adotado. Resultados devem virar
    diagnósticos comuns e evitar três análises equivalentes por padrão.
24. **cargo-audit / cargo-deny:** candidatos Rust para vulnerabilidades,
    advisories, fontes e licenças. Diferenciar análise local do lockfile de
    atualização de bancos pela rede e exigir ação/consentimento explícitos.
25. **gcov / lcov:** candidatos de cobertura C/C++ com build instrumentado,
    coleta separada e relatório por arquivo/linha; preservar presets e targets
    do projeto, sem injetar flags silenciosamente.
26. **cargo-tarpaulin:** candidato de cobertura Rust, a comparar com alternativas
    atuais, compatibilidade de toolchain e custo Linux. A fonte de testes segue
    sendo `cargo test`.
27. **Ghidra:** candidato de engenharia reversa e decompilação como ferramenta
    externa pesada. Avaliar importação/exportação e navegação ELF/assembly sem
    embutir toda a suíte ou prometer edição round-trip.
28. **GDB/LLDB MI:** fallback estruturado apenas para capacidades de baixo nível
    realmente ausentes no DAP. Preferir DAP; qualquer MI precisa de parser de
    protocolo, máquina de estados, timeout e testes, nunca scraping do terminal.
29. **libelf / goblin:** candidatos para ELF, seções, símbolos e mapas de
    memória. Comparar biblioteca nativa/FFI com crate Rust segura e pequena;
    não duplicar Bloaty quando um relatório externo bastar.
30. **DWARF / gimli:** padrão e biblioteca candidata para mapear endereços a
    fontes/símbolos quando DAP não fornecer o dado. Exigir limites de memória,
    parsing lazy e fixtures de ELF reais.
31. **udevadm / libudev:** candidatos Linux para detectar probes e placas por
    VID/PID. O core deve observar eventos e sugerir configuração; nunca iniciar
    flash automaticamente sem confirmação do usuário.
32. **PlatformIO Storage Architecture / sysroots isolados:** estudar somente a
    estratégia de manifests, pins, checksums, isolamento e cache, sem incorporar
    PlatformIO Core ou baixar toolchains silenciosamente. Instalação, rede,
    licença, proveniência, rollback e espaço em disco exigem política própria.
33. **CMSIS-Pack:** candidato para descrição ARM, SVD, startup, drivers e mapa
    de memória. Tratar `.pack` como conteúdo não confiável, validar manifestos,
    caminhos e licenças e nunca executar código do pacote durante a leitura.
34. **DTS / DTB:** candidato para navegação, validação e compilação de Device
    Tree via ferramentas maduras. Qualquer editor gráfico deve preservar o
    texto autoritativo e round-trip verificável.
35. **SWO / ITM:** candidatos de trace não intrusivo ARM, integrados ao
    transporte/probe selecionado. Separar aquisição de alta taxa, decode,
    backpressure, persistência e visualização.
36. **CTF / LTTng:** candidatos para trace estruturado e timelines, sobretudo
    Linux/RTOS. Avaliar leitores maduros e streaming limitado antes de criar
    armazenamento ou gráficos próprios.
37. **libclang / Clang C API:** candidato explícito do usuário para AST e
    análises profundas/call graphs. A sessão futura deve medir custo de memória,
    ABI/distribuição e duplicação com clangd + Tree-sitter. Só adotar quando uma
    capacidade concreta não puder ser obtida pelos serviços existentes; o peso
    maior é aceito apenas com benefício e orçamento demonstrados.
38. **gcov-kernel / kcov:** candidatos especializados para cobertura de kernel
    Linux e drivers, fora do fluxo padrão de usuário; exigem privilégios,
    isolamento e documentação de ambiente próprios.
39. **MQTT / CoAP / Mosquitto:** candidatos para telemetria de bancada remota.
    Diferenciar cliente, broker e protocolo; rede permanece opt-in, com destino
    visível, TLS/credenciais protegidos e nenhuma telemetria da IDE.
40. **TimescaleDB / InfluxDB / LMDB / RocksDB:** candidatos distintos para
    séries temporais ou armazenamento local de captura. Bancos-servidor não são
    equivalentes a motores embutidos; escolher somente após medir volume,
    retenção, consulta, operação e portabilidade.
41. **Jinja2 / Tera:** candidatos de template para geração explícita de código
    de inicialização. Preferir Tera se a camada permanecer Rust, mas só após
    definir arquivos gerados, ownership, preview, diff, regeneração e proteção
    contra sobrescrever edição manual.
42. **OpenModelica / OMSimulator:** candidatos para simulação física
    mecânica/elétrica/térmica por CLI/API. Avaliar licença, distribuição,
    execução longa cancelável, artefatos e importação de resultados sem acoplar
    o core ao runtime da suíte.
43. **FMI / FMUs:** padrão candidato para co-simulação. Tratar FMU como pacote
    de binário não confiável: inspeção, compatibilidade de arquitetura,
    sandbox, limites e consentimento precedem qualquer carregamento.
44. **GSL / ndarray:** candidatos de computação numérica para ferramentas ou
    fixtures científicas, não APIs automaticamente expostas ao código do
    usuário. Definir primeiro o caso de uso e evitar transformar a IDE em um
    runtime matemático próprio.
45. **perf / Hotspot:** candidatos Linux para profiling e flame graphs. Preferir
    produzir/consumir formatos maduros, detectar permissões do kernel e manter
    coleta separada da visualização.
46. **tokio-console:** candidato Rust para tarefas assíncronas instrumentadas.
    A integração deve detectar a instrumentação necessária e apresentar
    claramente que não funciona em binários Tokio não preparados.
47. **Frama-C / Kani:** candidatos de verificação formal para C e Rust. Entram
    como jobs especializados, opt-in e reprodutíveis; não devem ser rotulados
    como prova de segurança total nem misturados ao lint rápido padrão.
48. **OTAWA / análise WCET:** candidato de pior tempo de execução para sistemas
    críticos. Avaliar maturidade, arquiteturas suportadas, modelos de hardware,
    licença e validade das premissas antes de qualquer promessa de certificação.
49. **lm-sensors / Open Hardware Monitor CLI:** candidatos de observação do host
    durante simulação/profiling. Para Linux-first, avaliar `lm-sensors` primeiro;
    correlação energética não autoriza coleta persistente ou telemetria padrão.
50. **D-Bus:** API de integração Linux para eventos realmente necessários de
    sistema/energia/rede. Assinar apenas interfaces allowlisted; não usar como
    barramento genérico nem monitorar o desktop inteiro.
51. **ccache / sccache:** candidatos de cache de compilação, ativados por
    toolchain/preset explícito e diagnóstico de hit/miss. Não reescrever linhas
    de compilação nem enviar cache remoto sem configuração e consentimento.
52. **distcc:** candidato de compilação distribuída C/C++ para laboratórios.
    Exige confiança entre hosts, toolchains idênticas, rede configurada pelo
    usuário, falha segura e medição que justifique a complexidade.
53. **OpenGL e tecnologias gráficas afins:** trilha de rendering para simulação
    gráfica Linux-native citada pelo usuário. Tratar como subsistema de
    visualização/simulação com API e isolamento próprios, não como plugin de
    linguagem nem lógica embutida no editor.

#### A5.1 — arquitetura futura da biblioteca de integrações

O usuário quer uma aba visual semelhante a uma biblioteca, capaz de mostrar e
ativar/desativar integrações. O nome de produto pode ser “Plugins”, mas a
arquitetura deve registrar a natureza real de cada item: ferramenta externa,
protocolo, formato, biblioteca vinculada, serviço ou visualizador. Desenhar em
sessão própria antes de implementar. A arquitetura-base desta sessão fica
fechada assim:

- registry tipado no Rust Core com id estável, categoria, versão detectada,
  licença, origem, capacidades, requisitos, conflitos e estado de saúde;
- ativação global ou por workspace persistida por schema, com defaults mínimos,
  lazy start, orçamento de CPU/RAM e desligamento/cancelamento determinísticos;
- adapters pequenos por integração atrás de contratos de domínio; a UI lista,
  configura e solicita ações, mas não inicia processos nem carrega plugins;
- dependências, permissões, USB/rede/privilégios e downloads sempre visíveis e
  confirmáveis; pins, checksums, rollback e diagnóstico para pacotes geridos;
- nenhum extension host genérico, marketplace executável, código remoto ou
  telemetria. Ativar/desativar não pode alterar arquivos/toolchains do projeto
  silenciosamente;
- separar aquisição, processamento, armazenamento e visualização para sinais,
  trace, profiling e simulação; aplicar backpressure e limites desde o início;
- estudar Code OSS, IntelliJ Community, Zed/Lapce/NetBeans e ferramentas
  oficiais pertinentes por revisão atual, registrando invariantes e adaptação
  Qt/QML → IPC → Rust Core antes de cada fatia.

O nome visível pode ser **Plugins**, mas os nomes internos serão
`integration`/`capability`/`adapter`. Não haverá API para carregar código de
terceiros. O crescimento é incremental, sem criar um crate ou framework vazio:

```text
QML Plugins/Integrations view (apresenta e solicita)
        ↓ CoreClient
kinein-protocol::integration (descriptor, state, action, permission)
        ↓
kinein-core/src/integration/ (registry + policy + health)
        ↓
adapter pequeno do domínio → Job cancelável → ferramenta/protocolo externo
        ↓
eventos tipados → Problems/Tests/Profiler/Trace/Simulation
```

O primeiro recorte nasce junto de EditorConfig e registra também as ferramentas
já detectadas; só depois o módulo é dividido. Um descriptor precisa carregar
`id`, natureza, categoria, modo A–D, origem/licença, versão detectada,
capacidades, requisitos, permissões, instalação, escopo de ativação e saúde.
Uma ação carrega risco, efeitos, necessidade de rede/USB/privilégio, preview e
tipo de Job. Estado e configuração usam schema; segredo fica fora do registry.

#### A5.2 — sequência linear vinculante de capacidade

Cada nível depende dos anteriores. Não começar uma integração de nível alto
apenas porque sua CLI é fácil de chamar.

| Nível | Entrega arquitetural primeiro | Integrações que validam o nível |
| --- | --- | --- |
| L0 | fechar A3.1–A3.4 e orçamentos; sem plataforma genérica antes da baseline | nenhuma nova |
| L1 | `integration` v1 por fatia vertical: descriptor/health/configuração, leitura do registry existente e aba somente informativa | EditorConfig; inventário de clangd, rust-analyzer, CMake, Cargo, Git, rg, fd, lldb-dap e Clippy |
| L2 | resultado comum para diagnóstico/teste/cobertura, artefatos e Jobs canceláveis | cargo-audit/deny, Cppcheck/Clang Static Analyzer, Valgrind, GTest/Unity/Criterion, gcov/lcov e cobertura Rust |
| L3 | Project Graph/targets/perfis explicáveis, cache provenance e geração de artefatos | Bear, ccache/sccache, Bloaty, Doxygen e Sphinx/Breathe |
| L4 | DAP sólido, sessão de profiling, importador de relatório e permissões do kernel | lldb/gdb DAP, Heaptrack, perf/Hotspot, tokio-console; MI/ELF/DWARF apenas por lacuna |
| L5 | `RemoteContext`: host keys, credenciais externas, path mapping, desconexão, sync e Jobs remotos. **DOCKER ENTRA AQUI** — container é um contexto remoto (`container://` ao lado de `ssh://`), não um nível próprio; **domínio NATIVO do core** (decisão do autor, 2026-07-17) | OpenSSH/Open Remote UX; spec `devcontainer.json` (aberta) como formato de entrada; Podman a auditar (rootless muda a permissão); SSHFS e bindings SSH só como alternativas |
| L5.5 | **BANCO DE DADOS** — **domínio NATIVO** (decisão do autor, 2026-07-17), não plugin de terceiro. Vertical própria: conexão (credencial vem do L5) → schema browser (leitura) → query console (query é Job cancelável, L2) → result grid (o componente mais caro; não é o começo). Um driver só, não uma matriz | **O IntelliJ Community NÃO serve de referência: Database Tools é Ultimate** (verificado 2026-07-17). Candidatos a auditar: DBeaver, Database Navigator, SQLTools. Rust puro > FFI (precedente ADR-0004) |
| L6 | `Target`/`Device`/`Probe`, detecção USB, package trust, flash preview e confirmação | udev, CMSIS-DAP/Pack, DTS/DTB, OpenOCD, pyOCD, probe-rs, avrdude, esptool, stlink, QEMU e Tera |
| L7 | streaming com backpressure, timestamp, canais, retenção e segurança de rede/dispositivo | sigrok, SWO/ITM, CTF/LTTng, MQTT/CoAP/Mosquitto, lm-sensors e D-Bus allowlisted |
| L8 | armazenamento medido e API de visualização isolada do editor/core | banco local ou de séries temporais escolhido por benchmark, OpenGL/Qt rendering e computação numérica específica |
| L9 | sandbox de pacote/binário, compatibilidade de arquitetura e co-simulação reproduzível | FMI/FMU antes de OpenModelica/OMSimulator; Wokwi somente opt-in externo/nuvem |
| L10 | laboratório opt-in, sem promessa de suporte diário | SCIP/indexação persistente, Ghidra, libclang, gcov-kernel/kcov, Frama-C/Kani, OTAWA e distcc |

Gate de promoção de nível: ao menos uma integração vertical real, testes de
falha/cancelamento, orçamento medido, configuração reversível e nenhum
processo/handle órfão. A aba visual não desbloqueia o nível; o contrato e o
serviço comprovados o desbloqueiam.

#### A5.3 — decisão linear para os 53 candidatos

Legenda: **manter** = já existe; **adotar** = candidato principal após o gate;
**condicional** = só com lacuna/PoC; **referência** = estudar ou interoperar,
sem incorporar; **substituir** = avaliar primeiro a alternativa indicada.

| # | Candidato | Decisão | Ordem e limite |
| ---: | --- | --- | --- |
| 1 | Open Remote SSH | referência | L5; UX de referência, implementação por OpenSSH direto |
| 2 | SSHFS / sshfs-win | condicional | L5 depois do RemoteContext; nunca transporte padrão; `sshfs-win` fora do foco Linux |
| 3 | Bear | adotar | L3, somente fallback quando o build não fornece compile database confiável |
| 4 | CodeLLDB | manter/referência | L4; `lldb-dap` via DAP já é a integração, sem Extension Host |
| 5 | libssh / ssh2-rs | condicional | L5, apenas se OpenSSH CLI não cobrir sessão/SFTP/túnel necessário |
| 6 | Valgrind/Memcheck | adotar | L2, Job opt-in e achados navegáveis; não bloquear lint rápido |
| 7 | Heaptrack | adotar | L4, coleta separada de visualização e importação de artefato |
| 8 | Clippy | manter | L1/L2, melhorar UX de diagnósticos; não integrar segundo Clippy |
| 9 | sigrok/PulseView | adotar/referência | L7; motor/CLI e formatos em Mode-A, PulseView pode abrir externamente |
| 10 | Serial Studio | referência | L7; interoperabilidade/formato apenas, sem copiar GPLv3 nem módulos Pro |
| 11 | Wokwi CLI / QEMU | separar | QEMU em L6 local-first; Wokwi em L9, externo, token/rede/upload explícitos |
| 12 | Unity/GTest/Criterion | adotar protocolos | L2; descobrir/executar via CTest/Cargo e consumir relatórios, sem empacotar frameworks |
| 13 | Doxygen | adotar | L3 como Job de projeto explícito |
| 14 | Sphinx/Breathe | condicional | L3 depois de Doxygen, para projetos que já escolheram essa cadeia |
| 15 | Bloaty | adotar | L3, análise de artefato/flash sem parser binário próprio |
| 16 | gix/libgit2 | não agora | L10 apenas se Git CLI demonstrar lacuna; evitar duplicação e FFI |
| 17 | DAP/lldb-dap/gdb-dap | manter/adotar | L4; consolidar DAP/lldb e avaliar `gdb.dap`; uma sessão/contrato comum |
| 18 | LSIF | substituir | L10; avaliar SCIP atual, não iniciar implementação nova em LSIF |
| 19 | OpenOCD | adotar | L6, principal adapter JTAG/SWD/GDB remoto |
| 20 | pyOCD | adotar condicional | L6, complementar Cortex-M/CMSIS-DAP por cobertura de target |
| 21 | CMSIS-DAP | modelar | L6 como capacidade de probe, não plugin/UI |
| 22 | avrdude/esptool/stlink | adotar | L6, adapters distintos com argv/preview/versão/cancelamento |
| 23 | Cppcheck/Clang Static Analyzer | condicional | L2; no máximo um default além de clang-tidy, sem diagnósticos triplicados |
| 24 | cargo-audit/cargo-deny | adotar | L2; banco/rede separados da análise local e consentimento explícito |
| 25 | gcov/lcov | adotar | L2; perfil instrumentado explícito e resultado por arquivo/linha |
| 26 | cargo-tarpaulin | substituir primeiro | L2; avaliar `cargo-llvm-cov` primeiro; Tarpaulin fica fallback validado |
| 27 | Ghidra | referência/externo | L10; abrir/importar artefato, nunca embutir a suíte |
| 28 | GDB/LLDB MI | fallback | L4 depois de DAP, somente capacidade comprovadamente ausente |
| 29 | libelf/goblin | condicional | L4/L10; preferir crate Rust segura (`goblin`) se Bloaty/DAP não bastarem |
| 30 | DWARF/gimli | condicional | L4/L10; parsing lazy e limitado somente para contrato concreto |
| 31 | udevadm/libudev | adotar em degraus | L6; `udevadm`/sysfs primeiro, binding apenas se eventos persistentes exigirem |
| 32 | PlatformIO Storage Architecture | referência | L6; aprender pins/cache/isolamento, sem incorporar Core nem download silencioso |
| 33 | CMSIS-Pack | adotar | L6; parser de pacote não confiável, licença e paths validados, zero execução |
| 34 | DTS/DTB | adotar | L6; ferramentas oficiais e texto autoritativo com round-trip |
| 35 | SWO/ITM | adotar | L7 depois de Probe/Target e streaming com backpressure |
| 36 | CTF/LTTng | adotar condicional | L7 para Linux/RTOS, formatos maduros e retenção limitada |
| 37 | libclang/Clang C API | não agora | L10; somente caso medido que clangd + Tree-sitter não resolvam |
| 38 | gcov-kernel/kcov | laboratório | L10, privilégios/isolamento próprios e fora do fluxo padrão |
| 39 | MQTT/CoAP/Mosquitto | adotar opt-in | L7; cliente separado do broker, TLS/segredos e destino visível |
| 40 | TimescaleDB/InfluxDB/LMDB/RocksDB | escolher, não somar | L8 após benchmark de volume/retenção; servidor e embutido são decisões distintas |
| 41 | Jinja2/Tera | condicional | L6; preferir Tera no Rust, com preview/diff/ownership antes de gerar |
| 42 | OpenModelica/OMSimulator | adotar tarde | L9 depois do contrato FMI/FMU, como Job externo cancelável |
| 43 | FMI/FMUs | adotar fundação | L9; inspecionar/sandbox antes de carregar binário de FMU |
| 44 | GSL/ndarray | por feature | L8/L9; preferir `ndarray` em módulo Rust quando um visualizador provar necessidade |
| 45 | perf/Hotspot | adotar/referência | L4; `perf` coleta, Hotspot abre/importa externamente antes de UI própria |
| 46 | tokio-console | condicional | L4; só para binários instrumentados e com pré-requisito visível |
| 47 | Frama-C/Kani | laboratório | L10, Jobs separados e sem alegação genérica de prova de segurança |
| 48 | OTAWA/WCET | pesquisa | L10, somente após maturidade/arquitetura/modelo de hardware comprovados |
| 49 | lm-sensors/Open Hardware Monitor | substituir | L7; usar lm-sensors no Linux; não depender de Mono/WinForms do OHM |
| 50 | D-Bus | condicional de plataforma | L7, interfaces allowlisted; não aparece como plugin genérico |
| 51 | ccache/sccache | adotar | L3, ativação explícita por perfil e telemetria de hit/miss só local |
| 52 | distcc | laboratório | L10 depois de RemoteContext, confiança/toolchain/rede explícitas e benchmark |
| 53 | OpenGL/afins | subsistema, não plugin | L8; renderer isolado, API de dados limitada e compatível com o stack Qt |

#### A5.4 — correções trazidas pela auditoria atual

- [SSHFS](https://github.com/libfuse/sshfs) 3.7.6 tem release de 2026, mas o
  próprio projeto informa ausência de contribuidores ativos regulares e
  manutenção focada em problemas de alto impacto; por isso não é a base remota.
- [Wokwi CLI](https://docs.wokwi.com/wokwi-ci/cli-usage) usa token; a
  [arquitetura oficial](https://docs.wokwi.com/wokwi-ci/getting-started)
  executa a simulação em nuvem e recebe o firmware. Só pode ser opt-in com
  preview/envio explícito, nunca substituto local do QEMU.
- [Serial Studio](https://github.com/Serial-Studio/Serial-Studio) separa core
  GPL-3.0 e recursos Pro proprietários; é referência/interoperabilidade, não
  fonte de código nem componente redistribuído pela Kinein.
- Sourcegraph recomenda [SCIP no lugar de
  LSIF](https://sourcegraph.com/blog/announcing-scip), e em 2026 anunciou
  governança aberta para SCIP; qualquer índice persistente novo começa pela
  avaliação de SCIP.
- Para Rust, [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)
  trabalha sobre instrumentação LLVM, Cargo/nextest e múltiplas arquiteturas;
  o Tarpaulin ainda usa ptrace x86_64 como backend Linux padrão. Por isso
  `cargo-llvm-cov` é o primeiro PoC.
- Open Hardware Monitor exige Mono/WinForms no Linux; `lm-sensors` mantém a
  decisão Linux-first mais simples e nativa.

Esta triagem não dispensa o gate completo de licença, revisão, checksum,
segurança e manutenção imediatamente antes de cada adoção. Ela decide a ordem
e evita gastar arquitetura em candidatos já superados ou inadequados.

### A6 — sessão reservada para o novo repositório público

O usuário pretende abrir uma sessão própria em breve para organizar o projeto
inteiro antes de criar um **novo** repositório público no GitHub. `.gitignore`
não é barreira de publicação nem remove nomes/histórico já rastreados. Aplicar
`docs/roadmaps/21` M7.3: exportação por allowlist para árvore separada, histórico novo,
dry-run, rejeição de Markdown extra e auditoria de segredos. No início dessa
sessão, obter do usuário a lista exata dos nomes de arquivos `.md` e diretórios
que não podem aparecer nem como caminho. Até lá, não criar repositório, não
alterar visibilidade e não fazer push.

## 3. Consolidações necessárias antes de declarar substituição diária

| Frente | Consolidação pendente | Evidência de aceite |
| --- | --- | --- |
| Distribuição | ampliar matriz Ubuntu/Fedora, canal de release, atualização e diagnóstico de runtime | AppImage validado em distros-alvo e procedimento de release repetível |
| Entrada no trabalho | validar workspaces recentes e restauração previsível em uso prolongado | retomar projeto em um clique, sem sessão cruzada |
| Modelo de projeto | aprofundar capacidades já detectadas em Project Graph/Context Matrix | targets e contextos Cargo/CMake/Qt explicáveis por arquivo |
| Editor | resposta imediata local + semântica progressiva | cenários e latências medidos |
| Terminal | rajadas, scrollback e múltiplas sessões prolongadas | teste de estresse sem atraso ou perda de interação |
| Build/Run/Test | fluxos reais e cancelamento em projetos externos | processos encerram sem órfãos e resultados são navegáveis |
| Debug | sessão básica confiável e inspeção útil | breakpoint, step, pilha e variáveis em fixture real |
| Segurança de dados | soak tests de save, mudança externa, crash e drafts | nenhuma escrita silenciosa sobre snapshot antigo |
| Ergonomia | confortos puxados pelo dogfooding | nenhum editor auxiliar necessário por lacuna diária |

## 4. TR2 — primeiro patamar “um nível abaixo do CLion”

Implementar o KSWE em fatias pequenas, mantendo CMake/Cargo/clangd/
rust-analyzer como fontes autoritativas e sem criar uma engine genérica
paralela.

### B1 — Project Model autoritativo

- ampliar CMake File API para codemodel, targets, configurações, sources,
  compile groups, includes, defines e artefatos;
- consumir Cargo Metadata para packages, targets, features e workspace;
- normalizar ambos em snapshot versionado de Project Graph + Context Matrix;
- atualizar por geração, descartar resultado obsoleto e respeitar orçamento;
- criar fixtures CMake, Cargo e híbrida.

### B2 — targets, perfis e toolchains como entidades

- seleção explícita de target/configuração/toolchain;
- contexto efetivo por arquivo e target;
- presets CMake, perfis Cargo, sysroot e compile database rastreáveis;
- interface visual fiel às Configuration Actions das specs.

### B3 — inteligência semântica coordenada

- scheduler LSP com prioridade ao arquivo visível e cancelamento;
- Symbol Broker e Diagnostic Broker sem duplicar clangd/rust-analyzer;
- Effective Compile Context explicável ao usuário;
- caches limitados e invalidação por geração/fingerprint;
- fallback Tree-sitter continua instantâneo e independente do LSP.

### B4 — debug de IDE

- watches/expressões, variáveis, pilha, breakpoints condicionais e
  pretty-printers;
- GDB/LLDB via DAP, sem UI chamar debugger diretamente;
- sessões reais C/C++ e Rust com encerramento/cancelamento confiável.

Aceite do TR2: desenvolver a Kinein por uma semana sem abrir CLion por falta de
compreensão do projeto, build, navegação semântica ou debug básico.

## 5. TR3 — profundidade equiparável e embarcados

- múltiplos targets/contextos concorrentes e cache semântico persistente;
- correlação de símbolos, diagnósticos e refatorações mais profundas;
- toolchains cruzadas, sysroots e SDKs Yocto/Buildroot;
- flash, serial, QEMU, OpenOCD/pyOCD e GDB remoto;
- testes prolongados em projetos C, C++, Rust e embarcados reais;
- otimização de CPU/RAM/latência sem transformar indexação em trabalho eager.

## 6. Backlog complementar, depois das consolidações imediatas

- opção guiada **Outra IA**: abrir uma sessão de terminal dedicada e instruir
  o usuário a digitar o comando de inicialização da CLI que já instalou;
- biblioteca de funções / Configuration Actions para configurar CMake/Cargo e
  ambiente com preview, evidência e controle do usuário;
- exportador allowlist para qualquer cópia do código entregue a terceiros, com
  `--dry-run`, auditoria de segredos e recusa de Markdown além de `README.md`,
  `MANUAL.md` e `Tutorial.md`; sem `.git/`/histórico privado e com o
  repositório-fonte permanecendo privado;
- **preview de markdown renderizado ("modo imagem") no editor**, estilo
  JetBrains: alternar entre fonte e visualização renderizada de arquivos `.md`.
  Reusar `TextEdit.MarkdownText` (já validado no `DocumentationDialog`); fatia de
  UI com toggle por aba/atalho, sem editar o markdown pela visualização.
  Pedido do autor em 2026-07-16.
- **remake de UI/UX do shell, com o IntelliJ IDEA Community como layout-base.**
  Pedido do autor em 2026-07-16. Não é fatia pequena e não deve ser iniciada de
  improviso: exige spec antes de código, como a barra da janela (`docs/roadmaps/20`).

  Sintomas concretos já relatados:

  1. **A barra de "Target: host local", perfil, Compilar e Run não tem bordas
     arredondadas**, ao contrário dos demais painéis (o `SideRail` usa
     `Theme.radiusLarge`, a faixa da toolbar não). Inconsistência visível entre
     superfícies vizinhas.
  2. O conjunto App Bar + toolbar + rail + painéis cresceu por acréscimo e não
     por um sistema; falta uma regra única de raio, gap, elevação e densidade.

  Referência profissional autorizada: **IntelliJ IDEA Community** (política do
  `AGENTS.md`/roadmap de adaptação — estudar revisão atual, registrar
  invariantes, adaptar para Qt/QML **sem copiar** código ou tradução mecânica).
  Extrair o *layout base*: hierarquia tool window / editor / status bar, a faixa
  de ações, e como a densidade e o raio são sistematizados. Cruzar com
  `docs/specs/KINEIN_VECTIS_LAYOUT_SYSTEM.md` e
  `KINEIN_VECTIS_UI_COMPONENTS_SYSTEM.md`, que já são a visão-alvo — o remake
  deve convergir para as specs, não inventar uma terceira direção.

  Imagem de referência do autor para a barra: `imagens/bugs/ReformularBarra.png`.
  O polimento P3 da barra da janela (`docs/roadmaps/20`: snap, escala
  fracionária, multimonitor) pertence a esta mesma frente.

  Não bloqueia L0/L1. Entra quando o autor priorizar.
- **integrar os pacotes de ícones ao app (não é fatia de packaging).** Pedido do
  autor em 2026-07-16: "os ícones atuais e os novos devem estar no AppImage".

  Auditoria do estado real: **isso já é automático e não pode ser esquecido.**
  Ícones entram pelo `RESOURCES` do `qt_add_qml_module` em `ui/CMakeLists.txt`,
  são compilados no executável, e o `empacotar-appimage.sh` empacota o
  executável. Não existe passo separado de copiar ícone para o AppImage — o que
  está no `RESOURCES` está no AppImage por construção.

  O gap real é outro: **de 163 SVGs em `docs/iconografia/`, apenas 5 estão no
  app** (`ui/assets/icons/tree/`: folder-closed/open, file-c/cpp/rust). O pacote
  de ícones de arquivos especiais foi commitado em `docs/`, que é design, não
  asset. Então a fatia é de **UI**, não de empacotamento:

  1. escolher quais famílias entram (o índice `docs/iconografia/README.md` diz
     qual é a fonte de verdade de cada uma) e copiar os SVGs para `ui/assets/`
     preservando os bytes;
  2. declarar em `RESOURCES` no `ui/CMakeLists.txt`;
  3. mapear extensão/tipo → ícone (existe `FILE_ICON_MAPPINGS.json` no pacote de
     arquivos especiais) e consumir na árvore;
  4. o AppImage passa a carregá-los sem nenhuma mudança de packaging; validar com
     `testar-appimage.sh` e `testar-appimage-portatil.sh`.

  Regra permanente: ícone novo entra em `RESOURCES` no mesmo commit em que entra
  na UI. Se está no `RESOURCES`, está no AppImage.
- **teste Rust intermitente sob disputa de recurso (P3, observado 2026-07-16).**
  `cargo test --workspace` falhou UMA vez (exit 101) logo após rodar
  `medir-performance.sh`, e passou na repetição sem mudança de código. A
  medição sobe vários cores, terminais PTY e servidores LSP; a suspeita é
  disputa de recurso (teto de sessões, PTY, memória), não defeito lógico.
  Não investiguei — registrado para não sumir. Se reaparecer sem a medição ao
  lado, vira P2: teste que passa na segunda tentativa é gate que não protege.
- **varredura de lógica de negócio na UI/UX — EXECUTADA em 2026-07-16.**
  Pedido do autor após a fatia da roda. Os dois defeitos daquele dia (`handleWheel`
  decidindo o gesto e o `Column` decidindo a geometria da grade) foram o **mesmo
  erro duas vezes**: a UI decidindo o que é do backend. A varredura procurou por
  (a) tradução de gesto/tecla em semântica de domínio, (b) política/validação que
  o core deveria impor, (c) heurística sobre estado do backend, (d) montagem de
  comando/caminho e (e) regra por programa/ferramenta.

  **Achado 1 — RESOLVIDO em 2026-07-16 (protocolo 0.61.0).** Era: a UI decidia
  o que é formatável, duplicando o core.
  `ui/qml/editor/EditorController.qml`:

  ```qml
  function formattableLanguage() {          // linha 310
      return language === "rust" || language === "cpp";
  }
  function formattablePath(path) {          // linha 327
      return lower.endsWith(".rs") || lower.endsWith(".c") || ... // 9 extensões
  }
  ```

  O core **já é a autoridade**: `format::formatter_for_path()` decide e o handler
  recusa com `InvalidParams` ("nenhum formatter registrado para esta extensão").
  A UI mantinha uma segunda lista, escrita à mão, que divergia por construção:
  `formattableLanguage` conhecia 2 linguagens, `formattablePath` conhecia 9
  extensões, e as duas nem concordavam entre si. Adicionar linguagem exigia
  editar QML.

  Correção: `format.capabilities` publica o catálogo, derivado da MESMA
  constante (`FORMATTER_EXTENSIONS`) que `formatter_for_path` usa para decidir —
  o que a UI recebe é, por construção, o que o `format.text` aceita. A UI perdeu
  as duas listas e passou a ter uma pergunta só, respondida pelo core, pedida uma
  vez por conexão (o catálogo é estático). Capacidade e disponibilidade seguem
  distintas: binário ausente no `PATH` continua sendo `TOOL_NOT_FOUND` do
  `format.text`.

  Cobertura: 3 testes Rust (catálogo publicado == decisão real, extensão fora do
  catálogo, sem sobreposição entre formatters) e
  `scripts/qml-harness/tst_format_capabilities.qml` — que falha se alguém
  reintroduzir lista literal no QML, porque testa que **nada** é formatável antes
  de o catálogo chegar.

  **Achado 2 — P3, latente: encoding VT mora na UI.**
  `TerminalInputController.qml` traduz tecla em bytes (`\x1b[A` vs `\x1bOA`
  conforme `applicationCursor`) e `TerminalPanel.qml` embrulha o paste
  (`\x1b[200~`) conforme `bracketedPaste`. É o **mesmo formato** do bug da roda —
  encoding VT no frontend —, mas **funciona**, porque ao contrário do
  `handleWheel` estas duas LEEM o modo que o core publica. É escolha registrada
  ("a UI continua burra em relação ao TUI: apenas respeita o estado VT"). Fica
  como dívida consciente: qualquer modo novo que afete encoding (kitty keyboard
  protocol, `modifyOtherKeys`) vai exigir mudança na UI em vez de só no core.
  Não corrigir sem necessidade concreta.

  **Achado 3 — P3, trivial: `alternateScreen` é código morto.**
  `TerminalPanel.qml:41` expõe a propriedade e **ninguém a consome** desde que a
  decisão da roda foi para o core. Era o dado que o `handleWheel` ignorava.

  **Achado 4 — P3: `assistantTerminalWidth` órfã** em `SettingsController.qml`,
  `settings.rs` e no schema — resto do painel do assistente removido no 0.59.0.
  Ver §0.2c.

  **Limpo:** não há montagem de comando/caminho de ferramenta na UI (o `+ "/"` do
  `EditorDocumentController`/`ProjectTreeController` é composição de caminho para
  exibição, sobre dados que o core já confinou); não há allowlist nem validação de
  segurança no frontend; não há heurística adivinhando estado do backend; os
  nomes de ferramenta no QML (`"cargo"`, `"cmake"`, `"git"`) são rótulo, chave de
  aba ou parâmetro repassado ao core — não decisão.

  Conclusão: a camada está mais saudável do que os dois defeitos do dia sugeriam.
  O padrão perigoso é específico — **UI decidindo semântica de terminal/ferramenta
  em vez de ler o contrato** — e sobrou concentrado no Achado 1. Cada achado vira
  fatia própria; a varredura não mudou comportamento.
- **FEITO (aguarda seu gesto visual) — emulador VT migrado para
  `alacritty_terminal`** (`docs/adr/ADR-0004-alacritty-terminal-emulator.md`).
  Causa-raiz era entregar um VT raso (`vt100`) enquanto anunciávamos
  `xterm-256color` — por isso Claude/Codex funcionam fora e quebravam aqui.
  Agora o motor é o mesmo do Alacritty/Zed. Contrato `event.terminal.render`
  **inalterado**: UI, harnesses e sonda não mudaram. 261 testes verdes, clippy
  estrito limpo, sonda e2e verde (scrollbackMax, eco, clamp, multi-sessão).
  `vt100`/`vte` saíram das deps; o `CursorStyleTracker` manual e o
  `scrollback_capacity` foram deletados (o emulador expõe os dois). **Teste com
  Claude/Codex reais e me diga se o cursor/scroll ficaram certos.**
- Contexto da decisão (`docs/roadmaps/26` §4.5). Pedido real do autor: "meu
  terminal inteiro dentro da IDE". Claude/Codex
  funcionam no terminal do sistema e só quebram na Kinein, logo a variável é o
  emulador. A crate `vt100` é rasa; o candidato é **`alacritty_terminal`** (Rust,
  Apache-2.0, mantido, mesmo ecossistema do `vte` já linkado — é o que o Zed usa;
  Code OSS usa xterm.js e IntelliJ usa JediTerm pelo mesmo motivo). Substitui só
  o motor de grade/estado VT, preservando `portable-pty`, IPC tipado e renderer
  Qt/QML. Exige ADR, auditoria/pin, mapeamento de `event.terminal.render` e
  testes de paridade. Deve vir ANTES da decisão de renderer (R3) e tende a
  resolver de uma vez cursor flutuante, tela alternada, mouse e scroll de TUIs.
- **ACEITO em 2026-07-16 — cursor/TUI corrigido. Causa-raiz: linha vazia
  colapsava no positioner.** O `Column` do Qt Quick descarta filhos de largura
  zero; uma linha vazia chega do core como `[]` e vira um `Row` sem filhos, logo
  sem largura. Cada linha vazia sumia do layout e todo o texto abaixo subia uma
  linha, enquanto o cursor ficava na posição correta da grade. **O cursor nunca
  esteve errado — o texto escorregava para cima.** Correção: a linha é
  posicionada por `metrics.yForRow(index)`, a mesma métrica do cursor; o
  `Column` saiu. Detalhes e evidência em `docs/roadmaps/26` §4.7. O usuário
  testou e aprovou. A pendência de aceite visual do ADR-0004 está encerrada.

  O que destravou foi a **captura de tela** — item 3 do R0, tratado como
  opcional por três fatias. As medições numéricas (R0 §4.6) e o R1 confirmaram
  hipóteses reais (H1, H2) que não eram a causa: o erro era de linha inteira, não
  de sub-pixel. Lição registrada no §4.7 do roadmap.
- **FEITO (aguarda seu gesto) — roda do mouse encaminhada ao aplicativo**
  (fatia R4, protocolo `0.60.0`, `docs/roadmaps/26` §11.4 Opção A). A causa não
  era interferência: era **ausência**. O `open_command` não tem política por
  programa nenhuma; o que faltava é que a UI decidia sozinha que roda = rolar
  histórico, sempre. O Codex rolava (desenha inline, tem histórico na grade) e o
  Claude não (alt-screen, sem histórico por semântica VT).

  Medido nas CLIs reais, não suposto: `claude` liga `?1049 ?1000 ?1002 ?1003
  ?1006` — alt-screen **e captura de mouse SGR**; `codex` não liga nenhum. Logo o
  conserto do Claude é **relatório de mouse**, não `alternate scroll` (ele não
  pede `?1007`). O desenho que eu ia escrever primeiro teria sido inerte.

  `terminal.mouse` carrega o contrato de mouse completo; só `wheel` está
  implementado, `press`/`release`/`motion` ficam fixados e recusados
  explicitamente (R5). A decisão virou `wheel_action(mode, shift)` no core — a
  regra que estava no QML, agora num lugar só. Referências: `scroll_wheel` do
  Zed, `MouseStateService` do xterm.js, `JediTerminal.mouseReport` do JediTerm
  (cujas constantes `SCROLLDOWN/SCROLLUP` estão trocadas e foram descartadas).

  Gate: `scripts/verificar.sh` integral verde, 264 testes Rust (10 novos),
  clippy/C++/QML estritos, 11 harnesses e smoke offscreen (`exit 124`, sem
  saída). **Falta o seu gesto:** abrir o Claude no terminal e rolar.
- **cursor/TUI continua torto e NÃO é consertado pela roda.** Reportado em
  2026-07-16 nos dois agentes, já com o emulador novo — o que fecha a hipótese
  de que a profundidade do emulador era a causa. É renderer (R0–R3 de
  `docs/roadmaps/26`: métrica de célula, baseline, DPR), fatia própria. O
  mapeamento no core foi conferido e está correto.
- Windows é uma frente futura separada; não diluir o objetivo Linux-first atual.

## 7. Gate e definição de pronto de cada fatia

1. Ler `GUIAIA.md` e as fontes do domínio antes de editar.
2. Para funcionalidade de IDE, cumprir a referência profissional obrigatória:
   fonte oficial atual, revisão/invariantes registradas e adaptação própria sem
   cópia, conforme a seção 2.1 do roadmap de adaptação.
3. Manter UI → CoreClient → protocolo → handler → serviço; UI não chama
   ferramenta externa ou filesystem de workspace diretamente.
4. Operação longa vira Job cancelável e nunca bloqueia a UI.
5. Criar teste de core e harness QML quando houver estado visual.
6. Executar `bash scripts/verificar.sh` e a sonda específica do domínio.
7. Para UI/layout, validar o gesto em tela real; para packaging, executar
   `scripts/testar-appimage.sh` e `scripts/testar-appimage-portatil.sh`.
8. Atualizar contrato, schema, manual e arquitetura quando afetados.
9. Registrar conclusão em `ContextoIA.md` e no doc do domínio; remover o item
   concluído deste arquivo, sem manter listas riscadas ou post-mortems aqui.

## 8. Onde ficou o histórico concluído

Este é apenas um mapa para evitar duplicação:

- estado técnico, decisões e checkpoints: `ContextoIA.md`;
- rede de segurança, drafts e escrita atômica: `docs/seguranca/23-rede-de-seguranca.md`;
- autocomplete, terminal, paridade diária e UI: `docs/roadmaps/24-paridade-e-fundacao.md`;
- Tree-sitter e workspace edits: `docs/roadmaps/25-syntax-tree-semantic-foundation.md`;
- plano de daily driver e longo prazo: `docs/diario/18-daily-driver-plan.md` e
  `docs/roadmaps/21-long-horizon-roadmap.md`;
- apresentação pública: `README.md`; uso da IDE: `MANUAL.md`; distribuição e
  instalação: `Tutorial.md`;
- alterações exatas e checkpoints: histórico Git.
