# 59 — O fechamento da 0.3.9: janelas acopladas, banco completo e pente fino

> **Classe: PLANO** (`DocsPublic/README.md`). Este documento diz **o que falta**
> para encerrar a 0.3.9 e **em que ordem**. O que já foi feito, com data e prova,
> fica no [`40.7`](40.7-registro-das-entregas.md). O estado do projeto, no
> [`40`](40-estado-e-continuidade.md). Cada item, ao ser entregue, ganha a data e
> o parágrafo do 40.7 que o prova; nenhum item sai daqui sem isso.

## 1. A decisão (o autor, 2026-10-03, noite)

A série 0.3.6–0.3.9 reorganizou a casca da IDE. Ela **não se encerra** no estado
de 2026-10-03 (protocolo `0.150.0`). Antes, o autor quer:

- **Painéis que não são pop-up.** Ferramenta de trabalho contínuo vira
  **janela acoplada** ao layout, no modelo das tool windows da JetBrains. Abre
  no slot do lado do trilho em que o ícone está (40.7 §7.207).
- **Banco de dados completo**, nas palavras dele: "um ambiente completo para
  banco de dados e com certa profundidade prática e segura para usar banco de
  dados na IDE". O MongoDB entra **com escrita** (era só leitura).
- **Grafana com visualização web dentro da IDE.** Ela é opcional e vem
  **desligada por padrão**: "vai pesar apenas no download, e não em consumo de
  memória RAM, pois o usuário pode ativar/desativar". Desligada, a IDE não
  carrega o motor web.
- **Pente fino no fim:** polimento de frontend, bugs, comportamento e
  desempenho.

Depois disso a 0.3.9 fecha **oficialmente**, "para não ficarmos nisso
eternamente". A **0.4 é inteira do ecossistema embarcados**
([`52`](52-arquitetura-executavel-da-0.4.md)). Ela já começa com a base de
frontend para painéis acoplados, porque os painéis da 0.4 também não serão
pop-up.

### 1.1 O que continua pop-up, por decisão

| Painel | Por quê |
| --- | --- |
| Aviso de escrita SQL (impacto, 40.7 §7.209) | Decisão do autor ("faz sentido e eu quero que isso se mantenha"): uma decisão pontual e perigosa pede o foco inteiro. |
| Diálogo da conexão do Banco | Configuração pontual. A JetBrains também usa diálogo ("Data Sources and Drivers"). |
| Bibliotecas, Instalar ferramentas | Tarefa pontual: escolher, ver a prévia e aceitar. |
| Configurações, seletor de pastas, renomear, excluir, descartar no Git, colar no terminal | Decisões pontuais. |
| Toolchain e Python (abrem abaixo do widget do topo) | São contextuais ao widget. |
| Embarcados | Fica como está até a 0.4, que o redesenha. |

## 2. A ordem (consolidada com o autor em 2026-10-03, noite)

Cada passo termina com:

- a **prova na tela real, com o mouse e o teclado do autor**;
- os gates verdes;
- o registro no 40.7.

| # | Passo | Situação |
| --- | --- | --- |
| 1 | **Containers acoplados** (§3), **tela de boas-vindas** (§3.1) e **interruptor moderno** (§3.2) | feito (40.7 §7.210–§7.211) |
| 2 | **Topo:** o ☰ esconde por um momento os widgets do topo, em vez de empurrá-los (§3.3) | feito (40.7 §7.213) |
| 3 | **Modernizar os controles antigos** no padrão interativo (§3.4) | feito (40.7 §7.214) |
| 4 | **Configurações redesenhada** (§3.5) e **seletor "Abrir projeto"** (§3.6) | feito (40.7 §7.215), com os campos de texto refeitos e o menu que não deixa o mouse atravessar |
| 5 | **Remoto acoplado** (§4) | feito (40.7 §7.216), provado contra sshd reais, com as conveniências de SSH (confiar no servidor, último contato, programa lembrado) |
| 5b | **Painel de baixo em relevo** (pedido do autor, 2026-10-04: "um fundo e melhorar a separação visual") | feito (40.7 §7.217): bandeja e poço para todas as abas; o texto do terminal na grade |
| 6 | **Grafana: visualização web opcional** (§6) | feito (40.7 §7.218–§7.219, protocolo `0.154.0`), provado contra um Grafana 11.2.0 real, com a aba Web endurecida. O AppImage com o QtWebEngine foi testado (121 MB) e **só volta depois do pente fino** (decisão do autor) |
| 7 | **Banco completo** (§5.1–§5.6), com o MongoDB lendo e escrevendo | a camada de segurança 1 e a grade estão feitas; o resto, a fazer |
| 8 | **Pente fino e fechamento** (§7) | a fazer |

