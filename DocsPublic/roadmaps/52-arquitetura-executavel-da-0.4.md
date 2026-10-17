# 52 — Arquitetura executável da 0.4: embarcados com compiladores e ecossistemas por baixo

> **Classe: PLANO / ARQUITETURA.** Escrito em 2026-10-01, a pedido do autor,
> depois do adendo do [roadmap 45 §7](45-etapa4-backend-lsp-edicao-compiladores.md).
> Nada aqui está implementado por estar escrito. O código e o
> [roadmap 40](40-estado-e-continuidade.md) prevalecem para o estado real; este
> documento prevalece para o **desenho** de cada fatia da 0.4, e cada sessão que
> implementar uma fatia relê a seção dela antes do código.
>
> **Por que este documento existe:** o autor pediu o desenho e o fluxo nos
> mínimos detalhes, porque arquitetura de qualidade facilita o código, evita
> bugs e **evita código duplicado**. O núcleo é a §3: para cada pergunta que a
> 0.4 precisa responder, **um dono que já existe** no código, o que se
> acrescenta a ele e o que é proibido criar ao lado.
>
> **Fontes de escopo:** [45 §7](45-etapa4-backend-lsp-edicao-compiladores.md)
> (tabela fluxo → fatias), [49 §5](49-frontend-0.3.6-e-sequencia-0.5.md) (o que
> a 0.4 herda da casca 0.3.6), [42](42-trilha-profunda-embarcados.md) (pilares
> P0–P7 e o "efeito JetBrains" medido), as especificações do
> [modelo semântico](../especificacoes/modelo-semantico-do-projeto-0.4.md), do
> [cache de compilação](../especificacoes/cache-de-compilacao.md) e de
> [embarcados](../especificacoes/embarcados-targets-flash-serial-qemu.md).
>
> **Onde cada fatia começa no código** (arquivos reais, o mecanismo a
> estender, arquivos no limite que precisam ser divididos antes):
> [`58`](58-onde-cada-versao-comeca-no-codigo.md) §4.4.
>
> **Decisões do autor de 2026-10-01 que ampliam a 0.4:** o **PlatformIO é
> cidadão de tier 1** — as cinco jornadas abaixo valem para ele, não só o
> compiledb (§5.10) — e os **emuladores entram como alvo** (§10.1).

## 0. Como ler e como usar

- A §1 diz o que o usuário consegue fazer ao fim da 0.4. A §2 diz o que não
  se quebra. A §3 é a regra anti-duplicação e o mapa de donos.
- As §§4–6 são o modelo (contexto efetivo), os fluxos passo a passo e os
  contratos IPC. Cada fluxo diz, em cada passo, **quem é o dono**.
- As §§7–10 são UI, concorrência, estados de erro e provas. A §11 é a ordem;
  a §12, os pontos de rollback; a §13, o que ainda é decisão do autor.
- **Nomes marcados *(proposto)*** não existem no código hoje. Todo nome sem
  essa marca foi conferido no checkout em 2026-10-01.

## 1. Resultado ao fim da 0.4

Cinco jornadas, provadas com ferramentas reais e, onde houver, com a placa do
autor (sem gravar sem pedido):

```text
J1 ABRIR      abrir um projeto de firmware (CMake cross, ESP-IDF, Zephyr,
              pico-sdk, PlatformIO, Cargo embarcado ou Makefile de fabricante) e ver, sem
              editar JSON: framework, alvo, kit, toolchain efetiva, o que falta
              e o passo oficial para resolver
J2 ENTENDER   salvar um .c/.cpp/.rs e ver o erro do compilador do ALVO na linha
              (sem build completo); perguntar "por que este arquivo compila
              assim" e ver compilador, padrão, defines, includes, sysroot,
              target — de onde cada valor veio
J3 CONSTRUIR  build cross repetível e rápido (cache quando houver), com o
              tamanho por região, seção e SÍMBOLO, e a diferença para o build
              anterior
J4 GRAVAR     escolher a sonda pela lista, gravar e abrir o monitor, pelo
              mesmo caminho de "Executar" (configuração de execução)
J5 DEPURAR    depurar no alvo (sonda, OpenOCD, emulador — QEMU, Renode — ou
              gdbserver remoto), ver e ESCREVER registradores de periférico
              pelo SVD
```

O editor acompanha: signature help, destaque de ocorrências, formatação pelo
servidor, progresso de indexação e servidor avisado de mudança em
`CMakeLists.txt`/`compile_commands.json`.

## 2. Invariantes

Herdadas do [48 §2](48-arquitetura-executavel-da-serie-0.3.md) e do
[45 §5](45-etapa4-backend-lsp-edicao-compiladores.md), sem reabrir:

1. **O core decide fatos; o QML apresenta e envia intenção.** QML nunca
   executa ferramenta, nunca interpreta saída de compilador, nunca guarda um
   valor efetivo próprio.
2. **Operação longa é Job.** Tem `jobId`, progresso, cancelamento por
   `job.cancel` e saída bruta acessível. Nenhum domínio cria cancelamento
   paralelo.
3. **Mutação de projeto passa por Configuration Actions** (`configAction.list/
   preview/apply`): prévia, consentimento, escopo. Nenhum painel escreve arquivo
   do projeto por conta própria.
4. **Executar, gravar e depurar são configurações de execução** (`runConfig.*`,
   `run.start`, `debug.start`). Gravar não é domínio novo (decisão de
   2026-09-11 registrada em `flash/mod.rs`).
5. **Detectar é agnóstico e tem um dono só:** `ToolDetector`
   (`crates/kinein-core/src/tools/`). Ninguém faz `Command::new` de descoberta
   fora dele.
6. **Qual binário cumpre cada papel tem um dono só:** o domínio `toolchain`
   (kit por projeto, `.kinein/toolchain.json`, schema 2).
7. **A IDE detecta deriva, não resolve versão.** Quem resolve é `cargo`,
   `west`, `idf.py`, `conan`; a IDE roda por Job e explica.
8. **Nada roda `sudo`, nada instala em silêncio**, a placa do autor nunca é
   gravada sem pedido explícito.
9. **Evidência ou dica, nunca silêncio:** toda dedução carrega a fonte
   (arquivo, comando, versão); o que não se decide vira `hint`.
10. **Régua de desempenho:** latência da tecla (`KINEIN_PERF_TYPING*`) e
    primeiro frame não pioram; toda fatia mede antes e depois.
11. **Protocolo versionado:** cada mudança de contrato sobe `PROTOCOL_VERSION`
    e é registrada em `arquitetura/03-protocolo-ipc.md`.
12. **O artefato distribuído é a prova final:** `testar-appimage.sh` e
    `testar-appimage-portatil.sh` passam com a lista `scripts/avisos-qml.txt`
    (lição de 2026-10-01, 40 §7.148: o Qt do pacote não é o do checkout).

## 3. A regra anti-duplicação e o mapa de donos

### 3.1 A regra

> **Antes de criar um tipo, um método, um módulo, um controller ou um painel,
> nomeie o dono atual da pergunta.** Se ele existe, a fatia **estende** o dono.
> Um dono novo só nasce quando a pergunta é nova, e então a fatia registra
> por que nenhum dono existente serve.

