# 65 — A avaliação do Neovim como motor de edição (ENG-60)

> **Classe: PLANO** (`DocsPublic/README.md`), com medidas datadas. É o dono
> da avaliação do ENG-60 do [62 §3.4](62-base-publica-0.4.md) e da proposta
> para a DP-10 do [64](64-revisao-da-arquitetura-e-estrategia-b.md) §11: o
> Neovim como motor de edição e cliente LSP da Kinein Vectis.
>
> **Situação em 2026-11-15.** As fases A (§1 a §18, de 2026-11-14), B
> (§20) e C (§21) estão prontas. **Decisão do autor de 2026-11-15: a DP-10
> escolhe o Neovim, com condições** (§21). O produto não mudou: a
> integração é a fase D do §14, depois do B2 e do B3 do 64 §8, e o editor
> atual fica até o produto integrado passar nas condições do §21. A fase A
> recomendou GO só para a prova isolada; a prova rodou fora do produto, e o
> código e os resultados dela estão no ramo local `prova/eng-60` (§20.1).
>
> **Ponto de partida.** Commit `308c1561`, protocolo 0.223.0. Na máquina de
> medição do projeto (Linux 7.3, Landlock ABI 11): Neovim 0.12.6, Qt 6.12.0,
> clangd 23.1.4, rust-analyzer 1.96.1, ruff 0.17.4, qmlls 6.12.0. Toda
> medida é de 2026-11-14, salvo data própria. "Lido" e "executado" são
> marcados separados.

## 0. Como ler

O §1 traz os critérios de refutação, escritos antes da prova. O §2 é o
diagnóstico do editor atual; o §3, o que o Neovim oferece; o §4, os
clientes gráficos que existem. O §5 é o mapa de donos. O §6 é a integração
com o Qt Quick; o §7, o LSP; o §8, a segurança. O §9 classifica o código
atual; o §10 cruza com a estratégia B; o §11 avalia a dependência. O §12 é a
prova; o §13, a jornada de self-hosting; o §14, a implementação na 0.4, se
a DP-10 aprovar. O §15 estima o esforço; o §16 diz como a arquitetura nova
se verifica. O §17 responde às perguntas da avaliação; o §18 lista o que é
decisão do autor; o §19, o que não foi medido. O §1 ao §18 são a fase A,
escrita antes da prova, e ficam como foram escritos, salvo as notas
datadas. O §20 traz os resultados da prova (fase B); o §21, a decisão do
autor (fase C).

## 1. Os critérios de refutação

Escritos antes da prova, para a prova não ser lida a favor do candidato. Um
critério reprovado é NO-GO, salvo onde a coluna diz "pondera". A medida do
editor atual sai da mesma máquina, com o mesmo arquivo e o mesmo método.

| ID | O que se mede | Método | Reprova se |
| --- | --- | --- | --- |
| R1 | Latência da tecla ao quadro | O método do `scripts/medir-performance.sh` (A3.3-1): offscreen, o mesmo arquivo de 20 mil linhas, 40 teclas | p95 acima de 1,25 vez a do editor atual, ou pior caso acima de 2 vezes |
| R2 | Abrir o arquivo de 20 mil linhas até o primeiro quadro | Relógio na UI | acima do editor atual |
| R3 | Projeto hostil | A fixture do 64 §2.1 mais os vetores do Neovim (§8): `modeline`, `.nvim.lua`, `.exrc`, `:!`, `system()`, `jobstart()` | uma marca que seja |
| R4 | Queda do Neovim no meio da edição (`kill -9`) | Texto digitado antes da queda, depois da recuperação | perde mais do que o editor atual perde no mesmo teste |
| R5 | Processos órfãos | `kill -9` no core, na UI e no `nvim`; depois, a lista de processos | um órfão que seja |
| R6 | Dois donos do texto | Caminho de código em que a Kinein muda o texto fora do Neovim; edição da Kinein com `changedtick` velho aplicada | existe |
| R7 | Sessão LSP duplicada | Servidores por linguagem e raiz, durante a jornada | mais de um |
| R8 | Edição sem modo (memória JetBrains, RNF8 do [19](../arquitetura/19-compromissos-de-arquitetura.md)) | A tabela de atalhos do §12.3 | uma ação que exija saber o modo do Vim |
| R9 | Unicode e IME | Acento morto do pt-BR, composição visível no cursor, caractere de largura dupla e emoji | posição ou texto errado; sem framework de IME instalado, o item fica NÃO PROVADO, e não aprovado |
| R10 | Acessibilidade | A árvore AT-SPI do editor atual contra a do componente novo: texto da linha e posição do cursor | sem caminho viável para a mesma informação |
| R11 | Contexto da toolchain | O clangd usa o `compile_commands.json` e o `--query-driver` que a Kinein decide; trocar o padrão no preset muda o diagnóstico | o diagnóstico não segue a troca |
| R12 | Memória | RSS da UI mais o do `nvim`, contra a UI de hoje, com o mesmo arquivo | acima de 1,5 vez sem causa medida (pondera) |
| R13 | Código próprio | Linhas que a integração exige (renderizador, ponte, Lua, relé), projetadas para o produto, contra as eliminadas (§9) | maior que o eliminado sem ganho de roteiro que o compense (pondera) |

A prova não está concluída porque um arquivo abre e recebe texto: cada
linha acima precisa de número ou de NÃO PROVADO explícito.

## 2. O editor atual

### 2.1 Como funciona (lido no código do `308c1561`)

- **A UI é dona do texto.** Um `TextEdit` do Qt Quick
  (`ui/qml/editor/EditorTextSurface.qml`, linha 227) guarda o buffer. O
  realce é um `QSyntaxHighlighter` em C++ (`ui/src/editor_highlighter*`).
- **A árvore sintática vem do core inteira.** O `syntaxTree.update` leva o
  buffer inteiro e traz a árvore inteira a cada edição (ENG-55; 64 §2.4).
- **Cada pedido LSP leva o conteúdo inteiro.** Os sinais do
  `EditorLanguageController.qml` mandam `path, content, line, column`. O
  core manda `didChange` com o texto completo quando o hash do conteúdo
  muda (`lsp/framing.rs`, `lsp/server.rs`).
- **16 métodos `lsp.*`:** `completion`, `hover`, `definition`,
  `references`, `rename`, `codeActions`, `applyCodeAction`,
  `documentSymbols`, `workspaceSymbols`, `semanticTokens`,
  `switchSourceHeader`, `didChange`, `restart`, `workspaceEdit`,
  `workspaceEdit.apply` e `workspaceEdit.cancel`.
- **Rascunhos:** `draft.save` grava o texto não salvo no `kinein.db`, com
  temporizadores de 1,2 s a 3 s (`EditorPersistenceController.qml`).
- **Gravação:** pelo core (`fs.write` com o conteúdo esperado), seguida da
  checagem do compilador ao salvar (`handlers/syntax_check.rs`).
- **Edição com prévia:** o `WorkspaceEdit` do rename e das code actions vira
  uma transação com prévia, gravação atômica e desfazer em caso de falha
  (`lsp/transaction.rs`), com a evidência do compilador na prévia
  (`handlers/lsp/verification.rs`).

### 2.2 O tamanho (medido)

Linhas pelos arquivos que o git rastreia; no Rust, a produção vai até o
primeiro `#[cfg(test)]`.

| Grupo | Arquivos | Produção | Teste no arquivo |
| --- | --- | --- | --- |
| core: cliente LSP (`lsp/`) | 18 | 4.354 | 1.169 |
| core: handlers LSP (`handlers/lsp/`) | 7 | 988 | 0 |
| core: tree-sitter do editor (`lang/`) | 8 | 1.107 | 486 |
| core: `handlers/syntax.rs`, `commands/editor.rs`, `handlers/syntax_check.rs` | 3 | 400 | 185 |
| core: rascunhos (`db/`, `handlers/draft.rs`) | 2 | 280 | 64 |
| core: formatação (`format.rs`, `handlers/format.rs`) | 2 | 416 | 183 |
| protocolo: lsp, syntax, draft, format (`kinein-protocol`) | 4 | 569 | 393 |
| ui: QML do editor (`ui/qml/editor/`) | 61 | 8.758 | — |
| ui: roteador do editor (`ui/qml/ipc/Editor*`) | 2 | 320 | — |
| ui: realce em C++ (`ui/src/editor_highlighter*`) | 7 | 1.426 | — |
| ui: despacho LSP na ponte | 1 | 179 | — |

