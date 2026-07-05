# ContextoIA - Continuidade do Kinein Vectis

Este arquivo registra decisoes de produto/arquitetura para IAs que continuarem
o desenvolvimento do repositorio. Use junto de `AGENTS.md` e dos documentos em
`docs/`.

## Direcao do produto

- Produto: Kinein Vectis.
- Objetivo: IDE open source, Linux-first, rigida por padrao, visualmente
  familiar para usuarios de IDEs JetBrains, mas com identidade propria.
- Uso inicial: projeto de uso proprio do autor, com foco em qualidade alta e
  ergonomia diaria.
- Filosofia: a IDE orquestra ferramentas maduras; nao reimplementa compilador,
  LSP, debugger, build system ou analisadores quando ja houver ferramenta aberta
  consolidada.

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
- Abrir outro workspace:
  - se for o mesmo root, pode preservar abas abertas;
  - se for outro root, a UI deve limpar abas e estado visual do workspace
    anterior.
- Fechar projeto deve fechar automaticamente abas abertas e limpar explorer.

## Estado tecnico atual

- Arquitetura: Qt/QML UI <-> JSON-RPC local/stdin-stdout <-> Rust core.
- Core Rust:
  - `core.ping`
  - `core.shutdown`
  - `command.list`
  - `tools.detect`
  - `tools.status`
  - `workspace.browse`
  - `workspace.open`
  - `workspace.status`
  - `workspace.close`
  - `fs.list`
  - `fs.read`
  - `fs.createFile`
  - `fs.createDirectory`
  - `fs.write`
  - `fs.rename`
  - `fs.delete`
  - `fs.findFiles`
  - `build.run`
  - `test.run`
  - `quality.run`
  - `lsp.didChange`
  - `lsp.definition`
  - `lsp.hover`
  - `lsp.completion`
  - `lsp.references`
  - `lsp.rename`
  - `fs.search`
  - `run.start`
  - `run.stdin`
  - `run.stop`
  - `terminal.open`
  - `terminal.input`
  - `terminal.close`
  - `lsp.semanticTokens`
  - `job.list`
  - `job.cancel`
- Protocolo IPC atual: `0.19.0`.
- **Job System (2026-07-05):** `kinein-core/src/jobs/` tem um `JobManager` que roda
  operacoes longas de forma assincrona (retorna id na hora, emite
  `event.job.created/progress/output/finished`, cancel via `JobContext`).
  `job.list`/`job.cancel` expostos. Ver `docs/ARCHITECTURE.md` §7.
- **`build.run` migrado para job assincrono/cancelavel:** responde na hora com
  `{ jobId }`; o build roda em background emitindo `event.build.*` (com `jobId`,
  para Problems/build tool window) + `event.job.*` (status bar); `job.cancel` mata
  o processo de build (`process::stream_command_lines_cancelable`, com drain
  limitado para netos nao travarem o retorno). MUDANCA DE CONTRATO: build.run nao
  retorna mais `BuildRunResult` na resposta; o resultado vem em
  `event.build.finished`. `quality.run`/`test.run` seguem SINCRONOS (proximos a
  migrar).
- `workspace.browse` lista subdiretorios para o seletor proprio da UI. Ele
  canonicaliza caminhos e retorna `{ path, parent, entries }`.
- `fs.*` continua confinado ao workspace aberto e nao deve ser usado para
  navegar fora do workspace. `fs.createFile` cria arquivo novo e
  `fs.createDirectory` cria diretorio novo, ambos sem sobrescrever caminho
  existente; `fs.write` continua salvando apenas arquivos ja existentes.
  `fs.rename` renomeia/move e `fs.delete` remove (recursivo para diretorios),
  ambos confinados a raiz e recusando a propria raiz do workspace.
- `fs.findFiles` usa `fd` para busca de arquivos por nome, sem indexador
  proprio. Falta de `fd` retorna `TOOL_NOT_FOUND`.
- `build.run` executa a ferramenta de build pelo core e emite
  `event.build.started/output/diagnostic/finished`.
- `lsp.didChange` sincroniza o buffer aberto com o LSP gerenciado pelo core.
  Eventos `event.lsp.status` e `event.lsp.diagnostics` alimentam a UI.
- `lsp.definition` e `lsp.hover` fazem navegacao semantica minima via LSP,
  sempre passando pelo core e usando posicao 1-based do editor.
- A UI sobe `kinein-core` como processo filho via `CoreClient`.
- O usuario testa normalmente pelo icone "Kinein Vectis" ja instalado no
  menu de aplicativos. Esse icone executa `scripts/kinein-vectis`.
- O launcher `scripts/kinein-vectis` prefere:
  - UI release: `build/linux-clang-release-hardened/ui/kinein-vectis`;
  - core release: `target/release/kinein-core`;
  - e so cai para debug se os binarios release nao existirem.
- Depois de alterar core/UI, atualizar os binarios usados pelo icone com:
  `cargo build --release -p kinein-core` e
  `cmake --build --preset dev-local-release`.
