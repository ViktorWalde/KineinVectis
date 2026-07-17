# 18 — Plano de daily driver e escada de rigor

> **Status:** ativo
> **Prioridade:** P0 (direção de produto/engenharia)
> **Fonte de verdade:** direção e ordem de execução; UX-alvo continua em
> `docs/specs/`, contratos em `docs/arquitetura/03-ipc-protocol.md`, ponte backend→UI em
> `docs/roadmaps/BACKEND_TO_UI_UX_ROADMAP.md`
> **Ultima revisao:** 2026-07-08

## Objetivo

Tornar o Kinein Vectis a IDE principal do autor **o quanto antes**, sem perder
qualidade, e sustentar isso com rigor crescente de engenharia. Réguas
explícitas:

```text
- Navegação/refactor/analise: régua JetBrains (não "um VS Code").
- Velocidade e composição de ferramentas: régua Neovim super configurado
  (LSP + treesitter + telescope + dap + gitsigns + conform + toggleterm).
- Filosofia inalterada (AGENTS.md): orquestrar ferramentas maduras, nunca
  reimplementar compilador, LSP, debugger ou build system.
```

## Critério de pronto: dogfooding

Cada marco é medido por uma pergunta concreta no repositório do próprio
Kinein Vectis (Rust + C++/CMake, os dois kinds já suportados):

```text
M0  "Consigo abrir, editar, buildar e testar?"                 [feito]
M1  "Consigo passar um dia inteiro só no Kinein?"              [curto prazo]
M2  "Consigo depurar e rodar qualquer alvo sem terminal?"      [médio prazo]
M3  "Consigo commitar, revisar diff e refatorar sem sair?"     [médio/longo]
M4  "Está rápido, estável e agradável o dia inteiro?"          [contínuo]
```

Quando a resposta de um marco vira "sim" no uso real (não em demo), o marco
está pronto. Regressão em marco anterior bloqueia o seguinte.

**Convergência visual:** a UI atual não reflete `docs/specs/`; a
convergência é gradual e vinculante, com fatias C0–C6 amarradas a estes
marcos — regras, inventário e ordem em `docs/roadmaps/20-ui-spec-convergence-plan.md`
(decisão D12 em docs/arquitetura/19). Em resumo: C0 (auditoria) fecha com o M1; C1
(tokens/dimensões da spec) vem ANTES de qualquer UI nova do M2; C2–C3
acompanham o M2; C4 o M3; C5–C6 o M4.

## M0 — Base funcional (feito, 2026-07-08)

Editor com abas/salvar/highlight, project tree confinada ao workspace,
build/test/quality como jobs assíncronos canceláveis, aba Jobs genérica,
Problems com build/quality/LSP, LSP básico (diagnostics, hover, completion,
definition, references, rename), run + terminal PTY, scan de ambiente,
Project Health banner mínimo. Gate único `scripts/verificar.sh` verde
(Rust + C++ + QML estrito).

## M1 — Edição diária confortável (curto prazo)

Meta: um dia inteiro de edição sem abrir outra IDE. Fatias, em ordem
recomendada (cada uma pequena, com UI visível e gate verde):

```text
1. [feito 2026-07-09] Formatação orquestrada: rustfmt/clang-format via
   format.text (síncrono; ver design abaixo), Ctrl+Alt+L na UI.
   Régua Neovim: conform.nvim. format-on-save ficou para a fatia de
   Settings/Storage; qmlformat, para quando o tree QML for reformatado.
2. [feito 2026-07-09] Semantic tokens completos na UI (ver "Fatia M1.2"
   abaixo; a suposição antiga de que "faltava pedir refresh no didChange"
   estava errada — o refresh já existia; o gap real era o mapa de kinds).
   Régua Neovim: treesitter/semantic tokens.
3. [feito 2026-07-09] Code actions / quick fixes LSP (ver "Fatia M1.3"):
   Alt+Enter → popup de ações no cursor → Enter/clique aplica. Validado
   ponta-a-ponta com clangd real (fix-it aplicado em disco via stdio).
4. [feito 2026-07-09] Go-to-symbol de arquivo e workspace no Search
   Everywhere: "@" lista/filtra símbolos do arquivo, "#nome" busca no
   workspace (ver "Fatia M1.4"). Régua Neovim: telescope lsp_*_symbols.
5. [feito 2026-07-09] Sessão por workspace: reabrir o mesmo root restaura
   abas e aba ativa via .kinein/session.json (ver "Fatia M1.5").
6. [feito 2026-07-09] Ergonomia de editor: Ctrl+D duplica, Alt+Shift+↑/↓
   move, Ctrl+/ comenta (token pela linguagem do highlighter), Ctrl+Y
   deleta linha, Ctrl+G vai para linha:coluna (ver "Fatia M1.6").
```

Fora de escopo do M1 (explicitamente pós-V1, custo alto no TextEdit atual):
multi-cursor real, minimap, split de editor. Não fingir paridade aqui.

### Fatia M1.1 — Formatação orquestrada (design fechado em 2026-07-08)

Estruturação feita antes do código, seguindo `docs/arquitetura/ARCHITECTURE.md`.

**Decisões (e porquês):**

```text
- Ferramenta direta (rustfmt/clang-format), NÃO via LSP: independe de servidor
  vivo, é o modelo do conform.nvim, e os próprios LSPs delegam às mesmas
  ferramentas. Menos partes móveis = menos latência e menos manutenção.
- Formata o BUFFER, não o disco: request leva o texto atual; a UI substitui o
  conteúdo do editor e marca a aba como modificada. Salvar continua sendo
  decisão do usuário; zero side effect em disco.
- Síncrono (request/response), NÃO job: formatar um arquivo é curto (<300ms
  típico), mesma classe do tools.detect. Vira job só se um dia existir
  "formatar workspace inteiro" (fora desta fatia).
- cwd do formatter = workspace root: rustfmt.toml/.clang-format do projeto
  valem automaticamente.
- rustfmt: stdin→stdout (--emit stdout); se o root não tiver rustfmt.toml,
  usa --edition 2021 como fallback (rustfmt standalone não lê Cargo.toml).
- clang-format: stdin→stdout com -assume-filename=<path> (acha .clang-format
  e reconhece a linguagem pela extensão).
- Confinamento: path precisa estar dentro do workspace (mesma regra de fs.*).
```

**Fora do escopo desta fatia (explícito):** format-on-save (entra junto com
Settings/Storage para persistir a escolha), qmlformat (o tree QML ainda não
foi reformatado — degrau próprio na escada de rigor), formatar seleção,
formatar workspace inteiro.

**Contrato `format.text` (novo domínio `format`):**

```text
request : { "path": string, "text": string }
response: { "path": string (canônico, ecoado p/ descartar resposta stale),
            "text": string, "changed": bool,
            "formatter": "rustfmt"|"clang-format" }
erros   : NO_WORKSPACE; INVALID_PARAMS (path fora do workspace ou extensão sem
          formatter); ToolNotFound (binário ausente, com sugestão de pacote);
          InternalError (formatter saiu com erro; stderr na mensagem)
```

**Fluxo (guardrails de docs/arquitetura/17 preservados):**

```text
Ctrl+Alt+L → GlobalShortcuts → EditorController.formatCurrentFile()
  → signal formatRequested(path, text) → Main.qml → CoreClient::formatFile
  → core format.text → CoreClient signal fileFormatted(path, text, changed)
  → EditorEventRouter → EditorController.handleFormatResolved
    (descarta resposta stale se a aba mudou; substitui o texto preservando o
    cursor clampado; a substituição passa pelo fluxo normal de edição, então
    marca modificado e dispara lsp.didChange + semantic tokens sozinha)
```

**Arquivos e status:**

```text
[feito] crates/kinein-protocol/src/format.rs      tipos + testes serde
[feito] crates/kinein-core/src/format.rs          domínio: formatter por
                                                  extensão + processo stdin/stdout
[feito] crates/kinein-core/src/handlers/format.rs router + handler fino
[feito] crates/kinein-core/src/lib.rs             format na cadeia de routers
[feito] crates/kinein-core/src/commands.rs        descriptor p/ command.list
[feito] crates/kinein-core/src/tests/format.rs    testes de comportamento
[feito] ui/src/core_client.*                      formatFile + fileFormatted
[feito] ui/qml/editor/EditorController.qml        formatCurrentFile/handleFormatResolved
[feito] ui/qml/ipc/EditorEventRouter.qml          rota do resultado e da falha
[feito] ui/qml/shell/GlobalShortcuts.qml          Ctrl+Alt+L (JetBrains-like)
[feito] ui/qml/Main.qml                           fio formatRequested → CoreClient
[feito] docs/arquitetura/03-ipc-protocol.md                   contrato documentado
```

**Testes de comportamento (core):** extensão→formatter (rs/c/cc/cpp/h/hpp;
desconhecida → INVALID_PARAMS); roundtrip com formatter fake em tempdir
(texto transformado, changed=true); changed=false quando a saída é idêntica;
binário ausente → ToolNotFound; exit≠0 → InternalError com stderr; path fora
do workspace → INVALID_PARAMS; sem workspace → NO_WORKSPACE.

### Fatia M1.2 — Semantic tokens completos (design fechado em 2026-07-09)

**Diagnóstico antes do design (sondas reais via stdio do core, clangd 22 e
rust-analyzer desta máquina):** o pipeline inteiro JÁ funciona —
`lsp.semanticTokens` responde 46 tokens num .cpp e 88 num .rs de teste, a UI
recebe, e o `EditorHighlighter` pinta spans por linha, com refresh no
debounce de edição (600ms, junto do `lsp.didChange`) e na troca de aba. A
suposição registrada anteriormente ("falta pedir refresh no didChange")
estava desatualizada. Os gaps reais medidos:

```text
1. Suspeita de bug na troca de aba DESCARTADA na leitura do código: o setter
   de filePath do EditorHighlighter já limpa os spans semânticos ao trocar
   de arquivo (o Q_INVOKABLE clearSemanticTokens é que nunca precisou ser
   chamado pelo QML).
2. Gap real: kinds descartados pelo mapa da UI (retornam das sondas e viram
   "sem cor"): rust-analyzer → lifetime, selfKeyword, typeAlias, const,
   boolean, character, generic, attributeBracket (+ builtinType, static,
   derive, escapeSequence, formatSpecifier, union da legend); clangd →
   operator, bracket (+ concept, label, enumConstant da legend).
```

**Decisões:**

```text
- Completar o mapa kind→formato por FAMÍLIA de cor já existente (constantes
  do highlighter), sem inventar cor nova fora do sistema visual:
    tipos      += typeAlias, builtinType, union, concept, generic
    keyword    += selfKeyword, boolean
    string     += character
    meta       += lifetime, attribute, attributeBracket, derive,
                  escapeSequence, formatSpecifier, label
    constantes += const, static, enumConstant (família property)
- operator, bracket, punctuation e unknown ficam SEM cor de propósito:
  recolorir pontuação é ruído visual (padrão JetBrains) e dispara
  rehighlight maior sem ganho de leitura.
- Sem mudança de contrato IPC e sem mexer no core: o gap é 100% UI.
- Espans ficam ancorados por linha entre uma edição e a resposta do refresh
  (~600ms + roundtrip): aceito nesta fatia; correção incremental de offsets
  é complexidade de editor que não paga o custo hoje (ver docs/arquitetura/19).
```

**Arquivos e status:**

```text
[feito] ui/src/editor_highlighter.cpp   famílias de kinds completadas
                                        (lifetime em itálico; operator/
                                        bracket sem cor de propósito)
[feito] docs/diario/18 + ContextoIA.md         notas antigas corrigidas, fatia
                                        marcada
```

**Validação:** sondas reais repetidas nos dois servidores (kinds novos
pintáveis), gate completo `scripts/verificar.sh`, smoke offscreen debug e
release. Não há harness de teste C++ na UI hoje — validação do highlighter é
empírica (limitação registrada em docs/arquitetura/19).

### Fatia M1.3 — Code actions / quick fixes LSP (design fechado em 2026-07-09)

**Decisões (e porquês):**

```text
- Só ações com `edit` inline (CodeAction literal). SEM codeAction/resolve e
  SEM workspace/executeCommand nesta fatia: executeCommand exigiria atender
  requests servidor→cliente (workspace/applyEdit), uma máquina nova inteira.
  Sem resolveSupport declarado no initialize, rust-analyzer e clangd enviam
  os edits eagerly — exatamente o que queremos. Ações disabled ou só-command
  são filtradas fora. Gatilho de revisita: ação importante aparecer sem edit.
- Contexto de diagnostics vem do CORE, não da UI: a thread leitora do LSP
  (que já recebe textDocument/publishDiagnostics) passa a cachear o array
  cru por URI. lsp.codeActions envia range-ponto no cursor + os diagnostics
  do cache que intersectam a linha do cursor. A UI não devolve diagnóstico
  (uma fonte de verdade só).
- Duas chamadas com cache de consulta no core: lsp.codeActions responde só
  [{title, kind}] e o core guarda as ações cruas da última consulta
  (path + lista). lsp.applyCodeAction { path, content, actionIndex } valida
  path/índice contra a consulta ativa e aplica. Evita trafegar
  WorkspaceEdit pelo QML (UI burra) e evita recomputar na escolha.
- Aplicação REUSA o caminho do rename: workspace_edit_plan + o loop
  confinar→ler→apply_text_edits→escrever→sync_if_open, extraído para helper
  compartilhado entre lsp.rename e lsp.applyCodeAction (anti-duplicação).
- UX: Alt+Enter (JetBrains) abre popup no cursor, no padrão visual dos
  popups existentes (completion/usages); Esc fecha, Enter/clique aplica,
  ↑/↓ navegam; consulta sem ações mostra "Nenhuma ação disponível aqui";
  arquivos afetados recarregam nas abas como no rename.
```

**Contrato (protocolo 0.22.0):**

```text
lsp.codeActions     { path, content, line, column }
                 →  { actions: [{ title, kind }] }
lsp.applyCodeAction { path, content, actionIndex }
                 →  { title, files: [...], edits }
erros: NO_WORKSPACE; LSP indisponível; INVALID_PARAMS (índice fora da
lista, consulta ativa de outro arquivo/inexistente, ação sem edit).
```

**Arquivos e status:**

```text
[feito] kinein-protocol/src/lsp.rs         tipos novos + testes serde
[feito] kinein-protocol/src/lib.rs         PROTOCOL_VERSION 0.22.0
[feito] lsp/server.rs                      capability codeAction (literal,
                                           sem resolve) + cache de
                                           diagnostics por URI no handle
[feito] lsp/parse.rs                       code_action_infos (filtra
                                           disabled/só-command) + testes
[feito] lsp/manager.rs                     code_actions + consulta ativa +
                                           edit da ação escolhida
[feito] handlers/lsp.rs                    rotas novas + helper de aplicação
                                           compartilhado com o rename
[feito] commands.rs + tests/*              descriptors + testes de dispatch
[feito] ui/src/core_client.*               requestCodeActions/applyCodeAction
                                           + sinais resolved/applied
[feito] ui/qml/editor/EditorActionsPopup.qml  popup novo (padrão completion)
[feito] ui/qml/editor/EditorController.qml    estado/fluxo das ações
[feito] ui/qml/editor/EditorTextSurface.qml   teclas ↑/↓/Enter/Esc do popup
[feito] ui/qml/editor/EditorPane.qml          instância/posição do popup
[feito] ui/qml/shell/ShellWorkspaceHost.qml   plumbing controller→pane
[feito] ui/qml/shell/GlobalShortcuts.qml      Alt+Enter
[feito] ui/qml/Main.qml + ipc/EditorEventRouter.qml  fios de sinal
[feito] ui/CMakeLists.txt                     registro do QML novo
[feito] docs/arquitetura/03-ipc-protocol.md               contrato documentado
```

**Testes:** parse (mantém ordem; filtra disabled e só-command; aceita lista
vazia/null); handlers (exigem workspace e manager; apply sem consulta ativa
ou índice inválido → INVALID_PARAMS); command.list inclui os dois métodos;
sonda e2e manual com rust-analyzer/clangd reais (quick fix aplicado de
verdade via stdio).

**Descoberta na validação (virou decisão):** cada request posicional
re-sincronizava o documento com `didChange` mesmo sem mudança de conteúdo,
bumpando a versão — o clangd invalida os fix-its da versão anterior e o
`codeAction` voltava vazio. Correção: `did_open`/`did_change`/`sync_if_open`
guardam um hash do conteúdo por documento e **pulam o sync redundante**.
Além de destravar os quick fixes, corta um didChange de documento inteiro em
todo hover/completion/definition (ganho de performance sem complexidade
nova). A consulta ativa de ações só é invalidada quando o conteúdo muda de
verdade. Sonda final via stdio: `lsp.codeActions` → `insert ';'` (quickfix),
`lsp.applyCodeAction` → arquivo corrigido em disco, reaplicar o índice →
INVALID_PARAMS como projetado.

### Fatia M1.4 — Go-to-symbol (design fechado em 2026-07-09)

**Decisões (e porquês):**

```text
- Integra no Search Everywhere existente (modelo único com discriminador
  kind já aceita command/file — símbolo é um kind novo), em vez de criar
  diálogo dedicado: zero superfície nova, mesma memória muscular.
- Prefixos no campo de busca (convenção VS Code, dica no placeholder):
    "@nome"  → símbolos do ARQUIVO atual (textDocument/documentSymbol,
               filtro client-side pelo texto após o @; "@" sozinho lista
               a estrutura do arquivo)
    "#nome"  → símbolos do WORKSPACE (workspace/symbol, query server-side)
  Com prefixo ativo, comandos e arquivos ficam fora da lista (só símbolos).
- O servidor consultado é o do ARQUIVO ATIVO (mesma regra dos outros
  lsp.*): sem arquivo aberto com LSP, a busca por símbolo mostra erro
  humano no próprio diálogo. Consulta multi-servidor (rust+cpp ao mesmo
  tempo) fica explicitamente fora desta fatia — gatilho: uso real em
  workspace misto incomodar.
- documentSymbol pode voltar hierárquico (DocumentSymbol[]) ou plano
  (SymbolInformation[]): o parse achata os dois no mesmo shape, com
  container preservado (nome do pai) e ordem do documento.
- Aceitar um símbolo abre o arquivo e salta para linha/coluna reusando o
  caminho existente de problemas (openDiagnostic) — caminho ABSOLUTO vem
  do core; nada de montar path na UI.
- Caps: 500 símbolos por arquivo, 100 do workspace (mesma filosofia dos
  caps de completion/references).
```

**Contrato (protocolo 0.23.0):**

```text
lsp.documentSymbols  { path, content }
                  →  { symbols: [{ name, kind, path, line, column, container? }] }
lsp.workspaceSymbols { path, content, query }
                  →  { symbols: [...] } (mesmo shape; query não vazia)
erros: NO_WORKSPACE; LSP indisponível; INVALID_PARAMS (path fora do
workspace; query vazia no workspaceSymbols); arquivo sem servidor → erro
estruturado do LSP (UnsupportedFile → INVALID_PARAMS).
```

**Arquivos e status:**

```text
[feito] kinein-protocol/src/lsp.rs + lib.rs   tipos + testes; versão 0.23.0
[feito] lsp/server.rs                         capabilities documentSymbol
                                              (hierárquico) + workspace.symbol
[feito] lsp/parse.rs                          flatten dos dois shapes + kinds
                                              1..26 nomeados + testes
[feito] lsp/manager.rs                        document_symbols/workspace_symbols
[feito] handlers/lsp.rs                       rotas novas (padrão dos irmãos)
[feito] commands.rs + tests/*                 descriptors + guardas de dispatch
[feito] ui/src/core_client.*                  requests + sinal lspSymbolsResolved
[feito] ui/qml/search/SearchController.qml    prefixos @/#, erro sem arquivo
                                              ativo, kind "symbol" no modelo
[feito] ui/qml/ipc/SearchEventRouter.qml      rota do resultado/falha
[feito] ui/qml/command/SearchEverywhereDialog.qml  dica de prefixos no hint
[feito] ui/qml/Main.qml                       fios (path/content do editor;
                                              abrir símbolo via openDiagnostic)
[feito] docs/arquitetura/03-ipc-protocol.md               contrato documentado
```