Testes fora desses arquivos: 10 arquivos de teste IPC do editor no core
(3.636 linhas) e 10 harnesses QML do editor (1.293 linhas).

### 2.3 O que o motor atual bloqueia

O [21](21-roadmap-de-longo-prazo.md) §M5.4 já registrava: o `TextEdit`
bloqueia split view, minimap, multi-cursor real, virtualização de arquivos
gigantes e inlay hints posicionados. A medida do 64 §2.4 acrescenta o custo
do `syntaxTree.update` (345,0 ms no laço e 8.218 KiB por edição num arquivo
de 20 mil linhas). A fachada `EditorController.qml` tem 791 linhas contra o
limite de 400 (ENG-24).

## 3. O que o Neovim oferece

Lido na documentação do Neovim 0.12.6 instalado (`:help`), salvo onde diz
"executado".

- **Processo.** `nvim --embed` usa o stdin e o stdout como canal
  msgpack-RPC; "se o canal fecha (salvo por `:detach`), o Nvim sai". Ele
  espera um `nvim_ui_attach()` antes de ler a configuração e os buffers.
  `--listen` abre outro canal, num socket local.
- **Contrato da API (`api-contract`).** Assinatura de função não muda
  depois do lançamento; mapas e listas só ganham campos opcionais; eventos
  não perdem nem reordenam parâmetros; "funções obsoletas não são removidas
  até o Nvim 2.0". Interface sem `:help` não é coberta.
- **UI remota.** `ext_linegrid` (o protocolo de grade por linha, com
  `grid_line`, `grid_scroll`, `hl_attr_define`), `ext_multigrid` (cada
  janela numa grade), `ext_popupmenu`, `ext_cmdline`, `ext_messages`,
  `ext_tabline`, `ext_hlstate` e `ext_termcolors`. Cada extensão passa uma
  parte da interface do Neovim para o cliente desenhar.
- **Eventos de buffer.** `nvim_buf_attach()` entrega
  `nvim_buf_lines_event` com o `changedtick` e as linhas mudadas; em Lua,
  `on_bytes` dá a mudança por byte.
- **Cliente LSP nativo.** O `runtime/lua/vim/lsp/` traz completion,
  diagnostics, code lens, inlay hints, semantic tokens, folding range,
  linked editing, on-type formatting, inline completion e document color,
  além de definição, referências, rename, code actions, hover, assinatura,
  formatação e símbolos. `vim.lsp.config` e `vim.lsp.enable` configuram; o
  campo `cmd` aceita uma função, e `vim.lsp.rpc.connect()` conecta a um
  pipe nomeado (Windows), a um socket Unix ou a TCP, **sem o Neovim criar o
  processo do servidor**.
- **Tree-sitter.** O pacote traz os parsers de C, Lua, Markdown, query, Vim
  e vimdoc. C++, Rust, Python e QML precisam de parser instalado; o
  `kinein-core` já compila as gramáticas de C, C++, Rust e Python (64
  §5.14). A compatibilidade de ABI entre as duas cópias não foi medida.
- **Padrões que pesam na segurança.** `modeline` ligado (desligado para
  root); `modelineexpr` desligado; `exrc` desligado.
- **Sem modo.** O `'insertmode'` saiu do Neovim; o `vim_diff.txt` traz a
  emulação por `inoremap`. É a base de uma camada sem modo (§6.4).
- **O que não é nativo no 0.12:** multi-cursor e minimap.
- **Tamanho do pacote oficial para Linux (executado):** 40 MiB no disco
  (`bin/nvim` 11 MiB, `lib` 2,7 MiB, `share` 27 MiB).

## 4. Os clientes gráficos que existem

| Cliente | Licença | Base gráfica | O que serve à Kinein |
| --- | --- | --- | --- |
| Neovim-Qt (último commit `c8f98f1`, 2026-10-14) | ISC | Qt 6; o shell é um `QWidget` (`src/gui/shellwidget/shellwidget.h`) | A biblioteca de RPC (Qt Core e Network, com `msgpack-c`): 2.334 linhas, mais 40.066 linhas de API gerada. O modelo de células e de destaque (`cell`, `highlight`, `shellcontents`) não usa `QWidget` |
| Neovide | MIT | Rust, janela própria | Referência de desenho de grade com ligaduras; não embarca no Qt Quick |
| goneovim | MIT | Go, com ligação para Qt Widgets | Referência; não embarca no Qt Quick |

Nenhum cliente em Qt Quick foi encontrado (busca de 2026-11-14). O
Neovim-Qt mostra o tamanho de um cliente completo em Qt: 5.910 linhas no
`src/gui` e 2.817 no `shellwidget`. Ele mostra a composição do IME numa
dica flutuante (`src/gui/shell.cpp`, linhas 1893 a 1903) e não tem
interface de acessibilidade (nenhum `QAccessible` no código).

**Por que não embarcar o widget do Neovim-Qt.** As duas formas de pôr um
`QWidget` dentro de uma cena Qt Quick custam caro: `QQuickWidget` inverte a
casca inteira, e `QWidget::createWindowContainer` cria uma janela nativa
filha, que fica por cima de tudo (os popups e o HUD da Kinein deixariam de
cobrir o editor) e quebra a cadeia de foco da QML. O ganho seria evitar o
renderizador; o §12 mede se ele é pequeno. A forma proposta é um
`QQuickItem` próprio, que reaproveita o desenho de RPC e, se couber, o
modelo de células do Neovim-Qt, com o aviso da ISC preservado.

## 5. Um dono por responsabilidade

| Responsabilidade | Dono | Os outros | Como se impõe |
| --- | --- | --- | --- |
| Texto e versão do documento aberto | Neovim (`changedtick`) | Core e UI recebem eventos versionados; nunca editam por conta própria | A Kinein só muda texto por uma função Lua que confere o `changedtick` esperado e recusa o velho (R6) |
| Arquivo sujo e gravação | Neovim (`:write`) | Core recebe `BufWritePost` e dispara a checagem ao salvar e o índice | A forma da gravação é decisão do §18, item 2 |
| Cursor e seleção | Neovim | A UI desenha o que a grade diz | Não há estado de cursor na QML |
| Diagnósticos no texto | O servidor publica; o Neovim mostra no buffer | Core observa as mesmas notificações pelo relé e monta o painel de problemas, junto com o build | Os dois são leitores; nenhum altera o fato do servidor |
| Sessão LSP (processo) | Core, pelo lançador (B2) | Neovim conecta por socket; não cria processo | O Neovim roda sem poder executar programa (§8) |
| Protocolo LSP do documento (`didOpen`, versões, pedidos) | Neovim | Core não sincroniza o mesmo documento | Um cliente por documento; o relé não manda `didOpen` |
| Configuração do servidor | Core (kit, preset, ambiente Python, confiança) | Neovim recebe a configuração pronta por RPC | O Lua da Kinein não decide argumento nem caminho |
| Rename e code actions | Neovim executa | A prévia de edição em vários arquivos é investigada (§18, item 7) | — |
| Identidade de projeto e workspace | Core | Neovim recebe a raiz e o diretório de trabalho | — |
| Estado persistente e recuperação | Core (estado XDG, D15 do 19) | Neovim com `shada` e swap na pasta de estado da Kinein, ou sem swap (§18, item 3) | — |
| Resultado velho | Neovim, para o LSP; Core, para os próprios resultados, pelo `changedtick` | — | Resultado da Kinein com `changedtick` velho é descartado (teste) |
| Análise especializada da Kinein | Core | Mostra como diagnóstico de um namespace próprio no Neovim, ou num painel QML | O Core lê instantâneos versionados; não escreve no buffer |

## 6. A integração com o Qt Quick

### 6.1 Processos e canais

```text
UI (Qt Quick)                         kinein-core                     nvim --embed --listen
  NvimView (QQuickItem) <======== socket privado: UI remota, teclado, mouse =======> canal da UI
  painéis, HUD          <-- JSON-RPC (como hoje) -->  core
                                                      core <== stdio msgpack-RPC ==> canal do core
                                                      core --lançador--> clangd, rust-analyzer, ...
                                    nvim --socket por servidor (relé do core)--> servidor de linguagem
```

- **O caminho da tecla não passa pelo core.** A UI fala direto com o
  `nvim` pelo socket; a latência da digitação não depende do laço do core.