Isso é a mesma disciplina que a 0.3 aplicou ("procurar implementação e
estrutura existentes; ampliar os donos atuais"), agora escrita como mapa para
a 0.4 inteira.

### 3.2 O mapa de donos

| Pergunta | Dono hoje (core · IPC · UI) | O que a 0.4 acrescenta | Proibido criar |
| --- | --- | --- | --- |
| Que ferramenta existe nesta máquina, e em que versão? | `tools::ToolDetector`, `tools/known.rs`, `tools/search_dirs.rs` · `tools.detect`, `tools.status` | Entradas novas em `KNOWN_TOOLS` (ccache, sccache, bear, west, idf.py, pio) | Outro scanner, `which` espalhado, detecção dentro de handler |
| Qual binário cumpre cada papel neste projeto? | `toolchain` (`catalog.rs`, `store.rs`, `arguments.rs`) · `toolchain.get/set/setKit` · `ToolchainController.qml`, `EmbeddedKitField.qml` | Papéis novos no catálogo (`compilerLauncher` *(proposto)*, `sizeTool` já coberto por prefixo) | Um segundo "kit" em outro arquivo, cópia de escolha em QML |
| O que é este projeto (framework, SDK, alvo, artefatos)? | `project` (`detect.rs`, `sdk.rs`, `artifacts.rs`, `esp.rs`) · `project.model` | Map file e SVD no modelo de artefatos; ecossistemas novos só como detecção com evidência | Detecção de framework em outro domínio, palpite sem fonte |
| Com que flags este arquivo compila? | `index/context` (`cdb.rs`, `cargo.rs`), `cmake/model.rs` (`compileGroups`, sysroot) · `index.context` | Campo de origem por valor (CDB, file-api, kit) e o sysroot/target efetivos | Parser próprio de `CMakeLists.txt`, segunda leitura da CDB |
| A CDB existe, está velha, onde? | `cdb.rs` (crate raiz) · via `index.context`/`cmake.status` | Geração por Bear para Makefile (Configuration Action) | Outro detector de `compile_commands.json` |
| Como configurar e construir? | `cmake/mod.rs` (diretório único `.kinein/build`), `build/` (`engine.rs`, `parse.rs`, `make.rs`, `frameworks.rs`) · `cmake.configure`, `build.run` | Preset no configure automático; toolchain file gerado em `.kinein/` | Outro diretório de build, outro executor de build |
| Que diagnóstico existe neste arquivo? | `build/parse.rs` (gcc-like), `lsp/diagnostics_merge.rs`, `protocol::DiagnosticSource {Build, Quality, Lsp, Toolchain}` · `DiagnosticsController.qml`, `ProblemsPanel.qml` | `DiagnosticSource::Compiler` *(proposto)* fundido pelo mesmo merge | Segundo parser de saída de compilador, segunda lista de problemas |
| Quanto o binário ocupa? | `size.rs` (`size -A`, regiões `MEMORY` do `.ld`) · `build.size` · `EmbeddedSizeView.qml` | Por símbolo (map file / `nm --size-sort`) e delta contra o build anterior | Outra leitura de linker script, outro painel de tamanho |
| Como gravar? | `flash/` (`mod.rs`, `firmware.rs`, `frameworks.rs`) · `runConfig.flashProposal`, `run.start` · `EmbeddedFlashView.qml` | Sonda escolhida na lista entra na linha; config OpenOCD deduzida de VID:PID | Domínio "flash" com execução própria, botão que roda fora de `run.start` |
| Quem está plugado? | `probe.rs`, `serial/` · `probe.list`, `serial.list/identify/monitor` | Seleção de sonda gravada no kit | Abrir porta serial para "descobrir" (reseta a placa) |
| Como depurar? | `dap/` (`session.rs`, `adapter.rs`, `target.rs`, `gdb_pick.rs`) · `debug.*` (inclusive `debug.readMemory`, `debug.disassemble`) | `debug.writeMemory` *(proposto)* e o modelo SVD | Outro cliente de gdb, leitura de memória fora do DAP |
| O que o editor sabe da linguagem? | `lsp/` (`manager.rs`, `session.rs`, `sync.rs`, `registry.rs`) · `lsp.*` · `EditorLanguageController.qml`, `LspStatusController.qml` | `lsp.signatureHelp`, `lsp.documentHighlight`, `lsp.formatting`, progresso e arquivos observados *(propostos)* | Segunda sessão LSP, cliente LSP em QML |
| O que falta instalar, e como? | `setup/` (`catalog.rs`, `catalog_embedded.rs`, `distro.rs`), `toolchain/install` · `setup.list`, `toolchain.install/installable` · `SetupPanelHost.qml` | Entradas novas no catálogo, com fonte oficial e SHA-256 | Download ou `sudo` escondido |
| O que o usuário quer mudar no projeto? | `configaction/` · `configAction.*` · `ConfigActionController.qml` | Ações novas: habilitar cache, gerar CDB por Bear, gerar toolchain file | Escrita direta de arquivo por painel |
| Saúde do projeto, aviso e próximo passo | `ProjectHealthController.qml`, `project.model` (`hint`) | Avisos de deriva (§5.9) pelo mesmo canal | Faixa de aviso paralela |
| Ação disponível pelo teclado e pela paleta | `CommandDispatcher`, `command.list` | Comandos novos por id | Segunda paleta, atalho fora do gate de atalhos |

### 3.3 Padrões que já existem e são reusados, não reescritos

- **Job + evento + saída bruta:** `jobs/`, `JobsController.qml`.
- **Leitura pura testável com fixture:** o padrão de `size.rs`, `probe.rs`,
  `project/detect.rs`: o parser recebe texto e devolve tipo; o processo fica
  fora. Toda saída de ferramenta nova segue isso.
- **Ferramenta falsa no teste integrado:** o padrão `rsync_falso` de
  `tests/remote_mirror.rs` e o `mpremote` falso do `verificar-python-debug.sh`.
- **Store versionado:** `schemaVersion` explícito; desconhecido = vazio
  (`toolchain/store.rs`, `runconfig.rs`, `settings.rs`).
- **Fusão de diagnósticos por fonte:** `lsp/diagnostics_merge.rs`.
- **Prévia + consentimento:** Configuration Actions.
- **Moldura de painel comum:** `KvPanelFrame` e família.

### 3.4 Checklist de cada fatia (vai na descrição do commit)

```text
[ ] dono nomeado para cada pergunta da fatia (linha da §3.2)
[ ] `rg` pelo conceito antes de criar tipo/método/arquivo; resultado citado
[ ] método novo só se nenhum método existente comporta um campo novo
[ ] parser puro com fixture real da ferramenta (versão anotada)
[ ] Job para tudo que pode passar de ~100 ms
[ ] erro e cancelamento provados, não só o caminho feliz
[ ] harness QML para estado/intenção; teste integrado para IO
[ ] medida de tecla/primeiro frame antes e depois
[ ] AppImage: smoke com avisos-qml.txt
[ ] registro em 40 §7 e, se mudou contrato, 03-protocolo-ipc.md
```

### 3.5 Sinais de alerta na revisão

Reprovar e reescrever quando aparecer: `Command::new` de descoberta fora do
`ToolDetector`; regex de saída de compilador fora de `build/parse.rs`; leitura
de `compile_commands.json` fora de `cdb.rs`/`index/context`; um `property`
em QML que guarda flags, toolchain ou alvo "para mostrar depois"; um painel
novo que repete o que o Environment, o Problems ou o Jobs já mostram; um
botão que roda processo sem passar por `run.start`, `build.run` ou Job.

## 4. O modelo central: contexto efetivo, derivado e nunca copiado

### 4.1 Problema

Hoje a resposta "com o quê este arquivo compila" está espalhada em fontes
corretas, mas separadas: o kit (`toolchain`), o modelo do projeto
(`project.model`), o modelo do CMake (`cmake/model.rs`) e a unidade de
compilação (`index.context`). A UI precisa mostrar **um** contexto efetivo com a
origem de cada valor (configurado, detectado, selecionado, efetivo, como pede o
[49 §4 F3](49-frontend-0.3.6-e-sequencia-0.5.md)), sem que alguém copie esses
valores para um store novo.

### 4.2 Decisão

O **contexto efetivo é computado no core, sob demanda, a partir dos donos**, e
não é persistido. Ele é a **extensão do `index.context`**, que já é a pergunta
"com que este arquivo é compilado", e não um domínio novo.

```text
                 toolchain store (kit)     project.model     cmake model     CDB
                        │                        │               │            │
                        └──────────┬─────────────┴───────┬───────┘            │
                                   ▼                     ▼                    │
                          index/context  ◄────────────────────────────────────┘
                                   │   (compõe; não guarda)
                                   ▼
                     index.context { path } → FileContext + origem por valor
```

### 4.3 Forma *(proposta)* — extensão do `FileContext` existente

```text
FileContext (existe)            + effective (proposto)
  path, language                  compiler      { value, origin }
  unit: CompileUnit?              standard      { value, origin }
  cargo: CargoUnit?               target/triple { value, origin }
  python: PythonEnv?              sysroot       { value, origin }
  targets[], source?, hint?       defines[]/includes[] com origem agregada
                                  kit           { key, name }
                                  drift[]       (§5.9)

origin ∈ { cdb, cmakeFileApi, kit, cargoMetadata, projectModel, default }
```

Regras: o valor vem **do dono** (CDB para flags reais; kit para escolha; file-
api quando não há CDB). A origem é **dado**, não texto livre. Campo novo é
opcional e retrocompatível. Nenhum consumidor recalcula isso em outro lugar.

## 5. Fluxos, passo a passo

Convenção: cada passo diz **dono**, entrada, saída e falha.

### 5.1 J1 — Abrir um projeto embarcado (zero-config)

```text
1  UI    usuário abre pasta (menu, drop ou `kinein`)            → workspace.open
2  core  handlers/workspace.rs activate_workspace (ordem JÁ medida no 42 §8.4):
         project.model → índice (job) → args do clangd (query-driver do kit)
3  core  project/detect.rs: framework com evidência (arquivo que prova)
4  core  toolchain store: kit do projeto; vazio → padrão por papel (PATH +
         search_dirs); kit sem sysroot para triple cross → hint
5  core  cmake.status: não configurado → ProjectHealthController pede
         cmake.configure (já existe). NOVO: com UM preset não oculto, o
         configure automático usa esse preset (42 §8.1 "falta (a)"); com
         vários, pergunta pela faixa de saúde, sem adivinhar
6  core  configure gera CDB em .kinein/build (CMAKE_EXPORT_COMPILE_COMMANDS)
7  core  setup.list: o que falta, com passo oficial
8  UI    header mostra alvo/kit/toolchain efetiva (contrato 49 F3);
         Environment mostra detalhe; nada exige JSON
falha    configure falha → aviso com saída bruta no Job e ação "Configurar";
         framework ambíguo → hint com as duas evidências; nunca escolha calada
```

### 5.2 J2a — Diagnóstico do compilador ao salvar (C1)

```text
gatilho  fs.write bem-sucedido de .c/.cc/.cpp/.cxx/.h* com unidade na CDB
dono     build (execução) + index/context (flags) + diagnostics_merge (fusão)

1  core  ao confirmar a escrita, agenda "checar unidade" por arquivo com
         debounce de 400 ms (salvar várias vezes não enfileira várias)
2  core  index/context resolve a CompileUnit do arquivo (a MESMA usada pelo
         clangd); header sem unidade própria → a unidade de um .c/.cpp que o
         inclui não é adivinhada nesta fatia: header fica só com o LSP
3  core  monta a linha: o comando da CDB com -fsyntax-only, sem -o, sem -M*,
         -fdiagnostics-format=json quando o compilador aceita (gcc>=10,
         clang aceita -fdiagnostics-format=sarif/json conforme versão — medir),
         senão o formato clássico que build/parse.rs já casa
4  core  Job curto por unidade; um novo salvamento CANCELA o anterior da mesma
         unidade (job.cancel interno) — resultado velho nunca sobrescreve novo
5  core  parse pelo dono existente (build/parse.rs); DiagnosticSource::Compiler
         (proposto); diagnostics_merge funde com Lsp por arquivo
6  UI    DiagnosticsController/ProblemsPanel já mostram a união; nada novo
Rust     o mesmo gatilho para .rs chama `cargo check --message-format=json`
         (cargo.check já existe), por pacote, com o mesmo cancelamento
falha    compilador ausente → diagnóstico Toolchain com o passo do setup;
         timeout (10 s) → Job falha com saída bruta; nunca "sem problemas"
medida   tempo salvar→diagnóstico (mediana/p95) e tecla durante o job
```

### 5.3 J2b — "Por que este arquivo compila assim" (C2)

```text
dono     index/context (§4); UI: área Ambiente da 0.3.6 (49 F3), sem painel novo
1  UI    comando "Contexto de compilação do arquivo" (paleta e menu do editor)
2  core  index.context { path } com effective + origem
3  UI    inspector: cada valor com a origem; "abrir a CDB na linha da unidade";
         "abrir o kit" (mesmo dono de edição: ToolchainController)
4  UI    diferença configurado × efetivo destacada (ex.: kit pede sysroot X,
         CDB mostra --sysroot Y → deriva, §5.9)
falha    arquivo fora de qualquer unidade → diz isso e por quê (sem CDB,
         fora dos targets, header)
```

### 5.4 J3a — Toolchain cross ponta a ponta (C3)

```text
dono     toolchain (kit) + cmake (configure) + configaction (arquivo gerado)
1  core  kit com triple cross e sysroot; preset sem toolchain file
2  core  Configuration Action "gerar toolchain file" (proposta): escreve
         .kinein/toolchains/<kit>.cmake (CMAKE_SYSTEM_NAME, C/CXX compiler,
         CMAKE_SYSROOT, FIND_ROOT_PATH_MODE_*) — prévia mostra o conteúdo
3  core  configure com -DCMAKE_TOOLCHAIN_FILE quando o preset não declara um
         (regra que já existe para o arquivo do SDK)
4  core  build.run; build.size; flash proposal — tudo pelo mesmo kit
prova    arm-none-eabi (Arm GNU ou xPack do provedor existente) e
         aarch64-linux-gnu da distro, projeto mínimo, foto e número no 40
falha    sysroot sem usr/include → toolchain.inspectSysroot já dá veredito;
         a ação não é oferecida sem sysroot válido
```

### 5.5 J3b — Tamanho por símbolo e delta (C7, estende `build.size`)

```text
dono     size.rs (hoje: size -A + regiões MEMORY do .ld) · build.size
1  core  NOVO: com o map file do linker (-Wl,-Map) quando existir no build/,
         ou `nm --print-size --size-sort -C` pelo prefixo do kit
2  core  parser puro: símbolo → seção → região, tamanho
3  core  guarda o resumo do último build em .kinein/size-last.json
         (schemaVersion) para o delta; desconhecido = sem delta
4  UI    EmbeddedSizeView ganha "maiores símbolos" e "mudou desde o build
         anterior"; nada de painel novo
falha    sem map nem nm → mostra só seções (o que já existe) e diz por quê
```

### 5.6 J3c — Cache de compilação (C5)

Segue a [especificação de cache](../especificacoes/cache-de-compilacao.md),
fatia C1 dela:

```text
dono     tools (detecção ccache/sccache) + toolchain (papel compilerLauncher,
         proposto) + configaction (habilitar) + cmake (configure)
1  core  ToolDetector encontra ccache/sccache com versão
2  UI    Configuration Action "usar cache de compilação" com prévia: o kit
         ganha compilerLauncher; o configure passa
         -DCMAKE_<LANG>_COMPILER_LAUNCHER (não toca CMakeLists.txt)
3  core  estatísticas por `ccache -s --print-stats` / `sccache --show-stats`
         em Job, parser puro, exibidas no Environment
falha    launcher configurado e ausente → deriva (§5.9) com a ação de remover
```

### 5.7 J1b — Makefile de fabricante sem CDB (C4)

```text
dono     cdb.rs (detecção) + configaction (gerar) + tools (bear)
1  core  sem CDB e com Makefile → hint "gerar compile_commands.json"
2  UI    Configuration Action com prévia do comando (`bear -- make -n`?
         medir: bear precisa do build REAL; -n não gera) e do efeito
3  core  roda como Job; clangd reiniciado (lsp.restart existe) ao terminar
falha    bear ausente → setup.list com o passo oficial; nunca instalar
```

### 5.8 J4/J5 — Gravar e depurar no alvo

```text
gravar   dono: flash (linha) + runConfig (salvar) + run.start (executar)
1  core  probe.list dá VID:PID:serial; NOVO: a escolha vai para o kit
         (campo de sonda, proposto) — persistência no dono do kit
2  core  OpenOCD: interface deduzida do VID:PID (0483:* stlink, 1366:* jlink,
         2e8a:000c cmsis-dap) como DEDUÇÃO com evidência; o usuário confirma
3  core  runConfig.flashProposal já compõe a linha; nada roda fora de run.start
depurar  dono: dap
4  core  debug.start com o servidor do kit (OpenOCD/QEMU/probe-rs/gdbserver)
5  core  SVD: parser em crate (svd-parser, MIT/Apache, conferir na fatia)
         dentro de project (artefato) ou dap (uso) — decisão na fatia, um dono
6  core  periférico → registrador → campo: leitura por debug.readMemory
         (existe); escrita por debug.writeMemory (proposto) com releitura
         confirmando o valor
7  UI    painel de periféricos dentro da área Embarcados; campo editável
falha    escrita não confirmada pela releitura → mostra o valor lido e o erro;
         sem SVD → diz de onde obter (CMSIS-Pack do fabricante, licença lida)
regra    gravar na placa do autor só com pedido explícito na sessão
```

### 5.9 Deriva — detectar, explicar, nunca resolver

```text
dono     index/context (§4) compõe; project.model e toolchain fornecem
exemplos kit pede sysroot X e a CDB foi gerada com Y; launcher no kit e
         ausente na máquina; triple do kit ≠ triple do compilador detectado;
         alvo Rust do kit não instalado (toolchain.get já mostra instalados)
saída    drift[] com { kind, configured, effective, evidence, action? }
UI       ProjectHealthController (mesmo canal de hint) e o inspector da §5.3
proibido "corrigir" sozinho; a ação é sempre Configuration Action ou passo
         oficial do ecossistema
```

### 5.10 Ecossistemas

```text
Rust embarcado   rust-analyzer recebe o alvo do kit (rust-analyzer.cargo.target
                 pela inicialização do servidor; dono: lsp/registry + arguments);
                 probe-rs pelo caminho do §5.8; cargo decide dependências
Zephyr           west como ferramenta detectada; build pelo build/frameworks.rs
                 (dono existente); SDK por importKit (42 §8.5 "falta")
ESP-IDF          idf.py e a receita flasher_args.json (já lidos em project/esp.rs)
pico-sdk         CMake por baixo; nada próprio além da detecção que já existe
PlatformIO       TIER 1 (decisão do autor, 2026-10-01): as cinco jornadas.
                 J2: `pio run -t compiledb` como Configuration Action (C4);
                 J3: `pio run` (existe), `pio test` como runner e `pio check`
                 como provider de qualidade; J4: `pio run -t upload` (existe);
                 J5: o `.elf` do env como alvo de debug e o servidor do
                 `debug_tool` do `platformio.ini`. O que existe e o que falta,
                 por arquivo: 58 §4.4
MicroPython      mpremote (já usado em run.script/serial.files)
```

### 5.11 Editor que entende o alvo (L2, L3, L5, L6, L7)

```text
dono     lsp/manager.rs (consultas), lsp/session.rs (wire), lsp/sync.rs (texto);
         UI: EditorLanguageController.qml (quem já pede ações e hover)
L2  lsp.signatureHelp { path, line, column } (proposto) — pedido ao digitar
    '(' e ',' com debounce; popup fecha no ')' ou Esc; nunca bloqueia a tecla
L3  lsp.documentHighlight (proposto) — cursor parado 200 ms; resposta velha
    (versão do documento mudou) descartada pela versão já usada no sync
L5  lsp.formatting / rangeFormatting (propostos) — "formatar ao salvar" por
    linguagem; format.text continua para ruff/rustfmt quando o servidor não
    formata (dono format.rs); o plano de edição é aplicado pelo mesmo caminho
    de lsp.workspaceEdit (confinado)
L6  $/progress → LspStatusController (existe) mostra "indexando… 43%"
L7  didChangeWatchedFiles alimentado pelo fswatch existente para
    CMakeLists.txt, Cargo.toml, compile_commands.json
fora L4, E6, L8/L9, E4/E5 (45 §7)
```

## 6. Contratos IPC

Regra: **campo novo em método existente antes de método novo.** Cada linha
abaixo é proposta; a fatia confirma o nome contra o código e sobe a versão.

| Contrato | Tipo | Dono | Observação |
| --- | --- | --- | --- |
| `index.context` + `effective`, `drift` | campo novo | index/context | retrocompatível; §4 |
| `DiagnosticSource::Compiler` | enum novo | protocol/diagnostic | fundido em diagnostics_merge |
| gatilho de checagem ao salvar | interno + `event.build.diagnostics`? | build | medir se o evento existente serve antes de criar outro |
| `build.size` + `symbols[]`, `delta` | campo novo | size.rs | §5.5 |
| `toolchain.setKit` + `compilerLauncher`, `probe` | campo novo | toolchain store (schema 3) | migração 2 → 3 com fallback |
| `configAction` ids: `toolchain.generateFile`, `cache.enable`, `cdb.generateWithBear` | entradas no catálogo | configaction | sem método novo |
| `debug.writeMemory` | método novo | dap | espelha `debug.readMemory` |
| `project.model` + `svd`, `mapFile` em artefatos | campo novo | project/artifacts | |
| `lsp.signatureHelp`, `lsp.documentHighlight`, `lsp.formatting`, `lsp.rangeFormatting` | métodos novos | lsp/manager | um por pergunta do servidor |

`toolchain.json` schema 3 *(proposto)*: acrescenta campos opcionais ao kit;
schema 2 abre como 3 com os campos vazios; schema desconhecido = vazio (regra
do store).

## 7. UI: onde cada coisa aparece, sobre a casca da 0.3.6

- **Header:** alvo, kit e toolchain efetiva como chips do contrato F3 da 0.3.6;
  clique abre o dono (ToolchainController/Environment), não um editor novo.
- **Área Embarcados** (contextual no trilho da 0.3.6): Placa · Projeto ·
  Gravar · Kit, que já existem, mais Tamanho (estendido) e Periféricos (novo,
  dentro da mesma área).
- **Ambiente/Inspector:** contexto de compilação (§5.3) e deriva (§5.9).
- **Problems:** diagnósticos do compilador ao salvar, fundidos; nenhuma lista nova.
- **Jobs:** checagem por salvamento, Bear, cache stats.
- **Editor:** signature help, highlight, formatação, progresso (§5.11).
- **Não entra:** um ícone por ferramenta, um painel por provider (a Library da
  0.5 é quem organiza descoberta).

## 8. Concorrência, cancelamento e desempenho

- Uma fila por unidade de compilação para C1; salvar cancela a anterior.
- Resposta com versão de documento/arquivo antiga é descartada (mesmo
  mecanismo do sync LSP e do `requestedPath` do Remote).
- Nenhuma checagem na UI thread; QML só recebe eventos.
- Orçamentos: tecla sem regressão; C1 mediana < 1,5 s num TU médio do projeto
  de prova (medir e registrar; se não couber, ajustar o debounce e não a régua).
- Contagem de processos: nada de spawn por arquivo ao abrir; detecção em lote
  e em cache (já é o padrão do ToolDetector).

## 9. Estados e erros

Vocabulário único em texto e ícone, nunca só cor: **configurado, detectado,
selecionado, efetivo, desconhecido, desatualizado, falhou, indisponível**.

| Situação | O que o usuário vê | Ação |
| --- | --- | --- |
| Compilador do kit ausente | diagnóstico Toolchain + passo oficial | setup.list |
| CDB velha (cdb.rs já sabe) | "desatualizada: <motivo>" | Configurar |
| Checagem ao salvar falhou | Job falhou com saída bruta | reabrir o Job |
| Sem sonda | "nenhuma sonda" só quando `probe-rs list` respondeu; senão "não foi possível listar" | setup |
| Escrita de registrador não confirmada | valor relido + erro | repetir |
| Deriva | origem dos dois valores | Configuration Action |

## 10. Provas

```text
unidade        parsers puros com fixture REAL anotada com versão
               (gcc/clang json e clássico, nm, map do ld/lld, ccache -s,
                sccache --show-stats, svd)
integração     ferramenta falsa no padrão rsync_falso: compilador falso que
               ecoa argv (prova as flags vindas da CDB e o -fsyntax-only),
               cancelamento entre dois salvamentos, timeout
real           exercitação com arm-none-eabi e aarch64-linux-gnu reais;
               OpenOCD/QEMU (o ciclo verificar-embarcado.sh existente)
QML            harness por controller (intenção, estado, resposta velha)
mutação        cada guarda nova provada removendo-a
artefato       AppImage com avisos-qml.txt + Debian mínimo
hardware       placa do autor: só leitura/monitor sem pedido; gravar com pedido
```

### 10.1 Emuladores: o que cada um prova e o que fica NÃO PROVADO

Decisão do autor (2026-10-01): os emuladores entram na 0.4 como **alvo** onde
o firmware roda, pelo `debugServer` do kit — o mesmo caminho que o QEMU já usa
no `scripts/verificar_embarcado.py`. **Não é simulação física/matemática**, que
continua fora do produto (40 §5): a IDE não modela o mundo, ela sobe o emulador
que o usuário escolheu e o depura. Também é a resposta para quem não tem placa:
é o que dá para provar sem hardware.

```text
                 QEMU (arm)         Renode              QEMU Espressif
estado hoje      MEDIDO no gate     sem código          sem código
                 (lm3s6965evb,
                 gdb -i dap)
J1–J3            não dependem de emulador (build e contexto são do host)
J4 gravar        carrega o ELF      carrega o ELF       carrega a imagem
                 (-kernel); gravar  (hipótese)          (hipótese)
                 em placa real fica
                 NÃO PROVADO
J4 monitor       serial do emulador por pty/tcp (hipótese, a medir)
J5 depurar       ciclo DAP inteiro  ciclo DAP inteiro   ciclo DAP inteiro
                 (provado)          (hipótese)          (hipótese)
J5 periféricos   pobre no           modela periféricos  periféricos do ESP32
pelo SVD         lm3s6965 (medir)   (hipótese: o forte  (hipótese)
                                    dele)
sempre NÃO       sonda física, OpenOCD com placa, tempo real, consumo, rádio
PROVADO sem placa
```

"Hipótese" quer dizer não medido nesta máquina: cada célula vira medida na
fatia que a implementa, ou continua dita como hipótese. O emulador entra na
matriz quando um ciclo do `scripts/verificar_embarcado.py` passar com ele;
ferramenta ausente é NÃO PROVADO, nunca verde calado.

## 11. Ordem e marcos

Dependências primeiro; cada marco é publicável sozinho se as provas passarem.

```text
0.4.0  fundação: §4 contexto efetivo + origem (index.context), preset no
       configure automático (§5.1.5), DiagnosticSource::Compiler e C1 ao
       salvar (§5.2), L6/L7 (§5.11); PlatformIO compiledb (§5.10, é o
       contexto); a matriz de emuladores (§10.1) escrita com o que já mede
0.4.1  cross: toolchain file gerado (§5.4), C2 inspector (§5.3), deriva (§5.9),
       toolchain.json schema 3
0.4.2  construir: C7 símbolo/delta (§5.5), C5 cache (§5.6), C4 Bear (§5.7);
       PlatformIO `pio test` e `pio check`
0.4.3  gravar/depurar: sonda no kit, OpenOCD por VID:PID, SVD leitura e
       debug.writeMemory (§5.8); PlatformIO depurar; Renode como
       debugServer (§10.1)
0.4.4  editor: L2, L3, L5 (§5.11)
0.4.5  Banco para embarcados (decisão do autor, 2026-10-16): o LSP por
       conexão (o passo 10 do 59 §2.3: instâncias isoladas, Postgres
       Language Server, syntaqlite, o servidor do MongoDB; arquitetura/38) e
       o InfluxDB 3 nativo com a linguagem dele (o passo 11); e o fechamento
       da versão, com o AppImage (só com a 0.4 finalizada)
```

## 12. Pontos de rollback

- C1 atrás de setting "checar ao salvar" por linguagem (padrão ligado para
  C/C++ com CDB); desligar não afeta LSP.
- Schema 3 do kit lê o 2; voltar para o binário anterior lê o 3 como 2 se os
  campos forem opcionais (provar).
- `debug.writeMemory` sem suporte do adaptador → painel fica só leitura.
- Toolchain file gerado mora em `.kinein/`; remover o arquivo volta ao
  comportamento anterior.

## 13. Decisões do autor ainda abertas

1. C1 ligado por padrão para C/C++? **Decidida (2026-10-15): sim, com CDB**;
   Rust por padrão só `cargo check` quando o rust-analyzer não está ativo,
   para não duplicar o flycheck.
2. Onde fica o painel de periféricos? **Decidida (2026-10-15): aba da área
   Embarcados.**
3. Delta de tamanho? **Decidida (2026-10-15): só do último build.**
4. Quais placas definem "pronto" para a 0.4 (o 42 §0 lista famílias; confirmar
   com o que o autor tem na mesa)? **Em parte respondida (2026-10-01):** o que
   um emulador cobre prova-se nele (§10.1); a placa real confirma o que o
   emulador não cobre (sonda, gravação física). **Em 2026-10-15, sem placa:**
   o autor pausou o desenvolvimento e autorizou o fechamento autônomo da 0.4
   ([40](40-estado-e-continuidade.md) §5, "modo autonomo"); o que só a placa
   prova fica NÃO PROVADO, dito. Sem AppImage (a prova de artefato do §10
   fica não feita por decisão do autor).
5. **Decidida (2026-10-16):** o Banco para embarcados (o LSP por conexão e o
   InfluxDB 3 nativo, adiados na 0.3.9) volta dentro da 0.4, no fim (0.4.5);
   o AppImage sai só com a 0.4 finalizada, e então a prova de artefato do §10
   vale para ele.

## 14. O desenho fino das fatias (escrito antes do código de cada uma)

Cada fatia da §11 ganha aqui, antes do código, o que foi medido no checkout,
o desenho e as provas; o resultado vai ao 40.7. A ordem dentro da 0.4.0
segue o 58 §4.4: o contexto efetivo primeiro, porque o C1, o inspector e a
deriva leem dele.

### 14.1 0.4.0a — o contexto efetivo com a origem de cada valor (§4; 2026-10-16)

**Medido no checkout (2026-10-16).** O `index.context` devolve o
`FileContext` com a unidade de compilação (`unit`, da CDB ou do file-api do
CMake), o alvo do cargo, o interpretador Python, os targets do CMake, uma
`source` em texto e uma `hint`. O que falta (58 §4.4): o kit e o alvo
efetivos e a origem de cada valor como dado. O kit tem dono (`toolchain`):
desde a 16.7b ele também diz o compilador que o cache do CMake do projeto
registra. O consumidor na tela é a barra de status ("contexto: c++ ·
gnu++20 · 0 -I · 0 -D", e o detalhe ao pairar), sem tela nova.

**O desenho:**

1. **Protocolo 0.173.0 (aditivo):** o `FileContext` ganha `effective`
   (opcional), com `compiler`, `standard`, `target` e `sysroot`, cada um
   `{ value, origin }`, o `flagsOrigin` (de onde vêm os `-I`/`-D` da
   unidade) e o `kit` (`{ preset }`, o kit que vale). `origin` é um
   vocabulário fechado: `cdb`, `cmakeFileApi`, `cmakeCache`, `kit`. O
   `cmakeCache` é novo diante da lista do §4.3 (é o que a 16.7b passou a
   ler, e o §4.3 era proposta); `cargoMetadata`, `projectModel` e `default`
   ficam de fora até um valor precisar deles.
2. **A precedência é a do §4.3, "o valor vem do dono":** as flags reais
   vencem a escolha. Compilador: o da unidade (`cdb` ou `cmakeFileApi`),
   senão o fixado no kit (`kit`), senão o do cache do CMake (`cmakeCache`).
   `standard`: o `-std=` da unidade. `target`: o `--target=`/`-target` da
   unidade, senão o `targetTriple` do kit. `sysroot`: o `--sysroot` da
   unidade (ou o do file-api), senão o do kit. Sem fonte, o campo fica
   ausente: nada de padrão suposto (a regra da 16.7b).
3. **O kit que vale** é o do último configure (o preset anotado no build
   dir, `cmake::status`), senão o padrão. É o mesmo kit cujas escolhas o
   configure usou.
4. **Donos:** o `index/context` preenche o que a unidade sabe (ele sabe se
   ela veio da CDB ou do file-api); uma função pura nova,
   `index/context/effective.rs`, completa com os fatos do kit, que o
   handler do `index.context` resolve pelo `Toolchain::resolve_kit` (o dono
   do kit). Nenhum valor é guardado: é derivado a cada pergunta.
5. **UI:** o detalhe da barra (`IndexController.contextDetail`) diz o
   efetivo com a origem ("alvo arm-none-eabi, do kit"; "sysroot …, da
   compile_commands.json"). O resumo não muda.

**Provas planejadas:** testes puros do `effective.rs` (cada precedência e
cada ausência); teste integrado do `index.context` com uma CDB que traz
`--target`/`--sysroot`, com um kit que fixa sysroot e alvo, e com o cache do
CMake sem CDB; harness do `IndexController` (o detalhe com a origem; campo
ausente não vira "undefined"); mutações; no Xvfb, o detalhe ao pairar.

**Resultado (2026-10-16).** Como desenhado, com uma correção antes do
código: o vocabulário de `origin` ficou só com os quatro valores que algum
dono diz hoje (`cargoMetadata`, `projectModel` e `default` esperam um valor
que precise deles). O `effective.rs` é puro (a unidade e o kit entram como
dado); o handler resolve o kit pelo dono e completa. Provas: 3 testes puros,
3 pelo despacho real (a CDB com `--target=` e `--sysroot` separado vencendo
o kit; sem CDB, o cache do CMake e o kit; ninguém dizendo nada e nada
suposto), o `tst_index.qml` com o detalhe, 7 mutações reprovando; no Xvfb,
o projeto de fluxo sem CDB mostra ao pairar "compilador /usr/bin/c++ (do
cache do CMake)", e a barra diz IPC 0.173.0 (40.7 §7.307).

### 14.2 0.4.0b — o preset do configure automático: o anotado, o único, ou a pergunta (§5.1, passo 5; 2026-10-16)

**Medido no checkout (2026-10-16).** O P0 de 2026-09-17 (40.7 §7.46) já fez
o configure automático usar um preset: o do kit que a tela escolheu (a ponte
C++ guarda o último `toolchain.get` e o manda, `presetSource: "kit"`), senão
o **primeiro** `configurePresets` não oculto de `CMakeUserPresets.json`,
senão de `CMakePresets.json` (`cmake::default_preset`). O §5.1 deste plano,
escrito depois (2026-10-01), pede outra coisa para o caso de vários: "com
UM preset não oculto, o configure automático usa esse preset; com vários,
pergunta pela faixa de saúde, sem adivinhar". E há um defeito que o desenho
achou: o kit da tela só vive na sessão da ponte. Reaberta a IDE, o
reconfigure ao salvar um `CMakeLists.txt` (e o botão Reconfigurar) volta ao
"primeiro preset" e pode trocar o preset de um build já configurado, em
silêncio. O preset do último configure que deu certo já fica anotado no
build dir (`.kinein-preset`) e ninguém o lê para escolher. O próprio modelo
de projeto C++ da IDE cria dois presets (Debug e Release), então a pergunta
vai aparecer em todo projeto novo: é o que o VS Code faz no primeiro
configure.

**O desenho:**

1. **A precedência do configure sem preset pedido** (dono:
   `handlers/cmake.rs`, `escolher_preset`): o pedido (o kit da tela) →
   o anotado no build dir, se ainda existe entre os presets do projeto
   (`presetSource: "buildDir"`) → o único preset não oculto do primeiro
   arquivo que tem algum (`CMakeUserPresets.json`, depois `CMakePresets.json`)
   → **vários: o configure é recusado com `INVALID_PARAMS`** dizendo o
   arquivo e os nomes. Nenhum job roda, nada é suposto.
2. **Protocolo 0.174.0 (aditivo):** `cmake.status` ganha `presetChoices`
   (os nomes, quando o configure sem preset seria recusado por haver
   vários) e `presetChoicesFile`; `presetSource` ganha `"buildDir"`.
   `cmake::default_preset` passa a dizer "nenhum", "um" ou "vários" (um tipo
   no lugar da `Option`).
3. **UI, a faixa de saúde pergunta:** com `presetChoices`, o
   `ProjectHealthController` não dispara o configure automático e mostra
   "o projeto tem N presets de configure (arquivo): escolha um", com o
   botão "Escolher preset…". O botão abre o menu suspenso que a casca já tem
   (o `AppMenuPopup`) com um item por preset; escolher um faz o que o
   seletor de kit da área Embarcados faz (`ToolchainController.selectPreset`)
   e configura (`cmake.configure`), e o Jobs aparece.
4. **Quem não muda:** o preset do kit continua vencendo; projeto sem
   presets configura sem preset; o configure explícito com preset também.

**Provas planejadas:** testes do core (o anotado vence o padrão e é ignorado
se sumiu dos presets; um só; vários recusam o configure sem job e aparecem
no `cmake.status`; o kit vence tudo; o `CMakeUserPresets` antes do
`CMakePresets`); harness do `ProjectHealthController` (com escolhas, nada de
configure automático e a faixa certa) e do menu de presets (o item configura
com o preset); mutações; no Xvfb, um projeto do modelo C++ da IDE (dois
presets): a faixa, o menu, o configure com o escolhido, e depois de reabrir
o reconfigure usando o anotado.

**Resultado (2026-10-16).** Como desenhado. A geometria do menu ficou com a
própria faixa (`ProjectHealthBanner.menuSpace`: o ponto logo abaixo do
botão), porque a `ShellWorkspaceHost` estava no teto de 400 linhas; o item
escolhido passa pela `ShellHeaderHost.executeMenuAction` (prefixo
`cmake.preset:`), que já é quem executa as ações do menu compartilhado.
Provas: o teste unitário do `default_preset` (um, vários, o arquivo do
usuário vencendo, o inválido não escondendo o do projeto) e o integrado
`several_presets_ask_and_the_recorded_one_wins_until_it_vanishes`; o
`tst_preset_choice.qml` com o controller e a faixa reais; 6 mutações
reprovando; no Xvfb, um projeto com `debug` e `release`: a faixa, o menu
abaixo do botão, o configure com `release` (Ninja, `CMAKE_BUILD_TYPE=Release`,
anotado), e, reaberto numa sessão nova e confiada, o reconfigure ao salvar
o `CMakeLists.txt` com `params {}` saindo com `preset: "release"` e
`presetSource: "buildDir"` (IPC gravado), IPC 0.174.0 (40.7 §7.308).

### 14.3 0.4.0c — o diagnóstico do compilador ao salvar, C/C++ (C1, §5.2; 2026-10-16)

**Decisão do autor que vale (§13.1, 2026-10-15):** ligado por padrão em C/C++
com CDB; Rust só com `cargo check` quando o rust-analyzer não está ativo.

**Medido no checkout e na máquina (2026-10-16).** O GCC 15.2 e o clang 21.1.8
desta máquina dão o formato clássico `arquivo:linha:coluna: error: …` com
`-fsyntax-only` (o GCC também dá `json` e `sarif-stderr`); o
`build/parse.rs` já lê esse formato (`parse_gcc_like_line`), e é o dono
(§3.2): nada de leitor de JSON novo. Um `.cpp` pequeno leva 0,01 s (GCC) e
0,02 s (clang); uma unidade com os cabeçalhos do Qt é a medida que importa, e
entra no resultado. O autosave grava sozinho depois de uma pausa na
digitação (`autoSave`, ligado por padrão): o C1 roda quase "enquanto se
escreve", e o debounce com cancelamento por unidade é o que o mantém
barato. A UI já mostra a união por arquivo do `event.lsp.diagnostics`, que a
fusão do core monta por "servidor" (`lsp/diagnostics_merge.rs`).

**O desenho:**

1. **Protocolo 0.175.0 (aditivo):** `DiagnosticSource` ganha `compiler`; as
   configurações ganham `compilerCheckOnSave` (efetivo: ligado).
2. **`build/syntax_check.rs` (puro):** da unidade da CDB, a linha da
   checagem: tira `-o <x>`, `-c`, `-M*` (com o argumento dos que levam um) e
   as cores; põe `-fsyntax-only` e `-fdiagnostics-color=never`; unidade no
   estilo do MSVC (`cl.exe`, `/Fo`) fica de fora nesta fatia (o porte para o
   Windows está pausado). Da saída, os diagnósticos por arquivo
   (`parse_gcc_like_line`), com caminho relativo resolvido contra o
   `directory` da unidade.
3. **Agendamento (`handlers/syntax_check.rs`):** depois de um `fs.write`
   bem-sucedido de um `.c/.cc/.cpp/.cxx` com unidade na CDB e a configuração
   ligada, **num projeto confiável** (correção feita ao implementar: a CDB
   nomeia o compilador, e uma CDB que veio num repositório clonado poderia
   nomear qualquer programa — salvar executaria código do projeto sem a
   confiança da 16.4b), um job curto por unidade ("Checar `a.cpp`", risco baixo,
   cancelável): espera 400 ms; um novo salvamento da mesma unidade CANCELA o
   anterior (o job velho nunca sobrescreve o novo); roda com prazo de 10 s.
   Cabeçalho sem unidade própria fica só com o LSP (§5.2, passo 2).
4. **Fusão:** o resultado entra na mesma fusão do LSP com a chave
   `compiler`, para cada arquivo citado e para os que a unidade citou da vez
   anterior (é o que apaga o erro corrigido); a UI recebe a união, como já
   recebe a do ruff com o basedpyright. Falha (compilador ausente, prazo) não
   apaga nada: o job falha com a saída bruta; nunca "sem problemas" por
   engano.
5. **UI:** a caixa "Checar com o compilador ao salvar (C/C++)" nas
   configurações do editor; os Problemas e o editor já mostram a união.

**Provas planejadas:** testes puros da linha (as flags tiradas e postas, as
duas formas `-o x`/`-ox`, `-MF x`) e da leitura (relativo resolvido, o
arquivo incluído também); teste integrado com um compilador FALSO que ecoa o
argv e imprime erros (o padrão `rsync_falso`): o `-fsyntax-only` vindo da
CDB, o cancelamento entre dois salvamentos, o prazo, a configuração
desligada, o cabeçalho sem unidade, e a fusão apagando o erro corrigido;
harness da caixa nas configurações; mutações; no Xvfb, um erro digitado
aparecendo nos Problemas como "compiler" antes de qualquer build, e
sumindo ao corrigir; a medida salvar→diagnóstico (mediana e p95) numa
unidade deste repositório com Qt, e a tecla→quadro sem regressão.

**Resultado (2026-10-16).** Como desenhado, mais a trava de confiança
(acima) e o prazo de 1 s no build de teste (o teste do prazo não segura a
trava dos executáveis por 10 s). Provas: 3 testes puros (a linha, o
`cl.exe` de fora, a leitura), 3 integrados com o compilador falso (o ciclo
salvar → erro `compiler` → corrigir → apagado, e o projeto não confiável sem
rodar; dois salvamentos numa checagem só, o cabeçalho e a configuração
desligada; o prazo falhando o job sem apagar o erro de antes); o
`tst_compiler_check_setting.qml`; 8 mutações reprovando. No Xvfb, o projeto
de fluxo confiado: o erro digitado e salvo chega primeiro do clangd e depois
do GCC do projeto, fundidos nos Problemas ("invalid conversion from ‘const
char*’ to ‘int’", sem código de regra, ao lado do "Cannot initialize…" do
clangd), e corrigido some dos dois (IPC gravado). **Medido** (o tempo do
compilador, que domina; a espera de 400 ms soma): C de firmware com o
`arm-none-eabi-gcc`, 12 ms; C++ com `<iostream>`, 220 ms; uma unidade da UI
deste repositório com o Qt, 1,9 a 2,4 s (GCC 15.2), acima da régua de 1,5 s
do §8. A régua cabe no projeto de firmware, que é o alvo da 0.4; nas
unidades com Qt, o custo é dos cabeçalhos, e o debounce não o muda (40.7
§7.310). A tecla→quadro não muda: a checagem roda num job, fora da UI.

### 14.4 0.4.0d — o progresso do servidor na barra (L6); L7 medido e dispensado (§5.11; 2026-10-16)

**Medido (2026-10-16), com uma sonda LSP direta, fora da IDE**, no projeto
de fluxo (clangd 21.1.8) e num `cargo init` (rust-analyzer 1.96.1):

```text
                         sem as capacidades        com workDoneProgress e
                         (como a IDE anuncia hoje)  didChangeWatchedFiles dinâmico
clangd                   nenhum progresso           "indexing": começo, 2 relatos, fim;
                                                    nenhum pedido de vigiar arquivo
rust-analyzer (20 s)     nenhum progresso           13 frentes ("Fetching", "Building
                                                    CrateGraph", "Roots Scanned",
                                                    "Indexing", "cargo check"…), 145
                                                    relatos (~7/s); pede para o cliente
                                                    vigiar os arquivos (2 registros)
```

E, só com o progresso anunciado (sem o registro dinâmico), um `Cargo.toml`
mudado por fora fez o rust-analyzer recomeçar em 0,1 s ("Fetching",
"Indexing", "cargo check"): ele vigia sozinho quando o cliente não se
oferece. O clangd não pede vigilância, e a CDB nova depois de um configure
já é tratada fechando os documentos C/C++ (`roadmaps/29` §5b,
`handlers/cmake.rs`).

**A decisão de escopo, pela medida:** a **L7 sai** — anunciar
`didChangeWatchedFiles` obrigaria a IDE a cumprir os globs que o
rust-analyzer registra, trocando um vigia que funciona por um que teria de
ser feito; nada medido pede isso. A **L6 entra**.

**O desenho da L6:**

1. **Handshake:** o cliente anuncia `window.workDoneProgress: true`; o
   `window/workDoneProgress/create` já é respondido com `null`, que é o que a
   especificação pede.
2. **Core (`lsp/server.rs`, a thread leitora):** o `$/progress` de cada
   servidor vira `event.lsp.progress { language, title, message?,
   percentage?, done }`; começo e fim sempre passam, e os relatos de uma
   mesma frente no máximo 4 por segundo (o rust-analyzer manda ~7). Um
   servidor parado não manda mais nada (o `stopping`, como na §7.309).
3. **Protocolo 0.176.0:** o evento novo, na lista canônica do 03 (61
   eventos) e no `03-dominios/lsp.md`.
4. **UI:** a ponte C++ emite `lspProgress`; o `LspStatusController` guarda a
   frente ativa por linguagem, e a barra diz "LSP … rust · Indexing 43%"
   enquanto ela dura (o detalhe ao pairar leva a mensagem); no fim, volta ao
   "LSP ● N".

**Provas planejadas:** testes do core (o `$/progress` lido de um servidor
falso: começo, relatos limitados no tempo, fim; o servidor parado sem
evento); a fiação do evento pelo gate; harness do `LspStatusController`
(frente ativa, porcentagem ausente sem "undefined", fim que volta ao
resumo); mutações; no Xvfb, um projeto Rust abrindo com a barra mostrando o
progresso do rust-analyzer e, ao fim, "LSP ● 1".

**Resultado (2026-10-16).** Como desenhado; o ritmo e a frente ficaram num
módulo puro, `lsp/progress.rs`, que a thread leitora chama. Provas: 2 testes
puros (começo, relatos no ritmo, fim; token numérico, frente desconhecida),
o integrado com o servidor falso (`--progresso`: o `initialize` anuncia
`workDoneProgress`, e dos seis relatos só o que respeita o intervalo passa),
o `tst_lsp_progress.qml`, a fiação com 61 eventos, 5 mutações reprovando. No
Xvfb, um projeto Rust confiado: a barra disse "LSP … rust · Fetching",
"cargo check", "Indexing 23%" e voltou a "LSP ● 1"; IPC 0.176.0 (40.7
§7.312).
