# Changelog — Kinein Vectis

Versões beta (série 0.x; até a 0.2 era teste fechado). O detalhe de cada
mudança, com data, medida e prova, está em
`DocsPublic/roadmaps/40.7-registro-das-entregas.md`; a situação de cada versão,
em `DocsPublic/roadmaps/57-mapa-de-versoes-ate-a-1.0.md`.

## 0.3.6 a 0.3.9 — em desenvolvimento, um lançamento só (não lançada)

A reorganização da casca da IDE (roadmaps 53 e 57). As quatro etapas saem
juntas, num pacote só (decisão do autor, 2026-10-02). O que está abaixo existe
no checkout, não em nenhum pacote publicado.

- **Contrato operacional dos adaptadores** (D1a.3, 40.7 §7.237): mensagens
  de operação, catálogo, resultados, prévia e decisão na API externa `1.0`.
  Validação pura recusa contexto/ordem/alvo incorretos e excesso cumulativo
  de dados, preservando NULL e texto vazio. Runtime e migração de perfis
  seguem na fila; IPC UI/core permanece `0.164.0`.

- **Negociação de adaptadores externos** (D1a.2, 40.7 §7.236): contrato
  tipado de API própria `1.0`, identidade, recursos e limites; erros numéricos
  viram mensagens públicas sem expor texto do adaptador. Implementação pura,
  sem iniciar processo ou banco; IPC UI/core permanece `0.164.0`.

- **Catálogo de conexões protegido** (D1a.1, 40.7 §7.235; protocolo
  mantido em `0.164.0`): arquivo inválido, futuro, com opções desconhecidas
  ou campos duplicados é preservado. Salvar/remover e criar bancos são
  recusados antes dos efeitos; o Banco mostra a mensagem. Perfis válidos
  usam escrita atômica e limite de 1 MiB. Formato extensível segue na fila.

- Protocolo `0.164.0` — **Registro de provedores do Banco** (D1,
  40.7 §7.234): formulário e menu usam os descritores dos quatro motores
  atuais fornecidos pelo core. Campos de rede, DSN, credenciais, TLS e amostra
  seguem esse registro; um motor desconhecido fica indisponível. Perfis
  salvos continuam iguais. Drivers externos e LSP seguem na fila.

- Protocolo `0.163.0` — **Desconectar pelo menu do Banco**
  (40.7 §7.231): aguarda os trabalhos e o encerramento dos drivers do
  destino, revoga prévia sem decisão e consentimento ODBC, preservando
  perfil e rascunho. Escrita/COMMIT já aceitos conservam seu desfecho.
  Respostas antigas do console não reconectam depois; outro banco continua
  disponível. F5 ou nova consulta explícita usam o fluxo habitual.

- **Novo banco no menu do Banco** (40.7 §7.230, protocolo mantido em
  `0.162.0`): +/Alt+Insert abrem a criação existente, sem criar ao abrir ou
  cancelar. Perfil, rascunho e editor são preservados; No servidor exige
  PostgreSQL salvo, sem alterações pendentes e com escrita permitida.

- Protocolo `0.162.0` — **Releitura do catálogo após execução**
  (validada no checkout; 40.7 §7.229): alterações de estrutura pedem releitura
  da conexão correspondente. Invalidações durante uma leitura são agrupadas;
  resultados antigos/duplicados não iniciam leituras. Falha parcial pode
  pedir releitura, conservando o erro da consulta.

- Protocolo `0.161.0` — **Modelos do catálogo e ações com impacto**
  (validados no checkout; 40.7 §7.228): SELECT e modelos INSERT/UPDATE gerados
  pelo core chegam ao console como inserções que podem ser desfeitas,
  conservando alterações não salvas. Esvaziar/remover reutilizam a
  confirmação existente. ODBC conserva o SELECT do driver; QML deixa de
  montar consultas de leitura. Desenvolvimento reunido na main (§7.227).

