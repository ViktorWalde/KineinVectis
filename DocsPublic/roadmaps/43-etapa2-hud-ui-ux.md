# 43 — Etapa 2: HUD, UI e UX — o desenho medido antes do código

> Escrito em 2026-09-18, depois do pente-fino (`40` §7.54), por decisão do
> autor: *"otimização de HUD/UI/UX da IDE; ver o sucesso das IDEs JetBrains
> e adaptar todo o efeito psicológico positivo, visual JetBrains-like
> adaptado ao contexto do projeto"*. Este documento é o DESENHO. Nada dele
> vira código antes de o autor aprovar a lista da §5; cada item da lista
> tem uma medida que diz se ficou pronto.

## 1. O método: o que se copia e o que não se copia

**Não se copia** tema, ícone, paleta nem código da JetBrains (licença e
identidade: a IDE tem `Theme.qml`, `iconografia/`, `KvButton`/`KvToggleChip`
próprios). **Copia-se o comportamento observável que produz o efeito** — a
sensação de que a janela *sabe* do projeto e responde na hora — e ele é
medível. A referência foi lida nas fontes (a documentação do New UI e do
tema Islands da própria JetBrains, e a literatura de UX que sustenta cada
princípio), não em capturas de tela:

```text
Princípio                     Fonte (2026-09-18)                          O que se mede aqui
reduzir complexidade visual,  JetBrains, "New UI" (help/idea/new-ui):     itens visíveis por superfície;
mostrar o essencial, revelar  "reduce visual complexity, provide easy     cliques ate' a acao comum
o resto progressivamente      access to essential features, progressively
                              disclose complex functionality"
uma barra principal com 3     idem: Project widget, VCS widget, Run        a barra superior de hoje tem
widgets (projeto, VCS, run)   widget — e' TUDO que fica sempre a vista     12 controles (foto 02)
separacao clara de areas,     JetBrains, "Islands" (2025-12): "clear      contraste editor/paineis;
cantos, respiro; aba ativa    separation between working areas", "the      a aba ativa distinguivel a
obvia                         active tab is very obvious"                  1 m de distancia
status bar como o lugar do    JetBrains SDK, "Status Bar Widgets": "only   o que a barra diz durante um
que e' relevante SEMPRE       information or settings relevant enough to   build/teste/indexacao
                              be always shown"
resposta em < 400 ms mantem   Doherty (IBM, 1982), Nielsen: 0-100 ms       tempo do clique ao primeiro
o fluxo; acima disso, sinal   instantaneo, ate' 400 ms fluido, 1 s+ exige  feedback, por acao (medido
de progresso                  progresso                                    no binario, com marcador)
progressive disclosure reduz  Nielsen (1995); Sweller (carga extrinseca)   opcoes avancadas a 1 clique,
carga EXTRINSECA              — uxpin/ixdf/lawsofux                        nunca escondidas a 3
minimalismo + feedback        Zed: "8 ms por frame; toda interacao produz  o mesmo criterio de 400 ms,
instantaneo como identidade   feedback visual instantaneo"                 e 60 fps no scroll do editor
```

## 2. O que a IDE mostra HOJE — medido na IDE abrindo (2026-09-18)

Medido com o hook novo desta sessão: `kinein-vectis <pasta>` abre o projeto
direto e `KINEIN_SCREENSHOT=<png>` fotografa a janela headless (1280×800,
offscreen). As três fotos estão em `imagens/prints/2026-09-18-etapa2/`.

**Foto 01 — tela inicial.** Título "Kinein Vectis / IDE para C, C++, Rust,
Python e sistemas embarcados"; um cartão "Começar" com quatro botões (o
primeiro amarelo); "Workspaces recentes" com **três entradas `/tmp/…` —
caminho ausente** (lixo dos gates que abriam pastas temporárias no config
do autor; corrigido nesta sessão: os gates isolam `XDG_CONFIG_HOME`);
"Estado do ambiente: 37 de 62 ferramentas detectadas" com a lista inteira.
Barra de status só com "IDE" e "core conectado · IPC 0.122.0". Juízo: a tela
inicial já é informativa; a lista de 62 ferramentas na primeira dobra é
carga extrínseca — o número e um "ver" bastam.

**Foto 02 — workspace aberto (este repositório).** Menu de 10 itens;
**barra superior com 12 controles**: `Target: host local`, `Cargo: debug`,
um triângulo sem rótulo, `Cargo` (amarelo, primário), um ✓ sem rótulo,
`CMake`, ✓, ⚠, ✗, ▶ verde; à direita "● Cargo + CMake". Trilho lateral
com **10 ícones sem rótulo**. Explorer com `.cargo .git .idea .kinein
.ruff_cache build dist target` no mesmo peso que `crates` e `ui`. Faixa
"Project Health: configurando o projeto CMake automaticamente…" com
botão Jobs. Editor vazio com "Clique em um arquivo no explorer para abrir".
**Barra de status colidindo**: "3 alterações" por cima de "IDE" (defeito de
layout; corrigido nesta sessão — a faixa esquerda para antes da direita e
o git vem antes dos números do índice, que são os que cedem). Juízo: os
dois sistemas de build lado a lado com ícones iguais e sem rótulo são a
maior fonte de dúvida ("qual ✓ testa o quê?"); o Run widget da referência
resolve isso com UMA configuração escolhida e um só botão.

