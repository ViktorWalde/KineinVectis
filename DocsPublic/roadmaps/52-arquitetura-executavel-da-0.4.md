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

**Refinamento do autor (2026-10-19):** a área Embarcados precisa de
restilização moderna, coerente com o ecossistema embarcado e com as evoluções
de frontend já entregues. A preparação ESP-IDF e o uso da placa precisam ter
fluxo no frontend da IDE; a execução pelo terminal não encerra esse aceite.
O ponto de partida medido é `EmbeddedPanel.qml`, hospedado por
`EmbeddedPanelHost.qml`, com as views de Placa, Projeto, Gravar e Kit e seus
controllers existentes. O desenho deve reutilizar `Theme`/`Kv*` e a
[arquitetura de frontend](../especificacoes/arquitetura-de-frontend-0.3-em-diante.md),
dar hierarquia à identidade/conexão, ambiente/projeto e próxima ação, e
apresentar os estados de ferramenta ausente, permissão, operação em curso e
resultado no lugar correspondente. Layout e dimensões finais dependem da
inspeção da janela real; sem aprovação visual presumida por leitura de QML.
O aceite pedido é na Kinein do notebook, por teclado/mouse na sessão real,
com fotos antes/depois; Xvfb e telas internas não atendem a este aceite.

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

**Momento do artefato, atualizado pelo autor em 2026-10-21:** as provas
funcionais/gates continuam no fechamento da 0.4.9; o AppImage só é gerado e
testado após a edição especial imediatamente seguinte. Revalidar as fontes
finais dessa edição e exercitar o artefato polido com os smokes e o ambiente
mínimo acima. Não reutilizar o gate da 0.4.9 como prova de código alterado
depois dela. Identificação/versão do pacote ainda a definir no 57 §1.1.

### 10.1 Emuladores: o que cada um prova e o que fica NÃO PROVADO

Decisão do autor (2026-10-01): os emuladores entram na 0.4 como **alvo** onde
o firmware roda, pelo `debugServer` do kit — o mesmo caminho que o QEMU já usa
no `scripts/verificar_embarcado.py`. **Não é simulação física/matemática**, que
continua fora do produto (40 §5): a IDE não modela o mundo, ela sobe o emulador
que o usuário escolheu e o depura. Também é a resposta para quem não tem placa:
é o que dá para provar sem hardware.

```text
                 QEMU (arm)              Renode            QEMU Espressif
na maquina       10.2.1 do Ubuntu 26.04, ausente           ausente (o
(2026-10-17)     20 maquinas Cortex-M                      qemu-system-xtensa
                                                           do Ubuntu nao tem
                                                           a maquina esp32)
J1–J3            nao dependem de emulador (build e contexto sao do host)
J4 gravar        PROVADO no gate:        hipotese          hipotese
                 carrega o ELF (-kernel)
J4 monitor       MEDIDO fora da IDE:     hipotese          hipotese
(a serial)       o UART0 sai por
                 -serial tcp; com -S, o
                 cliente ligado antes do
                 continue recebe tudo.
                 NA IDE, NAO: o stdout
                 do debugServer vai para
                 /dev/null e o pty nao
                 esta' no sysfs
J5 depurar       PROVADO no gate: o      hipotese          hipotese
                 ciclo DAP inteiro
                 (gdb -i dap, o gdb
                 multiarch)
J5 perifericos   NAO PROVADO: sem SVD    hipotese (o forte hipotese
pelo SVD         do LM3S6965 na maquina, dele)
                 e o SVD so' vai ao
                 probe-rs, nao ao gdb
sempre NAO       sonda fisica, OpenOCD com placa, tempo real, consumo, radio
PROVADO sem placa
```

"Hipótese" quer dizer não medido nesta máquina: cada célula vira medida na
fatia que a implementa, ou continua dita como hipótese. O emulador entra na
matriz quando um ciclo do `scripts/verificar_embarcado.py` passar com ele;
ferramenta ausente é NÃO PROVADO, nunca verde calado.

**As medidas (2026-10-17, 0.4.0f, 52 §14.6).** O gate da noite de 16 para
17 passou o ciclo do `lm3s6965evb` (attach, breakpoint, variável, 93 ms do
continue ao stop). A serial, com um firmware que escreve no UART0
(`0x4000C000`), fora da IDE: `-nographic` a põe no stdout do QEMU;
`-serial tcp:127.0.0.1:<porta>,server=on,wait=off` abre a porta em 0,05 s e
perde o que sai antes de alguém ligar; `wait=on` segura o QEMU até o cliente;
com `-S` (como o `debugServer` do gate sobe), o cliente ligado antes do
`continue` recebeu tudo em 0,41 s. O `gdb` 17.1 do Ubuntu não conhece ARM
("unknown architecture arm"); o `gdb-multiarch` sim — e é o que o core
escolhe (`dap/gdb_pick.rs`).

**Para a 0.4.3 (proposta, não decisão):** a serial do emulador na IDE — o
stdout do `debugServer` (o `-nographic`) num painel, em vez de `/dev/null`;
o SVD também para o adaptador `gdb`; o Renode e o QEMU Espressif instalados
no usuário e medidos com o mesmo ciclo.

## 11. Ordem e marcos