- CORRECAO (2026-07-03): nesta maquina (Pop!_OS), o CONFIGURE do CMake deve
  usar os presets locais `dev-local` / `dev-local-release` de
  `CMakeUserPresets.json` (fora do git). Eles herdam os presets oficiais e
  apontam o clang para `--gcc-install-dir=.../gcc/x86_64-linux-gnu/13`,
  porque o diretorio GCC 14 do sistema esta incompleto (sem libstdc++) e
  quebra o link. Em outras maquinas, usar os presets oficiais.

## Modularizacao pos-V1 (2026-07-05, Opus)

- Fase de "monolito modular" do doc 15 executada: os arquivos-monolito do core,
  protocolo e CLI foram quebrados por responsabilidade, um commit atomico por
  arquivo, com gate completo (test + clippy estrito + fmt) verde entre passos e
  superficie publica preservada. Sem mudanca de comportamento.
- Novas pastas no core: `src/lsp/` (types/manager/server/framing/parse/edit/uri),
  `src/fsops/` (error/confine/ops/search/find), `src/workspace/`
  (error/detect/open/create) e `src/tests/` (por dominio). `handlers/` ja existia.
- `kinein-protocol/src/` agora tem um modulo por dominio (rpc, command, core,
  tools, workspace, fs, run, terminal, lsp, build) re-exportado flat — os
  consumidores continuam usando `kinein_protocol::TipoX`.
- `kinein-cli` ganhou um lib target (`kinein_cli`) com `commands`/`error`;
  `main.rs` virou shim fino sobre `kinein_cli::run`.
- Rename kernwerk -> Kinein Vectis CONCLUIDO (2026-07-05): crates renomeados para
  `kinein-*` (imports `kinein_*`), dir de dados `.kernwerk` -> `.kinein`, UI/Qt
  (target `kinein-vectis`, modulo QML `KineinVectis`, namespace C++ `kinein`,
  `KineinStrictOptions.cmake`), launcher `scripts/kinein-vectis`, schemas e docs
  ativos. Verificado: gate Rust (147 testes) + configure/build completo do UI Qt.
- Docs reorganizados: os specs canonicos da Kinein Vectis estao em `docs/specs/`;
  os docs era-kernwerk superados foram para `docs/archive/legacy/`.
- Arvore de arquivos atualizada em `docs/02-repository-structure.md`; conclusao
  registrada em `docs/15-engineering-debt-and-refactor.md`.

## Strict mode

- Rust deve continuar com o maximo rigor:
  - `unsafe_code = "forbid"`;
  - warnings como erro;
  - docs/debug impls obrigatorios onde configurado;
  - clippy pedantic/nursery;
  - sem `unwrap`, `expect`, `panic`, `todo`, `dbg!` fora de casos aceitos por
    testes existentes.
- C++/Qt usa C++23, warnings-as-errors, sanitizers em Debug e hardening/LTO em
  Release via `cmake/KineinStrictOptions.cmake`.
- Futuramente deve existir seletor de nivel de rigidez:
  - Strict como padrao;
  - Balanced;
  - Relaxed apenas por escolha explicita.

## Prioridade imediata

Estabilizar UX/UI basica antes de avancar para features grandes:

1. Fluxo de abrir/fechar workspace. [feito]
2. Explorer previsivel (arvore expande/recolhe). [feito]
3. Abas e salvamento. [feito]
4. Layout arredondado, denso e tecnico inspirado no mockup em
   `imagens/layout-mockup.png`. [feito]
5. Top bar, sidebar de icones, paineis inferior e direito. [em andamento]
   - Sidebar de icones (esquerda): Projeto (mostra/oculta explorer),
     Ferramentas e Logs (abrem o painel inferior). [feito 2026-07-03]
   - Painel inferior com abas "Logs" e "Ferramentas". A aba Ferramentas usa
     `tools.detect` (status, versao e sugestao pacman por ferramenta) com
     botao de redeteccao. [feito 2026-07-03]
   - Painel direito (Assistente KW): casca visual pronta (icone ✦ na sidebar,
     header com badge "offline", historico de mensagens e input). SEM provider
     de IA ligado; resposta e um aviso honesto sobre a Fase 7 e a politica de
     confirmacao. Nao fingir IA funcionando. [feito 2026-07-03]
6. Polir a arvore Project para a sensacao limpa/confortavel JetBrains-like.
   - Reduzir peso visual do painel e das linhas.
   - Evitar visual de explorer pesado/fechado estilo VS Code.
   - Manter diretorio como expandir/recolher, nao trocar workspace.
   - Preferir icones pequenos e tooltips futuros para acoes.

## DECISAO DE PRIORIDADE (usuario, 2026-07-03)

O usuario decidiu: UI/UX fica no nivel "basico/funcional" atual; a prioridade
agora e integrar ferramentas para usar a IDE como ambiente de desenvolvimento
real. Ordem acordada:

1. Syntax highlighting no editor. [FEITO 2026-07-03]
2. Fase 4 - Build. [FEITO 2026-07-03 — ver sessao abaixo; cancelamento e
   run/execucao do binario ainda pendentes]