**Foto 03 — editor.** Abas (`Main.qml`, `mod.rs` ativa com ✕), migalhas
(`crates › kinein-core › src › remote › mod.rs`), numeração, cores de
sintaxe, um botão **"Salvar" amarelo permanente** mesmo sem alteração;
sem realce visível da linha atual; o explorer NÃO destaca o arquivo
aberto. Juízo: o botão primário mais chamativo da tela é uma ação que a
referência nem tem (autosave); a aba ativa distingue-se pouco da inativa.

**O que NÃO se mediu ainda (precisa da IDE em uso):** os painéis de baixo
(Build, Problems, Testes, Jobs, Terminal, Git) durante um build real; os
diálogos de ambiente; o tempo do clique ao primeiro feedback por ação. A
§4 diz como medir cada um antes de mexer.

## 3. O que já existe e é a base (não se reescreve)

`Theme.qml` (tokens de cor/espaço/fonte), `KvButton`/`KvToggleChip`/
`KvIconButton`/`KvTooltip`, `iconografia/` (a decisão dos ícones),
`arquitetura/32` (o editor por responsabilidade), a faixa de saúde
(`ProjectHealthController`), a `WorkspaceStatusBar`, o `SideRail`, o painel
Jobs (todo job com risco, cancelável), a paleta de comandos (`command.list`
com atalho verificado pelo gate), a lâmpada do Alt+Enter. A Etapa 2
**rearranja e afina** isto; não cria uma segunda casca.

## 4. As fatias, em ordem, cada uma com a medida

```text
F0  infra de medicao (FEITA em 2026-09-18)
    kinein-vectis <pasta>; KINEIN_SCREENSHOT; a status bar sem colisao; os gates
    sem poluir os recentes.
    Medida: a foto sai do gate; o autor ve o mesmo que o agente.

F1  a barra superior vira TRES widgets (projeto · git · executar)
    Projeto: nome + kind + menu de recentes. Git: branch ↑↓ + alteracoes, clique =
    painel Git. Executar: UMA configuracao ativa (runConfig.setActive ja' existe) com
    ▶ Rodar, 🐞 Depurar, e o menu "…" com Build/Testes/Analise dos sistemas presentes.
    O que sai da barra: os 12 controles de hoje; Cargo/CMake viram opcoes do menu
    do widget, com rotulo. O trilho lateral ganha rotulo ao pairar (KvTooltip) e o
    modo compacto/expandido.
    Medida: controles sempre visiveis na barra <= 6; toda acao de build/teste a
    <= 2 cliques; harness do RunWidgetController; foto antes/depois.

F2  a barra de status diz O QUE ESTA ACONTECENDO
    Esquerda: workspace · branch. Centro: o job em curso com progresso e
    cancelar (build 12 s, indexando 1.451 arquivos, configurando CMake, rsync
    para pi) — o JobsController ja' sabe. Direita: contexto do arquivo ativo
    (linguagem · linha:coluna · encoding), LSP (● clangd), core.
    Medida: durante build/teste/indexacao/deploy a barra mostra o job e o
    progresso; nada colide a 1024 px (foto no gate a 1024 e 1280).

F3  aba ativa, linha atual, arquivo aberto no explorer, "Salvar" some
    Aba ativa com borda de acento e fundo do editor (Islands: "obvious");
    realce da linha atual; o explorer segue o arquivo ativo (autoscroll from
    source, como a referencia); o botao Salvar vira indicador "●" na aba +
    Ctrl+S (o autosave e' decisao a parte: `40` §5 nao proibe; propor em F6).
    Medida: foto; harness do EditorController para "arquivo ativo -> explorer".

F4  explorer: o que e' do autor pesa mais que o que e' da maquina
    `.git .idea .kinein .ruff_cache build target dist` em cinza e no fim (a
    referencia marca "excluded"); pastas do projeto primeiro; o kind do
    workspace e' um chip discreto, nao um botao amarelo.
    Medida: foto; a ordem e' regra pura com teste (harness do ProjectTree).

F5  Problems com o proximo passo, e o painel de baixo com identidade
    Cada problema: arquivo:linha, a mensagem, e a ACAO (a lampada quando ha
    code action; "configurar" quando falta CDB; "instalar X" quando falta
    ferramenta). Abas de baixo com contagem (Problems 3, Testes 12/14).
    Medida: harness; foto com um build que falha.

F6  resposta ao gesto: < 400 ms ou progresso
    Medir por acao (clique -> primeiro feedback) com KINEIN_PERF no binario:
    abrir arquivo, trocar aba, abrir painel, Ctrl+P, salvar. O que passar de
    100 ms ganha feedback imediato (estado pressionado, esqueleto); o que
    passar de 400 ms ganha progresso. Autosave (com o rascunho do seguranca/23
    ja' existente) e' proposto AQUI, para o autor decidir.
    Medida: tabela de tempos antes/depois no roadmap 40.

F7  tela inicial: comecar em 1 clique, ambiente em 1 linha
    Recentes com o ultimo em destaque (Enter abre); "37 de 62 ferramentas" com
    "ver" que abre o painel; "caminho ausente" some da lista (com desfazer).
    Medida: foto; harness do RecentWorkspacesController.

F8  paineis de ambiente (banco, remoto, embarcados, containers): mesma forma
    Cabecalho igual (titulo, subtitulo de uma linha, acao primaria a direita),
    veredito igual (a faixa verde/vermelha), grade igual (a do banco vira o
    componente comum). E' a "experiencia a DataGrip adaptada" do 40 §4.
    Medida: um componente de grade; harness; fotos dos quatro paineis.
```

## 5. O que se pede ao autor aprovar — e o que ele decidiu