- **Ações da árvore do Banco** (40.7 §7.226, protocolo mantido em
  `0.160.0`): barra de releitura/console/dados/recolher, menu de motores,
  menu de contexto e seleção por teclado que sobrevive à releitura.
  Shift+F10 abre o menu com foco na árvore; F5 relê o catálogo. Menu fecha
  ao mudar perfil/workspace/objeto, restaura foco antes da ação e cabe nos
  dois docks. Estado vazio com botão e Alt+Insert. Cores por motor.

- Protocolo `0.160.0` — **Console e execução com identidades estáveis**
  (validado no checkout; 40.7 §7.225). Nomes parecidos ganham consoles
  distintos e abas legíveis; arquivos antigos ambíguos permanecem intactos.
  Criação recusa links/FIFO e não substitui arquivos existentes. O core
  extrai a instrução com o léxico comum e offsets UTF-16, preservando
  strings/comentários e descartando respostas de outro contexto. Repetir
  o ▶ mantém a aba na mesma posição, com sessão e saída novas; arquivos
  diferentes e shells continuam separados.

- **Editor com a profundidade do terminal** (validado no 40.7 §7.224):
  bandeja das abas mais clara e fundo rebaixado comum ao código, gutter,
  Markdown e conteúdo do painel inferior. Mesmas margens e espaço útil;
  desenho compartilhado no componente KvInsetSurface.

- Protocolo `0.159.0` — **Prévia PostgreSQL** (validada no checkout;
  aceite no 40.7 §7.223). Uma instrução INSERT/UPDATE/DELETE elegível pode
  executar dentro de transação pendente, com amostra RETURNING, confirmação
  única ou rollback e prazo de 60 segundos. Contexto alterado descarta a
  decisão pendente; COMMIT aceito tem desfecho real, incluindo resultado
  desconhecido se a resposta se perder. Avisos explicam sequências e efeitos
  externos que não são revertidos. Corrigido TLS obrigatório que podia cair
  em conexão sem cifra e a grade que interpretava dados como texto formatado.
  Desenho, diagrama e limites em arquitetura/37.

- Protocolo `0.158.0` — **Banco: produção, somente leitura e contexto**
  (validado no checkout, aceite no 40.7 §7.222). Produção pede aviso
  para toda escrita e nomes completos nas remoções/alterações globais.
  Somente leitura recusa escrita e lotes/CTE mutantes antes de conectar.
  Respostas antigas não preenchem consulta, teste, catálogo ou remoção de
  outro contexto. Senha de sessão vinculada ao destino; Enter repete o pedido
  original do console. Remoção PostgreSQL aceita senha de sessão e conserva
  perfil substituído enquanto o job rodava. Desenho e limites em arquitetura/37.

- Protocolo `0.157.0` — **Outro banco (ODBC)**: DSN do unixODBC, catálogo
  padrão, console e grade limitada com NULL. Carregar driver nativo exige
  gesto explícito por sessão/perfil/projeto; cancelar não conecta. A IDE
  nunca baixa drivers. SQL desconhecido e escrita pedem confirmação genérica
  pelo nome da conexão. Diagnósticos arbitrários do driver não ecoam
  credenciais. Documentação técnica, diagrama e limites em arquitetura/37
  e ADR-0007; provas no 40.7 §7.221. Corrigido o mínimo declarado de Rust
  para 1.88, já exigido pelo código e dependências existentes.

- Protocolos `0.155.0`–`0.156.0` — **Banco: escrita MongoDB e confirmação
  seletiva** (fatia de 2026-10-05). O console lê, insere, altera e apaga
  documentos pela gramática JSON, com Extended JSON. Remoções e alterações
  destrutivas pedem confirmação; inserção, criação e alteração filtrada podem
  rodar diretamente. A medição silenciosa impede alterar todos os registros
  sem confirmação, mesmo com filtro. O diálogo usa **Conectar banco** e
  **Criar banco…**, seletores segmentados e padrões por motor, preservando
  valores personalizados e o tamanho da amostra. Corrigidos o percurso de
  Tab no Banco, a recusa de remoção escondida depois de leitura no lote SQL
  e a exigência do nome completo de coleções MongoDB com ponto.
  Arquitetura pública com
  diagramas no 37; provas e pendências no 40.7 §7.220.
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
- **Editor parado não trabalha mais à toa.** Com um arquivo markdown
  aberto, a IDE reanalisava o texto cerca de dez vezes por segundo sem
  ninguém digitar (o realce era confundido com edição). Agora só a edição
  de verdade dispara a análise.