## 3. Containers acoplados

**Inspiração:** o Docker Desktop, adaptado ao estilo JetBrains da IDE. É a
mesma base da janela do Banco.

- **Os arquivos:**
  - `ContainersWindow.qml`: cabeçalho com o motor (● podman · rootless), o
    filtro e "parados".
  - `ContainerListPane.qml`: seções EM EXECUÇÃO, PARADOS e IMAGENS, com ponto
    de estado, nome, imagem e porta do host. Ao passar o mouse aparecem
    parar/iniciar, logs e shell.
  - `ContainerDetail.qml`: o container escolhido, com imagem, id, status,
    portas (a do host vira link para `localhost`) e ações conforme o estado.
  - `ContainerRows.qml`: as linhas, puras.
- **Abre sem projeto**, porque o motor é da máquina (`ShellDocks.needsProject`).
  As abas de logs e shell continuam exigindo projeto, e a tela diz isso.
- **O compose do projeto** fica no rodapé. Desligado, ele diz o que falta.
- **Saíram:** `ContainerPanel.qml`, `ContainerPanelHost.qml` e
  `ContainerListView.qml`. O `ContainerController` perdeu `close()` e
  `panelVisible`; `open()` pergunta ao motor e pede a janela
  (`windowRequested`).
- **Provas:** `tst_container_rows` e `tst_containers_window` (a promessa do
  compose); falta a tela.

### 3.1 A tela de boas-vindas (decisões do autor, 2026-10-03)

- **Só boas-vindas.** Não há trilhos, janelas, painel de baixo, atalhos de
  área (Ctrl+Alt+W/J/O/M/K/P) nem o "Abrir projeto" da barra de cima. O
  conteúdo fica: os projetos recentes, criar, o cartão "Abrir projeto" e
  Configurações.
- **A linha "Ambiente: N de M ferramentas" saiu.** O autor achou que ela
  "atrapalha"; as ferramentas ficam no painel Ferramentas, dentro do
  projeto.
- **O texto é próprio**, e não cópia do site: "Bem-vindo ao Kinein Vectis /
  Do código à placa, no mesmo lugar."