**Testes:** parse (flatten hierárquico com container; SymbolInformation
plano; kinds numéricos → nomes; caps; uri inválida ignorada); handlers
(workspace/manager ausentes → erros estruturados; query vazia →
INVALID_PARAMS); command.list inclui os dois; sonda e2e com rust-analyzer
real (símbolos de arquivo e busca de workspace via stdio).

### Fatia M1.5 — Sessão por workspace (design fechado em 2026-07-09)

Meta: reabrir o mesmo workspace restaura as abas abertas e a aba ativa.

**Decisões (e porquês):**

```text
- Persistência no CORE, em .kinein/session.json com schemaVersion (padrão
  já usado por workspace.json). A UI nunca toca disco (RNF2); armazenamento
  usa caminhos RELATIVOS ao root (robusto a mover a pasta do projeto), mas
  o contrato IPC só fala em caminhos ABSOLUTOS canônicos — o core
  relativiza ao salvar e resolve/valida ao carregar.
- Restore embutido na resposta de workspace.open (campo opcional session):
  zero round-trip extra e zero método de leitura novo. O campo é injetado
  só na resposta do open — WorkspaceInfo/estado/status não carregam sessão.
- Escrita via workspace.saveSession { openFiles, activeFile? }, disparada
  pela UI com debounce (~1.2s) a cada mudança de abas/aba ativa, do
  EditorController (controllers têm timer; composition root não). Sem
  hook de "salvar ao sair": o debounce durante o uso cobre o real, e a
  janela perdida máxima é ~1s de mudança de abas (aceito e documentado).
- Robustez a estado velho: ao CARREGAR, arquivos inexistentes/fora do root
  são filtrados; schemaVersion desconhecida ignora a sessão inteira
  (nunca quebra o open). Ao SALVAR, entradas inválidas são puladas e o
  resto é salvo (sessão não pode falhar por causa de uma aba órfã).
- Restore na UI reusa o fluxo normal de abrir arquivo (fs.read por aba,
  sequencial): como o core responde em ordem e cada load seleciona a
  própria aba, pedir a ATIVA POR ÚLTIMO a deixa selecionada — sem estado
  de restauração novo.
- Fora do escopo (explícito): cursor/scroll por aba, layout de painéis,
  histórico de workspaces recentes — entram em fatia própria se o uso
  real sentir falta.
```

**Contrato (protocolo 0.24.0):**

```text
workspace.open        → resultado ganha campo opcional
                        session: { openFiles: ["/abs/..."], activeFile? }
                        (presente só quando há sessão válida não vazia)
workspace.saveSession { openFiles: ["/abs/..."], activeFile? }
                      → { files: N }  (N = entradas efetivamente salvas)
erros: NO_WORKSPACE; I/O de escrita → erro estruturado de fs.
```

**Arquivos e status:**

```text
[feito] kinein-protocol/src/workspace.rs      WorkspaceSession + params/result
[feito] kinein-protocol/src/lib.rs            PROTOCOL_VERSION 0.24.0
[feito] crates/kinein-core/src/workspace/session.rs   save/load com
                                              schemaVersion, relativização e
                                              filtragem (novo, com testes)
[feito] handlers/workspace.rs + lib.rs        session no open + rota saveSession
[feito] commands.rs + tests/workspace.rs      descriptor + testes de dispatch
[feito] ui/src/core_client.*                  saveSession + sinal sessionRestored
[feito] ui/qml/editor/EditorController.qml    restoreSession (ativa por último)
                                              + debounce de saveSession
[feito] ui/qml/ipc/EditorEventRouter.qml      rota do sessionRestored
[feito] ui/qml/Main.qml                       fio saveSessionRequested
[feito] docs/arquitetura/03-ipc-protocol.md               contrato documentado
```

**Testes (core):** roundtrip salvar→reabrir com sessão na resposta do open;
arquivo apagado entre sessões é filtrado no load; caminho fora do root é
pulado no save (conta só o válido); session.json com schemaVersion
desconhecida é ignorado sem erro; saveSession sem workspace → erro; o JSON
em disco guarda caminhos relativos. Sonda e2e via stdio (dois processos do
core: salvar num, reabrir noutro).

### Fatia M1.6 — Ergonomia de editor (design fechado em 2026-07-09)

Primeira fatia 100% de UI: **nenhuma mudança de protocolo/core** (o texto já
está no buffer; são operações locais de edição).

**Operações e atalhos (memória muscular JetBrains):**

```text
Ctrl+D            duplicar linha atual ou seleção
Alt+Shift+↑ / ↓   mover linha (ou linhas da seleção) para cima/baixo
Ctrl+/            comentar/descomentar linha ou seleção (toggle)
Ctrl+Y            deletar linha atual (extra da mesma família, JetBrains)
Ctrl+G            ir para linha (diálogo "linha" ou "linha:coluna")
```

**Decisões (e porquês):**

```text
- Operações vivem no EditorTextController (que já tem lineStartAt/
  lineEndAt/selectedLineStarts/indentSelection) e editam via
  surface.remove/insert — NUNCA reatribuindo .text inteiro: preserva o
  undo nativo do TextEdit e evita rehighlight total. O fluxo normal de
  edição (marca modificado, didChange, semantic refresh) dispara sozinho.
- Token de comentário vem da linguagem do highlighter (Q_PROPERTY
  `language` já existente — fonte única; nada de re-detectar por extensão
  no QML): rust/cpp/js → "//"; python/shell/cmake/toml → "#"; json/plain
  → sem token (Ctrl+/ é no-op honesto). Toggle: se toda linha não vazia
  do span já começa com o token (após indentação) → remove (token + um
  espaço quando houver); senão insere "token + espaço" no primeiro
  caractere não branco. Linhas em branco no meio da seleção são puladas.
  Edições aplicadas de baixo para cima (offsets estáveis).
- Mover linha troca o bloco de linhas inteiras da seleção (ou a linha do
  cursor) com a vizinha; nas bordas do arquivo é no-op. Cursor/seleção
  acompanham o bloco movido.
- Após Ctrl+/ a seleção pode colapsar para cursor (edições programáticas
  do TextEdit limpam seleção) — aceito nesta fatia; refinar só se doer.
- Ir para linha: diálogo mínimo no padrão do SymbolRenameDialog (estado
  goToLineVisible no EditorController; visual no EditorPane), pré-preenchido
  com a linha atual; aceita "N" ou "N:C"; entrada inválida fecha sem
  saltar; linha além do fim clampa para a última.
- Validação: fatia sem superfície testável no core (lógica QML — gap D9 de
  docs/arquitetura/19). Gate completo + smoke offscreen + uso real pelo usuário.
```

**Arquivos e status:**

```text
[feito] ui/qml/editor/EditorTextController.qml   duplicate/move/toggle/
                                                 delete/goToLine
[feito] ui/qml/editor/EditorTextSurface.qml      alias readonly `language`
[feito] ui/qml/editor/EditorController.qml       wrappers + estado do diálogo
                                                 + token por linguagem
[feito] ui/qml/editor/EditorGoToLineDialog.qml   novo (padrão rename dialog)
[feito] ui/qml/editor/EditorPane.qml             instância/props do diálogo
[feito] ui/qml/shell/ShellWorkspaceHost.qml      plumbing controller→pane
[feito] ui/qml/shell/GlobalShortcuts.qml         seis atalhos novos
[feito] ui/qml/Main.qml                          fio do abrir diálogo
[feito] ui/CMakeLists.txt                        registro do QML novo
```

## M2 — Build/Run/Debug de verdade (médio prazo)

Estruturado em 2026-07-09. **Gate de entrada: C1** (fundação visual de
docs/roadmaps/20) vem antes de qualquer UI nova deste marco. Cada fatia ganha sua
seção de design (como as do M1) na hora de ser executada — a ordem abaixo é
a ordem de execução.

```text
M2.1 Terminal unificado [decisão do usuário em 2026-07-09]:
     Requisito literal: as abas "Executar" e "Terminal" fazem quase a mesma
     coisa para o usuário — fica SÓ a aba/ícone "Terminal". Reforço do
     usuário: a aba Executar é pouco útil também porque o controle
     ▶ Iniciar já embute o Parar (e vice-versa) — o que importa preservar
     é a SAÍDA do processo e o stdin, que migram para a sessão de execução
     dentro da aba Terminal. Arquitetura:
     os dois backends CONTINUAM separados (run.start controlado ≠ shell
     PTY; regra "não misturar PTY interativo com run controlado" do
     roadmap), mas a superfície visual vira uma só: a aba Terminal ganha
     um seletor de sessão (shell | execução). Shift+F10 continua rodando o
     projeto e passa a focar a sessão de execução DENTRO da aba Terminal
     (stdin, parar e histórico preservados); a aba "Executar" some da tab
     bar. Melhorias de integração na mesma fatia: foco automático do input
     ao abrir, ação de limpar, indicador de processo vivo na aba.
M2.2 CMake service: cmake.configure como job + cmake.presets.list +
     cmake.targets.list (contrato tipado novo); compile_commands.json
     gerado passa a alimentar o clangd (hoje roda em fallback).
M2.3 Cargo service: cargo.metadata (members/targets/features) como base
     para seletores; cargo.check como job separado do build.
M2.4 Run configurations: runConfig.list/create/update/delete persistidas
     em .kinein/ com schema + run.start { configId }; seletor visual entra
     junto com a C3 (Main Toolbar, que nasce conforme spec).
M2.5 Debugger via DAP — a maior peça, define o prazo do marco: orquestrar
     lldb-dap (e gdb como alternativa) num domínio novo debug/ do core,
     contrato tipado antes de UI, operações longas como jobs; breakpoints
     na gutter (entra junto com a C4), stack/variáveis/continue-step no
     painel inferior. Régua Neovim: nvim-dap + nvim-dap-ui. Será dividida
     em sub-fatias (sessão/breakpoints/inspeção) no design dela.

Convergência no caminho: C1 antes de M2.1; C2 (iconografia) durante;
C3 (Main Toolbar) junto de M2.4; C4 (gutter/breadcrumbs) junto de M2.5.
```

### Fatia M2.1 — Terminal unificado (design fechado em 2026-07-09)

Fatia 100% de UI: os contratos `run.*` e `terminal.*` NÃO mudam; muda onde
o usuário vê as coisas.

**Decisões (e porquês):**

```text
- A sessão vira ESTADO do RuntimeController (guardrail: estado em
  controller): terminalSession = "shell" | "run". Os componentes
  TerminalPanel (PTY) e RunPanel (processo controlado) continuam
  existindo como estão — viram o conteúdo das duas sessões dentro da aba
  Terminal, chaveadas por visible. Zero mudança nos backends (a regra
  "não misturar PTY interativo com run controlado" permanece).
- A aba "Executar" sai do BottomTabBar (decisão do usuário). Fluxos:
  Shift+F10/▶ → startRun() abre a aba Terminal já na sessão "Execução";
  Alt+F12 → abre a aba Terminal na sessão "Shell". Um seletor compacto de
  sessão (chips Shell | Execução) aparece no topo do painel quando a aba
  Terminal está ativa; a Execução mostra um ponto quando o processo está
  vivo.
- Indicador na aba: o rótulo "Terminal" ganha "●" enquanto houver processo
  controlado rodando (running) — o usuário vê vida sem abrir o painel.
- Melhorias de integração incluídas: foco automático do input certo por
  sessão (abrir/alternar sessão foca o campo daquela sessão) e botão
  "limpar" que zera a saída da SESSÃO ativa (shell: texto do PTY; run:
  modelo de saída) — funções novas no RuntimeController.
- Sem persistência da sessão ativa (volta em "shell" a cada execução da
  IDE); entra com o resto do layout em Settings/M4.
```

**Arquivos e status:**

```text
[feito] ui/qml/runtime/RuntimeController.qml   terminalSession +
                                               setTerminalSession/clear da
                                               sessão + startRun/open com
                                               sessão e foco
[feito] ui/qml/shell/BottomTabBar.qml          remove "Executar"; "●" no
                                               rótulo Terminal com processo
                                               vivo
[feito] ui/qml/panels/bottom/BottomPanelHost.qml  chips de sessão + painéis
                                               chaveados por sessão + foco/
                                               limpar por sessão
[feito] ui/qml/shell/ShellWorkspaceHost.qml    plumbing sessão/limpar
[feito] MANUAL.md + ContextoIA.md              atual seção 5; estado
```

**Validação:** fatia de UI (gap D9): gate completo + smoke offscreen + uso
real (Shift+F10 abre Execução com saída/stdin; Alt+F12 abre Shell; chips
alternam; limpar zera só a sessão ativa; ● aparece durante execução).

### Fatia M2.2 — CMake service (design fechado em 2026-07-09)

**Motivação medida:** o configure implícito do build.run hoje NEM passa
`-DCMAKE_EXPORT_COMPILE_COMMANDS=ON` — o clangd roda sem a compilation
database real (flags/includes de fallback). Esta fatia dá configure
explícito, presets, targets e liga o clangd na CDB.

**Decisões (e porquês):**

```text
- Diretório de build ÚNICO e imutável: <root>/.kinein/build — o mesmo que
  build.run e run.start já usam. Preset NÃO muda o diretório: quando
  informado, o configure roda `cmake --preset <nome> -S <root> -B
  .kinein/build` (o -B tem precedência) — assim build/run/clangd nunca
  divergem de onde estão os artefatos.
- cmake.configure é JOB (operação longa, regra RNF3), com eventos
  event.cmake.started/output/finished e cancelamento — mesmo esqueleto do
  build.run. Sempre passa -DCMAKE_EXPORT_COMPILE_COMMANDS=ON e escreve a
  query do file-api ANTES de rodar (targets saem da resposta oficial do
  CMake, não de parser próprio de CMakeLists — regra AGENTS).
- cmake.presets.list é síncrono (ler dois JSONs pequenos):
  CMakePresets.json + CMakeUserPresets.json, só configurePresets não
  hidden, na ordem dos arquivos.
- cmake.targets.list lê a resposta codemodel-v2 do file-api gerada pelo
  último configure ({ name, kind }); sem configure → lista vazia (a UI
  explica). cmake.status é síncrono: { configured, hasCompileCommands,
  buildDir } por stat de CMakeCache.txt/compile_commands.json.
- clangd passa a receber --compile-commands-dir=<root>/.kinein/build no
  spawn QUANDO compile_commands.json existe. Limite honesto documentado:
  servidor cpp já em execução não recarrega flags de arquivos abertos —
  configure e reabra o arquivo (ou o workspace) para valer tudo.
- UI mínima desta fatia (toolbar/paineis CMake ficam para C3/M2.4):
  (a) Project Health ganha o estado "CMake sem configure" com ação
  [Configurar] (missão do banner: explicar modo degradado — roadmap);
  (b) comando "CMake: Configure" no palette (Search Everywhere);
  (c) progresso/saída aparecem na aba Jobs (genérica, já existe).
  A UI sabe o estado via cmake.status (pedido ao abrir workspace cmake e
  re-pedido quando event.cmake.finished chega).
```

**Contrato (protocolo 0.25.0):**

```text
cmake.configure    { preset? }  → { jobId } + event.cmake.started/
                                  output/finished { jobId, success,
                                  exitCode, hasCompileCommands }
cmake.presets.list {}           → { presets: [{ name, displayName? }] }
cmake.targets.list {}           → { targets: [{ name, kind }] }
cmake.status       {}           → { configured, hasCompileCommands, buildDir }
erros: NO_WORKSPACE; kind != cmake → INVALID_PARAMS; jobs desabilitados →
INTERNAL_ERROR (configure); cmake ausente → TOOL_NOT_FOUND no job.
```

**Arquivos e status:**

```text
[feito] kinein-protocol/src/cmake.rs + lib.rs   tipos + testes; 0.25.0
[feito] crates/kinein-core/src/cmake.rs         domínio: configure cmd,
                                                presets parse, file-api
                                                query/reply, status (+testes)
[feito] crates/kinein-core/src/handlers/cmake.rs router + handlers + job
[feito] crates/kinein-core/src/lib.rs           rota cmake.* na cadeia
[feito] lsp/server.rs                           --compile-commands-dir
[feito] commands.rs + tests/*                   descriptors + guardas
[feito] ui/src/core_client.*                    cmakeConfigure/cmakeStatus +
                                                sinais + refresh no finished
[feito] ui/qml/workspace/ProjectHealthController.qml  estado "sem configure"
[feito] ui/qml/shell/ShellWorkspaceHost.qml + Main.qml  ação Configurar
[feito] ui/qml/command/CommandDispatcher.qml    comando cmake.configure
[feito] docs/arquitetura/03 + MANUAL + ContextoIA           contrato/uso/estado
```

**Testes (core):** presets (merge dos dois arquivos, hidden fora, arquivo
ausente → vazio, JSON inválido → erro humano); status (não configurado →
false/false; com CMakeCache+CDB → true/true); targets (fixture de reply
codemodel-v2 → nomes/kinds; sem reply → vazio); query do file-api escrita
pelo configure; guardas de dispatch (workspace/kind/jobs); command.list.
Sonda e2e com CMake REAL via stdio: configure num projeto mínimo → status
vira configurado com CDB → targets.list traz o executável → clangd novo
recebe a flag.

### Fatia M2.3 — Cargo service (design fechado em 2026-07-09)

**Decisões (e porquês):**

```text
- cargo.metadata é SÍNCRONO: `cargo metadata --format-version 1 --no-deps`
  é local (só manifests do workspace, sem rede/resolução) e tipicamente
  <300ms (RNF3) — medido na sonda e2e. O core resume o JSON gigante do
  cargo num payload pequeno: pacotes (nome, versão, features, targets com
  kind) + membros do workspace. Gatilho de revisita: workspace real onde
  isso passe de ~1s → vira job.
- cargo.check é JOB e REUSA o pipeline do quality (event.quality.*): o
  `cargo check --workspace --all-targets --message-format=json` fala o
  MESMO JSON do clippy/build, então os diagnósticos caem na aba Problemas
  hoje, sem parser nem UI novos. Aliasing consciente e documentado: até o
  Problems 2.0 ter facetas por origem (roadmap P1), check aparece como
  origem "quality"; o título do job ("Cargo Check" vs "Quality") os
  distingue na aba Jobs. Kind rustCargo apenas (como o clippy).
- Project Health ganha o sinal "cargo metadata falhou" (lista do roadmap):
  a UI pede cargo.metadata ao abrir workspace rustCargo; falha estruturada
  vira banner warning com ação [Tentar de novo]. Sucesso limpa o estado.
- Seletores visuais de features/targets ficam para a C3/M2.4 (Main
  Toolbar) — esta fatia entrega o CONTRATO + o check utilizável já.
```

**Contrato (protocolo 0.26.0):**

```text
cargo.metadata {} → { packages: [{ name, version, features: [..],
                      targets: [{ name, kind }] }], workspaceMembers: [..] }
cargo.check    {} → { jobId } + event.quality.started/diagnostic/finished
erros: NO_WORKSPACE; kind != rustCargo → INVALID_PARAMS; cargo ausente →
erro humano (metadata) / TOOL_NOT_FOUND no job (check).
```

**Arquivos e status:**

