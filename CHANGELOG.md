# Changelog — Kinein Vectis

Versões beta (série 0.x; até a 0.2 era teste fechado). O detalhe de cada
mudança, com data, medida e prova, está em
`DocsPublic/roadmaps/40.7-registro-das-entregas.md`; a situação de cada versão,
em `DocsPublic/roadmaps/57-mapa-de-versoes-ate-a-1.0.md`.

## 0.3.6 a 0.3.9 — em desenvolvimento, um lançamento só (não lançada)

A reorganização da casca da IDE (roadmaps 53 e 57). As quatro etapas saem
juntas, num pacote só (decisão do autor, 2026-10-02). O que está abaixo existe
no checkout, não em nenhum pacote publicado.

- Protocolo `0.145.0` — **depurador sem globais, explicado.** Com o gdb < 16
  (o 15 do Ubuntu 24.04), o painel de variáveis de um alvo bare-metal mostrava
  só registradores, sem dizer por quê. Agora diz: o gdb anterior ao 16 não
  expõe globais pelo DAP; a variável se lê pelo nome em Watches. O gdb que
  vale é o ≥ 16 (decisão do autor, 2026-10-01).
- Protocolo `0.146.0` — **o layout volta como você deixou, por projeto.**
  Janela da esquerda, larguras, Estrutura e painel de baixo são lembrados por
  projeto. Os painéis respeitam o mínimo de que o conteúdo precisa e nunca
  espremem o editor abaixo de 480 px; a janela menor não apaga o tamanho que
  você escolheu. O rodapé do Git não estoura mais a 1024 px: o **Amend** desce
  para a própria linha.
- **Trilho por áreas (começo da 0.3.7).** O trilho da esquerda mostra o
  Projeto e as Ferramentas sempre, e as outras áreas (Embarcados, Banco,
  Containers, Remoto, Observabilidade) quando o projeto ou a máquina as usam —
  um perfil de banco salvo, um alvo remoto, podman ou docker instalado. O que
  fica de fora está no novo **⋯ Mais**, com o motivo e o atalho; o botão
  direito num ícone fixa, desafixa ou oculta a área, por projeto.
- **Painel de baixo contextual.** Terminal e Problemas ficam sempre; Build,
  Testes, Jobs, Debug e Busca aparecem quando há atividade, e a aba que você
  abre nunca some. O botão direito numa aba a fixa. Nada se perde: o menu
  Exibir, a paleta e os atalhos continuam abrindo qualquer aba.
- **Barra de status sem texto cortado ao meio.** Numa janela estreita, os
  resumos do projeto saem inteiros por ordem de importância (o Python, depois
  o contexto do compilador, por último o índice), em vez de um deles aparecer
  partido.
- **"Projeto" em toda a interface.** Onde a IDE dizia "workspace" (tela
  inicial, menu, cabeçalho, mensagens, manual), agora diz "projeto".
- Protocolo `0.147.0` — **navegar pastas sem aperto.** Abrir e criar
  projeto ganharam um navegador largo: locais à esquerda (Início, Documentos,
  Downloads, Raiz e os projetos recentes), o caminho em partes clicáveis com
  voltar, avançar e subir, e a lista grande, com as **pastas de projeto
  marcadas** (Rust/Cargo, CMake, Python…). Pastas ocultas só quando você
  pede; digite "/" para escrever o caminho; setas, Enter e Backspace navegam.
  Trocar o local de um projeto novo, ou criar uma pasta para ele, não perde
  mais a linguagem e o nome escolhidos.
- **Arraste para organizar.** Os ícones do trilho, as abas do painel de
  baixo, os widgets do cabeçalho e os itens da barra de status se arrastam
  para a ordem que você quiser — cada um dentro da própria barra. Durante o
  arrasto o item fica translúcido e uma linha mostra onde ele vai cair; a
  ordem fica salva por projeto. Clique continua sendo clique.
