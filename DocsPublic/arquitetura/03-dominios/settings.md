# `settings.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

### Settings (`settings.get` / `settings.set`)

Implementado no protocolo `0.36.0` (fatia M4.1). Dois níveis:
GLOBAL (`$XDG_CONFIG_HOME` ou `~/.config`, então `kinein-vectis/
settings.json`, vale para todos os workspaces) e por-WORKSPACE
(`.kinein/settings.json`, sobrepõe o global campo a campo). Ambos com
`schemaVersion` (arquivo inválido/schema desconhecido = vazio, nunca
quebra). O efetivo = `default ← global ← workspace`.

**O arquivo que a IDE não entende (`0.220.0`, INF-07, INF-11 e INF-12).**
Até o `0.219.0`, o arquivo que a IDE não entendia valia como vazio, e o
primeiro `settings.set` daquele escopo o trocava pelo dele, sem aviso. Desde
o `0.220.0`, o `SettingsResult` leva `globalProblem` e `workspaceProblem`,
cada um presente só quando o arquivo daquele escopo existe e não foi lido, com
o caminho e o motivo: não é arquivo regular, passa de 1 MiB, não está em
UTF-8, não é JSON válido (com a linha e a coluna), não é um objeto, não tem
`schemaVersion`, tem um `schemaVersion` que esta versão não conhece, ou um
valor conhecido de tipo errado. Os valores daquele escopo vêm vazios, como
antes, e o efetivo cai para o escopo de baixo. O `settings.set` daquele escopo
é recusado com `INVALID_REQUEST` e o mesmo motivo, e o arquivo fica como
estava. No escopo do workspace, o `.kinein` e o `settings.json` que são link
simbólico também são recusados, e nada é gravado fora do projeto (INF-12); o
global segue o link, porque é da pessoa e pode vir de dotfiles, e recusa o
alvo que não é arquivo regular. Num arquivo entendido, a escrita guarda as
chaves que esta versão não conhece, com os valores delas (INF-11), em vez de
apagá-las. Veja o [62 §3.4](../../roadmaps/62-base-publica-0.4.md#34-inventário-sistemático-de-dívida-técnica).

```text
settings.get {} → SettingsResult
settings.set { scope: "global"|"workspace", values: SettingsValues }
             → SettingsResult
SettingsValues { formatOnSave?: bool, editorFontSize?: u32,
                 autoClosePairs?: bool, autoSave?: bool (0.123.0),
                 rigorProfile?: "strict"|"balanced"|"relaxed",
                 explorerWidth?: u32, contextWidth?: u32,
                 assistantTerminalWidth?: u32,
                 bottomPanelHeight?: u32, outlineWidth?: u32,
                 outlineCollapsed?: bool, railExpanded?: bool (0.128.0),
                 compilerCheckOnSave?: bool (0.175.0),
                 grafanaWebView?: bool (0.154.0, só global),
                 platformioWebView?: bool (0.191.0, só global),
                 layout?: Layout (0.146.0) }
                                          (campos ausentes = não setados)
EffectiveSettings { formatOnSave, editorFontSize, autoClosePairs, autoSave,
                    rigorProfile, explorerWidth, contextWidth,
                    assistantTerminalWidth,
                    bottomPanelHeight, outlineWidth, outlineCollapsed,
                    railExpanded, compilerCheckOnSave (0.175.0),
                    grafanaWebView (0.154.0), platformioWebView (0.191.0),
                    layout? (0.146.0) }
Layout (schemaVersion 1) { schemaVersion: 1,
                 leftWindow?: "explorer"|"git", leftVisible?: bool,
                 sizes?: { explorer?, outline?, bottom? }   (PREFERIDOS, px)
                 outlineCollapsed?: bool,
                 bottom?: { visible?: bool, tab?: string } }
SettingsResult { settings: EffectiveSettings, global: SettingsValues,
                 workspace: SettingsValues,
                 globalProblem?: string, workspaceProblem?: string (0.220.0) }
```

- `settings.get` NÃO exige workspace (o global existe sempre; sem
  workspace, `workspace` vem vazio). Padrão de mutação do repo: `set`
  responde o estado COMPLETO novo.
- `settings.set` faz MERGE parcial de `values` no escopo (campo ausente
  mantém o valor gravado; reverter/limpar override é pós-v1). Defaults:
  `formatOnSave=false`, `editorFontSize=14`, `autoClosePairs=true`,
  `autoSave=true` (0.123.0, decisão do projeto na Etapa 2),
  `compilerCheckOnSave=true` (0.175.0: a checagem do compilador ao salvar
  C/C++ com CDB, decisão do projeto de 2026-10-15, `roadmaps/52` §13.1),
  `grafanaWebView=false` e `platformioWebView=false` (as páginas web
  dentro da IDE: o Grafana local, 59 §6.1, e o PIO Home, 52 §14.45). Só o
  global decide essas duas, e o projeto não opina. Qualquer uma ligada
  faz a IDE compartilhar o contexto OpenGL na abertura (~13 ms e ~9 MiB,
  59 §7.1.6), o que o QtWebEngine exige. A UI lê essa decisão antes do
  core subir (`ui/src/startup_web.cpp`), só do arquivo global e com as
  regras de leitura do core: segue o link, recusa o que não é arquivo
  regular sem abri-lo, o que passa de 1 MiB e o que não tem `schemaVersion`
  1 (INF-14, 2026-11-12; antes ela abria também o `.kinein/settings.json`
  do projeto da linha de comando, e um FIFO ali prendia a partida). `rigorProfile="strict"`, larguras `280/360/640/220` (Project, seletor KV,
  terminal KV ativo e Estrutura), painel inferior `260` e Estrutura expandida.
- Erros: `NO_WORKSPACE` (set `scope=workspace` sem workspace);
  `INVALID_PARAMS` (`editorFontSize` fora de 8..=40, ou `rigorProfile`
  fora do enum; largura/altura de painel fora dos limites, inclusive terminal
  KV ativo fora de 300..=720); `INVALID_REQUEST` (0.220.0: o arquivo do
  escopo existe e a IDE não o entende, o mesmo motivo do `globalProblem` ou
  do `workspaceProblem`); `INTERNAL_ERROR` (falha de escrita).
- Consumidores atuais: `editorFontSize` → `Theme.fontSizeEditor`;
  `autoClosePairs` → auto-close da E1; `formatOnSave` → Ctrl+S formata e
  então salva; `autoSave` → o `EditorPersistenceController` salva o buffer
  sujo sozinho (2 s de pausa; ao trocar de aba; ao perder o foco) pelo
  MESMO caminho do Ctrl+S (`fs.write` com `expectedContent` — o disco
  mudado por fora continua recusado); o rascunho do `seguranca/23` segue
  gravando aos 1,5 s como rede entre um salvar e outro; `rigorProfile` (M4.5) → flags de `quality.run`/`build.run`
  NO PROJETO DO USUÁRIO. Rust: clippy pedantic/nursery e `-D warnings` no
  strict; clippy default no balanced; só `clippy::correctness`, e o build
  sem `RUSTFLAGS`, no relaxed. C/C++ com CMake (desde 2026-10-03, 40.7
  §7.202): `CMAKE_COMPILE_WARNING_AS_ERROR=ON` no strict, `OFF` no balanced
  (o projeto decide), e `OFF` mais `--compile-no-warning-as-error` no relaxed.
  O build reconfigura quando o perfil anotado em `.kinein/build/.kinein-rigor`
  difere do atual (`crates/kinein-core/src/cmake/rigor.rs`). Python: as
  regras do ruff por perfil, quando o projeto não declara as dele. O perfil
  NUNCA regula o gate do repositório Kinein.
  As dimensões persistem o layout que o usuário redimensionou;
  `outlineCollapsed` persiste o recolhimento da Estrutura.
- **`layout` (0.146.0)** é o retrato do shell, por workspace: a UI grava no
  escopo do workspace aberto (sem workspace, no global, que vira o padrão de
  quem ainda não tem o seu). `sizes` são os tamanhos **preferidos**: o exibido
  passa pelos limites do `ShellController` (mínimo declarado pelo conteúdo,
  máximo que deixa o editor com 480 px), e a janela menor não reescreve o
  preferido. `bottom.tab` nunca restaura processo: a aba Terminal volta vazia
  até o usuário pedir um shell. `railExpanded` fica fora do layout: é
  preferência do usuário, sempre no global. `contextWidth` e
  `assistantTerminalWidth` não têm consumidor na UI e não entram no layout.
  `diffBase` (head|index) fica para uma micro-fatia futura (precisa de
  `base` no `git.fileDiff`). `order` (2026-10-02, aditivo no schema 1, sem
  mudar o protocolo — o core guarda o `layout` cru) é a ordem que o usuário
  arrastou em cada barra: `{ "bottom": ["tools", "terminal", …], "rail":
  […], "railRight": […], "header": […], "statusLeft": […], "statusRight":
  […] }`; a UI ignora chave que não conhece e põe item novo no fim.
  `rail.sides` (mesma data) guarda o trilho de cada área que o usuário
  arrastou para o outro lado: `{ "outline": "left" }`.