**Decidido em 2026-09-18 ("prossiga"):** a ordem F1 → F8 como está;
**autosave sim** (a decisão entra na F3/F6, com o rascunho de
`seguranca/23` como rede); **um widget Executar com menu** (não dois grupos
rotulados). **F1 FEITA no mesmo dia** (`40` §7.56): a barra passou de doze
controles a três widgets — foto 04. **F2 FEITA** (`40` §7.57): o job em
curso com progresso e cancelar no centro da status bar, os servidores de
linguagem à direita — foto 05. **F3 FEITA** (`40` §7.58): aba ativa,
linha atual, explorer segue o arquivo, ● no lugar do Salvar, autosave —
foto 06. **F4 FEITA** (`40` §7.59, pedido direto do autor): o chip "Cargo + CMake"
saiu do explorer (já mora no widget de projeto da barra), linhas mais
densas (22 px, ícone 16, recuo 12), as pastas da máquina (`.git`, `build`,
`target`, `.idea`, `.kinein`…) em cinza e depois das do autor — foto 07.
**A F3 mediu o defeito que manda na F6:** o core BLOQUEIA o laço
inteiro até 4 s por pedido LSP (15 s no `initialize`) — com o
rust-analyzer indexando este repositório, nenhum `fs.list` foi respondido
por ~20 s e a IDE inteira ficou parada; a F6 começa por isso (resposta
assíncrona do LSP), não por medir cliques. **F5 FEITA** (`40` §7.61): o próximo passo em cada problema (Ações /
Configurar CMake / Ferramentas), regra pura `ProblemNextStep`; foto 09 com
um build que falha. **F6-b FEITA** (`40` §7.62): `tools.detect` adiado e o `workspace.open`
de 1,4 s → 25 ms; a tabela de tempos; Problems sem repetição build/LSP; a
toolchain da status bar segue o projeto — foto 10. **F7 FEITA** (`40` §7.63): o último
recente em destaque com Enter, os de caminho ausente ocultos com desfazer, o
ambiente numa linha com "Ver" — fotos 11a/11; de quebra, o `serial.identify`
que não saía e o gate que passa a ler o stderr do QML. **F8 FEITA** (`40` §7.65): os quatro painéis de ambiente com a mesma
moldura, a mesma primeira linha, o mesmo veredito e a grade comum — fotos
13a–13d; o Embarcados deixou de vazar. **A Etapa 2 fechou o desenho da
§4 (F0–F8) em 2026-09-18.** **F6-a FEITA** (`40` §7.60): as
consultas LSP respondem fora do laço, o handshake corre numa thread, e o
`syntaxTree.update` caiu de ~0,9 s para 0,05 s (o `utf16_position` varria o
arquivo inteiro a cada realce). A cadeia do explorer que levava 30 s
completa em 6 s; nada fica mais de ~300 ms sem resposta durante a
abertura de um arquivo Rust neste repositório.



A ordem F1 → F8 (F0 feita). Cada fatia: desenho de uma página no registro
Codex do dia, foto antes/depois, harness quando há lógica, catraca (view
300 / controller 400), `verificar-fiacao-ipc` (toda ponta ligada), o
binário abrindo, e **o teste prático do autor na tela** antes do commit da
fatia seguinte. Duas decisões ficam explicitamente para o autor: autosave
(F3/F6) e o destino do botão "Cargo/CMake" duplo (F1: um Run widget com
menu, ou dois grupos com rótulo).

## 6. Fontes lidas em 2026-09-18

- JetBrains, *New UI* (documentação do IntelliJ IDEA): widgets de projeto,
  VCS e run; tool windows; modo compacto; a frase da motivação.
- JetBrains Blog, *Meet the Islands Theme* (2025-12): separação de áreas,
  cantos, aba ativa, "easier on the eyes", alinhamento com macOS/Windows 11.
- JetBrains Platform SDK, *Status Bar Widgets*: o critério "relevante o
  bastante para estar sempre à vista".
- Laws of UX / LogRocket, *Doherty Threshold*: 400 ms; a escala 100 ms /
  400 ms / 1 s / 10 s de Nielsen.
- UXPin / IxDF, *Progressive disclosure*: Nielsen 1995; carga extrínseca de
  Sweller; os perigos (esconder demais).
- Zed (site e cobertura): o orçamento de 8 ms por frame e "toda interação
  produz feedback visual instantâneo" como identidade de produto.

## 7. Estado ao fim de 2026-09-18 — F0 a F7 feitas, F8 aberta

Escrito antes da F8, a pedido do autor. Uma linha por fatia: o que entrou,
como foi medido, a foto, e onde está o registro (o `40` §7.N tem o
detalhe; o `DocsPrivate/Codex/2026-09-18-etapa2-*.md` tem as provas).