- **O core cria o `nvim` pelo lançador** e fica com o stdio. Se o core
  morre, o canal fecha e o `nvim` sai (§3): não sobra órfão (R5).
- **O socket** fica numa pasta privada (`$XDG_RUNTIME_DIR`, modo 0700) no
  Linux e num pipe nomeado no Windows. O core passa o endereço à UI.
- **O core recebe o espelho** pelo `nvim_buf_attach` no próprio canal,
  fora do caminho da tecla.

### 6.2 O desenho

- `ext_linegrid` com `ext_hlstate`; o `ext_popupmenu` passa a lista de
  completion para um popup QML com a identidade visual da Kinein. O
  `ext_multigrid` (splits e janelas flutuantes como grades separadas) fica
  para depois da prova.
- **Um `QQuickItem` com `updatePaintNode`:** um nó por linha da grade, só
  as linhas sujas redesenhadas; texto por `QSGTextNode::addTextLayout`
  (público no Qt 6.12 instalado, `qsgtextnode.h`), um `QTextLayout` por
  trecho de mesmo destaque.
- **Fonte:** a monoespaçada do tema, com o fallback do Qt. O caractere de
  largura dupla ocupa duas células (a segunda vem vazia no `grid_line`).
- **Ligaduras:** fora da prova; o trecho por destaque permite ligar depois.

### 6.3 Entrada, foco e o resto

- **Teclado:** o evento do Qt vira a notação do `nvim_input`. Os atalhos
  globais da Kinein são tratados antes, pela QML.
- **IME:** a composição (`QInputMethodEvent::preeditString`) é desenhada no
  cursor pelo próprio item; o texto confirmado vai pelo `nvim_input`;
  `inputMethodQuery(Qt::ImCursorRectangle)` devolve o retângulo do cursor.
- **Mouse e rolagem:** `nvim_input_mouse`.
- **Tamanho:** `nvim_ui_try_resize`. **Foco:** `nvim_ui_set_focus`.
- **Área de transferência:** `g:clipboard` com um provedor Lua da Kinein
  que pede à UI, que usa o `QClipboard`.
- **Acessibilidade:** um `QAccessibleInterface` do item, com o texto da
  linha e do cursor lidos do buffer. O Neovim-Qt não tem; é trabalho novo
  (R10).

### 6.4 A edição sem modo

A RNF8 do 19 pede a memória muscular do JetBrains. O mandato de 2026-07-11
(21 §M5.4) descartou o Neovim embutido por essa tensão; a avaliação a trata
como critério (R8), não como questão encerrada.

A proposta é uma camada em Lua da Kinein: o buffer entra e fica no modo de
inserção (a emulação do `'insertmode'` do `vim_diff.txt`), `Shift` com as
setas seleciona (`keymodel` e `selectmode`), e os atalhos JetBrains da
§12.3 são mapeados. O modo do Vim fica como opção da pessoa, desligado por
padrão (§18, item 1).

## 7. A estratégia de LSP

- **Um processo por servidor, linguagem e raiz, criado pelo lançador do
  core** (B2), com a política de confiança do 64 §5.5. Isso impede o
  rust-analyzer duplicado por construção: o mapa do core tem uma entrada
  por chave (R7).
- **O relé.** O core abre um socket por servidor e repassa os quadros LSP
  entre o Neovim e o processo. Ele observa, sem alterar, o
  `publishDiagnostics` e o `$/progress`, para o painel de problemas e o
  status.
- **O Lua da Kinein** declara `vim.lsp.config(nome, { cmd =
  vim.lsp.rpc.connect(socket) })`, com `root_dir`, `settings` e
  `init_options` vindos do core por RPC.
- **O contexto vem do core:**
  - C e C++: o `compile_commands.json` do preset ativo e o
    `--query-driver` do kit (64 §5.4, E5);
  - Rust: workspace, features, alvo e toolchain do Cargo;
  - Python: o interpretador e o ambiente escolhidos (sem confiança, sem o
    interpretador do projeto, DP-02);
  - QML: o `qmlls` com a pasta de build e os caminhos de import do preset.
- **Os algoritmos especializados** (o verificador de correções) precisam
  falar com o servidor. A opção avaliada é o relé injetar os pedidos do
  core com ids próprios, no padrão dos multiplexadores de LSP; não é uma
  segunda sessão. Fica em investigação (§18, item 7).

**Maturidade por linguagem.** Nada foi medido nesta fase. C, C++ e Rust têm
servidores maduros; o `qmlls` 6.12 tem menos recursos que o clangd e o
rust-analyzer, e a paridade não é anunciada sem prova. Device tree, linker
script, Kconfig e SVD não têm servidor no pacote do Neovim: a experiência de
embarcados (SVD, registradores, memória, gravação) continua nos painéis da
Kinein.

## 8. Segurança e confiança

**O fato de um processo ser iniciado pelo Neovim não significa que esteja
autorizado pela Kinein.** Por isso a proposta é que o `nvim` não possa
iniciar processo nenhum:

- **Linux:** o lançador aplica ao `nvim` um conjunto de regras Landlock que
  trata `LANDLOCK_ACCESS_FS_EXECUTE` e não permite caminho nenhum. `:!`,
  `system()`, `jobstart()`, o terminal embutido e o servidor LSP criado pelo
  Neovim falham no `execve`. O servidor real nasce pelo lançador do core,
  com a confiança decidida lá.
- **Windows:** um Job Object com `JOB_OBJECT_LIMIT_ACTIVE_PROCESS` e limite
  1 impede o processo filho. Não foi executado.
- **Configuração, como defesa adicional, não como garantia:** `-u` com o
  `init.lua` da Kinein; `XDG_CONFIG_HOME` e `XDG_DATA_HOME` em pastas da
  Kinein, para a configuração da pessoa não entrar (R3); `runtimepath` e
  `packpath` só com o runtime do Neovim e o da Kinein, nunca uma pasta do
  projeto; `nomodeline`; `noexrc`; netrw desligado; `vim.ui.open` pede à
  Kinein. Nada disso é tratado como segurança por si: `--clean` e plugins
  desligados não provam nada.

**Limites reais:**
- O direito `EXECUTE` do Landlock vale para o `execve`, não para carregar
  biblioteca. Parser tree-sitter e módulo Lua em C carregados de uma pasta
  do projeto seriam código do projeto; daí o `runtimepath` fechado, que a
  prova confere.
- Sem Landlock (kernel anterior ao 5.13 ou módulo desligado), a contenção
  cai para a configuração. O que fazer nesse caso é decisão do §18, item 4.
- O servidor de linguagem cria os próprios filhos (o rust-analyzer roda o
  `cargo`): isso é o INF-23, e a política é a do lançador no B2, igual com
  ou sem Neovim.
- Formatadores rodam pelo core, com confiança; o resultado entra no buffer
  pela função que confere o `changedtick`.

| Vetor do briefing | Tratamento | Verificação |
| --- | --- | --- |
| Configuração Lua e plugins | Só o runtime do Neovim e o da Kinein | R3 com `init.lua` plantado na configuração da pessoa |
| Configuração local do projeto | `noexrc`; nenhuma pasta do projeto no `runtimepath` | R3 com `.nvim.lua` e `.exrc` |
| Modelines | `nomodeline` | R3 com modeline hostil |
| Servidores LSP | Lançador do core | R3 e o aceite do B2 |
| Formatadores | Core, com confiança | Teste do B2 |
| Comandos externos do Neovim | Sem `execve` (Landlock) | R3 com `:!` e `system()` |
| Toolchain do projeto | Core (E5, 64 §5.4) | R11 |
| Processos filhos | Nenhum do `nvim`; os do servidor pelo lançador | R5 e R3 |
| Arquivos de sessão e estado | `shada` e swap na pasta de estado da Kinein | Teste de que nada é gravado no projeto |
| Alterações vindas do projeto | O projeto é texto; nada nele é executado sem confiança | R3 |

## 9. Eliminar, adaptar, preservar, investigar

Classificação por arquivo, com as linhas de produção do `308c1561`. A
classe é **hipótese** desta avaliação, não decisão: a prova a confirma ou
refuta, e nada sai do produto antes de a substituição estar validada (fase
E do §14).