Dependências primeiro; os números abaixo organizam o trabalho. A série
acumulada fecha funcionalmente na 0.4.9; a edição especial vem em seguida,
forma a base da 0.5 em diante e precede a geração/teste do AppImage, conforme o
[57 §1.1](57-mapa-de-versoes-ate-a-1.0.md#11-fronteira-da-entrega-049--decisão-de-2026-10-20).

**Execução automática renovada pelo autor (2026-10-20; pacote atualizado em
2026-10-21):** concluir até a 0.4.9, executar a edição especial em seguida
e somente depois gerar/testar o AppImage local. Isso substitui o marco de
pacote imediatamente ao fechar a 0.4.9. Cada marco inclui revisão da lógica,
exercitação visual dos fluxos e das falhas, correção de bugs e medidas de
desempenho. O polimento dos painéis acompanha as fatias: hierarquia clara,
próxima ação contextual, detalhes avançados sob demanda e estados de espera,
erro e conclusão compreensíveis. A direção visual é contemporânea, mantendo
os componentes e os donos existentes. A prioridade é completar Embarcados,
incluindo C puro, bare metal, Linux embarcado, a distinção
CPython/MicroPython/CircuitPython e integrações nativas ativáveis pelo
usuário. A conclusão se mede
pelas jornadas e gates; o número da versão sozinho não encerra trabalho.

```text
0.4.0  fundação: §4 contexto efetivo + origem (index.context), preset no
       configure automático (§5.1.5), DiagnosticSource::Compiler e C1 ao
       salvar (§5.2), L6/L7 (§5.11); PlatformIO compiledb (§5.10, é o
       contexto); a matriz de emuladores (§10.1) escrita com o que já mede
0.4.1  cross: toolchain file gerado (§5.4), C2 inspector (§5.3), deriva (§5.9),
       fechar reserva/recuperação serial; reformular Instalar Ferramentas
       junto do Setup no dock; depois toolchain.json schema 3
0.4.2  construir: C7 símbolo/delta (§5.5), C5 cache (§5.6), C4 Bear (§5.7);
       PlatformIO `pio test` e `pio check`; assistente C puro/padrão C visual;
       Quality Center inicial sobre build/lint/testes existentes (Q1)
0.4.3  gravar/depurar: sonda no kit, OpenOCD por VID:PID, SVD leitura e
       debug.writeMemory (§5.8); PlatformIO depurar; Renode como
       debugServer (§10.1); integração própria de CircuitPython (§14.14, 41 §3.4)
0.4.4  editor: L2, L3, L5 (§5.11)
0.4.5  Banco para embarcados (decisão do autor, 2026-10-16): o LSP por
       conexão (o passo 10 do 59 §2.3: instâncias isoladas, Postgres
       Language Server, syntaqlite, o servidor do MongoDB; arquitetura/38) e
       o InfluxDB 3 nativo com a linguagem dele (o passo 11).
       Marco funcional preservado; não é mais o fechamento do pacote
0.4.6  consolidar as jornadas nativas de C, Python e embarcados, os fluxos
       bare metal/Linux e a ativação/desativação de integrações (§14.14–14.15).
       Fechar lacunas do inventário, sem adiar os básicos de C para a 0.5;
       Quality Center com gate local, resultados e recuperação (Q2/Q3)
0.4.7  revisão integrada de qualidade do frontend (§14.16–14.17): fechar
       inconsistências de componentes, HUD/UI/UX, painéis e navegação
0.4.8  consolidar interação, animação, foco, acessibilidade, atalhos e
       cores semânticas; medir resposta e desempenho, fechar regressões
0.4.9  estabilização integrada: corrigir os bloqueadores das auditorias,
       repetir jornadas de aceitação, gates completos, documentação e
       versões coerentes; fecha a base funcional e libera a edição especial
especial  em seguida à 0.4.9: otimização/acabamento do §14.17.1, base da
          0.5 em diante; validar o conjunto polido antes do pacote
AppImage  somente após concluir a edição especial: alinhar identificação/
          metadados do artefato, gerar e testar localmente (§10)
```

A divisão 0.4.6–0.4.9 é a organização operacional deste plano para executar
o escopo pedido pelo autor. Não exige esperar a 0.4.6 para corrigir uma falha
já encontrada nem interrompe o polimento por fatia. C e Python podem ser
concluídos antes dela; 0.4.6 cobra a integração ponta a ponta. A revisão de
qualidade do frontend ocorre antes do fechamento funcional da 0.4.9.
A edição especial vem imediatamente depois e antes do AppImage; está
definida no §14.17 e não deve
receber pendências funcionais ou de usabilidade desta série.

**Pendências reafirmadas pelo autor em 2026-10-21:** todos os itens citados
no handoff pertencem à entrega 0.4.x. Atribuição operacional: concluir proteção
de run/MicroPython/gravação e recuperação de ocupação, Setup no dock e schema 3
na 0.4.1; assistente de projeto C puro e padrão C visual na 0.4.2; separar e
integrar CircuitPython na 0.4.3; consolidar CPython/MicroPython/CircuitPython,
as jornadas bare metal/Linux, preferências de ativação nativa e as demais
janelas de trabalho em painéis até a 0.4.6. Auditorias 0.4.7–0.4.8 corrigem
o conjunto antes do aceite funcional da 0.4.9. O AppImage espera a edição
especial posterior. Menus, dicas e confirmações
compactos permanecem conforme a decisão do autor. O aceite físico continua
explicitamente pendente até poder ser medido; não substituí-lo por fixture.
Nada desta lista fica para a 0.5 ou para a edição especial.

**Atualização do autor em 2026-10-21:** reformular Instalar Ferramentas na
própria migração do Setup e incluir **Quality Center inicial na 0.4.x**,
aproveitando o trabalho de frontend. Evoluir/polir a base nas versões
seguintes. A edição especial associada à 0.5 concentra otimização,
consistência e pequenos detalhes; não recebe essas entregas funcionais.
Os donos de escopo são [Instalar Ferramentas](../especificacoes/instalar-ferramentas.md)
e [Quality Center e gates](../especificacoes/quality-center-e-gates.md).
O §11.1 organiza a execução, sem antecipar Library/providers completos ou ROS 2.

**Compatibilidade e evolução, reforço do autor em 2026-10-21:** Embarcados
precisa atender projetos legados e novos dentro da matriz suportada, mantendo
os adaptadores de ferramenta/linguagem extensíveis. Critérios da 0.4.x:

- Abrir e inspecionar não migra nem reescreve o projeto. Respeitar padrões,
  SDKs, ambientes, lockfiles, presets e ferramentas definidos nele; mudanças
  de formato/configuração exigem ação explícita e prévia quando houver migração.
- Resolver a ferramenta realmente selecionada e suas capacidades/versão,
  sem substituí-la silenciosamente pela mais nova ou pelo runtime do host.
  Quem resolve dependências continua sendo a ferramenta oficial (§2, regra 7).
- Adaptar sintaxe e saída no dono existente do provedor; uma versão nova não
  deve exigir alterar o executor, o painel ou espalhar condicionais pelo core.
  Dados de catálogo e parsers/builders puros continuam pontos de extensão.
- Versão desconhecida ou incompatível não impede abrir arquivos. Explicar
  quais ações não puderam ser validadas e oferecer escolha de ambiente,
  ferramenta ou comando manual, sem fingir suporte nem descartar configuração.
- Conferir projeto antigo/novo, ambiente existente/instalação limpa e ao menos
  uma versão anterior/atual nos fluxos alterados. Registrar a matriz realmente
  medida por ferramenta/versão/alvo, incluindo recusas e recuperação pela UI.

A base modular não prova essa matriz sozinha. A sintaxe esptool atualmente
fixa na linha v5 é um caso concreto a adaptar na gravação nativa; a consulta
Python de projetos reconhecidos só pelo modelo também deve entrar nos aceites.
Esses requisitos pertencem às integrações e à consolidação 0.4.1–0.4.6, com
regressão integrada antes da 0.4.9; não são promessa de compatibilidade universal
com qualquer ferramenta ou versão futura ainda não medida.

**Entrada de terminal (autor, 2026-10-21):** consolidar o comando curto na
0.4.6 e verificá-lo no AppImage após a edição especial. Contrato, compatibilidade e estado em
[Projetos e desktop §3](../especificacoes/projetos-arquivos-e-integracao-desktop-0.3.md#3-p0--abrir-pelo-terminal-e-pelo-desktop).

**Saída de cada 0.4.x:** conferir a coerência entre modelo do core e estado
mostrado, percorrer sucesso/erro/ausência/cancelamento na UI, corrigir bugs,
medir alterações de desempenho quando aplicáveis e atualizar manual,
contratos, estado e registro ao fim da tarefa. Função incompleta continua
pendente; não promover suporte só porque a CLI externa já existe. Os
inventários 41/42 e integracoes/39 delimitam a cobertura por ferramenta,
versão e alvo; manter explícitos os casos ainda não provados.

**Esclarecimento do autor após o handoff (2026-10-20):** priorizar eficiência
e fechar cada escopo sem deixar suas pendências para trás. A 0.4.9 deve
entregar o conjunto “redondo”, apto a receber um eventual polimento visual
mais amplo depois. Isso exige concluir a UI/UX necessária ao uso de cada
recurso junto de sua integração, e não concentrar correções numa etapa final.

**Critério de fechamento por escopo:**

- Inventariar os compromissos e lacunas no dono existente, com estado e
  evidência de cada item; conferir o inventário antes de encerrar o marco.
- Validar a jornada completa pela UI, do reconhecimento/configuração ao
  resultado, incluindo falha, cancelamento, retomada e estados indisponíveis
  pertinentes. Core, IPC, painel, paleta e terminal devem usar o mesmo
  contexto e explicar o que acontece.
- Corrigir no próprio escopo os bugs e dificuldades de uso encontrados;
  mudanças de escopo dependem de decisão explícita do autor. Não reclassificar
  uma pendência como polimento futuro para declarar o marco concluído.
- Preservar desempenho e acessibilidade, medir as mudanças relevantes,
  executar os gates aplicáveis e registrar provas e limitações reais.
  A distinção já autorizada entre emulador e hardware continua valendo (§10.1).
- Atualizar os documentos e entregar uma fatia revisável. Reusar ferramentas,
  componentes e donos existentes; escolher verificações que cobrem o risco
  real da mudança e evitar retrabalho ou duplicação de mecanismos.

As 0.4.7–0.4.9 verificam coerência e regressões do conjunto já concluído por
escopo. O pacote depende desse aceite integrado, sem pendências conhecidas
que impeçam ou contradigam as jornadas assumidas para a série.

### 11.1 Fluxo de execução reorganizado — 2026-10-21

**Objetivo:** fechar jornadas completas com menos retrabalho entre core e
frontend. Agrupar alterações no mesmo host/componente quando houver uma
dependência concreta; conservar fatias pequenas, revisáveis e com gate
próprio. Esta organização substitui a ideia de deixar a reformulação do
instalador ou o primeiro Quality Center para o polimento pós-0.4.9.

**Estado de partida:** h4b1 concluída em `a5a4f0e`, protocolo 0.185.0,
binário 0.3.9. h4b2 ainda não implementada nesta reorganização. A tabela
abaixo é plano, não lista de funcionalidades concluídas.

| Ordem | Marco/fatia | Dependência e entrega | Condição para seguir |
| --- | --- | --- | --- |
| 1 — próxima | 0.4.1h4b2: gravação/ocupação | Usar runconfig preservado e reserva serial existentes; intenção tipada/schema 2, adaptação por versão/capacidade e recuperação com identidade verificada (§14.27) | UI/core revalidam contexto, artefato e porta; sem parsing de shell salvo nem migração na abertura; matriz serial e limitações registradas |
| 2 | 0.4.1: Setup S1/S2 | Migrar ao dock **e reformular Instalar Ferramentas na mesma frente**, com busca, filtros, categorias, estados, fonte e revisão dos comandos | Guia e acessos anteriores preservados, retorno ao editor, teclado, erro/recuperação e envio explícito ao Terminal; sem depender de ROS 2 |
| 3 | 0.4.1: kits/schema 3 | Depois dos bloqueadores de orquestração e Setup; completar contexto e configuração dos kits no dono atual | Origem e valor efetivo coerentes, formatos legados preservados, migração somente explícita |
| 4 | 0.4.2: construir/C + Q1 | Build/test/check PlatformIO, cache/Bear/tamanho e assistente C no escopo do marco; expor build/lint/testes já existentes no Quality Center | Cada jornada nova fecha com UI; Q1 reusa rigor, Jobs e resultados, com ferramentas ausentes explicadas |
| 5 | 0.4.2–0.4.6: Q2/Q3 em incrementos | Depois de Q1, acrescentar gate local selecionável, execução coordenada, relatório e consolidação; usar as jornadas C/Rust/Python como casos reais | Gate passa/falha/não verifica/cancela sem falso verde; persistência, obsolescência e recuperação provadas antes da 0.4.7 |
| 6 | 0.4.3 | Depuração/gravação, PlatformIO, emuladores e CircuitPython, conforme §11 | Identidade, permissões e runtime corretos; limites de hardware explícitos |
| 7 | 0.4.4 | L2/L3/L5 do editor conforme §5.11 | Navegação/diagnósticos e desempenho sem regressão |
| 8 | 0.4.5 | Banco para embarcados, com os limites já decididos | Aceite dos donos de Banco; sem reabrir adiamentos |
| 9 | 0.4.6 | Consolidar C/CPython/MicroPython/CircuitPython, bare metal/Linux, integrações opcionais, janelas funcionais no dock, comando de terminal e Q3 | Inventário reconciliado com uso real e nenhum básico transferido à edição especial |
| 10 | 0.4.7–0.4.8 | Auditoria integrada de frontend, acessibilidade e desempenho (§14.17) | Corrigir achados, fotos comparáveis e regressões; não redesenhar a disposição principal |
| 11 | 0.4.9 | Estabilização, jornadas, gates completos e documentação/versões; fechamento funcional | Base funcional validada para a edição especial, sem gerar o pacote intermediário |
| 12 | Imediatamente após a 0.4.9 / edição especial | Otimização e acabamento do conjunto; estabelecer a base polida da 0.5 em diante | Jornadas e gates sobre o conjunto final; nenhum básico funcional da 0.4 adiado para esta etapa |
| 13 | Após concluir a edição especial | Alinhar identificação/metadados, gerar e testar o AppImage local com a base polida para teste de usuários | Critério de artefato do §10/57; smokes/ambiente mínimo; sem push/tag/publicação |

A linha 5 pode ser intercalada às demais em fatias concluídas, **sem aguardar
todos os marcos para começar Q2 e sem manter várias implementações pela
metade**. Sua dependência funcional é Q1 e os runners envolvidos, não Banco,
emuladores ou o catálogo completo da Library. Quando uma capacidade ainda
não estiver integrada, mostrar sua indisponibilidade; não bloquear a
entrega inicial para montar um framework genérico de providers.

**Como reduzir retrabalho:**

- Conferir Theme/Kv*, hosts, Settings, Jobs e Configuration Actions antes de
  editar a superfície. Corrigir apenas a base comum necessária à tarefa e
  reutilizá-la em Setup/Qualidade; não começar por um redesign global.
- Setup, Quality Center e futuro Environment Center compartilham contexto,
  componentes e acessos aos donos; não mantêm três cópias de toolchain,
  disponibilidade, rigor ou configuração efetiva.
- Validar primeiro a capacidade existente de ponta a ponta. Só depois
  acrescentar contrato/dado novo, com consumidor, desenho fino e prova.
- Usar strings traduzíveis nos componentes tocados, textos longos e inglês
  nos cenários de layout desde agora. A internacionalização pública da 0.5
  está no 57; não traduzir todo o produto no meio de uma fatia funcional.
- Rodar verificações focadas durante o desenvolvimento; depois o gate
  estrito da fatia. Não repetir rodada pesada sem mudança/falha nova que a
  justifique, não compilar UI durante análise C++ e não substituir prova
  nativa por protótipo. Antes do pacote, gate completo obrigatório.
- Ao concluir, atualizar dono, 40/40.7 e arquivos/medidas de retomada;
  preservar mudanças locais alheias. Próximo agente inicia pela primeira
  linha pendente e mede o código antes de reaplicar rascunhos antigos.

**Fora deste replanejamento:** implementação de produto, novo protocolo,
flash físico, mudança de default de rigor, novos providers, hooks/CI de
usuário ou antecipação de ROS 2. A próxima implementação continua h4b2.

**Pesquisa de frontend recebida em 2026-10-21:** consolidada no
[rascunho de HUD/UX de Embarcados](../especificacoes/embarcados-hud-ux-0.4.md),
com cinco imagens conceituais. A imagem 5 é a aproximação preferida pelo
autor, ainda sujeita a refinamento/aprovação visual. Nem as imagens nem as
soluções sugeridas são fonte da verdade ou design aprovado; evoluir a UI
atual conforme a [regra comum de aprovação](../especificacoes/arquitetura-de-frontend-0.3-em-diante.md#11-aprovação-visual-e-referências--decisão-de-2026-10-21).
Marcos e frentes já decididos continuam; o desenho visual permanece rascunho.
Pontos funcionais de contexto/artefatos/operação acompanham h4b2 e integrações
0.4.1–0.4.6; hierarquia/consistência entram junto dos fluxos e na 0.4.7;
acabamento na 0.4.8 e edição especial. Preservar o feedback do §14.16.
Não bloqueia trabalho autorizado independente do desenho, nem permite usar
aceite de protótipo/agente como aprovação visual do autor.

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
   prova fica NÃO PROVADO, dito. Naquela autorização não havia AppImage.
   **Atualização de 2026-10-20/21:** geração e teste locais autorizados
   somente após a edição especial que segue a 0.4.9 (§11); a prova de
   artefato do §10 continua obrigatória nesse fechamento do pacote.
5. **Decidida (2026-10-16):** o Banco para embarcados (o LSP por conexão e o
   InfluxDB 3 nativo, adiados na 0.3.9) volta dentro da 0.4, na 0.4.5.
   **Atualização de 2026-10-20/21:** fechamento funcional na 0.4.9; edição
   especial imediatamente em seguida; AppImage somente depois dela (§11).
   O marco funcional do Banco permanece na 0.4.5 (§11).

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

### 14.5 0.4.0e — o `compiledb` do PlatformIO como Configuration Action (§5.10, J2; 2026-10-16)

**Decisão do autor que vale (2026-10-01):** o PlatformIO é cidadão de tier 1;
na J2, `pio run -t compiledb` como Configuration Action (58 §4.4: "sem CDB,
o clangd não vê o projeto").

**Medido (2026-10-16).** O PlatformIO não estava na máquina; entrou no
usuário com `pipx install platformio` (Core 6.2.0, `~/.local/bin/pio`), como
o modo autônomo autoriza (40 §5). Num projeto mínimo `platform = native`, o
`pio run -t compiledb` grava a `compile_commands.json` na RAIZ do projeto
(onde o `cdb::status` já procura e o clangd acha sozinho): 5,6 s na primeira
vez (prepara a plataforma), menos de 1 s depois. No código: o projeto
PlatformIO já é detectado (`BuildSystem::PlatformIo`, `0.117.0`) e
construído (`build/frameworks.rs`, `pio run`); as Configuration Actions só
conhecem os escopos CMake e Cargo, e o efeito "job" está fixo no
`cargo.check` (`configaction/mod.rs` devolve sempre esse método).

**O desenho:**

1. **Protocolo 0.177.0 (aditivo):** `ConfigActionScope` ganha `platformio`;
   o catálogo ganha `platformio.compiledb` ("Gerar compile_commands.json",
   efeito `job`, afeta `compile_commands.json`, risco baixo), disponível num
   projeto PlatformIO com o `pio` detectado e recomendada quando não há CDB.
2. **O efeito "job" deixa de ser fixo:** cada ação de job diz o seu método
   (`cargo.check`, `platformio.compiledb`), e o handler delega a quem o
   atende — sem executor novo.
3. **O job (`build/frameworks.rs`, o dono do `pio`):** `pio run -t
   compiledb` na raiz, com a saída no Jobs. No sucesso, o mesmo caminho do
   `cmake.configure` que termina: o contexto do índice é relido (a CDB nova)
   e os documentos C/C++ são fechados no clangd para a UI reabri-los com as
   flags novas (`roadmaps/29` §5b).
4. **Nada muda sem gesto:** a ação aparece na lista de ações de
   configuração; a pessoa a aplica. Num projeto não confiável, ela recusa,
   como o configure (o `pio` roda scripts do projeto).

**Provas planejadas:** teste do catálogo e da disponibilidade (PlatformIO com
e sem `pio`, recomendada sem CDB); integrado com um `pio` FALSO que grava a
CDB (o argv `run -t compiledb`, o job, o contexto relido com a unidade, a
recusa sem confiança); a ação de `cargo.check` intacta; harness da lista de
ações se a UI mudar; mutações; real: o `pio` desta máquina no projeto
`native`, e no Xvfb a ação aplicada, a CDB criada e o clangd vendo o
`#include`.

**Resultado (2026-10-16).** Como desenhado, com cinco correções que a prova
na tela trouxe (40.7 §7.313):

1. **O clangd que subiu sem CDB não via a nova.** Medido com uma sonda LSP
   direta (clangd 21.1.8): a CDB que nasce na raiz depois de o clangd não
   achar nenhuma só vale uns 30 s depois, mesmo com o documento reaberto; em
   `.kinein/build`, nunca. O mesmo buraco existia no configure do CMake com
   o `.cpp` aberto antes do primeiro configure (o projeto ainda não
   confiável). O core lembra se o clangd subiu vendo uma CDB e, se não, o
   reinicia no fim do configure e do `compiledb`
   ([03-dominios/lsp](../arquitetura/03-dominios/lsp.md)).
2. **A faixa de saúde não oferecia a confiança num projeto PlatformIO** — a
   recusa mandava confiar "na faixa", que nada dizia. O PlatformIO entrou nos
   build systems que pedem o gesto, e o `pio` e o clangd passaram a ser
   cobrados (também no projeto misto, com o `CMakeLists` do ESP-IDF ao lado).
3. **O diálogo das ações:** o "não altera nenhum arquivo" (o texto do risco
   baixo e o da prévia) era falso para uma ação que grava a CDB — o risco
   baixo diz agora o que é, e a prévia de um job diz que ele roda e o que
   grava; a confirmação do apply era apagada pela releitura da lista que o
   próprio apply pede (em toda ação, desde antes), e a lista não se relia
   quando o job terminava; o catálogo de bibliotecas do CMake aparecia num
   projeto sem CMake; os chips de escopo eram fixos (CMake e Cargo) e agora
   saem do que o projeto tem.
4. **A dica do contexto** num projeto PlatformIO sem CDB mandava configurar
   com o CMake; agora aponta a ação.
5. **Disponibilidade:** a ação aparece em todo projeto PlatformIO; sem o
   `pio`, o `apply` recusa com `ToolNotFound` e o passo, como o `build.run`
   (a faixa cobra antes).

Provas: o integrado com o `pio` falso (recomendada sem CDB, a dica, a recusa
sem confiança e sem `pio`, o argv `run -t compiledb` NA RAIZ, a saída no job,
a unidade nova no contexto com o `-D`, a ação deixando de ser recomendada);
o do clangd reiniciado sem CDB e só reaberto com ela, no configure e no
`compiledb` (o job de outro dono e o que falhou não mexem); os harnesses
`tst_configaction`, `tst_configaction_layout` e `tst_project_trust`; 19
mutações do core e 10 da UI reprovando. Real: o `pio` desta máquina gerou a
CDB com `-DPLATFORMIO=60200 -Iinclude -Isrc` em 0,9 s. No Xvfb, num projeto
`native` com o cabeçalho em `include/`: o `'app.h' file not found` sem CDB,
a faixa pedindo a confiança, a ação aplicada (o job em menos de 0,4 s, a
lista relida), os erros sumindo e o Ctrl+B em `APP_VALUE` abrindo o
`include/app.h` no `#define`.

### 14.6 0.4.0f — a matriz de emuladores com o que já mede (§10.1; 2026-10-17)

**O que o §11 pede:** fechar a 0.4.0 com a matriz do §10.1 "escrita com o
que já mede" — cada célula medida nesta máquina, com data, ou dita como
hipótese; o Renode e o QEMU Espressif como `debugServer` são da 0.4.3.

**O que medir (só o QEMU ARM, que está na máquina):**

1. **Disponibilidade, por comando:** `qemu-system-arm` (versão, máquinas
   Cortex-M), `renode`, o QEMU da Espressif (`qemu-system-xtensa` com a
   máquina `esp32`; o do Ubuntu não a tem — medir), `arm-none-eabi-gcc`,
   `gdb`.
2. **J4 monitor — a serial do emulador:** um firmware mínimo (a fixture com
   uma escrita no UART0 do LM3S6965) sob `-serial tcp::<porta>,server,nowait`
   e sob `-nographic`; ler pelo socket. E o que a IDE faz hoje com ela: o
   `debugServer` sobe com o stdout em `/dev/null` (`dap/server.rs`), e o
   monitor serial lista só portas do sysfs.
3. **J5 periféricos pelo SVD no lm3s6965:** o que existe (um SVD público do
   LM3S6965?) e o que o QEMU modela; sem SVD, a célula fica NÃO PROVADO.

**O desenho:** sem código de produto. O §10.1 reescrito com as células
medidas e datadas; o 36 (ferramentas de embarcados) com a disponibilidade;
o que a medida pedir de código vira item da 0.4.3, não desta fatia.

**Resultado (2026-10-17).** Como desenhado (§10.1, com as medidas): o QEMU
10.2.1 com 20 máquinas Cortex-M; o Renode e o QEMU da Espressif ausentes (o
`qemu-system-xtensa` do Ubuntu sem a `esp32`); gravar e depurar PROVADOS no
gate; a serial MEDIDA fora da IDE (`-serial tcp`, e com `-S` nada se perde)
e ausente NA IDE (o stdout do `debugServer` em `/dev/null`, o pty fora do
sysfs); os periféricos pelo SVD NÃO PROVADOS (sem SVD do LM3S6965, e o SVD
só vai ao probe-rs). O que pede código ficou como proposta para a 0.4.3.
**Com isto a 0.4.0 está completa** (§11): o contexto efetivo, o preset, o
C1, a L6 (a L7 medida e dispensada), o `compiledb` do PlatformIO e a matriz.

### 14.7 0.4.1a — o cross sem o host vazando, e o kit exportável como toolchain file (§5.4, C3; 2026-10-17)

**Medido (2026-10-17), pelo core real** (o `toolchain.setKit` e o
`cmake.configure`/`build.run` por stdio), com um projeto que faz
`find_package(ZLIB)`:

```text
kit                                configure + build      find_package(ZLIB)
arm-none-eabi (bare metal)         ok; fw.elf ARM EABI5   nada (o Generic nao
                                                          procura no host)
aarch64-linux-gnu, sem sysroot     ok; ELF aarch64        INC=/usr/include: e' onde
(como o Ubuntu empacota)                                  o multiarch do Debian poe
                                                          os cabecalhos dos :arm64
aarch64 + sysroot (montado com     ok                     INC=/usr/include: o HOST
links para /usr/aarch64-linux-gnu)                        vaza no build da placa
o mesmo + MODE_*=ONLY              ok                     NOTFOUND (o certo)
sysroot=/usr/aarch64-linux-gnu     o teste do compilador  -
                                   falha (sem usr/include)
```

E duas medidas do `CMake` 4.2.3 que decidem o desenho: o `set()` de um
toolchain file **vence o `-D` da linha** (pedido o `/usr/bin/gcc` na linha,
ficou o compilador do arquivo); e o `aarch64-linux-gnu-gcc` do Ubuntu diz
`-print-sysroot` = `/`. O kit já põe o cross na linha do configure
(`toolchain/arguments.rs`: compiladores, `CMAKE_SYSROOT`,
`CMAKE_SYSTEM_NAME`/`PROCESSOR`, `TRY_COMPILE` no bare metal) — o "ponta a
ponta" da §5.4 já anda pelos dois kits; faltam o vazamento com sysroot e o
mesmo configure fora da IDE.

**O desenho:**

1. **Com sysroot no kit, o host não vaza:** quando o kit tem sysroot e põe o
   `CMake` em cross, e nem o preset nem o kit trazem toolchain file, o
   configure passa também `CMAKE_FIND_ROOT_PATH_MODE_PROGRAM=NEVER` e
   `LIBRARY`/`INCLUDE`/`PACKAGE=ONLY` — o que os `find_*` acham vem do
   sysroot; os programas (o `protoc`, o `python` do build) continuam os do
   host. Sem sysroot nada muda: o multiarch do Debian depende do padrão.
2. **O kit exportável:** a ação de configuração `cmake.exportToolchainFile`
   ("Exportar o kit como toolchain file", escopo CMake, efeito `edit`, risco
   baixo) grava `.kinein/toolchains/<kit>.cmake` com os MESMOS valores que o
   configure da IDE passa — um gerador só, no dono (`toolchain/arguments.rs`)
   —, para o terminal e o CI: `cmake --toolchain .kinein/toolchains/<kit>.cmake`.
   **A IDE não o lê:** o kit continua a fonte, e o arquivo, se lido, venceria
   em silêncio o kit que mudasse depois (medido acima). A prévia mostra o
   conteúdo (de novo, o diff); disponível quando o kit é cross e não tem
   toolchain file próprio; `alreadyApplied` quando o arquivo já diz o mesmo.
   O kit é o do configure (o do preset do último configure, se houver, senão
   o padrão), como no contexto efetivo.
3. **Protocolo 0.178.0 (aditivo):** a ação nova; o configure cross com
   sysroot passa os modos (`03-dominios/configAction` e `toolchain`).

**Provas planejadas:** unitários dos argumentos (com sysroot, os modos; sem,
nenhum; com toolchain file, nenhum) e do arquivo (os mesmos valores da
linha); o integrado da ação (sem kit cross, parcial com o motivo; a prévia
com o conteúdo; aplicar grava; de novo, `alreadyApplied`); mutações; real: o
arquivo exportado num `cmake --toolchain` no terminal dando o mesmo ELF
(aarch64 e ARM) e o configure da IDE com o sysroot de teste sem o cabeçalho
do host; no Xvfb, a ação aplicada no diálogo.

**Resultado (2026-10-17).** Como desenhado. As variáveis que o kit fixa
viraram uma lista só (`Toolchain::cmake_definitions`), de onde saem a linha
do configure e o arquivo; o arquivo mora em `toolchain/export.rs`, o "kit
ativo" (o mesmo do contexto efetivo) em `cmake::active_kit` — no `toolchain`
ele criava o ciclo `cmake ↔ toolchain`, que o mapa de módulos reprovou —, e
o plano com a prévia em `configaction/kit.rs`, que recebe o arquivo pronto do
handler (`configaction::Inputs`: o domínio só lê a raiz). O catálogo das
ações passou do teto de 500 linhas com a entrada nova e foi dividido por
escopo (`catalog/cmake.rs`, `cargo.rs`, `platformio.rs`). Provas: os
unitários (os quatro modos com sysroot; nenhum sem sysroot ou com toolchain
file; cada `-D` da linha é um `set()` do arquivo; nome e aspas de entrada
estranha); o integrado (parcial sem kit cross, a prévia com o arquivo novo,
aplicar grava, `alreadyApplied`, e de novo disponível quando o kit muda);
10 mutações reprovando. Real, pelo core: com o sysroot de teste, o configure
da IDE deu `INC=NOTFOUND` (antes, `/usr/include`); o arquivo exportado num
`cmake --toolchain` no terminal configurou e compilou nos três casos (ELF
aarch64 com e sem sysroot, ARM EABI5 no bare metal), com a mesma linha de
`find_package` da IDE. No Xvfb: a ação achada pela busca, a prévia com o
arquivo novo, aplicada, e a lista dizendo "ativa neste projeto".

### 14.8 CI pública — a camada rápida antes da 0.4.1b (2026-10-19)

**Origem:** decisão do autor de 2026-10-17, [40](40-estado-e-continuidade.md)
§5, "0.9 antecipada em parte". **Antes, medido em 2026-10-19:** não há
`.github/`; o orquestrador rejeita `--ci-rapida` (exit 2). O `--rapido`
local exige os presets CMake, compila a UI e roda C++, QML e integrações;
copiar parte dele para YAML criaria uma segunda ordem de gates.

**Desenho, antes do código:**

- `scripts/verificar.sh` ganha `--ci-rapida`: Rust (fmt, testes do workspace,
  Clippy, cargo-deny) e os gates estáticos existentes de shell, AppImage,
  identificadores, QML, IPC, atalhos, documentação, mapa, arquitetura,
  adiados Windows e transição de workspace, na mesma ordem local. O perfil
  não precisa de Qt nem de presets. UI compilada, harnesses e integrações
  ficam nas camadas seguintes; o veredito distingue esta cobertura do gate
  local. O `--rapido --estrito` continua obrigatório para cada commit.
- `scripts/test_verification_profiles.py` executa o orquestrador real numa
  cópia temporária, com ferramentas falsas, nos três perfis: cobertura,
  builds/smokes, fail-fast e NÃO PROVADO estrito. Roda no começo do gate.
- `scripts/verificar-appimage.sh --invariantes` limita a checagem às regras
  estáticas no perfil CI; o padrão local mantém o smoke oportunista de
  artefatos em `dist/`. Sem esse recorte, uma cópia com artefato deixaria a
  CI rápida executar uma prova gráfica fora do escopo anunciado.
- `.github/workflows/ci.yml`: push, pull request e disparo manual, Ubuntu
  24.04, Rust do `rust-toolchain.toml`, cargo-deny 0.20.2 (o instalado nesta
  máquina), pré-requisitos explícitos, cache Cargo por toolchain e lockfile,
  permissões de leitura, cancelamento de execução obsoleta e timeout. O
  único comando de validação é `bash scripts/verificar.sh --ci-rapida
  --estrito`; o YAML só prepara o runner. Actions fixadas por SHA, conferidas
  nas releases oficiais de [checkout](https://github.com/actions/checkout/releases/tag/v4.2.2)
  e [cache](https://github.com/actions/cache/releases/tag/v4.2.3).
- `contribuindo/07` é dono do perfil e suas fronteiras; `04` e os comandos
  de build apontam para ele. O estado e a entrega vão ao 40 e 40.7.

**Provas previstas:** perfil real nesta máquina; mutações em cópia
temporária para provar a seleção das etapas, fail-fast e o NÃO PROVADO
estrito; sintaxe shell e YAML; gates locais. **Fica fora:** executar ou
publicar o workflow no GitHub (é do autor), CI de UI/integrações e matriz
Arch, geração de AppImage, templates/labels/issues (outra fatia, publicação
após o fechamento da 0.4, agora 0.4.9, conforme o 57 §1.1),
protocolo e código do produto.

**Fronteira esclarecida em 2026-10-19:** o repositório-fonte já é público;
é o destino recomendado da CI e dos PRs. A cópia limpa não leva `.github/`
nem todos os documentos dos gates porque é um artefato de distribuição,
não uma segunda fonte de contribuição. A dúvida sobre espelho vinha de
um trecho superado do 21; ver contribuindo/07 §2.1 e
[contribuindo/11](../contribuindo/11-ci-publica-e-tarefas-para-colaboradores.md).

**Resultado (2026-10-19):** camada rápida preparada e validada localmente;
gate local rápido estrito TUDO VERDE, sem NÃO PROVADO. Provas, mutações,
limites e o que não foi feito no [40.7 §7.316](40.7-registro-das-entregas.md).
A execução no GitHub permanece pendente; a próxima fatia do produto é C2.

### 14.9 0.4.1b — inspector do contexto de compilação (2026-10-19)

**Antes, medido:** o `IndexController` só resume `FileContext` no rodapé;
`effective` já traz a origem por valor, mas a unidade não aponta sua linha
na CDB. O menu Código é o menu de ações do editor; não existe menu de
contexto próprio dele. O Ambiente já hospeda overlays sob demanda.

**Desenho anterior ao código:**

- `index.context` continua sendo a única consulta. `FileContext` ganha
  `cdbLocation? { path, line, column }` (absoluto, coordenadas desde 1),
  localizado pelo leitor existente, no início do objeto JSON escolhido
  (coluna UTF-16, como a navegação do editor).
  Duplicatas mantêm a regra atual (última unidade vence), com a localização
  correspondente; JSON compacto e escapes não viram uma busca textual pelo
  nome do arquivo. Protocolo 0.179.0, aditivo. File-api e ausência de unidade
  não oferecem localização da CDB.
- O `IndexController` mantém o ciclo abrir/fechar do inspector, acompanhando
  o arquivo ativo e descartando respostas de outro arquivo como já faz.
  Sem arquivo, diz isso; ao trocar de projeto, fecha e limpa. Um modelo de
  apresentação próprio traduz os fatos recebidos, sem resolver flags ou kit.
  Fechar devolve o foco ao editor, inclusive ao vir da paleta ou de um menu;
  restaurar só o foco anterior poderia apontar para um popup já escondido.
- `CompileContextInspector` nasce no primeiro uso, dentro dos overlays
  de Ambiente (`KvPanelFrame`), sem tool window/trilho novo. Mostra caminho,
  origem, compilador, padrão, target, sysroot, kit, targets, diretório,
  includes, defines, saída e argumentos separados, com texto selecionável;
  ausências e `hint` aparecem explicitamente. Rust/Python mostram os fatos
  que o mesmo contrato já fornece, sem fingir uma unidade C/C++.
- A paleta e os menus Código e Ambiente abrem o mesmo inspector pelo comando
  `index.context`. “Abrir CDB na unidade” usa a navegação do editor; “Abrir
  kit” pede o preset de `effective.kit` ao `ToolchainController` e abre seu
  menu existente. Nenhum caminho ou kit é deduzido da frase `source`.
  O menu recebe opcionalmente o preset a consultar, numa única requisição:
  selecionar por consulta assíncrona e depois abrir consultaria o preset
  anterior de novo; o harness usa o controller real para proteger esse salto.
- Esta fatia explica o contexto **efetivo**. A comparação configurado ×
  efetivo e os alertas persistentes pertencem à deriva (§5.9), próxima fatia;
  schema 3 também fica depois. Sem instalação, configure ou execução nova.

**Provas previstas:** contexto integrado com JSON compacto, multilinha,
Unicode/escapes e unidade repetida; ausência de localização no fallback;
harness de apresentação, ciclo do inspector, navegação e dispatcher; fotos
antes/depois no Xvfb com CDB real, clique para abrir a unidade e kit;
gates locais rápidos estritos. Resultado só após a execução.


**Resultado (2026-10-19):** 0.4.1b encerrada no checkout. Inspector e
navegação provados pelo core, harnesses e gestos no Xvfb; cinco mutações
reprovaram. Gate rápido estrito TUDO VERDE, sem NÃO PROVADO. Detalhe no
[40.7 §7.319](40.7-registro-das-entregas.md). Próxima fatia: deriva (§5.9),
antes do schema 3.

### 14.10 0.4.1c — deriva de compilação, sem correção automática (2026-10-19)

**Desenho anterior ao código:** a continuação linear autorizada pelo autor.
O inspector já explica os valores efetivos; agora compara fatos explícitos
da unidade com os valores do kit que o último configure usou.

- Protocolo 0.180.0 aditivo: `FileContext.drift[]` opcional/vazio por padrão,
  com `{ kind, configured, effective: { value, origin }, evidence, action? }`.
  Os kinds desta fatia são `compiler`, `target` e `sysroot`; ação `inspectKit`.
  O core deriva a lista em `index/context`, antes de completar lacunas com o
  kit. Não persiste diagnósticos nem lê outra CDB. Ausência de unidade ou de
  valor explícito, e valor herdado do kit, não provam divergência.
- O `--target` da unidade é comparado literalmente com o triple do kit:
  o aviso fala de opções distintas, não de incompatibilidade de ABI.
  Sysroots absolutos são comparados com links resolvidos quando disponíveis;
  um sysroot relativo da unidade usa seu diretório de trabalho. Kit relativo
  ou caminho não resolvido com `..` não gera conclusão. Compiladores só
  divergem quando ambos têm identidade de arquivo resolvida; nome solto no
  PATH e wrapper não são adivinhados. A identidade de arquivo em `platform`
  é compartilhada com o casamento de candidatos de `toolchain/project`.
- Medido com `clang++ -###`: opções repetidas usam o último `--target` e
  `--sysroot`; o extrator efetivo existente passa a respeitar isso, sem
  segundo parser. Os testes protegem as formas colada/separada e repetida.
- O modelo do inspector apresenta as diferenças em primeiro lugar, com
  configurado, efetivo, origem e evidência. O `ProjectHealthController`
  recebe o mesmo `FileContext` aceito pelo índice e oferece “Inspecionar”;
  confiança, ferramentas ausentes, configure e CDB velha mantêm precedência.
  A dispensa distingue arquivo e conteúdo das diferenças, mas uma resposta
  idêntica não ressuscita o aviso. Trocar arquivo/projeto limpa os dados.
- Resposta de alteração/consulta do kit pede novamente o contexto ativo;
  configure continua usando a recarga do índice existente. O botão só abre
  o inspector/kit: nenhum configure, gravação ou instalação automática.

**Fronteira:** esta fatia cobre deriva por arquivo de C/C++. O aviso de alvo
Rust ausente continua no dono `toolchain.get`/`rustTargetHint`; incorporar
fatos de instalação sem executar `rustup` por arquivo será outra fatia.
Launcher ausente depende do schema 3/cache da sequência. Comparar triple
inferido de compilador e compatibilidade de ABI requer evidência adicional;
não se deduz do nome do executável. Próximo: schema 3, sem trocar a ordem.

**Provas previstas:** dispatcher real com kit nomeado, divergência e acordo,
sem unidade, flags repetidas, caminhos relativos/aliases e informação
insuficiente; atualização após alteração de kit e configure. Harnesses de
apresentação, prioridade/dispensa da faixa e ação de inspeção; mutações em
cópias temporárias; fotos antes/depois e gestos no Xvfb; gate rápido estrito.

**Ajuste de implementação:** a entrada de ferramentas do contexto ainda se
chamava `Ferramentas`. Para usá-la nos testes novos sem ampliar o legado de
identificadores em português, o tipo passa a `ContextTools` no dono e nos
consumidores. Só o nome interno muda; baseline e contrato ficam
inalterados.

A catraca de duplicação também exige que a identidade da dispensa venha do
próprio aviso: `apply` recebe uma chave opcional com caminho/diferenças,
montada no ramo da deriva. A dispensa genérica não interpreta o nome da
ação, que pertence ao despacho; nenhuma exceção nova na baseline.


**Resultado (2026-10-19):** 0.4.1c encerrada no checkout. Comparação por
arquivo, inspector e aviso de saúde provados com os donos reais; seis
mutações reprovaram. Gate rápido estrito TUDO VERDE, sem NÃO PROVADO;
1189 testes Rust, 8 C++ e 153 harnesses QML. Provas e notas de método no
[40.7 §7.320](40.7-registro-das-entregas.md). Próxima fatia: schema 3 dos
kits, mantendo as fronteiras acima.

### 14.11 Bancada física do autor — ESP32 por USB/serial (2026-10-19)

**Pedido anterior à preparação:** o autor conectou um ESP32 simples e
confirmou que será o único hardware disponível. Pediu configurar ESP-IDF e
afins e acompanhar os gestos na Kinein do notebook, usando a sessão real de
teclado/mouse. Este aceite usa o desktop real; não usa Xvfb nem máquina
virtual. A sequência de produto continua no schema 3 dos kits (§11).

**Estado inicial medido:** conversor CP2102 em `/dev/ttyUSB0`, sem acesso
para o usuário `hugh` (grupo `dialout`, usuário fora dele); isso identifica
a interface, ainda não o chip. PlatformIO Core 6.2.0 presente, sem plataforma
ESP32; ESP-IDF ainda ausente. Nenhuma janela Kinein estava aberta.

**Preparação:** projeto de diagnóstico separado em
`~/Projetos/esp32-idf-teste`, sem GPIO presumido; saída de contador pela
serial. ESP-IDF e ferramentas oficiais no usuário, versão fixada após
consulta às fontes oficiais; nenhuma alteração no Python do sistema ou no
shell global. Abrir o projeto na janela real da Kinein, preparar build e CDB,
verificar contexto e permissões pelos donos existentes. Antes de gravar,
apresentar o firmware concreto e confirmar a substituição do conteúdo atual
da placa. Outros chips/sondas continuam sem aceite físico; debug por JTAG
depende da interface efetivamente identificada, sem prometer uma sonda pelo
conversor serial.

**Fontes para a instalação:** [ESP-IDF no Linux](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/get-started/linux-setup.html)
e [EIM por linha de comando](https://docs.espressif.com/projects/idf-im-ui/en/latest/headless_usage.html),
consultadas em 2026-10-19. Resultado somente após execução.

**Retomada após perda do contexto (2026-10-19):** checkout ainda em
`4a1f826`; a instalação EIM de `v6.1`, seu ativador e o projeto de
diagnóstico já estão no disco. O README da bancada registra uma identificação
anterior como ESP32-D0WD-V3 rev. 3.1, flash 4 MB; esta retomada ainda não
repetiu essa leitura. A porta continua em `/dev/ttyUSB0`, sem acesso para
`hugh`. A ferramenta de controle de desktop desta sessão devolveu inventário
vazio de aplicativos e navegadores, com controle nativo desabilitado; portanto
os gestos na sessão real continuam sem aceite. Próximo passo desta retomada:
ativar o ambiente isolado, compilar o diagnóstico e conferir CDB/receita de
gravação; depois resolver o acesso serial com o autor. Nenhuma gravação foi
autorizada por este pedido de retomada.

**Medição da retomada (2026-10-19):** o autor executou o comando de ACL e
`os.access` confirmou leitura/escrita em `/dev/ttyUSB0`. O diagnóstico compilou
com ESP-IDF v6.1 e GCC Xtensa 15.2.0: CDB com 901 unidades, aplicação de
142192 bytes, partição de aplicação de 1 MiB; receita DIO/40 MHz/4 MB com
bootloader em `0x1000`, tabela em `0x8000` e aplicação em `0x10000`.
Não houve gravação nem nova identificação nesta retomada. O teste direto do
esp-clangd revelou cinco opções GCC desconhecidas e falta de `float.h`;
preparar `.clangd` local conforme a orientação oficial da Espressif
(`CompilationDatabase`, remoção das opções incompatíveis e
`BuiltinHeaders: QueryDriver`), preservando a CDB original. Fonte:
[Espressif-IDE LSP](https://docs.espressif.com/projects/espressif-ide/en/latest/additionalfeatures/clangd_cdt_support.html),
consultada em 2026-10-19. A ativação EIM com `$0=idf` falhou; o caminho
legado `export.sh` também reportou ambiente/ferramentas ausentes. O build
direto pelo ativador EIM com shell bash passou; o fluxo de build da IDE
precisa ser verificado na janela real antes de ser declarado pronto.

**Aceite na sessão real iniciado (2026-10-19):** depois de o autor
autorizar explicitamente `xdotool`, a Kinein do checkout foi aberta no
XWayland do desktop real (`DISPLAY=:1`, sem Xvfb, HOME/XDG reais). A captura
da própria janela usa seu pixmap XComposite. Foram acionados pela UI:
confiar no diagnóstico revisado, abrir Embarcados, identificar, selecionar
os compiladores GCC/G++ Espressif
Xtensa, reiniciar LSP, compilar pelo menu e montar prévias sem gravar.
O botão Identificar leu ESP32-D0WD-V3 rev. 3.1, mas falhou ao iniciar o
stub; `flash-id --no-stub --baud 115200`, digitado no terminal da própria
Kinein, confirmou cristal 40 MHz e flash 4 MB. O build pelo menu terminou
com `Project build complete`; o editor ficou sem erros e com um aviso de
include não usado. A checagem avulsa do clangd ainda retorna um erro do
tweak `SwapBinaryOperands` na macro `pdMS_TO_TICKS`, sem erro de parsing.

**Ajuste local seguinte:** habilitar `CONFIG_ESPTOOLPY_NO_STUB=y` no
diagnóstico (defaults e configuração corrente), opção do Kconfig oficial
do ESP-IDF instalado; recompilar pelo menu e deixar a prévia `idf.py`
selecionada. Isso prepara a receita sem gravar. O firmware continua sendo
somente o contador serial; substituir o conteúdo da placa exige pedido
explícito do autor.

**Lacunas medidas para o frontend (§7) e a integração:**

- A escolha de porta aparece como “Executar (MicroPython)” também no
  ESP-IDF, embora a mesma seleção alimente a gravação. Identificar está
  reduzido a ícone; o monitor serial não ficou visível na linha da porta.
- A identidade parcial aparece junto do erro de stub, sem ação de repetir
  pela ROM. A leitura completa pelo terminal não atualiza o estado dessa
  view; ela não pode ser apresentada como identificação completa pela UI.
- O kit oferece o driver genérico `xtensa-esp-elf-gcc`; a CDB usa o wrapper
  `xtensa-esp32-elf-gcc` da mesma instalação. O inspector acusa a diferença
  concreta. O `.clangd` local usa o driver escolhido para consultar headers;
  isso não altera o compilador registrado na CDB nem dispensa a deriva.
- Configure nativo produz `.kinein/build`; o build ESP-IDF produz `build/`.
  A prévia direta `esptool` escolheu a receita de `.kinein/build`, cujos
  binários ainda não foram compilados. A opção `idf.py` usa `build/` e é o
  caminho preparado para esta bancada; falta alinhar os donos e validar a
  existência dos artefatos antes de habilitar a gravação direta.
- O build pela IDE concluiu com o ambiente EIM herdado do lançador, mas o
  banner do `export.sh` legado reportou falha de ativação. Preferência do
  ativador EIM e nome do shell no wrapper continuam pendentes de correção;
  não atribuir esse sucesso a uma ativação autônoma correta.

**Ponto de retomada em 2026-10-20:** recompilação pelo menu da Kinein
concluída com sucesso após habilitar `CONFIG_ESPTOOLPY_NO_STUB=y`; a receita
em `build/flasher_args.json` agora tem `stub=false`, e a saída do build
mostra `--no-stub`. Aplicação de 142192 bytes; não houve gravação. Kit e
chip ficaram salvos no projeto de diagnóstico; o lançador aceita
`KINEIN_QPA_PLATFORM=xcb` para a sessão real com xdotool. Capturas, log da
leitura ROM e manifesto com hashes dos binários preservados em
`~/Projetos/esp32-idf-teste/evidencias/2026-10-19/`. Verificação documental
e sintaxe do lançador passaram; sem mudança de código da IDE, sem commit
ou push nesta preparação. A reformulação solicitada é detalhada no §14.12.

### 14.12 Correções da bancada na interface — desenho em 2026-10-20

**Pedido e limite confirmado:** o autor pediu melhorar o comportamento e a
UX/UI de Embarcados e corrigir o ícone da IDE no alternador de janelas. Após
esclarecer o comportamento atual, decidiu manter o botão por arquivo
exclusivo para scripts; esta fatia não amplia execução de C/C++ por arquivo.

**Antes medido:** janela real sem `_NET_WM_ICON` nem desktop id; painel de
640×560 com todo o conteúdo rolando, ações da porta na mesma linha
insuficiente, seleção chamada “Executar” mesmo em ESP-IDF, todos os motores
numa linha e erro de ferramenta de sonda ocupando a aba da placa serial.
Além disso, uma prévia pode continuar com a porta anterior depois de mudar
a seleção. Fotos da bancada em `evidencias/2026-10-19` do diagnóstico.

**Implementação prevista, sem contrato IPC novo:**

- `main.cpp`, recursos Qt e lançador: publicar o ícone existente e o desktop
  id correspondente (desenvolvimento no lançador, distribuição por padrão),
  preservando nome da aplicação e diretórios de estado. Referência consultada
  em 2026-10-20: [QGuiApplication](https://doc.qt.io/qt-6/qguiapplication.html),
  propriedades `windowIcon` e `desktopFileName`.
- `EmbeddedPanel`/host: painel integrado ao slot lateral da HUD, conforme
  confirmação posterior do autor; substituir o popup, aproveitar os docks,
  largura por janela e mudança de lado existentes. Cabeçalho e navegação
  fixos, rolagem apenas no conteúdo e primeira aba com visão geral e ações
  de preparação. Sonda e depuração ficam em Kit; Placa prioriza a serial.
- `EmbeddedSerialView`: dispositivo em cartão, seleção com nome neutro,
  Identificar e Monitor serial com texto e quebra de linha; arquivos e dica
  de execução MicroPython somente quando esse framework foi reconhecido.
- `EmbeddedFlashView`/controller: motores com quebra de linha, porta visível,
  comando copiável, ação de gerar prévia separada da gravação; trocar porta,
  motor, firmware ou modelo invalida a prévia e descarta resposta atrasada.
- Corrigir os bloqueadores já medidos na preparação: ativador EIM compatível
  com o SDK explícito e shell bash; preferir receita com imagens existentes
  ao configure incompleto mais recente, sem apresentar caminho inexistente
  como pronto para gravar.
- Reusar `ToolchainRoleRow` dentro de Kit para escolher compiladores C/C++ e
  monitor serial sem buscar outro menu. Catálogo e importação devem quebrar
  suas ações na largura do dock. Nos testes da receita completa, a tabela
  binária passou a existir também: manter a leitura do CSV do projeto para
  o mapa de memória quando o artefato localizado for binário.
- Ciclo do dock: trocar de projeto com a janela visível deve renovar a
  descoberta; abrir o monitor conserva o painel ao lado do terminal, sem a
  rotina antiga de fechar um popup. O host visual é dono dessa visibilidade.
  A aba do monitor tem título curto pelo nome da ferramenta; a linha bash
  de ativação continua como tooltip, sem ocupar toda a barra de abas.
  Reusar o modelo de terminais e `TooltipController` para esse detalhe.
- Incorporar à correção da bancada a recuperação de Identificar: somente
  após o erro explícito `Failed to start stub`, repetir a leitura uma vez
  pela ROM com `--no-stub --baud 115200`, que já funcionou nesta placa.
  Manter cancelamento/prazo e comando final no evento existente; nenhum
  firmware é escrito. Testar com esptool falso a recuperação e a ausência
  de repetição para erro de conexão comum. Aceite físico ainda pendente.

**Provas:** harness de prévia (troca de porta com resposta em voo), geometria
da linha serial em largura reduzida, testes Rust dos bloqueadores, build,
gates do projeto e antes/depois na Kinein do notebook. Verificar propriedades
nativas do ícone na janela real. Não presumir aceite de flash/JTAG nem
resultado de inspeção somente por leitura do QML.

**Reuso auditado a pedido do autor (2026-10-20):** `KvSegmentedControl`
já implementa escolha exclusiva com ícone, foco e setas, e substitui a
fileira antiga de chips das abas. O dock usa `ShellDocks` e o host lateral;
`KvPanelFrame`/`KvBackdrop` continuam próprios dos diálogos, sem participar
desta janela integrada. `KvPanelHeader` fica fixo;
`KvInsetSurface` serve à prévia; `KvVerdict` cobre andamento e resultado;
`KvButton`, `KvIconButton` e `KvToggleChip` preservam as interações atuais.
As transições existentes usam `Theme.motionFast`/`motionMedium` e
`easingStandard`; não criar temporizadores contínuos nem outro sistema de
animação nesta tela. `KvFloatingLayer` é o dono de tooltips/menus, não uma
animação genérica, e continua na composição existente. O ícone nativo deve
reusar a marca já embutida nos recursos. Só extrair componente novo quando
dois consumidores concretos compartilharem a mesma responsabilidade.
`FlickableScrollBar`, já usado pelas listas e painéis, também atende ao
corpo do dock; reutilizá-lo conserva a rolagem visual da casca.

**Continuidade autorizada (2026-10-20):** o autor pediu prosseguir até a
0.4.5 (limite ampliado depois para 0.4.9, §11) e atualizar a documentação
ao concluir cada etapa. A correção desta
bancada vem primeiro; depois, retomar as fatias de produto do §11, mantendo
provas e commits separados. Meta persistente registrada nesta conversa.
Retomada a cada cinco horas foi solicitada, mas não está agendada: a sessão
não expõe ferramenta de agendamento. Não confundir a meta ativa com garantia
de reinício após limite do serviço; deixar o próximo passo concreto neste
documento e no handoff a cada fatia.

**Medição da implementação em andamento (2026-10-20):** a janela real da
Kinein, reiniciada no XWayland do notebook, mostrou a visão geral no dock de
400 px, as ações da porta com quebra de linha, Kit com os seletores C/C++ e
monitor, e o comando selecionável. O botão da visão geral concluiu
`idf.py build`; a prévia direta usa `build/flasher_args.json` e suas três
imagens existentes, com `--no-stub`, em lugar do configure incompleto.
`_NET_WM_ICON` contém 64×64 e `_KDE_NET_WM_DESKTOP_FILE` contém o id de
desenvolvimento; não confundir esta propriedade com a inexistente
`_KDE_NET_WM_DESKTOP_FILE_NAME`. Registrado o lançador no perfil do autor,
sem sudo. A confirmação visual do alternador do KDE continua não medida.

Os cliques foram enviados à janela XWayland por XSendEvent, com capturas
XComposite da própria janela. A rodada Rust completa passou 1191 testes;
análise C++, harnesses e fechamento do gate ainda em curso. A primeira rodada encontrou
fixtures de gravação sem imagens, agora representando builds completos.
Nenhum firmware gravado. A identificação com recuperação da falha do stub
ainda é pendência separada desta fatia.

**Limite da evidência visual (2026-10-20):** as capturas XComposite comprovam
a renderização e os cliques enviados à janela por XSendEvent; o aceite
interativo no desktop visível solicitado pelo autor continua pendente.

**Revisão visual apresentada ao autor (2026-10-20):** as quatro abas foram
renderizadas a partir dos componentes QML atuais, em 400 px, com dados
demonstrativos da bancada e sem avisos QML. A janela completa foi capturada
a 1440×900 com uma cópia temporária do diagnóstico, mostrando o dock ao
lado do editor. A pedido do autor, a IDE atualizada também foi aberta no
projeto de diagnóstico com Embarcados na Visão geral e o ESP-IDF ativado.
Essas prévias não substituem a confirmação visual do autor nem o aceite
físico de Identificar. Os cinco harnesses direcionados (layout, prévia,
docks, trilho e terminais) passaram; a rodada geral está em curso.

**Polimento solicitado pelo autor (2026-10-20), antes do código:** reduzir
a interpretação exigida no primeiro contato e exercitar ações e falhas.
No refinamento seguinte, o autor retirou o ensaio de sensor/relé: a prova
prioriza a integração ESP-IDF/ESP32 e o frontend. Depois, conferir a cadeia
MicroPython e CPython existente e completar lacunas medidas. O autor
esclareceu que CircuitPython também entra; ele tem detecção e transporte
próprios, sem ser tratado como um alias de MicroPython.

- Visão geral: estado de cada etapa e uma próxima ação coerente com o que
  falta. SDK ausente leva à preparação do ambiente; artefato ausente leva
  à compilação. Não tratar SDK detectado como prova de compilação funcional.
- Gravar: escolha automática explícita; motores alternativos sob demanda,
  comando revisável e consequência de salvar/ativar descrita no botão.
  Manter porta, avisos e resultado da proposta visíveis.
- Kit: distinguir as escolhas usuais dos campos avançados de alvo,
  depuração e SDK; conservar os donos e componentes atuais. Atualizar é
  ação secundária, sem competir com a próxima ação da jornada.
- Exercitar uma prévia seguida de nova identificação da mesma porta: a
  capacidade da flash pode mudar os avisos da proposta. Invalidar também
  esse contexto antes de permitir Gravar/Salvar; provar com regressão.
- Corrigir a dica de ausência de artefatos: BIN/HEX e receita completa também
  são artefatos; projetos MicroPython não exigem compilar firmware a cada
  script. A visão geral deve orientar à execução no editor, sem oferecer
  Compilar firmware para um projeto MicroPython.
- Conferir os argumentos de `mpremote` pelo despacho real, incluindo porta
  com espaço/apóstrofo: `run.start` monta uma linha de shell, enquanto
  `run.script` passa argumentos diretamente. A mesma porta precisa chegar
  inteira nos dois caminhos.
- A identificação física continua distinta de gravar firmware. Preparar e
  revisar um artefato concreto antes de pedir autorização para substituí-lo;
  não usar o pedido de validação como autorização implícita de gravação.
- O gate de fiação IPC encontrou o desktop id como se fosse método novo:
  ele varria literais de qualquer C++ da aplicação. Restringir a extração
  aos arquivos da ponte `core_client*` e da CLI, os donos dos pedidos, sem
  criar exceção para esse id. Provar que um método desconhecido da ponte
  continua sendo detectado e que um id de aplicação não vira pedido.
- A revisão do caminho alternativo `idf.py flash` encontrou uma segunda
  montagem do wrapper ainda com `$0=idf`, embora build/monitor já usem bash.
  Reusar `build::engine::idf_command_args` na proposta do wrapper, preservando
  aspas de todos os argumentos; provar com ativador e idf.py falsos que o
  shell é bash e a porta com espaço/apóstrofo chega inteira. Não executar
  esse teste com o gravador real ou com a placa.

**Ampliação da HUD pelo autor (2026-10-20), antes da próxima fatia:**
Embarcados abrange também Linux embarcado. A visão geral deve distinguir
microcontroladores, Python na placa, PlatformIO e Linux/SSH, usando fatos do
modelo e os donos atuais de build, execução, serial e remoto. Yocto/Buildroot
não devem receber uma jornada obrigatória de porta USB e flash de MCU.
PlatformIO precisa de acesso visual claro aos ambientes e às tarefas;
**destaque dentro de Embarcados**, confirmado pelo autor, abrindo a mesma
superfície integrada. A animação acompanha abertura,
mudança de contexto e progresso com os tokens existentes; não executa
descoberta contínua nem deve deslocar o conteúdo durante a leitura.

**Estado de verificação da reformulação (2026-10-20):** implementação
concluída no checkout (40.7 §7.321). Passaram 154 harnesses QML, as regressões de identificação/prévia,
artefatos e argumentos MicroPython, e o ciclo real de CPython/debugpy (arquivo,
módulo e attach; launch de arquivo em 2045 ms). Uma cópia temporária do
diagnóstico foi compilada pelo `build.run` real, com ESP-IDF v6.1 e duas
tarefas de build: sucesso em 69,19 s, zero diagnósticos, três imagens e
proposta esptool com `--no-stub`. A porta nessa proposta é parâmetro de
montagem do comando; nenhum acesso físico ocorreu. A descoberta USB da
última rodada estava vazia, após uma rodada anterior detectar a CP2102 sem
permissão de leitura/escrita. Não há novo aceite físico de Identificar.

Na UI real em Xvfb próprio, renderização por software e estado isolado,
foram exercitados Visão geral, Placa, Gravar sem porta (erro visível) e
Ambiente a 1440×900; primeiro frame em 471 ms. O ativador IDF foi herdado,
e o ambiente passou a aparecer detectado. A primeira sonda visual sem esse
ambiente corretamente apontava ferramentas ausentes. Isso não é comparação
de desempenho entre as duas configurações. Capturas e provas temporárias
em `/tmp/kinein-embedded-preview/visual-final` e
`/tmp/kinein-esp32-orchestration`. As janelas de teste já foram fechadas;
a janela nativa do autor não foi fechada nem reiniciada nesta rodada.

O build release da UI concluiu; core/adaptador release também compilaram.
O gate rápido estrito passou TUDO VERDE: 1191 testes Rust, 8 C++ e 154 QML,
sem NÃO PROVADO. As suites Rust foram reexecutadas após a correção final do
wrapper `idf.py flash`. A rodada anterior
parou numa expectativa antiga do teste uv após a correção das aspas; outra
parou na fiação IPC confundindo desktop id com método. Ambas as causas
foram corrigidas, com regressões. O protocolo permanece 0.180.0, sem método/evento novo.

No editor real release, a prova de digitação inseriu 40 caracteres no arquivo
de 2463 linhas/56226 bytes: mediana tecla→quadro de 9,45 ms e p95 de 13,27 ms,
offscreen/software, com análise C++ concorrente. É uma medição do custo local,
sem vsync, não uma alegação de ganho nem a latência no compositor do desktop.

### 14.13 HUD por contexto e destaque PlatformIO — desenho em 2026-10-20

**Origem:** refinamento explícito do autor depois da reformulação inicial:
Linux embarcado faz parte de Embarcados; PlatformIO fica em destaque dentro
do painel. Não acrescentar uma segunda entrada de trilho ou uma segunda
instância da área para o mesmo projeto.

**Medido na partida da fatia:** `project.target.family` já diz `linux` para
Yocto/Buildroot. `EmbeddedOverviewView` oferecia a todos a sequência
ambiente → porta serial → firmware; isso induz o fluxo errado para Linux.
`EmbeddedController` já possui frameworks, alvo, SDKs e artefatos; os comandos
`remote.list`, `terminal.open`, `build.run` e `configAction.list` já chegam
aos donos. PlatformIO tem build/upload/monitor e a Configuration Action
`platformio.compiledb`; test/check ainda pertencem à 0.4.2. Não prometer uma
tarefa habilitada se o handler ainda não a executa.

**Desenho da implementação:**

- Derivar a jornada do framework principal/alvo do modelo existente. Um
  exemplo Python aninhado não pode mudar sozinho a jornada principal de um
  projeto CMake/PlatformIO. Não analisar `platformio.ini` no QML.
- Visão geral para microcontrolador mantém as ações de preparação, conexão,
  build e revisão de gravação. Python na placa orienta ao editor e ao
  transporte adequado. Linux troca a exigência de porta/flash por ambiente,
  imagem/aplicação e alvo remoto. Console UART continua opcional.
- Apresentar o destaque PlatformIO com evidência do projeto e acesso às
  tarefas dentro da mesma área. Build usa `build.run`; upload passa pela
  prévia de `runConfig.flashProposal`; monitor usa a porta escolhida.
  Ambientes e tarefas detalhadas exigem modelo estruturado do core, com
  contrato antes de acrescentá-lo. O texto `detail` não vira parser na UI.
- Para Linux, navegar para os alvos SSH e para o terminal pelos comandos
  existentes. Geração de imagem Yocto/Buildroot não vira botão fictício:
  distinguir suporte já disponível de orquestração ainda por implementar.
- Transições curtas com `Theme.motionFast`/`motionMedium` e easing existente:
  foco e posição de rolagem previsíveis, sem piscar o conteúdo durante um
  refresh nem animar tamanho de listas continuamente.
- Para Linux, expor alvo e sysroot e a importação do SDK nas escolhas usuais;
  chip/SVD/sonda não são pré-requisitos do fluxo SSH. Carregar ferramentas
  avançadas sob demanda, conservando a instância e os rascunhos entre abas
  do mesmo projeto; trocar de workspace deve descartá-los. Medir o ganho,
  sem chamar de otimização algo que só mudou a aparência.

**Provas previstas:** harnesses da tabela de contextos (ESP-IDF, PlatformIO,
MicroPython, Yocto, Buildroot e projeto misto), gestos que chegam ao comando
correto e ausência de gravação implícita; geometria a 400 px e janela menor;
estados sem ferramenta/sem alvo/em andamento; antes/depois no Xvfb e medidas
de primeiro frame e tecla→quadro. Registrar o resultado e o próximo passo no
handoff ao concluir a tarefa, conforme o pedido do autor de 2026-10-20.

**Prévia inicial, antes da implementação (2026-10-20):**
`/tmp/kinein-hud-next` contém a proposta QML, o teste de contextos e fotos
comparáveis com dados demonstrativos para ESP-IDF/PlatformIO/Yocto. Na
versão atual o teste falha pelo destaque inexistente e por um exemplo
MicroPython mudar a classificação da UI; na proposta passa pelos comandos
de build/setup, revisão sem execução, navegação SSH e campos do SDK Linux.
Naquele momento, o protótipo não mudava o checkout; a seleção detalhada
de ambientes continua pendente.
Na aplicação, conferir a chave real do SDK: `sdk::requisitos` entrega `pio`,
não `platformio`. A fixture da UI deve usar a mesma chave para não aprovar
um estado de instalação impossível no produto. A classificação MicroPython
do painel e do despacho de execução/monitor deve seguir o framework
principal; um exemplo aninhado não pode trocar o runtime do projeto.
Revisão adicional do modelo: `build::engine` já dá precedência ao PlatformIO,
mas a ordem de `detect::frameworks` ainda favorece ESP-IDF. Yocto/Buildroot
ficam depois de MicroPython. Alinhar essa ordem ao dono efetivo: PlatformIO
antes dos frameworks que gerencia, e todos os frameworks nativos/Linux
antes do exemplo MicroPython. Os SDKs de PlatformIO são geridos pelo `pio`;
os de um exemplo MicroPython não são pré-requisitos do build nativo/Linux.
Provar ordem, alvo e requisitos com a mesma fixture mista, além do despacho
de script auxiliar. É correção das deduções existentes, sem campo/método novo.
Ao estender o teste do ambiente de SDK, migrar o tipo interno `Ambiente`
para `Environment` e o callback `binario` para `binary`, no dono e nos usos:
o gate exige identificadores novos em inglês, sem ampliar a dívida legada.
A rodada de build/flash do wrapper EIM continua sendo provada separadamente
no despacho Rust. Preservado o binário UI release anterior em
`/tmp/kinein-hud-before-ui` para a comparação de desempenho nas mesmas
condições depois da implementação; não compilar UI enquanto roda a análise
C++ da fatia anterior.

**Aplicado e fechado na 0.4.1e (2026-10-20):** destaque PlatformIO,
jornada Linux, campos de SDK/sysroot visíveis para Linux, carregamento do
formulário Ambiente no primeiro acesso com preservação entre abas, detalhes
sob demanda e transição curta nas abas. A chave `pio` é a do SDK real.
Os testes de regressão reproduziram antes as falhas de contexto/SDK da UI,
o redirecionamento de script auxiliar para mpremote e os requisitos indevidos
do exemplo MicroPython; após a correção os testes direcionados passaram.
A ordem do modelo agora acompanha o dono do build e os requisitos geridos
pelo PlatformIO. Compilação UI release passou; o fechamento integral abaixo
supera o ponto de handoff parcial. Os gestos nativos são aceites do agente,
separados do feedback do autor.

**Revisão visual real (2026-10-20):** UI release e core atual, em Xvfb
próprio/software a 1440×900: destaque PlatformIO em projeto também ESP-IDF,
abas/campos Linux em Yocto, sem gravação nem implantação. Fotos locais em
`/tmp/kinein-hud-acceptance/`; primeiro frame de 422 ms (PlatformIO) e
425 ms (Yocto), uma amostra por projeto, sem afirmar ganho. Os processos
continuaram vivos e sem avisos QML proibidos.

**Primeiro ponto de handoff, superado pelo fechamento abaixo (2026-10-20):** a rodada do gate
rápido passou 1193 testes Rust, Clippy, análise limpa dos 47 arquivos C++ e
8 CTest, e parou no qmllint: `Loader.item` era tipado como `QObject` ao
chamar `applyKit`. O cast para `EmbeddedEnvironmentView` foi aplicado;
depois dele o build debug, qmllint estrito e os **155 harnesses QML**
passaram. Naquele ponto ainda faltava uma execução integral verde de
`verificar.sh --rapido` após a correção e as contradições visuais abaixo
continuavam abertas; o fechamento abaixo registra a rodada subsequente.
Logs locais `/tmp/kinein-hud-gate-final.log`,
`/tmp/kinein-hud-qmllint-final.log` e `/tmp/kinein-hud-qml-final.log`.

**Medida comparável, sem ganho material demonstrado (2026-10-20):** três
rodadas por binário release, antes (0.4.1d) e depois, mesmo arquivo de
2463 linhas/56227 bytes, 40 inserções em cada rodada, offscreen/software.
Mediana das medianas tecla→quadro: 9,02 → 8,975 ms; mediana dos p95:
9,86 → 9,77 ms. Ambiente com builds concorrentes, não compositor físico.
O cenário não abre painel durante a digitação; a primeira tentativa com
abertura no startup perdeu caracteres já na base e foi excluída por
misturar tarefas. Dados em `/tmp/kinein-hud-performance/report.json`.
No harness PlatformIO, os nós visuais iniciais caíram de 599 para 343
com conteúdo contextual e carga sob demanda; isso não mede RSS nem ganho
de latência. A versão release final foi medida novamente no fechamento:
três rodadas por binário, mesmas 40 inserções e arquivo. Mediana das
medianas 8,625 → 8,725 ms; mediana dos p95 9,95 → 9,84 ms, sem ganho
material demonstrado. Dados `/tmp/kinein-hud-performance-final/report.json`;
as amostras variam e não medem o compositor real.

**Achados da primeira revisão, corrigidos nesta fatia:** o cabeçalho expunha o
`workspace.kind` CMake quando o motor efetivo é PlatformIO. Essa precedência
de marcadores está no contrato `workspace.*`; corrigir a apresentação do
contexto efetivo sem mudar silenciosamente a classificação compatível.
No Ambiente Linux, seletores automáticos C/C++ diziam que o CMake
escolheria ao configurar: distinguir SDK/imagem gerida de aplicação CMake e
apresentar controles relevantes. A revisão cobrou também uma aplicação CMake cross com kit Linux além
das fixtures Yocto/Buildroot; prova de compilação e fotos no fechamento
abaixo. Isso não anuncia implantação remota nem uma imagem Linux pronta.

**Complemento antes da correção (2026-10-20):** o cabeçalho receberá o nome
do framework principal do `EmbeddedController`, mantendo os sistemas de
build do workspace como fallback. Não alterar `workspace.kind` nem copiar
a detecção do core para a UI. No Ambiente, PlatformIO e imagem Yocto/Buildroot
explicam quem gerencia seus compiladores; o seletor do compilador CMake
permanece na aplicação CMake com SDK Linux. O aviso de alvo Rust só aparece
no contexto Rust. Formulário e rascunhos preservam o ciclo de vida existente.

O modelo hoje ignora `targetTriple` do kit e só recebe o chip; uma aplicação
CMake com kit `aarch64-linux-gnu` aparece como projeto não embarcado. Passar
o triple do kit efetivo, escolhido pelo mecanismo `cmake::active_kit` já
usado pelo contexto do editor; preencher os campos existentes `target.triple`,
`target.family` e `embedded`, com evidência. O catálogo de instalação deve
usar o mesmo modelo computado. Após salvar kit, emitir o evento de modelo
existente para o painel atualizar sem reabrir. Nenhum método/campo novo.
Provar aplicação CMake sem marcador de framework, kit padrão e nomeado ativo,
troca de triple, evento e ausência de motor de gravação MCU em Linux.

**Fechamento da 0.4.1e (2026-10-20):** correções acima aplicadas; gate rápido
estrito integral **TUDO VERDE**, sem NÃO PROVADO: **1194 Rust**, Clippy,
47 arquivos C++ sem achados, 8 CTest, qmllint estrito e 155 harnesses QML,
além dos ciclos reais do gate. Log `/tmp/kinein-context-gate.log`.
O contexto/SDK da UI e o despacho de script misto falharam antes e passaram
depois; a aplicação CMake com alvo Linux também falhou antes de propagar
o triple. O teste do kit nomeado simula cache/preset para isolar o modelo;
não representa um configure real desse preset.

**Exercitação real adicional:** aplicação C17 por `build.run`, compilador
`aarch64-linux-gnu-gcc` instalado, kit Linux explícito, zero diagnósticos;
`readelf -h` confirmou ELF AArch64. O modelo não inventou framework nem
motor de gravação de MCU. Sem execução/deploy no alvo, sem alegar SDK de
imagem Yocto completo. Dados `/tmp/kinein-linux-context-proof/report.json`.
UI release/core debug atualizados, Xvfb próprio/software 1440×900: fotos
PlatformIO/Yocto em `/tmp/kinein-hud-acceptance/` e aplicação Linux em
`/tmp/kinein-linux-context-proof/ui-shots/`, sem avisos QML proibidos.
Compilador C do kit visível e aviso Rust ausente na aplicação Linux;
compiladores gerenciados explicados e fora dos seletores CMake na imagem/PIO.
Schema 3 e as jornadas seguintes permanecem no §11; escolha detalhada de
ambientes PIO, test/check/debug e integração completa de imagem/remoto não
foram antecipadas por esta fatia de contexto/HUD.

### 14.14 Auditoria de C puro antes do fechamento — 2026-10-20

**Pergunta do autor:** o ecossistema de C está completo ou ainda faltam
básicos? A leitura do código confirma a fundação compartilhada C/C++:
compiladores e kits cross, CMake/Make, clangd, diagnósticos ao salvar,
clang-format, depuração e CTest. Isso não prova uma jornada de C puro.

**Decisão do autor, 2026-10-20:** incluir as pendências de C na série 0.4,
antes do fechamento da 0.4.9 (§11, limite ampliado pelo autor). A jornada
visual deve ser prática e intuitiva,
com configuração simples e explicação do que acontece; terminal permanece
como escolha normal. O mesmo critério vale para bare metal e Linux embarcado.
O assistente C e a configuração visual do padrão C entram como fatia própria;
ajuda de parâmetros/formatação de seleção permanecem na 0.4.4 e as tarefas
PlatformIO de testes/check na 0.4.2. O aceite é executar a tarefa pela UI;
um comando que só funciona no terminal não encerra a integração visual.
Detalhar a proposta no domínio existente, com contrato antes do código,
e corrigir as lacunas funcionais encontradas nessa jornada dentro da 0.4.

O autor reiterou que o ecossistema de embarcados deve ser completo na parte
de software/ferramentas existentes: a IDE orquestra os executáveis e
protocolos, sem reimplementar essas ferramentas. O inventário continua no
41/42 e em integracoes/39; o critério de integração é nativo e plug and play
quando os pré-requisitos estão disponíveis. Ausência de ferramenta,
permissão ou alvo deve aparecer com estado e próximo passo coerentes na UI.

**Lacunas medidas:** `WorkspaceProjectTemplate` só oferece `CppCmake`,
Rust, Python e pasta vazia; a ação de padrão CMake usa
`CMAKE_CXX_STANDARD`, sem equivalente de C. Ajuda de parâmetros e formatação
de seleção pelo LSP são pendências da 0.4.4 (§5.11); a formatação do arquivo
inteiro via clang-format já existe. PlatformIO test/check continuam na
0.4.2. Não confundir uma opção configurável manualmente no CMake/CDB com
um fluxo visual já implementado.

**Primeira prova real (2026-10-20):** fixture temporária apenas C, C17
estrito (`-std=c17`, sem extensões), com `.c`/`.h`, `_Generic` e
`_Static_assert`. Pelo core real: `workspace.open`/trust, `build.run`,
`run.start` (saída `c-basics: 5`, exit 0), `test.discover` e `test.run`
(dois casos CTest passaram), `format.text` com clang-format e
`lsp.definition`/hover entre header e implementação passaram. Um erro de
sintaxe introduzido de propósito fez o build falhar com um diagnóstico;
o fonte foi restaurado ao final. Evidência local em
`/tmp/kinein-c-basics/report.json`; prova auxiliar em
`/tmp/kinein-prove-c-basics.py`. Não cobre o assistente ausente, depuração
de C puro nem gestos na interface. O clangd resolveu C corretamente com a
CDB desta fixture; auditar o languageId no wire continua previsto.

**Aceite adicional no fechamento da 0.4:** exercitar um projeto apenas C,
com `.c` e `.h`, desde criar/abrir até configurar, compilar, executar,
diagnosticar erro, navegar, formatar, descobrir/rodar testes e depurar.
Conferir o languageId e os flags recebidos pelo clangd; o registro atual
compartilha a chave `cpp`. Registrar as lacunas como pendentes até haver
implementação e prova; o assistente C e a ação de padrão C precisam de
desenho/contrato antes do código, sem substituir a jornada existente C++.

### 14.15 Integrações nativas opcionais — decisão de 2026-10-20

**Direção do autor:** o usuário pode ativar/desativar as integrações nativas;
elas devem ser profundas e naturais dentro da IDE. Na 0.5 a gestão será
refinada e integrações que ficaram de fora poderão entrar. Isso não cria
um runtime de plugins nem altera os donos existentes de build, testes,
execução, gravação, serial, depuração e remoto.

**Desenho a concluir antes da implementação:** preferência persistida nas
Configurações, com estado efetivo vindo do core e consumido pela UI. Usar
os modelos de settings/capacidades existentes; contrato antes de adicionar
campos. Distinguir reconhecimento do projeto, integração habilitada e
ferramenta instalada: desativar não significa desinstalar nem ferramenta
ausente. A descoberta automática apresenta o contexto sem obrigar ativação
manual repetida; a escolha explícita do usuário prevalece.

Uma preferência deve alcançar todos os caminhos do mesmo recurso, incluindo
paleta/atalho, painel e IPC; ocultar um botão sozinho não desativa uma
integração. Preservar visualização e controle dos jobs já iniciados, com
regra explícita para novos pedidos. Estado, consequência e próximo passo
devem ser claros, com opções avançadas acessíveis e terminal disponível.
Validar persistência, reabertura, troca de workspace e ausência de contradição
entre capacidades, ações e mensagens. **Ainda não implementado:** esta seção
registra a direção e os critérios, sem anunciar a preferência como entregue.

Desativar uma integração automática não proíbe o usuário de executar seus
próprios comandos no terminal. Também não deve causar fallback silencioso
para outra ferramenta: por exemplo, MicroPython desativado não pode fazer o
mesmo script de placa rodar em CPython. Essa distinção precisa de regressão
no core, além do estado visual da preferência.

### 14.16 Navegação compacta dos painéis — prévia solicitada em 2026-10-20

O autor quer visualizar alternativas para dar feedback de praticidade e
modernização: ícones/abas recolhidas à esquerda, abrir só o conteúdo da seção
escolhida, ou menu hambúrguer que oculta as opções. Preparar comparação
interativa da mesma tarefa, mantendo também as abas atuais como referência
e uma alternativa de seções recolhíveis. Seleção ativa, título da seção,
nomes expansíveis, teclado e foco devem continuar compreensíveis.

Referência MODE-D consultada: [IntelliJ IDEA 2026.2, New UI](https://www.jetbrains.com/help/idea/new-ui.html),
em 2026-10-20: faixas de ícones com nomes opcionais, menu principal recolhível
e apresentação progressiva das funções. O produto e os tokens existentes
continuam sendo a base; a referência informa o padrão, sem importar seu código.

A prévia deverá usar dados demonstrativos e navegação local, para comparar
área útil e acesso às funções em ESP-IDF, PlatformIO e Linux. Caminho local
planejado: `.kinein/design-previews/embarcados-navigation.html` (ignorado pelo
Git). **Atualização de 2026-10-20:** arquivo criado, com quatro variantes
e navegação local; JavaScript e 25 verificações de interação/layout por
largura passaram em WebKitGTK a **1024, 736 e 320 px**, com Xvfb próprio/
software. Fotos locais `/tmp/kinein-navigation-*.png`, logs
`/tmp/kinein-navigation-test-*.log`. Preparada para apresentação na conversa;
o autor recebeu também o comando para abrir a UI nativa. A escolha de
layout depende do feedback, sem substituir as abas nativas nesta fatia. Não substitui a validação nativa nem anuncia um
layout aprovado. A UI Qt continua com as abas da fatia em validação até
incorporar o feedback. Este pedido acrescenta revisão visual ao trabalho
em curso; os aceites funcionais e o fechamento da 0.4 continuam obrigatórios.

### 14.17 Auditoria geral de frontend antes do AppImage — decisão de 2026-10-20

**Decisão consolidada do autor após o handoff, em 2026-10-20:** cada
escopo fecha eficiente, funcional e com UI/UX adequada; a 0.4.9 entrega a
base pronta. A revisão integrada desta seção permanece antes do AppImage.
**Após concluir a 0.4.9, haverá uma etapa especial de revisão de HUD/UX/UI**
para procurar algo esquecido: componentes com estilo/customização antigos,
estados ou superfícies que não tenham sido atualizados e inconsistências
com o contexto visual moderno da IDE. O autor a chamou de “edição especial”.
Seu objetivo é conferir a cobertura visual do conjunto pronto e corrigir
os achados justificados, preservando as jornadas e o desempenho validados.
**Atualização de 2026-10-21:** o autor associa essa revisão especial à
**0.5**; identificação exata do artefato, patch e cronograma continuam sem
definição. Instalar Ferramentas e Quality Center inicial entram na própria
0.4.x (§11.1); a revisão especial melhora/otimiza a base e detalhes que
escaparam. **Decisão posterior na mesma data:** ela vem imediatamente após
a 0.4.9 e estabelece a base da 0.5 em diante; o AppImage só é gerado/testado
após seu fechamento, para oferecer o conjunto polido aos usuários de teste.
A etapa especial não transfere para depois os bugs, as lacunas
funcionais ou a usabilidade necessários ao aceite da 0.4.9.

O autor pediu examinar toda a IDE: componentes com aparência antiga,
comportamentos que prejudicam o uso, HUD, aproveitamento do espaço,
estilização, animações, interações, resposta, informações, cores do editor e
atalhos. Com a extensão até a 0.4.9, este plano coloca a auditoria nas
0.4.7/0.4.8, **depois da consolidação funcional e antes do pacote**, mantendo
a revisão de bugs e o polimento necessário em todas as fatias. O objetivo
é fechar a qualidade da série; a etapa especial posterior confere se algum
componente ou comportamento escapou dessa atualização visual. A direção é moderna e discreta:
o usuário deve reconhecer contexto, estado e próxima ação ao olhar; a
interface deve preservar o foco no trabalho. A praticidade JetBrains é
referência a adaptar aos componentes e à identidade da Vectis.

| Frente | Revisão e critério de aceite |
| --- | --- |
| Componentes e estilo | Inventariar as superfícies reais e seus estados; corrigir inconsistências de tipografia, espaçamento, controles, ícones e densidade com os tokens existentes. Aparência moderna deve acompanhar legibilidade e função |
| Painéis e navegação | Comparar as alternativas do §14.16 com a mesma tarefa e largura; conteúdo ativo ganha área, rótulos/foco permanecem claros, avançado sob demanda. Incorporar feedback antes de generalizar a casca |
| Informação e contexto | Mostrar só o necessário à etapa, com detalhes acessíveis. Cabeçalho, ações e mensagens devem concordar com core, ferramenta e alvo; nenhum estado “pronto” inventado pela UI |
| Interações e animação | Transições breves que expliquem mudança, sem atrasar comando ou competir com editor. Conferir foco, teclado, hover, seleção, carga, erro, cancelamento e reabertura; respeitar preferência de movimento reduzido |
| Resposta e desempenho | Medir primeiro quadro, abrir/trocar painéis, digitação e consumo antes/depois no mesmo cenário; examinar carga sob demanda e jobs concorrentes. Registrar amostras, ambiente e limitações, sem confundir redução de nós com ganho de latência |
| Cores do editor/LSP | Avaliar realce semântico de variáveis, funções, tipos e diagnósticos em código C/C++, Rust e Python; distinguir símbolos com contraste legível e tonalidades confortáveis. Conferir seleção, busca, inativo e diagnósticos nos temas, sem usar apenas cor para comunicar erro |
| Atalhos e acesso | Auditar utilidade, conflitos, nomes da paleta, localização e descoberta. Ações frequentes devem ser acessíveis; avaliar anexar/fixar painéis nos lados e novos atalhos quando houver tarefa concreta, mantendo menu, paleta e bindings coerentes |

**Entrega da auditoria:** inventário com achado, reprodução, impacto e dono;
correções em fatias revisáveis, fotos antes/depois em larguras comparáveis,
gestos visuais e regressões funcionais pertinentes. Recursos apenas
decorativos não encerram os aceites de usabilidade. A prévia com dados
demonstrativos não prova a integração nativa. Registrar o que o agente
exercitou e o que recebeu feedback do autor como evidências diferentes.

Na 0.4.9, repetir as jornadas C, C++, Rust, CPython, MicroPython,
CircuitPython, PlatformIO, MCU e Linux que foram implementadas; confrontar
UI/IPC/core e ferramentas reais disponíveis. Revisar permissões,
pré-requisitos ausentes e operações canceladas. Toda limitação física ou
de ferramenta deve estar explícita na matriz de prova. Fechar bloqueadores,
atualizar os documentos donos e aplicar o corte de entrega do 57 §1.1 antes
da edição especial. Após validá-la, gerar e exercitar o AppImage conforme
`packaging/appimage/` e §10, com identificação/versão do artefato alinhadas.

#### 14.17.1 Direção da edição especial associada à 0.5 — 2026-10-21

**Direção geral decidida, solução visual em rascunho:** pesquisas/imagens
recebidas nesta data não são design final. Refinar sobre a UI real e
registrar aprovação visual do autor, conforme
[frontend §1.1](../especificacoes/arquitetura-de-frontend-0.3-em-diante.md#11-aprovação-visual-e-referências--decisão-de-2026-10-21).

Junção pedida pelo autor: apresentação contemporânea de ferramenta de
engenharia, orientação compreensível para iniciantes e foco no código.
Preservar disposição de barra superior, explorador à esquerda, abas/editor,
ferramentas inferiores e status; continuar com Qt/QML, core Rust, Theme e
componentes Kv*. Aproveitar superfícies por contraste discreto e identidade
escura/âmbar, sem criar outro desktop, dashboard ou biblioteca visual.

| Prioridade de auditoria | O que conferir |
| --- | --- |
| Legibilidade e consistência | Tipografia importante sem tamanho micro obrigatório, hierarquia, alturas, espaçamentos, ícones, raios e estados nos tokens atuais; validar densidade compacta/confortável sem aumentar tudo |
| Navegação e foco | Aba ativa/alterada, seleção, hover, foco de teclado, atalhos descobríveis e retorno ao editor; controles de leitura não parecem ações |
| Espaço de trabalho | Painéis com altura/largura adequadas ao conteúdo, redimensionamento, restauração e pouco ruído; nenhuma mudança estrutural compulsória |
| Diagnósticos e Qualidade | Problema, origem, evidência, explicação e ação; navegação, revisão de correção e resultados verificáveis; polir a base do Quality Center entregue na 0.4 |
| Ambiente e instalação | Clareza de detectado/selecionado/efetivo, disponibilidade e revisão dos comandos; polir Instalar Ferramentas já reformulado na 0.4 |
| Interação e desempenho | Feedback de carga/falha/cancelamento/conclusão; tempos curtos pelos tokens, movimento reduzido, nenhum trabalho pesado em delegate/binding de animação |
| Internacionalização | Português/inglês, pluralização, mensagens, nomes acessíveis e layout com expansão do texto; marco público no 57 |

Faixas do estudo são **hipóteses para teste**, não alteração de tokens já
aprovada: hover 80–120 ms, seleção 100–150 ms e expansão 140–200 ms; abas
compactas 28–30 px e confortáveis 32–36 px. Medir e ajustar nos componentes
atuais. Rever tremor de validação quando competir com explicação/foco;
evitar animação contínua da árvore, código e estados de erro.

Provar as superfícies relevantes a 1280×800, 1536×960 e 1920×1080, com
100%, 125% e 150% de escala, teclado e movimento reduzido. Medir resposta
durante build/análise e conferir QML Profiler quando houver custo de UI;
registrar ambiente, amostras e diferenças, sem anunciar fluidez por aparência.

As referências e o fluxo do instalador têm dono em
[Instalar Ferramentas](../especificacoes/instalar-ferramentas.md). Regras,
perfis e gates têm dono em [Quality Center](../especificacoes/quality-center-e-gates.md).
A experiência ROS 2 foi consolidada no
[integracoes/41](../integracoes/41-ros2-hud-ux-analise-projeto.md), para sua
frente posterior; não é requisito funcional da edição especial. Expansão
ampla de regras/hooks/CI e providers não entra como “pequeno detalhe”.

Pequenos incrementos encontrados na revisão precisam de problema concreto,
dono e aceite próprios. Não são uma fila aberta para aumentar o escopo nem
motivo para reabrir decisões ou reescrever o shell.


### 14.18 Painéis de trabalho e acesso ao ESP32 — pedido de 2026-10-20

**Decisão do autor:** Instalar ferramentas deve ser painel acoplado; as
janelas funcionais abertas como pop-up serão migradas para painéis. Na
clarificação explícita, menus de seleção, dicas e confirmações continuam
compactos. Modernizar textos, hierarquia, destaque das informações importantes
e retorno de erro/problema com transição leve e mensagem que explique a
causa e a próxima ação. Reusar componentes e os donos existentes, estendendo
pontos concretos segundo open/closed; não copiar um segundo sistema de dock,
mensagens, animação ou execução. A revisão especial após a 0.4.9 procura o
que escapar dessa atualização, sem adiar a qualidade de cada escopo.

**Bancada observada, sem entrada global:** captura solicitada do notebook em
`/tmp/kinein-notebook-esp32-feedback.png`. A IDE detecta `/dev/ttyUSB0`,
mas apresenta os botões desativados e várias repetições do diagnóstico.
`stat` e `getfacl` confirmaram `root:dialout`, modo `0660`, sem ACL do
usuário; os grupos do processo da IDE não incluem dialout. É bloqueio real
de acesso à porta, separado de falha de reconhecimento da ponte USB.
Nenhuma abertura serial, identificação, gravação ou alteração de permissão
foi executada nessa conferência. A foto do desktop permanece fora do repo.

**Próxima fatia de UX (antes de schema 3):** separar o veredito principal
“placa conectada; acesso à porta bloqueado” das instruções e da evidência.
Reutilizar o diagnóstico de `serial.access` e manter os detalhes sob demanda,
com retorno claro ao atualizar o acesso. Identificar/monitorar continuam
sujeitos à permissão real; não mascarar um bloqueio do sistema como sucesso.
Inspecionar `selectPort` e os gestos antes de mudar o bloqueio de seleção:
selecionar um destino e operar na porta têm responsabilidades diferentes.

Instalar ferramentas passa de `ShellEnvironmentOverlays` para o slot de
`ShellLeftWindowHost`, usando `ShellDocks`, layout persistido, o mesmo
`SetupController` e as linhas `SetupToolRow`. Preservar editor/terminal,
comandos visíveis e ausência de instalação automática. Listar as demais
janelas funcionais por dono e migrar em fatias, com teclado, foco, estado,
redimensionamento e erro verificados. Não anunciar todas como migradas ao
concluir apenas Setup. Harnesses de navegação e erro, fotos nas larguras de
uso e gate integral antes de fechar a fatia; não fechar/reiniciar a janela
original do autor durante a implementação.

### 14.19 Diagnóstico serial coerente — prioridade do backend (2026-10-20)

**Ordem do autor:** corrigir primeiro o backend; a UI representa o seu
veredito. Modernizar os componentes existentes e só criar extensões quando
houver necessidade concreta. Testar rigorosamente sucesso, recusa e recuperação,
inclusive para uma instalação nova, com preparação prática no painel.

**Defeito medido:** a conta já consta em `dialout` em `/etc/group`, mas o
processo da IDE ainda não tem esse grupo. O core confundia cadastro com
credencial ativa e recomendava cadastrar novamente. Também atribuía acesso
ao udev apenas pela presença da tag, sem provar uma ACL, e o monitor não
revalidava a porta antes de lançar a ferramenta.

**Fatia 0.4.1f, contrato antes do código:** `SerialAccess.state` no protocolo
0.181.0 distingue acesso disponível, sessão desatualizada, cadastro necessário,
permissão negada por outra causa, nó indisponível e plataforma sem suporte.
Uma única avaliação alimenta listagem, diagnóstico e recusa dos três gestos
que abrem a porta. `access(2)` continua dono do sucesso; modo, identidade do
processo (incluindo grupo primário) e cadastro local apenas explicam a falha.
Identidade/cadastro desconhecidos não viram afirmação de ausência no grupo.
O cadastro local não pretende resolver identidades NSS externas.

O painel oferece Preparar acesso e Verificar novamente. Selecionar uma porta
não a abre; operações continuam dependentes de acesso. A sessão desatualizada
recebe instrução para salvar o trabalho e sair/entrar na sessão do sistema,
sem repetir `usermod`, prometer que reiniciar só a IDE basta ou encerrar a
sessão automaticamente. Passos sem comando são instruções visuais. Passos
com comando usam o terminal existente, por clique explícito: esse caminho
envia Enter e executa a linha, não é apenas uma cópia. Nenhuma instalação,
elevação ou alteração na placa pelo agente.

**Aceite previsto:** regressões de credenciais ativas/cadastro, grupo primário,
proprietário sem bits, informações ausentes, remoção da porta e recusa do
monitor; harness de preparo/atualização e instrução sem execução; foto isolada
e gate integral estrito. A migração de Setup continua no §14.18, após esta
correção. Não declarar acesso físico resolvido sem nova medição real.

Nesta mesma auditoria, distinguir falha ao ler sysfs de lista vazia e manter
uma porta pedida que desapareceu como problema visível no diagnóstico. São
recuperações da mesma jornada serial, sem abrir dispositivo ou mudar o sistema.

**Prova parcial da implementação (2026-10-20):** 40 testes focados de serial
passaram; harness de recuperação passou em 400/300 px, e qmllint estrito
limpo. RPC real com o core debug confirmou `sessionRefreshRequired` em
`/dev/ttyUSB0`; listagem, diagnóstico e recusas de identificar, monitorar e
arquivos retornam a mesma causa, sem comando de recadastro. A conta local
consta no grupo, o processo não. Nenhuma porta aberta ou permissão alterada.
Dados em `/tmp/kinein-serial-recovery-proof/report.json`. Fotos nativas do
agente em Xvfb/software a 1440×900 mostram seleção, preparo e reverificação;
sem avisos QML proibidos, processo vivo. **Fechamento:** gate rápido estrito
TUDO VERDE, 1200 Rust, C++ limpo, 8 CTest, qmllint e 156 harnesses, sem
NÃO PROVADO. Log `/tmp/kinein-serial-gate-final.log`; registro no 40.7 §7.326.

**Bloqueio do gate e ajuste necessário (antes do lockfile):** a auditoria
atualizada em 2026-10-20 acusou RUSTSEC-2026-0382 (`rustls-webpki` 0.103.15)
e RUSTSEC-2026-0381 (`webpki-roots` 1.0.9), sobre certificados emSign
retirados de confiança. Fontes primárias conferidas:
[webpki #546](https://github.com/rustls/webpki/pull/546) e
[roots #133](https://github.com/rustls/webpki-roots/pull/133). Atualizar apenas
os dois patches no lockfile para 0.103.16 e 1.0.10, sem alterar features,
política de confiança própria ou criar exceção no cargo-deny; repetir o gate
integral. Essa correção de dependências é necessária para fechar a fatia.

**Refino de coerência da 0.4.1f:** atualizar/trocar o workspace durante uma
consulta de acesso invalida sua resposta pendente. Preservar um único pedido
em voo até receber e descartar a resposta antiga, impedindo que ela restaure
um diagnóstico depois de limpar o painel. Cobrir no harness. Rótulos positivos
nos canais auxiliares dizem Verificação concluída, pois `ModemManager.ok` não
significa que a porta serial tenha permissão de leitura/escrita.
O descarte mantém a consulta única enquanto o transporte existe; perda da
conexão com o core libera essa espera e limpa o diagnóstico, pois não haverá
resposta pendente. O roteador de eventos usa o `statusChanged` existente.

### 14.20 Próxima auditoria do backend: uso concorrente da porta (2026-10-20)

**Reproduzido durante a 0.4.1f, ainda não corrigido nesta fatia:** dois
`serial.identify` na mesma porta são aceitos e executam ao mesmo tempo. Prova
com core debug real, ferramenta falsa e arquivo temporário comum como porta,
sem abrir dispositivo: `/tmp/kinein-serial-concurrency-audit/report.json`.
Os dois inícios antecedem os dois términos. As travas `busy` dos controllers
são locais a cada operação e não substituem a arbitragem no core.

**Prioridade seguinte, 0.4.1h após a correlação da 0.4.1g (§14.21), antes de Setup/schema 3:** desenhar e implementar um
único dono por porta, considerando aliases canônicos/by-id, identificação,
arquivos, monitor e execução/gravação que usam o canal. Reutilizar o ciclo de
vida dos jobs e `TerminalManager::is_open`; não criar um segundo executor ou
um lock persistente sem recuperação. Erro deve dizer qual operação da IDE
ocupa a porta e como encerrá-la, sem matar automaticamente trabalho existente.
Ferramentas externas continuam responsáveis pela abertura/exclusividade do
sistema; não prometer detectar todo processo externo por uma medição de ACL.

**Aceite:** duas operações da mesma porta recusadas antes da ferramenta;
portas diferentes independentes; liberar após sucesso, falha, cancelamento,
saída/fechamento de terminal e troca de workspace; aliases não escapam da
proteção. Cobrir monitor × arquivos × identificação × run/flash, não só
desativar botões na tela. O mecanismo só fica concluído com a matriz coberta.
A correção de permissões da 0.4.1f não prova esta exclusividade.

**Pontos da inspeção para regressão na auditoria de orquestração:**
`CoreClient::sendRequest` ignora pedido quando o processo está parado e
`handleFinished` limpa métodos pendentes sem falha individual (exceto export).
Verificar que nenhum controlador fica ocupado sem resposta após queda do core.
`RuntimeController.handleTerminalOpened` consome o comando pendente na primeira
sessão que abre; conferir a correlação com a sessão solicitada, inclusive quando
um monitor/SSH abre ao mesmo tempo, antes de ampliar os gestos de preparação.
A correlação de terminal também foi reproduzida em teste isolado do
`RuntimeController` real, sem PTY nem execução: após solicitar o shell, uma
resposta de monitor não relacionado recebeu `echo AUDIT_ONLY` com Enter;
o shell solicitado ficou sem a linha. Evidência em
`/tmp/kinein-terminal-correlation-audit.log`. Corrigir a correlação de resposta
com o pedido antes de ampliar os gestos de preparação. O caso de processo
parado ainda depende de reprodução específica. A preparação por comando da
0.4.1f usa esse mecanismo existente; ele ainda não foi reformulado e não
permite declarar a orquestração de terminais/portas pronta.

### 14.21 0.4.1g — comando associado ao terminal solicitado (2026-10-20)

**Fatia anterior fechada:** 0.4.1f em `0f28695`, gate rápido estrito integral
verde. A correlação é o primeiro bloqueador do §14.20; a exclusividade serial
no core vem na 0.4.1h, antes de Setup/schema 3. Não marcar a matriz de portas
como corrigida ao resolver somente o envio de comandos ao terminal.

**Contrato antes do código, sem mudar o fio 0.181.0:** reutilizar o id da
requisição JSON-RPC já criado por `CoreClient`. `terminalOpen(cwd)` passa a
devolver esse id localmente; `terminalOpened(id, shell, requestId)` e
`terminalOpenFailed(requestId, message)` entregam o contexto da resposta à UI.
Os métodos e payloads do core permanecem os mesmos. O contrato da ponte vive
no arquitetura/36 §10; o protocolo terminal aponta para ele.

`PendingShellInput` continua dono único da linha, prazo e associação. O
roteador prende o id devolvido pelo cliente ao pedido que tem comando. Só
a resposta correspondente libera o texto com Enter. Monitor, container,
execução, outro terminal e respostas atrasadas não consomem a linha. Um segundo
comando enquanto o primeiro aguarda é recusado com mensagem, sem substituir
a intenção anterior. Erro, timeout, troca de workspace ou queda do core
cancelam a linha; ela não reaparece num terminal futuro. Pedido recusado
antes de sair do cliente também encerra a espera.

**Apresentação:** expor o erro desse fluxo no `TerminalPanel` existente por
`KvVerdict`, preservando o grid e recalculando sua geometria. Nenhum novo
executor, sistema de notificação ou componente paralelo. Confirmar coordenadas
com a mensagem visível e manter os gestos normais quando não há erro.

**Aceite antes do gate integral:** controller real com respostas intercaladas,
clique duplo, falhas de id certo/errado, timeout, queda e troca de projeto;
roteamento da ponte com os ids reais de resposta; teste visual em Xvfb/software
com processo/porta falsos e comando inofensivo, sem sudo, instalação ou placa.
Regenerar mapa de módulos antes de congelar a fonte para o gate. Compilar e
verificar debug/release, qmllint, harnesses e o gate completo da fatia C++;
nenhuma compilação de UI durante verificar-cpp.

O harness de superfície usa um stub de dados de `DebugFlags`, singleton C++
que não é carregado pelo runner QML de fontes. Ele desliga o overlay de
diagnóstico, sem substituir controller, renderer ou transporte testados.
O aceite nativo valida também o registro C++ real; nenhum aviso é suprimido.

**Resultados fechados em 2026-10-21:** os 157 harnesses QML passaram. O novo
`tst_shell_correlation` usa controller, roteadores e painel reais; cobre
respostas intercaladas, repetidas e tardias, segundo clique, recusa, timeout,
troca de projeto e desconexão. A desconexão posterior à recusa preserva a
causa recebida. A troca de projeto limpa erro anterior sem enviar a linha.

Aceite nativo do agente em Xvfb próprio/software, 1440×900, com core real e
proxy que entrega a resposta do monitor antes da resposta do shell. Porta é
arquivo regular temporário; monitor falso nunca abre dispositivo. O único
comando é `printf 'CORRELATION_OK\n'`, inofensivo. O binário anterior, guardado
antes da recompilação, enviou a linha ao monitor `t1`; a UI corrigida a enviou
apenas ao shell solicitado `t2` (pedido JSON-RPC 29). A mesma prova distingue
o defeito da correção, sem depender da posição visual das abas.
Relatórios, trace e fotos em `/tmp/kinein-shell-native-proof/before/` e
`after/`; roteiro `/tmp/kinein-shell-native-probe.py`.

Em outros dois ensaios nativos, diretório de trabalho removido foi recusado
pelo core real e a interrupção do transporte antes da resposta deixou a
mensagem “o core parou antes de abrir o terminal” visível, sem injetar a
linha. Fotos conferidas em `error/` e `disconnected/` na mesma pasta, sem
avisos QML; o grid permanece abaixo da mensagem. O primeiro gate completo estrito parou na complexidade cognitiva do despacho
C++ (27, limite 25). Antes de continuar: extrair o despacho de falhas
para uma função privada coesa, preservando os mesmos sinais e contextos;
não elevar limite nem suprimir diagnóstico. Revalidar o arquivo antes de
repetir o gate completo. A separação passou pelo clang-tidy isolado e pela
segunda rodada integral de análise C++, sem exceção. Nessa rodada também
passaram os 1.200 Rust, 8 CTest, qmllint, 157 harnesses e integrações; o gate
parou no smoke debug por `tst_clipboard_files` antigo, anterior à mudança de
`libsystemd.so.0`. Binário e três objetos guardados em backup, recompilados;
o teste passou novamente e a UI debug abriu em 385 ms, com passeio de 33
superfícies limpo. Recusa e desconexão nativas repetidas depois da separação,
com fotos finais conferidas. UI e core release compilados; nenhuma
biblioteca instalada foi alterada. A rodada final do gate completo estrito
ficou **TUDO VERDE**, sem NÃO PROVADO: 1.200 Rust, 8 CTest, qmllint e 157
harnesses, integrações externas, builds e abertura debug/release. Log
`/tmp/kinein-shell-full-gate-green.log`. Primeiro frame 350 ms em debug e
346 ms em release; cada passeio de 33 superfícies terminou sem avisos.
A prova de correlação foi repetida com UI e core release em `after-release/`
na pasta de evidências: apenas o shell `t2` recebeu a linha, pedido 29.
Estes números não afirmam ganho de desempenho.

**Inspeção para a próxima fatia:** o comando de Gravar é repassado como texto
para `run.start` ou salvo em run config editável; hoje não carrega identidade
de recurso serial. A exclusividade precisa acompanhar esse fluxo sem tentar
inferir porta por parsing de shell. Em ensaio com monitor falso, o processo
permaneceu vivo após `workspace.open` de outro projeto; com `SIGHUP` ignorado,
também após `terminal.close`. Relatório em
`/tmp/kinein-monitor-lifetime-u8rjn62e/report.json`, sem dispositivo aberto;
os processos de teste foram encerrados. Corrigir ciclo de vida e liberação
real junto à matriz do §14.20; não declarar exclusividade pronta só por
remover uma aba ou marcar um booleano na UI.

### 14.22 0.4.1h1 — encerrar o processo antes de liberar seu recurso (desenho, 2026-10-21)

Primeira parte da auditoria do §14.20, após fechar a 0.4.1g. O aceite nativo
anterior já foi preparado em `/tmp/kinein-shell-native-proof/lifetime-before/`:
clique de fechar, aba ainda visível e processo falso vivo; nenhuma entrada no
monitor. O core anterior foi guardado com SHA-256 em
`/tmp/kinein-terminal-lifetime-proof/before/`. **Concluída no principal:** desenvolvida no checkout
`/home/hugh/KineinVectis-worktrees/serial-lifecycle`, target próprio, e
incorporada após `bf65481`. Nove testes focados, 50 da família terminal e
o gate rápido estrito passaram (40.7 §7.328). O ensaio do §14.21 mostrou monitor vivo após trocar de
workspace e após fechar uma sessão cujo processo ignora `SIGHUP`. A matriz de
exclusividade permanece aberta até as partes seguintes; esta parte corrige
seu pré-requisito de ciclo de vida, sem declarar arbitragem de portas pronta.

**Referência oficial conferida:** a API de
[`Child`](https://docs.rs/portable-pty/0.9.0/portable_pty/trait.Child.html)
fornece `try_wait` sem bloquear e `wait` bloqueante. A fonte da versão 0.9.0
usada pelo projeto mostra que o `clone_killer` Unix só envia `SIGHUP`, ao
contrário do `kill` no handle original, que pode forçar o término. O spawn
Unix do PTY cria a sessão com `setsid`. A IDE deve usar esses mecanismos e a
camada `platform` existente; nenhum executor ou terminal paralelo.

**Desenho antes do código:** compartilhar o handle do filho entre fechamento
e waiter, com exclusão mútua curta. O waiter consulta o término sem segurar
um lock durante `wait` bloqueante. O fechamento verifica o estado sob o
mesmo lock, evitando sinalizar um PID já coletado, e encerra o grupo próprio
do PTY Unix; nos demais sistemas usa o handle do processo. Não remover a
sessão quando a solicitação de término falha. Somente o término observado
produz `event.terminal.closed`; recursos de longa duração não serão soltos
por um booleano de UI. Preparar reader/writer antes do spawn para que falha
nessa preparação não abandone um filho recém-criado.

Trocar para outro workspace encerra os terminais do anterior no dono comum
da transição (`activate_workspace`/`deactivate_workspace`), incluindo o
`run_terminal`. Reabrir a mesma raiz preserva suas sessões. Uma recusa real
de encerramento impede a troca, com erro; não afirmar que o projeto mudou
com um processo esquecido. O fechamento de todas as sessões tenta cada uma
e preserva as que falharam. Não cancelar indistintamente jobs globais de
instalação ou identificação ao trocar projeto.

**Extensão contratada antes do código (0.182.0):** falha ao consultar o
estado não equivale a término. O render da sessão ganha `notice?`, um
`JsonRpcError` tipado existente, com código e mensagem do core. A superfície
mostra sua mensagem no `KvVerdict` já usado pelo Terminal, preservando grid,
cursor e saída do programa. O aviso desaparece quando a consulta volta a
funcionar; nunca produz `terminal.closed` por si só. Não escrever diagnóstico
no buffer VT do programa nem usar saída stderr não estruturada.
A versão sobe antes de emitir esse campo; sem método/evento novo. O restante
é correção do ciclo de vida já contratado. **Aceite:** regressões devem reprovar o código anterior;
processo que ignora HUP, filho do wrapper, encerramento independente de outra
sessão, saída natural, stop/drop e troca de workspace. Usar só fixtures de
processos/portas; nenhum firmware ou arquivo de placa. Medir latência e
conferir que a nova observação não gera espera ocupada. Depois, desenhar
reserva por porta e metadados de execução/gravação; a matriz só fecha inteira.


**Execução isolada:** base atualizada para `bf65481`, mantendo a fonte e os
binários principais intactos durante o gate da 0.4.1g. Oito testes focados
passaram antes desta extensão de diagnóstico. O Clippy apontou a emissão
stderr provisória, corrigida pela extensão tipada acima, e dois ajustes de
estilo. O uso do handle original elimina o antigo workaround do killer
clonado Windows; a API original normaliza seu resultado. A implementação
Windows do grupo de PTY usa fallback no handle, mantendo a assinatura comum
(a sugestão de const dessa implementação não se aplica à função Unix).

A remoção do workaround antigo reduz cfg(unix) de 52 para 51; catraca
reduzida, sem adiar testes. O vocabulário recebe cinco palavras inglesas
normais e SRCH, constante POSIX/rustix para grupo que já terminou.

O registro da sessão também permanece após aceitar o pedido de fechamento,
até o término observado. Isso permite repetir o fechamento se a consulta
falhar; aceitar o pedido não apaga a identidade do processo. Os testes que
antes só verificavam um booleano após sinalizar agora aguardam o evento real.


**Aceites parciais, 2026-10-21:** três mutações reprovadas por asserções em
runtime: killer fraco, ausência de fechamento na troca de workspace e aviso
oculto pela UI. Fonte restaurada e testes repetidos verdes. Relatório
`/tmp/kinein-lifetime-mutations.json`. RPC real com ferramenta/porta falsas:
fechamento com HUP ignorado e troca de workspace terminaram o processo
(`/tmp/kinein-monitor-lifetime-wcxf33ba/report.json`). Na IDE nativa, a aba
sumiu somente após o processo sair: `lifetime-after/` na pasta de aceites do
§14.21, comparada ao `lifetime-before/`. Sem hardware, entrada ou comando no
monitor. Programa em primeiro plano num grupo diferente do shell também
terminou, incluindo após manter o registro da sessão: relatório em
`/tmp/kinein-foreground-proof-_o8937gt/report.json`.

Amostra controlada do protótipo, quatro monitores falsos, cinco segundos por
binário: CPU do core 0,2% antes e 0,6% depois de um núcleo; pedido de fechar
até saída observada 1,60/14,56 ms. Nenhuma espera ocupada observada nesta
amostra; há custo de observação periódica, sem alegar ganho de desempenho.
Dados em `/tmp/kinein-lifetime-profile-lez53n0l/report.json`.

A primeira rodada rápida do principal pegou uma regressão no despacho:
reter a identidade após aceitar fechar também deixou `terminal.input`
aceitar entrada. Correção: `closing` separado recusa novas ações imediatamente e mantém o
handle até o fim observado. A asserção existente foi preservada; os 50
testes da família de terminal passaram antes da rodada integral.


**Fechamento, 2026-10-21:** gate rápido estrito TUDO VERDE, 1.209 Rust,
158 harnesses QML, análise C++ limpa, 8 CTest, qmllint e nenhum NÃO PROVADO.
Log `/tmp/kinein-lifetime-gate-final.log`. Core debug/release reconstruídos;
aceite nativo repetido com UI e core release em
`/tmp/kinein-shell-native-proof/lifetime-final-release/`: processo que ignora
HUP terminou, aba sumiu, nenhuma entrada no monitor e log QML limpo. RPC
release confirmou fechamento e troca de projeto em
`/tmp/kinein-monitor-lifetime-ka1oqiq2/report.json`. Sem hardware ou gravação.

**Regressão adicional reproduzida para a próxima parte:** o router/controller
real de Embarcados mantém sete estados ocupados após queda do core
(identificação, arquivos, prévia, sondas, portas, projeto e tamanho), sem
mensagem. Fixture e saída em `/tmp/kinein-audit-embedded-disconnect.qml` e
`/tmp/kinein-embedded-disconnect-audit.log`. Corrigir esperas pendentes e
resultados acionáveis sem repetir ações após recuperação. A inspeção também
confirmou `lastRunMessage` sem consumidor visual: expor as recusas de
run/script/stop no componente existente antes de ampliar erros de ocupação.
A matriz de reserva serial do §14.20 continua aberta.

**Smokes finais:** debug/release chegaram ao primeiro frame em 368/348 ms
(nesta medição), com passeio de 33 superfícies por preset, sem aviso QML.
Logs `/tmp/kinein-lifetime-smoke-debug.log` e
`/tmp/kinein-lifetime-smoke-release.log`.

### 14.23 0.4.1h2 — falhas de transporte e execução visíveis (desenho, 2026-10-21)

**Base:** `54c061f`, ciclo de vida fechado. Antes da reserva por porta,
corrigir a regressão reproduzida no §14.22: o core desaparece, mas o painel
continua ocupado e permite usar uma prévia de gravação do processo anterior.
Também há recusas de run/script/stop guardadas em lastRunMessage sem consumidor
visual. Não declarar arbitragem concluída; o desenho inicial de coordenação
entre janelas está em `/tmp/kinein-serial-ownership-design.md`, para a parte
seguinte com contrato próprio.

**Desenho anterior ao código:** reutilizar os controllers, roteadores e
KvVerdict. A perda da conexão termina as esperas de Embarcados, invalida
resultados acionáveis e confirmação de escrita, e informa a interrupção.
Preservar intenção de porta/motor/firmware e a aba visível; nenhum comando é
reexecutado ao reconectar. O pedido feito enquanto desconectado recebe recusa
local de transporte antes de aguardar o core. Centralizar o encaminhamento
de erros no dono existente, sem repetir a política em cada router. A UI não
inventa diagnóstico de hardware nem determina se o processo físico terminou.

Execução tem erro separado de conclusão positiva e da saída VT do programa.
As recusas de iniciar/parar/fechar e a perda de transporte aparecem na mesma
faixa existente do Terminal, preservando mensagens específicas já recebidas.
Nova tentativa/contexto limpa o erro; sucesso de programa não vira erro.
Uma recusa de fechar não limpa as identidades de outras abas em encerramento.
Sem alteração do fio 0.182.0 ou de ui/src nesta parte.

**Aceites:** fixture do §14.22 passa sem alterar a asserção; teste dos
controllers/routers/painel reais cobre pedidos sem conexão, queda antes/depois
do aceite de job, confirmação descartada, prévia inválida, recuperação sem
repetição, preservação de escolhas e erro legível sem sobrepor o grid. Foto
nativa antes/depois com core real e falha de execução; injeção controlada de
queda durante identificação falsa, sem dispositivo. Gate rápido estrito e
smokes antes de fechar. Depois, reserva da porta cobrindo toda a matriz.

**Refinamento antes de fechar:** o aceite nativo mostrou que, após perder a
conexão, o cabeçalho dizia “nenhuma placa” e a lista sugeria conectar USB,
sem uma consulta válida. Distinguir snapshot não consultado de lista recebida
vazia, sem interpretar texto do diagnóstico ou inventar ausência física.
Contrato local: portsMeasured só sobe ao receber serial.list e cai ao trocar
workspace/perder conexão/falhar consulta. Cabeçalho vazio diz “nenhum dispositivo
listado”, fato sobre a lista, nunca diagnóstico da placa. Interrompida a rodada
do gate antes de recompilar UI; repetir com fonte congelada após essa correção.

**Fechamento da 0.4.1h2 (2026-10-21):** gate rápido estrito TUDO VERDE,
1.209 Rust, análise C++ limpa, 8 CTest, qmllint e 160 harnesses, sem NÃO PROVADO.
Log `/tmp/kinein-h2-gate-final.log`. Smokes debug/release: 355/341 ms nesta
medição; passeio de 33 superfícies por preset sem aviso, logs
`/tmp/kinein-h2-smoke-{debug,release}.log`. Três mutações reprovadas em runtime:
queda sem invalidar operações, erro de execução oculto e perda da identidade
de outra aba em fechamento. Fontes restauradas antes da rodada final.

Fotos/RPC nativos, UI/core release, em `/tmp/kinein-shell-native-proof/`:
`run-failure-before/` versus `run-failure-final/` mostra a recusa real por
falta de comando padrão agora visível; `identify-loss-before/` versus
`identify-loss-final-focused/` mostra o aceite real do job falso, perda
controlada do transporte e espera liberada com causa visível. O último usa
atalho com foco no Xvfb, sem macro de startup repetida na recuperação. Todos
os dispositivos são arquivos regulares e ferramentas falsas não os abrem.
Ensaios gráficos finais sequenciais e log QML limpo. Uma tentativa paralela
compartilhou display por corrida no helper e foi descartada; não é prova do
produto. O runner de cenas por macro também reaplica a macro ao recuperar
workspace; por isso o aceite final usa abertura pelo atalho.

**Observação da 0.4.1h2 resolvida na parte seguinte:** a dica de Identificar
permanecia após destruir o card, por mais de três segundos. A regressão e a
correção do TooltipController existente foram incorporadas à 0.4.1h3 (§14.25),
com foto nativa antes/depois. A reserva isolada sobre 54c061f foi rebaseada
sobre 75330ef, corrigida após a prova inicial de crash reprovada e incorporada
no principal com fio 0.183.0; fechamento e limites no §14.24–14.25.

### 14.24 0.4.1h3 — reserva serial de jobs e monitores (desenho, 2026-10-21)

Implementação sobre 75330ef, inicialmente isolada; incorporada e concluída
no principal após os aceites e o gate registrados abaixo. Prioridade e matriz no §14.20.
Esta parte cobre identificar, arquivos e monitor; run/flash e ocupação visual
continuam obrigatórios nas partes seguintes, sem declarar Embarcados pronto.

Um registro compartilhado do core concede uma reserva RAII por identidade de
arquivo, resolvida por metadata sem abrir tty: rdev para dispositivo de
caractere Unix e dev/inode para fixtures/aliases/hardlinks. Cada job captura
a reserva até seu trabalho terminar, incluindo fallback e cancelamento. O
waiter do PTY segura um guard Send até observar a saída; falha de spawn solta,
falha de observação/close preserva. O terminal não conhece política serial.

No core com persistência, complementar o mesmo registro com flock em arquivos
privados de coordenação por usuário, permitindo janelas/workspaces distintos.
Core de teste permanece em memória por padrão. Arquivos não são removidos:
o kernel libera ao fechar/crash; registro em disco sem lock não prova uso.
Aquisição usa lock compartilhado global e exclusivo por identidade; a escolha
automática terá de obter global exclusivo na próxima parte. Falha de storage
ou de coordenação recusa antes da ferramenta, sem fallback desprotegido.
Metadata de dono é publicada antes do processo e só lida após confirmar lock
ocupado. O escopo não inclui programas externos/versões antigas da IDE.

Contrato 0.183.0, escrito antes do produtor: SERIAL_PORT_BUSY com details.device
e details.owner {operation,title,workspace?,corePid}. Mensagem específica diz
que operação ocupa o canal, em que projeto/janela, e orienta encerrar no
Terminal ou cancelar no Jobs, sem encerrar trabalho existente automaticamente.
A referência específica de job/terminal e as ações diretas da ocupação visual
precisam ser adicionadas com seus consumidores; não inventar por parsing shell.

API oficial rustix::fs::flock e FlockOperation conferida em 2026-10-21:
https://docs.rs/rustix/latest/rustix/fs/fn.flock.html e
https://docs.rs/rustix/latest/rustix/fs/enum.FlockOperation.html.
Reutilizar rustix existente, sem nova dependência nem elevar MSRV 1.88;
std::File::try_lock é estável desde 1.89, conferido na documentação oficial.
Fronteiras Unix ficam na platform existente, sem cfg novo/adiamento de teste.

Aceites: duas operações da mesma porta e aliases recusadas antes de iniciar
ferramenta; duas portas independentes; liberar sucesso/falha/cancelamento,
saída/fechamento do terminal e troca de workspace. Dois processos, crash e
storage indisponível. Fixture regular e ferramentas falsas; nenhum dispositivo.
A matriz inteira só fecha com execução/gravação e recuperação visual integradas.

A metadata de dono pode estar indisponível na publicação inicial; owner null
nessa recusa não inventa identidade, mas preserva o lock ocupado. O módulo
usará a mesma exceção local de visibilidade de platform para tipos crate-only
(redundant_pub_crate versus unreachable_pub), sem ampliar alcance ou limites.

**Revisão obrigatória antes de integrar (2026-10-21):** teste real do protótipo
matou apenas o core e encontrou ferramenta externa ainda viva; outro core
aceitou a mesma porta. Relatório `/tmp/kinein-serial-owner-crash-v9erukuf/report.json`.
Logo, lock mantido só pelo core não protege a vida real do recurso após crash.
NÃO integrar nem declarar pronto esse protótipo sem corrigir e repetir a prova.

Desenho revisto: dois arquivos por porta, reserva de lançamento do core e
reserva de execução do processo. Ao adquirir, core toma lançamento exclusivo,
sonda execução exclusiva e publica token único/dono antes de soltar a sonda.
O executável core tem um modo interno de exec, sem outro executor: abre a
reserva de execução, valida o token de lançamento e a vida do core originador,
publica processId e substitui-se pela ferramenta via exec. Somente nesse
processo filho dedicado remove CLOEXEC dos descritores de execução/global;
nunca altera herança de FD no core multithread. O kernel mantém a reserva
na ferramenta após crash do core. Um lançamento antigo que perder a corrida
para novo dono vê token diferente e recusa antes de executar. Identidade do
processo continua a do PTY/job, pois exec não muda PID. Sem dependência nova,
sem shell adicional ou reimplementação de ferramenta. Reutiliza o runner e
o terminal existentes; a saída permanece a da ferramenta.

Metadados de disco separam Owner de token privado. details.owner ganha
processId? quando o helper publicou a identidade. Mensagem descreve outra
instância, sem supor uma janela ainda viva. Ações de recuperação visual e
encerramento de órfão precisam usar identidade verificada; pertencem à parte
seguinte com consumidor próprio. A proteção não pode aceitar nova operação
quando o processo antigo ainda mantém a reserva. A mesma prova de crash
deve recusar o segundo início enquanto a ferramenta falsa estiver viva.

A suíte paralela também mostrou um fork herdando brevemente descritor CLOEXEC
antes do exec, com metadata já limpa. Aquisição pode aguardar até 50 ms,
dormindo 1 ms por tentativa, somente quando o lock está ocupado sem metadata
legível, antes de devolver ocupação desconhecida. Teste determinístico deve
cobrir essa janela; não desabilitar paralelismo nem enfraquecer asserções.

O modo de exec confirma o PPID nativo igual a corePid antes de trocar de
programa; não trata apenas existência de um número PID como prova de pai.
Assim, reutilização do PID não autoriza um lançamento abandonado.

O helper executa a imagem atual via /proc/self/exe quando disponível; não
segue cegamente o caminho do binário que pode ter sido substituído por
rebuild/atualização. Assim usa o mesmo contrato interno mesmo com o arquivo
antigo removido. Fora desse caso, plataforma resolve o executável atual.
Clippy sugeriu remover collect no teste de concorrência, o que esperaria
no primeiro join antes de criar as outras threads da barreira; usar laço
explícito de criação e só depois joins, sem exceção de lint.

A reserva deve ser solta após observar o processo terminar e antes de publicar
o desfecho serial: serial.files bem-sucedido dispara refresh no consumidor.
Manter o guard até o retorno externo do job deixaria uma janela em que a UI
recebe conclusão e o próximo pedido ainda é recusado. Runner passa a possuir
o guard e o libera explicitamente antes do evento final, após todos os
fallbacks/parsers, preservando o executor existente. Cobrir pedidos imediatos
a partir do evento, sem aguardar job.finished.

A conclusão do guard não apaga o último dono em disco: marca o Record privado
como completed. Uma ferramenta/descendente pode ainda manter o descritor de
execução; perder essa identidade prejudicaria a recuperação. O kernel continua
a decidir ocupação. Lock com Record completed recebe a mesma espera curta
para descritor herdado por fork; dados obsoletos sem lock são ignorados. O
helper recusa também Record completed, mesmo que o token não tenha sido
substituído. A mensagem de outra instância não manda cancelar no Jobs do core
novo; parada/ocupação visual e execução/gravação continuam na próxima parte.

Leitura da catraca antes da integração: lib.rs chegou a 503/500. A conversão
do marcador Deferred pertence a outcome.rs, que já possui RequestOutcome;
mover a conversão para From<JsonRpcResponse> nesse dono, mantendo o despacho no core,
sem novo módulo nem alteração de comportamento. Nomes tocados do runner
serial migram para inglês e a linha de base só encolhe.

### 14.25 0.4.1h3 — feedback visual acompanha a operação (desenho, 2026-10-21)

Bug já fotografado no §14.23: a dica de Identificar continua por cima do
painel após a destruição do card, mesmo depois de três segundos. Corrigir
TooltipController existente, mantendo os mesmos consumidores/host/estilo.
A referência visual deve ser tipada e perder a exibição quando o dono deixa
de existir ou de estar visível, inclusive por ancestral oculto. Um evento
atrasado do dono anterior não pode apagar a dica do controle novo. Atualização
de texto e dica em botão desabilitado continuam válidas. Sem polling, singleton
novo ou limpeza duplicada em cada componente.

Estender tst_tooltip_update com controles reais dinâmicos, destruição/ocultação
e troca de dono; provar vermelho antes e verde após. Foto nativa antes/depois
na mesma jornada de identificação interrompida. Sem alteração de IPC/ui/src.
A proteção serial está em validação no principal. Integrar este feedback
à mesma fatia antes dos gates QML, para fechar recusa, apresentação e vida
do card juntas; sem repetir análise C++ sobre fontes inalteradas.

A foto final da reserva (§14.24) também confirma recusa correta mas texto
pequeno e inteiro vermelho na identidade. Reusar KvVerdict nas falhas de
Identidade/Arquivos/Prévia de gravação, com título contextual, corpo legível,
transição curta e o texto integral do backend. Não inferir causa no QML.
O componente comum deve tratar mensagem/título como texto simples: retorno
contendo tags literais precisa continuar legível, sem interpretação de HTML.
Provar com superfícies reais, mensagens longas, larguras estreitas e recuperação
que remove a faixa. A mudança é feedback da operação, sem redesenhar a navegação
ou substituir os controllers. Preservar identidade parcial e saída técnica.

O ensaio extra no Qt 6.4 falhou antes de carregar o teste: o espelho QML
passou a usar rg, ausente na imagem do runner. A montagem do módulo deve
usar find (já disponível), sem instalar ferramentas no host/container nem
pular esse aceite. Registrar essa falha de infraestrutura e repetir a prova.

**Aceites da 0.4.1h3 no principal (2026-10-21):** 18 testes novos de reserva,
incluindo aliases/hardlinks, threads, dois registries/processos, storage,
crash, token obsoleto, descritor mantido pela ferramenta, PTY/jobs reais e
pedido imediato após evento de resultado. Teste de observação do terminal
agora mantém uma reserva real durante falha/recuperação. Duas mutações de
liberação precoce/exec com CLOEXEC foram recusadas por asserções de runtime
(`/tmp/kinein-h3-mutations.json`). 1.227 testes Rust da fonte final passaram
(`/tmp/kinein-h3-final-source-tests.log`), após a conversão de outcome.

Core release real, estado isolado, ferramenta falsa e arquivo regular:
- crash: `/tmp/kinein-serial-owner-crash-1n4b4rqs/report.json` confirma
  ferramenta viva e segunda operação recusada com SERIAL_PORT_BUSY/processId;
- imagem removida: `/tmp/kinein-self-exec-90tlphq5/report.json` confirma
  identificação após remover apenas a cópia privada do binário em execução;
- monitor persistido/PTY real: `/tmp/kinein-serial-monitor-persisted-ss4awi30/report.json`
  confirma exec do tio falso que ignora HUP, recusa concorrente, saída real
  no fechamento e nova abertura depois da liberação.

Foto da recusa antes do polimento: `/tmp/kinein-shell-native-proof/serial-busy-main-final/shots/02-busy-visible.png`.
Real core release + UI nativa; segunda instância recebe recusa, ferramenta
chamada uma única vez, ui.log vazio. Não confundir esta fixture com placa
física. Smokes prévios ao polimento: debug/release 391/377 ms nesta amostra,
33 superfícies por preset sem aviso. Repetir após recompilar as mudanças QML.
Nenhum dispositivo aberto e nenhuma gravação/instalação no host.

O §14.25 usa os mesmos controllers e KvVerdict. A suíte de feedback compara
larguras 320/736, causa literal contendo tags, identidade parcial e limpeza.
Retirar PlainText produz links e reprova asserções; retirar a limpeza da dica
reproduz o texto preso. Provas no Qt local e 6.4, sem desabilitar falhas.
A primeira prova Qt 6.4 falhou por rg ausente no espelho; find corrigiu o
preparo sem instalar dependência. Traversal de teste usa índice/guarda para
listas de filhos do Qt 6.4. A rodada com mutações simultâneas não é aceite;
provas finais estáveis: `/tmp/kinein-h3-feedback-main.log`,
`/tmp/kinein-h3-tooltip-main.log`, `/tmp/kinein-h3-feedback-qt64-main.log` e
`/tmp/kinein-tooltip-qt64-after.log`. Gate geral ainda em andamento.

A mensagem pode conter um identificador USB longo sem espaços; a faixa comum
deve quebrar esse trecho quando necessário, mantendo a causa integral dentro
da largura. Aceite usa um identificador de 160 caracteres e mede contentWidth.

**Fechamento da 0.4.1h3 (2026-10-21):** TUDO VERDE no gate rápido estrito
(`/tmp/kinein-h3-gate-final.log`): 1.227 Rust, Clippy, análise C++ limpa,
8 CTest, qmllint e 161 harnesses QML; nenhum NÃO PROVADO. Conversão final
retestada na suíte Rust completa. O protótipo reprovado acima foi substituído
pela reserva de execução, com a prova real de crash aprovada antes de integrar.
Os fontes C++ não mudaram; compilações QML nativas ocorreram após terminar
verificar-cpp.sh. Todas as fontes finais passaram nas checagens correspondentes.

UI/core debug e release reconstruídos. Smokes finais 540/462 ms nesta amostra,
33 superfícies por preset sem aviso (`/tmp/kinein-h3-smoke-debug-final.log` e
`/tmp/kinein-h3-smoke-release-final.log`). Aceites nativos sequenciais com core
release, Xvfb próprio/software e ui.log vazio: recusa legível com uma única
chamada da ferramenta em `/tmp/kinein-shell-native-proof/serial-busy-polished-final/`;
queda durante identificação sem dica presa em
`/tmp/kinein-shell-native-proof/identify-loss-tooltip-cleared-final/`.
Fotos antes/depois vistas. A primeira tentativa desse passeio coincidiu com
o link do binário e não iniciou a UI; descartada, servidor X próprio encerrado
e prova repetida somente após o build concluir. Nenhum processo da IDE do
usuário foi encerrado, nenhuma entrada global ou dispositivo foi usado.

**Próxima parte obrigatória, 0.4.1h4:** o caminho nativo de run.start/run.script
(MicroPython com porta explícita e automática) e gravação precisam usar a mesma
reserva. A seleção automática exige coordenação global; host CPython/comandos
manuais não devem reservar uma porta só por estar selecionada na UI. Gravação
nativa precisa transportar intenção estruturada no run/configuração salva,
revalidando contexto/artefato/porta no backend; não inferir por parsing shell.
Depois, snapshot e ações de ocupação ligadas a job/terminal ou processo com
identidade verificada, incluindo ferramenta órfã após queda do core. Não matar
PID apenas porque consta em metadata antiga. Reusar executores existentes.
Aceites incluem portas distintas, concorrência, cancelamento, troca de projeto,
reconexão, configurações antigas e nova instalação. Esta fatia não fecha a
matriz inteira nem comprova flash físico. Setup no dock/schema 3/C/Python e
o plano até 0.4.9 permanecem na fila; nada foi transferido à etapa especial.

### 14.26 0.4.1h4a — execução MicroPython participa da reserva (concluída, 2026-10-21)

Base 950c8f8. A auditoria do caminho run confirma que start/script abrem o
PTY sem claim: monitor/identify/files já usam a reserva, execução MicroPython
não. Provar a falha por dispatch com mpremote falso e arquivo regular antes
de alterar o produtor. Esta parte conclui a execução MicroPython; gravação
estruturada/configuração salva e ações de ocupação são as partes seguintes
da h4, sem marcar a matriz completa como pronta.

Contrato 0.184.0 antes do core: run.start padrão e run.script .py, quando o
lançador resolvido for Mpremote, podem recusar SERIAL_PORT_BUSY. device
explícito reusa validação de acesso e reserva por identidade. Ausente mantém
a seleção do mpremote; por não conhecer a porta a priori, reserva exclusiva
global durante a operação. details ganha scope device|automatic, device null
no caso automático; owner.automatic distingue reserva ampla de uso conhecido.
Fora de MicroPython, comando explícito e configuração manual salva não
reservam nada só porque a UI tem uma porta selecionada. O campo não passa
por parsing shell nem muda a precedência existente.

A reserva automática precisa de dois passos, sem pai/filho esperando o mesmo
lock exclusivo: launch.lock compartilhado para portas explícitas, exclusivo
para automático. Sob esse guard, sondar global.lock exclusivo; depois soltar
a sonda para o helper tomar exclusivo e manter na ferramenta. A reserva de
lançamento fica no core até o PTY sair. Portas explícitas preservam global
compartilhado e o lock específico; duas portas distintas seguem independentes.
Isso também coordena cores anteriores da h3, que já seguram global compartilhado.
Dono desconhecido continua null; dado antigo de seleção automática não prova
o dono de um conjunto de locks compartilhados. Token/PPID/imagem atual e
proteção após crash continuam obrigatórios. Sem novo executor/dependência.

A abertura comum do run recebe uma invocação tipada com programa, argv,
texto mostrado e identidade; o guard opcional segue para o waiter existente.
A decisão de reservar vem do PythonLauncher resolvido, não do nome do projeto
repetido nem da seleção visual. Preservar comando/identidade mostrados e a
execução manual por shell. Não lançar a ferramenta ao recusar, falhar preparo
ou não haver ponto de entrada. Core decide e a faixa existente do Terminal
apresenta a causa, sem novo estado duplicado no QML.

Aceites: monitor bloqueia run.start/run.script; execução bloqueia jobs/monitor;
portas distintas independentes; automático exclui qualquer porta e vice-versa;
sucesso, saída não zero, spawn falho, stop/close/troca de workspace e crash;
ausência/permissão de porta; comandos manuais/config salva/host Python sem
reserva; argv com espaços/aspas literal. Reusar fixture de jobs/PTYs e estender
sem fabricar /dev/tty* nos testes. Provar kernel/duas instâncias com ferramenta
falsa, sem hardware, e erro visível no Terminal nativo. Mutação da reserva e
da exclusividade global deve reprovar asserções. Depois gate rápido estrito,
documentação e commit próprio antes de continuar a gravação.

O handoff de origem está no 40.7 §7.331; a implementação/prova final abaixo
substitui aquele ponto de retomada.

No desenvolvimento da h4a, a resolução mostrou outro ramo a conferir:
EntryPoint::InstalledScript retorna executável do host mesmo quando o launcher
é Mpremote. Provar com a fixture de project.scripts já existente e recusar
esse padrão no MicroPython, como já ocorre com pacote -m. Nenhum fallback
para executar Python do host numa intenção de placa. A fixture comum fixa
tio no papel de monitor para manter o conflito estável; sem fixar, projeto
MicroPython prefere o próprio mpremote como monitor. O primeiro teste do
handoff usava esse caminho curto; a regressão definitiva deve manter monitor
realmente vivo, arquivo de liberação ausente e ferramenta de run não chamada.

Na conferência nativa da h4a apareceu contradição adicional: o cabeçalho usa
project.model e reconhece MicroPython, enquanto ProjectHealth usa apenas o
build system e afirma que build/run estão indisponíveis. Corrigir o controller
existente com o reconhecimento já produzido pelo EmbeddedController, sem
redetectar arquivos no QML. Grupos de ferramentas vazios não podem encerrar
antes dos avisos Python. Projeto sem build detectado também não implica que
comandos manuais ou scripts estejam proibidos. Harness deve cobrir contexto
recebido/retirado, Python isolado e prioridade dos avisos; foto nativa depois.

Implementação integrada no principal sobre e44def1. Regressões focadas:
32 testes de reserva e 26 de run passaram, além do teste de project.scripts,
Clippy do workspace e harness de saúde novo/existente. O novo harness também
passou no Qt 6.4. Três mutações Rust (sem reserva, lock automático compartilhado,
conflito só por chave) e duas de saúde (ignorar reconhecimento, retorno precoce
sem ferramentas) reprovaram asserções; fontes restauradas e suíte focada verde.
Core real em duas instâncias manteve a proteção após crash e liberou após
saída da ferramenta falsa, tanto run.script automático como run.start explícito.
Fotos nativas da recusa comparadas antes/depois, sem segunda chamada do mpremote.
Fotos finais com o aviso de saúde corrigido também passaram, em debug e release.

O gate integral da h4a encontrou outra fixture antiga:
verificar_micropython_porta.py ainda usava /dev/ttyACM7 e /dev/ttyUSB* inexistentes.
Trocar por arquivos regulares criados na fixture (a ferramenta falsa só ecoa
argv), incluir recusa da porta ausente e isolar XDG_STATE_HOME. Garantir limpeza
do core também na falha. Não criar nós em /dev nem relaxar a validação real.
O primeiro gate passou Rust/Clippy/C++/CTest/qmllint, mas parou nessa fixture;
não é fechamento verde. Repetir o gate após corrigir o teste de integração.


**Fechamento h4a, 2026-10-21:** gate rápido estrito TUDO VERDE, sem NÃO
PROVADO: 1.241 Rust, Clippy, análise C++ limpa, 8 CTest, qmllint e 162 QML.
Log final `/tmp/kinein-h4a-gate-final-v2.log`; o primeiro log permanece como
prova da fixture que falhou, não como gate verde. A integração Python com
porta regular e debugpy real passou após a correção. Sem exceção nova de
arquitetura/Windows; vocabulário técnico inglês, legado só encolheu.

Provas nativas finais: `/tmp/kinein-shell-native-proof/h4a-final-debug-automatic/`
e `/tmp/kinein-shell-native-proof/h4a-final-release-explicit/`, fotos conferidas,
ui.log vazio, zero chamadas adicionais do mpremote na recusa. A comparação
anterior válida está em h4a-run-busy-before-valid/h4a-run-busy-after-valid.
Smokes debug/release: 391/362 ms nesta amostra, 33 superfícies por preset,
sem aviso. O primeiro lançamento de QA durante o link release foi descartado
(permissão temporária do artefato em construção); repetição esperou o build.

Core release real também passou no crash de seleção automática e explícita:
`/tmp/kinein-h4a-crash-audit-release-observed.log`. O primeiro helper assumia
que cmdline vazio após SIGKILL já significava todos os FDs fechados; recebeu
recusa transitória. O helper final espera liberação observada da reserva,
sem relaxar o produtor; após a saída, a segunda operação inicia e encerra.
Antes dela, nenhuma segunda ferramenta foi iniciada. Sem hardware/firmware.

Próximo: h4b1 preserva configurações, depois h4b2 integra a intenção nativa
de gravação e ações de ocupação/recuperação. A matriz serial inteira continua
aberta; Setup no dock/schema 3/C/Python e a sequência até a 0.4.9 seguem no §11.

### 14.27 0.4.1h4b1 — preservar configurações antes da gravação nativa (concluída, 2026-10-21)

Pré-requisito encontrado ao conferir a h4b: runconfig::read_file transforma
JSON inválido, erro de leitura ou schema desconhecido em lista vazia; save
pode sobrescrever o conteúdo anterior, e run.start pode cair no automático
quando uma configuração ativa não pôde ser lida. Provar por dispatch e bytes
antes/depois. Nesta parte, ausência real continua vazia; arquivo ilegível,
corrompido, não regular, schema desconhecido, ids duplicados/ativa inexistente
ou entradas sem nome/comando devem ser recusados sem alterar o arquivo.
Nada renomeia, apaga ou repara dados do usuário automaticamente. Reusar a
gravação atômica existente e coordenar mutações entre instâncias; conflito
é recusa explícita, não perda da atualização de outro processo. Preservar
schema 1 nesta parte: intenção tipada/schema 2 entram na h4b2 seguinte.

Contrato 0.185.0 antes do produtor: falha de armazenamento de runConfig.*
e de seleção ativa em run.start é INVALID_REQUEST com caminho e orientação;
parâmetro inválido continua INVALID_PARAMS. Comando explícito pode executar
sem depender de ler configurações, mantendo a recuperação pelo Terminal.
ConfigAction não pode sobrescrever o arquivo inválido por um caminho lateral.
A interface representa a recusa no editor/menu de configuração; não fecha
ou perde nome/comando enquanto o salvamento falha. Sucesso vem da resposta
do core, não do clique; desconexão solta espera sem repetir a mutação.
Harness, dispatch, escrita interrompida/concorrente, mutação e foto nativa.
Sem gravação física, nova ferramenta, executor ou mudança de schema nesta parte.

A coordenação precisa preservar Windows/MSRV 1.88: Unix reusa o arquivo e
flock existentes; Windows abre o arquivo de coordenação com share_mode(0),
sem unsafe/dependência nova. Não usar File::lock, estabilizado após o MSRV.
Nenhum lock é removido para evitar separar inodes de duas instâncias.
A confirmação local do editor usa requestId, como terminal.open, sem tratar
snapshot da lista como confirmação de salvar. A migração das janelas de
trabalho ao dock permanece no escopo já alocado; esta correção altera o
componente existente e não cria outra janela/editor.

O primeiro teste paralelo do guard detectou a janela fork/CLOEXEC já conhecida
na reserva serial: um filho de teste ainda não executou a nova imagem e
segura brevemente descritores herdados. Não serializar a suíte nem remover
o lock: tentar por até 50 ms antes de recusar, preservando a coordenação
real. Aplicar espera limitada também ao compartilhamento exclusivo Windows.

O novo guard faz o Windows ultrapassar a catraca de 500 linhas de produção.
Separar sua responsabilidade de arquivos (abertura sem reparse, enumeração,
publicação/atributos e guard) em platform/windows/files.rs, preservando a
fachada existente e os testes. Processos e resolução de ambiente/caminhos
continuam no windows.rs; sem wrapper por função nem baseline maior.

A primeira tentativa de foto do salvamento não exercitou o envio: Tab não
levou a digitação ao comando e o campo ficou vazio. Descartar como prova de
armazenamento, repetir com clique explícito no campo e conferir o comando
persistido. Conferir a navegação de teclado do editor com foco real antes de
atribuir a falha ao componente ou ampliar a mudança do campo compartilhado.

Também recusar campos não reconhecidos dentro de cada entrada do schema 1:
ignorar um campo de intenção e regravar apenas nome/comando perderia esse
dado. Reusar RunConfigInfo com desserialização estrita e provar preservação;
sem duplicar um segundo tipo de configuração só para o arquivo.

Conferir na consolidação Python da 0.4.x: PythonController pede python.status
apenas quando workspaceBuildSystems contém python; um main.py MicroPython
reconhecido só pelo project.model pode ficar sem essa consulta e sem o aviso
de stubs. Reusar o contexto primário já produzido pelo backend para a análise,
sem redetectar arquivos nem tratar exemplos MicroPython aninhados como runtime
principal. A h4a corrige a prioridade/contradição no ProjectHealth, não comprova
essa consulta completa do ambiente. Pendência funcional dentro da 0.4.x.

O erro de configuração continua pertencendo ao RunConfigController. A faixa
já existente do Terminal o recebe por binding, sem copiar a falha para o estado
da execução; snapshot válido limpa só esse erro, preservando falhas independentes
de run/terminal. Confirmar também a renderização da faixa e os campos mantidos
após falha. A lista recebida continua sem confirmar uma mutação pendente.

Aplicar o limite de leitura já existente do editor (fsops::MAX_READ_BYTES,
1 MiB em 2026-10-21) ao arquivo de configurações: leitura limitada também se
o arquivo crescer durante a operação. Recusar publicação que produziria um
arquivo acima desse limite, preservando o anterior. Assim o próprio save não
cria um estado que a próxima leitura recusaria. Provar com arquivo esparso e
atualização grande; sem limite paralelo definido só para este domínio.

O harness com renderização passou no Qt local, mas a busca do componente
usando for..of sobre children não rodou no Qt 6.4. Corrigir o helper do teste
para percorrer a lista por índice, sem mudar o componente do produto nem
aceitar o timeout como verde. Repetir os dois runtimes após a correção.


Integração h4b1 no principal sobre 7cab76f (2026-10-21): contratos 0.185.0
aplicados antes do produtor. Testes focados após restaurar mutações passaram:
12 Rust de runconfig e harness de confirmação nos Qt local/6.4. Acrescentar
o dispatch alternativo configAction.preview/apply com arquivo corrompido
entre prévia e aplicação: o caminho delegado também deve recusar sem perder
bytes. Builds e aceite nativo em andamento; gate completo ainda pendente.
Decisão de entrada curta de terminal registrada no dono de desktop §3;
implementação separada na consolidação 0.4.6, com prova no pacote 0.4.9.


Rechecagem nativa h4b1 confirmou a falha de teclado, sem atribuí-la ao helper:
com foco explícito no nome, Tab levou o foco ao botão de minimizar do header;
a digitação seguinte não chegou ao comando. Foto
`/tmp/kinein-runconfig-native-proof/after-debug/shots/03-keyboard-navigation.png`.
Corrigir a sequência no editor existente (nome/comando/cancelar/salvar,
Tab/Shift+Tab), usando keyPressed/KeyNavigation já disponíveis; sem alterar
o campo compartilhado nem criar outro editor. Provar o gesto nativo e a
ordem nos dois Qt, mantendo a confirmação correlacionada do salvamento.


**Fechamento de sessão solicitado pelo autor em 2026-10-21:** concluir a h4b1,
validar, registrar e fazer o último commit, sem iniciar a h4b2 nesta sessão.
A correção de foco ficou no editor existente; o harness reprova ao retirar
os handlers de Tab e passou após restaurá-los. Offscreen mede a intenção de
foco; a janela Xvfb comprova as teclas reais. Fotos nativas finais debug em
`/tmp/kinein-runconfig-native-proof/after-final-debug/shots/`: campos preenchidos
via Tab, retorno com Shift+Tab, recusa sem perda de bytes e salvamento após
reparo explícito da fixture. O report final traz preserved=false porque o
helper corrigiu a fixture e salvou intencionalmente; a asserção de preservação
foi executada antes desse reparo. ui.log vazio, sem hardware.

Os builds preliminares que saíram com 143/interrompidos não são prova de
build concluído; repetir antes do aceite. O primeiro gate completo também
foi interrompido durante a compilação Rust, sem veredito. A rodada final é
`/tmp/kinein-h4b1-gate-final-v2.log`, ainda em curso. Não publicar um gate
parcial como verde. Decisões de CLI têm dono na especificação de desktop
§3/§3.1; a análise ampliada foi alocada após 0.4.9 e revisão especial, sem
implementar novos comandos nesta etapa.


A revisão final encontrou um estado antigo que o novo erro de leitura torna
visível: RunConfigController.clear fechava a janela, mas mantinha lista/perfil
do workspace anterior. Se a nova leitura falhasse, o header continuava com
o perfil antigo. Acrescentar a regressão antes de corrigir; resetar também
modelo, ativa e identidade de edição no reset já existente. Não tratar falha
de leitura como lista válida vazia nem alterar dados do disco. A rodada v2
teve os 1.251 Rust verdes, mas foi parada antes de prosseguir com fontes
QML antigas; repetir o gate completo com a correção integrada.


**Fechamento h4b1 (2026-10-21):** gate completo estrito TUDO VERDE, sem NÃO
PROVADO: 1.251 Rust, Clippy, análise C++ limpa, 8 CTest, qmllint e 163 QML;
builds e abertura debug/release aprovados, passeio de 33 superfícies em cada
preset. Log `/tmp/kinein-h4b1-gate-final-v3.log`. Os logs anteriores
interrompidos não substituem esse veredito. Qt 6.4 também passou no harness
final de confirmação/reset, fora do gate local.

Provas: cinco mutações do armazenamento/ack em
`/tmp/kinein-h4b1-mutations.json`, mais a retirada dos handlers de foco
(`/tmp/kinein-h4b1-mutation-focus.log`). A regressão de reset falhou antes e
passou após a correção; foto nativa conferida em
`/tmp/kinein-h4b1-reset-native/shots/04-new-invalid-profile.png`, ui.log vazio.
O teste de configAction também passou com arquivo corrompido entre prévia e
aplicação. Dois cores reais mantiveram 16 atualizações/ids distintos e
comando manual explícito executou preservando o armazenamento inválido:
`/tmp/kinein-runconfig-core-proof-paoh3lu3/report.json`.

Fotos de erro, campos mantidos, teclado e recuperação após reparo explícito
conferidas em `/tmp/kinein-runconfig-native-proof/after-final-debug/` e
`after-final-release/`, logs da UI vazios. O caso anterior válido está em
`before-valid/`: a interface fechava e o arquivo era substituído. Sem placa
ou gravação de firmware. O guard Windows foi preservado pela fachada e
separação de arquivos, mas compilação/aceite Windows nativo não foram feitos
nesta máquina; o porte continua pausado conforme 60.

**Sessão encerrada após este commit, a pedido do autor.** Retomar pelo
principal: h4b2 ainda não iniciada — intenção tipada/gravação nativa/schema 2,
com capacidades/versões do binário selecionado (incluindo esptool v4/v5),
reserva serial pelo caminho resolvido e ações de ocupação/recuperação.
Não interpretar comandos shell salvos; não migrar configurações na abertura.
Depois: Setup no dock/schema 3, C e os demais escopos do §11 até a 0.4.9.
O checkout serial-lifecycle é rascunho anterior, já incorporado; não reaplicar
seu diff sobre o principal. Nenhuma nova etapa foi iniciada neste fechamento.
Decisões de CLI estão no dono de desktop §3/§3.1 e no 57 §3 item 9.
