# 45 — Etapa 4: o backend de novo — LSP profundo, edição inteligente e a biblioteca dos compiladores

> Aberto em 2026-09-19, ao fim da Etapa 3 (roadmap 44; a E3-7 ficou
> pendente e em 2026-09-22 foi agendada para a 0.3.5). Este documento é o
> **brief** da etapa: o que o autor pediu, o
> que a IDE já faz (medido no código, não de memória), o que falta em
> relação ao VS Code e às IDEs JetBrains, e a ordem proposta. **A
> arquitetura fina de cada fatia é escrita na sessão que a implementa,
> antes do código** — a regra desde a Etapa 2.

## 0. Cache de compilação — entra aqui, na fase dos embarcados (2026-09-26)

O autor trouxe um relatório de pesquisa sobre **evitar recompilação**. Ele foi
quebrado em duas metades, por classe:

- o que **já existe pronto no mercado** — `ccache`, `sccache`, orquestrados
  como processo, do jeito que a IDE já faz com `cmake`, `ninja` e `gdb` —
  virou alvo em
  [`../especificacoes/cache-de-compilacao.md`](../especificacoes/cache-de-compilacao.md);
- a **hipótese de longo prazo** — diff de AST, *fingerprints*, modo sombra —
  foi para a pasta nova [`../pesquisa/`](../pesquisa/README.md), que **não
  autoriza implementação**.

**Fase:** 0.4, junto dos embarcados, antes da 1.0. Nada disto entra na 0.3 —
mesma regra que mantém o trilho e os embarcados fora dela.

## 1. O pedido do autor (2026-09-19, literal e interpretado)

> "A próxima sessão vai focar no aprofundamento do backend, adicionar
> funcionalidades convenientes que o VS Code/JetBrains possuem. Integração
> mais profunda/responsiva com os LSP, indentação automática e etc.
> Otimizar/polir a biblioteca de funcionalidades dos compiladores. Creio
> que o foco se volta para o backend novamente."

Sim: o foco volta ao backend. A Etapa 2 e a Etapa 3 foram tela (HUD,
tool windows, moldura comum). O que o autor sente agora no teste é o
**miolo**: o que o editor faz enquanto ele digita, o que o language
server devolve e quão rápido, e o quanto a IDE entende do compilador do
projeto. Três famílias:

1. **LSP profundo e responsivo** — mais do protocolo, e o que já existe
   sem travar nem atrasar.
2. **Edição inteligente** — indentação automática de verdade, e as
   conveniências de digitação que se espera de uma IDE.
3. **A biblioteca dos compiladores** — o modelo de compilação por arquivo
   exposto, os diagnósticos do compilador (não só do LSP), e as toolchains
   cross como cidadãs (o `44` §8 já anotava isso).

## 2. O que a IDE já faz (medido em 2026-09-19, no código)

### 2.1 LSP — o que o core fala hoje

`crates/kinein-core/src/lsp/` (manager, session, sync, query, transaction,
registry, parse_symbols, diagnostics_merge) fala **catorze** métodos
`textDocument/*`: `didOpen`, `didChange`, `didClose`, `didSave`,
`publishDiagnostics` (recebe), `completion`, `hover`, `definition`,
`references`, `rename` (com prévia e transação/rollback), `codeAction`
(com prévia), `documentSymbol`, `semanticTokens/full`,
`switchSourceHeader` (clangd). Servidores: clangd, rust-analyzer,
basedpyright (com o interpretador do projeto), ruff (formatador/lint).
`rename` e `codeActions` já rodam **fora do laço** (`Core::defer_then`,
`LoopEvent::Continue` — `arquitetura/04` §6); `workspace/symbol` é pedido
pelo Search Everywhere (`#nome`) e chega pela **mesma** resposta C++
(`lspSymbolsResolved`) que `documentSymbol` — o que impediu a aba
Símbolos de usá-lo (E3-2; `40` §7.81).

**Não fala**: `signatureHelp` (a assinatura enquanto digita os
argumentos), `inlayHint` (tipos e nomes de parâmetro inline —
rust-analyzer e clangd têm), `documentHighlight` (as ocorrências do
símbolo sob o cursor), `formatting`/`rangeFormatting`/`onTypeFormatting`
(hoje formatar é `format.text` por ferramenta externa), `foldingRange`
(o folding é do tree-sitter local — `lang/folding.rs`), `codeLens`,
`prepareRename`, `semanticTokens/full/delta` e `range`,
`workspace/didChangeWatchedFiles` (o watcher existe — `fswatch.rs` — mas
não avisa o servidor), `$/progress`/`window/workDoneProgress` (o
"indexando…" do rust-analyzer não aparece), `completionItem/resolve`
(documentação do item ao selecionar), `textDocument/typeDefinition`,
`implementation`, `declaration`, `callHierarchy`, `typeHierarchy`.