| Classe | Arquivos | Linhas no core | Linhas na UI | Total |
| --- | --- | --- | --- | --- |
| Eliminar | 41 | 2.866 | 4.705 | 7.571 |
| Adaptar | 37 | 2.736 | 4.002 | 6.738 |
| Investigar | 18 | 1.172 | 1.176 | 2.348 |
| Preservar | 15 | 771 | 800 | 1.571 |

- **Eliminar** (o Neovim passa a fazer): no core, a sincronia, o
  handshake, a análise das respostas e as consultas do cliente LSP
  (`lsp/document.rs`, `handshake.rs`, `parse.rs`, `query.rs`, `sync.rs`,
  `edit.rs`; `handlers/lsp/consulta.rs`, `documento.rs`) e o tree-sitter do
  editor (`lang/service.rs`, `indent.rs`, `folding.rs`, `positions.rs`;
  `handlers/syntax.rs`). Na UI, o realce em C++ e a superfície de texto com
  o que mora em volta dela: digitação, recuo, operações de linha, seleção
  crescente, calha, destaque de linha, hover, salto, ir para a linha e a
  fachada `EditorController.qml`.
- **Adaptar** (a responsabilidade fica, a implementação passa a usar o
  Neovim): a sessão, o servidor, o progresso e a fusão de diagnósticos (o
  relé); rascunhos; formatação; a checagem ao salvar; os comandos do editor
  na paleta; na UI, os popups de completion e de ações, a busca, os
  documentos abertos, a gravação, a mudança externa, a persistência e o
  roteador.
- **Investigar:** a transação com prévia e a verificação pelo compilador; o
  painel de estrutura e a busca de símbolos (servidor ou tree-sitter do
  core); a prévia de edição em vários arquivos.
- **Preservar:** o registro de servidores (`lsp/registry.rs`), o
  enquadramento LSP (que o relé usa), as URIs, a toolchain do LSP, o
  extrator do índice e o registro de linguagens; na UI, as abas, os
  recentes e o Markdown.

**Testes afetados:** os 10 arquivos de teste IPC do editor (3.636 linhas) e
os 10 harnesses do editor (1.293 linhas) testam o que a classe Eliminar
remove. Cada comportamento deles ganha um teste equivalente sobre o
Neovim antes de o teste antigo sair.

## 10. O impacto na estratégia B

| Fase | Com o Neovim, se a DP-10 aprovar |
| --- | --- |
| B1 | Igual; o `nvim` e o relé ganham span |
| B2 | Igual, e mais importante: o lançador cria o `nvim` com contenção e os servidores; o modo restrito vale para o editor novo |
| B3 | Igual; o laço do core sai do caminho dos pedidos LSP do editor |
| B4 | Os 16 métodos `lsp.*` e os `syntaxTree.*` saem do contrato; entram o ciclo do editor (abrir, endereço do socket, estado do `nvim`) |
| B5 | O piloto da ponte não usa o editor; o domínio do editor da ponte encolhe |
| B6 | O domínio `lsp` do core encolhe para lançador, relé e registro |
| B7 | Rascunhos, `shada` e swap na pasta de estado (D15 do 19) |
| B8 | Eliminado: o Neovim analisa de forma incremental no próprio processo, e o ENG-55 fecha pela remoção |
| B9, B10 | Iguais |
| B11 | O ENG-24 (a fachada do editor) sai pela remoção, e não pela divisão |

**As dez perguntas do impacto:**
1. Continuam necessárias: B1, B2, B3, B7, B9, B10 e B11.
2. Mudam: B4, B5, B6 e B7, na parte do editor.
3. Ficam redundantes: B8 e a divisão do ENG-24.
4. Componentes novos: o item Qt Quick, a ponte msgpack-RPC da UI, o
   supervisor do `nvim` no core, o relé LSP e o Lua da Kinein.
5. Contratos que mudam: o IPC do editor (B4) e o contrato novo entre a
   Kinein e o Neovim, pela API pública e pela interface do Lua da Kinein.
6. Testes preservados: os de projeto, build, toolchain, depuração,
   terminal, Git, Banco e embarcados; os do editor são substituídos item a
   item.
7. Processos e concorrência: um processo a mais, filho do core; a tecla vai
   da UI ao `nvim` sem o core; o core recebe eventos fora do caminho da
   tecla.
8. Persistência e recuperação: o estado do `nvim` mora na pasta de estado
   da Kinein; a recuperação após queda é o R4.
9. Algoritmos especializados: leem instantâneos versionados e devolvem
   diagnóstico num namespace próprio ou painel (§5).
10. Acoplamento: a Kinein depende só da API pública (`api-contract`) e de
    um conjunto pequeno de extensões de UI; o Lua da Kinein é a única
    camada que conhece o Neovim por dentro, e a versão mínima é conferida
    no início (`nvim_get_api_info`).

## 11. A dependência por cinco anos

| Tema | Avaliação |
| --- | --- |
| Licença | Neovim: Apache 2.0, salvo as partes sob a licença do Vim (`LICENSE.txt` do repositório). Compatível com a distribuição AGPL da Kinein, com os avisos. Neovim-Qt, se reaproveitado: ISC |
| Distribuição | O pacote oficial tem 40 MiB. O AppImage leva um `nvim` de versão fixa; a alternativa é exigir uma versão mínima do sistema (§18, item 5) |
| Compatibilidade | O `api-contract` garante a API pública até o 2.0. A Kinein confere o nível da API ao iniciar e recusa versão abaixo da mínima com mensagem clara |
| Atualização e segurança | Atualizar o `nvim` fixado é mudar uma versão e rodar a regressão do editor; o Lua da Kinein é o ponto que pode quebrar |
| Testes de compatibilidade | A regressão do editor roda contra a versão fixada e contra a estável mais nova |
| Linux e Windows | Há pacotes oficiais para os dois; o socket vira pipe nomeado; a contenção muda de Landlock para Job Object |
| Plugins | Nenhum de terceiros na prova. Um plugin futuro (multi-cursor, por exemplo) entra com licença conferida, como componente |
| Fork | Não. Um fork só com necessidade técnica que a API pública não resolva, demonstrada |
| Custo da ponte | O renderizador, a entrada, o IME, a acessibilidade e o Lua passam a ser da Kinein; o Neovim-Qt é a referência de tamanho (§4) |
| Risco de incompatibilidade | Um evento de UI novo e opcional não quebra o cliente; uma mudança fora do `:help` não é contrato e não pode ser usada |

## 12. A prova de conceito

### 12.1 Escopo e lugar

- **Fora do checkout**, num diretório de cache local, reversível: apagar o
  diretório desfaz tudo. O produto não muda.
- **Componentes:** um aplicativo Qt Quick com o item `NvimView`; o cliente
  msgpack-RPC; um supervisor que faz o papel do core (lançamento com
  Landlock, relé LSP, espelho do buffer, rascunho); o `init.lua` da Kinein.
- **Fora da prova:** Windows, `ext_multigrid`, ligaduras e a integração com
  o core real.

### 12.2 Os quinze itens

| # | Item | Aceite | Métrica |
| --- | --- | --- | --- |
| 1 | Inicialização controlada | `nvim` sobe pelo supervisor, com a configuração da Kinein, sem ler a da pessoa | tempo até o primeiro quadro; R3 com `init.lua` plantado |
| 2 | msgpack-RPC | pedidos, respostas e notificações nos dois canais | erros de protocolo: zero |
| 3 | Desenho no Qt Quick | grade, cores, cursor e rolagem corretos | captura de tela comparada |
| 4 | Editar C++ | abrir, digitar, recuar, salvar | texto em disco igual ao esperado |
| 5 | Editar Rust | o mesmo | o mesmo |
| 6 | Seleção, cursor, desfazer e salvar | pela tabela da §12.3 | todas as ações |
| 7 | Unicode, teclado e IME | R9 | — |
| 8 | clangd | completion, definição, diagnóstico, pelo relé | tempo até o servidor responder |
| 9 | rust-analyzer | o mesmo | o mesmo |
| 10 | Contexto da toolchain | R11 | — |
| 11 | Diagnósticos e ações LSP | diagnóstico no texto e no painel do supervisor; code action aplicada | — |
| 12 | Recuperação após queda | R4 | texto perdido |
| 13 | Encerramento sem órfãos | R5 | processos restantes |
| 14 | Projeto não confiável | R3 | marcas |
| 15 | Desempenho | R1, R2, R12 em arquivos de 100, 2 mil e 20 mil linhas | latência, abertura, RSS, CPU |

