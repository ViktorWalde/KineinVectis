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
- Ainda pendentes antes do fechamento: dogfooding do autor em shell/TUI, um SSH
  real e a auditoria de acessibilidade. A série 0.3 completa não está entregue;
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
