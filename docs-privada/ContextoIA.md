# ContextoIA - Continuidade do Kinein Vectis

> ## ESTE ARQUIVO E' UM **LOG**. NAO E' O ESTADO DO PROJETO.
>
> Classe **LOG** (`docs/README.md`): registro datado do que foi decidido **naquele
> dia**. Envelhecer e' a funcao dele; nunca reescrever entrada antiga.
>
> ```text
> Serve para:     "por que isto e' assim?"  -> leia a entrada datada.
> NAO serve para: "o que existe hoje?"      -> isso se MEDE no codigo.
> ```
>
> **Ate 2026-07-17 este arquivo estava em PRIMEIRO na ordem de precedencia,
> descrito como "estado real e decisoes vigentes".** Ele e' um log append-only e
> ja foi rebaixado. Log em primeiro e' o mecanismo que faz uma sessao nova confiar
> em registro velho: foi assim que uma IA reimplementou um seletor que o autor
> mandou remover no mesmo dia, e listou como pendente um harness entregue havia
> 24 horas.
>
> **Estado vivo:** `PONTO_ATUAL.md` (fila) e `GUIAIA.md` (mapa) — e os dois se
> curvam ao codigo. **Contrato:** `AGENTS.md` + `docs/arquitetura/ARCHITECTURE.md`.

Registra decisoes de produto/arquitetura para quem continuar o repositorio, em
ordem cronologica. Contrato e estado detalhado vivem em
`docs/arquitetura/03-ipc-protocol.md`, `docs/arquitetura/ARCHITECTURE.md` e
`docs/roadmaps/BACKEND_TO_UI_UX_ROADMAP.md`.

## Ambiente revalidado apos troca de distro (2026-07-08, Arch)

A maquina atual e Arch Linux. Duas falhas que aparecerem de novo tem causa
conhecida e correcao simples:

- **`ninja: error: '/usr/lib/x86_64-linux-gnu/...' missing`**: cache de CMake
  configurado na distro anterior (caminhos Debian/Ubuntu). Reconfigurar por
  cima NAO corrige — o CMake preserva valores antigos do cache. Correcao:
  apagar o diretorio de build inteiro e rodar `cmake --preset <preset>` do
  zero (feito em 2026-07-08 para `build/linux-clang-release-hardened`; o
  orfao `build/dev-local-release`, de um preset antigo, foi removido).
- **`error due to GNU_PROPERTY_1_NEEDED_INDIRECT_EXTERN_ACCESS` ao executar**:
  binario compilado em outra distro fazendo copy relocation contra simbolo
  protegido da Qt do Arch. O Qt6 do Arch propaga `-mno-direct-extern-access`
  (GCC) / `-fno-direct-access-external-data` (Clang) via
  `/usr/lib/cmake/Qt6/Qt6Targets.cmake`; basta rebuildar limpo na maquina
  atual que o flag entra sozinho. Nao adicionar flag manual em preset.

## Organizacao documental e diagnostico de terminal (2026-07-16)

- **Documentacao reorganizada por assunto** (sessao autonoma, sem commit): os
  `.md` tecnicos foram movidos por `git mv` para
  `docs/{arquitetura,build,seguranca,roadmaps}` (nomes preservados);
  referencias raiz-relativas atualizadas em docs, comentarios de codigo, scripts
  e CMake; `docs/README.md` reescrito como indice; 0 links quebrados; `cargo
  check` verde. Documentos numerados agora vivem sob subpastas (ex.:
  `docs/arquitetura/03-ipc-protocol.md`, `docs/roadmaps/24-...`). Raiz mantem
  `README/MANUAL/Tutorial/AGENTS` e os pessoais. Plano interno completo em
  `docs-legada/PLANO_ORGANIZACAO_E_HANDOFF.md` (nao publicar).
- **Scroll do agente Claude no KV Context — causa-raiz identificada:** o Claude
  interativo usa tela alternada (sem scrollback) e nao tem flag inline como o
  `--no-alt-screen` do Codex; a IDE ainda nao encaminha a roda ao app em tela
  alternada. Diagnostico e duas opcoes de correcao em `docs/roadmaps/26` secao
  11. Nao aplicado: exige gesto visual (regra do proprio R26).

## Direcao do produto

- Produto: Kinein Vectis.
- Objetivo: IDE open source, Linux-first, rigida por padrao, visualmente
  familiar para usuarios de IDEs JetBrains, mas com identidade propria.
- Uso inicial: projeto de uso proprio do autor, com foco em qualidade alta e
  ergonomia diaria.
- Filosofia: a IDE orquestra ferramentas maduras; nao reimplementa compilador,
  LSP, debugger, build system ou analisadores quando ja houver ferramenta aberta
  consolidada.
- **DECISAO OBRIGATORIA (usuario, 2026-07-11): adotar as TECNOLOGIAS que os
  plugins do Neovim usam, direto na nossa IDE, SEM depender do Neovim/Vim.**
  Nao embutir o Neovim (a tensao modal x identidade JetBrains e alta e foi
  descartada); em vez disso plugar as bibliotecas maduras por baixo:
  - **tree-sitter** (OBRIGATORIO): parser incremental para realce, indentacao
    e textobjects — substitui/complementa o hibrido regex + semantic-tokens
    atual. E a maior lacuna vs. o ecossistema Neovim que ja da pra plugar
    direto (biblioteca C + gramaticas). Candidato a fatia propria de alto
    valor; ancorar na decisao de engine do editor (M5.4 do docs/roadmaps/21).
  - LSP (clangd/rust-analyzer) e DAP (lldb-dap) JA sao plugados direto pelo
    core Rust — nada de Neovim no meio.
  Regra: quando uma capacidade do Neovim for desejada, buscar a
  biblioteca/tecnologia madura que ele orquestra e plugar ela, nunca o editor
  inteiro.
- **AMPLIACAO (usuario, 2026-07-11): best-of-breed OPEN-SOURCE, venha de onde
  vier — MAS so o que e VSCodium/Open VSX, NUNCA proprietario da Microsoft.**
  Alem das tecnologias do ecossistema Neovim, adotar tambem o que ha de melhor,
  maduro e solido no ecossistema **VSCodium** (a build 100% open source do
  VSCode, que usa o registro **Open VSX**, NAO o Marketplace fechado da MS).
  **DESCARTAR explicitamente qualquer plugin/extensao PROPRIETARIO do VS Code**
  (ex.: extensao C/C++ da MS `ms-vscode.cpptools`, Pylance, o C# devkit, o
  debugger proprietario — licenca fechada, fora do Open VSX). Principio unico:
  **plugar a TECNOLOGIA/ferramenta open-source consolidada, NUNCA o editor,
  o extension host, nem extensao de licenca fechada.** Na pratica, a maioria
  das extensoes OPEN-SOURCE de VSCodium e plugins de Neovim envolvem as MESMAS
  ferramentas maduras (LSP servers como clangd/rust-analyzer, adapters DAP como
  lldb-dap/codelldb, tree-sitter, formatadores, linters) — essas a gente ja
  pluga ou pluga direto pelo core Rust. Onde uma extensao open-source do Open
  VSX embrulhar uma biblioteca/ferramenta open-source unica e boa que ainda
  nao usamos, avaliar plugar essa ferramenta (nunca rodar a extensao JS/TS).
  Criterio: maduro + solido + **open source de verdade (Open VSX / nao-MS-
  proprietario)** + cabe no modelo "core Rust orquestra ferramenta madura via
  contrato tipado + UI propria das specs".
- **ESCOPO INICIAL (usuario, 2026-07-11): so C/C++ e Rust + ferramentas de
  CONVENIENCIA/auxilio (tipo tree-sitter).** Nada de plugin/tech de Python ou
  outras linguagens por enquanto. Fazer um ambiente C++ e Rust bem feito ja e
  trabalhoso — foco total nisso agora. Ao avaliar plugar qualquer ferramenta,
  perguntar: serve C/C++/Rust ou e conveniencia de dev (highlight/indent/nav/
  refactor/debug)? Se for so de outra linguagem, ADIAR.
- **NORTE AUTORITATIVO de adocao de ferramentas: `KINEIN_VECTIS_OPEN_PLUGIN_
  ADAPTATION_ROADMAP.md` (raiz do repo, fornecido pelo usuario 2026-07-11).**
  Antes de adotar QUALQUER ferramenta/tecnologia externa, seguir esse doc:
  os 4 MODOS (A=integracao direta do executavel/protocolo — PREFERIDO;
  B=referencia de UX; C=port seletivo com licenca compativel; D=so
  referencia); o GATE de auditoria (licenca SPDX, telemetria, rede em
  runtime, shell, segredos, pin por commit); preferencia de licenca
  MIT>Apache>BSD>MPL, GPL/AGPL so referencia (nunca copiar p/ o core
  MIT/Apache); ordem P0->P3. Ja integrados MODE-A: clangd, rust-analyzer,
  lldb-dap, cargo, cmake (file-api/presets), git. P0 faltando: tree-sitter
  (obrigatorio), EditorConfig, ripgrep, fd. Quando a adocao de tools comecar
  de verdade, criar `docs/tooling/OPEN_COMPONENT_REGISTRY.json` + ADRs em
  `docs/adr/` como o roadmap manda (Fase 0). Regra de ouro do roadmap: nao
  instalar plugins do Neovim/VSCodium dentro do Kinein; reaproveitar
  protocolo/ferramenta aberta + UI/UX propria + zero telemetria + controle
  total.
- **MANDATO DE REFERENCIA PROFISSIONAL (usuario, 2026-07-15):** toda
  funcionalidade de IDE nova ou alterada deve estudar a fonte oficial atual
  pertinente de **Code OSS (`microsoft/vscode`)**, **IntelliJ IDEA Community**,
  **Zed**, **Lapce** e/ou **Apache NetBeans**, seguindo a secao 2.1 do roadmap
  acima. A referencia serve para extrair comportamento, invariantes, erros,
  cancelamento, concorrencia, seguranca e testes com qualidade comprovada; nao
  autoriza copiar funcao/classe/modulo, fazer traducao mecanica, transplantar o
  runtime do host ou escolher codigo antigo so por conveniencia. Registrar no
  doc do dominio a revisao, subsistema, licenca/modo, licoes e adaptacao para
  Qt/QML -> CoreClient -> protocolo -> Rust Core. "VS Code" aqui significa o
  Code OSS publico, nunca distribuicao, Marketplace, servico ou extensao
  proprietaria. O terminal e o exemplo canonico: estudar backend/ciclo de PTY
  do Code OSS, mas implementar com `portable-pty`, parser VT, IPC tipado e
  renderer Qt/QML proprios da Kinein.

## Decisoes recentes de UX/workspace

- O fluxo de projeto/workspace deve seguir a memoria muscular de IDEs JetBrains.
- O seletor de pasta deve ser proprio da IDE, nao o dialogo nativo do desktop.
- A UI nao deve listar o filesystem diretamente. A navegacao do seletor passa
  pelo core via `workspace.browse`.
- A arvore de projeto deve ser limpa, leve e confortavel, mais proxima do
  Project View das IDEs JetBrains do que do explorer pesado/fechado do VS Code.
- O painel Project deve parecer uma arvore de projeto da IDE, nao um gerenciador
  de arquivos. Usar hover/selecao sutis, indentacao clara, poucas bordas
  internas e acoes compactas por icone.
- Clique em diretorio no explorer do projeto deve expandir/recolher, nao trocar
  workspace implicitamente.
- Abrir outro workspace: mesmo root pode preservar abas; root diferente limpa
  abas e estado visual do workspace anterior.
- Fechar projeto deve fechar automaticamente abas abertas e limpar explorer.

## PONTE DE ATENCAO — bugs RESOLVIDOS em 2026-07-11 (perda de dados no editor)

> STATUS: **TODOS RESOLVIDOS e validados** (gate verde + smoke + sonda +
> repro no app real). Esta secao fica como WATCHPOINT permanente: os tres
> sintomas vinham do MESMO refactor abandonado; se algo parecido voltar,
> comece por aqui.

**Resumo (o que estava quebrado -> corrigido):**
- [RESOLVIDO] Fechar aba nao fazia nada -> `closeTab` voltou a chamar
  `documents.closeTab(index)`.
- [RESOLVIDO] Arquivo ficava "modificado" para sempre depois de salvar ->
  `handleFileSaved(path)` voltou a derivar savedContent do content.
- [RESOLVIDO] **PERDA DE DADOS**: arquivo abrindo VAZIO, conteudo
  "copiado" entre arquivos e ZERADO no disco -> removida a linha bugada
  em `markCurrentModified` (detalhe abaixo).

**Causa comum:** um agente anterior deixou um refactor
"DocumentLifecycleController" **abandonado e NAO COMMITADO** no working
tree. **PONTE DE ATENCAO / regra:** o working tree acumula tudo (o
usuario faz UM commit unico); um refactor de UI pela metade fica ATIVO e
quebra producao. NAO deixar refactor de UI sem fiar em Main.qml, sem
registrar no CMakeLists e sem validar ponta a ponta (build + smoke +
sonda de open/edit/save/close). Se aparecer um `.qml` novo untracked que
"deveria" estar ligado, e sinal de refactor incompleto.

### Detalhe dos tres bugs

Um agente anterior deixou um refactor **abandonado e NAO COMMITADO** no
working tree ("DocumentLifecycleController") que quebrou o fluxo de
edicao:
- `EditorController.closeTab(index)` passou a emitir `tabCloseRequested`
  — sinal que NINGUEM escutava (o `DocumentLifecycleController.qml` que
  trataria ficou orfao, sem instanciacao, sem CMakeLists, untracked).
  Resultado: **fechar aba nao fazia nada**.
- `EditorEventRouter.onFileSaved(path)` chamava
  `handleFileSaved(path)` mas a versao do refactor esperava
  `(path, savedContent)`; sem o segundo arg, `savedContent=undefined` e
  o arquivo ficava **modificado para sempre depois de salvar**.
Correcao (2026-07-11): revertido ao comportamento do HEAD (closeTab ->
documents.closeTab; handleFileSaved(path) deriva savedContent do content
e marca modified=false); arquivo orfao e helpers mortos
(documentSnapshot*/modifiedDocuments/closeTabByPath) removidos. Validado
por sonda de open/edit/save/close + gate verde + smoke. **Licao: o
working tree acumula tudo (o usuario faz UM commit); um refactor pela
metade fica ativo e quebra producao — nao deixar refactor de UI sem
fiar/CMakeLists/validar ponta a ponta.**
**CAUSA RAIZ da PERDA DE DADOS achada e corrigida (2026-07-11):** o
mesmo refactor abandonado adicionou UMA linha em
`EditorDocumentController.markCurrentModified`:
`openFilesModel.setProperty(currentTab, "content", text)` (o HEAD NAO
tinha — o content so era sincronizado por `storeCurrentEditor` em
pontos definidos: troca de aba, save). Essa linha grava o texto do
surface no modelo em eventos de texto; ao trocar de aba / restaurar
sessao, com o surface momentaneamente vazio ou apontando para outro
buffer, ela CORROMPIA o content do modelo — resultado: arquivo abrindo
VAZIO, conteudo "copiado/substituido" entre arquivos, e ZERADO no disco
ao salvar o content corrompido. Reproduzido com o app real (sessao
[a.txt(conteudo), b.txt(vazio)] ativa=b: a.txt.content virava "" e
modified=true); removida a linha, a.txt preserva o conteudo. Fix =
reverter ao HEAD (markCurrentModified so mexe no flag `modified`;
content sincroniza por storeCurrentEditor). Guard de regressao na sonda
de open (markCurrentModified NAO altera content). **Explica os 3
sintomas do usuario de uma vez.**

Radar: `crates/kinein-core/src/tools.rs` tem 2 testes de deteccao
(`detected_tool_reports_path_and_version`,
`fd_detection_accepts_fdfind_binary_name`) FLAKY por corrida de
isolamento (ETXTBSY ao exec de script fake em paralelo); pre-existente
(tools.rs ja vinha modificado antes desta sessao). Passa na maioria das
rodadas; candidato a fix de isolamento proprio (EXEC_LOCK em todos os
testes que exec, ou fsync).

## Estado tecnico atual

- Arquitetura: Qt/QML UI <-> JSON-RPC local/stdin-stdout <-> Rust core.
- Protocolo IPC atual: `0.59.0` (2026-07-16). **0.59 REMOVE o dominio
  `aiBridge.*`** e, com ele, a superficie do KV Context, os perfis allowlisted e
  as settings `aiCliProfile`/`aiCliFlatTranscript`. O bridge era a interferencia:
  injetava argumentos no `claude`/`codex` (`--no-alt-screen`,
  `--ax-screen-reader`) e filtrava `CSI 3 J` da propria aplicacao — a IDE se
  metendo entre o programa e o terminal. Agora uma CLI de IA e um programa como
  outro qualquer: abrir o terminal e rodar `claude`, mesmo PTY/emulador/contrato
  de um `ls`. No `terminal.rs` cairam o `ScrollbackPreserver` e o
  `open_command_with_policy`; nao existe mais politica por programa. O KV Context
  pode voltar como UI pura (atalho que abre terminal comum), sem regra de negocio
  no core. 0.58 (revertido) tinha adicionado o toggle de transcript plano.
  0.57 preserva forma e piscagem do
  cursor pedidas por aplicações via DECSCUSR, incluindo a barra steady nativa
  usada por TUIs, sem perfil por Claude/Codex. 0.56 acrescenta a largura
  autoritativa em células VT a cada span de `event.terminal.render`, mantendo
  texto, seleção e cursor na mesma grade mesmo com glifos largos ou fallback de
  fonte. 0.55 entrega capacidades de
  workspace híbrido Cargo+CMake, seleção tipada de `buildSystem` em
  build/quality/test e `run.script` confinado, sem interpolação de shell. 0.54
  faz `lsp.semanticTokens` ecoar `path` e `version`, permitindo à UI descartar
  respostas de outro documento ou buffer já editado. 0.53 entrega Workspaces
  recentes globais: snapshot tipado, fixação, remoção/limpeza, disponibilidade
  calculada no core e retomada por `workspace.open`; 0.52 separa e persiste a largura da
  sessão ativa (300–720px), mantém `Project` independente, endurece a
  reconciliação do scroll durante nova saída e preserva o transcript das AI
  CLIs contra `CSI 3 J`; 0.51 tornou o KV Context terminal-first: Codex usa o
  modo inline oficial, o render VT expõe application-cursor/bracketed-paste/
  alternate-screen e a UI ganhou largura responsiva/maximização e resize
  coalescido. 0.50 formalizou o AI CLI Bridge
  Claude/Codex e persistência responsiva do layout; 0.49 trouxe Git diário
  (branches/checkout/create, pull/push jobs, stash); 0.48 trouxe `fs.replace`
  transacional; 0.47 trouxe preview/aplicar/cancelar de workspace edits LSP;
  0.46 trouxe `syntaxTree.update` com Tree-sitter incremental. 0.45 trouxe T2
  — watcher lazy/debounced de mudanças externas via `notify`,
  `event.fs.changed`/`event.fs.watchError` e `fs.write { expectedContent }`
  com erro `FILE_CHANGED`; 0.44 trouxe D2.3 —
  múltiplas sessões de terminal, com `id` em todos os comandos/eventos, `HashMap` no manager e
  abas independentes na UI; 0.43 trouxe `scrollback`/`scrollbackMax` para a
  barra de rolagem; 0.42 trouxe o
  terminal profissional D2 —
  `terminal.resize`/`terminal.scroll` + `event.terminal.render` (grid de
  spans/cursor) no lugar de `event.terminal.data`, PTY real via portable-pty
  + emulador vt100; copiar/colar via singleton C++ Clipboard; 0.40
  trouxe `draft.save`/`draft.clear` +
  `drafts` na resposta de `workspace.open` — rede de segurança contra
  perda de dado da fatia S1/docs/seguranca/23: autosave de buffer não salvo em
  SQLite + escrita atômica de `fs.write`; 0.39 trouxe `lsp.restart` +
  `event.lsp.restarted` — reinício de LSP travado da M4.3b; 0.38 trouxe
  `settings.rigorProfile` — perfis de rigor Strict/Balanced/Relaxed da
  M4.5; do 0.21 ao 0.37
  entraram: `format.text`, code actions, symbols, sessao por workspace,
  `cmake.*`, `cargo.*`, `runConfig.*`, `debug.*`, `git.*`
  status/diff/stage/commit, `git.blame`/`git.log`/`git.commitDiff`
  (0.33, M3.4), `lsp.switchSourceHeader` (0.34, T1), o `Diagnostic`
  enriquecido com endLine/endColumn/code (0.35, T6),
  `settings.get`/`settings.set` (0.36, M4.1) e `lsp.completion`
  `isIncomplete` (0.37, fix do autocomplete)). Lista completa de
  comandos/eventos e contrato: `docs/arquitetura/03-ipc-protocol.md` (nao duplicar
  essa lista aqui).
- Formatacao orquestrada (fatia M1.1 de `docs-privada/diario/18-daily-driver-plan.md`,
  2026-07-09): `Ctrl+Alt+L` formata o buffer atual via `format.text`
  (rustfmt/clang-format, stdin/stdout, cwd na raiz, sem tocar disco; a UI
  substitui o texto preservando cursor clampado e marca a aba como
  modificada). Format-on-save e Salvar tudo já estão implementados; a fila de
  saves descarta respostas obsoletas. Design completo e decisoes: docs-privada/diario/18.
- Job system assincrono/cancelavel (`job.list`/`job.cancel`, `event.job.*`)
  cobre `build.run`, `quality.run`, `test.run` e `environment.scan`: cada um
  responde `{ jobId }` na hora e emite `event.<dominio>.finished` com o
  resultado real. Arquitetura: `docs/arquitetura/ARCHITECTURE.md` §7. Impacto/estado de UI:
  `docs/roadmaps/BACKEND_TO_UI_UX_ROADMAP.md` (P0).
- Diagnostics parcialmente unificados: modelo comum `Diagnostic` em
  `kinein-protocol` usado por `event.build.diagnostic`/`event.quality.diagnostic`
  (faltam ids estaveis, actions e `logRef` — ver roadmap P1).
- `workspace.browse`/`fs.*`/`fs.findFiles` seguem confinados ao workspace
  aberto; `fs.*` nunca navega fora dele. Detalhe de cada metodo:
  `docs/arquitetura/03-ipc-protocol.md`.
