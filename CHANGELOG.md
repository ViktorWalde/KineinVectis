# Changelog — Kinein Vectis

Versões de teste fechado. O detalhe de cada mudança, com data, medida e
prova, está em `DocsPublic/roadmaps/40-estado-e-continuidade.md` §7.

## 0.3.0 — em desenvolvimento

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