- **A toolchain no cabeçalho, sem repetição.** O que o build vai usar
  ("Clang++ · Ninja", "Cargo") fica ao lado do projeto e do Git; um clique
  abre o seletor, agora organizado por papel — só os do seu projeto à vista,
  cada um com o que está valendo, e os outros recolhidos. O rodapé e a faixa
  de menus deixaram de repetir o nome e o tipo do projeto.
- **O Python do projeto à vista.** Num projeto Python, o cabeçalho mostra
  qual interpretador vale (".venv · 3.14", ou "sistema ⚠"); um clique abre
  o painel com caminho, origem, versão e o aviso — e o botão **Criar .venv**
  quando falta ambiente. É o único lugar do Python: o rodapé não o repete
  mais, e numa janela estreita o **⋯** do cabeçalho acende quando há aviso.
- **Abas de arquivo e cantos redondos.** A aba ativa é uma pílula com
  sublinhado; as outras não têm caixa. As áreas internas do editor e do
  terminal acompanham os cantos redondos das ilhas.
- **Criar e abrir projeto, cada um com a sua tela.** Criar mostra a
  linguagem em cartões, o nome com o caminho que vai nascer e a prévia, e só
  habilita "Criar projeto" com linguagem e nome; abrir diz qual pasta vai abrir.
- **Painel de áreas do trilho.** "⋯ Mais", o botão direito num ícone e
  Exibir → Áreas da IDE… abrem um painel com o que está no trilho e o que está
  fora, o estado de cada área em palavras e botões de fixar e ocultar.
- **Criar Projeto, com todas as linguagens.** A tela inicial tinha dois
  botões fixos, "Novo C++ / CMake" e "Novo Rust / Cargo", e o Python ficava de
  fora. Agora há um único **Criar Projeto**: escolha a linguagem (C/C++, Rust,
  Python ou pasta vazia) e depois o ecossistema dela, com a prévia dos arquivos.
  O mesmo gesto está no menu Arquivo e na paleta ("New Project" não fazia nada).
- **"LSP ✗" sem queda.** Fechar o projeto ou reiniciar o servidor de
  linguagem deixava a barra de status vermelha ("LSP ✗ cpp"), como se o
  servidor tivesse caído. Agora só uma queda de verdade aparece como falha.
- **Gates sem falso positivo num clone novo**, e o que a máquina não prova
  aparece como **NÃO PROVADO** em vez de verde (`scripts/verificar.sh --estrito`
  reprova). Detalhe em `DocsPublic/contribuindo/04-os-gates-que-dizem-nao.md`.

## 0.3.5 — lançada em 2026-10-01 (pré-release "Public Beta")