```text
fatia  o que entrou                                        medida / foto            40 §
F0     kinein-vectis <pasta>; KINEIN_SCREENSHOT(+_DELAY_MS); fotos 01-03            7.55
       status bar sem colisao; gates com XDG_CONFIG_HOME
       isolado (nao poluem mais os recentes do autor)
F1     barra principal: 12 controles -> 3 widgets            foto 04; harnesses      7.56
       (HeaderProjectWidget · HeaderGitWidget · HeaderRun-   dos menus (AppMenuItems)
       Widget com UMA config ativa e menu "…"); rotulo ao
       pairar no trilho (KvTooltip)
F2     status bar: esquerda workspace·branch; centro o job    foto 05;                7.57
       em curso com progresso e cancelar (StatusBarJob-      tst_status_bar_state
       Widget/ActiveJobController); direita LSP (LspStatus-
       Controller) e core
F3     aba ativa com acento; linha atual (EditorGutter);     foto 06/06b;            7.58
       explorer segue o arquivo (ProjectTreeReveal-          tst_editor_autosave,
       Controller); "Salvar" vira ● + Ctrl+S; AUTOSAVE       tst_project_tree_reveal
       (settings.autoSave, 2 s, flush ao trocar aba/foco)   0.123.0
F4     explorer: pastas da maquina em cinza e no fim         foto 07;                7.59
       (ProjectTreeRules); linhas 22 px; o chip "Cargo +    tst_project_tree_rules
       CMake" saiu (ja' mora no widget de projeto)
F6-a   o core NAO para mais pelo LSP: respostas adiadas      foto 08 (explorer aos   7.60
       (RequestOutcome::Deferred, defer_lsp), handshake em   6 s, antes 30 s);
       thread (HANDSHAKE_GRACE 300 ms), LineIndex            tests/lsp_deferred.rs;
       (syntaxTree.update 0,9 s -> 0,05 s)                   <= 300 ms por pedido
F5     Problems com o proximo passo (ProblemRules.stepFor:   foto 09;                7.61
       Acoes / Configurar CMake / Ferramentas);              tst_problem_rules;
       KINEIN_STARTUP_COMMANDS (ids da paleta ao abrir)     StartupCommands.qml
F6-b   workspace.open 1,4 s -> 25 ms (presenca sem           foto 10; tabela de      7.62
       --version); tools.detect adiado (defer_work);         tempos antes/depois
       Problems sem repeticao build/LSP (isDuplicate);
       toolchain da status bar pelo kind do projeto
F7     tela inicial: ultimo recente em destaque, Enter       fotos 11a/11;           7.63
       abre; ausentes ocultos com Desfazer; ambiente em      tst_recent_workspaces;
       1 linha com "Ver". De quebra: o serial.identify que   verificar-binario-abre
       nao saia (Connections no target errado) e o gate     le o stderr do QML
       binario-abre lendo o stderr do QML
F8     paineis de ambiente com a mesma forma: KvPanelFrame  fotos 12a-d -> 13a-d;    7.65
       (rola), KvPanelHeader (acao primaria), KvVerdict,    tst_grid_rules; os
       KvDataGrid+GridRules; o Embarcados cabe; de quebra,  hosts 60-80 -> 25-56
       o elo solto do despacho C++ (7.64)                   linhas
```

**Os juízos da §2, um a um.** Foto 01 (tela inicial): "a lista de 62
ferramentas é carga extrínseca" → F7. Foto 02 (workspace): "12 controles,
dois sistemas de build com ícones iguais" → F1; "10 ícones sem rótulo" →
rótulo ao pairar (F1), o modo expandido NÃO; "pastas da máquina no mesmo
peso" → F4; "status bar colidindo" → F0; "Project Health configurando…"
ficou como está (é a faixa de saúde, base declarada em §3). Foto 03
(editor): "Salvar amarelo permanente" → F3 (● + autosave); "sem realce da
linha atual" → F3; "explorer não destaca o arquivo" → F3; "aba ativa
distingue-se pouco" → F3. **O que NÃO se mediu** (§2, último parágrafo):
os painéis de baixo durante um build real → F5 mediu Problems com um build
que falha (foto 09); os diálogos de ambiente → F8; o tempo do clique →
F6-b mediu pelo core e pelo primeiro frame — o clique real fica com o
autor.

**Os números da etapa (debug, esta máquina, 2026-09-18):**

```text
                                        manha (F0)      noite (F7)
controles sempre visiveis na barra      12              3 widgets (<= 6 pedidos por F1)
lsp.* com o rust-analyzer subindo       ate' 20 s mudo  <= 300 ms
explorer segue o arquivo (cadeia)       30 s            6 s
syntaxTree.update (470 linhas)          0,9 s           0,05 s
workspace.open                          1,4 s           25 ms
primeiro frame offscreen                nao medido      718-840 ms (debug; o hardened deu 318 ms no §7.54)
harnesses QML                           45              51
testes Rust                             827             829
```

**O que fecha a etapa** está no `40` §4.2.2: a F8, a sincronização final
com fotos lado a lado, o release-hardened remedido, e as dívidas pequenas
ditas em cada "não feito" (Ln:Col na status bar; contagem na aba Testes;
foto a 1024 px; trilho expandido; o foco da tela inicial contra o
terminal). E o que só o autor mede: a sensação de resposta com mouse e
teclado reais.

## 8. F8 — o desenho medido (2026-09-18, noite; escrito ANTES do código)

### 8.1 O que os quatro painéis mostram hoje — fotos 12a–12d

Tiradas com `KINEIN_STARTUP_COMMANDS=datasource.list | remote.list |
probe.list | container.list`, este repositório aberto, `XDG_CONFIG_HOME`
isolado (nenhum perfil salvo), 8 s. **A primeira rodada de fotos achou um
defeito antes de mostrar qualquer desenho:** Containers e Embarcados
ficavam em "procurando…" para sempre (foto 12e, aos 15 s) porque a cadeia
de despacho da ponte C++ estava solta desde 2026-09-12 — oito domínios sem
resposta na tela (`40` §7.64; corrigido, e o gate de fiação ganhou a
quinta pergunta). As fotos 12a–12d são DEPOIS do conserto: o estado real
dos painéis.