- **Paleta nova, "ilhas".** Moldura cinza ao redor, e o explorador, o
  editor e o painel de baixo numa área escura só, separados por divisórias
  finas, o código integrado à área, textos neutros e um
  véu âmbar no canto do topo — inspirada no tema Islands da JetBrains.
- **Mais espaço para o código.** O topo virou uma faixa só: o ícone da IDE,
  um botão ☰ que mostra e recolhe os menus (Arquivo … Ajuda) na própria
  barra, o projeto, o Git, o contexto e o executar. O caminho do arquivo
  ("projeto › src › main.cpp") foi para a barra de status, e os modos do
  markdown para a faixa de abas: ao todo, cerca de 90 px a mais de editor.
- **Abas de arquivo arrastáveis.** Mude a ordem das abas arrastando; a
  ordem e a aba ativa voltam como estavam ao reabrir o projeto.
- **Arrastar na árvore do projeto, de verdade.** Puxar um arquivo para cima
  ou para baixo arrasta (antes a lista rolava); o cursor leva o nome do
  item, a origem fica esmaecida, a pasta de destino abre sozinha e diz
  "Mover para tests/"; soltar uma pasta dentro dela mesma aparece em
  vermelho e não acontece.
- **Terminal no trilho.** Um clique abre o terminal (o mesmo do Alt+F12) e,
  aberto, o recolhe. Numa janela estreita (abaixo de 1024 px) os Símbolos
  recolhem sozinhos e voltam ao alargar.
- **Teclado de área em área.** Ctrl+F6 leva o teclado para a próxima área
  (explorador, editor, painel de baixo) e Ctrl+Shift+F6 para a anterior; a
  área que recebe o foco fica marcada.
- **Modo Foco.** Ctrl+Shift+F12 (ou Exibir → Modo Foco) recolhe os painéis
  em volta do editor; o mesmo atalho os devolve exatamente como estavam.
  Exibir → Voltar ao editor devolve o teclado ao código de qualquer lugar.
- **Texto e ícones mais confortáveis.** O texto deixou o branco puro por um
  cinza-claro que continua bem legível (contraste acima de 7:1), e os ícones
  ganharam traço mais firme e uma cor própria, mais clara.
- **Dois trilhos, à esquerda e à direita.** Arraste um ícone de um trilho
  para o outro e a área passa a morar lá; os Símbolos (Alt+7) agora são um
  ícone do trilho, e não uma alça dentro do editor. A moldura ficou mais fina
  e contínua ao redor da área de trabalho.
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
- **Digitar sem engasgo em arquivo grande.** A primeira tecla depois de o
  servidor de linguagem colorir o arquivo congelava a tela por cerca de meio
  segundo (2.463 linhas), e uma tecla em cada três perdia um quadro. Agora
  as cores acompanham a edição e só as linhas que mudaram são repintadas: o
  pior caso medido caiu de 495 ms para 10 ms. Os diagnósticos e a busca
  também deixaram de repintar o arquivo inteiro a cada atualização.
- **Ícones redesenhados.** Uma família só, de traço firme e cantos
  redondos: martelo para compilar, caixa de ferramentas, servidor para o
  remoto, frasco para testes, inseto para depurar e um ícone próprio para os
  Símbolos.
- Protocolo `0.148.0` — **paleta só com ações.** A paleta de comandos deixou
  de oferecer o encanamento interno da IDE ("Ping Core", "Shutdown Core",
  "Read File"…). O "Shutdown Core" encerrava o núcleo com um clique.