- **O fundo é um degradê da paleta do ícone**, misturado em OKLab ("suave,
  não o sRGB"), com uma **onda animada**:
  - vem ligada e se desliga no interruptor "Animação"; a escolha fica no
    global, `welcomeAnimation`, protocolo 0.151.0;
  - só anda com a tela à vista e a janela em uso, então ao abrir um projeto
    ela para sozinha;
  - o relógio dá 20 passos por segundo: medido, 4,8% de um núcleo ligada e
    0% desligada (a 60 quadros por segundo eram 10,8%).
- **A arte do site foi tentada e trocada.** O autor preferiu o degradê,
  porque a arte competia com o conteúdo. A arte também não está sob a
  licença do código.
- **As cores do degradê são calculadas ANTES**, por um script em `scripts/`,
  e não em tempo de execução. Um gate confere que o resultado bate com o
  script.

### 3.2 O interruptor moderno

`KvToggleChip` com `switchStyle`: rótulo, ícone opcional e um trilho com a
bolinha que desliza (âmbar quando ligado). Todo chip acende ao pairar e
responde ao pressionar. Ele vale para toda preferência de liga/desliga.

### 3.3 O topo com o ☰ aberto

Pedido do autor: ao clicar no ☰, os menus aparecem na barra e os widgets do
topo (projeto, git, toolchain, python) **somem por um momento**, em vez de
serem empurrados para o lado. Ao fechar o ☰, eles voltam iguais.

### 3.4 Modernizar os controles antigos

Feito em 2026-10-03 (40.7 §7.214). O levantamento, por script, achou 31
campos de texto, ~20 botões e três menus desenhados à mão. Agora:

- **`KvTextField`** é o campo da IDE (placeholder, pairar, foco âmbar com
  anel, ícone opcional, × que limpa). Os 31 campos usam ele; fica só o
  caminho editável do seletor de pastas, revisto no passo 4.
- **`KvButton`/`KvIconButton`** acendem ao pairar e respondem ao pressionar,
  com transição; `primary` + `danger` é o vermelho cheio do irreversível.
- **Um menu só** (`AppMenuPopup`) para o ☰, a árvore, o terminal e a
  configuração de execução: ícone, atalho real à direita, separador entre
  grupos e a barra âmbar no item da vez.
- **O atalho do menu sai do catálogo de comandos**, e o gate de atalhos
  confere os mapas de exceção.

### 3.6 O seletor "Abrir projeto" (pedido do autor, 2026-10-03, noite)

Feito em 2026-10-04 (40.7 §7.215, protocolo `0.152.0`). O autor acrescentou,
vendo a tela: o seletor **abre sempre no Início** (`/home/<usuário>`), com o
projeto aberto marcado, e não dentro do projeto.

A navegação de pastas "está muito boa". O que muda:

- **LOCAIS só com o Início** (`/home/<usuário>`). É lá que os projetos
  ficam, e o Início já contém Downloads, Documentos e as outras pastas do
  usuário. Saem os atalhos "Documentos", "Downloads" e "Raiz do sistema", e
  os RECENTES ficam.
- **Dimensionamento e aproveitamento do espaço** revistos: a coluna de
  locais mais estreita, a lista com mais respiro e informação útil por
  linha.
- **O botão "+ Criar projeto…" sai deste painel.** Ele estava apagado e sem
  função clara no "Abrir"; criar continua no cartão da tela de boas-vindas e
  no menu.
- **O selo do ecossistema** (Rust/Cargo, Python…) fica. Uma pasta **híbrida**
  (duas ou mais linguagens que a IDE suporta, como o próprio Kinein: Rust e
  CMake/C++) ganha um selo que diz isso, com as linguagens no tooltip, em vez
  de mostrar só a primeira.

### 3.5 Configurações

Feito em 2026-10-04 (40.7 §7.215). Três seções à esquerda (Editor, Build,
Interface), a linha inteira clicável, a prévia da fonte, o perfil de rigor
com o que cada opção faz, a animação da tela de boas-vindas, e o selo
"neste projeto" onde o valor vem do projeto.

## 4. Remoto acoplado

Os alvos SSH, com deploy, sync e comandos, são trabalho contínuo: a saída é
acompanhada enquanto se edita. O painel pop-up vira janela acoplada:

- a lista de alvos com o estado da conexão;
- as ações do alvo escolhido;
- a saída da última ação.

O mesmo modelo serve de referência para a JetBrains ("Remote Host").

Feito em 2026-10-04 (40.7 §7.216): `RemoteWindow` no slot do lado do ícone,
com os alvos em cima (o ponto da última sonda), uma ação primária com o
porquê, as seções em chips que quebram a linha e o conteúdo da seção. O
pop-up (`RemotePanel`, `RemotePanelHost`, `RemoteList`) saiu. A área com a
janela aberta aparece no trilho enquanto estiver aberta.

## 5. Banco de dados completo

O que existe em 2026-10-03:

- a janela acoplada, com a árvore e os dados na própria janela;
- o console no editor, com cores;
- a grade revista;
- o aviso de escrita com o impacto contado;
- PostgreSQL, SQLite e MongoDB (só leitura).

Ver 40.7 §7.206–§7.209.

### 5.1 Barra, menus e árvore viva

- **Barra da janela**, como na referência JetBrains:
  - **+** com submenu de motores, cada um com ícone colorido, e "Desta
    máquina…" e "Novo banco…";
  - ler de novo, console, ver dados, recolher tudo e localizar o objeto do
    console.
- **Estado vazio** com o atalho: "Nenhuma conexão · Criar conexão… (Alt+Insert)".
- **Menu de contexto** (botão direito) na árvore:
  - **conexão:** abrir console, ler de novo, editar, desconectar;
  - **tabela:** ver dados, abrir console com `SELECT`, copiar nome, gerar
    `SELECT`/`INSERT`/`UPDATE`, esvaziar e remover. As duas últimas passam
    pelo aviso de impacto.
- **A árvore se atualiza sozinha.** Depois de um `CREATE`, `DROP` ou `ALTER`
  executado com sucesso, a estrutura da conexão é relida.
- **Ícones dos motores coloridos**, para reconhecer o motor de relance.

### 5.2 Console que ajuda

- **Completar nomes** de tabelas e colunas a partir da estrutura já lida, e
  palavras-chave. Isso funciona sem servidor de linguagem.
- **Histórico de consultas** por conexão (as últimas N), reabrível no
  console.
- **O LSP de SQL fica como opção registrada.** O candidato é o **Postgres
  Language Server** (Supabase, Rust, MIT). O sqls (Go, MIT) é multi-motor, e
  o sql-language-server (Node) fica fora.

### 5.3 Segurança em camadas

1. **Feito (40.7 §7.209).** O aviso mostra o comando e a consequência contada,
   e na destrutiva só roda com o nome digitado.
2. **Conexão de produção.**
   - Um perfil marcado como **produção** ganha uma cor de destaque na janela,
     no console e no aviso.
   - Toda escrita pede o aviso, mesmo a comum, e a destrutiva pede o nome da
     **conexão** além do alvo.
   - Uma opção "somente leitura" recusa qualquer escrita no core.
3. **Transação com prévia** (PostgreSQL). Uma opção no aviso: executar dentro
   de `BEGIN`, mostrar as linhas afetadas e só então **confirmar (COMMIT)** ou
   **desfazer (ROLLBACK)**.
4. **Mongo também passa pelo aviso.** `deleteMany({})`, `drop()` e
   `updateMany({})` sem filtro são destrutivos.

### 5.4 Grade de dados de trabalho

- **Carregar mais:** hoje a grade para nas primeiras 200 linhas; páginas
  seguintes por `LIMIT/OFFSET`, ou pelo teto do core.
- **Ordenar:** clique no cabeçalho.
- **Copiar:** célula ou linha, em TSV ou CSV.
- **Exportar** o resultado para CSV.
- **Editar célula** numa tabela com chave primária: gera o `UPDATE … WHERE pk`,
  que passa pelo aviso de impacto.

### 5.5 MongoDB completo

- **Escrita:** `insertOne`/`insertMany`, `updateOne`/`updateMany`,
  `deleteOne`/`deleteMany` e `drop`. A sintaxe do console é definida no
  passo, documentada no manual e com teste.
- **O aviso de impacto vale aqui também:** contagem por `countDocuments`
  com o mesmo filtro, e filtro vazio é destrutivo.
- **Teste com MongoDB real em container**, como o PostgreSQL.

### 5.6 Provado com PostgreSQL real

Até aqui tudo foi provado com SQLite, porque não há servidor no gate. A IDE
já sobe um PostgreSQL em container ("Novo banco…"). A bateria na tela inclui:

- leitura e escrita;
- o aviso com `DROP SCHEMA` (contagem por `information_schema`) e
  `UPDATE … FROM`;
- senha pedida;
- TLS `verify-full` (configuração);
- transação com prévia (§5.3).

As imagens `postgres:16-alpine` e `mongo:7` já estão no podman local.

### 5.7 "Todos os bancos": a pergunta do autor (2026-10-04), para decidir

O autor perguntou se há algo pronto que a IDE só orquestre, para a pessoa
escolher qualquer banco em vez dos quatro de hoje. As opções levantadas:

| Caminho | O que cobre | Custo |
| --- | --- | --- |
| **ODBC** (crate `odbc-api`, unixODBC) | Qualquer banco com driver ODBC: Oracle, SQL Server, MySQL, Firebird, DB2, Snowflake… A pessoa instala o driver do fabricante; a IDE lista os DSN do `odbcinst`. | Uma dependência de sistema (`unixodbc`). A árvore sai do catálogo padrão (`SQLTables`/`SQLColumns`). A qualidade varia por driver. |
| **ADBC** (Arrow Database Connectivity) | PostgreSQL, SQLite, DuckDB, Snowflake, BigQuery, Flight SQL | Bom para dados em colunas, mas com poucos motores ainda. |
| **`usql`** (cliente universal, um binário Go) | Mais de 40 bancos pela linha de comando | Orquestrar um processo externo e ler a saída como texto. Serve para console, não para árvore nem edição. |
| JDBC (o caminho do DBeaver e do DataGrip) | Praticamente todos | Exige uma JVM. Fora, pelo peso. |

**Proposta, a confirmar com o autor:** dois níveis.

1. **Nativos completos.** PostgreSQL, MySQL/MariaDB, SQLite e MongoDB, com
   árvore, edição, aviso de impacto e transação.
2. **"Outro banco (ODBC)" genérico.** Console, árvore pelo catálogo padrão e
   grade de leitura, com o aviso de escrita na forma genérica (sem contagem
   prévia).

É a forma que cobre "a escolha do usuário" sem a IDE manter um driver por
banco.

**Decidido pelo autor em 2026-10-04: vale, e entra no passo 7.** Nas palavras
dele: "os nativos e os por plugins que o usuário quiser usar, e a IDE apenas
orquestra". Ficam assim:

- os nativos completos, como acima;
- os outros bancos pelo driver ODBC que a pessoa instala;
- a IDE só lista os DSN e orquestra;
- a IDE não baixa driver sozinha. Um driver é código nativo de terceiros, e
  carregá-lo é gesto explícito, com aviso (§7, segurança).

## 6. Grafana: visualização web dentro da IDE

- **QtWebEngine.** O painel pop-up atual sai; ele está quebrado: o
  "configurar…" não faz nada.
- **Desligada por padrão.** A opção fica em Configurações e liga a
  visualização.
- **Desligada não pesa:** o módulo web não é carregado (nem importado) até a
  opção ser ligada. A prova é medir a RSS com a opção desligada contra a
  medida atual (117 MB, 40.7 §7.203).
- **Só endereços locais** (`localhost`, `127.0.0.1`, `::1`): a visualização
  é para o desenvolvimento do próprio Grafana do projeto.
- **Janela acoplada**, com o endereço, o estado e o painel web.
- **O AppImage** passa a levar o QtWebEngine. O aumento do download foi
  aceito pelo autor e é medido e registrado no passo.

### 6.1 O desenho decidido (2026-10-04, antes de codificar)

Levantado no código e na máquina; a próxima sessão começa daqui.

**Situação de partida.**

- O Grafana é um pop-up (`GrafanaPanelHost` → `KvPanelFrame`, criado pelo
  `ToolWindowPanels.observabilityPanel` dentro do
  `ShellEnvironmentOverlays`).
- A entrada do trilho é `observability`, com `kind: "overlay"`, atalho
  Ctrl+Alt+O e `active` lido de `grafanaController.panelVisible`.
- O `GrafanaController` (394 linhas; o teto é 400) tem `open()`/`close()`
  mexendo em `panelVisible`. Um `Timer` de 30 s anda com ele.
- O conteúdo (`GrafanaPanel`: cabeçalho, ajustes, autenticação, token,
  veredito, cruzamentos, filtro, achados) é reaproveitável.
- O "configurar…" quebrado é o `secondaryLabel` do `KvPanelHeader`, que só
  alterna `setupPinned`.

**A janela acoplada, pelo mesmo caminho do Remoto** (commit `d46b925`; ver o
`git show d46b925 -- ui/qml/shell`).

- O nome da janela é `observability`, o mesmo id do trilho.
- `ShellController.leftWindows` e `observabilityWindowVisible`.
- No `ShellLeftWindowHost`: instância, `minimumOf`, `focusSlot` e
  `Connections { onWindowRequested → showDockWindow("observability") }`.
- No `ToolWindows`: `kind: "dock-left"`, `active` pela janela, sem `panel`; o
  `case "observability"` faz `toggleDockWindow`.
- Saem `GrafanaPanelHost` e `observabilityPanel` do `ToolWindowPanels`.
- No controller: `signal windowRequested()`; `open()` emite o sinal;
  `prepare()` liga `panelVisible` e pede `getRequested`; a janela chama
  `prepare()` ao aparecer e `close()` ao sumir. Para caber em 400 linhas,
  aparar comentários como no `RemoteController`.
- Padrão novo de frontend: cabeçalho como o do `RemoteWindow`, com o título,
  o `statusPhrase`, uma engrenagem que abre e fecha os ajustes (substitui o
  "configurar…" quebrado) e o ×.
- Abas `RemoteSections` "Painel" e "Web".

**A aba Web (QtWebEngine), opcional.**

- Configuração global `grafanaWebView`, desligada por padrão, na página
  Interface das Configurações (`SettingsInterfacePage`, `SettingsToggleRow`).
  A aba Web desligada mostra um cartão: o que é, o custo de memória medido e
  um botão "Ligar" (atalho para a mesma configuração) mais "Abrir no
  navegador".
- **Sem import estático.** A view nasce com
  `Qt.createQmlObject("import QtWebEngine; WebEngineView {…}", slot)`
  dentro de `try/catch`, só quando a opção está ligada E a aba Web abre.
  Assim:
  - nada do módulo web carrega com a opção desligada (a prova é a RSS igual
    aos 117 MB do 40.7 §7.203);
  - o `qmlcachegen` e o `qmllint` não precisam do módulo;
  - sem o módulo instalado, a aba diz "instale `qml6-module-qtwebengine`" em
    vez de quebrar.
- **No `main.cpp`**, antes de criar o `QGuiApplication`:
  `QCoreApplication::setAttribute(Qt::AA_ShareOpenGLContexts)`. É o que o
  QtWebEngine exige quando é inicializado por plugin, e não custa nada
  desligado. Não linkar `Qt6WebEngineQuick`.
- **Só endereço local:**
  - `onNavigationRequested` recusa host fora de
    `localhost`/`127.0.0.1`/`::1`, e o link externo vai para o navegador do
    sistema;
  - `onNewWindowRequested` também vai para o navegador.
  - Compatível com Qt 6.4 e 6.10: `request.reject()` quando existir; senão,
    `request.action = WebEngineNavigationRequest.IgnoreRequest`.
- **Com a aba Web à vista,** a janela pede largura (`docks.widen`, como o
  `DatabaseWindow.widenRequested`, uns 900 px). Clicar num dashboard dos
  achados abre na aba Web quando ela está ligada; senão, no navegador.
- Fechar a janela destrói a view (libera o processo do Chromium). A RSS com
  a view aberta é medida e registrada.

**Ambiente de teste (sem sudo).**

- O QtWebEngine não está instalado no sistema. Os `.deb` (Qt 6.10.2) foram
  extraídos com `apt-get download` + `dpkg -x` numa pasta local; 94 MB de
  pacote, 271 MB extraído.
- Pacotes: `libqt6webenginecore6`, `libqt6webenginecore6-bin`,
  `libqt6webengine6-data`, `libqt6webenginequick6`,
  `qml6-module-qtwebengine`, `qml6-module-qtwebengine-controlsdelegates`,
  `libqt6webchannel6`, `libqt6webchannelquick6`, `qml6-module-qtwebchannel`
  e `libqt6positioning6`.
- Para rodar: `QML_IMPORT_PATH=<root>/usr/lib/x86_64-linux-gnu/qt6/qml`,
  `LD_LIBRARY_PATH=<root>/usr/lib/x86_64-linux-gnu`,
  `QTWEBENGINEPROCESS_PATH=<root>/usr/lib/qt6/libexec/QtWebEngineProcess`,
  `QTWEBENGINE_RESOURCES_PATH=<root>/usr/share/qt6/resources` e
  `QTWEBENGINE_LOCALES_PATH=<root>/usr/share/qt6/translations/qtwebengine_locales`.
- Grafana real: a imagem `docker.io/grafana/grafana:11.2.0` já está no
  podman, na porta 3000.

**O AppImage** (Qt 6.4, Debian 12) passa a levar o QtWebEngine 6.4. O
aumento do download é medido e registrado; a aba Web tem de passar no gate
Qt 6.4 (`verificar-qml-qt64`, `verificar-qml-logica-qt64`).

## 7. Pente fino e fechamento

- **AppImage, depois de todo o pente fino** (decisão do autor, 2026-10-04).
  Antes de empacotar de novo:
  - um QtWebEngine mais novo que o 6.4 do Debian 12, que tem o defeito dos
    avisos "is neither a QObject" (40.7 §7.219);
  - um caminho acelerado para a aba Web, já que o hook portátil força
    software e o Chromium avisa do modo degradado;
  - o `runtime-x86_64` fixado numa release com tag, em vez do canal
    `continuous`.
- **Terminal com reflow:** ao alargar, as linhas antigas continuam quebradas
  na largura estreita (visto com a janela do Grafana voltando ao tamanho).
  Terminais modernos refazem a quebra.
- **Segurança da casca** (pedido do autor, 2026-10-04: a IDE orquestra
  ferramentas prontas — ssh, Grafana, drivers de banco — e a falha não pode
  vir do lado dela). Revisar e testar, com teste que tenta quebrar:
  - **Injeção em linha de comando:** host, usuário, porta, caminho e nome de
    alvo viram argumentos de `ssh`/`rsync`/`ssh-copy-id`/`ssh-keyscan`.
    Valores que começam com `-`, ou que têm `;`, `$()`, espaço ou quebra de
    linha, num `remotes.json` de projeto clonado, não podem virar opção nem
    comando. O mesmo vale para runconfigs e para a linha armada no terminal.
  - **Projeto não confiável:** abrir um repositório de terceiros não executa
    nada, não grava chave e não conecta sem gesto explícito (`.kinein/`:
    runconfigs, remotos, Grafana, conexões).
  - **Credenciais:** o token do Grafana e a senha de banco ficam só na
    memória. Conferir a redação do log do cliente e do core com valores
    reais.
  - **Aba Web:** só localhost; nenhuma ponte JavaScript↔IDE (não há
    WebChannel exposto); link externo e janela nova vão ao navegador.
    Conferir que uma página local não alcança arquivo (`file://`).
  - **`known_hosts`/chaves:** grava só a chave vista; nunca sobrescreve a que
    mudou (`hostKeyChanged`).
  - **Caminhos:** o espelho remoto e o deploy não escapam da pasta (`..`,
    link simbólico, caminho absoluto).
  - **Cadeia de suprimento:** `cargo deny`, hashes fixados, e o runtime do
    AppImage tirado do canal `continuous`, que mudou sem aviso em 2026-10-04
    (o SHA fixado não bateu e o build recusou). Fixar uma release com tag.
  - **ODBC:** driver é código nativo de terceiros. A IDE carrega o que a
    pessoa instalou, com aviso, e nunca baixa driver.
- **Shift+F10** abre o menu de contexto no terminal e na árvore; Executar fica
  no Ctrl+Alt+R. **Confirmado pelo autor em 2026-10-04.**
- **A/B do `AA_ShareOpenGLContexts`** (40.7 §7.218): RSS e primeiro quadro
  com e sem a linha, N≥5 na mesma cena. Se custar, ligar só quando a opção
  `grafanaWebView` estiver ligada (lida antes do `QGuiApplication`).
- **Grafana:** o `KvButton` mostra o tooltip antigo depois que o rótulo muda,
  enquanto o mouse continua em cima ("testa o endereço…" sobre "Atualizar").
  É do `TooltipController`, não do Grafana.
- **Polimento de frontend:** consistência visual entre as janelas acopladas,
  textos, foco e teclado.
- **Caça a bugs de comportamento**, exercitando fluxos inteiros e não fotos
  paradas (regra do pente fino): abrir, editar, compilar com erro,
  Problemas, configurar, recompilar, banco, containers e remoto.
- **Desempenho:** primeiro quadro (orçamento de 400 ms; 247 ms em
  2026-10-03), RSS, e A/B intercalado para qualquer mudança medida.
- **Dívidas conhecidas:** resolvidas DENTRO da 0.3.9 (decisão do autor,
  2026-10-03: "tudo tem que estar preparado para iniciarmos a 0.4, evitando
  retrabalho; a 0.3.9 vai servir de base"). A lista:
  - o slot da esquerda tem uma largura só para as três janelas: alargar o
    Banco alarga o Projeto;
  - ~~os campos de texto feitos à mão~~: **resolvidos** em 2026-10-03
    (`KvTextField`, 40.7 §7.214), com os botões e os menus antigos.
  - **os `Flickable` que podem ficar rolados além do fim** (achado em
    2026-10-04 no Remoto: o conteúdo encolhe com a página rolada e o clique
    seguinte é gasto em "parar o movimento"). Corrigidos no Remoto e nas
    Configurações; varrer os outros 26 com o mesmo modo.
  - **mensagens do core sem acento** (achado em 2026-10-04, no veredito do
    Remoto: "nao alcancei … esta' ligada"): cerca de 60 textos que chegam à
    tela escritos sem acento. As três do Remoto e da instalação foram
    corrigidas; o resto se resolve no pente fino.
  - ~~os testes intermitentes do core~~: **resolvidos na causa** em
    2026-10-03 (40.7 §7.212, `crate::write_executable`); 25 rodadas
    paralelas sem falha, e o gate voltou a rodar em paralelo.
  - ~~a lista de recentes do ambiente de teste que apareceu alterada~~:
    confirmado em 2026-10-03, era o autor usando a instância de teste pela
    tela, e não um defeito.
- **Fechamento:** o 40, o 57 e o CHANGELOG dizem "0.3.9 encerrada", com a
  lista de provas.