**12a Banco de dados** (720×560). Título + uma linha de subtítulo; à
esquerda a lista de perfis ("Nenhuma fonte salva. Preencha ao lado e
salve." + botão "Nova fonte" no pé); à direita o formulário (motor em três
chips, nome, host, porta, banco, usuário, origem da senha em três chips,
TLS em dois chips, a consulta); rodapé com cinco botões (Remover · Salvar ·
Testar · Ler estrutura · Fechar) — todos cinza, **nenhum primário**. O
formulário **corta embaixo** (o campo "Consulta" e o botão "Executar" ficam
meio fora da moldura a 560 px). O veredito do teste (verde/vermelho) só
aparece depois de testar.

**12b Alvo remoto** (680×520). Mesma forma do banco (lista à esquerda,
formulário à direita) — é o painel mais recente e copiou o do banco.
Rodapé com Remover · Salvar · Sondar · Fechar, mais **cinco botões de ação**
no corpo (Enviar, Rodar em…, gdbserver → kit, debugpy → config, Shell no
terminal), todos desabilitados sem alvo. Nenhum primário.

**12c Embarcados** (560×até 780). Um painel de UMA coluna com **nove
seções empilhadas** (Projeto, Sonda com a saída crua do probe-rs em caixa,
Portas seriais + Permissões, Gravar com quatro chips + Prévia, Alvo do kit
com os presets, Chip/Alvo/Sysroot/SVD, Tamanho do binário, Depurador do
kit, Toolchains instaláveis, Sysroot e SDK). **O conteúdo vaza da moldura**:
a 800 px de altura, "Depurador do kit", "Toolchains instaláveis" e "Sysroot
e SDK" desenham por cima do editor e da barra de status, e "Medir tamanho"
sobrepõe "Procurar sonda e portas" (o `EmbeddedPanelHost` calcula a altura
por fórmula — 480 + 20 por sonda — e o `EmbeddedPanel` não tem `clip` nem
rolagem). É o painel com mais informação e o único que hoje não cabe.