- Project Health minimo (primeira etapa, 2026-07-08): banner discreto no topo
  da area do editor (`shell/ProjectHealthBanner.qml` +
  `workspace/ProjectHealthController.qml`), composto so com dados existentes
  (`workspace.kind`, `toolsList` do scan/detect, `scanningEnvironment`) —
  nenhum contrato IPC novo. Estados: ambiente nao verificado (acao
  "Verificar" -> `environment.scan`), ferramentas obrigatorias ausentes por
  kind (acao "Ferramentas" -> aba de tools), kind desconhecido (informativo) e
  scan em andamento. Quando o ambiente esta saudavel o banner some (decisao de
  produto: banner discreto, nao selo verde permanente); erros de compilacao e
  build em andamento ficam fora dele de proposito — ja tem badge de Problemas
  e status bar. Ha um "×" que dispensa o aviso atual ate a causa mudar.
- A UI sobe `kinein-core` como processo filho via `CoreClient`. O usuario testa
  pelo icone "Kinein Vectis" do menu de aplicativos (`scripts/kinein-vectis`),
  que prefere binarios release (`build/linux-clang-release-hardened/ui/
  kinein-vectis`, `target/release/kinein-core`) e cai para debug se release
  nao existir. Depois de alterar core/UI, rebuild obrigatorio:
  `cargo build --release -p kinein-core` +
  `cmake --build --preset dev-local-release`.
- **Toolchain local (2026-07-08, maquina atual, Arch):** GCC 16.1.1 e
  Clang 22.1.6. Com Clang recente,
  o proprio codigo gerado pelo moc do Qt dispara `-Wctad-maybe-unsupported`;
  com GCC nativo, o codigo QML gerado (`qmlcache_loader`,
  `qmltyperegistrations`, JS AOT de `Main.qml`/`FolderPickerDialog.qml`)
  dispara `-Wfloat-equal`/`-Wuseless-cast`/`-Wmissing-declarations`. Nenhum dos
  dois e bug nosso — e codigo gerado pelo Qt que este toolchain de ponta
  considera suspeito. Workaround **local** (`CMakeUserPresets.json`,
  gitignored, nao afeta CI/outras maquinas): `dev-local`/`dev-local-release`
  usam `g++`/`gcc` nativos e `KINEIN_WARNINGS_AS_ERRORS=OFF`.
  `cmake/KineinStrictOptions.cmake` (rastreado no git) permanece inalterado.
  Se o proximo agente ve warnings so em `.rcc/qmlcache/*`, `moc_*` ou
  `*_qmltyperegistrations.cpp`, e esse problema conhecido, nao regressao.
  Em outra maquina, ajustar `CMakeUserPresets.json` conforme o toolchain local
  (nao editar `cmake/KineinStrictOptions.cmake` sem motivo registrado).

## Modularizacao pos-V1

Fase de "monolito modular" concluida em 2026-07-05: core, protocolo e CLI
quebrados por responsabilidade; rename kernwerk -> Kinein Vectis concluido.
Detalhe completo (arquivos, pastas, verificacao): ver
`docs/arquitetura/15-engineering-debt-and-refactor.md`.

## Strict mode

- Rust: `unsafe_code = "forbid"`, warnings como erro, clippy pedantic/nursery,
  sem `unwrap`/`expect`/`panic`/`todo`/`dbg!` fora de casos aceitos por testes.
- C++/Qt: C++23, warnings-as-errors, sanitizers em Debug, hardening/LTO em
  Release via `cmake/KineinStrictOptions.cmake`.
- QML (desde 2026-07-08): qmllint estrito (zero warnings) no gate via
  `scripts/verificar-qml.sh`. Padrao do repositorio: `pragma
  ComponentBehavior: Bound` onde ha delegates, `required property` para
  roles, acesso por id qualificado (nunca `parent.parent.x` nem resolucao
  implicita de escopo). Detalhe: `docs/arquitetura/06-strict-mode.md`; degraus futuros de
  rigor: `docs-privada/diario/18-daily-driver-plan.md`.
- Requisitos (funcionais e nao funcionais) e trade-offs de arquitetura estao
  explicitos em `docs/arquitetura/19-architecture-tradeoffs.md` (criado 2026-07-09 a
  pedido do usuario). Decisao estrutural nova ou excecao entra la, com
  ganho/custo/gatilho de revisita. O doc tambem reafirma: **a UI/UX de
  `docs/specs/` e inegociavel** — visual nao se inventa nem se "melhora" de
  passagem.