Cada item é comparado com o editor atual nas mesmas condições, quando o
editor atual faz a mesma coisa. A prova também mede R6, R7, R8, R10 e R13.

### 12.3 A tabela de atalhos (R8)

Digitar insere; `Shift` com setas, `Home`, `End`, `Ctrl+Shift+setas`
seleciona; `Ctrl+C`, `Ctrl+X`, `Ctrl+V`; `Ctrl+Z` e `Ctrl+Shift+Z`;
`Ctrl+A`; `Ctrl+S`; `Ctrl+D` (duplicar linha); `Ctrl+Y` (apagar linha);
`Ctrl+/` (comentar); `Alt+Shift+↑/↓` (mover linha); `Tab` e `Shift+Tab`
(recuar); `Ctrl+Space` (completion); `Ctrl+B` (definição); `Shift+F6`
(renomear); `Alt+Enter` (ações); `Ctrl+F` (buscar). Os atalhos globais da
Kinein (paleta, áreas) continuam com a QML.

## 13. A jornada de self-hosting

Reproduzível, como aceite da fase F (§14), não da prova:
1. abrir o repositório da Kinein;
2. editar arquivos Rust, C++ e QML;
3. usar o rust-analyzer e o clangd; navegar por símbolos e dependências;
4. rodar o Cargo e o CMake; compilar o core e a UI; rodar os testes;
5. depurar o core;
6. trabalhar com o Git;
7. reiniciar a IDE sem perder alteração;
8. lidar com ferramenta ausente, travada ou incompatível.

A prova cobre 1, 2 (Rust e C++), 3 e 7. Self-hosting não substitui os
testes de segurança e de engenharia.

## 14. A implementação na 0.4, se a DP-10 aprovar

As fases seguem o briefing; cada fatia é um item do 62 §3.4 e um commit,
com aceite, evidência e reversão.

| Fase | O quê | Aceite | Reversão |
| --- | --- | --- | --- |
| A | Esta avaliação | Os gates de docs verdes | Documento |
| B | A prova (§12) | Os quinze itens com número ou NÃO PROVADO | Apagar o diretório |
| C | GO ou NO-GO | Os critérios do §1; decisão do autor | — |
| D | Integração por partes, depois do B2 e do B3: o item Qt Quick; o ciclo do `nvim` no lançador; os documentos; a configuração LSP e o relé; o contexto do core; a segurança; a recuperação; os painéis; a regressão | Cada parte com os testes da §16 | O editor atual segue ao lado, atrás de uma opção, até a fase E |
| E | Remover o que ficou redundante | Cada comportamento do teste antigo coberto pelo novo; só então o código sai | Reverter o commit de remoção |
| F | Consolidação | O gate completo, os testes adversariais, os projetos reais e a jornada do §13 | — |

Os dois motores não convivem de forma permanente: a convivência dura da
fase D até a fase E.

**Estado em 2026-11-15.** As fases A, B (§20) e C (§21) estão prontas. A
fase E só começa quando o produto integrado passar nas condições do §21,
além dos testes da §16; a fase D leva também os requisitos do §20.5.

## 15. O esforço economizado

- **Medido:** 7.571 linhas de produção candidatas a sair (§9); as 6.738 da
  classe Adaptar diminuem, em quanto a prova dirá.
- **Trabalho que deixa de existir no roteiro:** o B8 inteiro (ENG-55), a
  divisão do ENG-24, a decisão e a troca de motor da 0.6 (57) e os recursos
  que o `TextEdit` bloqueia e o Neovim tem: split, dobras, virtualização
  (só a área visível é desenhada), inlay hints e cada recurso LSP novo, que
  hoje custa método IPC, análise no core, ponte e QML.
- **O que entra:** o item Qt Quick, a entrada, o IME, a acessibilidade, o
  RPC, o supervisor, o relé e o Lua. A referência externa é o Neovim-Qt:
  8.727 linhas de GUI, mais a biblioteca de RPC (§4). A Kinein precisa de
  menos (abas, linha de comando e mensagens ficam na QML), mas acrescenta a
  acessibilidade, que o Neovim-Qt não tem.
- **Sem número ainda:** o saldo líquido. A prova mede as linhas próprias
  (R13), e o saldo só é afirmado com elas. Multi-cursor e minimap não vêm
  de graça: nenhum dos dois é nativo no Neovim 0.12.

## 16. Como a arquitetura nova se verifica

Pela ordem de preferência da R-GATE-13 do 63.

| O que impedir | Como |
| --- | --- |
| Dois donos do mesmo buffer | Tipo: o core não tem função que escreva texto; só a função Lua com `changedtick` esperado. Teste de propriedade: edição com `changedtick` velho é recusada |
| Sessão LSP duplicada | Estrutura: o mapa do lançador tem uma entrada por servidor e raiz. Teste: dois `vim.lsp.enable` não criam segundo processo |
| Resposta velha aplicada | Teste: resultado da Kinein com `changedtick` velho é descartado; o do LSP, pelo `ContentModified` do Neovim |
| Processo sem supervisão | Compilador: o `nvim` só nasce pelo lançador (`forbid`, 64 §2.7). Teste: `kill -9` no core, e o `nvim` sai |
| Comando não autorizado | Teste do projeto hostil com Landlock; mutante que tira a regra Landlock e reprova o teste |
| Perda de arquivo após falha | Teste de queda (R4), com mutante que desliga o rascunho |
| Build e análise divergentes | Teste R11: trocar o padrão no preset muda o diagnóstico do clangd |
| `nvim` que não sobe | Teste com binário ausente e com versão abaixo da mínima: mensagem clara, sem travar a UI |
| Configuração da máquina | Teste com `init.lua` e plugin plantados na configuração da pessoa: nada deles roda |

## 17. As perguntas da avaliação

- **O Neovim reduz a infraestrutura que a Kinein mantém?** O medido diz
  que sim no código que sai (7.571 linhas) e no roteiro que some (§15). O
  saldo depende do código novo, que a prova mede.
- **A integração é mais simples e sustentável que o editor atual?** No
  modelo de donos, sim: um dono do texto, um cliente LSP por documento, sem
  o texto inteiro indo e voltando. No componente gráfico, a Kinein passa a
  manter um renderizador; a prova diz o tamanho.
- **A Kinein continua capaz de experiências especializadas de embarcados?**
  Sim: SVD, registradores, memória, gravação, depuração e as ferramentas de
  engenharia ficam nos painéis e no core (§7).
- **O core continua com responsabilidades bem definidas?** Sim: projeto,
  toolchain, build, depuração, confiança, persistência, lançador e relé. Ele
  sai do caminho do texto.
- **SDKs e toolchains novos entram sem mexer no motor de edição?** Sim: o
  core produz o contexto e a configuração do servidor; o Neovim só conecta.
- **A 0.4 fica mais perto de uma versão pública confiável?** Só se a prova
  passar nos critérios do §1. Um GO acrescenta trabalho à 0.4 e tira o B8,
  o ENG-24 e a troca de motor da 0.6. Um NO-GO mantém o motor atual com o
  B8.
- **Há alternativa menos custosa com benefício comparável?** Pelas
  restrições do produto (Qt Quick, sem Widgets nem webview sem benefício
  demonstrado), não foi achada:
  - o `TextEdit` evoluído (B8 mais o 21 §M5.4, caminhos a e b) mantém o
    cliente LSP e todos os recursos de editor como código próprio;
  - KTextEditor e QScintilla são `QWidget`;
  - Monaco e CodeMirror exigem webview;
  - o Helix não tem API de embarque nem protocolo de UI remota;
  - um motor próprio é o último recurso do 21 §M5.4.

  A prova, se reprovar, deixa o `TextEdit` evoluído como alternativa, com
  o B8.

## 18. O que é decisão do autor

1. **Modo de edição:** sem modo por padrão (RNF8), com o modo do Vim como
   opção da pessoa. Recomendado.
2. **Quem grava o arquivo:** o Neovim (`:write`), com o core avisado, ou o
   core, pelo `BufWriteCmd`, com a gravação guardada de hoje. A prova mede
   os dois; a recomendação sai dela.
3. **Recuperação:** o swap do Neovim na pasta de estado da Kinein, ou o
   rascunho da Kinein pelo espelho do buffer, sem swap. A prova mede os
   dois (R4).
4. **Sem Landlock:** abrir o projeto não confiável só com a configuração
   fechada, ou abrir sem o editor novo. Recomendado: sem o editor novo.