```text
[feito] kinein-protocol/src/cargo.rs + lib.rs   tipos + testes; 0.26.0
[feito] crates/kinein-core/src/cargo.rs         domínio: comando metadata +
                                                parse do resumo (+fixture)
[feito] crates/kinein-core/src/build.rs         run_cargo_check (espelho do
                                                run_quality com cargo check)
[feito] handlers/cargo.rs + lib.rs              router cargo.* (metadata
                                                sync + check job)
[feito] commands.rs + tests/cargo.rs            descriptors + guardas
[feito] ui/src/core_client.*                    cargoMetadata/cargoCheck +
                                                sinal + auto-request no open
[feito] ui/qml/workspace/ProjectHealthController.qml  estado metadata falhou
[feito] ui/qml/ipc/WorkspaceEventRouter.qml     rotas resolved/failed
[feito] ui/qml/shell/ShellWorkspaceHost.qml + Main.qml  ação retry
[feito] ui/qml/command/CommandDispatcher.qml    comando cargo.check
[feito] docs/arquitetura/03 + MANUAL + ContextoIA           contrato/uso/estado
```

**Testes (core):** parse do metadata (fixture com 2 pacotes/features/
targets, membros; JSON inválido → erro humano); guardas (sem workspace,
kind errado, jobs indisponíveis no check); command.list inclui os dois.
Sonda e2e com cargo REAL: metadata do projeto mínimo (nome/targets/
features) + tempo medido; cargo.check num código com warning → diagnóstico
chega via event.quality.diagnostic.

### Fatia M2.4 + C3 — Run configurations + Main Toolbar (design 2026-07-09)

**Decisões (e porquês):**

```text
- Run config v1 = { id, name, command }: um comando nomeado rodando na
  raiz pelo run.start existente. Env/cwd/args separados ficam para quando
  o uso pedir (gatilho registrado). Persistência em .kinein/runconfigs.json
  com schemaVersion (mesmo padrão de session.json), caminho de leitura
  tolerante (inválido → vazio, nunca quebra).
- A config ATIVA vive no arquivo (activeId), não na UI: o ▶ (run.start sem
  command) resolve no core: comando explícito > config ativa > heurística
  atual (cargo run / executável único do CMake). "Automático" = activeId
  ausente. Todas as mutações (save/delete/setActive) respondem a LISTA
  completa + activeId — a UI nunca calcula estado derivado.
- runConfig.save unifica criar/editar (id ausente = criar; criar/editar
  torna a config ativa — memória muscular JetBrains). Nome e comando não
  vazios (INVALID_PARAMS).
- C3, interpretação registrada (R1): o TopHeaderBar atual (já 44px, já
  com Build/Testes/Análise/▶/■) VIRA a Main Toolbar da spec em vez de
  criar segunda barra — a região 1 (Title/App Bar com menus) só nasce na
  C5, e duas barras interinas só comeriam altura. Ordem da spec §11.1
  respeitada no que existe: [seletor de Run Config ▼] [Configurar (só
  cmake)] [Build] [Testes] [Análise] [▶] [■]; Target/Profile selectors e
  botão Debug entram com M2.5/fatias próprias. Botões de toolbar 28→32px
  (LAYOUT §6.3).
- Seletor de config: dropdown no toolbar (Automático | configs | Nova... |
  Editar atual | Excluir atual). Diálogo de config (nome+comando) no
  padrão dos diálogos existentes, estado no RuntimeController, visual no
  ShellOverlays.
```

**Contrato (protocolo 0.27.0):**

```text
runConfig.list      {}                     → { configs: [{id,name,command}], activeId? }
runConfig.save      { id?, name, command } → idem (config salva vira ativa)
runConfig.delete    { id }                 → idem (ativa limpa se apagada)
runConfig.setActive { id? }                → idem (id ausente = Automático)
run.start           {} passa a resolver: comando > config ativa > heurística
erros: NO_WORKSPACE; nome/comando vazio ou id inexistente → INVALID_PARAMS.
```

**Arquivos e status:**

```text
[feito] kinein-protocol/src/runconfig.rs + lib.rs  tipos + testes; 0.27.0
[feito] crates/kinein-core/src/runconfig.rs        CRUD JSON com schema
                                                   (+testes de domínio)
[feito] handlers/runconfig.rs + lib.rs             router runConfig.*
[feito] handlers/run.rs                            run.start usa config ativa
[feito] commands.rs + tests/runconfig.rs           descriptor + guardas
[feito] ui/src/core_client.*                       CRUD + sinal
                                                   runConfigsResolved +
                                                   auto-request no open
[feito] ui/qml/runtime/RuntimeController.qml       configsModel/ativa +
                                                   estado do diálogo
[feito] ui/qml/shell/TopHeaderBar.qml              vira Main Toolbar (seletor
                                                   + Configurar + 32px)
[feito] ui/qml/shell/ShellHeaderHost.qml           fios novos
[feito] ui/qml/shell/RunConfigDialog.qml           novo (nome+comando)
[feito] ui/qml/shell/ShellOverlays.qml + Main.qml  diálogo + fios
[feito] ui/CMakeLists.txt                          registro do QML novo
[feito] docs/arquitetura/03 + MANUAL + ContextoIA              contrato/uso/estado
```

**Testes (core):** CRUD roundtrip (criar→listar→editar→ativa muda→apagar
limpa ativa); schemaVersion desconhecida ignora; setActive com id
inexistente → INVALID_PARAMS; run.start sem comando usa a config ativa
(fake command echo validável); guardas de workspace. Sonda e2e via stdio:
criar config, reabrir core, lista persiste, ▶ roda o comando da ativa.

**Correção pós-validação do usuário (2026-07-09):** o dropdown do seletor
embutido no header ficava ATRÁS do editor/árvore — `z` em QML só ordena
irmãos do mesmo pai, e o header (44px) é desenhado antes dos painéis. O
menu virou `shell/RunConfigMenu.qml` instanciado nos ShellOverlays (mesmo
padrão do ProjectEntryContextMenu), com dismiss por clique fora; estado
`configMenuVisible/X/Y` no RuntimeController; o header só emite
`configMenuRequested(x, y)` e o Main.qml mapeia as coordenadas para os
overlays. Regra derivada (vale para os próximos seletores da toolbar,
Target/Profile): **popup/dropdown nunca é filho do header — sempre
overlay**. No mesmo ajuste, "Configurar" → "Configurar CMake" (usuário
não entendeu o rótulo curto).

### Fatia M2.5 — Debugger via DAP (design 2026-07-09)

A maior peça do M2. Dividida em três sub-fatias executadas em ordem — cada
uma entrega valor verificável sozinha:

```text
M2.5a  Core DAP completo: sessão lldb-dap, breakpoints, continue/step,
       eventos mastigados p/ UI, sonda e2e com adapter real.   [esta]
M2.5b  UI de debug + C4: breakpoints na gutter, linha de execução,
       aba Debug (controles+saída), botão Debug na toolbar, atalhos.
M2.5c  Inspeção: stack/threads/variáveis no painel + breadcrumbs (resto
       da C4). Watch/evaluate e breakpoints condicionais ficam pós-M2
       (gatilho: uso real sentir falta).
```

**Decisões (e porquês):**

```text
- Adapter único v1: lldb-dap (LLVM), para C/C++ E Rust — já vem no pacote
  lldb do toolchain (id novo `lldb-dap` no inventário de tools; sugestão
  de install = a do lldb). `gdb -i dap` (gdb>=14) fica registrado como
  alternativa se o lldb-dap decepcionar em campo — trocar o adapter é
  trocar o spawn, o protocolo DAP é o mesmo (filosofia: orquestrar, nunca
  reimplementar debugger).
- Framing: DAP usa o MESMO envelope Content-Length do LSP (o corpo é que
  não é JSON-RPC: { seq, type, command/event }). As funções de framing de
  lsp/framing.rs viram pub(crate) e o DAP as reusa — uma única
  implementação de framing no repo.
- Alvo do debug (v1) = binário, resolvido "Automático" espelhando o run:
  cargo → único executável no topo de target/debug (erro claro: compile
  antes / mais de um, escolha explícita); cmake → único executável de
  .kinein/build (reusa o scan do run.rs). debug.start aceita { program }
  explícito para escapar da heurística. Run configs NÃO alimentam o debug
  (guardam comando shell, não binário) — o Target selector visual da spec
  unifica isso na C5+ (gatilho registrado). debug.start NÃO compila antes:
  build é ação explícita (Ctrl+F9); orquestração build-then-debug é
  melhoria futura com gatilho.
- Breakpoints moram no CORE (DebugManager), por arquivo (map file→lines,
  normalizado: ordenado, sem duplicata), em memória. setBreakpoints antes
  da sessão só guarda (verified: false); no launch o core REPLAYA todos
  depois do evento `initialized` e só então manda configurationDone (a
  dança correta do DAP). Persistir em .kinein/ fica para a fatia de
  Settings/Storage (gatilho).
- Eventos IPC enxutos e mastigados: a UI não fala DAP. event.debug.stopped
  já chega com file/line — o core faz stackTrace(nível 1) internamente ao
  receber o stopped do adapter. Esse enrichment roda em thread própria: a
  thread leitora é quem entrega respostas, ela nunca pode bloquear
  esperando uma (deadlock).
- Sessão única por workspace (igual run/terminal). debug.start com sessão
  viva → DEBUG_ERROR. Requests ao adapter têm timeout (recv_timeout) —
  adapter travado nunca congela o core (mesma regra do LSP).
- threadId do último stopped fica na sessão e dirige continue/step; pause
  consulta threads e pausa a primeira. exited traz exitCode, terminated
  fecha a sessão — o core emite UM event.debug.finished.
```

**Contrato (protocolo 0.28.0):**

```text
debug.start          { program? }           → { program }
debug.setBreakpoints { file, lines:[int] }  → { breakpoints:[{line,verified}] }
                       (lines vazio = limpar; file confinado ao workspace)
debug.continue|next|stepIn|stepOut|pause {} → { status: "ok" }
debug.stop           {}                     → { status: "ok" }

event.debug.started   { program }
event.debug.output    { category, line }      (stdout|stderr|console)
event.debug.stopped   { reason, file?, line?, threadId }
event.debug.continued {}
event.debug.finished  { exitCode? }

erros: NO_WORKSPACE; TOOL_NOT_FOUND (lldb-dap ausente, sugestão na
mensagem); INVALID_REQUEST (estado de sessão/alvo); INTERNAL_ERROR
(adapter); INVALID_PARAMS.
```

**Arquivos (M2.5a):**

```text
[feito] kinein-protocol/src/debug.rs + lib.rs  tipos + testes serde; 0.28.0
[feito] kinein-core/src/dap/mod.rs             DebugManager: store de
                                               breakpoints, slot da sessão,
                                               API pública
[feito] kinein-core/src/dap/session.rs         sessão viva: seq atômico,
                                               stdin compartilhado, pending
                                               map, thread leitora,
                                               handshake/launch/replay
[feito] kinein-core/src/dap/target.rs          resolução do binário
[feito] kinein-core/src/lsp/framing.rs         framing compartilhado
                                               (pub mod, itens pub(crate))
[feito] kinein-core/src/run.rs                 scan de executáveis pub(crate)
[feito] kinein-core/src/handlers/debug.rs      router debug.*
[feito] kinein-core/src/{lib,rpc,commands}.rs  manager + helpers + descriptor
[feito] kinein-core/src/tools.rs               lldb-dap no inventário
[feito] kinein-core/src/tests/debug.rs         guardas de dispatch
[feito] scripts/instalar-ambiente.sh           lldb/lldb-dap na verificação
[feito] docs/arquitetura/03 + ContextoIA                   contrato/estado
```

**Validação executada (2026-07-09):** 173 testes core + 43 protocol +
clippy pedantic/nursery verdes. Sonda e2e com lldb-dap 22.1.6 real, via
IPC stdio do core, PROVOU em C (kind cmake) e Rust (kind cargo): alvo
automático resolvido, breakpoint setado ANTES da sessão replayado no
launch e atingido (reason=breakpoint, file/line exatos), re-set com
sessão viva vem `verified: true` do adapter, step over caiu na linha
seguinte (reason=step), continue até `finished` com exitCode 0, saída do
programa capturada linha a linha, e `debug.stop` pós-morte não trava.

**Testes (M2.5a):** target: cargo com 0/1/N executáveis e cmake single;
store de breakpoints normaliza e faz roundtrip sem sessão; dispatch:
NO_WORKSPACE em todos, setBreakpoints rejeita arquivo fora do root,
INVALID_PARAMS de params malformados; framing continua coberto pelos
testes do LSP. Sessão real não entra no cargo test (lldb-dap não é
garantido no ambiente) — a prova é a **sonda e2e** fora do gate, como nas
fatias LSP: fixture C compilada com -g, setBreakpoints, debug.start,
stopped no breakpoint com file/line corretos, continue, finished, e o
mesmo fluxo num projeto cargo.

**UI (M2.5b) — FEITA em 2026-07-09:**

```text
[feito] ui/src/core_client.*                 propriedade debugging, 8
                                             invokables debug*, sinais
                                             debugStarted/Output/Stopped/
                                             Continued/Finished (handlers
                                             extraidos p/ complexidade)
[feito] ui/qml/debug/DebugController.qml     estado da sessao + breakpoints
                                             por arquivo (breakpointsByFile
                                             + breakpointsRevision para os
                                             bindings do gutter)
[feito] ui/qml/ipc/DebugEventRouter.qml      eventos → controller
[feito] ui/qml/panels/bottom/DebugPanel.qml  controles (Continuar/Pausar/
                                             Step Over/Into/Out/Parar) +
                                             saida; aba "Debug" no
                                             BottomTabBar/BottomPanelHost
[feito] ui/qml/editor/EditorTextSurface.qml  C4: gutter NOVA (numeros de
                                             linha em janela visivel,
                                             bolinha de breakpoint, clique
                                             = toggle) + linha de execucao
                                             destacada dentro do Flickable;
                                             altura de linha derivada de
                                             contentHeight/lineCount (sem
                                             drift em arquivo longo)
[feito] ui/qml/editor/EditorPane.qml         pass-through burro
[feito] ui/qml/shell/ShellWorkspaceHost.qml  fios gutter/painel ↔ controller
                                             (helpers com args-dependencia
                                             para rebind em troca de aba)
[feito] ui/qml/shell/TopHeaderBar.qml        botao Debug (vira ■ Debug com
                                             sessao viva) + ShellHeaderHost
[feito] ui/qml/shell/GlobalShortcuts.qml     Shift+F9/F9/F8/F7/Shift+F8
                                             (steps gated na sessao)
[feito] ui/qml/command/CommandDispatcher.qml "Debug" no Search Everywhere
[feito] ui/qml/workspace/WorkspaceUiResetter.qml  limpa debug no fechar
[feito] ui/qml/Main.qml + ui/CMakeLists.txt  composicao + registro
[feito] MANUAL.md + docs/roadmaps/20                  atual seção 4.1 Depurar + C4 status
```

**Regra de atalhos (pedido do usuario, 2026-07-09):** teclados de
notebook exigem Fn para F1-F12, entao TODO atalho com F-key tem uma
sequencia alternativa sem F-key (Shortcut.sequences), e toda acao de
atalho tem acionamento manual por botao/comando. Mapa: build Ctrl+Alt+B,
testes Ctrl+Alt+T, run Ctrl+Alt+R, parar run Ctrl+Alt+X, debug
Ctrl+Alt+D, continue Ctrl+Alt+C, step over/into/out Ctrl+Alt+N/I/U,
usos Ctrl+Shift+U, rename Ctrl+Shift+R, terminal Ctrl+`. Os rotulos da
aba Debug mostram a variante sem Fn. Vale como regra para atalhos
futuros.

Validacao M2.5b: build release+debug, qmllint estrito zero warnings,
smoke offscreen limpo (gate completo verde). Teste interativo real de
clique/atalho fica com o usuario (R7 de docs/roadmaps/20) — o pipeline por baixo
foi provado ponta a ponta na sonda da M2.5a. Atalhos JetBrains mantidos:
Shift+F9 debug, F9 continue, F8 step over, F7 step into, Shift+F8 step
out. Fica para a M2.5c: stack/threads/variaveis e breadcrumbs (resto da
C4).

### Fatia M2.5c — Inspeção (design 2026-07-09)

Fecha o M2: com stack e variáveis o debugger vira ferramenta de verdade.

**Decisões (e porquês):**

```text
- Dois métodos novos (protocolo 0.29.0), sempre da THREAD PAUSADA:
  debug.stackTrace {} e debug.variables { frameId? | ref? }. A UI nunca
  fala DAP: "scopes" não existem no contrato — debug.variables com
  frameId resolve scopes(frameId) DENTRO do core e responde as variáveis
  do primeiro escopo não-caro (Locals no lldb-dap); Globals/Registers
  ficam pós-M2 (gatilho: sentir falta). `ref` (variablesReference do
  DAP) é o handle de expansão de structs: variável com ref > 0 expande
  via debug.variables { ref }.
- A resposta ECOA frameId/ref (mesmo padrão do format.text que ecoa
  path): é assim que a UI correlaciona a resposta com o nó da árvore
  sem estado pendente no CoreClient.
- Exigem sessão pausada: NotStopped/NotRunning viram INVALID_REQUEST
  com mensagem clara. stackTrace limita a 20 frames (JetBrains-like;
  paginação só se o uso pedir). frameId? XOR ref? — os dois ou nenhum
  → INVALID_PARAMS.
- UI: com o processo pausado a aba Debug divide em [saída | frames |
  variáveis]; rodando, a saída ocupa tudo. Parou → stackTrace
  automático → frame do topo selecionado → variáveis dele. Clicar num
  frame abre o arquivo:linha dele e carrega suas variáveis. Variável
  com filhos expande/recolhe no clique (modelo flat com depth, filhos
  inseridos/removidos in-place — sem TreeView do Controls).
- C4 nesta fatia: (a) linha do cursor destacada (surfaceSelected, some
  quando há seleção); (b) breadcrumbs = caminho relativo do arquivo em
  barra fina acima do editor (segmentos › separados; segmento de
  símbolo LSP fica pós-M2). Gutter de diagnósticos ADIADO com gap real
  documentado: o sinal lspDiagnostics do CoreClient não é consumido por
  nenhum QML hoje — pré-requisito é um store de diagnostics por arquivo
  no EditorController (fatia própria, junto de Problems 2.0/C6).
```

**Contrato (protocolo 0.29.0):**

```text
debug.stackTrace {}              → { frames: [{ id, name, file?, line? }] }
debug.variables  { frameId }     → { frameId, variables: [...] }
debug.variables  { ref }         → { ref, variables: [...] }
  variável: { name, value, type?, ref }  (ref 0 = folha)
erros: INVALID_REQUEST (sem sessão/não pausado); INVALID_PARAMS
(frameId e ref juntos ou ausentes); INTERNAL_ERROR (adapter).
```

**Arquivos — FEITOS em 2026-07-09:** kinein-protocol/src/debug.rs
(+lib 0.29.0); dap/session.rs (stack_trace/frame_variables/
reference_variables + parsers testados); dap/mod.rs (delegação);
handlers/debug.rs (rotas + XOR de frameId/ref); tests de dispatch;
ui/src/core_client.* (debugStackTrace/debugVariablesForFrame/ForRef +
sinais com echo); DebugController (framesModel/variablesModel flat com
depth, seleção de frame navegável, expansão/colapso in-place);
DebugPanel ([saída | Frames | Variáveis] ao pausar); EditorTextSurface
(linha do cursor destacada, some na seleção); EditorPane (breadcrumbs
do caminho relativo, segmentos ›); ShellWorkspaceHost/BottomPanelHost/
Main/DebugEventRouter (fios).

**Validação executada (2026-07-09):** sonda e2e com lldb-dap real:
stack com 5 frames (soma no topo, main presente, file:line exatos),
locals do frame (a=2, b=3, struct p expansível) e expansão devolvendo
x=2/y=3 com echo de ref — além de todo o fluxo M2.5a re-provado.
177 testes core + 44 protocol; gate completo verde; smoke offscreen
limpo. Interativo (cliques nos frames/variáveis) fica com o usuário
(R7). **Com isso o M2 fecha** — pendências deliberadas registradas:
threads view, Globals/Registers, watch/evaluate, breakpoints
condicionais (gatilho: uso real), build-antes-do-debug, gutter de
diagnósticos (pré-requisito: store de diagnostics por arquivo).

## M3 — Git e refactor (médio/longo prazo)

Estruturado em 2026-07-09 (M2 fechado e validado pelo usuário no mesmo
dia — debugger funcional em teste real; bugs futuros serão reportados no
uso). Pergunta do marco: **"consigo commitar, revisar diff e refatorar
sem sair?"**. Filosofia inalterada: orquestrar o binário `git` (já no
inventário de tools) — NUNCA libgit2/reimplementação; só saída estável
de plumbing/porcelain versionado (`--porcelain=v2 -z`), nunca parsear
saída localizada. Régua Neovim: gitsigns + fugitive.

```text
M3.1 Status read-only: dominio git/ no core (git.status sincrono:
     branch/upstream/ahead/behind + entries staged/unstaged/untracked/
     conflito), branch + contador na status bar, arquivos coloridos na
     arvore do projeto. A UI dirige o refresh (open/save/fs-ops/manual e,
     desde 0.45.0, lotes do watcher externo) — serviço Git stateless.