### 2.2 Edição — o que o editor faz hoje

O editor é QML sobre um `TextEdit` com o `EditorController` (790 linhas —
**o arquivo em débito da catraca**) e controllers filhos (texto, busca,
completion, ações). `insertNewline` (`EditorTextController.qml`) é uma
**heurística em JavaScript**: repete a indentação da linha, continua `//`
e `/* … */`; o Tab/Shift+Tab indenta a seleção; `closerBrace` e o
auto-close de pares são setting (M4.1). Realce, folding, outline e
"locals" vêm do **tree-sitter no core** (`lang/`: C, C++, Rust, Python;
`syntaxTree.update`), com o realce semântico do LSP por cima quando o
servidor responde. Diagnósticos na calha e no tooltip; a lâmpada do
Alt+Enter; F2 pula de problema.

**Não há**: indentação por gramática (o `indents.scm` do tree-sitter —
nada de "Enter dentro de `{` abre um nível e o `}` fecha"), reindent ao
colar, `Ctrl+/` que respeita a linguagem, mover linha (Alt+Shift+↑/↓
existe: `GlobalShortcuts` — verificar o que faz), duplicar linha,
seleção expandida por sintaxe (Ctrl+W da JetBrains), multi-cursor, o
"surround with", a formatação ao salvar como setting por linguagem
(existe `format.text` por gesto), auto-import (é `codeAction` do
servidor — falta a UI convidar), a assinatura no popup, os inlay hints,
o realce das ocorrências, a "breadcrumb" por símbolo (a barra de caminho
mostra só pastas — `EditorPane` mostra `ui › qml › git › GitRules.qml`).

### 2.3 Compiladores — o que o core sabe hoje

O **índice do projeto** (`index/`, pilar 0 do `42`) lê pastas, arquivos e
declarações e dá o **contexto de compilador por arquivo**
(`index.context`: a unidade da CDB para C/C++ com compilador, `-std`,
`-I`, `-D`; o crate/target/edition do Cargo; o interpretador do Python) —
a barra de status resume, o tooltip detalha. O modelo de projeto vem do
**CMake File API** (`cmake/`, presets, kits com chip/alvo/sysroot/SVD,
`cdb.rs`) e do `cargo metadata`. Toolchains: catálogo conferido,
instalação com SHA-256, sysroot lido, kit importado de SDK (Yocto,
Buildroot), `toolchain/`. Diagnósticos de compilador **só via build**
(`build.run` parseia a saída e vira `problems`); o clang-tidy do projeto
entra pelo clangd e pela CDB.