5. **Distribuição:** `nvim` fixado dentro do AppImage, ou exigido do
   sistema com versão mínima. Recomendado: fixado.
6. **Multi-cursor:** plugin de terceiro, com licença conferida, ou fora da
   0.4.
7. **O verificador de correções:** pedidos do core injetados pelo relé
   (§7), ou o verificador passa a pedir ao Neovim.
8. **O mandato de 2026-07-11** (21 §M5.4, sem Neovim embutido) fica revisto
   pela DP-10, qualquer que seja o resultado.

As decisões tomadas em 2026-11-15 estão no §21.

## 19. O que não foi medido

Refeito em 2026-11-15, depois da prova. A latência, a abertura, a memória,
a recuperação, os órfãos, o projeto hostil, a árvore AT-SPI dos dois
editores e o código próprio foram medidos (§20). Não foram:

- **O IME real.** A máquina de medição não tem fcitx5 nem ibus; o R9 foi
  provado só no caminho de eventos do Qt (§20.2).
- **O Windows.** Nada rodou no Windows: nem o Job Object do §8, nem o pipe
  nomeado.
- **O produto integrado.** O supervisor da prova faz o papel do core, e a
  janela da prova é mínima: R1 e R12 integrados, o R10 com o documento
  inteiro e o ciclo do `nvim` no lançador do B2 ficam para a fase D (§21).
- **A renderização por GPU.** As duas UIs mediram no renderloop `software`,
  offscreen.
- **A jornada de self-hosting do §13** como jornada; a prova cobre partes
  dos passos 1, 2, 3 e 7.
- **A compatibilidade da ABI tree-sitter** entre as gramáticas do core e o
  Neovim; a prova realça por regex.
- **A queda só do core na base.** No produto, o texto vive na UI, que
  sobrevive à queda do core.
- **O `qmlls`** contra o clangd e o rust-analyzer em recursos.
- **O multi-cursor e o verificador de correções** (§18, itens 6 e 7).
- **A causa da variação da base entre as sessões** (§20.6).
- O Neovim-Qt, o Neovide e o goneovim foram lidos, não executados.

## 20. Os resultados da prova (fase B)

Medido em 2026-11-14 e 15. Os números desta seção são da rodada de
2026-11-15 (§20.1), salvo onde a data diz outra coisa.

### 20.1 Método e evidência

- **Onde está.** O código e os resultados estão no ramo local
  `prova/eng-60`, sem histórico em comum com o `main` e fora do produto:
  a prova no `2d1ac38d`, o isolamento da pasta de runtime no `6e828799` e
  os resultados no `3beea8f5`. As medidas do INF-26 e do INF-28 (§20.4)
  vieram depois, no `37ed0789` e no `8e35e471`. O README do ramo diz como
  reproduzir.
- **A pilha.** A UI Qt Quick (`NvimView`, offscreen) fala com o
  `nvim --embed` 0.12.6 por socket. Um supervisor em Rust faz o papel do
  core: cria o `nvim` com Landlock e uma sonda que falha fechado, faz o
  relé LSP e guarda o espelho do buffer e o rascunho. O Lua da Kinein dá a
  edição sem modo, os atalhos do §12.3 e o LSP pelo relé.
- **A linha de base.** O produto no `308c1561`, em Release (GCC 16.2.1, com
  o endurecimento do `KINEIN_ENABLE_HARDENING`). Uma instrumentação mexe só
  no harness de digitação que o produto já tem
  (`ui/src/typing_perf_harness.cpp`), atrás de variáveis de ambiente, sem
  mudar o caminho da tecla. Ela dá a abertura até o primeiro quadro com o
  texto (R2), a linha digitada (R4), a queda provocada (R4) e o processo de
  pé para ler memória, CPU e a árvore AT-SPI (R10, R12).
- **A rodada.** A partir de uma cópia limpa do `6e828799`, em 2026-11-15,
  das 07:18 às 07:39 (-03:00): uma etapa por vez, sem outra medida em
  paralelo, na ordem suíte funcional, R10, recuperação da base, controle
  positivo, desempenho e render.
- **O isolamento.** Tudo roda offscreen; o R10 roda num Xvfb, numa sessão
  D-Bus sem ativação de serviço e numa pasta de runtime próprios. Cada
  rodada tem `HOME`, `XDG_*` e `XDG_RUNTIME_DIR` próprios. A listagem da
  pasta de runtime de quem roda é igual antes e depois da rodada.
- **As fixtures.** Projetos C++ e Rust compilados de verdade. As de
  desempenho saem da geradora do `medir-core.py` do `308c1561`: arquivos
  Rust de 85, 2.463 e 19.970 linhas (2, 60 e 487 funções). O "arquivo de 20
  mil linhas" do §1 é o de 19.970.

### 20.2 Os quinze itens e os critérios

| # | Item | Resultado |
| --- | --- | --- |
| 1 | Inicialização controlada | 0 marcas, com 7 vetores plantados: `init.lua` e plugin da pessoa, pacote em `site/pack`, plugin em `site`, `VIMINIT`, `VIMRUNTIME` falso e `NVIM_APPNAME`. O `nvim` fica pronto em 24,0 ms de mediana (p95 26,7), e o editor desenha o primeiro quadro em 76,3 ms, com o cache de fontes frio (não é o R2) |
| 2 | msgpack-RPC | 0 erros de protocolo nos testes que os contam: desenho, atalhos, as três edições (na UI e no supervisor) e as 15 rodadas de desempenho da prova |
| 3 | Desenho | 6 posições no arquivo de 2.463 linhas (abertura, PageDown, PageUp, `Ctrl+End`, `Ctrl+Home`, 45 setas para baixo), 38 linhas cada, conferidas contra o buffer: 0 glifos faltando ou sobrando |
| 4 | Editar C++ | O disco fica igual ao esperado, o build passa e o programa imprime 42, gravando pelo `nvim` e pelo core |
| 5 | Editar Rust | O disco fica igual ao esperado, o build passa e o programa imprime 18 |
| 6 | Atalhos (R8) | 30 de 30 ações do §12.3; depois de cada uma, a tecla seguinte insere |
| 7 | Unicode (R9) | 5 de 5 no caminho de eventos do Qt: acento morto, `ç`, `中文` (4 células), `😀` (2 células) e o combinante U+0301. **IME real NÃO PROVADO** |
| 8 | clangd | Primeira lista útil em 1,24 s, a frio; o popup em 41 a 43 ms, a quente; a definição em `util.h:8`; o diagnóstico chega do teclado ao buffer em 43,6 ms |
| 9 | rust-analyzer | Primeira lista útil em 3,92 s, a frio, pelo `Ctrl+Space`; o popup em 72 a 79 ms, a quente; a definição em `util.rs:1`; o diagnóstico chega do salvar ao buffer em 4,7 ms (entre 5 e 234 ms nas rodadas anteriores ao commit) |
| 10 | Contexto (R11) | Trocar C++17 por C++20 no preset leva o buffer de 4 erros a 0 em 1.398 ms; o relé espera a sessão anterior terminar (978 ms) antes de aceitar a nova |
| 11 | Ações LSP | O diagnóstico aparece no buffer e no painel; `Alt+Enter` remove o include; renomear muda 3 arquivos; o build passa depois |
| 12 | Recuperação (R4) | Pelo espelho, 0 teclas perdidas em 6 de 6 casos (queda do `nvim`, da UI ou do supervisor, 0 ou 200 ms depois da última tecla). O `nvim` volta em 77 a 78 ms; a UI ou o supervisor reabrem e restauram em 2,35 a 2,63 s. O mutante sem rascunho recupera 0 de 10 |
| 13 | Órfãos (R5) | 0 órfãos nos 4 cenários (queda da UI, do supervisor ou do `nvim`, e saída normal). Depois da queda do `nvim`, ficam 1 clangd e 1 rust-analyzer, reconectados ao `nvim` novo |
| 14 | Projeto hostil (R3) | 0 marcas. As 14 tentativas de execução foram barradas: `:!`, `system()` com texto e com lista, `jobstart()`, `vim.system()`, `vim.uv.spawn`, `io.popen`, `os.execute`, terminal, `ld.so` como trampolim, o próprio `nvim`, `:make`, `vim.ui.open` e `system(3)` pela FFI. A modeline é ignorada, o build sem confiança é recusado e o clangd roda sem `--query-driver` |
| 15 | Desempenho | §20.3 |