3. Fase 5 - LSP: diagnosticos MVP, go to definition e hover implementados
   para clangd e rust-analyzer. O core sobe servidores como processos
   gerenciados e traduz LSP para IPC. Nesta sessao `clangd` foi encontrado em
   `/usr/bin/clangd`; ainda nao assumir que `rust-analyzer` esta instalado.
   Completion (Ctrl+Space), find usages (Alt+F7) e rename (Shift+F6)
   entregues na Fase 5.2 [FEITO 2026-07-03]; semantic tokens e code actions
   continuam pendentes. [MVP EM ANDAMENTO]
4. Git (Fase 6) na sequencia. A Fase 5 passa a ser tratada como base LSP
   suficiente para C/C++ e Rust, sem tentar fechar todos os recursos avançados
   de IDE profissional antes de avançar.
5. Decisao atualizada (2026-07-04): reformular Fases 7-9 para foco de curto
   prazo em C/C++ e Rust. Java/Python saem do curto prazo e ficam para pos-V1
   ou retomada futura. IA nao deve virar sistema complexo de providers agora:
   o usuario consegue usar Claude/Codex/GPT pelo terminal; a necessidade da
   IDE e uma aba/terminal visualmente separado para chat de IA, separado do
   terminal geral usado para comandos do projeto.
6. Decisao de produto pos-V1 (2026-07-04): adicionar no futuro "modos de
   compilador" e uma "loja de funcoes" para C/C++ e Rust. Isso deve permitir
   ativar perfis mais rigidos ou menos rigidos de compilador/quality sem
   decorar flags. A loja deve ordenar funcoes por confianca: ISO/Standard
   primeiro em C/C++; Rust oficial primeiro em Rust; depois diagnosticos
   oficiais, ferramentas maduras, presets Kinein Vectis, regras locais e opcoes
   experimentais. A experiencia deve ser visual e JetBrains-like: janela de
   opcoes do ambiente do projeto com nome da funcao, explicacao simples,
   impacto, risco, fonte, previa de alteracoes e reversao. Nao bloquear a
   V1.0 com isso. Ver
   `docs/16-compiler-modes-and-function-store.md`.

## Visao pos-V1.0 (usuario, 2026-07-03)

Depois da V1.0 o foco e polir a IDE continuamente para chegar o mais perto
possivel das IDEs JetBrains em analise, navegacao e refatoracao — mesmo
orquestrando ferramentas abertas (CMake, LSPs etc.). O objetivo explicito e
NAO terminar como "um VS Code": a régua de qualidade de navegacao/refactoring
e JetBrains. Isso reforca a Fase 5 (LSP) como investimento central: semantic
tokens, go-to-definition, find usages, rename via LSP, e futuramente acoes de
refatoracao proprias por cima do que os LSPs oferecem.

## Logs de sessão (arquivados)

O registro narrativo, sessão a sessão, do desenvolvimento de 2026-07 foi
movido para [docs/archive/contextoia-session-logs-2026-07.md](docs/archive/contextoia-session-logs-2026-07.md)
para manter este arquivo focado em estado atual, decisões vigentes e próximas
prioridades. Consulte o arquivo apenas se precisar do histórico detalhado.

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
  tecnico atual" + sessoes datadas.
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
  via API num primeiro momento. Implementacao fica para a Fase 7 (o usuario
  disse "deixa mais pra frente" os detalhes).
- LOGS: o log de tráfego IPC/acoes da IDE e ferramenta de desenvolvimento DA
  IDE, nao do usuario final. Por isso a aba do painel inferior foi renomeada
  para "IDE". No fluxo real de projeto, "Logs/Problemas" devem mostrar erros
  DO PROJETO (build, diagnosticos) — essas abas nascem nas Fases 4/5.
- ERROS DA IDE EM .TXT: qualquer erro/mau funcionamento da IDE (crash do
  core, stderr do core, falha de processo, resposta IPC invalida, core nao
  encontrado) e gravado com timestamp ISO em:
  `~/.cache/kinein-vectis/logs/kinein-ui-erros.txt`
  (implementado em `CoreClient::appendErrorLog`; caminho exposto ao QML pela
  propriedade `errorLogFile`). Segue o diretorio de logs previsto em
  docs/07-tooling-lifecycle.md.

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

- `ui/src/core_client.h` e `ui/src/core_client.cpp`: cliente IPC da UI. Nao
  criar outro cliente IPC paralelo.
- `ui/qml/FolderPickerDialog.qml`: seletor proprio de workspace. Nao voltar
  para `QtQuick.Dialogs` e nao criar outro seletor de pasta duplicado.
- `ui/qml/Main.qml`: layout principal, sidebar, explorer, editor, painel
  inferior e estado visual (`showExplorer`, abas, logs/ferramentas).
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
- Atualizar `docs/03-ipc-protocol.md` e schemas quando mudar contrato IPC.
- Manter docs sincronizadas quando comportamento de workspace/editor mudar.