- **A UI atual NAO reflete docs/specs/** (paleta, dimensoes, iconografia,
  regioes faltantes). A convergencia e gradual e VINCULANTE, decidida com o
  usuario em 2026-07-09 (nao sera remake big-bang):
  `docs/roadmaps/20-ui-spec-convergence-plan.md` tem as regras (R1: UI nova nasce
  conforme spec; R2: fatia de convergencia nunca se mistura com feature),
  o inventario de divergencias e a ordem C0-C6 amarrada aos marcos de
  docs-privada/diario/18. Nenhum agente deve tratar a aparencia atual como referencia — a
  referencia e a spec.
- Futuramente: seletor de nivel de rigidez (Strict padrao / Balanced /
  Relaxed so por escolha explicita).

## Prioridade imediata (2026-07-05, decisao do usuario)

UX/UI basica (workspace, explorer, abas, layout, sidebar, paineis, syntax
highlighting, build/test/quality, LSP MVP) esta **feita** — ver histórico no
git e em `docs/roadmaps/BACKEND_TO_UI_UX_ROADMAP.md`. A partir de 2026-07-05 a ordem
mudou:

1. **Nao empilhar mais fases de backend antes de voltar ao Qt.** O usuario
   pediu explicitamente para nao atrasar o frontend: o projeto ja e
   medio/medio-grande, entao cada entrega de backend "invisivel" tem que vir
   acompanhada de um ganho de UI logo em seguida, nao de mais uma fase de
   contrato.
2. [feito 2026-07-05] Cancelar build/test/quality/environment-scan pela UI:
   `CoreClient` guarda o `jobId` aceito por dominio e ganhou
   `cancelBuild()/cancelTests()/cancelQuality()/cancelEnvironmentScan()`
   chamando `job.cancel`; `Main.qml` ganhou um "×" na status bar ao lado de
   cada indicador (compilando/testando/analisando/scan de ambiente).
3. Itens de backend ainda pendentes no roadmap (Process Runner unificado,
   CMake/Cargo service, Project Health, Settings/Storage, Risk Engine, Run
   Configs, Git, AI Bridge) ficam **explicitamente adiados**: nenhum deles
   bloqueia o cancelamento de jobs nem a proxima fatia de UI. Retomar um deles
   so quando uma fatia de UI concreta precisar dele.
4. **TAREFA CONCLUIDA EM 2026-07-06:** quebrar `Main.qml` (~4600 linhas
   originalmente) em componentes por dominio, controllers/stores nao visuais e
   roteadores IPC por dominio. Historico da sequencia abaixo; o estado atual e
   o bloco "Main deixou de ser god file/god controller".

   **Atualizacao CODEX 2026-07-06:** primeira fatia executada e validada.
   `Main.qml` caiu primeiro para ~4130 linhas. Foram extraidos e registrados em
   `ui/CMakeLists.txt`: `BottomTabBar.qml`, `BuildPanel.qml`,
   `TestsPanel.qml`, `ProblemsPanel.qml`, `IdeLogPanel.qml`,
   `ToolsPanel.qml` e `WorkspaceStatusBar.qml`. `Main.qml` continua dono dos
   `ListModel`s e dos handlers `Connections { target: coreClient }`; os
   componentes novos recebem dados por `property` e devolvem acoes por
   `signal` quando necessario (`ProblemsPanel.openRequested`,
   `WorkspaceStatusBar.cancel*Requested`, etc.). Tambem foi corrigido um lint
   C++ pequeno em `ui/src/editor_highlighter.cpp` (`Rule` com designated
   initializer). Verificacao feita: `cmake --build --preset dev-local`,
   `cmake --build --preset dev-local-release`, smoke offscreen release
   (encerrou por `timeout`, so mostrou warning antigo de `Shortcut`) e
   `scripts/verificar.sh --rapido` verde.

   **Atualizacao CODEX 2026-07-06 (segunda fatia):** tambem foram extraidos
   `TerminalPanel.qml`, `RunPanel.qml` e `SearchPanel.qml`; `Main.qml` caiu
   para ~3810 linhas. O terminal agora envia input via `coreClient.terminalInput`
   (antes o QML chamava `runInput`, metodo inexistente no `CoreClient`).
   Verificacao repetida: `cmake --build --preset dev-local`,
   `cmake --build --preset dev-local-release`, smoke offscreen release
   (mesmo warning antigo de `Shortcut`) e `scripts/verificar.sh --rapido`
   verde.

   **Atualizacao CODEX 2026-07-06 (arquitetura QML por responsabilidade):**
   os componentes QML agora ficam em pastas por dominio, nao soltos em
   `ui/qml/`. `Main.qml` caiu para **2745 linhas** e continua sendo o
   orquestrador de estado/modelos/IPC. Estrutura atual:
   `shell/` (`BottomTabBar`, `WorkspaceStatusBar`), `panels/bottom/`
   (`BuildPanel`, `TestsPanel`, `ProblemsPanel`, `TerminalPanel`,
   `RunPanel`, `SearchPanel`, `IdeLogPanel`, `ToolsPanel`), `workspace/`
   (`FolderPickerDialog`), `assistant/` (`AssistantPanel`), `project/`
   (`ProjectCreateDialog`, `ProjectEntryContextMenu`,
   `ProjectEntryRenameDialog`, `ProjectEntryDeleteDialog`), `command/`
   (`SearchEverywhereDialog`) e `editor/` (`SymbolRenameDialog`,
   `EditorTabsBar`, `EditorTextSurface`, `EditorHoverPopup`,
   `EditorCompletionPopup`, `EditorUsagesPopup`).
   `ui/CMakeLists.txt` usa `QT_RESOURCE_ALIAS` para manter os nomes dos tipos
   QML estaveis apesar dos arquivos fisicamente organizados em subpastas.
   Componentes recebem dados por `property` e retornam acoes por `signal`; nao
   chamam ferramentas externas nem falam diretamente com filesystem/core, salvo
   quando o pai (`Main.qml`) ja encaminha via `CoreClient`.

   Verificacao desta terceira fatia: `cmake --build --preset dev-local`,
   smoke offscreen debug, `cmake --build --preset dev-local-release`, smoke
   offscreen release (ambos encerraram por `timeout`; unico aviso novo/visivel
   segue sendo o warning antigo de `Shortcut`) e
   `scripts/verificar.sh --rapido` verde. Durante a extracao foi preservado o
   padrao arquitetural: `Main.qml` e dono dos `ListModel`s e dos handlers
   `Connections { target: coreClient }`; componentes novos so recebem modelos,
   texto, flags e emitem sinais.

   **Atualizacao CODEX 2026-07-06 (warning de Shortcut resolvido):** o aviso
   runtime `QML Shortcut: Only binding to one of multiple key bindings...`
   vinha de `Shortcut { sequence: StandardKey.Save }`. Corrigido sem camada
   nova, usando `sequences: [StandardKey.Save]`. Smoke offscreen debug e
   release ficaram sem saida QML; `scripts/verificar.sh` completo terminou
   verde e atualizou os binarios release usados pelo icone.

   **Atualizacao CODEX 2026-07-06 (Main deixou de ser god file/god
   controller):** `Main.qml` foi reduzido para **336 linhas** e virou
   composition root. Ele nao possui mais `ListModel`, nao possui
   `Connections { target: coreClient }`, `Shortcut`, `Timer`, helper de
   dominio ou componente visual pesado embutido, e nao concentra os
   estados/modelos de editor, project tree, jobs, busca, run/terminal,
   assistente ou workspace.
   Estado e orquestracao foram movidos para controllers/stores nao visuais:
   `editor/EditorController.qml`, `project/ProjectTreeController.qml`,
   `jobs/JobsController.qml`, `runtime/RuntimeController.qml`,
   `search/SearchController.qml`, `workspace/WorkspaceController.qml`,
   `assistant/AssistantController.qml` e `command/CommandDispatcher.qml`.
   Eventos do `CoreClient` foram separados em roteadores IPC por dominio:
   `ipc/WorkspaceEventRouter.qml`, `ipc/EditorEventRouter.qml`,
   `ipc/JobsEventRouter.qml`, `ipc/SearchEventRouter.qml` e
   `ipc/RuntimeEventRouter.qml`. A superficie visual central foi consolidada em
   `shell/ShellWorkspaceHost.qml` (sem acesso direto a `CoreClient`), o editor
   foi quebrado em subcontrollers e a implementacao C++ do `CoreClient` foi
   fatiada por responsabilidade interna.
   Validacao desta etapa: `cmake --build --preset dev-local`, smoke offscreen
   debug, `cmake --build --preset dev-local-release`, smoke offscreen release,
   smoke offscreen pelo launcher `scripts/kinein-vectis` e
   `scripts/verificar.sh --rapido` verde.

   **Atualizacao CODEX 2026-07-06 (ProjectExplorer extraido):** a arvore de
   projeto saiu de `Main.qml` e virou `ui/qml/project/ProjectExplorer.qml`,
   registrada em `ui/CMakeLists.txt` com `QT_RESOURCE_ALIAS`. `Main.qml` caiu
   para **2533 linhas**; `ProjectExplorer.qml` ficou com **253 linhas**.
   `treeModel`, selecao e IPC continuam no pai. O componente so recebe
   `workspaceName`, `workspaceKindLabel`, `selectedPath` e `entriesModel`, e
   emite sinais para criar arquivo/pasta, atualizar, fechar workspace,
   selecionar entrada, expandir/recolher diretorio, abrir arquivo e pedir menu
   de contexto. Validado com `cmake --build --preset dev-local`, smoke
   offscreen debug, `cmake --build --preset dev-local-release`, smoke offscreen
   release e `scripts/verificar.sh --rapido` verde.

   **Atualizacao CODEX 2026-07-06 (editor visual extraido):** a barra de abas
   e a superficie visual do editor sairam de `Main.qml` e viraram
   `ui/qml/editor/EditorTabsBar.qml` e
   `ui/qml/editor/EditorTextSurface.qml`, ambas registradas em
   `ui/CMakeLists.txt` com `QT_RESOURCE_ALIAS`. `Main.qml` caiu para
   **2340 linhas**; `EditorTabsBar.qml` ficou com **110 linhas** e
   `EditorTextSurface.qml` com **176 linhas**. `Main.qml` continua dono de
   `openFiles`, `currentTab`, `loadingEditorText`, dirty state, timers,
   requests LSP, save/read/write e IPC. `EditorTabsBar` so recebe modelo/indice
   e emite selecao, fechamento e salvar. `EditorTextSurface` encapsula
   `Flickable + TextEdit + EditorHighlighter`, expoe texto/cursor/selecao e
   emite sinais para texto editado, completion, hover/usages, indent/unindent
   e newline. Validado com `cmake --build --preset dev-local`, smoke offscreen
   debug, `cmake --build --preset dev-local-release`, smoke offscreen release e
   `scripts/verificar.sh --rapido` verde.

   **Atualizacao CODEX 2026-07-06 (Main deixou de ser god file/god
   controller):** a estrategia aprovada foi executada. O shell visual saiu
   para `ui/qml/shell/TopHeaderBar.qml`, `SideRail.qml` e `ShellLayout.qml`.
   O estado/orquestracao saiu para controllers/stores QML nao visuais:
   `workspace/WorkspaceController.qml`, `editor/EditorController.qml`,
   `project/ProjectTreeController.qml`, `jobs/JobsController.qml`,
   `runtime/RuntimeController.qml`, `search/SearchController.qml`,
   `assistant/AssistantController.qml` e `command/CommandDispatcher.qml`.
   Os handlers de `CoreClient` foram divididos em roteadores IPC por dominio:
   `ipc/WorkspaceEventRouter.qml`, `EditorEventRouter.qml`,
   `JobsEventRouter.qml`, `SearchEventRouter.qml` e
   `RuntimeEventRouter.qml`.

   `Main.qml` caiu para **336 linhas**. Ele nao tem mais `ListModel`, nao tem
   mais bloco `Connections { target: coreClient }`, `Shortcut`, `Timer`, helper
   de dominio ou componente visual pesado embutido; tambem nao guarda
   `openFiles`, `treeModel`, modelos de build/test/problems/search/run, timers
   de LSP, estado de terminal, command palette ou mensagens do assistente. O
   arquivo ficou como composition root: cria a janela, instancia `CoreClient`,
   controllers, roteadores e hosts de shell, e conecta sinais de alto nivel.

   **Decisao do usuario em 2026-07-06:** nao aceitar sobras pequenas como
   divida tecnica "adiavel". A fase de higiene arquitetural sem divida nova foi
   executada e ficou documentada em `docs-legada/17-architecture-hygiene-plan.md`.
   Daqui para frente, novas features grandes so devem entrar mantendo os
   guardrails: `Main.qml` composition root, controllers/stores por dominio,
   roteadores IPC por dominio e `CoreClient` como fachada QML unica com
   implementacao interna fatiada quando crescer.

   **Onde olhar primeiro agora:**
   - `ui/qml/Main.qml`: composition root; deve permanecer pequeno e nao voltar
     a concentrar modelos/handlers de dominio.
   - `ui/qml/editor/EditorController.qml`: fachada do editor; documentos,
     texto e completion ficam nos subcontrollers do mesmo diretorio.
   - `ui/qml/project/ProjectTreeController.qml`: `treeModel`,
     expand/collapse, selecao, create/rename/delete e sincronizacao com abas.
   - `ui/qml/jobs/JobsController.qml`: saida de build, problemas
     build/quality/LSP, testes, summary e historico generico de jobs.
   - `ui/qml/runtime/RuntimeController.qml`: terminal integrado, saida de run,
     stdin e start/stop de processo.
   - `ui/qml/search/SearchController.qml`: busca em arquivos e Search
     Everywhere.
   - `ui/qml/ipc/*.qml`: roteadores dos eventos `CoreClient` por dominio.

   **Padrao de extracao daqui para frente:** componente visual recebe dados por
   `property` e emite `signal`; controller/store guarda modelo, estado, timers
   e pequenas decisoes de UI; roteador IPC recebe evento do `CoreClient` e chama
   o controller certo. Nenhum componente visual deve acessar filesystem ou
   chamar ferramenta externa. Todo novo `.qml` em subpasta precisa entrar em
   `ui/CMakeLists.txt` com `QT_RESOURCE_ALIAS`.

   **Verificacao apos cada aba extraida:** `cmake --build --preset
   dev-local` e `--preset dev-local-release` (ambos devem linkar),
   `scripts/verificar-qml.sh` (desde 2026-07-08 QML tem gate automatico:
   qmllint estrito com zero warnings), smoke offscreen
   (`QT_QPA_PLATFORM=offscreen`), e o comportamento visual idempotente
   (build/test/quality continuam iniciando, mostrando saida e finalizando
   via eventos; Problems continua recebendo as tres origens).

   **Specs de UI/UX sao inegociaveis mesmo neste refactor estrutural.** Esta
   tarefa e reorganizacao de arquivo (extrair QML para componentes), NAO
   redesign — mas qualquer decisao de layout, espacamento, cor, icone,
   copy ou comportamento visual que aparecer no caminho tem que seguir
   estritamente os `.md` de `docs/specs/`, nunca inventar por conta propria
   nem "aproveitar para melhorar visualmente". Specs relevantes para as abas
   desta tarefa: `docs/specs/KINEIN_VECTIS_LAYOUT_SYSTEM.md`,
   `docs/specs/KINEIN_VECTIS_UI_COMPONENTS_SYSTEM.md`,
   `docs/specs/KINEIN_VECTIS_VISUAL_SYSTEM_ICONS.md` e
   `docs/specs/KINEIN_VECTIS_PRODUCT_FLOWS_BUILD_RUN_DEBUG.md` (indice geral:
   `docs/specs/KINEIN_VECTIS_SPEC_INDEX.md`). Se o comportamento atual de
   `Main.qml` divergir do spec, extrair mantendo o comportamento atual (bug
   de UX vira tarefa separada, com o usuario ciente) — nao corrigir de
   passagem escondido dentro do refactor estrutural.

   **Nao fazer:** nao mudar o layout/visual escondido dentro de refactor
   estrutural; nao recolocar `ListModel`, timers, `Connections` ou switchs de
   dominio em `Main.qml`; nao criar acesso direto a filesystem/ferramentas em
   componente visual. Modelos de dominio agora vivem nos controllers
   (`JobsController`, `EditorController`, `ProjectTreeController`,
   `RuntimeController`, `SearchController`, etc.) e eventos IPC vivem nos
   roteadores `ui/qml/ipc/`.

   **Atualizacao CODEX 2026-07-06 (higiene arquitetural finalizada):** a fase
   sem divida nova foi executada e documentada em
   `docs-legada/17-architecture-hygiene-plan.md`. Estado final validado:
   `Main.qml` ficou com **336 linhas**, sem `ListModel`, `Connections`,
   `Shortcut`, `Timer`, helper de dominio ou componente visual pesado
   embutido. O host visual central do workspace virou
   `ui/qml/shell/ShellWorkspaceHost.qml` (**248 linhas**, sem acesso direto a
   `CoreClient`). O `EditorController.qml` ficou com **318 linhas** e delega
   documentos, texto
   e completion para `EditorDocumentController.qml`,
   `EditorTextController.qml` e `EditorCompletionController.qml`. O
   `CoreClient` segue sendo a fachada QML unica, mas sua implementacao C++ foi
   dividida em `core_client_process.cpp`, `core_client_requests.cpp`,
   `core_client_dispatch.cpp`, `core_client_state.cpp` e
   `core_client_log.cpp`. `ui/CMakeLists.txt` foi sincronizado com os novos
   QML/C++ e tambem declara o prefixo QML usado pelo runtime, alem de tratar a
   politica Qt de `qmldir` extra quando disponivel. Validacao final:
   `cmake --build --preset dev-local`, smoke offscreen debug,
   `cmake --build --preset dev-local-release`, smoke offscreen release, smoke
   offscreen via `scripts/kinein-vectis` e `scripts/verificar.sh --rapido`
   verde.

   **Atualizacao CODEX 2026-07-06 (aba Jobs generica):** a UI passou a consumir
   `event.job.created/progress/output/finished` em uma aba dedicada "Jobs" no
   painel inferior, sem mudanca de contrato IPC nem backend novo. O
   `JobsController.qml` agora mantem `jobsModel`, `JobsEventRouter.qml`
   encaminha os sinais genericos de job, e o componente visual novo
   `ui/qml/panels/bottom/JobsPanel.qml` mostra titulo, status, risco nao baixo
   e ultima linha/progresso textual do job. `ui/CMakeLists.txt` foi atualizado
   com `QT_RESOURCE_ALIAS` para o novo QML. Validacao da fatia: build debug,
   smoke offscreen debug, build release, smoke offscreen release e smoke pelo
   launcher local verdes; `scripts/verificar.sh --rapido` tambem verde. O
   ruido restante segue restrito a qmlcache gerado pelo Qt/toolchain local.

   **Regra daqui para frente:** a fase nao deixa divida tecnica conhecida
   nessa frente. Nova feature deve manter o fluxo `QML visual ->
   controller/store -> CoreClient facade -> handler IPC interno -> Rust core`.
   Se algum arquivo passar dos limites de `docs-legada/17-architecture-hygiene-plan.md`
   ou misturar renderizacao, estado e IPC, a feature so esta pronta depois do
   split.

   **Proxima sequencia recomendada apos este checkpoint/commit (2026-07-06;
   status revisado 2026-07-08):**
   1. [feito 2026-07-08] Revalidar o ambiente local apos a troca de distro:
      build release reconfigurado do zero (cache stale removido), gate
      completo `scripts/verificar.sh` verde e smokes offscreen debug/release/
      launcher ok. Detalhe das falhas e correcoes: secao "Ambiente revalidado"
      no topo deste arquivo.
   2. [feito 2026-07-08, primeira etapa] **Project Health minimo e visivel**
      com dados ja existentes — banner discreto documentado em "Estado tecnico
      atual". Nenhum contrato novo foi necessario nesta etapa.
   3. Se `project.health` virar necessario (sinais que o core ainda nao expoe:
      CMake sem configure, compile_commands ausente, cargo metadata quebrado,
      LSP degradado, build dir stale), seguir o fluxo de
      `docs/arquitetura/ARCHITECTURE.md`: tipos em `kinein-protocol`, handler fino,
      servico de dominio no core, testes, docs/arquitetura/03 atualizado e UI por
      controller/roteador/componente visual. Operacao longa deve ser job.
   4. [em andamento] Seguir `docs-privada/diario/18-daily-driver-plan.md` (criado 2026-07-08
      a pedido do usuario: rigor maior + virar daily driver o quanto antes).
      Marco corrente: **M1 — edicao diaria confortavel**.
      Fatia M1.1 (formatacao orquestrada) FEITA em 2026-07-09.
      Fatia M1.2 (semantic tokens completos) FEITA em 2026-07-09 — o
      diagnostico por sondas reais mostrou que o pipeline ja funcionava
      (inclusive limpeza de spans na troca de aba, ao contrario da suspeita
      inicial); o gap real era so o mapa de kinds do EditorHighlighter, que
      descartava lifetime, selfKeyword, typeAlias, const, boolean, character,
      generic, builtinType, etc. Design/decisoes: docs-privada/diario/18, "Fatia M1.2".
      Fatia M1.3 (code actions / quick fixes) FEITA em 2026-07-09:
      Alt+Enter abre popup de acoes no cursor (lsp.codeActions com context de
      diagnostics cacheado no core; so acoes com edit inline), Enter/clique
      aplica via lsp.applyCodeAction reusando o caminho de aplicacao do
      rename. Validada ponta-a-ponta com clangd real (fix-it de ';' aplicado
      em disco via stdio). Bonus da fatia: didChange/didOpen agora pulam
      sync quando o conteudo nao mudou (hash por documento) — corrige a
      invalidacao dos fix-its do clangd e corta didChange redundante de todo
      hover/completion. Design: docs-privada/diario/18, "Fatia M1.3".
      Fatia M1.4 (go-to-symbol) FEITA em 2026-07-09: no Search Everywhere,
      "@" lista/filtra simbolos do arquivo atual (documentSymbol, achatado
      com containers) e "#nome" busca simbolos do workspace
      (workspace/symbol no servidor do arquivo ativo). Aceitar salta para
      linha/coluna. Validada e2e com rust-analyzer real. Design: docs-privada/diario/18,
      "Fatia M1.4".
      Fatia M1.5 (sessao por workspace) FEITA em 2026-07-09: reabrir o mesmo
      root restaura abas e aba ativa (.kinein/session.json, schemaVersion 1,
      caminhos relativos em disco/absolutos no IPC, arquivos mortos
      filtrados); a UI salva com debounce de 1.2s no EditorController e
      restaura pedindo a aba ativa por ultimo. Validada e2e com dois
      processos do core. Design: docs-privada/diario/18, "Fatia M1.5".
      Fatia M1.6 (ergonomia de editor) FEITA em 2026-07-09 — fatia 100% UI,
      sem mudanca de protocolo: Ctrl+D duplica linha/selecao, Alt+Shift+
      cima/baixo move bloco de linhas, Ctrl+/ comenta/descomenta (token vem
      da linguagem do highlighter; json/plain e no-op), Ctrl+Y deleta linha,
      Ctrl+G abre dialogo "linha[:coluna]" pre-preenchido. Operacoes no
      EditorTextController via remove/insert (undo nativo preservado).
      Design: docs-privada/diario/18, "Fatia M1.6".
      **M1 FECHADO em 2026-07-09**: as 6 fatias funcionais + a C0 (auditoria
      formal de convergencia visual — resultado completo com referencias de
      spec em docs/roadmaps/20, areas A-H). Existe um **MANUAL.md** na raiz para
      usuarios/testers (uso, funções, atalhos e troubleshooting dentro da
      IDE) — manter atualizado a cada fatia que mudar UX. Distribuição,
      instalação e geração do executável ficam em `Tutorial.md`.
      **M2 estruturado em docs-privada/diario/18** (ordem: M2.1 terminal unificado ->
      M2.2 CMake -> M2.3 Cargo -> M2.4 run configs -> M2.5 debugger DAP).
      DECISAO DO USUARIO (2026-07-09): as abas "Executar" e "Terminal"
      viram UMA aba Terminal (backends continuam separados; superficie
      visual unica com sessoes) — e o requisito da fatia M2.1.
      Fatia C1 (fundacao visual) FEITA em 2026-07-09, pendente de validacao
      visual do usuario (R7 de docs/roadmaps/20): Theme.qml agora tem OS VALORES DA
      SPEC (paleta inteira, escalas 4px/raios, tokens tipograficos novos),
      fundo de selecao virou surfaceSelected #222833 (ambar so como acento,
      COMP §10.4), dimensoes das regioes corrigidas (rail 52, status 28,
      bottom tabs 34, tab bar 36, explorer 280, bottom 260, context 360),
      editor 13->14px (a auditoria tinha medido o texto errado) e os tres
      paineis sao redimensionaveis via shell/PanelSplitter.qml (limites da
      spec no ShellController; persistencia de layout fica para Settings).
      Detalhe completo: docs/roadmaps/20, "Execucao da C1".
      Fatia M2.1 (Terminal unificado) FEITA em 2026-07-09 — fatia 100% UI,
      contratos run.*/terminal.* intactos: a aba "Executar" saiu; a aba
      Terminal tem sessoes Shell|Execucao (estado terminalSession no
      RuntimeController), Shift+F10 abre direto na Execucao, Alt+F12 no
      Shell, chip da Execucao e rotulo da aba mostram "●" com processo
      vivo, botao "limpar" zera so a sessao ativa, foco automatico por
      sessao. A atual seção 5 do MANUAL documenta o fluxo. Design: docs-privada/diario/18,
      "Fatia M2.1".
      Fatia M2.2 (CMake service) FEITA em 2026-07-09, protocolo 0.25.0:
      cmake.configure como job (file-api query + CDB exportada, preset
      opcional sem mudar o build dir .kinein/build), cmake.presets.list/
      targets.list (codemodel-v2)/status; clangd novo recebe
      --compile-commands-dir quando a CDB existe (limite documentado:
      servidor vivo nao recarrega flags — reabrir arquivo/workspace).
      UI: Project Health ganha "CMake sem configure" com acao [Configurar]
      (progresso na aba Jobs) e o palette tem "CMake: Configure".
      Validada e2e com cmake real (presets/configure/status/targets/CDB).
      TAMBEM nesta sessao: inputs do Terminal/Busca subiram 24->28px
      (regressao C1 apontada pelo usuario: fonte 13px cortava no topo).
      Fatia M2.3 (Cargo service) FEITA em 2026-07-09, protocolo 0.26.0:
      cargo.metadata sincrono (resumo de pacotes/features/targets; medido
      ~10ms; auto-pedido ao abrir workspace rustCargo) e cargo.check como
      job REUSANDO o pipeline do quality (event.quality.* -> aba Problemas
      sem parser novo; aliasing documentado ate o Problems 2.0). Project
      Health ganhou o sinal "cargo metadata falhou" com acao de retry;
      palette tem "Cargo: Check" e "Cargo: Metadata". Validada e2e com
      cargo real. TAMBEM: inputs de Terminal/Execucao/Busca corrigidos de
      vez (30px, margens verticais zeradas — report do usuario sobre "l"
      cortado; a busca tambem foi para fonte 13).
      Fatias M2.4 + C3 FEITAS em 2026-07-09, protocolo 0.27.0:
      runConfig.list/save/delete/setActive persistidos em
      .kinein/runconfigs.json (schemaVersion 1; toda mutacao responde
      lista+ativa; salvar torna ativa); run.start {} resolve comando >
      config ativa > heuristica. C3: o TopHeaderBar VIROU a Main Toolbar
      da spec (44px, botoes 32px, ordem [Run Config][Configurar so
      cmake][Build][Testes][Analise][>]) — interpretacao registrada em
      docs-privada/diario/18: regiao 1 (Title/App Bar) so nasce na C5; Target/Profile
      selectors e botao Debug entram com as fatias que os alimentam.
      Seletor com dropdown (Automatico|configs|Nova/Editar/Excluir) e
      RunConfigDialog no ShellOverlays. Validada e2e (config persiste
      entre processos e dirige o run.start).
      Fatia M2.5a (core DAP) FEITA em 2026-07-09, protocolo 0.28.0:
      dominio dap/ orquestra lldb-dap (sessao unica por workspace,
      breakpoints por arquivo guardados no core e replayados no launch,
      continue/next/stepIn/stepOut/pause/stop, eventos event.debug.*
      mastigados — stopped ja vem com file/line via stackTrace interno).
      Alvo "Automatico" espelha o run (unico binario de target/debug ou
      .kinein/build); debug.start NAO compila antes. Framing DAP reusa
      lsp/framing (mesmo envelope Content-Length). Validada por sonda e2e
      com lldb-dap real em C e Rust (breakpoint/step/continue/exitCode/
      saida provados). lldb-dap entrou no inventario de tools e na
      verificacao do instalar-ambiente.sh.
      Fatia M2.5b (UI de debug + parte gutter da C4) FEITA em 2026-07-09:
      a gutter NASCEU no EditorTextSurface (numeros de linha em janela
      visivel, breakpoint por clique, linha de execucao destacada);
      DebugController (estado da sessao + breakpointsByFile/Revision) +
      DebugEventRouter + DebugPanel (aba Debug: controles + saida) +
      botao Debug na toolbar (vira "■ Debug") + atalhos Shift+F9/F9/F8/
      F7/Shift+F8 + comando "Debug" no Search Everywhere; CoreClient com
      propriedade debugging, 8 invokables debug* e sinais de evento.
      A atual seção 4.1 do MANUAL.md ensina o fluxo. Falta validacao visual do
      usuario (R7).
      Fatia M2.5c (Inspecao) FEITA em 2026-07-09, protocolo 0.29.0 —
      **M2 FECHADO**: debug.stackTrace (20 frames) e debug.variables
      { frameId | ref } (core resolve scopes DAP internamente; resposta
      ecoa a chave para correlacao na UI). Aba Debug pausada vira
      [saida | Frames | Variaveis]: clique no frame navega e carrega
      variaveis; struct expande/colapsa in-place (modelo flat + depth).
      C4 somou linha do cursor e breadcrumbs de caminho; gutter de
      diagnosticos adiado (lspDiagnostics nao e consumido por nenhum QML
      — precisa de store por arquivo, entra com Problems 2.0/C6).
      Pendencias deliberadas do debugger (gatilho: uso real): threads
      view, Globals/Registers, watch/evaluate, breakpoints condicionais,
      build-antes-do-debug.
      M2 validado pelo usuario em 2026-07-09 ("ja testei e esta
      funcional por enquanto") — bugs futuros serao reportados no uso.
      Marco **M3 — Git MVP FECHADO em 2026-07-10** (M3.1 status
      read-only → M3.2 diff/gutter → M3.3 stage/commit → M3.4 blame/log).
      Regra do marco: orquestrar o binario git com saida estavel
      (--porcelain=v2 -z), core stateless, UI dirige refresh. Proximo
      foco: T6 (diagnostics na gutter) e depois M4.1 (Settings) — ver
      ordem no docs/roadmaps/21.
      Fatia M3.1 FEITA em 2026-07-09, protocolo 0.30.0: git.status
      (deteccao por exit code, --untracked-files=all, paths relativos ao
      root com filtro de fora-do-workspace e de .kinein/); branch/ahead/
      behind/contador na status bar; arvore colore por gitKinds
      (successSoft/infoSoft/textDisabled/errorSoft — sem tokens novos);
      refresh dirigido pela UI (open automatico no C++, save/fs-ops no
      GitEventRouter, manual via "Git: Atualizar status"). Sonda e2e com
      git real provou G1/G2/G3 (entries, subdir, nao-repo).
      Fatia M3.2 FEITA em 2026-07-09, protocolo 0.31.0: git.fileDiff
      (uma resposta, duas granularidades: hunks de --unified=0 p/ gutter
      + texto U3 p/ visao de diff; untracked = tracked:false + hunk
      added inteiro; echo de path p/ stale-drop). Gutter com barra 3px
      (added/modified/removed) atualizada em save/troca de aba/refresh;
      GitDiffDialog via "Git: Diff do arquivo". Diff e do arquivo EM
      DISCO (buffer nao salvo nao aparece — attach de buffer e melhoria
      futura com gatilho). Sonda e2e G4/G5 provou hunks e untracked.
      Trilha E (fluxo de digitacao profissional, pedido do usuario)
      estruturada em docs-privada/diario/18: E1 auto-close de pares → E2 Enter
      inteligente → E3 polimento; executar ENTRE fatias do M3.
      Fatia E1 FEITA em 2026-07-09 (junto da M3.2): auto-close de
      ( [ { " ' com type-over, surround da selecao, backspace de par
      vazio e regras anti-duplicacao de aspas — tudo em
      EditorTextSurface.handleTypingKey (sem IPC). Falta validacao de
      digitacao real do usuario (criterio: lado a lado com VS Code).
      Fatia E2 FEITA em 2026-07-10: Enter entre `{}` abre a linha interna
      indentada e move o fechador para a linha propria; comentarios `//`
      e blocos `/* */` continuam com `// ` / `* `. Implementacao 100% UI
      em EditorTextController.insertNewline, sem parser/engine propria;
      sonda Qt Quick cobriu os cinco cenarios e a indentacao simples.
      Fatia E3 FEITA em 2026-07-10 — TRILHA E COMPLETA (pendente do
      teste de digitacao real do usuario, criterio lado a lado com VS
      Code): `}` digitado em linha vazia alinha com o `{` casado;
      Ctrl+W/Ctrl+Shift+W expandem/encolhem selecao por escada
      heuristica (palavra → parens → linha → bloco → documento, sem
      AST); Home/Shift+Home alternam primeiro-texto ↔ coluna 0.
      Logica no EditorTextController (sonda Qt Quick, 35 casos verdes),
      Surface so emite sinais. Descobertas (read-back do select, degrau
      de span de linhas) e design: docs-privada/diario/18, "Fatia E3".
      Fatia M3.3 FEITA em 2026-07-09, protocolo 0.32.0: git.stage/
      unstage/discard/commit, todas respondendo o shape do git.status
      (um caminho so de atualizacao na UI); commit staged-only com
      guarda por exit code (diff --cached --quiet); discard destrutivo
      com split tracked(restore)/untracked(clean) e CONFIRMACAO na UI
      (GitDiscardDialog). Aba Git no painel inferior: stage por clique,
      chips diff/descartar, mensagem + "Commit (N)". Sonda e2e G6-G10.
      Radar registrado (pedido do usuario): "pequenas coisas" acumuladas
      = fluidez (principio na trilha E) e fatia futura "auto-setup ao
      abrir projeto pronto" (cmake.configure automatico no open sem
      .kinein/build; reconfigure ao salvar CMakeLists; docs-privada/diario/18).
      Roadmap de longo horizonte M4-M7 escrito em 2026-07-09 em
      **docs/roadmaps/21-long-horizon-roadmap.md** (pedido do usuario: qualidade
      de handoff para sessoes futuras sem prompt profissional; inclui o
      playbook de continuidade com o ritual por fatia e as convencoes
      aprendidas — LEIA O 21 ANTES DE FATIAS M4+).
      Fatia auto-setup ao abrir v1 FEITA em 2026-07-09 (so UI): projeto
      CMake nao configurado dispara cmake.configure sozinho no open
      (uma vez por workspace, anti-loop; falha devolve o banner
      acionavel) e salvar CMakeLists/CMakePresets reconfigura
      automatico. Re-attach do clangd pos-configure segue manual
      (reabrir arquivo; gatilho registrado).
      Fatia M3.4 FEITA em 2026-07-10, protocolo 0.33.0 — **M3 FECHADO**:
      git.blame (--porcelain, grupos por linha com autor/idade/sha;
      linha nao commitada = sha zerado; untracked/sem-HEAD =
      tracked:false vazio), git.log (formato %x1f/NUL estavel,
      maxCount 1..=500, repo vazio = entries []) e git.commitDiff (sha
      validado 4..=64 hex antes de virar argv). UI: blame como coluna
      da gutter (toggle "Git: Blame do arquivo", segue arquivo ativo,
      idade relativa formatada na UI); aba Git ganhou vistas
      [Mudancas|Historico] (chips), commit clicado reusa o
      GitDiffDialog. Bug de passagem: closeDiffDialog nao limpa mais a
      lista (isso e do clear/troca de workspace). Sonda e2e G11-G18 com
      git real. Design: docs-privada/diario/18, "Fatia M3.4".
      Fatia T1 FEITA em 2026-07-10, protocolo 0.34.0 (primeira da
      trilha T, docs/roadmaps/21): lsp.switchSourceHeader { path, content } ->
      { path? } via a extensao do clangd; gate de linguagem no core
      (arquivo nao-C/C++ -> INVALID_PARAMS, sem deixar rust-analyzer
      responder cru); Alt+O + comando "C/C++: Alternar header/source".
      Reusou FsWriteParams, path_for_uri e openDiagnostic (zero codigo
      novo de abertura). Sonda e2e com clangd real (header<->source +
      sem par). RADAR aberto: falta primitiva de "aviso discreto"
      (toast) na UI — switch sem par so nao navega hoje. Design:
      docs-privada/diario/18, "Fatia T1".
      **M3 validado tecnicamente (gate verde + sondas e2e); falta o
      teste de USO real do usuario (blame/historico/switch e digitacao
      da trilha E lado a lado com VS Code/CLion).**
      Fatia T6 FEITA em 2026-07-11, protocolo 0.35.0 — **fecha a C4**
      (gutter de diagnosticos) e a maior lacuna diaria de editor da
      trilha T: diagnosticos LSP EM TEMPO REAL (o pipeline didChange
      600ms -> publishDiagnostics -> event.lsp.* -> aba Problemas JA
      existia; a fatia somou o resto) agora tambem SUBLINHADOS no editor
      (ondulado por severidade no EditorHighlighter, merge por
      caractere), com MARCA na gutter + tooltip da mensagem e navegacao
      F2/Shift+F2 (+Ctrl+Alt+E). O Diagnostic ganhou endLine/endColumn/
      code; o novo DiagnosticsController e o store por arquivo (consome
      o mesmo lspDiagnostics). Aba Problemas mostra o code por linha.
      Validada e2e com clangd E rust-analyzer reais. Design/descobertas
      (setFormat substitui formato; reatividade via revision; tooltip
      proprio sem QtQuick.Controls): docs-privada/diario/18, "Fatia T6". Pendente:
      validacao visual do usuario (R7) do squiggle/gutter.
      REGRA DO USUARIO reforcada nesta fatia: a experiencia de
      diagnostico deve ser JetBrains — erro na hora no codigo E na aba
      Problemas com detalhe (code/regra + mensagem). Ja atendido para
      LSP; sublinhado de build/quality (compilador/clippy) precisa de
      range no parser deles e entra com o Problems 2.0.
      Fatia CR1 (conforto de leitura/digitacao) FEITA em 2026-07-11
      (100% UI, feedback do usuario pos-T6): marca de diagnostico na
      gutter mais visivel (dot 8px com borda + numero da linha tingido
      pela severidade); realce da stdlib C/C++ (streams cout/cin/cerr/
      clog, print/mem/str, containers/tipos) e Rust (tipos primitivos/
      std + macros \w+!) por regex, com override POS-semantic para
      cout/cin/cerr/clog manterem destaque (o clangd os marca
      "variable"); #include <> auto-close contextual. Reusa a paleta de
      docs/05 (sem cor nova); variaveis seguem quase-brancas. Confirmado
      por leitura + sonda que AUTOCOMPLETE LSP (C/C++ e Rust) JA existe
      e funciona (dispara sozinho, Tab/Enter aceita) e clangd/rust-
      analyzer JA estao prontos. Design/descobertas: docs-privada/diario/18, "Fatia
      CR1". Pende validacao visual/digitacao do usuario.
      RADAR (pedido do usuario, registrado): a "inteligencia" de
      cout << e de sugerir bibliotecas apos #include < deve vir do
      SNIPPET de completion do LSP (que ja existe), NAO de heuristica de
      par (auto-inserir << erraria — "<" e comparacao/template/shift).
      Fatia M4.1 (Settings/Storage) FEITA em 2026-07-11, protocolo
      0.36.0 — **M4 iniciado**: dois niveis (global XDG
      ~/.config/kinein-vectis/settings.json + workspace
      .kinein/settings.json; workspace sobrepoe campo a campo; schema +
      invalido->vazio como runconfig). settings.get {} / settings.set
      { scope, values } respondem o estado completo (efetivo + global +
      workspace). 3 consumidores reais: editorFontSize ->
      Theme.fontSizeEditor (deixou de ser readonly), autoClosePairs ->
      gate do auto-close da E1, formatOnSave -> Ctrl+S formata-entao-
      salva (robusto: salva mesmo se o format falhar). UI:
      SettingsDialog + SettingsToggleRow proprios (sem QtQuick.Controls),
      atalho Ctrl+Alt+S, edita escopo GLOBAL no v1. diffBase (head|
      index) adiado (precisa de `base` no git.fileDiff). Validada e2e
      (sonda com XDG_CONFIG_HOME isolado: persistencia global/workspace
      entre processos). ARMADILHA: `cargo test`/gate NAO recompilam
      target/debug/kinein-core — rodar `cargo build -p kinein-core`
      antes de sondas e2e. Design: docs-privada/diario/18, "Fatia M4.1".
      **T4 (cargo check no save) — DESCOBERTA 2026-07-11: JA ENTREGUE
      pelo rust-analyzer.** Sonda provou: `fs.write` ja manda `did_save`
      ao LSP (fs.rs:243) e o rust-analyzer roda `cargo check` no save
      sozinho (flycheck, on por padrao) publicando via LSP ->
      event.lsp.diagnostics -> T6 (Problemas + sublinhado). Ou seja,
      rodar NOSSO proprio cargo.check no save seria trabalho duplicado.
      NAO implementar o T4 original. O que sobra de valor: setting para
      DESLIGAR o flycheck em projeto grande (nicho) e clang-tidy no save
      p/ C++ (T2, adiado).
      **Fix de AUTOCOMPLETE (2026-07-11, protocolo 0.37.0) — queixa do
      usuario ("std::cout <<" nao completa):** o core cortava completion
      em 50 e a UI filtrava o CACHE ao digitar em vez de repedir ao
      servidor, entao itens fora dos primeiros N (ex.: `cout`) sumiam.
      Corrigido: `lsp.completion` devolve `isIncomplete`; teto 50->100;
      o EditorCompletionController REPEDE ao servidor (debounce) quando
      a lista e incompleta, em vez de so filtrar o cache. Sonda provou:
      `std::c` -> isIncomplete=true; `std::cou` -> cout presente.
      Lembrete: C++ so completa a stdlib com compile_commands.json
      (rodar "CMake: Configure").
      **M4.3 parte A (recuperacao de crash do core) FEITA em 2026-07-11**
      (so C++ do CoreClient, sem contrato novo): crash do core ->
      handleFinished relanca + reabre o ultimo root; reabrir MESMO root
      nao limpa a UI (abas preservadas); na recuperacao PULA o session
      restore (nao sobrescreve edicoes); anti-loop de fork (>=3 quedas
      em 4s pausa); emit recovered() -> UI re-sincroniza o LSP do arquivo
      ativo. Validado com kill -9 real (reconecta com tabs=1). Falta
      parte B (lsp.restart + auto-restart) e C (jobs orfaos) = fatia
      M4.3b. Design: docs-privada/diario/18 "Fatia M4.3".
      **M4.5 (perfis de rigor Strict/Balanced/Relaxed) FEITA em
      2026-07-11, protocolo 0.38.0:** um setting (extensao da M4.1) que
      regula O QUE A IDE RODA NO PROJETO DO USUARIO (quality.run/build.run
      do botao) — NUNCA o gate do proprio repo Kinein (imutavel). Default
      Strict (identidade do produto). clippy: strict = pedantic+nursery
      +`-D warnings`; balanced = clippy default; relaxed = so
      `clippy::correctness`. cargo build: strict = `RUSTFLAGS=-D warnings`
      (invalida o cache ao trocar de perfil — aceito); os outros sem
      RUSTFLAGS. Flags sao FUNCOES PURAS testadas (clippy_profile_args/
      rust_build_rustflags); `settings::effective_rigor_profile(root)` e
      FUNCAO LIVRE (nao usa self). UI: seletor segmentado de 3 opcoes no
      SettingsDialog (escopo global no v1). C++ inalterado (settingsSet ja
      serializa o mapa inteiro). Validada e2e (sonda_m45.py: default
      strict -> global relaxed vira efetivo -> workspace vence -> persiste
      em disco). C++ CMake `-Werror` por perfil fica FORA (gatilho: pedido
      real; perfil so afeta Rust no v1). Design: docs-privada/diario/18 "Fatia M4.5".
      **M4.2 (orcamento de performance) FEITA em 2026-07-11, protocolo
      INALTERADO:** medicao 100% local (offscreen + stdio + /proc, ZERO
      telemetria/rede). `scripts/medir-performance.sh` (+ `medir-core.py`)
      mede mediana de N: (A) UI time-to-first-frame via marker env-gated
      `KINEIN_PERF_MARKER` novo na main.cpp (`installStartupPerfMarker`,
      frameSwapped SingleShot, `qInfo` no stderr — clang-tidy `*` proibe
      fprintf/vararg), (B) workspace.open no repo, (C) fs.read de .txt 10k
      linhas, (D) RSS UI/core/LSP. 1a medicao (Ryzen 7 7735HS): TTF
      offscreen ~101ms, UI RSS 88MB, workspace.open 1.5ms, fs.read 10k
      0.1ms, core RSS 5MB, rust-analyzer 677MB (ferramenta externa
      indexando o repo, nao memoria do Kinein). Orcamento (mediana+folga,
      regressao=bug) na tabela do docs/roadmaps/21 M4.2; latencia de digitacao
      ficou MANUAL (precisa injecao de tecla na GUI). Descoberta: fs.read
      de .rs ja dispara did_open -> LSP sobe sozinho. Design: docs-privada/diario/18
      "Fatia M4.2".
      **M4.3b (lsp.restart + jobs orfaos) FEITA em 2026-07-11, protocolo
      0.39.0 — FECHA o M4.3 e o MARCO M4.** Parte B: LspManager conta
      timeouts consecutivos por servidor (timeout_streak); 3 seguidos
      (~12s) auto-reiniciam aquele LSP; qualquer resposta zera. lsp.restart
      { language? } (sem language = todos) + comando de paleta "LSP:
      Reiniciar servidor"; event.lsp.restarted -> EditorEventRouter
      re-sincroniza o arquivo ativo (refreshSemanticTokens = mesmo caminho
      do recovered()). Parte C: JobManager.cancel_all() + Drop com drain
      limitado (500ms) mata cargo/cmake/lldb no shutdown (core.shutdown,
      EOF da UI morta, unwind) — sem orfaos. Sonda sonda_m43b.py provou:
      lsp.restart devolve restarted:[rust]; build.run + core.shutdown NAO
      deixa cargo orfao. Design: docs-privada/diario/18 "Fatia M4.3b".
      **Estado do M4: FECHADO. M4.1 (Settings), M4.2 (performance), M4.3
      A+B (robustez de crash/LSP/jobs) e M4.5 (perfis de rigor) FEITOS; T4
      fora da fila (flycheck ja entrega). M4.4 (First Run) adiado pelo
      proprio gatilho (>1 usuario) — nao bloqueia uso solo.**
      **REDE DE SEGURANCA (fatia S1, docs/seguranca/23) FEITA em 2026-07-11,
      protocolo 0.40.0 — a pedido do usuario, ANTES do dogfooding.** Dois
      pilares contra perda de dado: (1) `fs.write` agora e ATOMICO (temp no
      mesmo dir + fsync + rename; mata o vetor "arquivo zerado" em crash no
      meio da escrita); (2) autosave de rascunhos em SQLite (rusqlite
      bundled, modulo `db/`, `.kinein/kinein.db` WAL). draft.save/clear +
      `drafts` na resposta de workspace.open (filtrada por disco). fs.write
      limpa o rascunho; rascunho so sobrevive a CRASH -> volta como aba
      MODIFICADA no reabrir. UI: autosaveDebounce 1.5s, restoreDrafts com
      overlay. Sonda sonda_drafts.py provou: SIGKILL sem salvar -> recupera;
      save limpa; escrita atomica grava sem temp solto. Store reutilizavel
      para Local History e migracao de sessao (P4/radar). Design+status:
      docs/seguranca/23.
      **FASE POS-REDE-DE-SEGURANCA (ordem DEFINIDA pelo usuario em
      2026-07-11, ver docs/roadmaps/24 — status vivo):** D1 autocomplete LSP AO VIVO
      -> D2 terminal -> D3 tree-sitter/plugins/views -> D4 remake.
      **D1 RESOLVIDO (2026-07-12) — causa-raiz achada:** NAO era foco, nem
      prontidao do servidor, nem posicao do popup, nem filtro. Era armadilha
      de semantica do Qt Quick: `Item.visible` LE a visibilidade EFETIVA
      (explicitVisible && pai efetivamente visivel), nao o valor gravado. O
      EditorCompletionController vive dentro do EditorController, que e um
      `Item { visible: false }` (e CONTROLLER, nao UI) — entao o alias
      `completionVisible: completionController.visible` ficava preso em
      false para SEMPRE (e nem emitia visibleChanged). refilter() gravava
      visible=true, o modelo enchia com os itens certos, e o popup nunca
      abria. Isso explica tudo: backend OK (sonda), fiacao OK, fuzzy OK — os
      itens estavam la, o popup e que era incapaz de ficar visivel (por isso
      o fix do fuzzy "nao resolveu"). FIX: estado em property PROPRIA
      (`property bool popupVisible`), alias aponta pra ela. REGRA: controller
      QML e Item invisivel — NUNCA use o `visible` dele como estado de UI nem
      faca alias pra `.visible` (use property propria; hoverVisible/
      usagesVisible/actionsVisible ja faziam certo — completionVisible era o
      UNICO alias pra `.visible` do modulo e a UNICA feature quebrada).
      Provado headless (cena minima + harness qml6 carregando o controller
      REAL: pre-fix reproduz o bug, pos-fix verde) + gate + smoke. A
      instrumentacao temporaria (singleton DebugLog) foi REMOVIDA. Falta so
      o OK do usuario digitando ao vivo. LACUNA EXPOSTA: nao existe TESTE DE
      QML no projeto (gate cobre Rust/C++/qmllint, mas nada EXECUTA a logica
      QML) — por isso um bug de UI sobreviveu a 2 ciclos de "correcao";
      proposta: alvo Qt Quick Test no CMake plugado no verificar.sh.
      **D2.1+D2.2 FEITOS (2026-07-12, protocolo 0.42.0):** terminal
      profissional — PTY real (portable-pty) + emulador vt100 (grid) no
      core; UI = renderer de GRADE (cores 256/cursor) + teclado char-a-char
      + resize (D2.1); copiar/colar (singleton C++ Clipboard, seleção com
      mouse, Ctrl+Shift+C/V, clique-do-meio) + scrollback (roda do mouse ->
      terminal.scroll -> vt100 set_scrollback) (D2.2). Sonda_terminal.py:
      comando no grid, resize 100x30, scroll traz o inicio do historico.
      Usuario testou D2.1 ao vivo e APROVOU ("como se estivesse no terminal
      do computador").
      **D2.3 FEITA (2026-07-14, protocolo 0.44.0):** múltiplas abas de
      terminal. `terminal.open` cria id monotônico; input/resize/scroll/close
      e render/closed carregam id; `TerminalManager` usa `HashMap` (teto 12),
      e a UI preserva render/scrollback por aba com `+` e `×`. Fechar uma
      sessão não afeta as outras; crash/fechamento de workspace limpa ids
      antigos. A validação achou e corrigiu deadlock preexistente no close:
      waiter segurava mutex durante `wait()` e bloqueava `kill()`; agora usa
      `ChildKiller::clone_killer()`. Provas: 215 testes core, sonda e2e real
      com t1/t2 isolados e `tst_multi_terminal.qml` headless. Falta OK ao vivo
      das abas. Links clicáveis, busca no scrollback e refinos de seleção
      continuam conforto, não D2.3.
      **T2 FEITO (2026-07-14, protocolo 0.45.0):** o core usa `notify 8.2.0`
      (pin exato auditado; inotify no Linux, polling como fallback) sem criar
      scanner próprio. O watcher é lazy/não recursivo: raiz + diretórios
      expandidos/lidos; ignora VCS, caches, builds e temporários atômicos;
      agrega eventos em 180 ms e publica `event.fs.changed` tipado. Editor
      limpo recarrega sozinho; editor sujo preserva o buffer e mostra escolha
      Recarregar/Manter local; árvore e Git atualizam pelo mesmo lote. A
      barreira final independe do watcher: `fs.write` compara o disco com
      `expectedContent`, retorna `FILE_CHANGED` se o snapshot envelheceu e só
      então faz a escrita atômica. Backend real + 221 testes core, 54 de
      protocolo, harness QML de conflito, qmllint, clippy e build UI verdes.
      Auditoria/decisão: `docs/tooling/OPEN_COMPONENT_REGISTRY.json` e
      `docs/adr/ADR-0001-notify-filesystem-watcher.md`.
      **RESPOSTA "o que falta pra usar sem medo (nao ir pro Zed)":** a
      infra dificil ja existe (build/debug/git/terminal/LSP-nav/diagnosticos/
      rede-de-seguranca). Os BLOQUEADORES REAIS eram 2, e os DOIS foram
      fechados em 2026-07-12: (1) D1 autocomplete ao vivo — CORRIGIDO
      (causa-raiz Item.visible, acima) — e (2) **D1b Find/Replace no arquivo
      (Ctrl+F/Ctrl+H) — IMPLEMENTADO.**
      **D1b (2026-07-12):** busca/substituicao DENTRO do arquivo aberto.
      Arquitetura: **100% UI, ZERO core** — o buffer ja vive no editor, entao
      buscar nele nao passa por IPC (protocolo SEGUE 0.42.0, sem RPC novo).
      Contraste deliberado com o Ctrl+Shift+F, que E do core porque o ripgrep
      varre o DISCO. O core so ganhou os descriptors `editor.find`/
      `editor.replace` pra paleta (execucao UI-side, igual settings.get).
      Pecas: EditorFindController.qml (logica) + EditorFindBar.qml (barra) +
      canal novo de spans no editor_highlighter (realca TODAS as ocorrencias,
      a atual mais forte). Enter/Shift+Enter navegam (circular), Esc fecha,
      contador "3 de 17", toggles Aa/W/.*, Substituir/Tudo, regex com $1.
      F3/Shift+F3 (+ alternativas sem F-key Ctrl+Alt+G/Ctrl+Alt+Shift+G).
      Armadilhas tratadas: regex de LARGURA ZERO (`a*`) avanca a forca (senao
      LACO INFINITO); replaceAll de TRAS PRA FRENTE (senao corrompe offsets);
      trocar de aba/digitar revarre (offsets envelhecem). Provado por harness
      headless (11 assercoes verdes no controller REAL) + gate + smoke.
      **B1+B2 FEITOS (2026-07-12, protocolo 0.43.0) — bugs reportados pelo
      usuario:** (B1) "a roda do mouse nao rola o terminal". A causa NAO era a
      fiacao (correta ponta a ponta): era bug do **vt100 0.15.2** que DERRUBAVA
      O CORE — `visible_rows()` faz `rows - scrollback_offset` em usize, entao
      offset maior que a altura da tela = OVERFLOW (panic em debug; tela errada
      em release). A UI alimentava o caso: o `onWheel` somava SEM TETO, e ~9
      cliques de roda estouravam. O core morria, a recuperacao de crash da M4.3
      relancava CALADA, e o usuario via "nao acontece nada". Fix em 3 camadas:
      vt100 0.15->0.16.2 (upstream satura a subtracao; de quebra o scrollback
      deixa de ser limitado a UMA tela — API mudou: set_scrollback/set_size
      foram do Parser pro Screen), core passa a mandar `scrollback` (offset
      real clampado) e `scrollbackMax` (quanto historico existe) no render, e a
      UI troca MouseArea.onWheel por WheelHandler + clamp + reconciliacao pelo
      render (core = fonte da verdade). (B2) NAO existia ScrollBar em NENHUM
      lugar do ui/qml — nasceu `shell/VerticalScrollBar.qml` (propria, sem
      QtQuick.Controls; generica na unidade: px no editor, LINHAS no terminal
      com eixo invertido), aplicada no editor, terminal e ListViews dos
      paineis inferiores (reaproveitando o mesmo componente). Sonda e2e
      `scripts/sonda_scrollback.py`. LICAO (a mesma do D1): sonda de core verde !=
      gesto do usuario funcionando; o que fechou o caso foi a sonda exercitar o
      CASO EXTREMO que o gesto real produz.
      Falta so o OK do usuario ao vivo. tree-sitter/remake
      deixam mais solido/bonito, nao "usavel". Depois: D3 tree-sitter + arquitetura de plugins
      (ADOCAO DIRETA no core, NAO extension host; camada lang/ registry por
      linguagem C/C++/Rust; highlight/fold/outline independentes do LSP) +
      views em arvore (explorer + outline; usuario fazendo os ICONES
      proprios) -> D4 remake de UI/UX (docs/roadmaps/20). Dogfooding em paralelo.
      Radar aberto: primitiva de "aviso discreto" (toast, para o "N
      recuperados"); diffBase como 4o setting;
      adotar tree-sitter (MANDATO obrigatorio, ver "Direcao do produto");
      preencher as metricas MANUAIS do orcamento (docs/roadmaps/21 M4.2) numa sessao
      GUI real; Local History completo + migrar session/settings pro mesmo
      SQLite (docs/seguranca/23 P4). Docs desta rodada: docs/build/22 (comandos de
      compilacao C/C++/Rust), docs/seguranca/23 (rede de seguranca).
      Ordem completa pos-M3 na secao final do docs/roadmaps/21. Trilha E
      concluida em 2026-07-10 (E1-E3). O docs/roadmaps/21 tambem
      ganhou (pedido do usuario) a **Trilha T**: paridade de toolchain
      C/C++ e Rust com regua Neovim ADAPTADA (inventario stack→estado +
      fatias T1-T9: switch header/source, tidy no editor, inlay hints,
      check continuo, sanitizers, diagnostics na gutter, runnables,
      expand macro, crates offline-first).
      Toolchain/Environment Settings, CMake/Cargo toolbar e Settings/Storage
      entram nos marcos M1/M2 conforme o plano.
5. `docs/roadmaps/BACKEND_TO_UI_UX_ROADMAP.md` continua sendo a ponte backend->UI: nao
   substitui `docs/specs/`, so evita que o backend avance sem mapear a
   experiencia visual futura. Atualizar os dois ao fim de cada entrega.

Decisoes anteriores (2026-07-03/04) ja cumpridas e resumidas: UI/UX
basica antes de features grandes; syntax highlighting; Fase 4 (build); Fase 5
(LSP: diagnosticos, go to definition, hover, completion, find usages, rename —
semantic tokens e code actions ainda pendentes); Java/Python fora do curto
prazo (pos-V1); IA usada via terminal (Claude/Codex/GPT), sem provider embutido
no MVP. Detalhe: git log e `docs/roadmaps/BACKEND_TO_UI_UX_ROADMAP.md`.

Ideia de produto pos-V1 (2026-07-04, nao bloqueia V1.0): "modos de compilador"
e "loja de funcoes" para C/C++ e Rust, com janela de opcoes visual e
JetBrains-like (nome, explicacao, impacto, risco, fonte, previa, reversao),
ordenando por confianca (ISO/Rust oficial primeiro). Ainda sem doc dedicado
apos a limpeza de `docs/archive/`; se for retomada, criar
`docs/arquitetura/17-compiler-modes-and-function-store.md` antes de implementar.

## Visao pos-V1.0

Depois da V1.0 o foco e polir a IDE continuamente para chegar o mais perto
possivel das IDEs JetBrains em analise, navegacao e refatoracao — mesmo
orquestrando ferramentas abertas (CMake, LSPs etc.). O objetivo explicito e
NAO terminar como "um VS Code": a régua de qualidade de navegacao/refactoring
e JetBrains. Isso reforca a Fase 5 (LSP) como investimento central: semantic
tokens, go-to-definition, find usages, rename via LSP, e futuramente acoes de
refatoracao proprias por cima do que os LSPs oferecem.

## Instrucoes por agente (confirmado pelo usuario em 2026-07-03)

O usuario trabalha com dois agentes de IA em paralelo e vai dividir
responsabilidades entre eles. Este arquivo e o ponto de sincronizacao: quando
os tokens de um acabarem, o outro continua o desenvolvimento lendo este
arquivo.

- FABLE (Claude Code): sessao de DESENVOLVIMENTO. Autorizado a alterar codigo
  quando o usuario pedir "prossiga". Responsavel por manter este arquivo
  sincronizado ao fim de cada entrega.
- CODEX: a instrucao "nao alterar codigo, apenas documentacao" registrada
  anteriormente valia para a sessao do Codex, NAO para o Fable (o usuario
  confirmou isso explicitamente). Se o usuario mandar o Codex desenvolver,
  ele deve ler este arquivo inteiro, seguir a politica anti-duplicacao e as
  decisoes registradas, e continuar do estado descrito na secao "Estado
  tecnico atual" + "Prioridade imediata".
- Regras comuns aos dois:
  - O icone/atalho do aplicativo ja existe. Nao recriar, reinstalar ou
    modificar icone/atalho sem pedido explicito. Recompilar os binarios
    release que o atalho executa e obrigatorio apos mudancas de codigo.
  - Ler o codigo existente antes de editar; nada de codigo duplicado.
  - Verificacao antes de dar por pronto: `cargo kw-fmt/kw-clippy/kw-test`,
    build debug + release da UI, `scripts/verificar-cpp.sh`, smoke offscreen.

## Decisoes de produto registradas em 2026-07-03

- ASSISTENTE KW (Fase 7): por baixo dos panos ele vai executar os CLIs
  `claude` (Claude Pro) e `codex` no terminal, como o usuario ja faz
  manualmente. O painel e um "atalho para IA": visualmente separa terminais
  por provider, evitando digitar `claude`/`codex` toda vez. NAO e integracao
  via API num primeiro momento.
- LOGS: o log de tráfego IPC/acoes da IDE e ferramenta de desenvolvimento DA
  IDE, nao do usuario final — por isso a aba do painel inferior e "IDE", nao
  "Logs". "Problemas" mostra erros DO PROJETO (build, diagnosticos).
- ERROS DA IDE EM .TXT: qualquer erro/mau funcionamento da IDE e gravado com
  timestamp ISO em `~/.cache/kinein-vectis/logs/kinein-ui-erros.txt`
  (`CoreClient::appendErrorLog`, propriedade QML `errorLogFile`).

## Politica anti-duplicacao de codigo

Regra obrigatoria: nao duplicar codigo, componentes, comandos IPC ou estado de
UI que ja existem.

Antes de implementar qualquer coisa, uma IA deve:

1. Buscar implementacoes existentes com `rg`.
2. Ler os arquivos relevantes antes de propor ou editar.
3. Reaproveitar, estender ou refatorar o que ja existe.
4. Criar novo arquivo/componente apenas quando houver necessidade real e sem
   sobrepor responsabilidade de componente existente.
5. Atualizar docs quando mudar contrato, arquitetura ou comportamento.

Elementos que ja existem e devem ser reaproveitados:

- `ui/src/core_client.h` e `ui/src/core_client*.cpp`: cliente IPC da UI. Nao
  criar outro cliente IPC paralelo; manter `CoreClient` como fachada QML unica
  e dividir so a implementacao interna por responsabilidade.
- `ui/qml/workspace/FolderPickerDialog.qml`: seletor proprio de workspace.
  Nao voltar para `QtQuick.Dialogs` e nao criar outro seletor de pasta
  duplicado.
- `ui/qml/Main.qml`: composition root da UI. Nao recolocar nele modelos,
  timers, handlers IPC ou blocos grandes de dominio se ja houver componente,
  controller ou roteador em `shell/`, `panels/bottom/`, `workspace/`,
  `assistant/`, `project/`, `command/`, `editor/`, `jobs/`, `runtime/`,
  `search/` ou `ipc/`.
- `ui/qml/Theme.qml`: tokens visuais. Nao espalhar cores soltas quando um token
  existente resolver.
- `workspace.browse`: navegacao de diretorios para abrir workspace. Nao criar
  outro metodo IPC para a mesma finalidade.
- `fs.list`, `fs.read`, `fs.write`: acesso a arquivos dentro do workspace. Nao
  acessar filesystem diretamente pela UI.
- `tools.detect` e `tools.status`: deteccao/status de ferramentas. Nao criar
  outro fluxo paralelo de deteccao na UI.

Exemplos do que nao fazer:

- Nao criar `CoreClient2`, `WorkspaceClient`, `FileSystemClient` ou similar se
  a responsabilidade pertence ao `CoreClient`.
- Nao criar outro `FolderPicker`/`OpenProjectDialog` com a mesma funcao do
  `FolderPickerDialog.qml`.
- Nao duplicar listas de arquivos no QML quando `treeModel`/explorer ja cobre o
  caso.
- Nao criar novo comando `workspace.list`, `workspace.folders` ou equivalente
  para navegar pastas se `workspace.browse` ja atende.
- Nao copiar blocos grandes de QML para variar layout; extrair componente ou
  refatorar quando a duplicacao ficar real.

Se uma IA achar que precisa duplicar algo, deve primeiro registrar o motivo no
plano da tarefa e pedir confirmacao do usuario.

## Cuidados para proximas IAs

- Nao adicionar acesso direto a filesystem na UI para contornar problemas de UX.
- Nao usar `QtQuick.Dialogs` para abrir workspace; o seletor proprio substitui
  essa dependencia.
- Nao duplicar protocolo de navegacao de pasta; use `workspace.browse`.
- Nao duplicar codigo existente; aplicar a Politica anti-duplicacao acima.
- Atualizar `docs/arquitetura/03-ipc-protocol.md` e schemas quando mudar contrato IPC.
- Manter docs sincronizadas quando comportamento de workspace/editor mudar.
- Nao recriar `docs/archive/`: foi removido de proposito em 2026-07-05 (ver
  `docs/arquitetura/15-engineering-debt-and-refactor.md`). Documento descontinuado vira
  resumo no doc numerado relevante e depois e apagado, nao guardado numa pasta
  de arquivo morto.

## Checkpoint 2026-07-14 — fundacao semantica, daily driver e convergencia UI

O estado real avancou para o protocolo **0.50.0**:

- Tree-sitter foi adotado diretamente no core para C/C++/Rust, com pins exatos,
  auditoria de licenca/ADR e design em `docs/roadmaps/25`. `lang/` fornece parsing
  incremental, highlight, folding, outline aninhado e locals sintaticos com
  cache LRU limitado. clangd/rust-analyzer continuam autoridades semanticas.
- Rename e code actions LSP agora produzem preview confirmavel. Aplicar valida
  snapshots e grava todos os arquivos com transacao/rollback compartilhada;
  cancelar nao toca no disco. A UI nao implementa executor concorrente.
- As lacunas T3–T6 foram fechadas: branches/checkout/create + pull/push/stash;
  `fs.replace` transacional; Salvar tudo; arquivos recentes via o Search
  Everywhere existente.
- A convergencia D4 ganhou componentes vetoriais centrais, App Bar, Main
  Toolbar, Start Screen, criacao de projeto com preview, scaffold C++23
  target-based e KV Context local-first. O unico gate restante da fatia e a
  validacao visual humana R7/C6 em tela real.
- A rodada de uso visual revelou lacunas funcionais sem exigir novo remake.
  O lote 0.50 corrige o empilhamento de tooltips e menus por overlays globais,
  liga as acoes Arquivo–Ajuda aos controllers existentes, expõe criar
  arquivo/pasta também no clique direito e persiste um layout responsivo. A
  Estrutura agora redimensiona e recolhe para uma aba discreta à direita.
- O KV Context deixou de simular chat/modos locais e formalizou o AI CLI
  Bridge: detecta os executaveis allowlisted `claude`/`codex`, usa Claude como
  preferencia inicial, inicia a escolha explicitamente no mesmo PTY manager e
  permite sair/trocar sem acoplar a superfície ao Terminal comum. As CLIs devem
  ser instaladas/autenticadas pelo usuario; a IDE nao chama API nem envia
  contexto automaticamente. Uma opcao guiada para outra IA e Configuration
  Actions/biblioteca de funcoes ficam para depois do aceite desta correcao.
- O complemento `docs/roadmaps/KINEIN_VECTIS_DEEP_SEMANTIC_ENGINE_CPP_RUST_WORKFLOW.md` e
  compativel com a arquitetura vigente. O recorte acionavel imediato (camada
  sintatica, versoes de documento e workspace edit conservador) esta feito.
  Project Graph, Context Matrix, CMake File API completa e scheduler multi-LSP
  sao a proxima fase KSWE e devem entrar em fatias desenhadas/medidas, sem
  duplicar CMake/Cargo/LSP/Job System ja existentes.
- Meta de longo prazo registrada pelo usuario: a IDE deve entrar em “simbiose
  com os compiladores”. Em termos tecnicos, isso significa construir contexto
  incremental profundo do projeto a partir de CMake File API,
  `compile_commands.json`, Cargo Metadata, LSP, compilador, analisadores e DAP,
  normalizado pelo Project Graph/Context Matrix do KSWE. O primeiro patamar e
  “um nivel abaixo do CLion”; a evolucao posterior busca fluxo equiparavel por
  profundidade de integracao, nunca por reimplementar compilador. O norte e a
  lista de capacidades ficaram vinculados em `docs/roadmaps/21-long-horizon-roadmap.md`.

Validacao automatizada desta entrega inclui testes Rust, harness QML real,
qmllint estrito, builds Qt e smoke offscreen. Nao marcar C6 como visualmente
aceita ate o usuario abrir a GUI e conferir contra as specs.

## Feedback pós-checkpoint 0.50 (2026-07-14)

- O checkpoint grande foi salvo no commit `b506d87` e publicado em
  `origin/main` antes desta nova rodada.
- A correção pós-checkpoint foi validada e publicada em `928fbb5`: scroll do
  terminal coalescido/ao vivo, fallback Tree-sitter imediato no autocomplete,
  menus em overlay global e `Ajuda → Manual da IDE` renderizado internamente.
- O dogfooding iniciado depois deste checkpoint revelou uma divergência nova:
  o KV Context não mostrava scrollback/barra no Codex e ficava estreito para
  uma TUI de uso diário. A correção 0.51 está descrita na seção final deste
  arquivo; o usuário pausou explicitamente o aceite visual restante em
  2026-07-15 e autorizou a fatia A1.
- Workspaces recentes foram entregues na Start Screen e em
  `Arquivo → Abrir recente`: lista global limitada, último acesso,
  fixar/remover/limpar e tratamento de caminhos inexistentes, sempre
  reutilizando `workspace.open`.
- Plataforma honesta: Linux-first. Arch/CachyOS é o alvo validado; há bootstrap
  para Debian/Ubuntu/Fedora, ainda dependente de CI/teste contínuo. Windows não
  é suportado hoje, embora Qt/Rust/portable-pty deem base para uma port futura.
- Distribuição AppImage x86_64 implementada: empacota UI, core, Qt/QML,
  plugins, manual e licenças para o usuário não precisar instalar Rust/Qt só
  para abrir a IDE. Compiladores, CMake/Ninja, LSPs e debugadores continuam
  externos e opcionais por linguagem.
- Qualquer entrega futura do código a terceiros deve usar uma cópia sanitizada
  gerada do repositório privado. A cópia leva o código do projeto e, entre
  Markdown, somente `README.md`, `MANUAL.md` e `Tutorial.md`; excluir
  ContextoIA, PONTO_ATUAL, GUIAIA, AGENTS, `docs/`, `docs-privada/prompts/`, specs,
  roadmaps e demais notas de agentes. Implementar exportador allowlist com
  dry-run, recusa de Markdown extra e auditoria de segredos antes de entregar
  ou criar um espelho. A entrega não inclui `.git/` nem o histórico privado; um
  eventual espelho começa com histórico próprio da árvore sanitizada. O
  repositório-fonte permanece privado e sua visibilidade não deve ser alterada.

## Mapa operacional e transição de IDE (2026-07-14)

- `GUIAIA.md` é o roteador operacional vivo entre documentação e código:
  organiza fontes por pergunta, mapeia UI → CoreClient → protocolo → handler →
  serviço → teste e registra arquivos que mudam juntos. Ele não substitui
  ContextoIA, ARCHITECTURE, specs ou contrato IPC e deve acompanhar qualquer
  mudança de módulo/domínio/router/controller/gate.
- Sequência confirmada: (TR0) aceite visual; (TR1) self-hosting e substituição
  do VS Code/editores generalistas; (TR2) KSWE/Project Graph/Context Matrix até
  o patamar inicial abaixo do CLion; (TR3) profundidade semântica e embarcados.
  O prefixo TR evita colisão com P0–P3 do roadmap de adaptação de plugins.
- Quando o usuário disser “estou no Kinein”, o dogfooding volta a ser a fonte
  principal do backlog: registrar cada saída para outra IDE e corrigir o motivo
  reproduzível antes de confortos hipotéticos.
- Para testadores Linux, o artefato preferido é AppImage com UI, core, runtime
  Qt, desktop/icon e checksum. PKGBUILD local é alternativa mais simples para
  um grupo exclusivamente Arch/CachyOS; checkout + bootstrap fica reservado a
  colaboradores que realmente vão compilar a IDE.
- Responsabilidade dos Markdown externos: `README.md` apresenta o produto,
  `MANUAL.md` cobre somente o uso da IDE e `Tutorial.md` cobre recebimento,
  checksum, execução, atualização, packaging e entrega sanitizada do código.

## Distribuição AppImage portátil (2026-07-14)

- Implementados `scripts/empacotar-appimage-portatil.sh` (builder Podman),
  `scripts/empacotar-appimage.sh` (AppDir/linuxdeploy),
  `scripts/instalar-appimage.sh` (um único atalho de usuário para a versão mais
  recente, com remoção opcional das anteriores),
  `scripts/testar-appimage.sh` (estrutura/core/primeiro frame) e
  `scripts/testar-appimage-portatil.sh` (runtime mínimo sem rede).
- Builder fixado em Rust 1.96.1/Debian 12 por digest; `linuxdeploy`, plugin Qt e
  runtime type-2 fixados por versão/commit/SHA256 no registry de tooling. A
  auditoria e o rollback estão em
  `docs/adr/ADR-0003-linuxdeploy-appimage-packaging.md`.
- O bundle contém `kinein-vectis`, `kinein-core`, Qt/QML, módulos
  `QtQuick`/`QtQuick.Window`/`QtQml.WorkerScript`, plugins xcb/Wayland/
  offscreen/minimal, ícone, desktop, AppStream, `MANUAL.md` e licenças.
- A receita exige explicitamente os plugins de plataforma xcb, Wayland,
  offscreen e minimal e falha antes da distribuição se o arquivo ou uma
  dependência dinâmica obrigatória estiver ausente. O diretório `dist/` recebe
  também `instalar-kinein-vectis.sh`; ele nunca usa sudo, mantém um único ícone
  e sempre o aponta para a maior versão disponível na pasta.
- Smoke aprovado no host Arch e, com rede desligada, em Debian 12 mínimo sem
  Qt, Rust, CMake ou compiladores. O artefato `0.1.0` foi reempacotado após a
  separação entre manual de uso e tutorial externo: 33.573.368 bytes, SHA256
  atual `ef5970f322aced46905c66dbca5f1360964cde3a4871030bb8f321d771277fe7`.
- Compatibilidade prometida nesta etapa: Linux x86_64 atual, baseline glibc
  2.36, com a pilha gráfica/fontes normal de um desktop. Ubuntu/Fedora,
  atualização e canal de release ainda precisam de matriz explícita; Linux
  histórico anterior ao baseline não deve ser chamado de suportado.
- O smoke na base mínima revelou e corrigiu duas lacunas que o Arch mascarava:
  imports QML ausentes no bundle e aliases de `EditorOutline*` fora da raiz do
  módulo. O strict build Qt 6.4 também passou após tornar as comparações de
  strings do terminal independentes da conversão `qsizetype`→`int`, sem
  relaxar warnings-as-errors.
- Após A1/A2, a próxima funcionalidade de produto é **responsividade medida**;
  a ordem viva está somente em `PONTO_ATUAL.md`.

## Workspaces recentes globais — A1 (2026-07-15)

- O protocolo `0.53.0` adiciona `workspace.recent.list/pin/remove/clear`; todos
  devolvem um snapshot completo e a UI não ordena nem consulta o filesystem.
- Aberturas e projetos criados com sucesso atualizam atomicamente
  `$XDG_CONFIG_HOME/kinein-vectis/recent-workspaces.json`, versionado por
  `schemaVersion: 1`, com no máximo 12 entradas. Fixadas vêm primeiro e o
  último acesso ordena cada grupo.
- `available` é calculado no core. Raiz ausente permanece visível,
  desabilitada e removível; uma entrada válida abre pelo `workspace.open` e
  herda a restauração de sessão por workspace já existente.
- A Start Screen exibe até quatro entradas e permite abrir, fixar, remover e
  limpar. O menu Arquivo expõe até oito entradas e a limpeza.
- Migração v0, schema futuro/inválido, ordenação, deduplicação, limite,
  fixação, caminho ausente e mutações estão cobertos no core; o harness
  `tst_recent_workspaces.qml` cobre o controller e o bloqueio de raiz ausente.
- `scripts/verificar.sh` passou completo (328 testes Rust, Clippy estrito,
  C++/QML estritos, 12 harnesses e builds debug/release); smoke offscreen e
  sonda com dois processos do core/XDG isolado também passaram.
- O storage contém somente nome, raiz, último acesso e fixação; não armazena
  arquivos, credenciais, contexto de IA ou estado de sessão duplicado.

## Dogfooding ativo e correção KV Context terminal-first (2026-07-14)

- O gatilho **“estou no Kinein”** foi recebido; self-hosting/dogfooding está
  ATIVO. `PONTO_ATUAL.md` já registra a transição e permanece a fila viva.
- Primeiro feedback real: embora o KV Context já executasse Claude/Codex no
  mesmo PTY do Terminal, o Codex entrava em alternate screen. Esse modo não
  produz scrollback, então a barra sintética não tinha histórico para mostrar;
  além disso, os 360px do seletor eram estreitos para a TUI e resize contínuo
  podia serializar grids demais.
- Correção 0.51: Codex é iniciado com sua opção oficial
  `--no-alt-screen` (argumento fixo/allowlisted, sem shell livre); o render
  expõe `alternateScreen`, `applicationCursor` e `bracketedPaste`; teclado
  cobre setas no modo pedido, Shift+Tab, Insert, F1–F12, Alt e Ctrl+Space;
  paste respeita os delimitadores VT; a barra reserva sua própria faixa e
  permanece perceptível; resize é coalescido a ~30 fps.
- A sessão ativa usa uma largura responsiva (metade da janela, até 720px,
  preservando o editor) e pode ser maximizada para a área de trabalho pelo
  header compacto. O seletor só existe antes da sessão; depois, a superfície é
  o `TerminalPanel` compartilhado, sem chat ou renderer paralelo.
- Segundo feedback real após o reinício: no modo inline do Codex, a linha onde
  se digita aparecia entre o aviso de usage e o status do modelo sem qualquer
  limite visual, parecendo flutuar. A captura do grid VT confirmou que essa
  ordem e o input pertencem à TUI real. O KV Context agora demarca somente a
  linha do cursor ao vivo com `Theme.currentLine`/`Theme.borderStrong`; o guia
  some ao consultar o histórico e não cria composer, chat ou parsing da saída.
- Terceiro feedback real: a faixa ainda não envolvia os glifos, o divisor era
  perdido durante a sessão, `Project` fechava compulsoriamente e o scroll
  desaparecia durante chats longos. A causa de layout era tripla: largura
  ativa recalculada em metade da janela, splitter oculto no estado ativo e
  `effectiveShowExplorer` amarrado à sessão. A causa de scroll combinava
  reconciliação exata demais após nova saída com versões do Codex que ainda
  emitem `CSI 3 J` mesmo usando `--no-alt-screen`.
- Correção 0.52: o texto da linha ativa é centralizado numa faixa com respiro e
  contorno acima dos spans ANSI; `assistantTerminalWidth` persiste 300–720px,
  o splitter continua ativo e `Project` só some na maximização explícita. O
  `TerminalScrollController` aceita deslocamento positivo causado por output,
  protege o snap contra frames atrasados e reseta por sessão. O bridge filtra
  somente `CSI 3 J`, inclusive dividido entre reads; o Terminal comum continua
  obedecendo `clear`.
- Quarto feedback ao vivo (2026-07-15): o dimensionamento foi confirmado como
  resolvido e ficou explicitamente fora do novo patch. A captura mostrou a
  limitação restante da faixa: ela representava só a última linha do cursor e
  sobrava como caixa vazia depois de Enter. A faixa agora nasce no primeiro
  caractere/paste, envolve todo o intervalo de uma entrada quebrada e some ao
  enviar/cancelar. A roda da sessão CLI também ganhou compatibilidade com os
  eventos `pixelDelta` que Qt/Wayland pode entregar no Arch, além do
  `angleDelta` tradicional; o fluxo IPC/core compartilhado não mudou.
- Para manter a regra de split, o antigo `TerminalPanel.qml` de mais de 500
  linhas foi dividido em `TerminalViewport.qml`,
  `TerminalSelectionController.qml`, `TerminalInputController.qml` e
  `TerminalScrollController.qml`; o header saiu para
  `AssistantTerminalHeader.qml`. Novos harnesses cobrem layout responsivo,
  teclado, seleção e scroll cruzado com output.
- Validação automatizada fechada: `scripts/verificar.sh` completo verde
  (testes, clippy `-D warnings`, C++/QML estritos e builds debug/release) e
  smoke release pelo launcher vivo por 8s, sem saída QML (`exit 124` esperado).
  Os binários release usados pelo ícone foram atualizados.
- Ciclo obrigatório: registrar ação/esperado/observado/ambiente → reproduzir →
  achar causa-raiz → adicionar teste/harness quando aplicável → corrigir na
  camada dona → rodar gate e validar o gesto real → sincronizar docs.
- Prioridade: P0 perda/corrupção de dados, segurança, crash ou IDE não abre;
  P1 bloqueio diário de edição/build/run/debug/terminal/Git/navegação; P2
  regressão funcional reproduzível; P3 conforto ou feature nova.
- O aceite humano restante da entrada multilinha e da roda permanece em TR0,
  mas foi pausado explicitamente pelo usuário em 2026-07-15; a autorização
  direta para A1 não equivale a aceitar esses dois gestos do KV Context.
- Com A1/A2 entregues, seguir responsividade medida e os confortos exigidos por
  saídas reais para outra IDE. Não criar outro roadmap:
  `PONTO_ATUAL.md`, `docs/roadmaps/21-long-horizon-roadmap.md` e os docs de domínio já
  são a fila.
- O usuário autorizou commits locais de checkpoint após cada marco crítico com
  gate verde. Push, publicação, alteração de visibilidade ou entrega externa
  continuam sem autorização. O
  repositório-fonte continua privado; código externo só pela futura árvore
  sanitizada descrita na política vigente.

## Estabilização de distribuição, editor e self-hosting (2026-07-15)

- O AppImage distribuído e o checkout de desenvolvimento agora coexistem no
  menu: `kinein-vectis.desktop` / **Kinein Vectis** pertencem ao AppImage;
  `kinein-vectis-development.desktop` / **Kinein Vectis (Desenvolvimento)**
  pertencem a `scripts/kinein-vectis`. `scripts/instalar-atalho.sh` migra
  apenas o atalho legado que aponta comprovadamente para o mesmo checkout e
  preserva um atalho de AppImage já instalado.
- Feedback de um testador: em alguns arquivos o indicador de breakpoint
  cobria o número da linha. Causa: a largura da coluna numérica era estimada
  como `8 px × dígitos`, apesar de `editorFontSize` variar de 8 a 40. A gutter
  foi extraída para `ui/qml/editor/EditorGutter.qml`; folding/breakpoint,
  diagnóstico, blame e números têm faixas independentes e a largura dos
  números vem de `FontMetrics`. DAP, estado de breakpoints e protocolo não
  mudaram.
- Os cinco SVGs fornecidos em `docs/iconografia/icones-da-arvore/` foram
  copiados sem alteração de bytes para `ui/assets/icons/tree/` e integrados no
  `KvIcon`/`ProjectExplorer`: pasta fechada/aberta, C, C++ e Rust. O Qt apenas
  os dimensiona no slot da árvore; o desenho original não foi redesenhado,
  simplificado ou substituído.
- A disputa aparente Tree-sitter/LSP era uma corrida de resposta: após uma
  edição, tokens semânticos da versão anterior ainda podiam chegar e vencer
  temporariamente o fallback estrutural atual. `syntaxVersion` e
  `semanticVersion` agora avançam imediatamente por edição, os semantic tokens
  anteriores são limpos, e cada resposta LSP carrega `path`+`version`; somente
  a resposta do documento e versão ativos é aplicada. Os papéis continuam
  separados: Tree-sitter é a base sintática instantânea e clangd/
  rust-analyzer são a autoridade semântica progressiva.
- O KV Context deixou de desenhar faixa/caixa própria sobre a entrada. Como no
  terminal integrado, a grade VT, seus spans ANSI e o cursor são a única fonte
  visual; teclado, paste, seleção, scrollback, resize coalescido e modo inline
  continuam reutilizando o mesmo `TerminalPanel`/`TerminalManager`. A referência
  foi o comportamento terminal-first do Code OSS/xterm.js, adaptado à UI da
  Kinein, sem copiar código nem criar um renderer de conversa.
- A2 foi entregue: `workspace.kind` permanece como classificação primária
  compatível, enquanto `workspace.capabilities.buildSystems` representa Cargo
  e CMake simultaneamente. Toolbar, menus, Project Health, build e testes
  apresentam/roteiam os dois sistemas usando Jobs e serviços existentes.
- Scripts `.sh`, `.bash` e `.zsh` dentro do workspace ganharam ação de executar
  na árvore e no menu de contexto, seguindo a praticidade de IDEs profissionais.
  A UI envia somente o caminho; o core o confina à raiz e chama o interpretador
  com argv explícito (`--` + caminho canônico), sem montar uma linha de shell.
- A toolbar reduz informações secundárias por largura para não invadir o
  editor. O strict mode permanece integral para fontes C++ próprias; somente
  MOC/QML AOT gerados pela Qt recebem supressão local de warnings de código
  gerado.
- Packaging usa diretórios CMake distintos (`native-host` e baseline portátil)
  e `cmake --fresh`, impedindo cache de `/workspace` de contaminar o host. Os
  mounts Podman usam contexto SELinux e os scripts são invocados por `bash`,
  portanto o build portátil não depende do bit executável no checkout. A saída
  única continua `dist/` e contém AppImage, checksum específico,
  `instalar-kinein-vectis.sh` e uma cópia atual de `Tutorial.md`; o teste recusa
  tutorial ausente ou divergente.
- Uma execução manual revelou que o worker de packaging ainda podia ser
  chamado diretamente no Fedora: além de voltar a expor cache CMake criado em
  `/workspace`, o Qt 6.11 do host representa o plugin Wayland de modo diferente
  do Qt 6.4/Debian do baseline. A entrada direta agora encaminha ao builder
  portátil, `--baseline-worker` ficou reservado ao container, e a entrega é
  preparada em staging antes de substituir arquivos em `dist/`. Assim uma
  falha de compilação, plugin ou geração preserva o último AppImage válido.
- Gate integral após a consolidação: 334 testes Rust (256 core, 63 protocolo,
  15 demais), Clippy `-D warnings`, clang-format/clang-tidy, qmllint zero
  warnings, 12 harnesses QML e builds UI Debug/Release passaram; os binários
  release do launcher de desenvolvimento foram atualizados.
- AppImage `0.1.0` final após a proteção da entrada e publicação por staging:
  33.737.208 bytes, SHA256
  `86b335b2b1ba8c81d958df4f1e45f7d9c0838fdad2a2567c84199f84b8dbdc0d`.
  Smoke do host e Debian 12 mínimo sem rede passaram. O instalador distribuído
  foi exercitado de fora da pasta de entrega em XDG isolado e gerou `.desktop`
  e PNG apontando para o AppImage ao lado do próprio script.
- A estabilização inteira foi preservada no checkpoint Git local `c75537d`
  (`feat: estabiliza distribuicao e self-hosting hibrido`); não houve push.
- A proteção final do empacotamento AppImage foi preservada no checkpoint
  local `948535d` (`fix: protege empacotamento AppImage portatil`), também sem
  push.

## Alinhamento da grade do terminal e conforto visual (2026-07-15)

- Novo feedback de dogfooding interrompeu A3: no KV Context, o caret vertical
  aparecia muito depois da última palavra, e os trechos verdes/bold do prompt e
  da pasta tinham saturação e peso desconfortáveis.
- Causa fechada: `Theme.monoFont` usava uma pilha CSS inteira em uma propriedade
  QML que recebe uma família, permitindo fallback proporcional; ao mesmo tempo,
  spans eram dimensionados por `implicitWidth`, mas o cursor por coluna VT.
- O protocolo `0.56.0` agora inclui `cells` em cada span. O core preserva a
  contagem de colunas inclusive na continuação de glifos largos; a UI usa a
  mesma métrica monoespaçada para span, resize, seleção e caret. O cursor é uma
  barra fina pulsante, o bold ANSI usa peso médio e os verdes normal/brilhante
  foram suavizados para tons verde-azulados coerentes com o tema.
- A sequência e o significado do prompt continuam pertencendo ao shell. A IDE
  apenas renderiza seus atributos ANSI pela própria paleta; nenhum parser,
  overlay ou segundo campo de entrada foi criado.
- Referências arquiteturais registradas em `docs/roadmaps/24`: Code OSS
  `234638618394269563dd77c0c395c270d8df8b12` (MIT, modo B), Zed
  `1e22d1a83f8b1b7acc528d15cfab0644852380c0` (modo D, somente referência) e
  documentação oficial Qt 6. Nenhuma função, classe ou implementação foi
  copiada.
- Gate integral verde após a correção: 335 testes Rust, Clippy `-D warnings`,
  C++/QML estritos, 12 harnesses e builds Debug/Release; smoke release
  offscreen vivo por 8 s, sem saída QML (`exit 124` esperado). O harness de
  seleção cobre glifo largo e o caractere posterior. Aceite visual do caret e
  da paleta permanece humano após reiniciar o atalho de desenvolvimento.
- Aceite parcial do usuário: posição horizontal letra a letra, cursor do
  Terminal comum e verdes suaves foram aprovados. O teste `-2` exclusivo do KV
  Context ficou perto, mas não resolveu o cursor da TUI; testar `0` também não.
  O usuário esclareceu que KV Context não é outro terminal: é somente uma aba/
  apresentação separada da mesma base PTY para manter Claude/Codex visível
  enquanto a aba Terminal fica livre. Todo offset por superfície foi removido.
- Pesquisa oficial fechou o contrato: Codex posiciona o cursor nativo do frame
  e pode pedir `SteadyBar`; o changelog público do Claude também o denomina
  native terminal cursor. O `vt100` preservava linha/coluna/visibilidade, mas
  não expunha DECSCUSR. O protocolo `0.57.0` agora carrega `shape` e `blinking`;
  o core observa somente essa sequência com `vte` já transitivo e resolve
  `DefaultUserShape` para a barra padrão. O QML apenas converte a forma concreta
  e a célula VT em pixels, com altura integral, igual no Terminal e KV Context.
  Revisões e adaptação estão em `docs/roadmaps/24`; o aceite visual ainda precede
  qualquer novo AppImage.
- Após remover o offset inteiro e unificar a geometria de `DefaultUserShape`,
  o gate integral 0.57 passou novamente: 337 testes Rust, Clippy `-D warnings`,
  C++/QML estritos, 12 harnesses e builds Debug/Release. O launcher release
  ficou vivo por 8 s sem saída (`exit 124` esperado); binários de
  desenvolvimento atualizados e `dist/` intocado.
- Resultado humano posterior: ao abrir a Kinein por `scripts/kinein-vectis`, o
  usuário não percebeu mudança relevante no caret de Claude/Codex. O suporte a
  DECSCUSR continua válido, mas não encerra a causa visual; a versão 0.57 não
  recebeu aceite do cursor e o polimento foi adiado sem novo offset.
- Handoff obrigatório: `docs/roadmaps/26-terminal-rendering-parity-roadmap.md` registra
  reprodução, hipóteses, referências e R0–R7. A retomada começa por fixture PTY
  e métricas instrumentadas de glifo/célula/baseline/DPR. A auditoria do Code
  OSS `234638618394269563dd77c0c395c270d8df8b12` e xterm.js
  `ce2169485677951c7701129516cbb68e01330d86` mostrou que o renderer separa a
  caixa do glifo da célula, arredonda em pixels físicos e compartilha a mesma
  geometria com cursor/resize. Isso é hipótese arquitetural forte, não
  causa-raiz provada na Kinein.
- Decisão explícita do usuário: a funcionalidade do terminal Code OSS é a base
  de paridade para shell, Claude e Codex, mas deve ser traduzida para a
  realidade da Kinein (`portable-pty` + Rust Core + IPC tipado + renderer Qt).
  Electron, Node, WebView, Extension Host e runtime xterm.js não entram. Se a
  composição QML não cumprir a paridade, avaliar um `QQuickItem` nativo por
  evidência/ADR, preservando o mesmo backend e o mesmo renderer entre Terminal
  e KV Context.
- O usuário decidiu deixar esse polimento para o futuro e confirmou que o
  Terminal comum está visualmente bom — as melhorias de grade/paleta o deixaram
  melhor. O problema adiado não bloqueia A3 nem a análise da próxima integração;
  continua sem autorização para novo offset ou renderer exclusivo do KV.
- Auditoria dos artefatos encontrou uma explicação concreta para o teste sem
  mudança: `target/release/kinein-core` estava mais antigo que
  `crates/kinein-core/src/terminal.rs`, embora a UI release já estivesse atual.
  O gate foi repetido com os presets corretos `debug-strict` e
  `release-hardened`; 337 testes Rust, Clippy, C++/QML estritos, 12 harnesses e
  builds passaram. O launcher `scripts/kinein-vectis` ficou confirmado sobre
  `build/linux-clang-release-hardened/ui/kinein-vectis` +
  `target/release/kinein-core`; smoke offscreen de 8 s ficou vivo, sem saída
  (`exit 124` esperado).
- Hashes dos binários de desenvolvimento reconstruídos em 2026-07-15 (a UI
  mudou novamente na mitigação da barra de janela):
  UI `e8b870635cd8fcb679a02920ab5ab07be9609333f2f346fcd37b775c208bcfa4`;
  core `00029096a66fb5eafd9cf4c927b67106f27b4159f6427d2bd7246fb26147c2fc`.
  Esses são executáveis ELF do checkout, não AppImage; `dist/` não mudou.
- A baseline release de A3 foi revalidada com `N=3`: primeiro frame 250 ms,
  UI 103 MB, `workspace.open` 3,4 ms, `fs.read` 10k 0,0 ms e core 7 MB, todos
  dentro do orçamento; rust-analyzer externo 1093 MB informativo. A3.1–A3.4
  foram detalhadas em `PONTO_ATUAL.md`, reutilizando os scripts M4.2. Code OSS
  (marcos/fases) e Zed (fixtures determinísticas/bench real de input/render)
  foram registrados em `docs/roadmaps/21`; nenhuma infraestrutura externa entrou.
- Depois de A3, EditorConfig é a primeira integração pequena recomendada para
  a sessão de “novo plugin”: P0, útil para C/C++ e Rust e compatível com uma
  adoção auditável sem Extension Host. A recomendação ainda exige confirmar
  biblioteca/licença e contrato antes de código.
- Uma sessão futura foi reservada para preparar um repositório público novo e
  separado, seguindo o exportador allowlist de `docs/roadmaps/21` em vez de confiar só
  em `.gitignore`. Nessa sessão serão definidos com o usuário os nomes exatos
  de Markdown e diretórios que não podem aparecer, auditados caminhos,
  conteúdo, segredos e histórico, e só então considerada qualquer criação ou
  publicação externa. Nada foi publicado ou teve visibilidade alterada agora.

## Conforto imediato da barra de janela (2026-07-15)

- O dogfooding registrou `imagens/bugs/ReformularBarra.png`: a decoração
  server-side repetia `workspace - Kinein Vectis` em branco puro acima da App
  Bar e causava fadiga visual. Essa cor/peso pertencem ao compositor, não ao
  QML, e não têm ajuste portátil por aplicação.
- A mitigação mínima mantém a moldura nativa e deixa o título server-side
  vazio. A App Bar continua dona da identidade `Kinein` e
  agora mostra o nome real do workspace em `Theme.textMuted`, com elide. Um
  binding direto cobre os estados com e sem workspace; core, protocolo e IPC
  não mudam. Como o aceite é visual e depende do compositor, o gate automatiza
  lint/build/smoke e o usuário confere o resultado em tela real.
- Correção posterior do estado real: no AppImage em Fedora/Wayland, o usuário
  confirmou que Minimizar, Maximizar e Restaurar não estão visíveis. A frase
  acima sobre preservação dos controles nativos fica invalidada pelo
  dogfooding. Isso é regressão P2 anterior a qualquer código novo da barra.
  Próxima correção deve oferecer Minimizar e Maximizar/Restaurar alternável pelo
  estado autoritativo da janela, preservando geometria, Fechar, arraste,
  acessibilidade e Wayland/X11; smoke offscreen não basta para o aceite.
- Atualização (a pedido explícito do usuário, 2026-07-15): a integração
  client-side foi antecipada da C5/C6 para agora, com os controles embutidos na
  barra da própria IDE no estilo de acabamento das JetBrains. O polimento
  restante (snap, escala fracionária e multimonitor auditados) continua sob
  `docs/roadmaps/20`; a decisão de usar `FramelessWindowHint` já agora foi do usuário.
- Integração client-side entregue: `Main.qml` passa a
  `Qt.Window | Qt.FramelessWindowHint` e a App Bar (`AppMenuBar.qml`) hospeda os
  controles. `WindowControls.qml` expõe Minimizar, um único alternável
  Maximizar/Restaurar e Fechar (ícones `minimize`/`maximize`/`restore`/`close`
  em `KvIcon.qml`, tooltip e nomes acessíveis Kinein). O ícone/rótulo do
  alternável e o `resizeEnabled` das bordas seguem o estado autoritativo do
  `QWindow`, exposto por `ui/src/window_chrome_controller.{h,cpp}`
  (`WindowChromeController`, `QML_ELEMENT` do módulo `KineinVectis`):
  `showMinimized`/`showMaximized`/`showNormal`/`close` e
  `startSystemMove`/`startSystemResize` do compositor. A região livre da barra
  arrasta a janela (limiar de 6px) e o duplo clique alterna maximizar;
  `WindowResizeHandles.qml` cobre as oito bordas via `startSystemResize`,
  desabilitadas quando maximizada/fullscreen. Sem booleano visual paralelo em
  QML; core, protocolo e IPC não mudam — a janela é responsabilidade do shell Qt.
- Referência profissional desta fatia: IntelliJ IDEA Community oficial
  `e3b4dba36d013fc221b8471b3a4a8bd5336c24cc` (2026-07-15), em
  `platform/platform-impl/.../customFrameDecorations` e
  `WindowButtonsConfiguration.kt`, Apache-2.0/Mode-D. Invariantes adotados:
  controles no header; ordem minimizar → maximizar/restaurar → fechar; alternável
  guiado pelo estado real da janela; atualização quando o estado externo muda.
  Adaptação nativa em Qt/QML — nenhum código, Swing/JBR ou classe do IntelliJ foi
  copiado ou portado. Referências anteriores mantidas: Zed
  `1e22d1a83f8b1b7acc528d15cfab0644852380c0` (separação client/server, Mode-D) e
  documentação oficial Qt 6 sobre `Window.title`, flags e janela frameless.
- Gates verdes (2026-07-15): `scripts/verificar.sh` integral (Rust fmt/clippy/
  testes, C++ clang-format/clang-tidy, qmllint estrito, 12 harnesses de lógica
  QML, builds debug e release); smoke offscreen de 8s vivo (`exit 124`) sem erro
  de QML nos binários debug e release. Correções durante a validação:
  `WindowChromeController` não pode ser `final` (o `QML_ELEMENT` deriva dele);
  o mapeamento de bordas usa `Qt::Edges::fromInt` (sem bitwise com sinal);
  `WindowControls` é `Row`, então `implicitWidth/Height` são read-only.
- **Aceite real (2026-07-15):** o usuário testou em Fedora/Wayland e confirmou
  que os controles funcionam. A regressão P2 está encerrada. O release-hardened
  foi reconstruído e um AppImage novo (0.1.0) foi gerado e testado com este
  código a pedido do usuário, que passa a usar a Kinein no lugar do CLion para
  ganhar o log da aba IDE como vantagem de dogfooding. Commit local `3543de9`
  (feature) mais o commit de aceite; sem push.

## Pacote de ícones de arquivos especiais (2026-07-15)

- `docs/iconografia/icones-de-arquivo/` (raiz do repo) é um pacote de assets
  originais que complementa o sistema de ícones para os tipos de arquivo
  especiais da árvore de projetos: `cmake-lists` (CMakeLists.txt),
  `project-config` (fallback de `.env`/`.env.*`, `.clangd`/`.clang-format`/
  `.clang-tidy`, `CMakePresets.json`/`CMakeUserPresets.json`,
  `compile_commands.json`, `Cargo.toml`/`Cargo.lock`, `rust-toolchain.toml`,
  `rustfmt.toml`, `clippy.toml`, `Cross.toml`, `.cargo/config.toml`), `sql`
  (`*.sql`) e `docker-yaml` (somente nomes/paths Compose reconhecidos). Traz
  light/dark em 16/20/24 px, fontes SVG 64 px, `preview/icon-gallery.svg`,
  `ICON_CATALOG.json` e o contrato de resolução `FILE_ICON_MAPPINGS.json`.
- O resolver segue a precedência nome exato → padrão especial → caminho especial
  → extensão composta → extensão → genérico; YAML genérico não recebe o ícone
  Docker e `.env` tem regra de segurança (não vazar valores/segredos em tooltip,
  log ou envio externo). A semântica, o sizing ótico, a implementação Qt/QML
  (`Image` + `qrc` + `sourceSize` por DPR) e o checklist de validação estão nos
  `docs/01–04` da própria pasta.
- Licença MIT OR Apache-2.0 (mesma expressão do projeto); metáforas próprias, sem
  logos oficiais. Por enquanto é um drop de assets + guia de implementação: ainda
  não está ligado ao delegate da árvore (`ui/qml/project/*`). Ao ligar, seguir o
  `docs/arquitetura/03` do pack e a spec `KINEIN_VECTIS_VISUAL_SYSTEM_ICONS`, medindo
  desempenho antes de trocar SVG por PNG pré-rasterizado. Ponteiros em
  `GUIAIA.md` §3.2 e §5.2.

## Escalonamento linear das integrações solicitadas (2026-07-15)

- Os 53 candidatos de `PONTO_ATUAL.md` A5 foram tirados da categoria vaga de
  “plugin”. A5.1 define a arquitetura `QML → CoreClient → protocolo
  integration → registry/policy/adapters no Rust Core → Job → ferramenta` sem
  Extension Host; A5.2 fixa níveis L0–L10; A5.3 decide cada item como manter,
  adotar, condicional, referência ou substituir.
- O desenvolvimento começa depois da baseline A3 por uma fatia vertical de
  EditorConfig. Ela cria apenas o mínimo do domínio `integration` e registra
  ferramentas já detectadas; não nasce um crate/framework vazio. Qualidade e
  testes vêm antes de build/cache, depois debug/profiling, remoto, embedded,
  streaming, visualização, simulação e, por último, laboratório especializado.
- Correções de triagem: OpenSSH é base remota e SSHFS fica opcional; QEMU é
  local-first e Wokwi exige opt-in de rede/token/upload; Serial Studio é apenas
  referência/interoperabilidade por GPLv3 + módulos Pro; SCIP substitui LSIF
  numa avaliação futura; `cargo-llvm-cov` precede Tarpaulin; `lm-sensors`
  precede Open Hardware Monitor no Linux.
- Nenhuma dessas decisões instala, baixa ou ativa ferramenta. Cada adoção ainda
  exige auditoria atual de licença/manutenção/segurança, ADR/registry quando
  aplicável, consentimento para rede/USB/privilégio e gate vertical medido.

## Atualização integral transacional do checkout (2026-07-15)

- `scripts/atualizar-tudo.sh` é a entrada única para dúvida de cache ou
  UI/core desencontrados. Usa lock, fingerprint, backup/restauração dos quatro
  executáveis, CMake `--fresh`, rebuilds `--clean-first`, limpeza dirigida dos
  crates, gate completo, materialização explícita do core Debug, rejeição de
  fonte concorrente, smoke do launcher e manifesto de hashes. Não faz rede,
  pull, update de dependências, AppImage ou push.
- A execução integral real terminou verde às `2026-07-15T21:51:07-03:00`, com
  `source_sha256=fff40c44220bda2296d03e3c7def2a5826eae5124f85e513449e845ce376e97f`,
  gate completo e smoke `124` esperado. O manifesto está em
  `build/kinein-build-manifest.env`.
- Hashes produzidos na mesma transação: UI Debug
  `e545876e40cb046e05b3b0f8aa9c960004562f310cb0c853d2b6675be966bd4f`,
  UI Release `e8b870635cd8fcb679a02920ab5ab07be9609333f2f346fcd37b775c208bcfa4`,
  core Debug `a0e677275f10cd27a0e77e22ef28ce95c5804451a208a88f4d30f549cc2d48ac`
  e core Release `00029096a66fb5eafd9cf4c927b67106f27b4159f6427d2bd7246fb26147c2fc`.

## Compatibilidade gráfica do AppImage (2026-07-15)

- O primeiro dogfooding do AppImage no Fedora/Wayland expôs uma lacuna que o
  smoke offscreen não cobria: o plugin `wayland-egl` era encontrado, mas o
  driver do host não criava o contexto RHI/OpenGL, abortando antes da UI.
- O mesmo artefato foi exercitado no desktop real com
  `QT_QUICK_BACKEND=software` e confirmou o primeiro frame em 936 ms. A UI
  atual é 2D e não usa `ShaderEffect`, portanto o `AppRun` passa a selecionar
  por padrão a adaptação raster oficial do Qt Quick. Isso desacopla abertura
  normal de EGL/Mesa/NVIDIA em Wayland e X11.
- `KINEIN_GRAPHICS_BACKEND=hardware` oferece aceleração como opt-in reversível;
  overrides Qt explícitos continuam prevalecendo. O smoke portátil agora limpa
  esses overrides e exige evidência `Loading backend software`, evitando que
  uma reconstrução volte silenciosamente a depender da GPU.
- “Linux portátil” continua significando neste artefato x86_64, glibc 2.36 ou
  posterior. Musl, ARM64 e glibc anterior exigem outro artefato/baseline; não é
  tecnicamente correto prometer um único ELF para toda distro e arquitetura.
- AppImage corrigido: 33.749.496 bytes, SHA256
  `6c1a3c24a2b13ac36509ec615971eea75d604d36855f601726e083aa8af00312`.
  Passou no runtime Debian 12 mínimo sem rede e no Fedora/Wayland real sem
  variável de correção, confirmando `Loading backend software` e primeiro frame
  em 939 ms no modo extraído e 987 ms na montagem type-2 normal. O atalho
  instalado já aponta para esse mesmo caminho em `dist/`.
## Digitação medida no editor real — A3.3 item 1 (2026-07-16)

- `ui/src/typing_perf_harness.cpp` mede tecla→frame no editor real, atrás de
  `KINEIN_PERF_TYPING`. Sem a env não custa nada, na mesma disciplina do
  `KINEIN_PERF_MARKER` e do `KINEIN_TERMINAL_DEBUG_GEOMETRY`. Não é
  configuração: não entra em settings, não tem schema e não aparece na UI.
- O harness dirige o fluxo normal (`workspace.open` → `fs.read` → foco) porque
  sem workspace o editor não aceita tecla; é a orquestração, não a
  cronometragem, que fez a fatia ser própria. Alcança `coreClient` e
  `editorController` por `QQmlContext::objectForName`, sem exigir `objectName`
  no `Main.qml`: instrumentação não deixa marca no código que ela mede.
- Medido (release, 40 teclas, fixture Rust de 2463 linhas): mediana 7,4 ms,
  p95 8,4 ms, pior 9,3–13,2 ms. Estável em três runs (7,4 / 7,6 / 7,4).
  Orçamento: mediana 16 ms, p95 20 ms. Pior caso é informativo — uma amostra
  ruidosa, e reprovar por ela ensinaria a reexecutar até passar.
- A fixture é a mesma do A3.1: `SYNTAX_FIXTURE_FUNCTIONS` no `medir-core.py`
  virou constante única, e `--emit-fixture` a grava em disco para quem mede fora
  daquele processo. Tamanhos divergentes tornariam Tree-sitter e digitação
  incomparáveis sem ninguém perceber.
- Quatro decisões sustentam o número, todas registradas em `docs/roadmaps/21`:
  carimbo na render thread (`DirectConnection` no `frameSwapped`, porque queued
  mediria a fila de eventos junto); pisca do cursor desligado; espera de
  quietude de 150 ms entre teclas (o realce volta ~280 ms depois e geraria frame
  creditado à tecla seguinte); e `typing_chars_inserted` obrigando N teclas a
  virarem N caracteres — o gêmeo da armadilha do eco do shell que o item 3 pagou.
- Offscreen não tem vsync: o número é o custo próprio da Kinein, piso do que o
  usuário sente a 60 Hz, e mede a tecla aparecendo, não o realce assentando.
- Com isso, nenhuma afirmação de responsividade do A3 depende mais de impressão
  visual. A rota do runner `qml` continua fechada com evidência e não deve ser
  retentada.

## Os testes de lógica QML passaram a poder reprovar (2026-07-16)

- Código de saída de processo tem 8 bits: `Qt.exit(256)` sai como 0. Como cada
  harness fazia `Qt.exit(bitmask)`, todo check com bit >= 256 era letra morta —
  falhava e o gate dizia `ok`. Afetava 7 dos 14 harnesses; o
  `tst_multi_terminal` usava bits até 2^31 (~24 checks mortos).
- Corrigido em todos: o bitmask vai para `console.error` (não trunca) e o exit
  só sinaliza passou/falhou. `scripts/verificar-qml-logica.sh` já tratava
  qualquer código != 0 como falha, então o runner não mudou.
- Os checks ressuscitados revelaram um problema, e era do TESTE: o
  `tst_multi_terminal` exigia que abrir um terminal comum mantivesse o contexto
  ativo, contradizendo a própria asserção anterior do arquivo.
  `handleTerminalOpened` termina em `selectTerminal(id)` e está correto.

## Autocomplete: a escolha do usuário para de ser desfeita (2026-07-16)

- Bug confirmado e corrigido: `refilter()` no `EditorCompletionController`
  terminava em `index = 0` incondicionalmente, e `handleResolved()` chama
  `refilter()`. Toda resposta do servidor desfazia a navegação do usuário —
  sintoma "preso no primeiro item". A janela é grande: a A3.2 mediu a primeira
  `completion` do rust-analyzer em 2520 ms, e nesse intervalo o usuário já
  desceu na lista que o fallback local do Tree-sitter abriu.
- A suspeita registrada no `PONTO_ATUAL` (tecla capturada por outra camada)
  estava errada. A cadeia `EditorTextSurface.Keys.onPressed` → `EditorPane` →
  `ShellWorkspaceHost` → `moveCompletion` → `move()` estava íntegra.
- `refilter(preservarSelecao)` separa as causas: resposta do servidor preserva
  (por identidade do `insertText`, não por posição — a lista nova pode vir em
  outra ordem); usuário digitando zera (prefixo e ranking mudaram, o topo volta
  a ser a melhor aposta, como VS Code). Item que sumiu cai para o primeiro:
  seleção fantasma aceitaria item que o usuário não vê.

## Detecção das CLIs de IA e fim do acoplamento a distro (2026-07-16)

- `claude` e `codex` entraram no `KNOWN_TOOLS` do core, **só como detecção**.
  Detectar `claude` é o mesmo que detectar `cargo`: é capacidade, e capacidade
  pode viver no core. Política — como o programa é executado — continua fora:
  nenhum ramo por programa, nenhum `ProfileSpec`, `flat_args` ou filtro.
- `ai_clis_are_detected_exactly_like_any_other_tool` compara o tratamento de
  `claude` com o de `cargo` (mesmo probe, mesma estrutura) e cai se alguém
  escrever `if spec.id == "claude"` no detector.
- **`pacman_package` removido do `ToolSpec`.** O core sugeria `sudo pacman -S`
  só quando `pacman` existia — morto na Fedora. A distinção: detectar é
  agnóstico e lê o PATH; instalar não se deduz do PATH, porque a ferramenta
  ausente é justamente a que não está lá. Regra nova: `install_command:
  Option<&str>`, só para comando canônico e independente de distro (npm nas
  CLIs de IA); `None` para ferramenta de distro. Custo aceito: Arch/CachyOS
  perde a sugestão que tinha.
- `install_suggestion_does_not_depend_on_the_distribution` prova que ter
  `pacman` no PATH não muda mais a resposta.
- A UI ainda não consome `suggested_install` (o campo era só protocolo+testes),
  então a mudança não alterou nada visível. O seletor (§0.2f passos 2–4) segue
  em aberto.

## Split do Main.qml por domínio de fiação — RequestRouter (2026-07-16)

- Motivo: a catraca de arquitetura travou a camada de shell. O seletor do KV
  Context foi bloqueado por ela — `Main.qml` (700/400) e `RuntimeController`
  (494/400) estão em débito e não podem crescer, e são justamente os dois
  arquivos que qualquer feature de shell precisa tocar. A catraca funcionou:
  forçou a arquitetura a ceder antes da feature.
- O padrão já existia pela metade. `ui/qml/ipc/<X>EventRouter.qml` cobria
  `coreClient → controller` (o que o core manda). O sentido inverso,
  `controller → coreClient` (o que a UI pede), não tinha casa e morava no
  composition root. Criado `<X>RequestRouter.qml`, simétrico.
- Feitos: `EditorRequestRouter` (19 pedidos), `RuntimeRequestRouter` (13),
  `DebugRequestRouter` (11). `Main.qml`: 700 → 610 linhas.
- Critério: só pedido ao core entra no router. Fiação de controller para HOST
  (abrir diálogo, focar find bar) não é IPC e fica no `Main.qml`.
- O `RuntimeRequestRouter` não interpreta terminal: repassa gesto cru. Em
  especial `terminalWheel` é transporte — quem decide o que a roda significa é o
  core (`wheel_action`, 0.60.0), conforme o modo VT.
- Limite do padrão, medido: extrair Git (58), Search (45) e ProjectTree (39)
  levaria o `Main.qml` a ~515, ainda acima de 400. O resto é binding de
  propriedade e bloco de host — é trabalho de composition root e não sai por
  router. Abaixo de 400 exige módulos por domínio.
- Armadilha encontrada: `verificar-qml.sh` procura o response file do qmllint em
  `build/linux-clang-debug-strict` ANTES de `build/dev-local`. Reconstruir só o
  dev-local faz o qmllint acusar tipo novo como inexistente — é build velho, não
  erro de código.

## Passo 2 — RequestRouter: até onde o padrão chega (2026-07-16)

- Seis `<X>RequestRouter` extraídos do `Main.qml`: Editor (19 pedidos), Runtime
  (13), Debug (11), Git (15), Search (7), ProjectTree (6). `Main.qml`: 700 → 537.
  Gate verde a cada passo (build dev-local + strict, qmllint estrito, catraca,
  lógica QML).
- Dois routers recebem `editorController` por dependência real, e está
  justificado no arquivo: `GitRequestRouter` (checkout/branch/pull/stash mexem na
  árvore de trabalho e não podem rodar com arquivo sujo — o git sobrescreveria
  edição não salva) e `SearchRequestRouter` (replace reescreve arquivo; symbols
  precisam do buffer ativo, cuja fonte de verdade é a UI). A guarda fica num
  lugar só em vez de espalhada por quem chama.
- **O padrão tem limite, e ele foi medido.** Toda a fiação de IPC saiu (-163
  linhas) e o `Main.qml` ficou em 537: 179 de controllers, 120 de hosts, 64 de
  instanciação de routers, ~170 de Window/Connections/dialogs. Módulos por
  domínio absorvem os 64 e parte dos 179 — piso ~440-470, ainda acima de 400.
  A estimativa original do doc 27 ("fecha abaixo de 400 com folga") estava
  ERRADA e foi corrigida lá (§4.1).
- Razão estrutural: sem registro/contribuição, composition root cresce
  linearmente com o número de domínios. É o que VS Code/Zed/IntelliJ resolvem
  com máquina que este projeto recusou. As 537 linhas não são dívida no sentido
  da §0.2g — é o trabalho que um composition root de IDE com ~12 domínios tem. O
  limite de 400 vem da regra genérica, herdada de quando o arquivo tinha 336
  linhas e não continha a IDE inteira. Decisão em aberto no doc 27 §4.1.
- **Erro de raciocínio corrigido (doc 27 §4.3):** os routers tiraram linhas do
  `Main.qml`, NÃO dos arquivos dos controllers. `RuntimeController.qml` continua
  em 494. Os dois bloqueios do seletor seguem de pé.

## Catraca de arquitetura passou a cobrir o core (2026-07-16)

- `scripts/verificar-arquitetura.sh` varria só `ui/qml` e `ui/src`. A regra §4
  da `ARCHITECTURE.md` (core: quebrar em pasta `<dominio>/` ao misturar
  responsabilidade ou passar de ~400–500 linhas fora dos testes) existia desde
  sempre e nunca havia sido verificada. Medição: 7 arquivos já a violavam.
- Dois não violam só tamanho, e sim a responsabilidade que a própria §4 lhes
  atribui por escrito: `lib.rs` (505; a regra diz "fino, só dispatch e estado")
  e `handlers/lsp.rs` (653; a regra diz "fino, sem lógica pesada").
- Baseline: 20 → 27 arquivos em débito. Congelados, só podem diminuir.
  `terminal.rs` (955) é o maior violador e fica por ÚLTIMO de propósito: é onde
  moram grade VT, cursor/DECSCUSR e `wheel_action`, e custou dois dias de
  estabilização. A catraca o congela sem exigir quebra — quebrar terminal para
  cumprir métrica trocaria risco real por número bonito.
- A contagem corta no `#[cfg(test)]`, como a §4 manda ("linhas de código fora
  dos testes"). Em Rust o teste unitário é co-localizado: contar o arquivo
  inteiro faria a catraca empurrar contra quem testa bem. Na UI não há teste
  inline, então a mesma regra serve às duas camadas sem virar exceção.
  Verificado nos dois sentidos: engordar o código reprova (955 -> 958);
  acrescentar linhas na seção de teste não.
- A mensagem de falha aponta a seção certa por camada: §4 para `crates/`, §6
  para a UI. Apontar a errada faz procurar a regra onde ela não está.

## ARCHITECTURE.md virou leitura obrigatória verificada (2026-07-16)

- Pedido do autor após a constatação de que a §4 do core "existia desde sempre,
  era boa, e nunca tinha sido verificada por ninguém".
- **Achado ao executar:** "obrigatório" JÁ estava escrito. O `AGENTS.md` dizia,
  no item 3 de uma lista de 8, "ler ARCHITECTURE.md — obrigatório", e o item 8
  já mandava estudar Code OSS/IntelliJ/Zed/Lapce/NetBeans como referência
  profissional obrigatória. As duas metades existiam e mesmo assim `lib.rs`
  voltou a 505 e `terminal.rs` chegou a 955, num documento que existe
  explicitamente para impedir a volta do monólito. A palavra não segurou nada;
  o que segurou foi a catraca.
- Por isso a mudança não foi reforçar o adjetivo, e sim: (a) tirar a regra de
  item 3 de uma lista e dar seção própria no topo do `AGENTS.md`, com os números
  medidos; (b) escrever na própria `ARCHITECTURE.md` (§1.1) que ela já falhou
  uma vez e por quê; (c) amarrá-la ao mecanismo que a verifica.
- `ARCHITECTURE.md` ganhou §1.1 (o fracasso medido), §1.2 (manutenção: o que
  muda — inventário e números; o que não muda sem decisão registrada — camadas
  §2, split §4/§6, ordem §5, crescimento §6) e §1.3 (as duas âncoras).
- **As duas âncoras, sempre juntas:** o documento + código medido contra
  ALUCINAÇÃO ("antes de propor, medir": duas vezes seguidas a resposta certa foi
  "o projeto já tem arquitetura, é boa, e não era aplicada"); IDEs open source
  consolidadas com revisão citada contra DOGMATISMO (impede "boa prática"
  inventada virar lei local). Só o documento produz umbiguismo; só as
  referências produzem importação de máquina recusada (Node, Electron, WebView,
  host de extensões). Arquitetura entra como MODE-D/MODE-B.
- Marcado em todos os pontos de entrada: `AGENTS.md` (seção nova no topo + item
  3), `docs/README.md` (índice), `GUIAIA.md` (roteiro e ordem de leitura),
  `docs/CONTRIBUINDO.md` (aviso no topo).

## Terceira âncora: documentação oficial da linguagem/tecnologia (2026-07-16)

- Pedido do autor: a documentação das linguagens/tecnologias usadas passa a ser
  padrão, em conjunto com a documentação do projeto.
- As âncoras da `ARCHITECTURE.md` §1.3 passaram de duas para três, cada uma
  contra um vício: documento + código medido (alucinação de arquitetura); IDEs
  open source consolidadas com revisão citada (dogmatismo); documentação oficial
  de Rust/Qt/QML/C++/CMake/POSIX (**API imaginada**).
- Não é teoria — todos estes custaram tempo neste repositório e são
  comportamento documentado que ninguém consultou antes: `Qt.exit(256)` sai como
  0 (8 bits do código de saída POSIX), e por isso 7 dos 14 harnesses tinham
  checks que nunca reprovavam; `frameSwapped` é emitido na render thread (uma
  conexão queued mediria a fila de eventos junto); `QProcess::start` é
  assíncrono e o `sendRequest` descarta em silêncio o que chega antes de
  `Running`; `QQmlContext::objectForName` existe desde Qt 6.5; o positioner do
  Qt Quick descarta filho de largura zero (os dois dias de TUI).
- Regra prática registrada: ao afirmar comportamento de API, citar fonte e
  versão. Sem fonte consultada, a frase correta é "não sei ainda", e a próxima
  ação é consultar ou medir — nunca supor.
- Marcado em `AGENTS.md`, `ARCHITECTURE.md` §1.3, `GUIAIA.md`.

## Limites acompanham o crescimento: a regra (2026-07-16)

- Exigência do autor: as catracas não podem fazer a realidade de hoje travar o
  desenvolvimento de amanhã.
- Regra registrada em `ARCHITECTURE.md` §4 regra 8: **o limite é por
  responsabilidade e NÃO cresce; o projeto cresce somando unidades, não
  engordando unidades.** Um controller que faz uma coisa não precisa de mais
  linhas porque o projeto ficou maior — precisam existir mais controllers. Um
  projeto com 12 domínios tem 12 arquivos de domínio, não um de 1000 linhas. O
  que escala é a CONTAGEM de módulos. Domínio novo (embarcados, simulação) traz
  arquivos novos e a catraca nem percebe: nascem dentro do limite.
- Exceção legítima: o composition root, cujo tamanho é função do número de
  domínios, não da qualidade do código. Saída: dividir a composição por área,
  fazendo a contagem de arquivos crescer em vez do tamanho — nunca subir o
  limite.
- Válvula explícita: subir um limite É permitido, como decisão registrada e
  justificada. Nunca em silêncio, nunca "porque incomodou hoje" — esse é o vício
  que a §1.1 documenta.
- Opção (b) APROVADA pelo autor: dividir a composição por área. Plano medido e
  pronto em `docs/arquitetura/27-modulos-por-dominio.md` §4.4 (AppDomains.qml,
  204 referências, ordem de execução e armadilha do qmllint).

## Ícone sai de dentro da IDE; fica só no atalho (2026-07-16)

- Decisão do autor: remake do ícone principal (500x500 -> 1254x1254) e, em
  seguida, remover a exibição do ícone DENTRO da IDE. Direção de layout: o visual
  limpo da IDE aberta do JetBrains — inspiração, não cópia; referência de print
  em `imagens/prints`. A janela já é frameless com chrome próprio, então isso é
  decisão de layout, não de sistema.
- Removidos os 4 usos: `main.cpp` (`setWindowIcon` + include `QIcon`),
  `AppMenuBar` (Image da marca + property `brandIconSource`), `StartScreen`
  (Image 48px) e `AboutDialog` (Image 64px).
- **Efeito medido:** `assets/app-icon.png` saiu do `RESOURCES` e o binário release
  caiu de 15.941.200 para 14.720.968 bytes — **1,2 MB a menos**. Verificado que o
  PNG não está mais embutido, não deduzido.
- O que NÃO mudou: o atalho `.desktop` aponta para `imagens/app-icon.png`
  (cópia fora do recurso Qt) e o `install(FILES assets/app-icon.png ...)` do
  ícone hicolor usa o arquivo do source tree. Os dois sobrevivem à remoção do
  recurso; o PNG continua versionado nas duas cópias.
- Consequência aceita pelo autor: sem `setWindowIcon`, a janela/barra de tarefas
  não carrega ícone próprio pelo Qt. No Wayland o ícone vem do `.desktop` via
  app_id (`StartupWMClass=kinein-vectis`).
- `AppMenuBar.qml` encolheu de 303 para 293 linhas — sai do débito da catraca.

## AppDomains: o Main.qml sai do débito (2026-07-16)

- Opção (b) executada. `ui/qml/app/AppDomains.qml` (326) passou a ser dono dos
  14 controllers e dos 14 routers; o `Main.qml` ficou com a janela, os hosts
  visuais, dialogs e atalhos. **`Main.qml`: 700 → 267.** Sai do débito.
- Cruzamento com os hosts entra por propriedade declarada (`workspaceHost`,
  `shellOverlays`, `folderPicker`, `workspaceUiResetter`, `hostWidth/Height`),
  nunca por id global. Dentro do AppDomains as referências entre controllers
  continuam ids simples do mesmo arquivo — não viraram pass-through.
- **Correção de rumo do autor, no meio da fatia:** a catraca não pode empurrar
  para desacoplamento inútil; o que se evita são os DOIS extremos. O gatilho foi
  concreto: com 325 linhas o AppDomains estourava o limite de 300 e eu o quebrei
  em dois, gerando um `AppIpcRouters` com 13 propriedades de pass-through —
  cerimônia pura para satisfazer um número, e "onde X é ligado" passava a ter
  duas respostas. A quebra foi DESFEITA.
- O diagnóstico certo era outro: o limite de 300 é de QML **visual**, e
  `ui/qml/app/` é composição sem um pixel. Era erro de categoria da catraca, não
  gordura do arquivo. `ui/qml/app/` agora vale 400, registrado no script com o
  motivo. Não se subiu limite de ninguém — corrigiu-se a classificação de um
  arquivo que nunca foi visual (§4 regra 8 exige que isso seja explícito).
- Verificado além do build: qmllint estrito limpo e a IDE sobe até o primeiro
  frame (187 ms, offscreen). Refactor de 204 referências não se valida só
  compilando — binding QML quebra em runtime.
- O seletor do KV Context (§0.2f) segue bloqueado pelo `RuntimeController`
  (494/400): separar run configs de terminais é a fatia que falta.

## RunConfigController: o RuntimeController sai do débito (2026-07-16)

- `ui/qml/runtime/RunConfigController.qml` (122) passou a ser dono das
  configurações de execução salvas: modelo, qual está ativa, estado do diálogo e
  do menu. **`RuntimeController`: 494 → 397.** Sai do débito; era o último
  bloqueio do seletor do KV Context.
- A costura é de responsabilidade, não de tamanho (§0.2g): "guardar como rodar
  um programa" não é "manter uma sessão de terminal". O controller novo não
  conhece PTY, id de sessão nem render; o RuntimeController continua dono de
  terminais, KV Context e da execução em si (run.start/stop/stdin), que
  compartilham a aba Terminal.
- Três armadilhas que o COMPILADOR não pega e o qmllint só pegou em parte —
  todas de sinal/propriedade que migrou e ficou sendo escutada no lugar antigo:
  o `RuntimeRequestRouter` ouvia save/delete/setActive no runtimeController; o
  handler de `runConfigDialogOpenRequested` ficou no bloco errado do AppDomains;
  e o `clear()` do reset de workspace ainda fechava diálogo e menu de config.
  Escutar no lugar errado não falha no build: **deixa de funcionar em silêncio.**
- O que só a execução real pegou: `runConfigController` chegava `null` no
  `ShellHeaderHost` e no `ShellOverlays`. qmllint dizia "tudo limpo" e a IDE
  cuspia `TypeError: Cannot read property 'activeConfigId' of null`. Refactor de
  fiação QML se valida rodando, não compilando.
- Verificado: qmllint estrito limpo e a IDE sobe até o primeiro frame (182 ms)
  sem um TypeError.

## Seletor do KV Context: o conceito sai do RuntimeController (2026-07-17)

- O seletor Claude/Codex voltou (§0.2f/§0.2j). `ContextAgentController.qml` (195)
  é dono do **KV Context inteiro**: quem é agente, a escolha, o comando (vindo do
  `path` DETECTADO, nunca de literal), quando a sessão abre, o nome da aba e
  quando o ícone do rail acende. `ContextAgentSelector.qml` (148) é só a escolha —
  o `AssistantPanel` antigo carregava render/input/scroll, um terminal PARALELO,
  e isso não voltou.
- **O corte no `RuntimeController` foi de CONCEITO, não de linhas.** Ele não
  perdeu só as funções de contexto: `grep -i context` nele não devolve mais nada.
  Em vez de blocos específicos ganhou um mecanismo genérico —
  `openLabeledTerminal(label, kind)` carimba rótulo e um `kind` **opaco** na aba,
  devolvido em `terminalOpened(id, kind)`, e ele nunca interpreta o `kind`. A
  fronteira que o §0.2j mandava "decidir com cuidado" se resolveu sozinha: o
  timeout guarda a marca pendente (dele, agora genérica) e ficou; `contextSeq` é
  numeração de KV Context e foi junto. 414 -> 393.
- **`\r` quebrava o build, e não era o `.qml`.** O `qmlcachegen` do Qt 6.11.1
  interpreta o escape e emite o byte CR **cru dentro do `QStringLiteral` gerado**;
  o pré-processador trata o CR como fim de linha e o erro sai como "unterminated
  argument list" 3800 linhas adiante. `\n` é escapado corretamente. O
  `submitShellInput` nunca provou nada sobre isso — ele não é compilado para C++;
  o `\r` só quebrou por cair num handler que é. Medido no `.cpp` gerado.
- **Terceiro caso da regra 9 (`ARCHITECTURE.md` §4).** A catraca reprovou o
  `ShellWorkspaceHost` por **+1 linha**. O defeito não era o arquivo (composition
  host: split é fatia própria) nem a categoria — era **a mudança**, que punha
  política de KV Context num host visual. Devolvida ao dono como
  `contextSessionVisible`, o arquivo caiu 583 -> 579 sem ninguém cortar linha.
  Regra registrada: quando a catraca dispara há três suspeitos — **a sua mudança,
  a categoria, o arquivo** — e o reflexo é olhar só o último, que errou em dois
  dos três casos medidos. E o teste de um corte por responsabilidade não é o
  número: é o **vocabulário**. Se o arquivo ainda nomeia o domínio que você diz
  ter extraído, você moveu código e manteve a responsabilidade.
- **Teste de mutação virou parte do trabalho, não enfeite.** 6 mutações no
  produto, 6 reprovadas pelo `tst_multi_terminal`. Uma sobreviveu na primeira
  tentativa e expôs um check MEU que era mentira: "aba comum não herda o comando"
  passava porque o controller sai cedo no `kind`, não pela limpeza que o check
  dizia exercitar. Cenário real (segunda sessão de contexto sem escolha nova)
  agora coberto. É o §0.2i aplicado antes de o bug nascer.
- **Como testar componente VISUAL real headless** (técnica nova, hoje só no
  scratchpad): os `.qml` do módulo existem em disco em
  `build/dev-local/ui/KineinVectis`, mas o `qmldir` tem `prefer :/KineinVectis/`
  e manda o engine no `qrc`, que o runner não enxerga. Copiar a pasta, remover
  essa linha e apontar `qml -I <copia>` carrega o módulo REAL de disco. Foi assim
  que o `ContextAgentSelector` (Theme/KvIcon/KvButton) rodou fora da IDE.
- Verificado além do build: gesto real dirigido (seletor abre sem PTY, ausente não
  é escolhível, escolha abre a sessão, comando digitado, rail acende), gate verde
  em fmt/clippy/C++/qmllint/catraca/lógica QML. A suíte Rust reprova no flake
  conhecido do §0.2h, que reproduz sem uma linha de Rust tocada.

## "KV Context" vira "Assistente", e ele ganha aba propria (2026-07-17)

- Rename fechado (§0.2d-2). Nome do autor: **Assistente**. Foi ate os internos e
  os docs vivos; `docs-privada/ContextoIA.md`, `docs-privada/diario/` e ADR-0004 ficaram com o nome da
  epoca de proposito — sao LOG, e reescreve-los faria o registro mentir.
- **Duas armadilhas que sed cego teria causado.** O `KvIcon.qml` usa `context`
  como variavel do Canvas 2D (`context.moveTo`): so `case "context":` era o
  icone. E o `tst_multi_terminal` comparava `title === "KV Context 1"` para provar
  que a numeracao nao repete — renomear so o produto deixaria o check comparando
  com string que nunca mais existe: verde para sempre. E' o §0.2i nascendo de um
  rename.
- **Aba propria: a medicao decidiu o desenho.** O autor queria o painel direito
  (spec §4.1 e a UI antiga). Medido que `terminalRenders[id] = render` **nao
  notifica binding** — a IDE desenha so a sessao ATIVA. Painel direito exigiria
  uma segunda vista notificante no RuntimeController; aba inferior nao exige
  nada. O autor escolheu a aba. RuntimeController, TerminalViewport, core, grade,
  roda e cursor: intactos.
- **A traducao da aba mora no dono.** O RuntimeController so pede `"terminal"` —
  ele nao conhece "Assistente" e nao pode conhecer. `AssistantController.tabFor()`
  traduz; o AppDomains aplica. `pendingKind` cobre a janela entre pedir a sessao e
  ela voltar, senao o painel pisca na aba Terminal ate o PTY abrir.
- **Catraca: quarto e quinto casos da regra 9, os dois "a culpa e da MUDANCA".**
  Ela reprovou `ShellWorkspaceHost` (+15) e `BottomPanelHost` (+9). No primeiro eu
  tinha posto politica de "qual sessao pertence a qual aba" num host visual: devolvida
  ao dono como `handleBottomTabClick()`, o arquivo voltou a 581. No segundo o meu
  diff era legitimo — e a medicao achou o real culpado, que nao tinha nada a ver
  comigo: a barra de chips tinha **183 linhas inline**, maior que qualquer painel
  (GitPanel: 53). Extraida como `TerminalSessionTabs.qml` (200), o
  `BottomPanelHost` caiu 550 -> **387** e SAIU do debito (24 -> 23 arquivos).
  O argumento nao foi tamanho: aquele host compoe PAINEIS e cada painel dele ja
  era componente proprio — a barra era a unica inline. Extrair foi aplicar a regra
  que o arquivo ja seguia.
- Mutacao: 3 mutacoes no `tabFor`, 3 pegas — e uma sobreviveu de inicio porque o
  MEU check de `tabFor("git")` rodava com sessao comum ativa, sem exercitar a
  guarda. Movido para depois do agente ativo.

## Assistente: construido, rodado e REMOVIDO no mesmo dia (2026-07-17)

- **Decisao do autor, e e' de produto, nao de arquitetura.** O §0.2f pedia o
  seletor de volta desde 2026-07-16; ele voltou, funcionou, e o uso decidiu:
  "do jeito que foi implementado nao faz sentido usar esse atalho visual, pois o
  usuario pode usar o claude/codex e outros agentes via terminal naturalmente".
  O atalho poupava digitar UMA palavra e cobrava um seletor, um rotulo, uma
  numeracao, uma aba e um icone. Linha de IA na IDE: FORA DE ESCOPO.
- **A licao que fica, e nao e' sobre IA.** A fatia estava arquitetonicamente
  CERTA — a linha detectar/executar do §0.2f foi respeitada, nenhum ramo por
  programa entrou no core, o RuntimeController nem sabia que o conceito existia.
  Passou em todo gate, em mutacao e no gesto real. E era inutil. **Nenhuma
  quantidade de rigor arquitetural torna util uma feature que nao se paga** —
  isso so o uso responde, e responder cedo custou um dia em vez de um trimestre.
- **A remocao foi maior que o codigo removido**: sem a feature, o mecanismo
  generico que eu tinha posto no RuntimeController (`openLabeledTerminal`,
  `kind`, `terminalOpened`) ficou sem usuario. Generico sem usuario e' pior que
  nenhum, entao saiu junto. `RuntimeController`: 494 (ontem) -> 397 (f7f472b) ->
  393 -> **309**. Menor do que jamais foi, e fazendo so uma coisa.
- **O que sobreviveu, e vale mais que a feature:**
  - o `fix` do `coreClient: coreClient` null em 15 roteadores — a IDE nao lia
    pastas nem criava projetos. So foi achado porque a feature obrigou a subir o
    binario novo; o refactor do AppDomains (0686213) estava quebrado desde 23:51
    e ninguem tinha rodado;
  - `scripts/verificar-qml-fiacao.sh` — trava do binding auto-referente;
  - `ARCHITECTURE` §4 regra 9 (3 casos + os tres suspeitos + o teste do
    vocabulario);
  - `TerminalSessionTabs.qml` — o `BottomPanelHost` caiu 550 -> 383 e saiu do
    debito.
- **Fica no core, de proposito:** `claude`/`codex` no `KNOWN_TOOLS`, como
  `cargo`/`clangd`. Nao e' assistente: e' o painel Ferramentas dizendo se o
  binario esta no PATH, e e' MAIS util agora que o fluxo e' o terminal direto. O
  teste `ai_clis_are_detected_exactly_like_any_other_tool` sobrevive e continua
  guardando o core contra politica por programa.
- **Resto conhecido:** `assistant_terminal_width` ainda existe em
  `kinein-config`/`settings.rs` e no protocolo (sobra do 0.59.0). A UI nao le
  mais. Tirar do core e' mudanca de contrato com decisao de versao: fatia propria.

## Cursor/TUI aprovado — e o aceite apagou duas fatias do roadmap (2026-07-17)

- **Aceite do autor:** o cursor e a renderizacao da TUI do claude/codex via
  terminal estao RESOLVIDOS. Fecha o sintoma que custou dois dias e duas fatias.
- A causa-raiz ja estava achada e corrigida em `ab3becc` (07-16 16:53): a linha
  vazia colapsava no positioner do Qt (`Column` descarta filho sem largura), o
  TEXTO escorregava para cima e o cursor — posicionado por `yForRow`, a grade
  matematica — nunca esteve errado. O registro do roadmap 26 dizia "reprovado no
  gesto humano" desde ANTES da correcao e ninguem tinha revalidado.
- **O ganho maior nao foi o bug: foi o roadmap que sumiu.**
  - **R3 (trocar o renderer Qt) DECIDIDO por evidencia: opcao 1, fica o QML.** O
    criterio estava escrito no proprio R3 — "manter se R2 corrigir o visual e
    cumprir orcamento de frame/memoria". Corrigiu e cumpre (A3). As opcoes 2
    (`QQuickItem` nativo) e 3 (componente Qt maduro) existiam para o caso de o
    modelo por span continuar divergindo. Nao continuou.
  - **R2 vira CONDICIONAL**: a razao que o criou era o sintoma. O R1 ja media que
    em DPR 1 ele e' nulo (~0,11px). So se justifica se uma MEDICAO em DPR != 1
    mostrar divergencia.
- **Licao de processo, ja registrada no 26 e agora confirmada:** o R0 exigia tres
  evidencias — tabela, fixture e SCREENSHOT — e dizia "sem isso, parar". As duas
  primeiras foram tratadas como suficientes, confirmaram hipoteses reais (H1, H2)
  que NAO eram a causa, e custaram a fatia R1 inteira. A captura resolveu em um
  minuto. Evidencia listada como obrigatoria e' gate, nao sugestao.

## Docker e banco de dados: nativos, de primeira classe (2026-07-17)

- **Decisao do autor.** Docker e banco de dados serao integrados de forma NATIVA,
  como "cidadaos de primeira classe" — dominios do core como `git`/`lsp`/
  `terminal`, nao plugins de terceiro. Implementar do zero, e o autor aceitou
  explicitamente que e' demorado. Coerente com a arquitetura ja escolhida: sem
  registro dinamico, sem host de extensoes, tudo compilado junto e auditavel.
- Os dois lados sao necessarios e nao se confundem: o `integration` v1 (L1) e' a
  PLATAFORMA (descriptor/health/config — como a coisa aparece para o usuario); o
  dominio nativo e' a FUNCIONALIDADE. Sem o L1 a IDE nao sabe DIZER que Docker
  existe; sem o dominio ela nao sabe FAZER nada com ele. O L1 segue sendo o
  gargalo mesmo com a decisao de nativo.
- **Docker pertence ao L5 (RemoteContext), nao a um nivel proprio.** Container e'
  contexto remoto: filesystem proprio, path mapping, ciclo de vida, execucao fora
  do host, falha por desconexao. Mesmo contrato do SSH — `container://` ao lado de
  `ssh://`. Nivel proprio duplicaria o RemoteContext inteiro.
- **FATO VERIFICADO NA FONTE, e derruba a premissa do pedido:** o **IntelliJ IDEA
  Community NAO tem Database Tools** — e' exclusivo do Ultimate (DataGrip e'
  produto pago a parte). Nao ha UI de banco no Community para estudar. Isso nao
  cancela o objetivo; troca a fonte: o idioma VISUAL do Community continua sendo a
  referencia, mas a referencia FUNCIONAL de cliente de banco e' o DBeaver (a
  auditar pelo gate). O plugin Docker, esse, e' instalavel no Community — da' para
  estudar o comportamento, so nao vem de fabrica.
- **UI/UX: divida continua e explicita.** Registro do autor: a UI/UX da IDE tem
  muito a ser otimizado/polido, com o IntelliJ IDEA Community como referencia —
  "essa UI/UX da JetBrains e' muito agradavel/confortavel para o desenvolvimento".
  A regra que ja e' contrato (§2.1) vale inteira: **nao e' copiar e colar**.
  Importa-se invariante, decisao, modo de falha e estrategia de teste; nunca
  codigo, runtime ou IntelliJ Platform/Swing. Onde o contexto da Kinein diverge
  (C/C++/Rust, offline-first, sem host de extensoes, frameless), vence o contexto
  da Kinein. Adaptacao realista e pragmatica.
- Ordem registrada: **C/C++/Rust solidos (L2-L4) ANTES de Docker e banco.** L2-L4
  e' profundidade no que ja existe; Docker e banco sao superficie nova. Uma IDE
  com Docker e sem cobertura de teste e' uma demo. Detalhe em
  `docs/roadmaps/28-plataforma-de-plugins-e-verticais.md`.

## Pente fino no core: E1 fechado e dois defeitos de dados (2026-08-29)

Varredura de duplicacao e logica em `crates/`, com a toolchain fixada
(`rust-toolchain.toml` → 1.96.1) instalada na maquina. `cargo fmt --check`,
`cargo test --workspace --all-features` e `cargo clippy -D warnings` verdes ao
fim. Os gates de C++/QML **nao** foram rodados nesta sessao (exigem Qt6 e preset
CMake configurado, ausentes na maquina); nada de C++/QML foi alterado.

**E1 (§0.2h) fechado — e o diagnostico registrado em 2026-07-16 estava meio
errado, o que importa mais que o fix.** O arquivo dizia que faltava "fechar o
descritor antes do exec (ou `O_CLOEXEC`)". Medido na fonte: `fs::write` ja fecha
ao retornar e o Rust ja abre com `O_CLOEXEC` no Linux. A janela real e' **entre
`fork` e `execve`** — o filho herda uma copia do descritor de escrita, e o
`O_CLOEXEC` so age quando o `execve` DELE tiver sucesso; ate la o arquivo esta
"aberto para escrita" e o `execve` do script recusa com `ETXTBSY` (`execve(2)`,
secao ERRORS, Linux man-pages 6.9). Nenhum lock entre os testes de `tools` fecha
essa janela — por isso o escopo do `EXEC_LOCK` **nao** cresceu, como o §0.2h
mandava. Fecharam a corrida: retry limitado em `ETXTBSY` no `probe_version`
(20 x 10 ms; condicao transitoria que existe em producao tambem — linker
gravando `target/debug/foo`, gerenciador de pacotes atualizando binario) e um
`exec_lock()` que ignora envenenamento, matando a cascata de `PoisonError` que
era a segunda metade do §0.2h. Provado por mutacao: com `EXEC_BUSY_ATTEMPTS = 1`
e o `unwrap()` de volta, os dois testes novos reprovam **e o
`fd_detection_accepts_fdfind_binary_name` reproduz o `PoisonError` original** — a
flake de 2026-07-16, agora deterministica.

**Defeito de dados 1: a suite de testes escrevia no estado GLOBAL real do
usuario.** `enable_lsp` ligava `persistence_enabled` por tabela e
`workspace/recent.rs` deduzia o caminho XDG sozinho, entao TODO teste que
abrisse um workspace gravava em `~/.config/kinein-vectis/recent-workspaces.json`.
O arquivo tem teto de 12 entradas: uma execucao de `cargo test` despejava a lista
de projetos recentes. Medido nesta data no arquivo real do autor: **12 de 12
entradas eram `/tmp/kinein-core-tests`, zero projetos reais** — a lista dele ja
tinha sido perdida antes desta sessao, e o dado nao volta. Correcao estrutural,
nao cosmetica: persistencia passa a ser gesto EXPLICITO
(`Core::enable_persistence(global_storage)`, chamado so pelo `run_stdio`) e a
raiz do estado global e' sempre RECEBIDA, nunca deduzida dentro do modulo — as
funcoes viraram `*_recent_workspaces_in(dir, …)`. O padrao seguro deixou de
depender de disciplina. Coberto por
`tests::workspace::test_core_never_writes_to_the_real_global_storage`, que
compara o diretorio global antes/depois.

**Defeito de dados 2: store de rascunhos orfa na troca de workspace.**
`self.workspace` mudava em tres caminhos (`open`, `createProject`, `close`), cada
um com a sua copia da lista de estado por-workspace, e as copias divergiram: so o
`open` trocava `self.drafts`. Consequencia — depois de criar um projeto pelo
assistente, o autosave do projeto NOVO ia para o `.kinein/kinein.db` do projeto
ANTERIOR; sem projeto anterior, `draft.save` respondia "persistencia local de
rascunhos indisponivel". A rede de seguranca de dados (docs/seguranca/23)
desligava em silencio, com build, clippy e 271 testes verdes. Correcao: um dono
unico da transicao (`activate_workspace` / `deactivate_workspace` em
`handlers/workspace.rs`) e **gate novo** `scripts/verificar-transicao-workspace.sh`,
que reprova qualquer `self.workspace`/`self.drafts` mexido fora do dono. Gate
testado por mutacao nos dois sentidos (handler alheio mexendo em `drafts`;
`createProject` voltando a atribuir direto).

**Terceiro achado do mesmo modulo:** `recover_drafts` tratava "arquivo ausente"
como "difere do rascunho" e oferecia de volta o buffer de um arquivo que o
usuario tinha APAGADO. Um rascunho so nasce contra arquivo existente
(`fsops::confine_file`), entao arquivo sumido = rascunho obsoleto. Fica ABERTO:
`fs.rename` nao MOVE o rascunho (ele e' descartado); mover pede operacao nova na
store, fatia propria.

**Duplicacao com risco real (nao cosmetica):** `fs.search` e `fs.replace` tinham
o walk de diretorios COPIADO, e as copias **ja divergiam** — diretorio ilegivel
abortava o `replace` inteiro e era apenas pulado no `search`. Isso e' grave
porque o usuario le o resultado da busca e manda substituir: conjuntos diferentes
significam a IDE editando arquivo que ela nao mostrou. Extraido `fsops/walk.rs`
(um dono), com teste de PARIDADE explicito
(`replace_touches_exactly_the_files_search_reports`). O resto da "duplicacao"
apontada pelo detector e' a forma canonica de handler fino (§4 regra 2) e de
cabecalho de dialogo QML — colapsar isso em macro pioraria a leitura, e nao foi
tocado.

**Codigo morto removido:** `ToolDetector::find_binary` era o ultimo resquicio do
aiBridge no codigo, com doc citando "AI CLI Bridge" — linha declarada FORA DE
ESCOPO em 2026-07-17. Zero chamadores.

**Ferramenta:** `scripts/instalar-ambiente.sh` instalava `stable` enquanto o
`rust-toolchain.toml` fixa `1.96.1`. Baixava uma toolchain que ninguem usa e
deixava a de verdade para o primeiro `cargo build` — exatamente a "surpresa
offline" que o bloco dizia evitar. Agora le o canal do TOML.

## As tres arvores de documentacao e o gate de links (2026-08-29)

Decisao do autor: separar a documentacao em `docs/` (lida em toda sessao),
`docs-privada/` (continuidade interna) e `docs-legada/` (superado/cancelado),
**com a regra de que uma sessao le `docs/` e mais nada**.

**Isto contraria uma decisao registrada, e por isso fica registrado aqui.** Em
2026-07-05 a `docs/archive/` foi removida com a regra "nao recriar uma pasta de
arquivo so para guardar". Aquela decisao continua certa para o que ela era:
deposito de material historico que documento ativo nenhum referenciava. O caso
de agora e' outro, e e' pior — havia **2380 linhas em duas specs COMPLETAS**
(`ASSISTANT_AI_ASSISTANCE`, `AI_CLI_BRIDGE_EXTERNAL_TERMINAL`) descrevendo
painel de IA, chat, seletor de agente e aba dedicada, tudo **cancelado em
2026-07-17**, morando dentro de `docs/specs/`. O aviso `⛔ FORA DE ESCOPO` no
topo de cada uma nao segura ninguem: quem chega pelo indice ou pelo grep nao le
o topo. Deletar perderia o registro da decisao de produto — que e' a parte cara;
o "o que" se reescreve numa tarde, o "por que nao" nao. A terceira saida e'
separar a arvore, com criterio ESTREITO: **so entra o que uma sessao poderia
confundir com alvo.**

Foram legados 6 itens (as 2 specs de IA + o SVG do KV Context, o
`17-architecture-hygiene-plan` — fase concluida cujos numeros envelheceram 3,4x
e enganaram uma sessao — e o `PLANO_ORGANIZACAO_E_HANDOFF`, que descreve o
estado ANTERIOR a reorganizacao de 2026-07-16). Antes de legar o handoff, as
faixas P/T/X foram EXTRAIDAS para `docs/README.md`, seguindo a propria regra de
2026-07-05: extrair o que tem valor, depois arquivar.

Para `docs-privada/` foram o `ContextoIA.md` (este arquivo), o `diario/` e os
`prompts/`. Mover o log para fora da arvore de leitura e' a decisao de
2026-07-17 (rebaixar o log na precedencia) levada ate o fim: quem so quer
trabalhar nao passa mais por 2414 linhas de registro datado; quem precisa do
"por que" vem aqui de proposito.

**Gate novo: `scripts/verificar-links-docs.sh`.** `git mv` nao atualiza link
nenhum, e este repositorio ja moveu documentacao por `git mv` uma vez
(2026-07-16: 21 arquivos, 410 referencias reescritas a mao) e moveu de novo
agora. Link morto em Markdown nao tem compilador: o gate fica verde e o proximo
a ler conclui que o documento nao existe. Foi testado por mutacao (apontar um
link do `docs/README.md` para arquivo inexistente reprova) ANTES de a
reorganizacao comecar — foi ele que guiou os 5 links que a mudanca quebrou.

Efeito colateral util: o gate expos que o `GUIAIA.md` ainda chamava o
`ContextoIA.md` de "estado real" em 4 lugares — descricao derrubada em
2026-07-17 e que sobreviveu 6 semanas dentro do mapa que as sessoes usam para se
orientar. Corrigido no mesmo gesto.

**Tambem nesta data:** `docs/LEITURA_TECNICA.md`, pedido pelo autor como
"leitura tecnica e explicita do projeto" para dar direcao. Classe ESTADO, todo
numero medido e datado. Ele responde numa leitura o que hoje exige abrir dez
documentos, e o achado mais acionavel que ele expoe e' medido: **`terminal.rs`
e' o maior arquivo do core (1374 linhas), o maior debito da catraca, e o unico
dominio grande com ZERO teste de integracao** — nao existe `tests/terminal.rs`.