**Não há**: o "por que este arquivo compila assim" como tela (o contexto
existe, a exposição é uma linha); diagnósticos do compilador **sem
buildar** (um `clang -fsyntax-only` / `cargo check` por arquivo salvo,
com os flags reais da CDB — o que o `44` §8 chamou de "diagnósticos do
compilador, não só do LSP"); a escolha da toolchain por arquivo/alvo na
UI além do kit; o cross-compile com o sysroot do kit provado ponta a
ponta com um compilador de distro (`integracoes/39` deixou a dica); o
`compile_commands.json` gerado para projetos sem CMake (Makefile puro,
`bear`); o cache de compilação (ccache/sccache) detectado e oferecido.

## 3. O que se propõe — as fatias, por família

Cada fatia é **contrato primeiro** quando toca o protocolo (o LSP quase
sempre toca: cada método novo é um `lsp.<x>` + evento), medida antes e
depois (a latência da tecla — `KINEIN_PERF_TYPING*` — é a régua desta
etapa: nada pode piorá-la), harness, registro, um commit. Os números da
régua hoje: a digitação em release-hardened e o primeiro frame estão no
`40` §7.79 e §4.2.1.

### 3.1 LSP profundo e responsivo

```text
L1  workspace/symbol com sinal próprio no C++ (lspWorkspaceSymbolsResolved)
    — desbloqueia a segunda fonte da aba Símbolos (E3-2) e o `#nome`.
    Contrato: nenhum método novo; o C++ separa os dois sinais.
L2  signatureHelp: `lsp.signatureHelp` (deferido), popup discreto acima da
    linha enquanto o cursor está entre parênteses; fecha no `)`.
L3  documentHighlight: `lsp.documentHighlight` no cursor parado (debounce
    200 ms), pintado como fundo suave nas ocorrências; some ao mover.
L4  inlayHint: `lsp.inlayHints { range }` para a janela visível; desenho
    inline no editor (é o mais difícil de desenhar — medir a latência).
    Setting por linguagem: ligado no Rust, desligado no C++ por padrão?
    (decisão do autor).
L5  formatting pelo servidor: `lsp.formatting`/`rangeFormatting` e
    `onTypeFormatting` (clangd formata ao digitar `;` `}` `\n`); "formatar
    ao salvar" como setting por linguagem; `format.text` continua para
    ruff/rustfmt quando o servidor não formata.
L6  $/progress e workDoneProgress: o "indexando… 43%" do rust-analyzer na
    barra de status (o `lspStatusController` já existe).
L7  didChangeWatchedFiles: o watcher do core avisa o servidor (Cargo.toml,
    CMakeLists, compile_commands.json) — hoje o servidor descobre tarde.
L8  completionItem/resolve e a documentação no popup; o auto-import como
    ação convidada ("importar `foo` de `bar`" na lâmpada).
L9  typeDefinition/implementation/declaration/callHierarchy — navegação
    (Ctrl+Shift+B, Ctrl+Alt+H da referência).
```

### 3.2 Edição inteligente

```text
E1  Indentação por gramática NO CORE: `lang.indent { path, line }` responde
    o nível a partir do tree-sitter (`indents.scm` por linguagem — C, C++,
    Rust têm no upstream; Python pelas regras de `:` e dedent). O QML
    pergunta no Enter e no `}`; a heurística JS cai para o "sem gramática".
    Medir: a tecla não pode esperar o core — a resposta é síncrona (< 1 ms
    na árvore já parseada) ou a UI usa a heurística e corrige depois.
E2  Reindent ao colar (a mesma regra sobre o bloco colado); Ctrl+/ por
    linguagem (`//` `#` e o bloco `/* */` na seleção).
E3  Duplicar linha (Ctrl+D), mover linha/seleção (Alt+Shift+↑/↓ — medir o
    que existe), apagar linha (Ctrl+Y da referência, Ctrl+Shift+K do VS
    Code — decidir e registrar no gate de atalhos).
E4  Seleção por sintaxe (Ctrl+W expande, Ctrl+Shift+W recua) sobre a
    árvore do core (`lang.selectionRange`).
E5  Breadcrumb por símbolo: a barra de caminho ganha `› fn parse_log`
    (o outline já tem os símbolos e a posição do cursor).
E6  Multi-cursor (Alt+clique, Ctrl+Alt+↑/↓) — é o mais invasivo no
    `TextEdit`; só depois de E1–E5 e se a catraca do EditorController
    for paga (o arquivo em débito precisa ser dividido ANTES).
```

### 3.3 A biblioteca dos compiladores

```text
C1  Diagnósticos do compilador sem buildar: ao salvar um .c/.cpp com
    unidade na CDB, `clang -fsyntax-only` (ou o compilador da unidade)
    com os flags reais, em job curto; ao salvar um .rs, `cargo check`
    incremental (já existe como comando — falta o gatilho por salvar e a
    fusão com os do LSP: `diagnostics_merge.rs` já funde fontes).
C2  "Por que este arquivo compila assim": um painel (na aba Símbolos? no
    tooltip do status? — decidir) com compilador, padrão, includes,
    defines, sysroot, o target do file-api, e o botão "abrir a CDB".
C3  Toolchain por alvo provada: um projeto CMake com o sysroot de um
    compilador cross de distro (arm-none-eabi ou aarch64-linux-gnu)
    compilando pelo kit, ponta a ponta, com a foto e o número; o que
    faltar vira fatia.
C4  Projetos sem CMake/Cargo: Makefile puro com `bear` gerando a CDB
    (oferecido, nunca silencioso), e o clangd passando a funcionar.
C5  ccache/sccache detectados e oferecidos como ação de configuração
    (o catálogo de ações já tem prévia e consentimento).
C6  O que o `44` §8 listou e não está acima: o mapa de símbolos por unidade
    (quem define o que, em qual TU) — depende do índice; e o `42` §8
    ("efeito JetBrains") continua sendo o critério de pronto.
```

## 4. A ordem proposta, e por quê

**L1 → E1 → L2 → L3 → C1 → L5 → E2/E3 → L6/L7 → L4 → E4/E5 → C2 → L8/L9 →
C3–C5 → E6.** L1 é uma dívida da Etapa 3 e custa pouco. E1 é o que o
autor citou nominalmente ("indentação automática") e muda a sensação de
cada Enter. L2/L3 são o que mais se sente faltando ao lado do VS Code.
C1 é a primeira peça da "biblioteca dos compiladores" que o usuário vê.
Inlay hints (L4) e multi-cursor (E6) ficam depois porque são os que mais
mexem no desenho do editor — e o `EditorController` em 790 linhas tem de
ser dividido antes de E6 (é a catraca cobrando a dívida antiga).

## 5. O que não muda

Contrato primeiro; catracas; medir antes de afirmar; registro datado com
evidências; um commit por fatia; nunca `git checkout <arquivo>`; a IDE
nunca roda `sudo` nem instala em silêncio; a placa do autor nunca é
gravada sem pedido; nada de push/release/AppImage sem o autor. E a
**régua desta etapa**: a latência da tecla e o primeiro frame não pioram
— toda fatia mede os dois antes e depois.

## 6. Pendências herdadas que esta etapa deve olhar

- **E3-7** (Grafana) — obrigatória na 0.3.5; deixou de ser apenas uma sessão
  curta de polimento. O fluxo está em
  `../especificacoes/grafana-ui-ux-0.3.5.md` e a execução no roadmap 48.
- A escolha do slot esquerdo (explorer/Git) **não é persistida** —
  `SettingsValues` novo, se o autor sentir falta.
- Logs/shell de container sem projeto — um `cwd` opcional no
  `container.open`, se o autor sentir falta.
- O `EditorController` (790) — a divisão é pré-requisito de E6 e alivia
  todas as fatias E*.

## 7. Adendo de 2026-10-01 — a Etapa 4 como motor da 0.4 dos embarcados

> **Decisão do autor em 2026-10-01.** A 0.4 continua sendo a versão dedicada
> aos embarcados ([roadmap 49 §5](49-frontend-0.3.6-e-sequencia-0.5.md)) e
> leva junto a integração mais profunda com compiladores e ecossistemas
> desta etapa. O embarcado é o **tema**; as famílias L/E/C deste documento
> são o **motor**. Cada fluxo embarcado puxa as fatias de que precisa, em vez
> de a etapa correr em paralelo como uma segunda versão dentro da 0.4.

Já entregues na série 0.3: **L1** (símbolos do workspace com sinal próprio) e
**E1** (indentação pela gramática no core), ambos na V6 do
[roadmap 47](47-estrutura-da-v0.3.md).

| Fluxo embarcado da 0.4 | Fatias que ele puxa |
| --- | --- |
| Criar ou abrir um projeto para um chip (kit) | **C3**: toolchain cross provada ponta a ponta (ex.: `arm-none-eabi`, sysroot, flags do alvo), pelo kit |
| Entender por que não compila | **C1**: diagnósticos ao salvar, sem build completo, com as flags reais da CDB; **C2**: "por que este arquivo compila assim" (compilador, padrão, defines, sysroot, target do file-api) |
| Build rápido e repetido | **C5**: `ccache`/`sccache` detectados e oferecidos (decisão da §0) |
| Caber na flash e na RAM | **C7**, extensão de `build.size`: o `size.rs` já mede por seção e pelas regiões `MEMORY` do linker script; falta **por símbolo** (map file / `nm`) e o delta contra o build anterior |
| Projeto sem CMake/Cargo (Makefile de fabricante) | **C4**: `bear` gerando a CDB, oferecido e nunca silencioso |
| Editor que entende o alvo | **L2** signature help, **L3** document highlight, **L5** formatação pelo servidor, **L6** progresso de indexação, **L7** arquivos observados (CMakeLists, `compile_commands.json`) |
| Rust embarcado, Zephyr, ESP-IDF | O **ecossistema decide** (`cargo`, `west`, `idf.py`) e a IDE mostra o que ele decidiu, pelo [modelo semântico](../especificacoes/modelo-semantico-do-projeto-0.4.md) |

Regras que continuam valendo, sem reabrir:

- **A IDE detecta deriva, não resolve versão.** Quem resolve dependência é a
  ferramenta do ecossistema; a IDE a executa por Job e explica o resultado.
- Nada roda `sudo` nem instala em silêncio; a placa do autor nunca é gravada
  sem pedido.
- A régua da §5 vale: latência da tecla e primeiro frame não pioram.

**Fora da 0.4**, por não servirem ao fluxo embarcado e por mexerem pesado no
desenho do editor: **L4** (inlay hints) e **E6** (multi-cursor), além de
**L8/L9** e **E4/E5**, que ficam conforme o uso real pedir. Eles seguem nesta
lista como candidatos de versões seguintes. A ordem da §4 passa a ser lida
**dentro** dessa seleção.

A arquitetura executável desta seleção, com o mapa de donos contra código
duplicado, os fluxos passo a passo, os contratos e as provas, está no
[roadmap 52](52-arquitetura-executavel-da-0.4.md).