| R | Veredito |
| --- | --- |
| R1 | **Passa na prova**, nos três tamanhos (§20.3). **No produto integrado, NÃO PROVADO**: a estimativa pessimista passa do limite de p95 no arquivo de 2.463 linhas |
| R2 | Passa: 32,0 ms contra 47.249 ms, no arquivo de 19.970 linhas |
| R3 | Passa (item 14). Os mutantes reprovam pelo motivo certo: sem a segunda camada do Landlock (a que o Lua da Kinein aplica dentro do `nvim`), o supervisor recusa entregar o editor; sem Landlock e sem a sonda, aparecem 13 marcas |
| R4 | Passa. O espelho perde 0 teclas. O swap perde as 40 em 2 de 6 casos (queda do `nvim` ou do supervisor, 0 ms depois da última tecla). A base perde as 40 em 2 de 2 casos (queda da UI 0 ou 200 ms depois da última tecla); com a queda 2,5 s depois, recupera as 40 em 2 de 3 rodadas, e a terceira terminou com erro do harness (§20.4) |
| R5 | Passa (item 13) |
| R6 | Passa: a edição com `changedtick` velho é recusada, a certa é aplicada e o espelho fica igual. O supervisor só muda texto pela função que confere o `changedtick` e pela abertura da recuperação, que roda antes de a UI receber o editor |
| R7 | Passa: 4 buffers, 1 processo clangd, 1 lançamento; a segunda conexão é recusada |
| R8 | Passa (item 6) |
| R9 | **NÃO PROVADO** para IME real, como o critério manda dizer (item 7) |
| R10 | **Caminho viável, abaixo do editor atual.** Pela libatspi (a biblioteca do Orca), o editor da prova aparece focado e editável; digitar "Olá " dá o texto e o cursor certos (posição 4), e a seta para baixo dá a linha seguinte. A prova expõe só a linha do cursor; o editor atual expõe o documento (1.912 caracteres na fixture, cursor em 970). É regressão a pagar na fase D (§21) |
| R11 | Passa (item 10) |
| R12 | **Passa na prova:** 83,4 contra 582,4 MiB de RSS, no arquivo de 19.970 linhas. **Integrado, NÃO PROVADO.** A estimativa pessimista soma à base o `nvim` e o supervisor: 593,4 MiB de PSS, 1,04 vez a base |
| R13 | 4.134 linhas próprias contra as 7.571 da classe Eliminar (§9): Lua 603; Rust 1.700, sem os testes (Landlock 182, RPC 186, espelho 207, relé 436, o resto do supervisor 689); C++ e QML 1.831 (msgpack 374, cliente RPC 180, `NvimView` 1.010, janela e ligação 267). Fora da conta: o controle de teste da UI e os testes. **É um piso:** faltam o documento inteiro na acessibilidade, a integração com o core real, o realce por tree-sitter e o Windows |

### 20.3 Desempenho

Pelo método do A3.3-1 (`medir-performance.sh`): 5 rodadas alternadas por
lado, 40 teclas por rodada (200 amostras por lado), na linha do meio, com
150 ms entre as teclas, offscreen, janela de 1280x800. A memória é a soma
dos processos depois de digitar: na base, a UI e o core; na prova, a UI, o
supervisor e o `nvim`. A CPU é a desses processos durante a digitação.

| Linhas | Lado | Tecla→quadro, mediana / p95 / pior (ms) | Abertura (ms) | CPU ao digitar (s) | RSS / PSS (MiB) |
| --- | --- | --- | --- | --- | --- |
| 85 | base | 10,76 / 12,89 / 16,12 | 245,3 | 2,31 | 211,9 / 201,7 |
| 85 | prova | 8,21 / 8,71 / 9,33 | 29,4 | 0,25 | 73,9 / 61,4 |
| 2.463 | base | 9,15 / 10,31 / 11,12 | 971,9 | 13,06 | 278,1 / 267,7 |
| 2.463 | prova | 8,45 / 9,00 / 9,68 | 30,4 | 0,27 | 75,0 / 62,5 |
| 19.970 | base | 20,75 / 27,15 / 85,71 | 47.249 (46.199 a 47.926) | 24,75 | 582,4 / 572,0 |
| 19.970 | prova | 9,21 / 9,94 / 10,16 | 32,0 | 0,38 | 83,4 / 70,9 |

O primeiro quadro da janela sai em 1.005 a 1.039 ms de mediana na base e em
37,55 a 38,24 ms na prova, conforme o tamanho.

**O R1 e a estimativa pessimista.** A janela da prova é mínima, e a IDE
inteira paga mais por quadro. O `render.py` mediu o custo de cada quadro
durante a digitação, 3 rodadas por lado no arquivo de 2.463 linhas: na
base, 728 quadros, mediana 1, p95 7 e pior 9 ms; na prova, 120 quadros,
mediana 2, p95 2 e pior 2 ms. A estimativa soma à prova a diferença: 5 ms
no p95 e 7 ms no pior caso.

| Linhas | Limite do p95 (1,25×) | p95 da prova | Estimado | Limite do pior (2×) | Pior da prova | Estimado |
| --- | --- | --- | --- | --- | --- | --- |
| 85 | 16,11 | 8,71 | 13,71 | 32,24 | 9,33 | 16,33 |
| 2.463 | 12,89 | 9,00 | **14,00** | 22,24 | 9,68 | 16,68 |
| 19.970 | 33,94 | 9,94 | 14,94 | 171,42 | 10,16 | 17,16 |

No arquivo de 2.463 linhas, o estimado passa 1,11 ms do limite. Por isso o
R1 integrado é condição do §21, e não consequência da prova.

### 20.4 O que a comparação mostrou no produto atual

Cada achado vira um item do [62](62-base-publica-0.4.md) §3.4, num commit
próprio, e a referência entra aqui com ele.

- **Abrir o arquivo de 19.970 linhas leva de 46,2 a 47,9 s** nesta rodada
  (24,8 s numa medida de 2026-11-15 anterior ao commit, §20.6). Naquela
  medida, um relógio externo confirmou 26,4 s, com a thread da UI a 100%
  de um núcleo, e 26,9 s sem a sonda da abertura. É o INF-25 do 62 §3.4.
- **A queda da UI até 200 ms depois da última tecla perde as 40 teclas.**
  No `EditorPersistenceController.qml` do `308c1561` (lido), cada mudança
  reinicia o temporizador do rascunho (1.500 ms) e, com a gravação
  automática ligada, o dela (2.000 ms), sem prazo máximo. A queda 2,5 s
  depois recupera as 40. É o INF-26 do 62 §3.4: com 200 teclas, em cerca
  de 100 s de digitação, a queda logo depois da última perde as 200, em 3
  de 3 rodadas (ramo `prova/eng-60`, `37ed0789`).
- **A UI desenha uns 6 quadros por tecla durante a digitação:** 728
  quadros para 120 teclas, com intervalo mediano de 35 ms entre eles; a
  prova desenha 120. A causa não foi investigada. É o INF-27 do 62 §3.4.
- **O harness de digitação do produto recomeça quando o editor relê o
  arquivo.** O controle positivo terminou com "teclas perdidas: 40
  enviadas, 0 inseridas" em 1 de 3 rodadas, e em 1 de 1 na primeira rodada
  completa (§20.6). Não é tecla perdida: depois da gravação automática, o
  editor relê o arquivo para comparar com o que gravou, e o harness toma a
  releitura pela abertura e recomeça. Com a queda 6 s depois da última
  tecla, o erro aparece em 3 de 3; com a queda 1 s depois, em 0 de 3 (ramo
  `prova/eng-60`, `8e35e471`). A queda do R4 e a memória do R12 da fase D
  seguram o processo depois da digitação e dependem da correção. É o
  INF-28 do 62 §3.4.

### 20.5 Defeitos, contornos e requisitos da fase D

A prova achou e corrigiu oito defeitos da integração. Cada um vira
requisito da fase D, com o teste que o pega:

1. **Colar sobre uma seleção deixava o modo `niI`**, e a tecla seguinte
   virava comando do Vim. Requisito: depois de cada ação composta, a camada
   sem modo volta à inserção; o teste dos atalhos confere que a tecla
   seguinte insere.
2. **O `clipboard=unnamedplus` sobrescrevia a área de transferência** ao
   substituir a seleção. Requisito: digitar ou colar sobre a seleção não
   muda a área de transferência.