M3.2 Diff + gutter de mudancas: git diff por hunks; marcas added/
     modified/removed na gutter do editor (regua gitsigns); visao de
     diff unificada do arquivo.
M3.3 Stage/unstage/commit: aba Git no painel inferior (lista de
     mudancas com stage/unstage, mensagem, commit); discard atras de
     confirmacao explicita (acao destrutiva).
M3.4 Blame/annotations no editor + log basico (pos-MVP, so depois do
     uso validar M3.1-M3.3).
Refactorings alem do rename (organize imports/extract) ficam por cima
do LSP quando os servers expuserem — regua JetBrains, Fase 5+.
```

### Fatia M3.1 — Git status read-only (design 2026-07-09)

**Decisões (e porquês):**

```text
- `git status --porcelain=v2 --branch -z`, cwd = workspace root: formato
  ESTAVEL e não-localizado, com staged (X) e worktree (Y) separados,
  rename com -z, e headers `# branch.*` (head/upstream/ahead/behind).
  Detecção de repo pela própria chamada: exit != 0 ou "not a git
  repository" → { repo: false } (não é erro — workspace sem git é
  estado normal; a UI esconde tudo).
- Paths do porcelain são relativos ao TOPLEVEL do repo; workspace pode
  ser subdiretório. O core converte para caminhos relativos ao ROOT do
  workspace (via `git rev-parse --show-toplevel`) e FILTRA entries fora
  dele — a UI nunca vê caminho que não consegue abrir.
- Síncrono (mesma classe do cargo.metadata, medido ~10ms; git status em
  repo médio é dezenas de ms). Gatilho para virar job: travar em repo
  gigante no uso real.
- Core STATELESS: git.status roda o comando na hora, nada de cache nem
  watcher dedicado de .git. Quem sabe QUANDO o status muda é a UI (ela mesma
  salva/renomeia/deleta): GitController pede refresh no workspaceOpened,
  apos save de arquivo, apos fs ops da arvore e por comando manual
  ("Git: Atualizar status" no Search Everywhere). Terminal/processos
  externos que mudam arquivos do workspace disparam o watcher `fs` desde o
  protocolo 0.45.0; mudança apenas interna de `.git` ainda aceita refresh
  manual.
- Shape mastigado por entry: { path, kind, staged } onde kind ∈
  modified|added|deleted|renamed|untracked|conflicted (consolidado do
  par XY pelo CORE — a UI não decodifica porcelain; conflito domina,
  senão vale o estado do worktree, senão o do index).
- UI: branch + "N alterações" na status bar (lado esquerdo, junto do
  root do workspace); arquivos coloridos na árvore. Decisão melhorada na
  implementação: NENHUM token novo — o Theme já tem cores semânticas que
  cobrem git (untracked/added = successSoft, modified/renamed =
  infoSoft, deleted = textDisabled, conflicted = errorSoft); ajuste fino
  na validação visual. Diretórios NÃO agregam estado dos filhos no v1
  (custo alto na árvore atual; gatilho: sentir falta no uso).
- Descobertas da sonda incorporadas: --untracked-files=all (sem isso o
  git colapsa diretório untracked numa entry "dir/", inútil para colorir
  e quebrada na conversão de prefixo) e filtro de `.kinein/` (metadados
  da própria IDE poluiriam o status a cada open; gatilho registrado para
  quem versionar .kinein de propósito).
```

**Contrato (protocolo 0.30.0):**

```text
git.status {} → { repo: bool,
                  branch?, upstream?, ahead?, behind?,
                  entries: [{ path, kind, staged }] }
  (branch ausente com HEAD destacada → detached: true + shortSha)
erros: NO_WORKSPACE; TOOL_NOT_FOUND (git ausente); INTERNAL_ERROR
(git falhou de verdade — repo:false NÃO é erro).
```

**Arquivos:**

```text
kinein-protocol/src/git.rs + lib.rs     tipos + testes serde; 0.30.0
kinein-core/src/git.rs                  runner (Command::output, padrão
                                        cargo.rs) + parser porcelain v2
                                        (-z, headers branch, rename,
                                        conflito) + conversão de paths
kinein-core/src/handlers/git.rs         rota git.status (guardas)
kinein-core/src/{lib,commands}.rs       cadeia + descriptor "Git:
                                        Atualizar status"
kinein-core/src/tests/git.rs            dispatch (NO_WORKSPACE; dir sem
                                        git → repo:false)
ui/src/core_client.*                    gitStatus() + gitStatusResolved
ui/qml/git/GitController.qml            estado: branch/counts/
                                        entriesByPath + revision (padrão
                                        breakpoints) + refresh()
ui/qml/Main.qml + roteador              fios de refresh (opened/saved/
                                        fs-ops) e resultado
ui/qml/shell/ShellStatusHost.qml        branch + contador
ui/qml/project/ProjectExplorer.qml      cor do nome via entriesByPath
ui/qml/Theme.qml                        gitAdded/gitModified/gitDeleted
ui/CMakeLists.txt                       registro
```

**Testes (core):** parser com fixtures reais do porcelain v2 (modified
unstaged, staged+unstaged, untracked, renamed com -z, conflito UU,
headers de branch com ahead/behind, detached); conversão root≠toplevel
filtra entries de fora; dispatch NO_WORKSPACE e diretório sem git →
repo:false. Sonda e2e fora do gate com git REAL: init temp repo,
commit, modificar/criar/stagear, git.status via IPC confere branch e
entries.

**Status: FEITA em 2026-07-09.** Core: 184 testes (parser+dispatch) e
sonda e2e com git real PROVOU: branch main + entries
modified/untracked/added-staged/renamed-staged corretas (G1), workspace
em subdiretório do repo vê só o próprio conteúdo com paths relativos
(G2) e diretório sem git responde { repo: false } sem erro (G3). UI:
GitController (gitKinds path→kind + revision, padrão breakpoints) +
GitEventRouter (refresh em save/create/rename/delete; git.status
automático no open, no C++), branch/↑↓/contador na status bar, nomes
coloridos na árvore, "Git: Atualizar status" no Search Everywhere,
reset no fechar workspace. Gate completo + qmllint + smoke verdes.

### Fatia M3.2 — Diff + gutter de mudanças (design 2026-07-09)

Régua gitsigns: marcas de added/modified/removed na gutter + diff do
arquivo sob demanda.

**Decisões (e porquês):**

```text
- Base do diff = HEAD (git diff HEAD -- <path>), como o gutter do
  JetBrains: o usuário pensa "o que mudou desde o último commit", não
  "desde o index". Base configurável (index) fica pós-MVP (gatilho).
- Diff do ARQUIVO EM DISCO, não do buffer: o gutter atualiza no save
  (mesmo momento do refresh do git.status) e na troca de aba — digitação
  ao vivo NÃO mexe nas marcas até salvar. Attach de buffer (diff do
  texto não salvo, como gitsigns faz) é fatia futura com gatilho: sentir
  as marcas "atrasadas" no uso real.
- Um método, duas granularidades: git.fileDiff roda git DUAS vezes no
  arquivo (barato): --unified=0 para os hunks estruturados do gutter
  (sem contexto = ranges exatos) e --unified=3 para o texto da visão de
  diff. hunks vêm do header @@ -a,b +c,d @@: b==0 → added (c..c+d-1);
  d==0 → removed (marca entre linhas, ancorada na linha c+1 do lado
  novo); senão → modified. A UI nunca parseia diff.
- Untracked: git diff não cobre; responde tracked:false + um hunk added
  do arquivo inteiro (o core conta as linhas) e text vazio — a UI mostra
  "arquivo novo" na visão de diff.
- path confinado ao workspace (canonicalize + starts_with, mesma regra
  de debug.setBreakpoints); resposta ECOA o path canônico (correlação e
  descarte de resposta stale na troca rápida de aba — padrão format.text).
- Gutter: barra de 3px encostada nos números — added successSoft,
  modified infoSoft; removed é um triângulo curto errorSoft no TOPO da
  linha âncora (mudança "entre linhas"). Lookup O(1): o GitController
  expande hunks num objeto js linha→kind (padrão gitKinds/breakpoints).
- Visão de diff: overlay GitDiffDialog (padrão dos diálogos): título com
  o path relativo, corpo monospace com +/− coloridos por linha, Esc
  fecha. Acionada por "Git: Diff do arquivo" (Search Everywhere). Painel
  dedicado de diff lado a lado fica com a M3.3 (stage por hunk decide o
  layout definitivo).
```

**Contrato (protocolo 0.31.0):**

```text
git.fileDiff { path } → { path (canônico), repo, tracked,
                          hunks: [{ kind: added|modified|removed,
                                    startLine, lineCount }],
                          text }