Fechamento da série 0.3: release
[`v0.3.5`](https://github.com/ViktorWalde/KineinVectis/releases/tag/v0.3.5),
protocolo `0.144.0`. Assets: `Kinein-Vectis-0.3.5-x86_64.AppImage` (SHA-256
`c2710023928767f39b583c1e56fc946ee46f977985fbd08930229d991a33ecf7`), o
`.sha256` dele e `KV0.3.zip` (a pasta `KV0.3/`, com instalador, tutorial e
notas). Requisitos: Linux x86_64, glibc 2.36+, Wayland ou X11. A série 0.x
continua beta: interfaces e configurações podem mudar até a 1.0. As provas
estão no `DocsPublic/roadmaps/40.7-registro-das-entregas.md` §7.147–§7.149.

- **Projeto e arquivos (P0–P3):** comando curto `kinein`, uma
  janela por workspace, navegação e seleção múltipla na árvore, ações por
  menu e teclado, clipboard de arquivos, transferências por lote com Jobs,
  colisões sem sobrescrita e importação externa por cópia. O arrasto interno,
  a importação do Nautilus, a abertura no editor e na tela inicial foram
  exercitados em X11 com arquivos temporários. O botão de remoção usa a
  lixeira do sistema; a exclusão permanente exige escolha separada.
- **Arquivo externo no editor:** uma URL local única abre aba somente leitura,
  sem importação, escrita, rascunho ou restauração na sessão. O protocolo
  `0.142.0` lê texto UTF-8 de até 1 MiB pela rotina segura já usada na
  importação. Provado com o mouse real a partir do Nautilus, em X11 e Wayland.
  A prova revelou e corrigiu um defeito: trocar de aba podia marcar a aba de
  destino como modificada com o texto da anterior.
- **Remote SSH 0.143.0:** a escolha de pasta começa na home do alvo; navega
  filhas e pai por Job SSH e abre o espelho pelo `remote.open` existente.
  A prova com `sshd` e `rsync` reais passou. Caminho longo da HOME agora
  desliga multiplexação quando excederia o limite do socket Unix.
- **AppImage com Qt 6.4 corrigido:** os cinco painéis de ambiente voltaram a
  abrir no pacote (o Loader do Qt 6.4 recusava os componentes) e os ícones SVG
  da árvore voltaram a aparecer (plugin `libqsvg` incluído). O smoke do
  AppImage agora reprova aviso do motor QML e a falta do plugin.
- Protocolo `0.144.0` — **pasta com espaço no espelho SSH.** O navegador
  oferece e o espelho abre pastas e arquivos com espaço no nome (`rsync -s`),
  provado contra `sshd` real. O caminho digitado precisa ser absoluto.
- **Compatibilidade:** um workspace com metadata, sessão e configurações da
  0.2 reabriu no core sem perder código, abas ou preferências. O binário do
  checkout abriu nativamente em Wayland. Arrastar da árvore para o Nautilus
  copia o item, provado pelo autor em Wayland. A recuperação pela Lixeira do
  GNOME foi provada na HOME.
- **Janela (H0):** gestos de menu, maximizar, restaurar, redimensionar e mover
  foram exercitados em X11; a captura a 125% foi aprovada pelo autor.

- Protocolo `0.136.0` — **recusar tem nome próprio.** A sonda do Grafana passa a
  dizer `authRefused` quando o servidor **negou** a credencial. Até aqui a tela
  só via `authenticated: false`, que é também o que ela vê quando ninguém
  ofereceu token: quem colava uma credencial errada lia "sem autenticação" e
  ficava sem caminho de volta. Achado contra um Grafana de verdade, não num
  mock — nenhum teste local recusava nada.
- **Observabilidade (Grafana) refeita.** Um gesto por estado em vez de
  `Salvar · Sondar · Esquecer` com o mesmo peso; a política de token só aparece
  quando o servidor pede; a configuração recolhe depois de pronta; filtro local,
  teclado nas listas e `Enter` que abre no navegador. O cruzamento com os bancos
  do projeto — o que separa isto de um link favorito — **aparece pela primeira
  vez**: a área que o desenhava tinha altura zero desde que nasceu.
- **O token do Grafana sobrevive a fechar o painel**, e só a isso: trocar de
  projeto, confirmar outra instância, esquecer ou ser recusado o apagam. Ele
  está preso ao par projeto + endereço confirmado, conferido na hora de ir para
  o fio.
- **`Esc` fecha os cinco painéis de ambiente** — a moldura comum nunca tinha
  ouvido o teclado.
- **O comando `kinein`** passou a ser instalado de verdade: sem argumento abre a
  pasta atual, e todo o resto vai intacto para o binário, que já era o dono do
  contrato de argumentos.
- Protocolo `0.135.0` — **a resposta diz sobre o que ela é.** A indentação
  pergunta à gramática (`syntaxTree.indent`) sem nunca fazer a tecla esperar: o
  fallback local aplica na hora, e a correção só entra se documento, versão e
  texto ainda coincidirem. A versão que volta é a da **árvore que respondeu**,
  não a que foi perguntada. E as duas buscas de símbolo passam a devolver `path`
  e `query`, porque até aqui eram indistinguíveis e uma resposta atrasada de
  `@nome` podia pintar a lista de `#nome`.
- `}` digitado alinha com a linha que abriu o bloco, e Enter indenta pela
  estrutura do código em C, C++, Rust e Python. Onde a gramática não sabe
  responder — árvore com erro em volta do cursor —, ela diz que não sabe, e vale
  o que o editor já aplicou.

- Terminal com menu contextual para copiar, colar, selecionar tudo ou a área visível,
  limpar tela/histórico e gerenciar sessões.
- `Ctrl+C` sempre envia interrupção, `Ctrl+Shift+C` copia e `Ctrl+V` cola,
  aliases tradicionais preservados e `Ctrl+Alt+V` para `^V`.
- Protocolo `0.130.0`: `terminal.clearScrollback` apaga apenas o histórico da
  sessão indicada, inclusive se estiver rolada para o histórico.
- Colagem arriscada com preview/confirmar/cancelar, opção explícita de uma
  linha e proteção contra ESC rompendo bracketed paste.
- Menu com teclado e altura limitada; seleção obsoleta invalidada, sem copiar
  texto alterado pela saída. Pesquisa e diferenças frente a VS Code/JetBrains
  registradas na especificação do terminal.
- Protocolo `0.131.0`: Selecionar Tudo alcança todo o buffer retido da sessão
  ativa, com cópia sob demanda, Unicode/wrap nativos e rejeição de seleção
  obsoleta. Selecionar não altera o clipboard.
- Nomes `terminal`, `terminal1` etc. reutilizam a primeira posição livre,
  mantendo IDs e buffers independentes. `Shift+F10` abre o menu com foco no
  terminal; fora dele continua Executar.
- Provado em automação com Bash e Vim reais: Selecionar Tudo copia o histórico
  fora da tela, a rolagem preserva a seleção, a TUI copia só a tela alternativa
  e `Ctrl+C` interrompe de imediato mesmo com seleção ativa.
- Protocolo `0.132.0` — Remote: o painel encontra os aliases do seu
  `~/.ssh/config` (inclusive os de `Include`), diz de qual arquivo cada um veio
  e mostra o que o `ssh` faria com ele (`o ssh vai em user@host:porta`, medido
  por `ssh -G`, sem conectar). Escolher um alias cria o alvo sem repetir
  usuário, porta nem chave. O texto de um `ProxyCommand` não sai do core.
- Protocolo `0.133.0` — Remote: quando a sonda diz que o alvo recusou a chave,
  a IDE oferece **Copiar minha chave (ssh-copy-id)** ali mesmo, junto da
  explicação. Ela mostra a linha antes de rodar e só executa com a sua
  confirmação; nunca gera chave nem digita senha.
- **Remote provado contra um SSH de verdade** (`scripts/testar-remote-ssh.sh`,
  um sshd em container): descobrir o alias, explicar com `ssh -G`, a sonda
  recusando sem chave, o `ssh-copy-id` instalando a chave, a sonda medindo o
  alvo e o deploy por `rsync`. Dois defeitos que só o alvo real revelou foram
  corrigidos: o **primeiro deploy para um alvo novo** falhava porque ninguém
  criava `~/kinein/<projeto>`, e a descoberta e a resolução podiam ler arquivos
  de configuração diferentes.
- Protocolo `0.134.0` — Remote: **configurar um servidor novo sem formulário**.
  Cole a linha `ssh` que você já usa e a IDE a lê (nunca executa), preenchendo o
  perfil e dizendo de onde tirou cada campo. O que um perfil não reproduz é
  recusado com o nome da opção, em vez de descartado em silêncio.
- **Abrir pelo terminal** com contrato de verdade: `--help` e `--version`
  respondem sem subir a IDE; caminho inexistente, arquivo no lugar de pasta,
  opção desconhecida e dois caminhos de uma vez são recusados **com o motivo**,
  em vez de a IDE abrir sem projeto calada. `--version` passou a dizer a versão
  do projeto — estava escrita à mão em `0.1.0`.
- **O C++ do projeto passou a ter teste.** Rust e QML eram medidos; o C++ da
  ponte tinha só lint e o smoke de "abre". O gate ganhou uma etapa (Qt Test +
  CTest) e reprova também se nenhum teste for declarado.
- **Painel Remoto reorganizado** em cinco seções — Visão geral · Workspace ·
  Executar · Sistema · Configurar — com **uma ação primária por estado** no topo
  e o motivo dela ao lado. Antes, tudo ficava numa coluna só e as ações do dia a
  dia caíam para fora da tela. Sem alvo, o painel abre onde há o que fazer;
  com alvos, ele já entra num alvo selecionado.
- Um comando desconhecido — na paleta, num menu ou no atalho — deixou de
  **não fazer nada em silêncio**: agora o dispatcher diz que ninguém o tratou.
- O ícone **Git** saiu do trilho da esquerda: o widget do cabeçalho abre o mesmo
  painel e mostra o que o ícone não mostrava — branch, ahead/behind e quantas
  mudanças há. Eram dois caminhos para o mesmo gesto, um deles cego.
- **Terminal validado pelo autor em 2026-09-24**: o roteiro real de shell/TUI
  passou. Continua pendente a auditoria de acessibilidade. A série 0.3 completa não está entregue;
  planos de CLI/pastas/bordas não são features já implementadas.

## 0.2.0 — 2026-09-19

Entre a 0.1.0 (2026-07-14; o AppImage de 2026-09-16) e esta versão
entraram a Etapa 2 (HUD/UI/UX) e a Etapa 3 (as tool windows) — protocolo
IPC de 0.111 a **0.129.0**, 162 métodos / 56 eventos, 843 testes.

**Tela**
- Barra principal com três widgets (Projeto · Git · Executar); trilho
  lateral compacto/expandido; `Ln:Col` na barra de status.
- **Git como janela em pé à esquerda** (alterna com o explorer): Commit
  (mudanças por pasta com checkbox de pasta, Amend, Commit e Push) e Log
  (grafo, refs, filtro por texto e branch). O diff e o commit abrem **no
  editor** como visualização.
- **Aba Símbolos** à direita (`Alt+7`): estrutura do arquivo + busca de
  declarações por nome no índice do projeto ("nesta pasta" antes de "no
  projeto").
- Trilho: Projeto · Git · Embarcados · Banco · Containers · Grafana ·
  Ferramentas (Busca/Build/Debug saíram — estão no cabeçalho, no menu e no
  painel de baixo).
- **Embarcados em abas** Placa · Projeto · Gravar · Kit, com o veredito da
  placa no cabeçalho; cabe numa janela de 800 px.
- **Containers**: filtro, grade com seleção, barra de ações.
- Moldura comum dos painéis de ambiente (`KvPanelFrame`/`Header`/
  `Verdict`/`DataGrid`); a faixa de abas de baixo rola e não bate no × a
  1024 px.

**Comportamento**
- **A execução (▶) roda numa aba do terminal** integrado (PTY real); a
  aba "Execução" saiu.
- **Banco**: descobre o que responde nesta máquina (sockets, portas,
  containers, arquivos `.sqlite`), **cria** um SQLite ou um servidor
  PostgreSQL/MongoDB em container, e **remove** o que criou (só o perfil
  ou com os dados).
- Rename e code actions do LSP não seguram mais o laço do core
  (`defer_then`); `container.status` idem.
- `git.log` traz pais e refs; `git.commit --amend`.
- Persistência do layout: largura dos painéis, aba Símbolos, modo do
  trilho.

**Ainda não** (na 0.3): o painel do Grafana com a moldura comum;
indentação por gramática, signature help, inlay hints, formatação ao
salvar pelo LSP (Etapa 4, roadmap 45).

## 0.1.0 — 2026-07-14

Primeira versão de teste: projetos C++/CMake e Rust/Cargo, editor com
Tree-sitter e LSP, build/test/run/debug, terminal PTY, Git diário,
configurações, Project Health, ambiente do projeto, banco, Grafana,
embarcados, containers, Python, toolchains, índice do projeto.
