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
assíncrona do LSP), não por medir cliques. **F6-a FEITA** (`40` §7.60): as
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