erros: NO_WORKSPACE; INVALID_PARAMS (path fora do workspace/inexistente);
TOOL_NOT_FOUND; INTERNAL_ERROR. repo:false/tracked:false NÃO são erros.
```

**Arquivos — FEITOS em 2026-07-09:** kinein-protocol/src/git.rs
(+lib 0.31.0); kinein-core/src/git.rs (file_diff com untracked
full-added + parse_hunks testado por fixture: added/modified/removed,
ranges com e sem vírgula); handlers/git.rs (rota + confinamento
canonicalize/starts_with); commands.rs ("Git: Diff do arquivo");
tests/git.rs (fora do workspace/inexistente → INVALID_PARAMS; sem git →
repo:false); ui/src/core_client.* (gitFileDiff + echo de path);
GitController (activeDiffPath + diffLineKinds linha→kind + revision;
diálogo com estado próprio e stale-drop pelo echo); gatilhos: troca de
aba (Connections em Main), save do arquivo ativo (GitEventRouter),
refresh() manual/status; EditorTextSurface (barra 3px na gutter:
added successSoft, modified infoSoft, removed = traço errorSoft no topo
da linha âncora; breakpoint deslocado 4px para conviver);
GitDiffDialog.qml (overlay z93, +/− coloridos, Esc/clique-fora fecha,
estados carregando/sem-mudanças/untracked) em ShellOverlays;
CommandDispatcher.

**Validação executada (2026-07-09):** sonda e2e com git real: arquivo
commitado modificado (linha trocada + linha nova) devolveu hunk
modified 1..2 e o texto U3 com "+um MUDADO" (G4); untracked devolveu
tracked:false + hunk added do arquivo inteiro (G5) — além de G1-G3
re-provadas. 186 testes core + 47 protocol; gate completo verde; smoke
offscreen limpo.

## Trilha E — Fluxo de digitação profissional (pedido do usuário, 2026-07-09)

Requisito literal do usuário: o fluxo de DIGITAÇÃO de código deve ficar
igual (ou o mais próximo possível) ao padrão das IDEs JetBrains/VS Code —
"quando for usar um `(` aparecer o segundo `)` automaticamente", e o
resto do pacote que essas IDEs trazem por padrão. Trilha própria (E de
editor), executada ENTRE fatias dos marcos — cada En é pequena, 100% UI
(EditorTextController/Surface), sem contrato IPC novo:

```text
E1 Auto-close de pares [FEITA em 2026-07-09, junto da M3.2]:
   - digitar ( [ { " ' abre e fecha: "()" com cursor no meio;
   - digitar o FECHADOR com o mesmo caractere à direita pula por cima
     (type-over), sem duplicar;
   - Backspace com o cursor entre um par vazio apaga os dois;
   - com SELEÇÃO ativa, digitar um abridor ENVOLVE a seleção (surround)
     em vez de substituí-la;
   - aspas: não duplicar quando o caractere anterior/seguinte é
     alfanumérico (evita don''t → don''''t) — regra VS Code.
   Implementação: EditorTextSurface.handleTypingKey/handlePairBackspace
   no Keys.onPressed do TextEdit (antes dos handlers de popup). Regras
   extras registradas: colchete antes de palavra/aspas não duplica
   (VS Code-like); Ctrl puro passa reto (atalhos), Ctrl+Alt (AltGr de
   layouts europeus) é caractere legítimo e entra no fluxo de pares.
   Validado por build+qmllint+smoke; teste de digitação real é do
   usuário (critério de aceite: lado a lado com VS Code).
E2 Enter inteligente [FEITA em 2026-07-10]:
   - Enter entre { e } abre bloco: linha extra indentada + fechador na
     linha própria (hoje o Enter já herda a indentação da linha);
   - continuação de comentário: Enter dentro de // ou /* */ repete o
     prefixo (// ou *) na linha nova — régua JetBrains.
   Implementação: EditorTextController.insertNewline, sem IPC e sem
   parser próprio. A continuação de bloco usa a última abertura `/*`
   ainda sem `*/` antes do cursor e preserva o alinhamento das linhas
   iniciadas por `*`.
   Sonda Qt Quick: divisão de {}, comentário //, primeira/segunda linha
   de bloco e regressão da indentação simples (5 casos, todos verdes).
E3 Polimento de digitação [FEITA em 2026-07-10]:
   - } digitado em linha só-whitespace desce/sobe para a indentação da
     linha do { casado (varredura reversa por profundidade; type-over
     da E1 tem precedência e não re-indenta);
   - Ctrl+W expande a seleção (palavra → conteúdo/par de ()[]{} →
     linha → bloco → documento) e Ctrl+Shift+W encolhe; escada
     heurística sem AST: o MENOR candidato que contém estritamente a
     seleção vence (versão LSP selectionRange fica registrada com
     gatilho);
   - Home alterna primeiro-texto ↔ coluna 0; Shift+Home estende a
     seleção; Ctrl+Home segue nativo.
   Implementação no EditorTextController (sonda Qt Quick com 35 casos
   verdes); Surface só emite sinais (padrão E2). O TextEdit aguentou
   tudo desta fatia sem briga — nenhum limite novo a registrar (os já
   conhecidos, inline hints/multi-cursor, seguem em M5.4). Design e
   descobertas: seção "Fatia E3".
Regras permanentes da trilha: nada de "engine" própria — tudo por cima
do TextEdit atual, testável no smoke; cada En atualiza o MANUAL (seção
Edição); comportamento sempre pode ser comparado lado a lado com VS Code
(critério de aceite do usuário).

**Princípio registrado (usuário, 2026-07-09): as "pequenas coisas"
acumuladas são o que torna a experiência fluida.** Vale além da trilha
E: toda fatia deve olhar os detalhes de ergonomia do caminho que toca
(foco certo, atalho com alternativa, estado visível, zero cliques
supérfluos) e registrar aqui os que não couberem nela — este plano é o
radar oficial dessas melhorias, para nenhuma se perder.
```

### Fatia E3 — Polimento de digitação (design 2026-07-10)

Fecha a trilha E do plano original: dedent do `}` digitado, Home
inteligente e expand/shrink selection estilo JetBrains — 100% UI, sem
contrato IPC novo, tudo por cima do TextEdit atual.

**Decisões (e porquês):**

```text
- Arquitetura igual à E2, não à E1: a lógica NOVA vive em
  EditorTextController (funções puras sobre o texto, testáveis pela
  sonda Qt Quick); a Surface só intercepta a tecla e emite sinal pela
  cadeia existente (Surface → EditorPane → ShellWorkspaceHost →
  EditorController → textController). A E1 ficou na Surface porque
  precisava DECIDIR consumir ou não a tecla (fallback nativo); aqui os
  três comportamentos são totalmente determinísticos, então dá para
  aceitar a tecla sempre e deixar o controller fazer tudo — e ganhar
  sonda de graça.
- Dedent do } digitado: só age quando a linha antes do cursor é 100%
  whitespace (é o caso "fechando bloco à mão"); acha o { casado por
  varredura reversa com contador de profundidade sobre o texto cru e
  copia a indentação DA LINHA do {. Strings/comentários não são
  tratados (mesma limitação registrada da E1 — sem engine própria).
  Sem par casado ou indentação já certa → insere } sem mexer em nada
  (nunca inventa). O type-over da E1 tem precedência e NÃO re-indenta:
  o fechador auto-inserido já nasceu no lugar certo.
- Home inteligente: Home alterna primeiro-caractere-de-texto ↔ coluna
  0 (régua JetBrains/VS Code); Shift+Home estende a seleção com a
  mesma lógica (âncora = extremidade oposta ao cursor). Ctrl+Home
  (início do documento) NÃO é interceptado — só NoModifier e Shift.
- Expand selection (Ctrl+W) / shrink (Ctrl+Shift+W): escada heurística
  SEM estado de degrau — a cada acionamento gera candidatos (palavra
  sob o cursor; linha sem indentação; linha inteira; conteúdo de cada
  par ()/[]/{} que envolve a seleção; par incluindo delimitadores;
  documento) e escolhe o MENOR que contém estritamente a seleção
  atual. Emergem os degraus JetBrains sem máquina de estados. Pares
  são achados numa varredura única com pilha (closer sem match é
  ignorado — tolerante a texto desbalanceado); strings/comentários de
  novo fora, documentado. Shrink usa pilha de seleções anteriores,
  invalidada quando o texto muda de tamanho ou a seleção atual não é
  mais a última expandida (guarda barata; troca de aba com mesmo
  comprimento E mesma seleção escapa — aceito e registrado).
  Alternativa AST-precisa existe (LSP textDocument/selectionRange),
  mas fatia E é 100% UI; fica registrada como melhoria futura com
  gatilho: primeira frustração real com a heurística.
- Ctrl+W está LIVRE no repositório (fechar aba não tem atalho) e é a
  memória muscular JetBrains; sem F-key, então não precisa de
  sequência alternativa (regra do repositório só exige para F-keys).
  Precedente M1.6: ação de digitação/seleção não ganha comando de
  palette (comandos vêm do core via command.list; adicionar comando
  quebraria o "100% UI" da trilha à toa).
- Correção de passagem em helper compartilhado (registrada aqui):
  lineStartAt(0) com texto começando em "\n" devolvia 1 (lastIndexOf
  com fromIndex 0 ainda olha o índice 0); guarda position <= 0 → -1.
  Afeta só o caso "cursor na posição 0 com primeira linha vazia", em
  que o comportamento antigo era errado para qualquer chamador.
```

**Contrato:** nenhum (fatia 100% UI; protocolo segue 0.32.0).

**Arquivos:**

```text
ui/qml/editor/EditorTextController.qml  insertCloserBrace/smartHome/
                                        expandSelection/shrinkSelection
                                        + guarda em lineStartAt
ui/qml/editor/EditorTextSurface.qml     sinais closerBraceRequested/
                                        smartHomeRequested; Keys p/
                                        Home e desvio do } digitado
ui/qml/editor/EditorPane.qml            re-emissão dos dois sinais
ui/qml/shell/ShellWorkspaceHost.qml     fiação sinal → controller
ui/qml/editor/EditorController.qml      fachadas com guarda
                                        editableFileOpen
ui/qml/shell/GlobalShortcuts.qml        Ctrl+W / Ctrl+Shift+W
```

**Testes (sonda Qt Quick offscreen, padrão E2):** dedent do } com par
casado/aninhado; } sem par e } com texto antes do cursor (inserção
crua); Home alternando e Shift+Home estendendo; escada completa de
expand (palavra → parênteses → linha → chaves → documento) e shrink
revertendo degrau a degrau.

**Fora (com gatilho):** dedent de } em type-over (o auto-close já
posiciona certo; gatilho: reclamação real); expand por AST/LSP
selectionRange (gatilho: heurística errar em uso diário); linha com
"\n" final como degrau próprio da escada (gatilho: idem).

**Execução (2026-07-10) — FEITA.** Descobertas incorporadas:

```text
- A escada tem um degrau emergente não previsto no design: o span de
  linhas da seleção (linha do início até a linha do fim) aparece entre
  o par mais externo e o documento — natural, mantido e coberto pela
  sonda.
- O TextEdit pode não aplicar exatamente o range pedido no select()
  (ex.: o "\n" final do documento). expandSelection registra a seleção
  REALMENTE aplicada (read-back) — sem isso o histórico do shrink
  invalida no topo da escada. Expansão sem efeito não vira degrau.
- Sonda Qt Quick (scratchpad, padrão E2): 35 casos verdes — dedent com
  par simples/aninhado/coluna-0 e inserção crua sem par ou com texto
  antes; Home alternando e Shift+Home estendendo com âncora; escada
  completa de expand (palavra → parens → linha → chaves → span →
  documento), estabilidade no topo, shrink degrau a degrau até
  colapsar e invalidação do histórico ao mover o cursor. Executar com
  QT_FORCE_STDERR_LOGGING=1 qml6 -platform offscreen (o logging padrão
  do Arch engole console.log; import de diretório com caminho absoluto
  falha silencioso no qml6 — usar Qt.createComponent).
- De passagem (fix-first): clippy pedantic pegou needless_pass_by_value
  em git/operations.rs (spawn_error, sobra da M3.3) — corrigido para o
  gate voltar a verde; e lineStartAt(0) corrigido conforme o design.
- Validação: gate completo scripts/verificar.sh TUDO VERDE + smoke
  offscreen pelo launcher (exit 124, zero output QML). qmllint exige
  rebuild do preset dev-local antes quando .qml muda de API (o lint lê
  o módulo copiado no build dir — comportamento já documentado no
  cabeçalho de verificar-qml.sh).
```

### Fatia E1b — auto-close com dono próprio e sonda (2026-07-17)

A E1 foi a única da trilha E sem sonda. O motivo está registrado na E3: ela
"ficou na Surface porque precisava DECIDIR consumir ou não a tecla", e a E2/E3
foram para o controller e "ganharam sonda de graça". O preço só apareceu agora:
as regras de par nunca tiveram teste nenhum, e o gate ficava verde sem olhar
para elas.

**O que mudou.** As REGRAS saíram para `EditorAutoClosePairs.qml` (`QtObject`
com `target: TextEdit`); a Surface continua decidindo o consumo no
`Keys.onPressed` — a razão registrada da E1 permanece intacta. Não foi para o
`EditorTextController` porque aquela cadeia é de sinais e não devolve valor para
`event.accepted`, que é justamente o que a E1 precisa. `autoCloseEnabled`
continua na Surface: é API que o ShellWorkspaceHost liga via EditorPane.
`EditorTextSurface.qml` 504 → 425 linhas (catraca: encolher é sempre aceito;
baseline atualizada na mesma fatia).

**Sonda nova:** `scripts/qml-harness/tst_autoclose.qml`, sobre um `TextEdit`
real. Provada capaz de reprovar antes de valer: quebrar `isWordChar` acusou
bitmask 3584 (os 3 checks dela), quebrar o backspace do par vazio acusou 192
(os 2 dele) — a doença da §0.2i era exatamente teste que não sabe falhar.

**Referência oficial atual, somente arquitetural (§2.1, procedimento
obrigatório):**

- Code OSS, revisão `85313ccbab834820137b97fbfc15a5ca5aa2a66b` (arquivo lido
  alterado pela última vez em `defdcd4e5dc2f22df1666a183bad6e70a45ca369`,
  2025-12-15), arquivo `src/vs/editor/common/cursor/cursorTypeEditOperations.ts`,
  licença MIT, modo de adaptação B. Invariantes extraídos: (a) o type-over é
  condicionado à ORIGEM do fechador — com `autoClosingOvertype` em "auto" (o
  padrão) só se pula o caractere que o próprio editor auto-inseriu, rastreado
  como intervalos que acompanham as edições; (b) aspas precedidas de barra
  invertida nunca fazem type-over; (c) só se auto-fecha quando o caractere
  seguinte é permitido OU é o fechador de outro par auto-fechado; (d) não se
  auto-fecha aspa depois de caractere de palavra. Nenhuma função, classe, teste
  ou texto de implementação foi copiado ou traduzido.

**DIVERGÊNCIA MEDIDA — a spec da E1, não só o código.** A E1 especificou
"digitar o FECHADOR com o mesmo caractere à direita pula por cima (type-over)"
e classificou o conjunto como "regra VS Code", com aceite "lado a lado com VS
Code". O invariante (a) acima mostra que o Code OSS é mais conservador: nós
pulamos QUALQUER fechador no cursor, ele só pula o que ele mesmo inseriu. Efeito
no usuário: em `foo(bar)` digitado à mão, com o cursor antes do `)`, digitar `)`
engole o caractere. O aceite lado-a-lado não pegou porque o caso exige um
fechador escrito à mão à direita do cursor. Registrado como check 10 do
`tst_autoclose.qml`, que hoje fixa o comportamento ATUAL e passa a reprovar
quando a correção entrar.

**Fatia própria, não feita aqui.** Corrigir exige rastrear a origem do fechador,
e é aí que mora o custo: um `int` de posição em QML não sobrevive a uma edição.
O Code OSS resolve com intervalos que acompanham o documento; o equivalente
nativo aqui é `QTextCursor` (que o Qt reposiciona sozinho a cada edição) no lado
C++, ou invalidação explícita da região em QML. Decidir isso é design, não
digitação — e misturá-lo na extração seria a §4 regra 9 ao contrário.

### Fatia M3.4 — Blame no editor + histórico básico (design 2026-07-10)

Fecha o M3 (Git MVP). Duas capacidades de LEITURA: "quem mudou esta
linha" (blame na gutter, toggle por comando) e "o que aconteceu neste
repo" (histórico de commits com diff por clique). Régua docs/roadmaps/21:
gitsigns blame_line + log básico do M5.2 antecipado para cá (decisão da
época: "blame/log basico" fecham o M3 juntos — ContextoIA).

**Decisões (e porquês):**

```text
- git.blame { path } usa `git blame --porcelain -- <path>` (formato
  ESTÁVEL e não-localizado; --line-porcelain repete metadado por linha
  e só engorda o payload). O core devolve GRUPOS (startLine/lineCount/
  sha/author/authorTime/summary/committed) — a UI expande para linha,
  como já faz com hunks do fileDiff. Metadado de sha repetido não vem
  de novo no stream porcelain: o parser memoiza por sha.
- Linha não commitada: o porcelain usa sha 0000... — a detecção é pelo
  sha zerado (estável), NUNCA pela string "Not Committed Yet". UI
  mostra "não commitado".
- authorTime é epoch (author-time do porcelain); formatação de idade
  relativa ("há 3d") é da UI — locale é problema de apresentação.
- Leituras seguem o padrão M3.1/M3.2: não-repo → repo:false; arquivo
  untracked → tracked:false com groups vazio (não é erro). Path
  confinado igual ao fileDiff (canonicalize + starts_with; blame de
  arquivo deletado não existe — precisa existir em disco).
- Blame é SÍNCRONO como status/fileDiff (por arquivo, dezenas de ms).
  Gatilho para virar job: travar em arquivo com histórico gigante.
- Diff/blame são do arquivo EM DISCO (mesma limitação registrada da
  M3.2; buffer sujo não aparece).
- git.log { maxCount? } usa formato ESTÁVEL:
  `git log --max-count=N -z --pretty=format:%H%x1f%h%x1f%an%x1f%at%x1f%s`
  (US \x1f separa campos, -z separa registros com NUL — summary pode
  conter qualquer coisa menos NUL). Default 100, clamp 1..=500.
- Repo recém-init (sem HEAD): `git rev-parse --verify --quiet HEAD`
  por exit code ANTES do log → entries vazio (estado normal), sem
  parsear o stderr localizado do git log.
- git.commitDiff { sha } → { sha (echo p/ correlação), text }:
  `git show <sha> --no-color --no-ext-diff --unified=3
  --pretty=format:` (header vazio; autor/summary a UI já tem da lista
  do log — payload só com o patch). sha validado no handler (4..64
  hex) antes de virar argv — nunca passar string arbitrária ao git.
  Merge commit mostra o combinado do git show (pode ser vazio em merge
  trivial; documentado). Não-repo → INVALID_REQUEST (NotARepo): a
  chamada só nasce da lista do histórico, que só existe com repo.
- UI blame: coluna extra na gutter do EditorTextSurface ("autor,
  idade" compacto, textMuted, mesma janela de delegates virtualizada),
  toggle pelo comando "Git: Blame do arquivo" no palette (sem atalho
  de teclado — paridade JetBrains, que também não dá atalho default a
  annotate; o acionamento manual exigido pela regra do repositório é o
  próprio comando). Blame segue o arquivo ATIVO: troca de aba com
  blame ligado re-consulta; save re-consulta; fechar tudo esconde.
  Echo de path descarta resposta velha (padrão format.text).
- UI histórico: a aba Git ganha duas vistas [Mudanças | Histórico]
  (chips no topo, padrão das sessões do Terminal M2.1) — commit tool
  window e log convivem na mesma aba; painel novo dedicado é overkill
  para uso próprio. Clique num commit abre o GitDiffDialog REUSADO
  (título "commit <shortSha> — <summary>", texto do commitDiff).
- O watcher 0.45.0 atualiza status/diff, não reconsulta o histórico:
  histórico recarrega ao abrir a vista e no refresh manual — mesmo princípio
  "a UI dirige o refresh" da M3.1.
```

**Contrato (protocolo 0.33.0):**

```text
git.blame      { path }        → { path, repo, tracked, groups: [
                                   { startLine, lineCount, sha,
                                     author, authorTime, summary,
                                     committed }] }
git.log        { maxCount? }   → { repo, entries: [
                                   { sha, shortSha, author,
                                     authorTime, summary }] }
git.commitDiff { sha }         → { sha, text }
erros: NO_WORKSPACE; INVALID_PARAMS (path fora do root/inexistente;
sha não-hex; maxCount fora de 1..=500); INVALID_REQUEST (commitDiff
fora de repo); TOOL_NOT_FOUND; INTERNAL_ERROR (git falhou).
```

**Arquivos:**

```text
kinein-protocol/src/git.rs       payloads novos (+lib.rs 0.33.0)
kinein-core/src/git/parse.rs     parse_blame (porcelain) + parse_log
kinein-core/src/git/operations.rs blame/log/commit_diff
kinein-core/src/git/mod.rs       re-exports
kinein-core/src/handlers/git.rs  rotas + validação de sha/maxCount
kinein-core/src/commands.rs      "Git: Blame do arquivo",
                                 "Git: Histórico"
kinein-core/src/tests/git.rs     parse fixtures + dispatch/guardas
ui/src/core_client.*             3 invokables + 3 sinais resolved
ui/qml/git/GitController.qml     estado blame (por linha, idade) +
                                 historyModel + commitDiff dialog
ui/qml/ipc/GitEventRouter.qml    3 handlers novos + blame follow no
                                 save
ui/qml/panels/bottom/GitPanel.qml vistas Mudanças|Histórico
ui/qml/editor/EditorTextSurface.qml coluna de blame na gutter
ui/qml/editor/EditorPane.qml + shell/ShellWorkspaceHost.qml  fiação
ui/qml/command/CommandDispatcher.qml  comandos novos
```

**Testes planejados:** parse_blame (grupos committed + sha zerado +
memoização de metadado + arquivo com 1 linha), parse_log (campos \x1f,
registros NUL, summary com espaço/aspas); dispatch: blame sem path →
INVALID_PARAMS, path fora do root → INVALID_PARAMS, log com maxCount 0
→ INVALID_PARAMS, commitDiff com sha inválido → INVALID_PARAMS, tudo
sem workspace → NO_WORKSPACE. Sonda e2e com git REAL: 2 commits de
autores distintos + edição local → blame com 3 grupos (2 shas + zero),
log com 2 entries (mais novo primeiro), commitDiff contendo a linha do
commit, blame de untracked → tracked:false, log em repo vazio →
entries [], blame/log fora de repo → repo:false.

**Fora (com gatilho):** blame por linha no hover/status bar (gatilho:
uso real pedir); log com paginação/filtro por arquivo e stash/branches
(M5.2); auto-refresh do histórico ao commitar (hoje: reabrir a vista);
blame de buffer sujo (junto do attach de buffer da M3.2).

**Execução (2026-07-10) — FEITA. M3 FECHADO.** Descobertas:

```text
- Bug de passagem corrigido (fix-first): closeDiffDialog() do
  GitController limpava a lista de mudanças e o stagedCount (herança da
  M3.3) — errado agora que o diálogo de diff também abre a partir do
  histórico. A limpeza da lista foi movida para clear() (troca de
  workspace), que é o único momento certo.
- Blame como coluna extra da gutter (não painel lateral): segue o
  arquivo ativo, toggle pelo comando "Git: Blame do arquivo" (sem
  atalho — paridade JetBrains, que também não dá atalho default ao
  annotate). Idade relativa é formatada na UI (min/h/d/m/a).
- Histórico e Mudanças são duas vistas (chips) da MESMA aba Git, não
  aba nova — commit tool window + log convivem, como no JetBrains.
  Clique num commit reusa o GitDiffDialog (título vira "commit <sha> —
  <resumo>"; emptyText próprio para merge sem diff textual).
- 3 clippy::assigning_clones (clone_into) no parser de blame pegos pelo
  gate — corrigidos.
- Testes: +5 (2 unit de parser blame/log, 3 dispatch: guardas de
  params/sha/maxCount + non-repo). Sonda e2e G11-G18 com git real
  (2 autores + edição local + untracked + repo vazio + fora de repo).
  Gate completo verde, smoke offscreen (exit 124).
```

### Fatia T1 — Switch header/source C/C++ (design 2026-07-10)

Primeira fatia da Trilha T (docs/roadmaps/21): micro-fatia de alto valor diário no
estilo CLion. Alterna entre `.h`/`.hpp` e `.c`/`.cpp`/`.cc` do MESMO
componente via a extensão do clangd — não é reimplementação, é orquestrar
o que o servidor já sabe.

**Decisões (e porquês):**

```text
- textDocument/switchSourceHeader é EXTENSÃO do clangd (não LSP padrão):
  params = TextDocumentIdentifier PLANO ({ uri }, sem envelope
  { textDocument }, ao contrário dos requests de posição); resposta =
  URI (string) ou null. O manager sincroniza o documento antes (clangd
  precisa conhecê-lo) e converte a URI de volta com o path_for_uri já
  existente.
- Gate de linguagem NO MANAGER: só o servidor "cpp" tem a extensão.
  Arquivo de outra linguagem → UnsupportedFile (INVALID_PARAMS limpo),
  nunca deixar rust-analyzer responder "method not found" cru. Régua:
  a lacuna é honesta e registrada, não fingida.
- Sem par → resposta path:None (não é erro; o clangd simplesmente não
  achou o contraparte). A UI mostra um aviso discreto, não um erro.
- Contrato ecoa nada além do path resolvido: a UI decide abrir a aba.
  Reusa o caminho de abertura de arquivo que definition/símbolos já
  usam (openDiagnostic-like) — zero código novo de abertura.
- Atalho Alt+O (memória muscular CLion) + alternativa sem F-key não é
  necessária (Alt+O não usa F-key); comando "C/C++: Alternar header/
  source" no Search Everywhere é o acionamento manual exigido. Alt+O
  está livre no repositório.
- Sync antes do request: reusa sync_document (didOpen/didChange). Custo
  desprezível; garante que um buffer sujo ainda não salvo resolve o par
  certo (o clangd usa o índice, o conteúdo do buffer não muda o mapa
  .h/.cpp — mas manter o documento aberto evita cold-open).
```

**Contrato (protocolo 0.34.0):**

```text
lsp.switchSourceHeader { path, content } → { path: string | null }
erros: NO_WORKSPACE; INVALID_PARAMS (arquivo fora do root; arquivo não
C/C++); LSP_UNAVAILABLE; TOOL_NOT_FOUND (clangd ausente);
INTERNAL_ERROR/timeout do servidor.
```

**Arquivos:** kinein-protocol/src/lsp.rs (params/result + lib 0.34.0);
lsp/manager.rs (switch_source_header: gate cpp + sync + request + parse
URI); lsp/mod.rs se precisar re-export; handlers/lsp.rs (rota); rpc.rs
reusa lsp_error_response; commands.rs (descriptor); ui/src/core_client.*
(invokable requestSwitchSourceHeader + sinal switchSourceHeaderResolved);
EditorController (fachada + abre o path resolvido reusando openDiagnostic);
EditorEventRouter (handler); GlobalShortcuts (Alt+O); CommandDispatcher.

**Testes:** dispatch — sem workspace → NO_WORKSPACE; arquivo .rs →
INVALID_PARAMS (gate de linguagem, sem clangd). Sonda e2e com clangd
REAL: par .h/.cpp de um componente → ida e volta resolvem o contraparte;
arquivo sem par → path:null.

**Fora (com gatilho):** criar o contraparte quando não existe (clangd
não faz; gatilho: pedido real); switch para .inl/.tpp templates (o
clangd já cobre parte via índice — verificar no uso).

**Execução (2026-07-10) — FEITA.** Descobertas:

```text
- Reuso máximo: params = FsWriteParams { path, content } (já existente,
  usado por documentSymbols/semanticTokens); result novo só com path
  opcional. path_for_uri (já existia) converte a URI do clangd de volta.
- A abertura do contraparte reusa documents.openDiagnostic(path,1,1) —
  zero código novo de abertura de arquivo.
- Clippy "too many lines" (101/100) em lsp_command_descriptors ao somar
  o descriptor: extraído lsp_core_command_descriptors + push do novo.
- RADAR (pequena coisa, sem gatilho ainda): quando NÃO há contraparte,
  o v1 apenas não navega — falta uma primitiva de "aviso discreto"
  (toast/status transitório) que a UI inteira não tem hoje. Nenhuma
  fatia criou isso; várias se beneficiariam (switch sem par, blame
  desligado, "nada a formatar"...). Candidata a micro-fatia de UI
  própria; NÃO inventar por baixo de outra fatia.
- Sonda e2e T1a-T1d com clangd 22 real (header↔source ida e volta +
  arquivo sem par → path null). Teste de dispatch: .rs → INVALID_PARAMS
  (gate de linguagem). Gate completo verde, smoke offscreen (exit 124).
```

### Fatia T6 — Diagnósticos no editor + Problemas rico (design 2026-07-11)

Trilha T, a maior lacuna diária de editor (docs/roadmaps/21). Régua explícita do
usuário: **experiência JetBrains** — o erro aparece na hora sublinhado
no código E na aba Problemas com detalhe. A infra de tempo real JÁ
existe (diagnóstico antes do design confirmou):

```text
Diagnóstico (leitura do código, não suposição):
- Digitar → EditorController.changeDebounce (600ms) → lsp.didChange →
  clangd/rust-analyzer republicam textDocument/publishDiagnostics →
  o core converte em event.lsp.diagnostics → JobsController.
  handleLspDiagnostics popula problemsModel por arquivo → aba Problemas.
  ISSO JÁ É TEMPO REAL e já chega na aba Problemas hoje.
- O que FALTA (o que esta fatia entrega):
  1. o editor NÃO consome diagnóstico nenhum (sem sublinhado, sem
     marca na gutter) — lspDiagnostics só vai para o JobsController;
  2. o Diagnostic do protocolo DESCARTA o fim do range, o `code`
     (E0425/nome do lint) e o tool source — parse.rs:42 só guarda o
     ponto inicial e a mensagem;
  3. não há navegação entre problemas;
  4. a aba Problemas mostra só ponto+mensagem, sem o code/detalhe.
```

**Decisões (e porquês):**

```text
- Enriquecer o Diagnostic comum (NÃO criar tipo novo): +endLine,
  +endColumn (1-based, fim do range) e +code (string; o LSP manda code
  string OU número — normalizar para string). Campos opcionais: build/
  quality preenchem quando tiverem; LSP passa a preencher sempre. Sem
  quebra de contrato (serde skip_serializing_if).
- Sublinhado = SpellCheckUnderline (ondulado) aplicado NO HIGHLIGHTER,
  reusando exatamente o mecanismo de spans por linha dos semantic
  tokens (setDiagnostics espelha setSemanticTokens). Cor por
  severidade. Multi-linha: por bloco, o span vai de (bloco==startLine?
  startChar:0) até (bloco==endLine? endChar:fim da linha). Zero engine
  nova; custo igual ao dos semantic tokens que já rodam.
- Offsets: o highlighter usa UTF-16 0-based dentro do bloco (QString é
  UTF-16). O store guarda o range CRU do LSP (0-based UTF-16) para o
  sublinhado E também line/column 1-based (do Diagnostic) para o salto
  do cursor e a lista de Problemas. Não reconverter à toa.
- Store por arquivo num CONTROLLER NOVO dedicado (DiagnosticsController,
  guardrail docs/arquitetura/17: concern próprio, não incha o JobsController).
  Alimentado pelo MESMO sinal lspDiagnostics (dois consumidores do
  mesmo evento: Problems no JobsController + editor no
  DiagnosticsController). O editor liga no arquivo ATIVO; troca de aba
  re-aponta. Navegação (próximo/anterior a partir do cursor, com wrap)
  vive aqui — funções puras, testáveis por sonda Qt Quick.
- Navegação: F2 / Shift+F2 (memória JetBrains) + alternativa sem F-key
  Ctrl+Alt+E / Ctrl+Alt+Shift+E (E de erro; regra do repo). Acionamento
  manual = a própria aba Problemas (clicar salta) — precedente M1.6:
  ação de navegação de editor é 100% UI, não ganha comando de core.
- Gutter: marca de severidade por linha com diagnóstico (ponto colorido
  na coluna da gutter) + ToolTip com a(s) mensagem(ns) da linha no
  hover — é o "ver o erro inline" sem depender da primitiva de toast
  que a UI ainda não tem (radar da T1).
- Aba Problemas enriquecida: cada linha ganha o `code` (dimmed) e
  ToolTip com a mensagem completa; ordena por arquivo→linha. Contadres
  de erro/aviso ficam para o Problems 2.0/C6 (o badge de contagem já
  existe na BottomTabBar).
- Foco da fatia = diagnóstico LSP (o tempo real). Sublinhar build/
  quality (compilador/clippy on-demand) precisa de range no parser de
  saída deles — RADAR, entra quando o Problems 2.0 unificar ranges.
```

**Contrato (protocolo 0.35.0):** só enriquece o `Diagnostic` comum
(usado por `event.lsp.diagnostics`, `event.build.diagnostic`,
`event.quality.diagnostic`):

```text
Diagnostic += endLine?: u64 (1-based), endColumn?: u64 (1-based),
              code?: string
event.lsp.diagnostics { path, diagnostics: [Diagnostic] } — agora com
range completo e code quando o servidor mandar.
```

**Arquivos:**

```text
kinein-protocol/src/diagnostic.rs   +endLine/endColumn/code (+lib 0.35.0)
kinein-core/src/lsp/parse.rs        diagnostics_event: fim do range + code
kinein-core/src/lsp/manager.rs      (só se precisar — testes)
ui/src/editor_highlighter.{h,cpp}   setDiagnostics/clearDiagnostics +
                                    underline ondulado por severidade
ui/qml/diagnostics/DiagnosticsController.qml  store byFile + navegação
ui/qml/ipc/JobsEventRouter.qml      encaminha lspDiagnostics também p/ ele
ui/qml/editor/EditorTextSurface.qml gutter marker + tooltip + setDiagnostics
ui/qml/editor/EditorPane.qml        fiação das props
ui/qml/editor/EditorController.qml  liga arquivo ativo + navegação
ui/qml/shell/ShellWorkspaceHost.qml fiação controller→pane
ui/qml/Main.qml                     instancia DiagnosticsController
ui/qml/shell/GlobalShortcuts.qml    F2/Shift+F2 + Ctrl+Alt+E(/Shift)
ui/qml/panels/bottom/ProblemsPanel.qml  code + tooltip + ordenação
ui/CMakeLists.txt                   novo QML
```

**Testes:** protocolo (serialização com/sem os campos novos); parser
LSP (range multi-linha + code string e code numérico → string);
sonda Qt Quick da navegação (próximo/anterior/wrap, arquivo sem
diagnóstico, ordenação por posição); sonda e2e com rust-analyzer E
clangd REAIS: escrever arquivo com erro conhecido (variável não usada /
`;` faltando) → didOpen/didChange → event.lsp.diagnostics chega com
startLine/endLine/code preenchidos.

**Fora (com gatilho):** sublinhado de build/quality (precisa de range no
parser de saída — Problems 2.0); contadores erro/aviso no header
(Problems 2.0/C6); mensagem no hover do MOUSE sobre o squiggle (precisa
hit-testing por range no TextEdit — gutter tooltip cobre o essencial);
quick-fix direto do squiggle (Alt+Enter no cursor já existe, M1.3).

**Execução (2026-07-11) — FEITA.** Descobertas:

```text
- Confirmado por leitura: a base tempo-real → aba Problemas JÁ existia
  (didChange debounced 600ms → publishDiagnostics → event.lsp.* →
  JobsController). A fatia foi tudo o que faltava em volta.
- Sublinhado no EditorHighlighter: setFormat do QSyntaxHighlighter
  SUBSTITUI o formato — aplicar só o underline apagaria a cor do
  texto. Solução: mesclar por caractere lendo format(pos) e somando
  SpellCheckUnderline (ondulado) + cor por severidade. Range vazio
  (';' faltando) marca ao menos 1 char; multi-linha cobre por bloco.
- Reatividade do store: mutar byFile no lugar NÃO notifica o QML — as
  propriedades derivadas (editorSpansList/gutterMap) dependem de
  `revision` (padrão do diff gutter da M3.2). handleLspDiagnostics só
  bumpa revision quando o arquivo é o ativo.
- Tooltip da gutter é próprio (a UI não usa QtQuick.Controls): popup
  Rectangle com Text; largura via min(implicitWidth, máx) para evitar
  o binding circular do WordWrap. MouseArea da marca com
  acceptedButtons NoButton para o clique de breakpoint passar reto.
- Navegação F2/Shift+F2 (+Ctrl+Alt+E/Shift) no EditorController
  consultando o DiagnosticsController (nextDiagnostic/prevDiagnostic
  ordenado com wrap); salto reusa textController.goToLine. Sem comando
  de palette (precedente M1.6: navegação de editor é 100% UI; a aba
  Problemas é a superfície manual).
- Aba Problemas ganhou o `code` (dimmed) por linha; contadores erro/
  aviso ficam para o Problems 2.0 (badge de contagem já existe).
- Sonda Qt Quick (17 casos): store, spans 0-based, multi-linha,
  gutter com severidade mais forte por linha, navegação com wrap,
  republish vazio limpa, clear. Sonda e2e com clangd E rust-analyzer
  REAIS: erro conhecido → event.lsp.diagnostics com range+code
  (clangd: code 'undeclared_var_use', range L2:12→L2:13; r-a: erro de
  sintaxe com range). Gate completo verde; smoke offscreen (exit 124).
- Pendente do usuário: validação visual (R7 de docs/roadmaps/20 — squiggle e
  gutter lado a lado com JetBrains) e teste de digitação real.
```

### Fatia CR1 — Conforto de leitura e digitação (feedback do usuário, 2026-07-11)

Fatia 100% UI (sem contrato IPC novo) a partir do feedback direto do
usuário após a T6. Diagnóstico antes do design (sonda de semantic tokens
com clangd real):

```text
- Autocomplete LSP (C/C++ e Rust) JÁ existe e atende o pedido: dispara
  sozinho ~250ms após digitar (também após "." e "::"), popup não
  invasivo, Tab/Enter aceita, Esc fecha. Nada a implementar — confirmado
  por leitura (EditorCompletionController) + uso nas sondas.
- clangd e rust-analyzer JÁ prontos (usados nas sondas T6/semtokens).
- Semantic tokens do clangd JÁ colorem std→namespace, cout→variable,
  endl/printf→function, string/vector→class, main→function (sonda).
- Gaps reais: (a) janela PRÉ-LSP (primeiros ~1-2s) é só regex, que não
  conhece cout/printf/std; (b) cout/cin ficam cor de VARIÁVEL (quase
  branco) mesmo com clangd — o usuário quer que se destaquem.
```

**Decisões (e porquês):**

```text
- Marca de diagnóstico mais visível (pedido literal): dot 6→8px com
  borda de contraste + número da linha com diagnóstico tingido pela
  severidade. Sem selo verde de "correto": a ausência de marca JÁ é o
  estado correto (padrão de todas as IDEs); contadores erro/aviso na
  status bar ficam para o Problems 2.0 (registrado).
- Realce da stdlib por REGEX (fallback pré-LSP e permanente onde o
  semantic concorda): funções C (printf/scanf/mem*/str*/malloc...),
  containers/tipos std (string/vector/map/optional/size_t/uintN_t...)
  e streams (cout/cin/cerr/clog/endl). NÃO inventa cor — reusa os
  tokens de docs/05 (funções=gold kFunctionRgb, tipos=teal kTypeRgb).
  Semantic tokens (aplicados DEPOIS) refinam/sobrepõem onde existirem —
  sem conflito, o regex é a base.
- cout/cin/cerr/clog: o clangd os marca "variable" (quase branco). Para
  manterem destaque, um override PÓS-semantic recolore só esse conjunto
  icônico de objetos de stream (regex fixa). Único caso em que a UI
  sobrepõe o semantic de propósito, e por um motivo de leitura
  registrado (o usuário os quer distintos de variáveis locais).
- Variáveis continuam quase-brancas (kVariableRgb #cdd6e4) — já é o
  "fixo/branco" que o usuário pediu; nada a mudar.
- Rust (uso "em breve"): rule de macro `\w+!` (println!/vec!/format!...)
  como meta, e tipos std (Vec/String/Option/Result/Box/Rc/Arc/HashMap/
  Vec...) como tipo. Mantém a linguagem colorida antes do rust-analyzer.
- #include <...> auto-close CONTEXTUAL: digitar "<" logo após
  `#include ` (linha casa `^\s*#\s*include\s+`) insere "<>" com cursor
  no meio. É o único caso de "<" seguro. `<<`/templates `<>` genéricos
  ficam FORA: "<" é comparação, template, shift — auto-inserir "<<"
  erraria na maioria (o próprio usuário levantou isso). A "inteligência"
  de `cout <<` viria do LSP (snippet de completion), não de heurística
  de par — registrado como ideia, não implementado.
```

**Contrato:** nenhum (100% UI; protocolo segue 0.35.0).

**Arquivos:** ui/src/editor_highlighter.{h,cpp} (rules stdlib C++/Rust +
applyStdlibOverride pós-semantic); ui/qml/editor/EditorTextSurface.qml
(marca de diagnóstico maior + tingido; `#include <>` no handleTypingKey).

**Validação:** gate verificar.sh + smoke offscreen; sonda de semantic
tokens confirma que o regex não quebra o que o clangd já colore (o
override só toca cout/cin/cerr/clog). Realce fino e o `#include <>` são
de aceite VISUAL/digitação do usuário (como E1/E3).

**Fora (com gatilho):** `<<`/template auto-insert (ambíguo; via snippet
LSP se algum dia); contadores erro/aviso na status bar (Problems 2.0);
modifiers de semantic token (defaultLibrary) para marcar TODA a stdlib
sem lista fixa — precisaria plumbar modifiers no protocolo (gatilho: a
lista fixa ficar insuficiente no uso).

**Execução (2026-07-11) — FEITA.** Descobertas:

```text
- setFormat do QSyntaxHighlighter SUBSTITUI, então o override pós-
  semantic de cout/cin/cerr/clog roda DEPOIS de applySemanticSpans e
  ANTES de applyDiagnosticSpans (ordem: rules → block → semantic →
  stdlib → diagnostics), para o sublinhado de diagnóstico (merge por
  caractere) ainda pegar por cima.
- As listas de stdlib reusam keywordPattern (mesmo \b(?:...)\b dos
  keywords); nenhuma cor nova (só kTypeRgb/kFunctionRgb de docs/05).
  clang-format reflowa as listas — rodar clang-format -i.
- Confirmado por sonda: semantic tokens seguem intactos (20 tokens no
  snippet); o regex é só fallback e o override toca apenas os 8 nomes
  de stream. Variáveis continuam kVariableRgb (o "branco" pedido).
- #include <> é o ÚNICO "<" que auto-fecha (regex de linha
  ^\s*#\s*include\s+$ antes do cursor). Aceite de digitação é do
  usuário (lado a lado com CLion), como E1/E3.
- Gate completo verde; smoke offscreen (exit 124). Realce fino e a
  marca de diagnóstico pedem validação VISUAL do usuário (R7).
- RADAR reforçado: a "inteligência" de cout << (e de sugerir
  bibliotecas após #include <) é trabalho do SNIPPET de completion do
  LSP, não de heurística de par — quando/se entrar, é via o autocomplete
  que já existe, não novo auto-close. Registrado para não se perder.
```

### Fatia M4.1 — Settings/Storage com schema (design 2026-07-11)

Primeira fatia do M4 (docs/roadmaps/21). É a fatia que destrava format-on-save,
tamanho de fonte, check-no-save e perfis — por isso o foco é a
INFRAESTRUTURA (storage + get/set + UI) provada por consumidores reais.

**Decisões (e porquês, honrando docs/roadmaps/21 M4.1):**

```text
- Dois níveis: GLOBAL (XDG: $XDG_CONFIG_HOME ou ~/.config, então
  kinein-vectis/settings.json) e por-WORKSPACE (.kinein/settings.json).
  Workspace SOBREPÕE global campo a campo. Mesmo padrão de robustez do
  runconfigs.json: schemaVersion + arquivo inválido/schema desconhecido
  → tratado como VAZIO (nunca quebra). Sem crate nova (dirs): XDG por
  std::env.
- Contrato (padrão de mutação do repo — resposta = estado COMPLETO):
  settings.get {} → { settings (efetivo), global, workspace }
  settings.set { scope: global|workspace, values } → mesmo shape novo.
  `values` é PARCIAL (merge campo a campo no escopo; campo ausente não
  muda). "Efetivo" = default ← global ← workspace. Reverter/limpar um
  override fica pós-v1 (registrado; merge só sobrescreve com Some).
- Settings são TIPADOS (docs/roadmaps/21: sem engine genérica de formulário).
  SettingsValues tem campos opcionais (o que está setado no escopo);
  EffectiveSettings tem todos resolvidos com default. Cada setting novo
  adiciona seu campo + seu controle na UI.
- Primeiros consumidores REAIS (3 dos 4 de docs/roadmaps/21; o 4º registrado):
  1. editorFontSize (u32, default 14): SettingsController escreve em
     Theme.fontSizeEditor (que deixa de ser readonly). Validação 8..=40
     no handler (fora → INVALID_PARAMS).
  2. autoClosePairs (bool, default true): gate do auto-close da E1 no
     EditorTextSurface.handleTypingKey.
  3. formatOnSave (bool, default false): Ctrl+S formata e SÓ ENTÃO salva.
     Assíncrono robusto: pendingSaveAfterFormat salva no resolved E no
     failed do format.text (servidor ausente/timeout não trava o save).
  ADIADO com motivo: diffBase (head|index) precisa de parâmetro `base`
  no git.fileDiff (mudança no domínio git) — entra como setting #4 numa
  micro-fatia própria (não inventado sem consumidor; consumidor existe,
  só custa o param no git).
- UI: SettingsDialog é um overlay simples (ShellOverlays), edita o
  escopo GLOBAL no v1 (caso comum), mostra os valores efetivos. Editar
  por-workspace pela UI é adição pequena futura (a resolução já honra o
  arquivo de workspace se existir — power user edita à mão). Atalho
  Ctrl+Alt+S (JetBrains; sem F-key) + comando "Configurações".
  First Run/onboarding NÃO entra aqui (é M4.4).
```

**Contrato (protocolo 0.36.0):**

```text
settings.get {} → SettingsResult
settings.set { scope: "global"|"workspace", values: SettingsValues }
             → SettingsResult
SettingsValues { formatOnSave?: bool, editorFontSize?: u32,
                 autoClosePairs?: bool }
SettingsResult { settings: EffectiveSettings, global: SettingsValues,
                 workspace: SettingsValues }
EffectiveSettings { formatOnSave, editorFontSize, autoClosePairs }
erros: NO_WORKSPACE (set scope=workspace sem workspace);
INVALID_PARAMS (editorFontSize fora de 8..=40; scope inválido).
```

**Arquivos:** kinein-protocol/src/settings.rs (+lib 0.36.0);
kinein-core/src/settings.rs (global XDG + workspace, merge/resolve/
schema) + lib.rs `pub mod settings`; kinein-core/src/handlers/
settings.rs + handlers.rs + dispatch em lib.rs; commands.rs
("settings.get" → "Configurações"); tests/settings.rs + tests/mod.rs;
ui/src/core_client.* (settingsGet/settingsSet + sinal resolved);
ui/qml/settings/SettingsController.qml + ipc/SettingsEventRouter.qml +
settings/SettingsDialog.qml; Theme.qml (fontSizeEditor gravável);
EditorTextSurface (gate auto-close); EditorController (save-after-
format); GlobalShortcuts (Ctrl+Alt+S); CommandDispatcher; Main.qml +
ShellOverlays + ui/CMakeLists.txt.

**Testes:** core — resolve (workspace sobrepõe global; defaults),
merge (set parcial não apaga outros campos), schema inválido → vazio,
validação de fonte, dispatch (set scope=workspace sem workspace →
NO_WORKSPACE; fonte inválida → INVALID_PARAMS). Sonda e2e via stdio:
set global fonte 16 → efetivo 16; abrir workspace, set workspace
autoClosePairs=false → efetivo reflete override; persiste entre dois
processos do core; schema desconhecido no arquivo → default.

**Fora (com gatilho):** diffBase (param no git.fileDiff); reverter/
limpar override pela UI; editar workspace pela UI; format-on-save só
formata linguagens suportadas (rust/cpp) — outras salvam direto;
perfis de rigor Strict/Balanced/Relaxed (M4.5, consomem este storage).

**Execução (2026-07-11) — FEITA. M4 iniciado.** Descobertas:

```text
- Espelhou runconfig.rs (schema + invalido→vazio). Global via XDG por
  std::env (sem crate dirs). SettingsFile usa #[serde(flatten)] para o
  arquivo ficar { schemaVersion, formatOnSave, ... } plano.
- SettingsValues SEM deny_unknown_fields (forward-compat do arquivo);
  SettingsSetParams COM (pega typo da UI). clippy pediu quebrar o 1º
  parágrafo do doc e tornar settings_result função livre (unused_self).
- TESTE não pode tocar o ~/.config real: os testes em processo só
  exercitam scope=workspace (.kinein isolado) + validações; o scope
  GLOBAL só na sonda e2e, que roda o core como SUBPROCESSO com
  XDG_CONFIG_HOME próprio (isolamento limpo). Sonda M1-M6 provou
  default, set global, validação, override workspace, herança,
  persistência entre 2 processos e schema inválido→default.
- format-on-save assíncrono robusto: pendingSaveAfterFormat salva no
  handleFormatResolved E no handleRequestFailed("format.text") — se o
  format falhar/timeout, o Ctrl+S salva mesmo assim (não trava). O
  EditorEventRouter passou a encaminhar a falha de format.text.
- Theme.fontSizeEditor deixou de ser readonly; o SettingsController
  escreve o efetivo nele. autoClose threaded EditorController →
  ShellWorkspaceHost → EditorPane → EditorTextSurface (early-return em
  handleTypingKey/handlePairBackspace quando off).
- SettingsDialog e SettingsToggleRow são próprios (a UI não usa
  QtQuick.Controls); atalho Ctrl+Alt+S + comando "Configuracoes".
  Edita o escopo GLOBAL no v1; a resolução honra o arquivo de workspace
  se existir. Gate completo verde; smoke offscreen (exit 124).
- ARMADILHA registrada: `cargo test`/gate NÃO recompilam
  target/debug/kinein-core; rodar `cargo build -p kinein-core` antes de
  sondas e2e (senão o binário está stale e some settings.* →
  METHOD_NOT_FOUND enganoso).
```

### Fatia M4.3 — Robustez: recuperação de crash do core (design 2026-07-11)

Fecha (parte A) o M4.3 do docs/roadmaps/21. Hoje se o `kinein-core` morre
(`handleFinished`) a IDE só marca "desconectado" e fica morta. Objetivo:
o core cair → a IDE se recupera sozinha em ~2s com o MESMO workspace,
sem reiniciar a janela (critério de aceite do docs/roadmaps/21).

**Decisões (e porquês):**

```text
- Só C++ do CoreClient (sem contrato IPC novo). handleFinished passa a:
  se a saída foi INESPERADA (crash OU exit enquanto havia workspace e não
  foi shutdown intencional) → RELANÇAR o core e, quando ele subir,
  REABRIR o último root.
- Preservar as abas/edições: reabrir o MESMO root NÃO limpa a UI
  (WorkspaceController só limpa em root diferente/vazio). Mas o
  handleWorkspaceOpened dispara session restore (lê o disco) que
  SOBRESCREVERIA edições não salvas. Solução: na RECUPERAÇÃO, a UI IGNORA
  o sessionRestored (flag recovering no CoreClient exposta à QML) — as
  abas já estão na UI; não reler do disco.
- m_lastWorkspaceRoot: guardado quando um workspace abre com sucesso, NÃO
  limpo no crash. É o que a recuperação reabre. Fechar workspace de
  propósito limpa (não recupera "vazio").
- Anti-loop: se o core cair de novo em janela curta (< ~4s) repetidas
  vezes (>=3), PARAR de tentar e mostrar estado de erro persistente
  ("core caiu repetidamente — ver log"); senão vira loop de fork.
  Contador zera após um período estável rodando.
- Shutdown intencional (fechar a janela) NÃO recupera: flag
  m_shuttingDown setada no destrutor/close antes de matar o processo.
- Re-sync do LSP: o servidor LSP morre junto do core; ele re-sobe
  lazy no próximo request. Após recuperar, a UI re-pede semantic tokens
  do arquivo ativo (highlighting/diagnóstico do arquivo atual voltam sem
  esperar edição). LSP.restart explícito (parte B do M4.3) e limpeza de
  jobs órfãos (parte C) ficam para uma fatia seguinte — registrado.
- Status bar: "recuperando..." durante, volta ao normal ao reconectar.
```

**Contrato:** nenhum (100% C++ do CoreClient; protocolo segue 0.37.0).

**Arquivos:** ui/src/core_client.h (m_lastWorkspaceRoot, m_shuttingDown,
m_recovering + property `recovering`, restart tracking); ui/src/
core_client_process.cpp (handleFinished recupera; handleStarted reabre
root na recuperação); ui/src/core_client_dispatch.cpp (guardar
lastWorkspaceRoot no open; expor recovering); ui/qml/ipc/
WorkspaceEventRouter.qml ou EditorEventRouter (ignorar sessionRestored
quando recovering); status bar mostra recovering.

**Validação:** gate + smoke; **teste de aceite manual/sonda:** abrir a
IDE com um workspace, `kill -9` no kinein-core → em ~2s a IDE reconecta
com o mesmo workspace e as abas; matar repetidamente → para no anti-loop.
(A parte de kill exige o app vivo; validar via instrumentação offscreen:
matar o processo filho e checar que reconecta.)

**Fora (com gatilho):** parte B (lsp.restart { language } + auto-restart
após N timeouts) e parte C (cancelar jobs órfãos no shutdown) — fatia
M4.3b; preservar edições NÃO salvas ANTES do crash (o buffer da UI
sobrevive, mas se a UI também cair é outra história — fora do escopo).

**Execução (2026-07-11) — FEITA (parte A).** Descobertas:

```text
- O destrutor do CoreClient JA desconecta os sinais antes de fechar
  limpo, então handleFinished só dispara em saída INESPERADA — não
  precisou de flag de shutdown intencional.
- Fluxo: handleFinished (crash) -> guarda anti-loop (>=3 quedas em 4s
  pausa) -> setRecovering(true) + status "recuperando..." -> start()
  -> handleStarted reabre m_lastWorkspaceRoot -> handleWorkspaceOpened
  com m_recovering PULA o sessionRestored (UI mantém as abas) ->
  setRecovering(false) + emit recovered() -> EditorEventRouter
  re-pede semantic tokens do arquivo ativo (LSP novo re-sincroniza).
- Reabrir o MESMO root não limpa a UI (WorkspaceController só limpa em
  root diferente/vazio) — chave para preservar abas/edições.
- TESTE DE ACEITE (real, offscreen): app com workspace aberto ->
  kill -9 no kinein-core filho -> log mostrou desconectado ->
  recuperando -> iniciando -> conectado -> RECOVERED com tabs=1
  (abas preservadas) e um NOVO kinein-core vivo. Anti-loop e status
  na barra funcionam. Gate completo verde; smoke offscreen (exit 124).
- ARMADILHA de teste: matar o core CERTO (filho do app via pgrep -P),
  não um kinein-core stale de sonda anterior.
```

### Fatia S1 — Rede de segurança contra perda de dado (2026-07-11)

Design, problemas e status vivem em **`docs/seguranca/23`** (a fatia é grande e o
usuário pediu um registro explícito dos problemas encontrados). Resumo:
`fs.write` virou ATÔMICO (temp+fsync+rename) e o buffer não salvo passou a
ser autosalvo em `SQLite` (`draft.*`, protocolo 0.40.0), recuperado no
`workspace.open` após crash. FEITA e validada e2e. Pedida pelo usuário
ANTES do dogfooding.

### Fatia M4.3b — LSP restart + jobs órfãos (design 2026-07-11)

FECHA o M4.3 (partes B e C que a parte A registrou como pendentes) e, com
isso, o marco M4. Dois problemas de robustez independentes:

**Parte B — o LSP travou.** Hoje um servidor que para de responder só
gera `LspError::Timeout` por request; não há como ressuscitá-lo sem
fechar o workspace. Objetivo: (1) comando explícito "reiniciar servidor
LSP" (como o "Restart language server" do VS Code) e (2) auto-restart
depois de N timeouts seguidos.

**Parte C — jobs órfãos.** No shutdown (limpo OU stdin EOF quando a UI
morre) um build/test/quality em andamento deixa o `cargo`/`cmake`/`lldb`
filho rodando sem dono. Objetivo: cancelar tudo que estiver vivo ao sair.

**Decisões (e porquês):**

```text
PARTE B (LSP restart):
- lsp.restart { language? } → reinicia UM servidor (ex.: "rust"|"cpp");
  sem language, reinicia TODOS os vivos. restart = kill + remove do
  handle; sobe lazy no próximo request (padrão já existente). Responde
  { restarted: [languages] }.
- Auto-restart: contador de timeouts CONSECUTIVOS por linguagem no
  LspManager (HashMap). send_request: timeout -> streak+=1; resposta OK
  -> streak=0. streak >= MAX_TIMEOUT_STREAK (3) -> restart daquele
  servidor + zera streak. 3 timeouts de 4s = ~12s preso antes de agir.
- Re-sync do documento: o handle novo não sabe dos arquivos abertos
  (ServerHandle guarda versão/hash, NÃO o conteúdo). Depois de reiniciar,
  emite event.lsp.restarted { language }; a UI re-sincroniza o arquivo
  ativo via refreshSemanticTokens() (mesmo caminho do recovered() da
  parte A: semantic_tokens chama sync_document -> did_open de novo ->
  highlighting + diagnósticos voltam). Uma mecânica só para crash e
  restart.
- UI: comando "LSP: Reiniciar servidor" no Search Everywhere/paleta ->
  coreClient.lspRestart(language vazio = todos). event.lsp.restarted ->
  EditorEventRouter re-sincroniza o arquivo ativo.

PARTE C (jobs órfãos):
- JobManager.cancel_all(): sinaliza cancel a todos os jobs canceláveis
  ainda Running (rápido, não bloqueia). Reusa o cancel() cooperativo.
- Drop for JobManager: cancel_all() + DRAIN limitado (~500ms, poll a
  cada 20ms) até nenhum job estar Running/CancelRequested. Os runners
  observam o cancel a cada 50ms (process.rs) e MATAM o filho; o drain dá
  essa janela antes do processo sair. Drop cobre TODOS os caminhos de
  saída: core.shutdown, stdin EOF (UI morta) e unwind de panic.
- core.shutdown também chama cancel_all() (sinal pronto + entrega as
  notificações event.job.* enquanto o canal está vivo); o Drain fica no
  Drop, depois da resposta escrita — não atrasa o shutdown response.
- Drop é BARATO quando não há job (cancel_all e o drain saem na hora);
  não pesa nos testes que criam/destroem Core sem jobs.
```

**Contrato (protocolo 0.39.0):** `lsp.restart { language? }` →
`{ restarted: [string] }`; novo evento `event.lsp.restarted { language }`.
Parte C é 100% interna (sem contrato).

**Arquivos:** kinein-protocol/src/lsp.rs (LspRestartParams/Result, lib
0.39.0); kinein-core/src/lsp/manager.rs (timeout_streak, restart_language,
restart_all, auto-restart no send_request, emit event.lsp.restarted);
handlers/lsp.rs (lsp.restart); jobs/manager.rs (cancel_all + Drop drain);
lib.rs (core.shutdown chama cancel_all); ui/src/core_client* (lspRestart
+ sinal lspRestarted); ui/qml/ipc/EditorEventRouter (onLspRestarted ->
refreshSemanticTokens); comando na paleta.

**Testes:** core — restart_language remove o handle e zera streak;
send_request incrementa/zera streak (unit com servidor fake se viável,
senão via o contador direto); cancel_all sinaliza jobs Running e o drain
espera encerrar. Sonda e2e: (B) lsp.restart responde restarted; (C)
subir um build longo e mandar core.shutdown -> processo cargo não fica
órfão (checar via pgrep após o core sair).

**Fora (com gatilho):** shutdown educado do LSP (textDocument/exit em vez
de kill) — kill é suficiente e simples; hard-kill de filho travado por
PID (o cancel cooperativo + drain cobrem o caso comum); backoff/limite de
auto-restart do LSP (se um server ficar reiniciando em loop — adicionar
anti-loop como o do core se aparecer na prática).

**[FEITA] em 2026-07-11 — FECHA o M4.3 e o marco M4.** Protocolo 0.39.0.
Parte B: `LspManager` ganhou `timeout_streak` (HashMap), `restart_language`/
`restart_all`, `note_timeout` (auto-restart em `MAX_TIMEOUT_STREAK=3`) e
emite `event.lsp.restarted`; `send_request` zera o streak em qualquer
resposta. Handler `lsp.restart` + comando de paleta "LSP: Reiniciar
servidor"; `CoreClient.lspRestart` + sinal `lspRestarted` →
`EditorEventRouter.onLspRestarted` → `refreshSemanticTokens` (re-sync pelo
mesmo caminho do `recovered()`). Parte C: `JobManager.cancel_all()`
(sinaliza) + `Drop` com drain limitado (500ms, poll 20ms) até os jobs
assentarem — cobre core.shutdown, EOF (UI morta) e unwind; `core.shutdown`
também chama `cancel_all` para sinal pronto. Testes: `cancel_all` sinaliza
os 2 jobs vivos; `restart_without_server` no-op (212 core verdes).
Sonda e2e `sonda_m43b.py`: (B) fs.read de .rs sobe o rust-analyzer →
`lsp.restart` responde `restarted:["rust"]`; (C) `build.run` gera cargo
filho → `core.shutdown` → cargo NÃO fica órfão (drain matou até netos).
Gate/qmllint/smoke verdes. Descoberta: o drain de 500ms bastou para os
netos (rustc) também morrerem no teste.

### Fatia M4.5 — Perfis de rigor Strict/Balanced/Relaxed (design 2026-07-11)

Fecha (junto de M4.2) o M4 do docs/roadmaps/21. Um setting (extensão da M4.1) que
regula O QUE A IDE RODA PARA O USUÁRIO nos projetos DELE — nunca o gate
do próprio repo Kinein (imutável). Default Strict (identidade do produto).

**Diagnóstico:** `run_quality` hoje roda `cargo clippy --all-targets
--message-format=json` SEM pedantic/nursery/-D warnings; `run_cargo_build`
roda `cargo build` sem RUSTFLAGS. O perfil ADICIONA flags conforme o
nível.

**Decisões (e porquês):**

```text
- rigorProfile: "strict" | "balanced" | "relaxed" (default strict),
  campo novo no Settings (M4.1). Enum RigorProfile no protocolo; arquivo
  com valor invalido -> default (padrao invalido->default do storage);
  settings.set com string invalida -> INVALID_PARAMS (serde falha).
- Regula SÓ o que a IDE roda no projeto do usuario (quality.run/build.run
  do botao), NUNCA scripts/verificar.sh nem KineinStrictOptions do repo.
- clippy por perfil (cargo clippy ... -- <flags>):
  * strict:  -W clippy::pedantic -W clippy::nursery -D warnings
  * balanced: (nada — clippy default: correctness/suspicious/style/perf)
  * relaxed: -A clippy::all -W clippy::correctness (só bugs reais)
- cargo build por perfil (RUSTFLAGS):
  * strict: RUSTFLAGS="-D warnings" (build falha em warning — coerente
    com a identidade). balanced/relaxed: sem RUSTFLAGS.
  * NOTA: mexer em RUSTFLAGS invalida o cache do cargo (rebuild ao
    trocar de perfil) — aceito; perfil se troca raramente.
- Flags viram FUNÇÕES PURAS (clippy_profile_args / rust_build_rustflags)
  testaveis; run_quality/run_cargo_build ganham o parametro profile; o
  handler le o efetivo (global<-workspace) e passa. C++ CMake -Werror
  por perfil fica FORA (injetar flag no build do usuario e invasivo;
  gatilho: pedido real) — o perfil so afeta Rust no v1.
- UI: SettingsDialog ganha um seletor de 3 opcoes (segmented/botoes) com
  descricao curta de cada; escopo global (v1, como os outros settings).
```

**Contrato (protocolo 0.38.0):** estende o Settings da M4.1:

```text
SettingsValues += rigorProfile?: "strict"|"balanced"|"relaxed"
EffectiveSettings += rigorProfile (default "strict")
```

**Arquivos:** kinein-protocol/src/settings.rs (RigorProfile + campo, lib
0.38.0); kinein-core/src/settings.rs (resolve default Strict);
kinein-core/src/build.rs (clippy_profile_args/rust_build_rustflags +
run_quality/run_cargo_build com profile + testes); handlers/build.rs
(ler efetivo, passar); ui/src/core_client.* (settingsSet ja manda o
mapa — rigorProfile vai junto); SettingsController (rigorProfile);
SettingsDialog (seletor); MANUAL (o que cada perfil liga).

**Testes:** core — clippy_profile_args/rust_build_rustflags por perfil;
resolve default strict; settings.set rigorProfile invalido ->
INVALID_PARAMS. Sonda e2e: set rigorProfile=relaxed -> efetivo reflete;
persiste entre processos.

**Fora (com gatilho):** C++ CMake -Werror por perfil; perfil por
workspace pela UI (contrato ja suporta arquivo); perfis afetarem T4
(cargo check flags) — quando o check-no-save ganhar controle proprio.

**[FEITA] em 2026-07-11.** Protocolo 0.38.0: `RigorProfile`
(strict|balanced|relaxed, `#[derive(Default)]` com `#[default] Strict`),
`rigorProfile` em `SettingsValues`/`EffectiveSettings`. Core:
`clippy_profile_args`/`rust_build_rustflags` como funções puras testadas;
`run_build`/`run_quality`/`run_cargo_build` recebem `profile`;
`settings::effective_rigor_profile(root)` é FUNÇÃO LIVRE (não método —
não usa `self`, clippy::unused_self) e os handlers de build/quality a
chamam antes de spawnar o job. UI: `SettingsController.rigorProfile`,
seletor segmentado de 3 opções no `SettingsDialog` (chip selecionado em
`Theme.accent`), C++ inalterado (o `settingsSet` já serializa o mapa
inteiro). Validação: gate (fmt/clippy `-D warnings`/210 testes core)
verde, qmllint estrito limpo, smoke offscreen vivo, sonda e2e
`sonda_m45.py` (default strict → global relaxed vira efetivo → workspace
strict vence global → persiste em `.kinein/settings.json` e XDG).

### Fatia M4.2 — Orçamento de performance medido (design 2026-07-11)

Fecha (junto de M4.5) o miolo do M4 do docs/roadmaps/21. NÃO é uma fatia de
feature: o entregável é (1) um script de medição repetível, (2) uma
TABELA DE ORÇAMENTO com números REAIS da máquina de referência, (3) o
gancho mínimo pra medir startup. **Regra dura: ZERO telemetria/rede — a
medição é 100% local (stderr + `/proc`), manual, roda quando o dev
quiser.** Máquina de referência (docs/build/14): AMD Ryzen 7 7735HS (16
threads), ~22 GB RAM, Arch, Qt 6.11.

**Filosofia:** só entra na tabela o que dá pra medir de forma
REPRODUTÍVEL e HONESTA neste ambiente (offscreen + stdio). O que exige
GUI real com injeção de teclas (latência de digitação) fica como
PROCEDIMENTO MANUAL documentado (o docs/roadmaps/21 já permite) — não invento
número automático. Orçamento = mediana medida × folga (~1.5–2×,
arredondado): assim uma regressão é sinal real, não ruído de medição.

**Decisões (e porquês):**

```text
- scripts/medir-performance.sh: roda N vezes cada métrica e reporta a
  MEDIANA (mais estável que média com outliers de warm-up). Aceita
  KINEIN_PERF_N (default 5). Não falha o build; imprime a tabela.
- (A) UI startup / time-to-first-frame: main.cpp ganha um QElapsedTimer
  no topo; se KINEIN_PERF_MARKER estiver setado, ao PRIMEIRO frameSwapped
  da janela raiz imprime "KINEIN_PERF first_frame_ms=<n>" no stderr; se
  KINEIN_PERF_EXIT, sai. Gated por env => ZERO efeito no uso normal, sem
  telemetria (só stderr local). Offscreen (QT_QPA_PLATFORM=offscreen)
  ainda renderiza e emite frameSwapped.
- (B) Core workspace.open: round-trip stdio no PRÓPRIO repo Kinein (o
  "workspace grande" do docs/roadmaps/21). Mede do spawn até a resposta do
  workspace.open. Puro core, determinístico.
- (C) Abertura de arquivo grande: gera arquivo sintético de 10k linhas
  em tmp e mede o round-trip de fs.read (proxy da abertura no editor do
  lado core — o custo de layout do TextEdit na GUI é medido à parte,
  manual). Determinístico.
- (D) RSS: (d1) UI em boot vazio, VmRSS de /proc após settle fixo; (d2)
  core em regime após workspace.open + LSP Rust vivo + alguns didOpen —
  footprint do BACKEND, dirigido headless. O cenário "10 abas + LSP" na
  GUI real fica MANUAL (procedimento no doc), pois exige injeção de UI.
- Latência de digitação (item c do docs/roadmaps/21): PROCEDIMENTO MANUAL — abrir
  arquivo de 10k linhas na GUI, digitar segurando uma tecla, observar se
  há atraso perceptível (>~50ms incomoda). Registrar a impressão do
  usuário; sem número sintético.
- Onde mora o orçamento: TABELA no docs/roadmaps/21 (fonte única do orçamento) +
  esta nota registra a 1ª medição. Regressão de orçamento = bug (fatia
  de correção antes de feature nova), como manda o docs/roadmaps/21.
```

**Contrato (protocolo INALTERADO, 0.38.0):** nada de RPC novo. Única
mudança de binário é o marker env-gated na `main.cpp` da UI (stderr).

**Arquivos:** ui/src/main.cpp (QElapsedTimer + marker gated);
scripts/medir-performance.sh (novo); docs/roadmaps/21 (tabela de orçamento com
números reais); docs/build/14 (documenta o script ao lado do verificar.sh);
docs/diario/18 (esta nota); ContextoIA (M4 quase fechado).

**Testes/validação:** o próprio script É a validação (roda verde e
produz números plausíveis); marker não muda o uso normal (smoke offscreen
sem a env continua idêntico); gate segue verde (só mudou main.cpp +
script + docs, sem tocar lógica).

**Fora (com gatilho):** medição automática de latência de digitação
(precisa de qmltestrunner injetando teclas — fatia própria se virar
gargalo); perf CI/histórico (só quando houver >1 dev e risco de
regressão silenciosa); orçamento de C++/CMake build time (o build já é
`cargo`/`ninja` puro, medível com `time` quando incomodar).

**[FEITA] em 2026-07-11.** `main.cpp` ganhou `installStartupPerfMarker`
(QElapsedTimer + `frameSwapped` `Qt::SingleShotConnection`, gated por
`KINEIN_PERF_MARKER`, `qInfo` no stderr — clang-tidy `WarningsAsErrors:*`
não aceita `fprintf`/vararg). `scripts/medir-performance.sh` (bash:
mediana de N, A e RSS da UI) + `scripts/medir-core.py` (stdio: B/C/D2).
Protocolo inalterado. 1ª medição na máquina de referência (N=3): TTF
offscreen ~101 ms, UI RSS 88 MB, workspace.open 1.5 ms, fs.read 10k
0.1 ms, core RSS 5 MB, rust-analyzer 677 MB (externo). Tabela de
orçamento em docs/roadmaps/21; script documentado em docs/build/14. Descoberta: `fs.read`
de um `.rs` já dispara `did_open` → o rust-analyzer sobe sozinho (usei
`.txt` para isolar o I/O em C). Gate segue verde (só main.cpp + scripts +
docs). Latência de digitação ficou como métrica MANUAL (docs/roadmaps/21).

### Fatia M3.3 — Stage/unstage/commit (design 2026-07-09)

Fecha o ciclo básico do Git MVP: revisar → stage → commit sem sair.

**Decisões (e porquês):**

```text
- Mutações respondem o MESMO shape do git.status (lista completa +
  branch): a UI atualiza status bar, árvore e aba Git de uma vez, sem
  segunda chamada (padrão runConfig.*: "a UI nunca calcula estado
  derivado").
- git.stage { paths }   → git add -- <paths>
  git.unstage { paths } → git restore --staged -- <paths>
  git.discard { paths } → DESTRUTIVO: tracked = git restore --staged
  --worktree; untracked = git clean -f. A CONFIRMAÇÃO é da UI (diálogo
  explícito "isso não tem desfazer"); o core executa sem perguntar
  (guardrail docs/arquitetura/19: ação destrutiva atrás de confirmação).
- git.commit { message } commita SÓ o que está staged (modelo git puro;
  commit de seleção estilo JetBrains fica pós-MVP). Guarda estável
  antes: `git diff --cached --quiet` (exit 1 = tem staged) — nada
  staged → INVALID_REQUEST com mensagem nossa, nunca parse do erro
  localizado do git. message vazia → INVALID_PARAMS. Falha real do
  commit (ex.: user.name ausente) → INTERNAL_ERROR com o stderr do git
  (informativo).
- paths: absolutos, confinados ao workspace (canonicalize +
  starts_with, arquivo DELETADO não canonicaliza → aceitar path dentro
  do root mesmo sem existir, via caminho lexicamente dentro do root;
  discard/stage de deletados é caso real).
- Aba Git no painel inferior: lista única de mudanças (como o Commit
  tool window do JetBrains): [☐/☑ staged] nome colorido pelo kind +
  path relativo + ação de diff por linha; clique no nome abre o
  arquivo. Rodapé: input de mensagem + botão "Commit (N)" habilitado
  com mensagem não vazia e N staged > 0. Mensagem de uma linha no v1
  (corpo multi-linha vem com Settings/History pós-MVP, documentado).
- Untracked staged (git add em novo) vira kind added staged no status —
  o checkbox cobre untracked também (JetBrains-like).
```

**Contrato (protocolo 0.32.0):**

```text
git.stage    { paths: [abs] } → GitStatusResult (novo estado)
git.unstage  { paths: [abs] } → GitStatusResult
git.discard  { paths: [abs] } → GitStatusResult   (DESTRUTIVO)
git.commit   { message }      → GitStatusResult
erros: NO_WORKSPACE; INVALID_PARAMS (paths vazio/fora do root; message
vazia); INVALID_REQUEST (não é repo; commit sem staged);
TOOL_NOT_FOUND; INTERNAL_ERROR (git falhou; stderr na mensagem).
```

**Arquivos — FEITOS em 2026-07-09:** kinein-protocol/src/git.rs
(+lib 0.32.0); kinein-core/src/git.rs (stage/unstage/discard com split
tracked/untracked + commit com guarda `diff --cached --quiet`; erros
NotARepo/NothingStaged próprios); handlers/git.rs (rotas, confinamento
lexical p/ path deletado, git_error_response/invalid_git_params
compartilhados); commands.rs ("Git: Commit..."); tests/git.rs (params
vazios, fora do root, mutação em não-repo → INVALID_REQUEST);
ui/src/core_client.* (4 invokables; respostas reusam o
gitStatusResolved — um caminho só de atualização); GitController
(changesModel com absPath, stagedCount, lastMutationError, diálogo de
discard e re-diff do arquivo ativo após mutação); GitPanel.qml (lista
com stage por clique, chips diff/descartar no hover, clique abre o
arquivo, input de mensagem + botão "Commit (N)" gated);
GitDiscardDialog.qml (confirmação destrutiva "não tem desfazer",
overlay z96); aba "Git" no BottomTabBar/Host; CommandDispatcher abre a
aba via "Git: Commit...".

**Validação executada (2026-07-09):** sonda e2e com git real (G6-G10):
stage marcou a.txt+c.txt (untracked staged vira added), unstage
devolveu c.txt a untracked, commit staged-only limpou o status e
manteve branch, commit sem staged → INVALID_REQUEST próprio, discard
restaurou o conteúdo do tracked E apagou o untracked do disco. 187
testes core + 47 protocol; gate completo verde; smoke offscreen limpo.

## Radar — Abrir projeto pronto sem fricção (usuário, 2026-07-09)

Requisito literal: ao ABRIR um projeto existente (em vez de criar um do
zero, fluxo que as specs de onboarding já preveem), a IDE deve ler e
identificar sozinha os arquivos de build (`CMakeLists.txt`,
`CMakePresets.json`, `Cargo.toml` e afins) e deixar o LSP preciso sem
passos manuais. Estado hoje: o kind já é detectado no `workspace.open` e
o cargo dispensa setup, mas projeto CMake exige o clique em
"Configurar CMake" para gerar o `compile_commands.json` do clangd.

```text
Fatia "auto-setup ao abrir" v1 — FEITA em 2026-07-09 (só UI; nenhum
contrato novo):
[feito] workspace.open kind=cmake não configurado → cmake.configure
        dispara SOZINHO uma única vez por workspace
        (ProjectHealthController.autoConfigureAttempted; reset na troca
        de root — anti-loop por construção). Banner vira o aviso
        discreto "configurando o projeto CMake automaticamente..." com
        ação [Jobs]; se o job falhar (cmakeConfigureFinished(false) →
        autoConfigureFailed) o banner VOLTA ao acionável com botão
        [Configurar] — sem novo auto-fire.
[feito] salvar CMakeLists.txt/CMakePresets.json pela IDE reconfigura
        sozinho (WorkspaceEventRouter.onFileSaved; job na aba Jobs sem
        roubar foco).
[feito] sinal novo cmakeConfigureFinished(success) no CoreClient
        (event.cmake.finished já re-consultava cmake.status).
[fica]  re-attach automático do clangd pós-configure (hoje: reabrir o
        arquivo/projeto, como documentado desde a M2.2) — gatilho:
        incomodar no uso real; design provável: re-didOpen dos .cpp
        abertos ao fim do job.
[fica]  escolha de preset do CMakePresets.json (entra com o Target
        selector da C5).
Validação: fatia 100% UI sobre pipeline já provado na M2.2 — build
release+debug, qmllint estrito e smoke offscreen verdes; teste
interativo real (abrir projeto CMake cru e ver o configure sozinho) é
do usuário (R7).
```

## Fatia A1 — Workspaces recentes [FEITA] (2026-07-15)

Fecha a lacuna entre a Start Screen já entregue e a retomada diária de um
projeto. O usuário pausou explicitamente o refino visual restante do KV Context
e autorizou esta fatia; ela não altera terminal, editor ou sessão por
workspace.

**Decisões (e porquês):**

```text
- Persistência global em recent-workspaces.json, no diretório XDG da Kinein,
  com schemaVersion=1. Histórico é estado global da IDE, não setting
  sobreponível por workspace e não entra em .kinein/.
- Limite de 12 entradas. Fixadas aparecem primeiro; dentro de cada grupo vale
  lastOpenedAt decrescente. Duplicata canônica é atualizada, nunca repetida.
- Só workspace.open/createProject concluído com sucesso toca o histórico.
  Falha de persistência não desfaz uma abertura válida; as mutações explícitas
  devolvem erro estruturado.
- workspace.recent.list/pin/remove/clear sempre respondem a lista completa.
  A UI guarda apenas esse snapshot e não calcula ordenação ou disponibilidade.
- available é calculado pelo core. Caminho removido fica visível e desabilitado
  até o usuário removê-lo; abrir continua usando workspace.open.
- O formato v0 (sem pinned) é aceito e promovido para v1 na próxima escrita;
  schema desconhecido/JSON inválido vira lista vazia sem impedir a IDE de abrir.
- Start Screen mostra até quatro entradas com abrir/fixar/remover/limpar. O menu
  Arquivo inclui Abrir recente e a limpeza; nenhum explorador paralelo nasce.
```

**Contrato implementado (protocolo 0.53.0):**

```text
workspace.recent.list {} → { workspaces: [RecentWorkspace] }
workspace.recent.pin { root, pinned } → mesmo resultado completo
workspace.recent.remove { root } → mesmo resultado completo
workspace.recent.clear {} → { workspaces: [] }
RecentWorkspace { name, root, lastOpenedAt, pinned, available }
```

**Arquivos:** `kinein-protocol/src/workspace.rs`; `kinein-core/src/workspace/
recent.rs`, `handlers/workspace.rs`, `tests/workspace.rs`, `lib.rs`; CoreClient
C++; `RecentWorkspacesController.qml`, `RecentWorkspacesCard.qml`,
`StartScreen.qml`, shell/menu/roteador e fios de composição; schema próprio +
IPC; este documento, `docs/arquitetura/03`, `MANUAL`, `ContextoIA`, `GUIAIA` e
`PONTO_ATUAL`.

**Testes e validação:** core cobre ordenação, deduplicação, limite, fixação,
migração v0, schema desconhecido e caminho ausente; dispatch cobre params e
resposta completa; harness QML cobre snapshot, abrir, fixar, remover, limpar e
bloqueio de entrada ausente. `scripts/verificar.sh` passou completo com 328
testes Rust, Clippy `-D warnings`, clang-format/tidy, qmllint, 12 harnesses e
builds debug/release. O smoke offscreen do release saiu com código 0. Uma sonda
com dois processos reais do core e XDG isolado confirmou abrir → persistir →
fixar → reler após reinício → remover → limpar; o gesto humano de um clique
fica para o próximo uso real, sem pendência técnica conhecida.

**Fora:** conteúdo de arquivos, credenciais, IA, sync/cloud, multi-root,
miniaturas, varredura de diretórios e restauração adicional de layout/cursor.

## Fatia A2 — projeto híbrido e execução prática [FEITA] (2026-07-15)

Fecha a lacuna de self-hosting em que a própria Kinein era classificada apenas
como Cargo, embora também tenha CMake/Qt/QML. Não cria Project Model paralelo;
é a camada mínima de capacidade sobre os serviços já entregues.

**Contrato implementado (protocolo 0.55.0):**

```text
WorkspaceInfo {
  kind, markers,
  capabilities: { buildSystems: ["cargo", "cmake", ...] }
}
build.run   { buildSystem? } → { jobId }
quality.run { buildSystem? } → { jobId }
test.run    { filter?, buildSystem? } → { jobId }
run.script  { path } → { command }
```

`kind` mantém a precedência/compatibilidade existente; capabilities vêm da
mesma varredura dos marcadores. Seleção explícita precisa estar no snapshot e
é rejeitada antes do job quando indisponível. Sem seleção, o sistema primário
preserva clientes anteriores. Toolbar, menu, Project Health e status mostram
Cargo+CMake e oferecem build/teste separados no workspace híbrido; os Jobs e
handlers existentes continuam sendo os únicos executores.

Como conforto diretamente ligado ao self-hosting, arquivos `.sh/.bash/.zsh`
ganharam ação de executar na linha da árvore e no menu de contexto, inspirada
na ação de gutter/contexto das IDEs JetBrains. A UI envia somente o path; o
core confina o arquivo e invoca `bash`/`zsh` com argv explícito, sem interpolar
shell. Saída, stdin e stop reutilizam `event.run.*` e o Terminal de execução.

**Concorrência editor:** a investigação do relato “Tree-sitter disputando com
LSP” encontrou resposta semântica obsoleta, não dois highlighters de mesma
autoridade. O protocolo 0.54.0 fez semantic tokens ecoarem path+version; a UI
avança a versão na edição, limpa tokens velhos e descarta resposta que não
corresponde ao buffer ativo. Tree-sitter continua instantâneo e o LSP
progressivo.

**UI adjacente:** `EditorGutter.qml` separa folding/breakpoint, diagnóstico,
blame/diff e números medidos por `FontMetrics`; a toolbar oculta target/config
secundários em larguras menores; os cinco SVGs fornecidos pelo usuário são
usados byte a byte pelo `KvIcon` para pasta/C/C++/Rust. O KV Context voltou ao
contrato terminal-first: grade VT e cursor autoritativos, sem moldura/input
paralelo.

**Testes:** core cobre detecção híbrida, roteamento Cargo/CMake, rejeição de
capacidade ausente, scripts com espaços/aspas e extensão inválida; protocolo
cobre payloads estritos; harness QML cobre ação de script e ausência de input
terminal paralelo. Rust fmt/clippy/testes, C++ clang-format/tidy/build strict,
qmllint zero warnings e harnesses QML passaram. O AppImage final é o gate de
distribuição separado desta fatia.

## M4 — Polimento contínuo (longo prazo)

## M4–M7 — Roadmap de longo horizonte

Detalhado em **`docs/roadmaps/21-long-horizon-roadmap.md`** (escrito em 2026-07-09
a pedido do usuário, com qualidade de handoff: decisões, fatias,
critérios de pronto, o que NÃO entra e a ordem sugerida). Resumo de uma
linha por marco — o detalhe vive no 21:

```text
M4 Confiança e configuração: Settings com schema (4.1), orçamento de
   performance medido sem telemetria (4.2), robustez/restart de
   processos (4.3), Start Screen/First Run (4.4, implementação material
   entregue em 2026-07-14), perfis de rigor (4.5).
M5 Paridade diária JetBrains: refactorings via LSP (5.1; workspace edits
   confirmáveis entregues em 2026-07-14), git avançado
   push/pull/branches/log/stash (5.2, entregue), call/type hierarchy e navegação
   pesada (5.3), DECISÃO de engine do editor com medição (5.4).
M6 Extensibilidade orquestrada (sem plugins de código): LSP (6.1) e DAP
   (6.2) configuráveis por settings, task runner com problem matcher
   (6.3), ponte de IA offline-first (6.4).
M7 Distribuição e comunidade: AUR/AppImage (7.1), CI pública = o gate
   (7.2), docs públicas/CONTRIBUTING/i18n (7.3), diagnóstico de falha
   local sem telemetria (7.4).
```

O `docs/roadmaps/21` também contém o **playbook de continuidade** (ritual por
fatia + convenções aprendidas) — leitura obrigatória de quem assumir o
projeto em qualquer sessão futura.

## Escada de rigor

Degrau novo só entra **verde**: primeiro zerar os achados, depois ligar o
gate (como foi feito com o qmllint em 2026-07-08: 117 avisos zerados e só
então `verificar-qml.sh` entrou no `verificar.sh`).

Hoje (2026-07-08):

```text
Rust : unsafe forbid; -D warnings; clippy all/pedantic/nursery deny;
       missing_docs deny; unwrap/expect/panic proibidos fora de teste;
       fmt --check; 170+ testes de comportamento.
C++  : clang-format --Werror; clang-tidy amplo com WarningsAsErrors;
       sanitizers em Debug; hardening+LTO em Release.
QML  : qmllint estrito (zero warnings) com contexto de módulo, via
       scripts/verificar-qml.sh; pragma ComponentBehavior: Bound e
       required properties em delegates são o padrão do repositório.
Gate : scripts/verificar.sh único, para no primeiro erro.
```

Próximos degraus, em ordem de custo/benefício:

```text
1. cargo-deny check no gate (deny.toml já existe; instalar cargo-deny).
2. shellcheck nos scripts/ (instalar shellcheck; são 5 scripts pequenos).
3. qmlformat --check: exige reformatar o tree QML inteiro uma vez — fazer
   numa fatia dedicada, sem misturar com feature.
4. ~~Testes de lógica QML para controllers não visuais~~ — **feito em
   2026-07-14:** harness `qml6` offscreen executa os controllers reais e faz
   parte de `scripts/verificar.sh` (completion, find, múltiplos terminais,
   mudança externa, outline e salvar/recentes). Expandir por regressão.
5. CI real (GitHub Actions rodando verificar.sh) quando o repo ganhar
   remote; até lá o gate local é o contrato.
6. Cobertura no core (cargo-llvm-cov) com piso acordado, sem virar métrica
   de vaidade.
```

## Princípios de lógica (valem para toda fatia nova)

```text
- Handler IPC fino; regra de negócio em serviço de domínio no core, com teste.
- QML visual burro: dados por property, ação por signal; estado/timer em
  controller; evento IPC em roteador (guardrails de docs/arquitetura/17).
- Uma fonte de verdade por estado; nada de estado derivado duplicado na UI.
- Operação longa = job cancelável com eventos (nunca bloquear UI/stdio).
- Erro estruturado no protocolo; a UI nunca parseia stdout de ferramenta.
- Contrato novo: tipos em kinein-protocol + docs/arquitetura/03 + schema quando persistir.
```

## Como usar este documento

```text
1. Pegar a próxima fatia do marco corrente (ordem acima).
2. Cruzar com docs/roadmaps/BACKEND_TO_UI_UX_ROADMAP.md (contrato/backend necessário)
   e docs/specs/ (UX alvo inegociável).
3. Entregar com gate verde + smoke, atualizar ContextoIA.md e este doc
   (marcar fatia feita com data).
4. Ao fechar um marco, revalidar os anteriores no uso real antes de seguir.
```