3. **O `Tab` e o `Shift+Tab` da Kinein escondiam o salto do snippet.**
   Requisito: com snippet ativo, o `Tab` salta de campo.
4. **O `vim.snippet` supõe `selection=inclusive`**, e a camada sem modo
   usa `exclusive`: digitar sobre o campo deixava a última letra.
   Requisito: o contorno da prova, com teste.
5. **O relé matava o clangd antes do reinício, e uma corrida recusava a
   reconexão** (R11). Requisito: o lançador espera a sessão anterior
   terminar antes de aceitar a nova.
6. **Um caminho de socket acima de 108 bytes desligava o LSP em silêncio.**
   Requisito: o socket fica num caminho curto, e um caminho longo demais
   falha com mensagem.
7. **Emojis eram descartados**, porque o `isPrint` era testado por unidade
   UTF-16. Requisito: a entrada trata o texto por ponto de código.
8. **A fábrica de acessibilidade era instalada depois da QML**, e o leitor
   de tela recebia texto vazio. O teste de Unicode não pegava, porque lê a
   view direto. Requisito: a fábrica antes do motor QML, e o R10 lido pela
   AT-SPI, não pela view.

**Os contornos e a API interna.** Os defeitos 1 e 4 contornam o
comportamento do `vim.paste` e do `vim.snippet` no 0.12.6. O `Ctrl+/` usa a
API interna `vim._comment`, que não tem `:help` e por isso não é contrato
(§3). É o que torna necessário o `nvim` fixado (§21, decisão 5): a regressão
do editor roda a cada troca de versão (§11). Nenhum relato foi aberto no
projeto Neovim.

**O limite do Landlock, confirmado.** Um Lua que já roda carrega uma
biblioteca nativa (`package.loadlib`), e o Landlock não impede: o direito
`EXECUTE` vale para o `execve`. Depois da carga, a sonda continua barrada,
porque a contenção do `execve` vale também para o código da biblioteca. O
que impede o projeto de chegar a esse ponto são os vetores passivos
fechados: o `runtimepath`, o `package.path` e o `package.cpath`, a modeline
e o `exrc` (§8). Requisito: o teste hostil da fase D cobre os vetores
passivos, e o limite fica escrito.

### 20.6 Repetições e limites da medida

**Repetições declaradas.**
- A primeira rodada completa foi a do `2d1ac38d`, em 2026-11-15, das
  06:43 às 07:03. Ela mostrou que os sockets e o barramento de
  acessibilidade iam à pasta de runtime de quem roda; o `6e828799` corrige
  isso, e a rodada do §20.1 é a de depois. Os vereditos das duas são os
  mesmos.
- O controle positivo da base rodou três vezes na rodada do §20.1, e uma
  terminou com o erro do harness (§20.4).
- Antes do `2d1ac38d`, dois testes reprovaram por defeito do próprio teste:
  às vezes o rust-analyzer manda o snippet do `push` com o `;`, e o teste
  digitava outro; e o teste da recuperação lia o buffer no meio da
  restauração. Depois da correção, o teste do rust-analyzer rodou mais 7
  vezes, para os dois ramos rodarem, e o da recuperação mais 2.
- No R10, o tema GTK3 do Qt carrega a `libatk-bridge`, que registra uma
  segunda aplicação, vazia, com o mesmo pid. O teste escolhe o registro
  que tem janela e grava quantos achou.

**A base variou entre as sessões; a prova, não.** Numa medida de
2026-11-15 anterior ao commit, com o mesmo método, a base abriu o arquivo
de 19.970 linhas em 24.781 ms, com a tecla em 17,68 / 31,5 / 64,63 ms; na
rodada do §20.1, em 47.249 ms. O binário da base daquela medida e o atual,
rodados de novo em alternância, deram de 47,5 a 48,7 s: a diferença vem do
estado da máquina, não do build, e a causa não foi isolada. No arquivo de 2.463
linhas, a mediana da tecla na base foi 6,53 ms naquela medida, abaixo da
prova (9,40), e 9,15 ms na rodada do §20.1, acima dela (8,45). A mediana
nesse tamanho não decide nada. Nenhum veredito muda entre as duas medidas.

**Limites da medida.**
- O custo de quadro sai do renderloop `software`, com resolução de 1 ms;
  não há medida com GPU.
- O clangd roda sem índice de fundo (`--background-index=false`).
- A prova realça por regex; o editor atual, por tree-sitter. A latência da
  prova não paga o realce do produto.
- A base roda com o endurecimento do release do produto, e a prova sem ele.
- A estimativa pessimista do R1 cobre só a diferença do custo de quadro, e
  não o resto da IDE no mesmo processo.

## 21. A decisão (fase C)

Decisão do autor de 2026-11-15, sobre os resultados do §20: **a DP-10
escolhe o Neovim como motor de edição e cliente LSP da Kinein Vectis, com
condições.** O mandato de 2026-07-11 ([21](21-roadmap-de-longo-prazo.md)
§M5.4) fica revisto: a exclusão do Neovim embutido deixa de valer, e o
tree-sitter continua obrigatório.

**As condições.** O editor atual fica no produto, ao lado do novo, até o
produto integrado passar nas quatro. Só então a fase E remove o que ficou
redundante.

| Condição | Por que a prova não basta | Aceite no produto integrado |
| --- | --- | --- |
| R1 e R12 pelo método da prova | A prova desenha uma janela mínima, e a IDE inteira paga mais por quadro (§20.3) | Os limites do §1 contra o editor atual, nos três tamanhos, pelo método do §20.3 |
| R10 com o documento inteiro | A prova expõe só a linha do cursor; o editor atual expõe o documento (§20.2) | Pela AT-SPI, o documento inteiro, o cursor e a seleção, como no editor atual |
| IME real | A máquina de medição não tem framework de IME (R9) | Composição com fcitx5 ou ibus: pré-edição no cursor e texto confirmado certo |
| Windows | Nada rodou no Windows (§19) | Os testes da etapa 10 do [62](62-base-publica-0.4.md) §3, com a contenção pelo Job Object (§8) |

Se uma condição reprovar, a DP-10 volta ao autor com a medida, e o editor
atual fica. O B8, o ENG-24 e a parte do editor do ENG-20 voltam como estão
escritos no 64 e no 63.

**A ordem.** A integração (fase D) começa depois do B2 e do B3 do 64 §8 e
não passa na frente deles; o INF-23 segue no início do B2. O B4, o B5 e o
B6 tratam a parte do editor pelo §10. O B8, a divisão do ENG-24 e a parte
do editor do ENG-20 não são implementados antes da fase E.

**As decisões do §18** (autor, 2026-11-15), com a evidência do §20:
1. **Sem modo por padrão**, com o modo do Vim como opção da pessoa.
2. **O core grava o arquivo**, pelo `BufWriteCmd`, com a gravação guardada
   de hoje. Os dois caminhos passaram; com o core gravando, o acesso a
   arquivo tem um dono só.
3. **A recuperação é pelo rascunho da Kinein a partir do espelho, sem
   swap.**
4. **Sem Landlock, o projeto não confiável abre sem o editor novo.** O que
   a pessoa vê nesse caso depois da fase E, quando o editor atual não
   existir mais, é desenho da fase D.
5. **O `nvim` fixado dentro do AppImage.** A prova depende de contornos
   sobre o comportamento de uma versão e de uma API interna (§20.5).
6. e 7. **Seguem abertas:** a prova não mediu multi-cursor nem o
   verificador de correções.

## Referências

- [Neovim — API e `api-contract`](https://neovim.io/doc/user/api.html)
- [Neovim — eventos de UI](https://neovim.io/doc/user/api-ui-events.html)
- [Neovim — LSP](https://neovim.io/doc/user/lsp.html)
- [Neovim — `LICENSE.txt`](https://github.com/neovim/neovim/blob/master/LICENSE.txt)
- [Neovim-Qt](https://github.com/equalsraf/neovim-qt), [Neovide](https://github.com/neovide/neovide), [goneovim](https://github.com/akiyosi/goneovim)
- [Qt — `QSGTextNode`](https://doc.qt.io/qt-6/qsgtextnode.html)
- [Linux — Landlock](https://docs.kernel.org/userspace-api/landlock.html)
- [Windows — limites de Job Object](https://learn.microsoft.com/windows/win32/api/winnt/ns-winnt-jobobject_basic_limit_information)