**12d Containers** (680×400). Título, subtítulo, a linha do motor com
bolinha verde ("Motor: Podman 5.7.0" + "responde · rootless · socket · sem
compose"), "Containers" com a dica do core e o `[]` cru, "Imagens" com três
linhas "· repo:tag · tamanho". Rodapé: chip "parados também" + compose up /
compose down / Atualizar / Fechar. É o único que já tem o veredito na
primeira dobra — e o único sem formulário.

**O que se mede, em números:** quatro molduras com quatro tamanhos (720×560,
680×520, 560×~780, 680×400); quatro rodapés com 4–5 botões e **zero ações
primárias**; três subtítulos de uma linha e um de duas; dois painéis com
lista+formulário, um com seções empilhadas, um com listas. O veredito
(verde/vermelho) existe em dois (`DataSourceVerdict`, `RemoteVerdict`, 77 e
106 linhas, o mesmo retângulo escrito duas vezes), é uma linha de texto no
terceiro e não existe no quarto. A grade de resultados existe só no banco
(`DataSourceQuery`, Flickable + Repeater, colunas de 120 px fixos).

### 8.2 A referência, lida para isto

A referência (JetBrains, *Database tool window* e *Services*) faz três
coisas que os quatro painéis não fazem igual: (1) **cabeçalho constante** —
nome do que se olha, um subtítulo de estado, a ação primária no canto
direito; (2) **veredito antes do detalhe** — a conexão testada aparece como
faixa antes do formulário, não depois dele; (3) **uma grade** para tudo
que é tabela (resultado, colunas, containers), com cabeçalho fixo e colunas
que cabem no que há. A progressive disclosure de Nielsen entra no
Embarcados: nove seções não cabem numa dobra — cabem em seções com o
resumo na linha do título (fechadas por padrão as que não têm nada: "Sonda
— nenhuma", "Portas — nenhuma").

### 8.3 O desenho: três componentes comuns, quatro painéis que os usam

```text
KvPanelFrame (chrome)          ui/qml/components/KvPanelFrame.qml
  Substitui o Rectangle+MouseArea repetido nos quatro *PanelHost. Recebe
  `panelWidth/panelHeight` desejados e os `maxAvailable*`; centraliza;
  dispensa por clique fora; e — o que falta hoje — o CONTEUDO ROLA quando
  nao cabe (Flickable com clip): o Embarcados deixa de vazar.

KvPanelHeader (cabecalho)      ui/qml/components/KvPanelHeader.qml
  `title`, `subtitle` (UMA linha, elide), `primaryLabel`/`primaryEnabled`
  -> `primaryRequested`, e o x -> `closeRequested`. Os quatro paineis passam
  a ter a mesma primeira linha; a acao primaria e' a que cada painel
  promete: Banco "Testar" (ou "Executar" quando ha' consulta), Remoto
  "Sondar", Embarcados "Procurar sonda e portas", Containers "Atualizar".

KvVerdict (veredito)           ui/qml/components/KvVerdict.qml
  `busy` + `busyText`, `ok`, `message`, `detail` (linhas opcionais) — a faixa
  verde/vermelha que DataSourceVerdict e RemoteVerdict desenham cada um por
  si. O Containers usa para o motor; o Embarcados para a sonda.

KvDataGrid (grade)             ui/qml/components/KvDataGrid.qml + GridRules.qml
  `columns: [{key,label,width?}]`, `rows: [ {...} ]` (ou arrays), `emptyText`,
  `mono`, `maxHeight`; cabecalho fixo, largura das colunas pela regra pura
  `GridRules.columnWidths(columns, rows, available)` (mede o texto por
  contagem de caracteres, teto e piso) — testavel no harness sem tela;
  `null` em italico. Usa: Banco (resultado e estrutura), Containers
  (containers, imagens), Embarcados (portas seriais, sondas), Remoto (o
  que a sonda achou no alvo).
```

**O que sai:** `DataSourceVerdict` e `RemoteVerdict` viram usos do
`KvVerdict` (o campo de senha do banco fica no painel, ao lado do
veredito); a grade caseira do `DataSourceQuery` vira `KvDataGrid`; os
quatro `*PanelHost` viram `KvPanelFrame { ...Panel {} }`. **O que não
muda:** os controllers (nenhuma regra de negócio move), o protocolo, os
formulários (campos e chips seguem como estão — a F8 é forma, não campos).

### 8.4 A medida

- Um componente de grade com harness (`tst_grid_rules`: larguras com
  piso/teto, distribuição do que sobra, `null`, colunas a mais que a
  largura → rolagem horizontal).
- Fotos 13a–13d dos quatro painéis DEPOIS, nas mesmas condições das
  12a–12d: as quatro molduras com a mesma primeira linha e a mesma faixa
  de veredito; o Embarcados cabendo (rolando) a 800 px; o Banco sem cortar
  o campo de consulta.
- Contagem: linhas duplicadas de veredito 77+106 → 0 (`verificar-qml-
  duplicacao` verde); `*PanelHost` de 51–80 linhas → ≤ 40 cada.
- Catracas: view 300, host/controller 400 — o `EmbeddedPanel` (275) e o
  `RemotePanel` (234) só perdem linhas.

### 8.5 O que a F8 NÃO faz (dito)

Não mexe nos campos nem nas ações de cada domínio; não junta os quatro
painéis numa "janela de serviços" (é a forma da referência, mas é outra
etapa); não prova PostgreSQL/Mongo/Pi/sonda reais (§4.2.3-b do 40); não
mede o clique (só o autor). O Grafana (`GrafanaPanelHost`, 5º painel de
ambiente) recebe o `KvPanelFrame` e o cabeçalho pelo mesmo caminho se
couber na sessão; senão fica dito.

### 8.6 O que a F8 entregou contra a medida (2026-09-18, noite)

- Componente de grade com harness: `KvDataGrid` + `GridRules`
  (`tst_grid_rules`, 10 asserções: célula, `null`, objeto/array, piso,
  teto, sobra, encolher, rolar, vazio). ✓
- Fotos 13a–13d nas mesmas condições das 12a–12d: as quatro molduras com
  a mesma primeira linha e a faixa de veredito; o Embarcados rolando a
  800 px; o Banco com o campo de consulta visível. ✓
- Veredito escrito duas vezes → uma (`KvVerdict`); `*PanelHost` 60–80 →
  25–56 linhas (o do banco fica em 51 porque repassa 20 propriedades — é
  fiação, não chrome). ✓ (a meta dizia ≤ 40; dois de quatro passam)
- Catracas: `EmbeddedPanel` 275 → 262, `RemotePanel` 234 → 213,
  `DataSourcePanel` 220 → 199, `ContainerPanel` 167 → 128. ✓
- Não feito (§8.5 e `40` §7.65): containers/portas como grade com ações;
  Grafana/Setup/Biblioteca na moldura comum.

## 9. A fila do TESTE DO AUTOR (noite de 2026-09-18) — o que entrou, e o desenho do que falta, linear

Escrito a pedido do autor antes de qualquer código novo ("pode ser que o
limite acabe no meio do desenvolvimento; todo o contexto deve ficar salvo
na documentação"). Quem retoma lê ESTA seção de cima a baixo e sabe onde
cada coisa está e o que fazer a seguir, sem depender da conversa.

### 9.1 O que o autor disse depois de testar a IDE (2026-09-18, ~19h30)

1. Banco: clicar em MongoDB acendia também PostgreSQL; não dava para criar
   nem descobrir um banco; "a maior parte aparentou ser apenas visual".
2. O painel de baixo (as dez abas, o terminal com "Execução" e "limpar")
   precisa da HUD reformulada no nível de polimento das F1–F8.
3. "Execução" dentro de Terminal não faz sentido — já há o terminal
   integrado e os botões de atalho no canto superior direito.
4. Git: "melhorar a parte visual do versionamento, histórico e afins; bem
   provável de ser necessário criar uma HUD única para o Git com base nas
   IDEs JetBrains".
5. "Ainda há muito a ser polido/otimizado."

Decisões dadas como padrão e aceitas ("prossiga"): ▶ roda numa aba de
terminal PTY (não numa saída própria reformada); criar servidor de banco em
container ENTRA (com o comando visível e confirmação por clique).

### 9.2 O que entrou, commit a commit (todos em `main`, nada enviado)

```text
commit   protocolo  o que                                              registro        fotos
cb46660  0.124.0    Banco: datasource.discover (socket/porta local,   40 §7.66        14
                    containers de banco, .sqlite do projeto) e         37 §6
                    datasource.create (SQLite em data/; PostgreSQL ou  DocsPrivate/Codex/
                    Mongo em container no loopback, comando visivel;   2026-09-18-banco-
                    CREATE DATABASE pelo query confirmado); a coluna   descoberta-criacao.md
                    "Nesta maquina" + "Salvos"; caixa "Novo banco";
                    o chip do Mongo (`!arquivo && !mongo`)
f35cac4  —          Seis ancoras perdidas desde 2026-09-03 (refactor   40 §7.67        15a-15d
                    dbdafa0): o painel Git sem nomes de arquivo, o
                    historico com o hash por cima da data. Gate:
                    verificar-qml-propriedades reprova margem de
                    ancora sem a ancora (provado por mutacao)
62d1168  0.125.0    A execucao e' uma aba de terminal: run.start/       40 §7.68-7.69   15e, 15f
                    run.script abrem PTY (terminalId na resposta;      manual §5
                    event.terminal.render/closed); RunManager,         2026-09-18-execucao-
                    run.stdin e event.run.* sairam; RunPanel, o chip    no-terminal.md
                    "Execucao" e o "limpar" sairam; a aba fica com
                    ✓/✗ N; um dono para abrir o shell (eram dois);
                    a faixa de abas do painel de baixo (sem borda,
                    pilula, contagens Problemas/Testes/Jobs, ordem do
                    uso, x que esconde)
686b68f  0.126.0    HUD do Git, fatia 1 (SAVE POINT): git.log com       40 §7.70        16a, 16b
                    parents/refs; git.commit { amend }; GitPanel em     2026-09-18-git-hud-
                    duas colunas (Mudancas por pasta | Historico com    fatia1.md
                    raias, pontos, anel de merge, chips de refs;
                    commit com Amend e Commit e Push; a direita o
                    inspetor: diff da mudanca ou o commit com sha,
                    autor, refs, arquivos +/- na grade, patch)
```

Antes desses, no mesmo dia: 4182769 F0 · 84b1e80 F1 · cf0de24 F2 ·
5517538 F3 · 9d97bd1 F4 · 271d9c7 F6-a · 225564c F5 · 409f375 F6-b ·
36f90fd F7 · 2985206 docs (40 §4.2) · 2f3aa34 o elo solto do despacho C++
(40 §7.64) · abb8cc8 F8.

**Estado medido ao fim:** protocolo 0.126.0 · 161 métodos · 55 eventos ·
838 testes Rust · 53 harnesses · 24 gates verdes · Clang-Tidy limpo ·
binário debug abre em ~750 ms sem aviso do QML.

### 9.3 O mapa dos arquivos que a fila tocou (para não procurar)

```text
Banco     crates/kinein-protocol/src/datasource_discover.rs   (tipos discover/create)
          crates/kinein-core/src/datasource/{discover,create}.rs
          crates/kinein-core/src/handlers/datasource_discover.rs (discover adiado por defer_work;
                                                                 create sincrono ou job)
          crates/kinein-core/src/tests/datasource_discover.rs   (podman falso; 4 testes)
          ui/src/core_client_datasource.cpp  ui/src/core_client_notifications.cpp
          ui/qml/datasource/{DataSourceDiscoveryController,DataSourceCreateBox,DataSourceList}.qml
          ui/qml/ipc/DataSource{Request,Event}Router.qml (Connections no FILHO `discovery`)
          scripts/qml-harness/tst_datasource_discovery.qml
Execucao  crates/kinein-core/src/run.rs (so' erro/catalogo/comando padrao)
          crates/kinein-core/src/handlers/run.rs::start_in_terminal   Core.run_terminal
          crates/kinein-core/src/tests/mod.rs::terminal_run_until_closed + render_lines
          scripts/verificar_python_debug.py::Core.texto_da_execucao   scripts/verificar-exercitacao.sh
          ui/src/core_client_requests_run.cpp::dispatchRunResult      m_runTerminalId
          ui/qml/runtime/RuntimeController.qml (runTerminalId, finishedRuns, runTabTitle)
          ui/qml/panels/bottom/{TerminalSessionTabs,TerminalPanel,BottomPanelHost}.qml
          ui/qml/shell/BottomTabBar.qml (badgeFor)  ui/qml/jobs/JobsController.qml (testsBadge)
Git       crates/kinein-protocol/src/git.rs (parents, refs, amend)
          crates/kinein-core/src/git/{operations,parse}.rs  crates/kinein-core/src/tests/git.rs
          ui/qml/git/GitRules.qml            regras puras: lanes, patchFiles, lineKind, folderOf, refChip
          ui/qml/git/GitInspectorController  o que esta' selecionado (filho do GitController)
          ui/qml/git/GitInspectorPane.qml    a coluna da direita
          ui/qml/git/GitPatchView.qml        o patch linha a linha (saiu do GitDiffDialog)
          ui/qml/git/GitCommitBox.qml        mensagem, Amend, Commit e Push, Commit
          ui/qml/git/GitChangesList.qml      secoes por pasta (role `folder`), selecao
          ui/qml/git/GitHistoryList.qml      raias (laneWidth 12), pontos, anel, chips de refs
          ui/qml/git/GitHistoryController    laneCount, entry(sha), refsText (US-separado)
          ui/qml/git/GitController.qml       398/400 — coisa nova vai para um FILHO
          ui/qml/panels/bottom/GitPanel.qml  duas colunas; fala com o controller
          scripts/qml-harness/tst_git_rules.qml
Gates     scripts/verificar_fiacao_ipc.py (pergunta 5: elo de despacho sem chamador)
          scripts/verificar_qml_propriedades.py (margem de ancora sem ancora)
          scripts/verificar_binario_abre.py (aviso do QML no stderr ate' o 1o frame)
```

### 9.4 O desenho da PRÓXIMA fatia — HUD do Git, fatia 2 (escrito antes do código)

Ordem dentro da fatia, cada item com o que muda e como se prova:

**(a) Stage por pasta.** No `GitChangesList`, o cabeçalho da seção
(`section.delegate`) ganha um checkbox: marcado quando TODAS as mudanças
da pasta estão staged, meio quando algumas. Clique → novo sinal
`folderStageToggleRequested(folder, stageAll)` → `GitController.
toggleFolderStaged(folder, stageAll)` → emite `stageRequested(paths)` /
`unstageRequested(paths)` com todos os caminhos da pasta (o core já aceita
vários `paths` em `git.stage`/`git.unstage`; nada de contrato). Harness:
`tst_git_staging` (existe? conferir `scripts/qml-harness/`; senão criar)
com três mudanças em duas pastas.

**(b) Filtro no Histórico.** Um campo de texto acima da lista
(`GitHistoryFilter.qml`, view) filtra por resumo/autor/sha localmente
(`GitRules.matchesFilter(entry, text)` — puro, no harness). Filtrar por
branch pede contrato: `git.log { maxCount?, ref? }` (0.127.0) → `git log
<ref>`; o handler valida `ref` (`git check-ref-format --branch`, ou a
regra: sem espaço, sem `..`, sem `-` inicial); `GitHistoryController.
refreshHistory(ref)`; o chip do branch atual no cabeçalho do histórico.
Teste de despacho em `tests/git.rs` (dois branches, o log de cada um).

**(c) As curvas do grafo.** Hoje `GitRules.lanes` devolve `{lane, merge,
laneCount}` por linha. Para desenhar as ligações, a regra devolve também
`edges: [{fromLane, toLane}]` por linha — de onde este commit sai (a raia
dele) para onde cada pai está na linha seguinte (o pai herda a raia; o
segundo pai está noutra raia). A vista (`GitHistoryList`, o `Item grafo`)
desenha, por aresta, uma linha da metade inferior desta linha à metade
superior da próxima: reta quando `fromLane === toLane`, diagonal (ou
`Canvas` com curva de Bézier, 2 pontos de controle) quando muda de raia.
Harness: o caso do merge da `tst_git_rules` ganha a asserção das arestas
(m: 0→0 e 0→1; x: 1→0). Foto 16c.

**(d) O branch na barra.** `HeaderGitWidget` (F1) tem `panelRequested()`;
o clique no nome do branch passa a abrir o `GitBranchMenu` (checkout,
criar) como popup ancorado ao widget — `ShellHeaderHost` já roteia
`git.branches` para `gitController.openBranchMenu()`; falta o menu nascer
ali e não só dentro do painel (mover o `GitBranchMenu` para
`ShellOverlays`, posicionado pelo widget, como o `RunConfigMenu` faz).

**(e) Confirmações.** "Commit e Push": um `KvVerdict` neutro na caixa de
commit dizendo "vai enviar para `origin/<branch>`" e o botão pede o
segundo clique (o mesmo padrão do `WRITE_CONFIRMATION_REQUIRED` do banco).
Amend quando `aheadCount === 0` (o HEAD já foi enviado): o aviso vira
vermelho ("reescreve um commit já enviado — vai exigir push forçado") e o
Commit pede confirmação. Sem contrato.

**Medida da fatia:** fotos 16c (grafo com curvas e filtro) e 16d (branch
pela barra); `tst_git_rules` estendido; gates; Clang-Tidy só se o C++
mudar (o (b) muda `gitLog(ref)` na ponte).

**Restrições que valem:** `GitController` 398/400 — (a) e (e) cabem num
filho `GitStagingController` ou dentro do `GitInspectorController`; view
300; controller 400; contrato primeiro (03 + bump + tests) quando (b).

**Estado em 2026-09-18, fim da noite:** a fatia 2 foi FEITA como
desenhada — (a) `0dfe956`, (b)+(c) `b5c68d6` (mais quatro âncoras
perdidas, `40` §7.72), (d)+(e) no commit seguinte (`40` §7.73). Fotos
16c–16e. O que resta do Git: cherry-pick/revert/reset, stash com lista,
o clique real.

### 9.5 Depois da HUD do Git — a ordem

1. **Fechamento da Etapa 2** (`40` §4.2.2): Ln:Col na status bar; a foto do
   gate a 1024 px; o trilho lateral compacto/expandido; o foco da tela
   inicial × terminal; rename/codeActions/workspaceEdit adiados
   (`defer_lsp`); `container.status` adiado (`defer_work`); Grafana/Setup/
   Biblioteca na moldura comum (`KvPanelFrame`); a primeira linha
   (`KvPanelHeader`) nos painéis de baixo se o autor achar que cabe; fotos
   das três telas lado a lado com as de manhã; release-hardened remedido;
   43 §5/§7 e leitura-técnica sincronizados.
2. **Uma sessão do autor na IDE aberta** com o roteiro do `40` §4.2.6 (o
   banco real por container é um clique dele; o "Identificar" do ESP32).
3. **Etapa 3, candidatas** (`40` §4.2.5): restos de banco e remoto; bloco
   F; validação com hardware conforme chegar; release/AppImage só quando
   ele pedir.

### 9.6 Como retomar do zero (se o contexto acabar)

Ler, nesta ordem: `DocsPrivate/Codex/README.md` → `HANDOFF-panorama.md` →
`PROMPT-proxima-sessao.md` (aponta para 9.4) → esta seção → `40` §7.66–
7.70 → os registros privados de 2026-09-18 da noite. Conferir `git
status`/`git log --oneline -12`. Rodar `cargo build -p kinein-core &&
cmake --build build/linux-clang-debug-strict --target kinein-vectis` e as
fotos headless (`KINEIN_STARTUP_COMMANDS=git.log`, `git.commit`,
`datasource.list`, `terminal.open,run.start`) para ver o estado com os
próprios olhos antes de mexer. As regras do projeto continuam: contrato
primeiro, catracas, medir antes de afirmar, registro datado + evidências,
nunca `git checkout <arquivo>`, nunca gravar a placa do autor, nada de
push/release sem ele pedir.