- **Pequenos acertos.** O seletor de pastas fecha com Esc; os diálogos usam
  o ícone de fechar da família, em vez de um "x" digitado; a tela inicial
  não mostra mais "1:1" sem editor aberto; sem projeto aberto, o trilho não
  mostra mais Projeto, Terminal e Símbolos apagados; os painéis de ambiente
  (Embarcados, Banco, Containers, Remoto, Grafana) não perdem mais o que
  você digitou quando outra parte da janela muda.
- Protocolo `0.149.0` — **o Banco integrado ao layout**, no modelo da janela
  Database da JetBrains. É uma janela acoplada, não mais um painel por cima
  do código:
  - **árvore:** conexão, esquema, tabela, coluna (no MongoDB, coleção e
    campos);
  - **dados:** clique duplo numa tabela traz as linhas para a **própria
    janela**, embaixo da árvore;
  - **console SQL:** é um arquivo do editor (`.kinein/consoles/<conexão>.sql`),
    onde **Ctrl+Enter** executa a seleção ou a instrução sob o cursor;
  - **largura:** a janela se alarga sozinha para a grade caber, sem espremer
    o editor;
  - **a grade:** mostra os valores de verdade (antes, tudo vinha `null`) e o
    botão "ver dados" não pisca mais.
- Protocolo `0.150.0` — **antes de escrever, a IDE mostra a consequência.**
  Uma escrita no console abre um painel com o comando e o que acontece,
  instrução por instrução, com as linhas contadas antes (a contagem é só
  leitura: "Apaga TODAS as linhas de clientes: 2 linhas", "Remove a tabela
  pedidos e 2 linhas dela"). O destrutivo, com a tabela inteira atingida,
  só roda depois que você **digita o nome** do que vai sumir. Isso vale para
  `DELETE`/`UPDATE` sem `WHERE` ou com um `WHERE` que pega tudo, `TRUNCATE`,
  `DROP TABLE`/`SCHEMA`/`DATABASE` e coluna removida. Cancelar é o padrão.
- **Cores no SQL** (e no console `.mongo`). No console, uma linha em branco
  também separa instruções, então um `select` sem `;` não gruda no
  `DELETE` de baixo.
- **A grade dos dados**, revista:
  - cada coluna na largura do texto, com a sobra para a última;
  - arrastar a borda do cabeçalho redimensiona a coluna, e o clique duplo
    volta ao natural;
  - números alinhados à direita;
  - o valor cortado aparece ao pairar.
- Protocolo `0.151.0` — **a tela de boas-vindas é só de boas-vindas.**
  - **O conteúdo:** a apresentação da IDE, criar, abrir, configurações e os
    projetos recentes, que agora destacam ao passar o mouse, dizem "há 2 h"
    e fixam ou removem com um clique.
  - **O que sai:** trilhos, painéis e atalhos de área, que só aparecem com
    um projeto aberto.
  - **O fundo:** um degradê suave nas cores do ícone, com uma **onda**
    animada. O interruptor **Animação** a desliga, e ela para sozinha ao
    abrir um projeto.
- **Containers como janela acoplada**, no espírito do Docker Desktop:
  - uma lista estável por nome, em que nada pula de lugar ao parar ou
    iniciar;
  - ações sempre à vista e coloridas, com "parando…" enquanto o motor
    trabalha;
  - remover em dois cliques;
  - a porta como link e as imagens "em uso".
- **Interruptores modernos**: liga/desliga com trilho e bolinha, e todos os
  chips respondem ao mouse.
- **Janelas do lado do ícone.** Projeto, Git e Banco abrem do lado do trilho
  em que o ícone está, e há um slot novo à direita:
  - arraste o ícone para o outro trilho e a janela vai junto, com tudo o que
    estava aberto;
  - os Símbolos (☰) escondem o slot da direita enquanto estão abertos e o
    devolvem ao fechar;
  - **Ctrl+F6** passa pelos dois lados.
- **Menus com hierarquia e atalhos de verdade.** O ☰, o clique direito na
  árvore, o menu do terminal e o seletor de execução são o mesmo menu: ícone,
  atalho à direita (o mesmo que a IDE obedece), grupos separados e o item da
  vez marcado; setas, Enter e Esc em todos.
- **Campos e botões que respondem.** Todo campo de texto tem o mesmo desenho
  (placeholder, foco âmbar, × para limpar nos filtros), e os botões acendem ao
  pairar e respondem ao clique. O ☰ esconde os widgets do topo enquanto está
  aberto, em vez de empurrá-los.
- Protocolo `0.152.0` — **"Abrir projeto" começa no seu Início.** O seletor
  abre em `/home/<usuário>` com o projeto atual marcado; à esquerda, só o
  Início e os recentes. Uma pasta com duas linguagens (como Rust e C/C++)
  leva o selo **híbrido**, que diz quais são ao passar o mouse.
- **Configurações redesenhadas**: seções Editor, Build e Interface, prévia da
  fonte, cada perfil de rigor explicado, a animação da tela de boas-vindas, e
  o selo "neste projeto" no que o projeto define.
- **Campos de texto vivos**: a linha âmbar cresce no foco, o cursor pisca
  suave, o rótulo do campo sobe ao digitar, e um valor recusado treme.
- **Correções:** o mouse não atravessa mais um menu aberto para acender o que
  está embaixo, e fechar um menu com Esc não desliga mais atalhos como
  Ctrl+O e Ctrl+Alt+S.
- Protocolo `0.153.0` — **o Remoto (SSH) acoplado e prático.** Fica ao lado do
  código; o alvo escolhido mostra o estado e a próxima ação. Na primeira
  conexão, a IDE mostra a impressão digital do servidor e **Confiar neste
  servidor** grava exatamente essa chave. **Copiar minha chave** roda numa aba
  própria e só pede a senha do alvo uma vez (e cria a chave, se você não tiver
  uma). Cada alvo diz quando respondeu pela última vez, mesmo depois de fechar
  a IDE, e o programa que você roda fica lembrado.
- **Campos de texto copiam e colam pelo mouse** (clique direito: Recortar,
  Copiar, Colar, Selecionar tudo, Limpar), além do teclado.
- **Correções:** o ▶ mostra a execução também numa IDE recém-aberta (uma
  execução rápida sumia junto com a aba); a aba leva o nome da configuração;
  enviar uma pasta para o alvo leva o conteúdo dela; um alvo novo não
  sobrescreve mais outro de mesmo nome.
- Protocolo `0.154.0` — **o Grafana ao lado do código, e (opcional) dentro
  da IDE.** A janela do Grafana fica acoplada, como o Banco, com a engrenagem
  para endereço e token. A nova aba **Web** mostra o seu Grafana local dentro
  da IDE. Vem desligada; ligue na aba ou em Configurações → Interface.
  Desligada, o navegador embutido nem carrega. Duplo clique num dashboard o
  abre ali.
- **Correções:** escolher "Pedir na sessão" agora abre o campo do token; o
  duplo clique abre a linha em qualquer grade; a janela do Grafana devolve a
  largura ao sair da aba Web; o texto do terminal vazio não transborda mais.
  A aba Web recusa por padrão janelas, área de transferência e permissões, e
  não grava nada no disco.
- **O painel de baixo em relevo.** As abas ficam numa bandeja um tom acima do
  editor, e o conteúdo (Terminal, Problemas, Jobs…) num fundo rebaixado, com
  borda e sombra no topo: dá para ver onde o código acaba e a saída começa.
- **Correções:** o texto colorido do terminal não abre mais buracos no prompt
  (`hugh@ruki :~`); a pasta de um alvo remoto aberta na IDE respeita o
  `XDG_CACHE_HOME`.

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
