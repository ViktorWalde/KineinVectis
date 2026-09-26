# 03 — Protocolo IPC

> **0.135.0 (2026-09-26) — a resposta diz sobre o que ela é.**
> Duas mudanças com o mesmo motivo. `syntaxTree.indent { path, version, line,
> column, trigger }` → `{ path, version, language, level, dedentTo? }`, em que a
> `version` devolvida é a da **árvore que respondeu**, e não a que foi
> perguntada: o autor continua digitando enquanto o pedido viaja, e comparar as
> duas é o que impede a correção de cair no buffer errado. Resultado `null` é
> resposta legítima — "a gramática não sabe aqui" —, e o editor fica com o
> fallback local que já aplicou.
> E `lsp.documentSymbols`/`lsp.workspaceSymbols` passam a devolver `path` e
> `query`: até aqui as duas respostas eram indistinguíveis, e uma resposta
> atrasada de `@nome` podia pintar a lista de `#nome`.
>
> **0.134.0 (2026-09-24) — a linha `ssh` colada vira um perfil.**
> `remote.parseCommand { command }` → `{ target, source[] }`. A pessoa cola a
> linha que já usa (`ssh -p 2222 pi@10.0.0.7`, ou só `pi@host`) e o core a **lê**
> — nunca a executa — devolvendo um perfil **proposto** com a procedência de
> cada campo. Nada é salvo: quem grava é o `remote.save`, depois da conferência.
> Não exige workspace.
>
> **0.133.0 (2026-09-24) — a falha do `ssh` vira um gesto.**
> `event.remote.probed` ganha `failure: authentication | host | network | other`
> — a MESMA causa que já escolhia a frase, agora tipada, para a UI oferecer um
> gesto sem ler a sentença. `remote.command` ganha `kind: copyId`, que **compõe**
> `ssh-copy-id [-p P] [-i K] [user@]host` com o `-p`/`-i` do perfil. Puro como
> os outros: nada roda. A UI mostra a linha e só a executa com um gesto
> explícito — copiar chave nunca acontece em silêncio.
>
> **0.132.0 (2026-09-24) — descobrir e explicar o SSH que a máquina já tem.**
> `remote.discover {}` → `{ aliases: [{ name, source }], sources[] }` lê o
> `~/.ssh/config` e os `Include` dele; só `Host` concreto entra. `remote.resolve
> { host }` → o subconjunto SEGURO de `ssh -G` (`hostName?`, `user?`, `port?`,
> `identities[]`, `proxyJump?`, `proxyCommand: bool`). Nenhuma das duas conecta
> nem varre rede, e o texto de um `ProxyCommand` nunca sai do core. Os dois são
> os únicos métodos de `remote.*` que **não** exigem workspace aberto.
>
> **0.131.0 (2026-09-24) — seleção do buffer completo do terminal.**
> `terminal.selectAll { id, selectionId }` marca todo o buffer ativo retido;
> `terminal.copySelection { id, selectionId }` lê essa seleção sob demanda.
> O render inclui `selectionId` (vazio quando invalidada). Saída nova,
> resize e limpeza invalidam; rolar preserva. Clipboard continua na UI.
>
> **0.130.0 (2026-09-22) — terminal ergonômico sem mentir sobre o PTY.**
> `terminal.clearScrollback { id }` descarta o histórico real da sessão no
> emulador e força um novo `event.terminal.render`; limpar a tela continua
> sendo o byte `Ctrl+L` (`\x0c`) enviado por `terminal.input`. A UI acrescenta
> atalhos de copiar/colar compatíveis com IDEs, menu contextual e seleção da
> área visível sem mudar a semântica de TUIs. Ver `terminal.*`.
>
> **0.129.0 (2026-09-19) — remover o que o banco criou.** `datasource.
> destroy { name, data? }`: sem `data`, só o perfil (o que `remove` faz);
> com `data`, por motor — o arquivo SQLite (só dentro do workspace; fora,
> fica, com `note`), o container `kinein-<name>` (`rm -f`, job High, com o
> comando), o banco dentro de um PostgreSQL (`DROP DATABASE` pelo banco
> `postgres` do mesmo servidor, job); MongoDB e o banco de manutenção nunca
> — só o perfil, com `note`. Resposta `{ profiles }` na hora ou `{ jobId,
> command }` + `event.datasource.destroyed { jobId, success, message,
> profiles? }`. Ver `datasource.*`.
>
> **0.128.0 (2026-09-19) — `SettingsValues.railExpanded`** (ausente =
> compacto): o trilho lateral com os rótulos ao lado dos ícones (o "modo
> expandido" que a F1 prometeu); o chevron do pé alterna e a escolha vale
> mesmo sem o resto do layout salvo. Sem método novo.
>
> **0.127.0 (2026-09-18, noite) — o Histórico por branch.** `git.log {
> maxCount?, ref? }`: `ref` é um branch ou tag de onde o log parte em vez
> do HEAD (validado como nome — sem espaço, sem `..`, sem `-` inicial;
> inexistente é erro do git). O filtro por texto (mensagem, autor, sha) é
> local à UI. Sem método novo.
>
> **0.126.0 (2026-09-18, noite) — a HUD do Git.** `git.log` ganha
> `parents` (`%P`) e `refs` (`%D`, já separados) por commit: o grafo e os
> chips do Histórico; `git.commit { message, amend? }` reescreve o último
> commit (`--amend`; um amend só de mensagem, sem nada staged, é
> legítimo). Sem método novo.
>
> **0.125.0 (2026-09-18, noite) — a execução é uma aba de terminal.**
> `run.start`/`run.script` → `{ command, terminalId }`: o comando abre num
> PTY (`sh -lc`, ou argv direto) e a saída chega por `event.terminal.render`
> dessa sessão, o fim por `event.terminal.closed { exitCode }`. Saíram
> `run.stdin` e `event.run.started/output/finished`; `run.stop` fecha a
> última execução. Decisão do autor no primeiro teste da Etapa 2: "já
> temos o terminal integrado". Ver "Execução".
>
> **0.124.0 (2026-09-18, noite) — o banco descobre e cria (pedido do autor no
> teste da Etapa 2: "a maior parte aparentou ser só visual").**
> `datasource.discover {}` → `{ candidates: [{ kind: localServer | container
> | file, label, detail, running, profile }], containerEngine?, hint? }` —
> o que responde NESTA máquina, medido: o socket do PostgreSQL de distro
> ou a porta 5432/27017 no loopback (200 ms), os containers com imagem de
> banco (parados também, ditos como parados; a porta publicada é a do
> perfil), os `.db/.sqlite/.sqlite3` do workspace com o cabeçalho `SQLite
> format 3` (até 4 níveis, sem `.git/.kinein/target/build/…`). Roda adiado
> (`defer_work`). `datasource.create { kind: sqliteFile, name, path? }` →
> `{ profile }` (arquivo criado e perfil salvo; recusa se existe) e
> `{ kind: containerServer, engine: postgres | mongo, name, port }` →
> `{ jobId, command }` (job `JobRisk::High`: `podman|docker run -d --name
> kinein-<name> -p 127.0.0.1:<port>:<interna> [-e POSTGRES_HOST_AUTH_METHOD=
> trust] <imagem pinada>`; ao subir, o perfil é salvo e vem em
> `event.datasource.created { jobId, success, profile?, message }`). Sem
> motor: `TOOL_NOT_FOUND`. Criar um banco DENTRO de um PostgreSQL é
> `CREATE DATABASE` pelo `datasource.query` confirmado — a tela compõe.
>
> **0.123.0 (2026-09-18) — Etapa 2, F3: `SettingsValues.autoSave`** (ausente
> = ligado; decisão do autor). O editor salva o buffer sujo sozinho — 2 s
> de pausa, troca de aba, perda de foco — pelo caminho do Ctrl+S; o rascunho
> de `seguranca/23` continua como rede. Sem método novo.
>
> **0.122.0 (2026-09-18) — P6 fatia 2, o workspace ESPELHADO (`roadmaps/42`
> §P6, desenho escrito antes do código).** A pasta de um alvo SSH vira um
> espelho local por `rsync`, e a IDE abre o espelho como workspace comum —
> `fs.*`, índice, busca, git e LSP não mudam; o que muda é a sincronia.
> `remote.open { name, path }` → job (`rsync -az -i --exclude .kinein -e 'ssh
> [-p] [-i] -o ControlMaster=auto -o ControlPath=<cache>/ssh-%C -o
> ControlPersist=60' [user@]host:<path>/ <espelho>/`) → `event.remote.synced
> { jobId, name, direction, success, command, changed[], error?, mirror }`;
> o espelho fica em `~/.cache/kinein-vectis/remote/<alvo>/<hash>/<basename>`
> com o marcador `.kinein/remote-mirror.json` e o alvo copiado para o
> catálogo DELE (o `.kinein` nunca sincroniza). `workspace.open` de um
> espelho responde `remote: RemoteMirror { name, host, path, mirrorRoot }`;
> `remote.status {}` → `{ mirror? }`; `remote.sync { direction: pull | push,
> paths? }` → job, nunca `--delete`; **um `fs.write` num espelho empurra o
> arquivo sozinho** (job `Empurrar <arquivo> para <alvo>`). Painel Remoto:
> "Abrir espelho", a faixa "este workspace é um espelho de…" com Puxar /
> Empurrar tudo. Métodos 160, eventos 57, domínios 36.
>
> **0.121.0 (2026-09-18) — banco: consultas, escrita e TLS (`roadmaps/35`
> §7.4, desenho escrito antes do código).** `datasource.query { name,
> password?, sql, maxRows? (500), confirmWrite? }` → job →
> `event.datasource.queried { jobId, name, success, columns[], rows[[texto |
> null]], rowCount, affected?, truncated, elapsedMs, message?, secretRequired
> }`. A primeira palavra classifica (SELECT/WITH/VALUES/TABLE/SHOW/EXPLAIN =
> leitura) e o **motor impõe**: `BEGIN READ ONLY` no PostgreSQL,
> `SQLITE_OPEN_READ_ONLY` no SQLite; o teto vem de fora do texto (`SELECT *
> FROM (<sql>) AS kinein_q LIMIT n+1`, ou o `step` até n+1). Uma instrução
> que não é leitura sem `confirmWrite: true` é recusada **antes do job** com
> o código novo `WRITE_CONFIRMATION_REQUIRED` — a UI pergunta e reenvia.
> Células em texto (protocolo simples do PostgreSQL; `ValueRef` do SQLite;
> Mongo `<coleção> <filtro JSON>` → `find` só leitura, colunas = chaves de
> primeiro nível). **TLS no PostgreSQL:** `DataSourceProfile.tls: disable |
> require` (+ `caFile` PEM) — `require` é o `verify-full` do libpq pelo
> `rustls` que o `mongodb` já trazia (`tokio-postgres-rustls` 0.14, MIT, +11
> crates, `cargo deny` verde). Métodos 157, eventos 56, domínios 36.
>
> **0.120.0 (2026-09-17, noite) — P6 fatia 1, o alvo Linux por SSH como
> recurso do projeto (`roadmaps/42` §P6, desenho escrito antes do código).**
> **Domínio novo `remote.*`** (o 36º), no molde do `datasource.*`: um perfil
> **sem senha em disco** (`RemoteTarget { name, host, user?, port?,
> identityFile?, deployDir? }` com `deny_unknown_fields` — um campo `password`
> é recusado pelo contrato, e um teste reprova qualquer chave que pareça
> segredo no `.kinein/remotes.json`); `remote.list/save/remove` síncronos;
> `remote.probe { name }` → job (`ssh -o BatchMode=yes -o ConnectTimeout=5
> [-p] [-i] [user@]host 'uname -m; uname -sr; command -v gdbserver python3
> rsync'`) → `event.remote.probed { jobId, name, success, arch?, kernel?,
> tools[{ id, found, path? }], error?, raw }` — a chave recusada vira "copie
> a sua com `ssh-copy-id user@host`"; `remote.deploy { name, source?, dest? }`
> → job (`rsync -az --delete -e 'ssh …' <origem> [user@]host:<dest>/`, ou
> `scp -r` sem `rsync`) → `event.remote.deployed { jobId, name, success,
> source, dest, command, error? }`; `remote.command { name, kind: run |
> debugServer | debugpy | shell, program?, port? }` PURO → `{ command,
> remoteTarget?, name, source[] }` — a linha `ssh -tt … '<comando>'` que a UI
> grava como configuração de execução ("Rodar em pi"), como `debugServer` +
> `remoteTarget = host:2345` do kit (`toolchain.setKit` só com os dois
> campos — a ponte ganhou `toolchainSetKitRemote`) ou manda ao terminal.
> Painel **Alvo remoto (SSH)** no menu Ambiente e comando `remote.list` na
> paleta. Métodos 156, eventos 55, domínios 36.
>
> **0.119.0 (2026-09-17, noite) — P5, qualidade: clang-tidy, gtest/Catch2,
> cobertura, a lâmpada proativa (D6–D8 do `roadmaps/41`).** (1) **clang-tidy
> em dois lugares**: o clangd sobe com `--clang-tidy` (o clangd 21 desta
> máquina lista a flag; ele lê o `.clang-tidy` do projeto sozinho — os
> avisos entram no canal dos diagnósticos, arquivo aberto a arquivo aberto) e
> o `quality.run` de um `CMake`/`Makefile` roda o projeto inteiro pela CDB:
> `run-clang-tidy -p <cdb> -quiet` (o script do LLVM) ou `clang-tidy -p <cdb>
> <arquivos da CDB>`; sem CDB, "configure" — sem `clang-tidy`, a ferramenta.
> (2) **gtest/Catch2 na árvore**: o `test.discover` de um `CMake` pergunta a
> cada binário do `ctest --show-only=json-v1` os casos de dentro
> (`--gtest_list_tests`; `--list-tests --verbosity quiet`) e os lista com o
> id `<teste do ctest>::<caso>`; `test.run { testId }` com esse id roda o
> binário com `--gtest_filter=` (ou o nome, no Catch2). (3) **Domínio novo
> `coverage.*`** (o 35º): `coverage.run {}` → job (`cargo llvm-cov --lcov
> --output-path .kinein/coverage.lcov`; `coverage.py run -m pytest` +
> `coverage lcov` pelo interpretador do projeto; C/C++ recusado com o motivo)
> → `event.coverage.finished { jobId, success, tool, path?, files: [{ file,
> linesFound, linesHit }], error? }`; `coverage.lines { file }` → `{ file,
> known, covered[], missed[] }` do último LCOV — a calha do editor pinta
> (barra verde/vermelha ao lado do diff). Comando `coverage.run` na paleta e
> no menu Build. (4) A **lâmpada**: na linha do cursor com diagnóstico a
> calha mostra 💡 antes do Alt+Enter; o clique é o Alt+Enter. Métodos 150,
> eventos 53, domínios 35.
>
> **0.118.0 (2026-09-17, noite) — P3, o que o depurador de embarcado MOSTRA
> (D1–D4 do `roadmaps/41` bloco D), como passagem do DAP padrão — a
> "solução pronta" não é uma extensão do VS Code, é o protocolo que
> `probe-rs dap-server` 0.32 e `gdb -i dap` 17 anunciam (medido nesta
> máquina: `supportsReadMemoryRequest`, `supportsDisassembleRequest`,
> `supportsSetVariable`, `supportsWriteMemoryRequest`).** (1) **O `launch`
> do probe-rs estava errado**: `program`+`chip` no topo falha com `missing
> field coreConfigs` (medido); agora é `{ cwd, chip?, coreConfigs: [{
> coreIndex: 0, programBinary, svdFile?, rttEnabled: true }] }` (lido em
> `server/configuration.rs` do 0.32.0). (2) **`svdFile` no kit**
> (`toolchain.setKit { svdFile? }`, `ToolchainResult.svdFile?`): o CMSIS-SVD
> do chip, que o probe-rs transforma no escopo `Peripherals`. (3) **RTT/
> defmt**: os eventos `probe-rs-rtt-channel-config`/`probe-rs-rtt-data` viram
> `event.debug.output { category: "rtt", channel, channelName?, line }`, e o
> core responde o `rttWindowOpened` que o probe-rs exige para começar a ler;
> `probe-rs-show-message` vira `console`. (4) **`debug.scopes { frameId }`**
> → `{ frameId, scopes: [{ name, ref, expensive }] }` (os escopos inteiros —
> `Locals`, `Registers`, `Peripherals` — que o `debug.variables { frameId }`
> esconde ao escolher um); **`debug.readMemory { memoryReference, offset?,
> count }`** → `{ address, unreadableBytes?, data (base64) }` e
> **`debug.disassemble { memoryReference, offset?, instructionOffset?,
> instructionCount }`** → `{ instructions: [{ address, instruction,
> instructionBytes?, symbol?, file?, line? }] }`, verbatim. Na UI, o filho
> `inspect` do `DebugController`. Métodos 148, eventos 52.
>
> **0.117.0 (2026-09-17, noite) — bloco E, os frameworks como MOTORES de
> build/gravar/monitorar (E1–E4 do `roadmaps/41`).** O `project.model` já
> reconhecia ESP-IDF, Zephyr, pico-sdk e PlatformIO; agora o `build.run` os
> COMPILA pelo wrapper de cada um, sem campo novo: **PlatformIO** → `pio run`
> (`platformio.ini` é tipo de projeto novo, `kind: platformIo` /
> `buildSystems: [platformIo]`; vence até um `CMakeLists.txt` do ESP-IDF ao
> lado); **ESP-IDF** → `idf.py build` DENTRO do ambiente ativado (`bash -c
> '. <ativação> && exec idf.py …'`, a ativação sendo o `export.sh` de
> `IDF_PATH`/`~/esp/esp-idf` ou o `activate_idf_<versão>.sh` mais novo de
> `~/.espressif/tools` do EIM — as duas formas do "Get Started"); **Zephyr**
> → `west build -d build [-b <placa>]`, a placa do `CACHED_BOARD` do
> CMakeCache ou do `west config build.board`; **pico-sdk** → o CMake de
> sempre com `-DPICO_SDK_PATH=<sdk>` no configure (também no
> `cmake.configure` automático). Framework reconhecido e ferramenta ausente é
> `TOOL_NOT_FOUND` com o passo do modelo ANTES do job. **Gravar** ganhou os
> motores `idf.py` (`-p <porta> flash`, ativado), `west` (`west flash -d
> build`, o padrão de um projeto Zephyr) e `platformio` (`pio run -t upload
> [--upload-port <porta>]`, o padrão de um PlatformIO); **monitor** ganhou
> o IDF Monitor (`idf.py -p <porta> monitor`, ativado) e `pio device monitor
> -p <porta> -b <baud>`, antes do catálogo quando o papel não está fixado.
> O modelo do projeto passou a ler os binários pelo detector do core (não
> mais o PATH do processo por conta própria). Métodos 145, eventos 52.
>
> **0.116.0 (2026-09-17, noite) — P4, MicroPython em três fatias (C2, C5, C4
> do `roadmaps/41` bloco C).** (1) **Arquivos na placa:** `serial.files
> { device, action: list|get|put|rm|mkdir, path?, local?, tool? }` →
> `{ jobId, command }` (job `mpremote connect <dev> fs <ls|cp|rm|mkdir>`;
> `event.serial.files { jobId, device, action, path, command, success,
> error?, entries?: [{ name, size, directory }], local?, raw }`). Todo `fs`
> entra no raw REPL e interrompe o programa da placa (`JobRisk::Medium`);
> o que escreve (`put`/`rm`/`mkdir`) é `High` e a tela confirma antes. Uma
> repetição quando a placa não deixa entrar no raw REPL (medido no ESP32 do
> autor: o firmware inundando a UART). `get` baixa para `placa/<caminho>`
> sob o workspace (o espelho da placa) e abre no editor. (2) **Firmware
> oficial:** o catálogo de `toolchain.installable` ganhou `kind:
> toolchain|firmware` e `firmware?: { board, engine, offset?, chip?, file }`
> — cinco releases pinadas do micropython.org v1.29.0 (ESP32_GENERIC,
> _C3, _S3, RPI_PICO, RPI_PICO_W), baixadas pelo mesmo provedor (arquivo
> só, sem `tar`; a fonte NÃO publica checksum: o SHA-256 foi medido no
> download de 2026-09-17 e o `source` o diz); `runConfig.flashProposal
> { firmware? }` grava o arquivo baixado pela linha da página da placa
> (`write-flash 0x1000` no ESP32 clássico, `0x0` em C3/S3; `picotool load
> -f -x` no Pico) com o aviso do `erase-flash` da primeira vez. (3)
> **Stubs por placa:** `python.stubs { port?, board? }` → `{ jobId,
> package, command, target }` (job `uv pip install --target <root>/typings
> micropython-<port>[-<board>]-stubs`, ou o `pip` do interpretador do
> projeto; `event.python.stubs { jobId, success, package, command,
> target }`); `python.status` ganhou `stubsPath?` e `stubsSuggested?` (o
> chip do kit/identidade escolhe a placa: `esp32c3` →
> `micropython-esp32-esp32_generic_c3-stubs`); o basedpyright recebe
> `basedpyright.analysis.stubPath` e `reportMissingModuleSource: none`.
> Também: `serial.identify` passou a devolver `chip: esp32` para os
> encapsulamentos do clássico (`ESP32-D0WD-V3` etc.; medido na placa —
> antes saía `esp32d0wdv3`), e o roteador QML dos filhos do painel de
> Embarcados (identidade, gravar, permissão) foi consertado — o core
> respondia e a tela não recebia. Métodos 145, eventos 52.
>
> **0.115.0 (2026-09-17) — P0, o modelo do projeto em três fatias.** (1) O
> `cmake.configure` sem `preset` escolhe o do **kit ativo** (a ponte manda o
> último `toolchain.get`) ou o **padrão do projeto** — o primeiro
> `configurePresets` não oculto de `CMakeUserPresets.json`, depois de
> `CMakePresets.json`, cuja `condition` não exclua o Linux —, diz de onde veio
> em `event.cmake.started { preset?, presetSource? }` e anota o usado para o
> `cmake.status { preset? }`. (2) **Makefile puro** é `kind: make` /
> `buildSystems: [make]` (`Makefile`/`GNUmakefile`, atrás do CMake); `build.run`
> roda `bear -- make` quando o `bear` existe (a CDB sai na raiz, onde o
> clangd a acha) e `make` a seco dizendo o passo quando não; `bear` entrou
> nas ferramentas conhecidas e no catálogo de instalação. (3) O `targetTriple`
> do kit chega ao **rust-analyzer** (`rust-analyzer.cargo.target`, secção
> `rust-analyzer` no `workspace/configuration` e no `didChangeConfiguration`;
> servidor vivo é reiniciado ao mudar o kit).
>
> **0.114.0 (2026-09-17) — E2, permissão POR CANAL:** `serial.access
> { device? }` → `{ channels: [{ kind: serial|probe|modemManager, device?,
> ok, detail, problem?, fix?: { steps: [{ explanation, command }],
> sourceUrl, checkedOn }, distroDidIt? }] }`. Mede (nunca supõe) o nó serial
> (grupo do nó × grupos do processo, ACL pela tag `uaccess`), o `ModemManager`
> (rodando × candidata × regra de ignorar) e a regra udev das sondas nas
> pastas do udev; para o que falta, o passo OFICIAL com fonte e data —
> escrito no terminal da IDE pelo usuário, nunca `sudo` pela IDE. Medido com
> o ESP32 real desta máquina: acesso por `uaccess` sem grupo, `ModemManager`
> candidato sem regra (passo: `77-mm-kinein-10c4-ea60.rules`), regras de
> sonda da distro. Método 143.
>
> **0.113.0 (2026-09-17, noite) — E4, gravar como CONFIGURAÇÃO DE EXECUÇÃO:**
> `runConfig.flashProposal { device?, engine?, flashSizeBytes? }` → `{ name,
> command, engine, source[], warnings[] }` é PURO — compõe a linha do motor
> (`esptool write-flash` da receita `flasher_args.json`; `probe-rs download
> --chip`; `picotool load -f -x`; `dfu-util` só STM32) a partir do que o
> `project.model` já leu e da porta escolhida, sem rodar nem salvar nada. A
> tela mostra a prévia; **Gravar agora** é `run.start { command }` e **Salvar**
> é `runConfig.save` (vira a ativa: o botão Executar grava). Não há domínio
> `flash`. Método 142.
>
> **0.112.0 (2026-09-17, fim de tarde) — E5, a identidade Espressif PELO
> CANAL:** `serial.identify { device, tool? }` → `{ jobId, command }` roda
> `esptool --port <device> … flash-id` como JOB (reseta a placa: gesto
> explícito, nunca ao abrir o painel), com parser tolerante e queda para o
> `flash_id` da v4; o desfecho é **`event.serial.identified`** (evento 50) com
> `identity` (chip, features, cristal, MAC, flash) e `target` — o kit que o
> chip SUGERE pelas tabelas do `project.model`; aplicar é clique
> (`toolchain.setKit`). Recusa antes de abrir a porta: nó inexistente,
> sem permissão, sem esptool. Método 141.
>
> **0.111.0 (2026-09-17, à tarde):** o stderr dos filhos de longa vida deixou
> de ir para `/dev/null`. Adaptador DAP: cada linha sai como `event.debug.output
> { category: "adapter" }` e a cauda (últimas 64 linhas) entra na mensagem do
> `debug.start` que falha. Servidor de debug do kit (`debugServer`): a cauda
> entra em "saiu antes de abrir a porta"/"não abriu em N s". Language server:
> evento novo **`event.lsp.log { language, line }`** por linha, e a cauda entra
> no `event.lsp.status` `failed` (via mensagem) e `exited` (`message`). Um LSP
> que falha o `initialize` agora é morto, não fica órfão. Medido nesta máquina
> (30 s após abrir um arquivo): rust-analyzer 4 linhas, clangd 66 (pico 18/s no
> índice), basedpyright+ruff 3 — sem agrupamento, por número.
>
> **0.110.0 (2026-09-17):** a porta escolhida chega ao Executar de MicroPython.
> `run.start { device? }` — o mesmo `device?` que `run.script` já tinha desde
> `0.102.0` — leva ao lançador padrão (`mpremote connect <device> run main.py`).
> `device` com `command` é contradição (`INVALID_PARAMS`); `device` presente e
> vazio é recusado nos dois métodos ("campo ausente" é o mpremote escolhendo;
> "campo vazio" seria `mpremote connect '' run`). A tela agora ESCOLHE a porta
> no painel de Embarcados (chip "Executar" por porta) e a passa nos dois gestos.
>
> **0.109.0 (2026-09-16):** debugpy attach via `debug.start { connect: { host,
> port } }`. Transporte DAP TCP reutiliza a sessão existente; `attached` no
> resultado/evento diferencia desconectar um processo externo de encerrar launch.


> **O `0.108.0` (2026-09-13, fim de tarde) é o "sintoma do Docker" medido e
> fechado até onde a medição alcança:** o core respondeu certo o tempo todo
> (`status`/`list`/`images` reais do Podman 5.8.4, start/stop reais); o que
> estava errado era a PROMESSA da tela — "compose up" primário e aceso sem
> projeto (um botão primário desligado vestia o âmbar), e aceso com projeto
> sem arquivo de compose (o job só podia falhar). `ContainerStatus.composeFile`
> nasce (o arquivo que a ferramenta pegaria na raiz do workspace), o
> `container.compose` sem arquivo recusa antes do job, e os botões da IDE só
> acendem com o que funciona. Campo novo sobe o minor.
>
> **O `0.107.0` (2026-09-13) fecha dois itens do polimento Python:** o
> **módulo como alvo de debug** — um projeto cujo ponto de entrada é um
> pacote com `__main__.py` é depurado como `-m pacote` (o `module` do launch
> do debugpy, medido no 1.8.21 e provado pelo core real no gate: para no
> breakpoint dentro do pacote, `x=21`, `dobro 42`); `DebugStartResult.program`
> mostra `-m pacote` — e **`run.capabilities`**: o core publica o que
> "Executar" e "Depurar" aceitam por extensão (`runnable: sh bash zsh py`,
> `debuggable: py`), a UI pede uma vez por conexão e não mantém lista
> própria (a mesma invariante do `format.capabilities`; a árvore e o
> explorador tinham a lista duplicada em dois QML). Método novo sobe o minor.
>
> **O `0.106.0` (2026-09-13) é a ÁRVORE DE TESTES antes do primeiro run** —
> o que faltou do B6 do [`roadmaps/41`](../roadmaps/41-ecossistema-embarcados-e-python.md):
> `test.discover { buildSystem? }` é um JOB que lista sem rodar (`pytest
> --collect-only -q`, `cargo test -- --list`, `ctest -N`) e termina em
> `event.test.discovered` com um `TestCaseInfo { id, name, file? }` por
> caso, no **id exato** que o mesmo runner aceita; `test.run` ganhou
> `testId?` — pytest recebe o node id **posicional** (não `-k`), cargo
> `<nome> -- --exact`, ctest `-R ^nome$` escapado — e ele vence `filter`. O
> painel Testes mostra a árvore com o status do último run e um "rodar só
> este" por linha; o caso que roda pinta a linha da árvore porque o id é o
> mesmo. Método, evento e campo novos sobem o minor.
>
> **O `0.105.0` (2026-09-13, à tarde) faz o Python APARECER na IDE** — o B8
> do [`roadmaps/41`](../roadmaps/41-ecossistema-embarcados-e-python.md)
> ("só aqui a tela diz Python"), pedido do autor: `workspace.createProject`
> ganhou o template `python` (PEP 621, layout plano, pytest em `[dev]`,
> `ruff`, `main.py` como ponto de entrada — nada é executado; o `.venv` é o
> clique da faixa de saúde), a barra de status mostra o Python do projeto
> (`python: .venv · 3.14.7 · pybind11 (scikit-build-core)`), o `.py` tem
> ícone na árvore (o `>>>` do REPL — não o logotipo, marca da PSF), o menu
> Build ganhou "Testar com pytest" e "Análise (ruff)". Na mesma passada, por
> decisão do autor: o rail ganhou o ícone de **banco de dados** acima de
> Containers e "Ferramentas" fecha a lista; e Logs/Shell de um container sem
> projeto aberto passaram a dizer antes do clique que a aba de terminal é do
> projeto (o core recusava depois). Valor novo de enum no wire sobe o minor.
>
> **O `0.104.0` (2026-09-13) é o GERENCIADOR DE TOOLCHAIN QUE LÊ O DISCO**
> (`roadmaps/42` §8 itens b e d; `integracoes/39` §3): `toolchain.
> inspectSysroot { path }` diz o que uma pasta de sysroot contém (headers,
> bibliotecas, `lib/`, os `usr/lib/<triple>` do multiarch, quantos `.pc`,
> qual libc — e um veredito: utilizável, só headers, só bibliotecas, vazia);
> `toolchain.importKit { path }` lê um SDK Yocto (o `environment-setup-*`
> carregado pelo `sh`), uma árvore Buildroot (`output/host`) ou uma pasta de
> toolchain e devolve uma PROPOSTA de kit — compiladores, gdb, sysroot,
> triple, arquivo de toolchain, evidência — sem gravar nada; o kit ganhou
> `toolchainFile` (`setKit`/`ToolchainResult`), que vira
> `-DCMAKE_TOOLCHAIN_FILE` no configure quando o preset não declara um.
> Métodos e campo novos sobem o minor.
>
> **O `0.103.0` (2026-09-13) é o PROVEDOR DE INSTALAÇÃO de toolchain**
> ([`integracoes/39`](../integracoes/39-toolchains-por-alvo.md) §5, o item
> de cima da fila depois da cadeia Python): `toolchain.installable` publica
> um catálogo PINADO — nove toolchains (Arm GNU 15.2.rel1 ×3, xPack ×2,
> ATfE 23.1.0, Bootlin 2026.08-1 ×3) com URL, tamanho, SHA-256 **lido na
> fonte em 2026-09-13**, licença, onde vai parar e se já está lá, e o que o
> projeto aberto recomenda; `toolchain.install { id }` é um JOB que baixa
> (`ureq`, TLS por rustls) para `~/.local/share/kinein-vectis/toolchains/
> <id>/<versão>`, confere o SHA-256 **antes** de desempacotar, desempacota
> com o `tar` do sistema e termina em `event.toolchain.installed`; o detector
> lê a pasta da IDE a cada busca, então a toolchain vira candidato do kit sem
> reiniciar. Nunca sem clique, nunca no sistema, nunca sem checksum, nunca
> "latest". Métodos e evento novos sobem o minor.
>
> **O `0.102.0` (2026-09-13) fecha a cadeia Python do
> [`roadmaps/41`](../roadmaps/41-ecossistema-embarcados-e-python.md) bloco B
> — a fatia 5, MicroPython e o módulo nativo:** num projeto MicroPython (a
> MESMA evidência do `project.model`: `main.py`/`boot.py` importando
> `machine`/`board`) o `serial.monitor` abre o **REPL da placa** (`mpremote
> connect <porta> repl`, candidato novo — e último — do papel
> `serialMonitor`) e o `run.script`/`run.start` de um `.py` roda o arquivo
> **na placa** (`mpremote [connect <porta>] run <arquivo>`; `run.script`
> ganhou `device?`); sem mpremote o erro diz o que instalar, em vez de rodar
> um `import machine` no Python do desktop — o pytest continua no host. E
> `python.status` ganhou `nativeModule` (pybind11/nanobind/PyO3 e a
> ferramenta que o instala no ambiente: maturin, scikit-build-core,
> setuptools-rust, setuptools — com a evidência e o comando oficial). Campo e
> parâmetro novos sobem o minor.
>
> **O `0.101.0` (2026-09-13) é a fatia 4 da cadeia Python do
> [`roadmaps/41`](../roadmaps/41-ecossistema-embarcados-e-python.md) bloco B —
> depurar Python com o debugpy DO interpretador do projeto:** um alvo `.py`
> em `debug.start` (explícito, ou o ponto de entrada do Executar num
> workspace Python) não passa pelo kit — o adaptador é `<interpretador> -m
> debugpy.adapter`, o mesmo interpretador do status, do índice, do
> basedpyright, do run e do pytest. Antes de subir, o core **pergunta ao
> interpretador** se o módulo existe (`-I -c "import debugpy"`); sem ele,
> `TOOL_NOT_FOUND` com o passo para instalar NO ambiente (nova variante
> `MissingAdapterModule`, daí o minor). O ciclo inteiro — breakpoint,
> locais, evaluate, saída do programa, `exitCode`, adaptador morto com a
> sessão — está provado contra o debugpy 1.8.21 real por
> `scripts/verificar-python-debug.sh` (a 23ª verificação do gate; fica "não
> provado" na máquina sem debugpy). A árvore ganhou "Depurar" para `.py`.
>
> **O `0.100.0` (2026-09-13) é a fatia 3 da cadeia Python do
> [`roadmaps/41`](../roadmaps/41-ecossistema-embarcados-e-python.md) bloco B —
> executar e testar Python com o interpretador DO PROJETO:** `run.script`
> aceita `.py` (o interpretador da precedência do `29` §4.1, ou `uv run
> python` quando o projeto tem `uv.lock` e o uv existe; sem shell, cwd no
> root); `run.start` sem comando acha o **ponto de entrada por evidência**
> (`main.py`/`app.py`/`__main__.py` na raiz, UM pacote com `__main__.py`, um
> script de `[project.scripts]` instalado) e diz o que procurou quando não
> acha; `test.run` aceita workspace Python — `python -m pytest -v` no mesmo
> interpretador, `-k` com o filtro, cada linha `-v` um `event.test.case`, e
> **sem o módulo pytest naquele ambiente** o erro diz como instalar NELE.
> `event.test.finished` ganhou `error` quando o runner nem correu (campo novo
> no wire, daí o minor). E a saída bruta dos testes finalmente chega à tela
> (`41` A6): o painel Testes mostra `event.test.started`/`output`.
>
> **O `0.99.0` (2026-09-13) é a fatia 2 da cadeia Python do
> [`roadmaps/41`](../roadmaps/41-ecossistema-embarcados-e-python.md) bloco B —
> três contratos existentes ganharam Python, sem método novo:** o servidor de
> Python (`basedpyright-langserver --stdio`, o binário DETECTADO) sobe **com o
> interpretador do projeto** — o core empurra `workspace/didChangeConfiguration
> { python.pythonPath, … }` logo após o `initialized` e responde ao
> `workspace/configuration` que o servidor pergunta, seção a seção; o
> `event.python.finished` com sucesso reinicia esse servidor com o ambiente
> novo. `format.text` formata `.py`/`.pyi` com `ruff format` (o `format.
> capabilities` publica o id `ruff` — valor novo no catálogo, daí o minor). E
> `quality.run` aceita workspace Python: `ruff check --output-format concise
> --no-fix` no root, cada linha um `event.quality.diagnostic`, e sem ruff o
> erro **nomeia a ferramenta e o passo oficial**, não "tipo não suportado".
>
> **O `0.98.0` (2026-09-12, noite) acrescentou o domínio `python`** — a fatia
> 1 da cadeia Python do [`roadmaps/41`](../roadmaps/41-ecossistema-embarcados-e-python.md)
> bloco B: `python.status` (o interpretador que o projeto resolve, se é
> ambiente próprio ou o Python do sistema, com que ferramenta a IDE criaria um
> `.venv`, os arquivos de projeto, a dica) e `python.createEnvironment` (`uv
> venv .venv` ou `python3 -m venv .venv` como JOB, com `event.python.finished`
> ao fim — e o `index.context` recarregado, para o interpretador do projeto
> passar a ser o do ambiente novo). Domínio novo sobe o minor.
>
> **O `0.97.0` (2026-09-12, noite) acrescentou ao `ToolchainResult` o que só
> um processo responde:** `rustTargets` (os alvos Rust instalados, pelo
> `rustup` detectado) e `sysrootHint` (o compilador cross de distro sem o
> sistema alvo, medido com `-print-sysroot`) — a base do gerenciador de
> toolchains do [`integracoes/39`](../integracoes/39-toolchains-por-alvo.md),
> que no mesmo dia pôs no catálogo os triples da indústria e ensinou o
> detector a procurar além do `PATH`. Campos novos sobem o minor.
>
> **O `0.96.0` (2026-09-12, fim de tarde) acrescentou o MODELO POR ALVO do
> `CMake`:** `cmake.targets.list` passa a trazer, por target, artefatos,
> fontes (geradas à parte), linguagens, padrão, includes/defines, sysroot,
> dependências e pasta de fonte — lidos do `codemodel-v2` do file-api, com a
> `toolchains-v1` (CMake ≥ 3.20) pedida junto; e `index.context` ganha
> `targets` (que targets compilam ou listam o arquivo — o inverso) e, quando
> não há `compile_commands.json`, a **unidade vinda do file-api** (grupo de
> compilação + compilador da `toolchains-v1`) — a "CDB em memória" do
> `roadmaps/42` §8. Campos novos sobem o minor.
>
> **O `0.95.0` (2026-09-12) acrescentou o CONTEXTO DE COMPILADOR por arquivo
> ao domínio `index`:** `index.context { path }` diz COM QUE cada arquivo é
> compilado ou executado — a unidade da `compile_commands.json` (compilador,
> `-std`, `-I`, `-D`, diretório) para C/C++, o pacote e alvo do
> `cargo metadata` para Rust, o interpretador resolvido para Python — e
> `IndexStats.context` resume o que o índice carregou (`cdbEntries`,
> `cdbStale`/`cdbStaleBecause`, `cargoTargets`, `pythonInterpreter`). É a
> segunda metade da exigência do autor ("integração profunda de leitura do
> contexto do código/compilador"). Campo e método novos sobem o minor.
>
> **O `0.94.0` (2026-09-12) acrescentou o domínio `index`:** a IDE passa a
> ler o projeto INTEIRO que abre — todas as pastas, arquivos e declarações
> (funções, tipos) de C, C++, Rust e Python — por decisão do autor no mesmo
> dia, sem esperar language server. `index.status` dá os totais, `index.symbols`
> busca por nome, `event.index.progress`/`finished` acompanham o job que o
> `workspace.open` sobe. É a primeira forma concreta do "entender o projeto
> inteiro" da especificação do KSWE, com as gramáticas Tree-sitter do editor.
> Domínio novo sobe o minor.
>
> **O `0.93.0` (2026-09-12) acrescentou o domínio `project`:** o MODELO do
> projeto embarcado (pilar 0 do `roadmaps/42`). `project.model` diz o que o
> projeto É — framework com o arquivo que o prova, SDKs exigidos e se estão
> aqui, artefatos do último build, alvo deduzido com evidência — e
> `event.project.changed` o reemite ao abrir o workspace e ao fim de
> configure/build. Nada é adivinhado calado; o que não se decide vira `hint`.
> No mesmo minor, `serial.monitor` e o papel `serialMonitor` do kit.
>
> **O `0.92.0` (2026-09-12) acrescentou o domínio `container`:** Docker e
> Podman como domínio NATIVO (decisão do autor de 2026-07-17, `roadmaps/28`
> §0, priorizada em 2026-09-12). `container.status` é a tela de "ativar a
> ferramenta" (motor, versão, rootless, socket, responde, compose, passo
> oficial); `list`/`images` leem `ps`/`images` em JSON dos dois motores;
> `action` e `compose` são JOBS com `event.container.finished`; `open` abre
> logs ou um shell numa aba de terminal. A UI nunca chama `docker`. Domínio
> novo sobe o minor.
>
> **O `0.91.0` (2026-09-11) acrescentou o domínio `serial`:** `serial.list`
> enumera as portas seriais USB desta máquina pelo sysfs — `ttyUSB*` (ponte)
> e `ttyACM*` (CDC) — com VID:PID, driver, o que o VID:PID diz do **elo** (nunca
> do chip atrás de uma ponte), a permissão **medida** com `access(2)` e o estado
> do ModemManager via `udevadm`. Nunca abre a porta: abrir aciona DTR/RTS e
> reseta a placa. Domínio novo sobe o minor. É a E1 do
> [`../integracoes/38`](../integracoes/38-conectividade-bare-metal.md) §6.
>
> **O `0.90.0` (2026-09-11) acrescentou `build.size`:** o tamanho do ELF
> medido por `<prefix>size` do kit, com a fração usada de cada região do
> linker script (`BuildSizeParams` → `SizeReport`). Método e tipos novos sobem
> o minor. Detalhe na seção Build; a medição está no
> [`../roadmaps/35`](../roadmaps/35-ambiente-cpp-e-embarcados.md) §5.7.
>
> **O `0.89.0` (2026-09-11) acrescentou o ALVO REMOTO do depurador:**
> `remoteTarget` e `debugServer` no `toolchain.setKit` e no `ToolchainResult`.
> Com eles o adaptador `gdb` (que fala DAP desde a v14) faz `attach` a um
> servidor GDB — QEMU, OpenOCD — que a IDE sobe e mata com a sessao, em vez de
> `launch`. Campos novos no contrato sobem o minor. A medicao esta' no
> [`../roadmaps/35`](../roadmaps/35-ambiente-cpp-e-embarcados.md) §5.7
> e o ciclo completo e' provado no QEMU pelo `scripts/verificar-embarcado.sh`.
>
> **Os `0.83.0`–`0.88.0` (2026-09-05 → 2026-09-10) foram o domínio `sim`** —
> a simulação por conceito, que **saiu do produto em 2026-09-12** por decisão
> do autor (métodos `sim.*` e tipos `Sim*`). As notas destas
> versões estão íntegras em `DocsPrivate/historico/simulacao/`; o número do
> protocolo não volta atrás: versão é história, não inventário.
>
> Antes disso, a sincronização de 2026-09-06 — e **os comentários dentro dos
> comandos, que ainda diziam 128/130**. Comentário dentro de comando envelhece
> igual a número solto; a diferença é que o gate não o vê.
>
> **Escopo, SINCRONIZADO em 2026-09-06 — e a dívida que este cabeçalho
> declarava foi paga.** Em 2026-09-05 ele foi corrigido para parar de afirmar
> cobertura que não tinha: cinco domínios estavam roteados pelo core e ausentes
> daqui. **Os cinco ganharam seção**: `command.*`, `setup.*`, `datasource.*`,
> `grafana.*` e `sim.*` (este último removido com o domínio em 2026-09-12).
>
> **E a sincronização achou dois números errados — nos comandos que os provam.**
> Ambos pela mesma causa: eles grepam literais sem saber o que os literais são.
>
> ```bash
> # METODOS: o braco de despacho nunca comeca com `event.`. Sem o filtro, a
> # contagem inclui dois nomes de EVENTO que aparecem num `match` dentro de
> # `#[cfg(test)]` em handlers/build.rs — e da' 132 em vez de 130.
> grep -rhoE '"[a-z][a-zA-Z]*\.[a-zA-Z][a-zA-Z.]*"\s*(\||=>)' \
>      crates/kinein-core/src/handlers/ crates/kinein-core/src/lib.rs \
>   | grep -oE '"[a-z][a-zA-Z]*\.[a-zA-Z][a-zA-Z.]*"' | tr -d '"' \
>   | grep -v '^event\.' | sort -u | wc -l          # 130
>
> # EVENTOS: cinco sao construidos por `format!("event.{domain}.*")` em
> # handlers/build.rs, com `domain` em {build, quality}. Um grep de literal
> # NAO OS VE, e da' 36 em vez de 41.
> { grep -rhoE '"event\.[a-zA-Z.]+"' crates/kinein-core/src/ | tr -d '"'
>   for d in build quality; do
>     for s in started output diagnostic finished; do echo "event.$d.$s"; done
>   done
> } | sort -u | wc -l                                # 41
> ```
>
> ```text
> protocolo   0.88.0
> metodos     131 roteados
> eventos     41 (36 literais + 5 construidos por format!)
> dominios    30, e os 30 tem secao neste documento
> ```
>
> **Os cinco que só existem por `format!`:** `event.build.started`,
> `event.build.output`, `event.build.diagnostic`, `event.quality.started` e
> `event.quality.output`. Os outros três da mesma família aparecem como literal
> em algum ponto do código e por isso o grep os via.
>
> O protocolo-**alvo** completo está em
> `DocsPublic/especificacoes/arquitetura-interna-core-ipc-jobs.md`. Onde
> divergir, vale o que está implementado no código + `DocsPrivate/ContextoIA.md`.

## Objetivo

O protocolo IPC permite comunicação entre:

```text
Kinein Vectis UI  ←→  Kinein Vectis Core
```

A UI deve mandar comandos e receber respostas/eventos. O core deve executar lógica, chamar ferramentas externas e emitir eventos de estado.

## Transporte inicial

Para o MVP:

```text
stdin/stdout JSON-RPC
```

Depois:

```text
Unix domain socket
```

Futuro:

```text
gRPC/local socket
```

## Formato base

### Request

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "core.ping",
  "params": {}
}
```

### Response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "status": "ok",
    "message": "pong"
  }
}
```

### Error

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": "TOOL_NOT_FOUND",
    "message": "clangd não foi encontrado",
    "details": {
      "tool": "clangd",
      "suggestedCommand": "sudo pacman -S clang"
    }
  }
}
```

### Event

```json
{
  "jsonrpc": "2.0",
  "method": "event.diagnostics.updated",
  "params": {
    "uri": "file:///home/vitor/dev/projeto/src/main.cpp",
    "errors": 1,
    "warnings": 0
  }
}
```

### Diagnóstico comum (`Diagnostic`)

Implementado no protocolo `0.20.0` como modelo comum para Problems. Os eventos
continuam sendo por domínio (`event.build.diagnostic`,
`event.quality.diagnostic`, `event.project.changed

event.lsp.diagnostics`), mas os itens de
diagnóstico usam campos comuns:

```text
Diagnostic {
  id?,
  source: "build|quality|lsp|toolchain",
  severity: "error|warning|note",
  category?,
  message,
  file?,
  line?,        // 1-based, início do range
  column?,      // 1-based, início do range
  endLine?,     // 1-based, fim do range (T6, para o sublinhado)
  endColumn?,   // 1-based, fim do range
  code?,        // código/regra: "E0425", "unused_variables", ...
  jobId?,
  command?,
  target?,
  logRef?
}
```

Estado atual: build usa `source: "build"` / `category: "compiler"`;
quality usa `source: "quality"` / `category: "lint"`; LSP usa
`source: "lsp"` / `category: "lsp"`. `id`, `command`, `target` e `logRef` já
existem no tipo de protocolo, mas ainda só aparecem quando um produtor tiver
dado real para preencher.

`endLine`/`endColumn`/`code` entraram no protocolo `0.35.0` (fatia T6):
`event.lsp.diagnostics` passa a mandar o range completo (o core deriva de
`textDocument/publishDiagnostics`; `end` ausente cai de volta ao início) e o
`code` do servidor (string ou número → sempre string). A UI usa o range para
o sublinhado ondulado no editor e a marca na gutter, e o `code` no detalhe da
aba Problemas. Build/quality ainda não preenchem range (sublinhado deles fica
para o Problems 2.0).

### Resultado de `tools.detect` / `tools.status`

Implementado no protocolo `0.2.0`. `tools.detect` sempre executa a detecção e
atualiza o registro interno; `tools.status` responde com o último resultado
conhecido (detectando na primeira chamada).

Inventário atual: cargo, rustc, rustup, rust-analyzer, cmake, ninja, git,
clangd, clang, clang++ (`id: clangxx`), gcc, g++ (`id: gxx`), gdb, lldb,
ripgrep (`rg`) e fd/fdfind.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "tools": [
      {
        "id": "cargo",
        "displayName": "Cargo",
        "status": "detected",
        "path": "/home/user/.cargo/bin/cargo",
        "version": "cargo 1.96.1"
      },
      {
        "id": "clangd",
        "displayName": "clangd",
        "status": "missing",
        "suggestedInstall": "sudo pacman -S clang",
        "message": "clangd nao foi encontrado no PATH."
      }
    ]
  }
}
```

Estados possíveis de ferramenta (`DocsPublic/07-tooling-lifecycle.md`):
`notConfigured`, `missing`, `detected`, `ready`, `running`, `failed`,
`disabled`. A detecção usa `missing`, `detected` e `failed`; os demais são
reservados para o gerenciamento de processos.

### Scan de ambiente (`environment.scan` — job assíncrono)

Implementado no protocolo `0.20.0`. Roda a mesma detecção de ferramentas de
`tools.detect`, mas como job assíncrono para o fluxo de First Run / Toolchain
Settings. Responde na hora com `{ "jobId" }`, não requer workspace aberto e,
ao concluir, atualiza o mesmo registry consultado por `tools.status`.

```text
event.environment.started   { "jobId", "tools" }
event.environment.tool      { "jobId", "tool": ToolInfo }
event.environment.finished  { "jobId", "success", "total", "detected", "missing", "failed", "tools": [ToolInfo] }
```

Além destes, emite `event.job.created/progress/output/finished`. A UI deve usar
`event.environment.*` para preencher telas de ambiente/toolchain e
`event.job.*` para status bar/lista de jobs. O core nunca instala ferramentas;
`suggestedInstall` continua sendo apenas uma sugestão para ação explícita do
usuário.

### Workspace (`workspace.browse` / `workspace.createFolder` / `workspace.createProject` / `workspace.open` / `workspace.recent.*`)

`workspace.open` foi implementado no protocolo `0.3.0`. `workspace.browse`
foi adicionado no protocolo `0.5.0` para o seletor proprio de workspace da UI.
`workspace.createFolder` e `workspace.createProject` foram adicionados no
protocolo `0.9.0` para permitir o fluxo JetBrains-like de criar pasta/projeto
sem sair da IDE.

`workspace.open` recebe
`{ "path": "/dir" }`, canonicaliza o caminho, identifica o tipo de projeto por
marcadores (precedência: `Cargo.toml` > `CMakeLists.txt` > `pom.xml` >
Gradle > Python) e persiste `.kinein/workspace.json`
(`schemas/workspace.schema.json`).

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "name": "meu-projeto",
    "root": "/home/user/dev/meu-projeto",
    "kind": "rustCargo",
    "markers": ["Cargo.toml", "CMakeLists.txt"],
    "capabilities": { "buildSystems": ["cargo", "cmake"] }
  }
}
```

Desde o protocolo `0.55.0`, `kind` continua sendo a classificação primária
compatível pela precedência de marcadores, enquanto `capabilities.buildSystems`
contém todos os sistemas reconhecidos na mesma varredura. Assim, um repositório
híbrido pode ser `rustCargo` e oferecer Cargo+CMake simultaneamente. O metadata
persistido usa `schemas/workspace.schema.json` 0.2.0; a UI consome o snapshot e
não repete detecção por arquivo.

**`kind: make` / `buildSystems: [make]` (`0.115.0`, P0):** um `Makefile` ou
`GNUmakefile` na raiz, sem marcador de precedência maior (um Makefile ao lado
de um `CMakeLists.txt` continua `cmake`, e `make` aparece em `buildSystems`).
`build.run` num `make` roda `bear -- <make>` na raiz quando o `bear` está
detectado (Bear 3, README: `bear -- <your-build-command>`; a
`compile_commands.json` sai na raiz e o `cdb` a vê como `"."`), senão `make` a
seco com uma linha de `event.build.output` dizendo que sem `bear` não há CDB
para o clangd — nunca um make "diferente" para fingir CDB. Sem alvo de run,
teste ou debug automático (o `Makefile` não os declara).

**`kind: platformIo` / `buildSystems: [platformIo]` (`0.117.0`, bloco E):**
um `platformio.ini` na raiz — atrás do `CMakeLists.txt` (um projeto PlatformIO
com `framework = espidf` também é uma árvore CMake; o MOTOR de build ainda
escolhe o `pio`, pelo framework), na frente de um `Makefile`. `build.run`
roda `pio run`; sem alvo de run, teste ou debug automático.

`workspace.browse` recebe `{ "path": "/dir" }`, canonicaliza o diretorio e
retorna apenas subdiretorios para a UI navegar sem depender de dialogo nativo do
desktop. A UI continua proibida de listar o filesystem diretamente.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "path": "/home/user",
    "parent": "/home",
    "entries": [
      {
        "name": "dev",
        "path": "/home/user/dev"
      }
    ]
  }
}
```

`workspace.createFolder` recebe `{ "parent": "/dir", "name": "modulo" }`.
O core canonicaliza `parent`, valida que `name` é apenas um segmento de caminho
e cria o diretório filho. A resposta é `{ "path": "/dir/modulo" }`.

`workspace.createProject` recebe
`{ "parent": "/dir", "name": "demo", "template": "empty|cppCmake|rustCargo|python" }`.
O core cria o diretório do projeto, aplica o template e abre o projeto como
workspace, retornando o mesmo payload de `workspace.open`.

- `empty`: cria diretório vazio e persiste `.kinein/workspace.json`.
- `cppCmake`: cria projeto C++23/CMake strict e target-based com
  `CMakeLists.txt`, presets Ninja Debug/Release, `src/main.cpp`, diretórios
  `include/` e `tests/`, `.gitignore` e `README.md`.
- `rustCargo`: usa `cargo new --bin --vcs none`; se `cargo` não existir,
  retorna `TOOL_NOT_FOUND`.
- `python` (`0.105.0`, 2026-09-13): geração interna, sem ferramenta —
  `pyproject.toml` como a PEP 621 escreve (`requires-python = ">=3.12"`,
  `[project.optional-dependencies] dev = ["pytest"]`, `[tool.ruff]`,
  `[tool.pytest.ini_options]` com `testpaths` e `pythonpath = ["."]`),
  `main.py` (o ponto de entrada que o botão Executar procura primeiro), o
  pacote `<nome_com_underscores>/__init__.py` **na raiz** — layout plano, para
  `python main.py` e `python -m pytest` acharem o pacote sem `pip install -e .`
  antes do primeiro clique —, `tests/test_main.py`, `.gitignore` com o `.venv`
  e `README.md`. O workspace nasce como `python`; o ambiente é o clique da
  faixa de saúde. Medido em 2026-09-13: o projeto gerado roda, passa no pytest
  e no `ruff check`/`format --check` sem editar nada.

`workspace.status` responde `{ "workspace": <objeto acima> | null }`.
`workspace.close` responde `{ "status": "ok", "closed": <root | null> }`.

**Sessão por workspace** (protocolo `0.24.0`, fatia M1.5 de `DocsPrivate/diario/18`): a
resposta de `workspace.open` ganha o campo opcional
`session: { openFiles: ["/abs/..."], activeFile? }`, presente apenas quando
`.kinein/session.json` existe, tem `schemaVersion` conhecida (1) e ao menos
um arquivo ainda válido — arquivos apagados/fora do root são filtrados no
carregamento e schema desconhecido é ignorado sem erro. A escrita é feita por
`workspace.saveSession { openFiles: ["/abs/..."], activeFile? }` →
`{ files: N }` (N = entradas efetivamente salvas; caminhos inválidos são
pulados, nunca falham a sessão inteira). Em disco os caminhos são RELATIVOS
ao root (mover a pasta do projeto preserva a sessão); no contrato IPC são
sempre absolutos canônicos. A UI salva com debounce (~1.2s) a cada mudança de
abas/aba ativa e restaura pedindo `fs.read` na ordem da sessão, com a aba
ativa por último.
Caminho inexistente ou sem `path` nos params retorna `INVALID_PARAMS`; falha de
IO ao persistir, listar ou criar diretorios retorna `INTERNAL_ERROR`.

**Workspaces recentes globais** (protocolo `0.53.0`, fatia A1 de `DocsPrivate/diario/18`):
somente `workspace.open` e `workspace.createProject` concluídos com sucesso
registram a raiz canônica. O core persiste até 12 entradas em
`$XDG_CONFIG_HOME/kinein-vectis/recent-workspaces.json` (ou
`~/.config/kinein-vectis/`), conforme
`schemas/recent-workspaces.schema.json`; a UI nunca consulta o filesystem.

```text
workspace.recent.list {} → { workspaces: [RecentWorkspace] }
workspace.recent.pin { root, pinned } → mesmo snapshot completo
workspace.recent.remove { root } → mesmo snapshot completo
workspace.recent.clear {} → { workspaces: [] }

RecentWorkspace {
  name: string,
  root: string,
  lastOpenedAt: inteiro (Unix epoch em milissegundos),
  pinned: bool,
  available: bool
}
```

- entradas fixadas vêm primeiro; cada grupo é ordenado por
  `lastOpenedAt` decrescente e uma raiz canônica nunca é duplicada;
- `available` é recalculado pelo core em cada snapshot. Uma pasta removida
  continua listada, desabilitada e removível, sem ser aberta pela UI;
- abrir uma entrada reutiliza `workspace.open` e, portanto, restaura a sessão
  por workspace já existente;
- o formato legado `schemaVersion: 0` sem `pinned` é aceito e promovido na
  próxima escrita. JSON inválido ou schema futuro é tratado como lista vazia;
- pin/remover exigem uma raiz absoluta já registrada e retornam
  `INVALID_PARAMS` caso contrário. Erro de escrita retorna `INTERNAL_ERROR`;
  uma falha ao atualizar o histórico não desfaz uma abertura válida;
- o arquivo guarda apenas nome, raiz, último acesso e fixação: nunca conteúdo,
  credenciais ou contexto de IA.

### Arquivos (`fs.list` / `fs.read` / `fs.createFile` / `fs.createDirectory` / `fs.write` / `fs.rename` / `fs.delete` / `fs.replace`)

Implementado no protocolo `0.4.0`. Todos exigem workspace aberto
(`INVALID_REQUEST` caso contrário) e todo caminho é canonicalizado e
confinado à raiz do workspace (`INVALID_PARAMS` se escapar).

- `fs.list { path }` → `{ path, entries: [{ name, kind: file|directory|other, size? }] }`,
  ordenado diretórios primeiro, depois nome case-insensitive.
- `fs.read { path }` → `{ path, content }`. Limites: arquivo regular, até
  1 MiB, UTF-8 válido (senão `INVALID_PARAMS` com mensagem humana).
- `fs.createFile { path, content? }` → `{ path, bytesWritten }`. Criado no
  protocolo `0.15.0`; o diretório pai precisa existir dentro do workspace e a
  operação falha se o arquivo já existir.
- `fs.createDirectory { path }` → `{ path }`. Criado no protocolo `0.15.0`;
  o diretório pai precisa existir dentro do workspace e a operação falha se o
  caminho já existir.
- `fs.write { path, content, expectedContent }` → `{ path, bytesWritten }`.
  Desde o protocolo `0.45.0`, sobrescreve apenas arquivos existentes cujo
  conteúdo atual ainda seja idêntico a `expectedContent` (o último snapshot
  lido pela UI). Divergência retorna `FILE_CHANGED` com `details.path` e não
  toca o disco. A substituição aceita continua atômica (temp irmão + `fsync` +
  `rename`).
- `fs.rename { from, to }` → `{ from, to }`. Criado no protocolo `0.19.0`;
  renomeia ou move um arquivo ou diretório dentro do workspace. `from` precisa
  existir; `to` não pode já existir e seu diretório pai precisa existir dentro
  do workspace. A raiz do workspace não pode ser renomeada (`INVALID_PARAMS`).
- `fs.delete { path }` → `{ path }`. Criado no protocolo `0.19.0`; remove um
  arquivo ou diretório (recursivo para diretórios) dentro do workspace. A raiz
  do workspace não pode ser removida (`INVALID_PARAMS`). **Desde 2026-09-02**,
  se o arquivo estava aberto num language server, o core manda
  `textDocument/didClose`: documento apagado que continua aberto deixa
  diagnóstico de um arquivo que não existe mais na aba Problemas. A forma da
  mensagem IPC não mudou.

**Mudanças externas (protocolo `0.45.0`, T2):** ao abrir o workspace, o core
inicia `notify` com backend nativo (`inotify` no Linux) e fallback por polling.
O registro é lazy e não recursivo: raiz, diretórios expandidos por `fs.list` e
diretórios de arquivos abertos por `fs.read`. Build/caches e temporários de
save atômico são ignorados. Eventos brutos são deduplicados por uma janela de
180 ms:

```text
event.fs.changed {
  changes: [{ path: "/abs/file", kind: "created|modified|deleted" }]
}
event.fs.watchError { message }
```

A UI recarrega automaticamente uma aba limpa; se houver edição local, preserva
o buffer e exige a escolha explícita entre recarregar o disco ou manter o
local. `event.fs.watchError` é visível, mas a proteção compare-before-save
continua ativa mesmo sem watcher.

### Formatação de buffer (`format.text` / `format.capabilities`)

Implementado no protocolo `0.21.0` (fatia M1.1 de
`DocsPrivate/diario/18-daily-driver-plan.md`). Requer workspace aberto. Formata o conteúdo
do editor com a ferramenta do projeto via stdin/stdout, sem tocar o disco —
salvar continua sendo decisão do usuário. O formatter roda com cwd na raiz do
workspace, então `rustfmt.toml`/`.clang-format` do projeto valem.

- `format.text { path, text }` →
  `{ path, text, changed, formatter }`.
- `path` precisa existir e estar confinado ao workspace (`INVALID_PARAMS` se
  escapar); é ecoado canonicalizado na resposta para a UI descartar respostas
  de uma aba que já mudou.
- Formatter por extensão: `.rs` → `rustfmt` (`--emit stdout`; acrescenta
  `--edition 2021` apenas quando o root não tem `rustfmt.toml`/
  `.rustfmt.toml`); `.c/.cc/.cpp/.cxx/.h/.hh/.hpp/.hxx` → `clang-format`
  (`--assume-filename=<path> --style=file --fallback-style=LLVM`);
  `.py/.pyi` → `ruff` (`0.99.0`, 2026-09-13: `ruff format --stdin-filename
  <path>` — o nome do arquivo decide qual `ruff.toml`/`pyproject [tool.ruff]`
  vale; com `--stdin-filename` e sem caminhos o ruff lê stdin, medido no
  0.16.4). Extensão sem formatter → `INVALID_PARAMS`.
- `changed: false` quando a saída é idêntica ao texto enviado.
- **O binário é o DETECTADO** (`0.99.0`): o handler pergunta ao detector de
  ferramentas — que procura no `PATH` e em `~/.local/bin`, onde pipx/uv põem o
  ruff e o `PATH` do processo da IDE pode não alcançar — e só cai no nome nu
  quando o detector não acha. Binário ausente → `TOOL_NOT_FOUND` (com
  `data.tool`); formatter com exit ≠ 0 → `INTERNAL_ERROR` com o stderr na
  mensagem.
- `format.capabilities {}` → `{ formatters: [ { id, extensions[] } ] }`
  (`0.61.0`). **Não** requer workspace: é o mapa estático de extensões, derivado
  da mesma constante que `formatter_for_path` usa para decidir — o que a UI
  recebe é, por construção, o que o `format.text` vai aceitar.

  **Invariante de camada.** Quem decide o que é formatável é o core; a UI
  consome. Até `0.60.0` o `EditorController.qml` mantinha duas listas escritas à
  mão (`formattableLanguage` por linguagem, `formattablePath` por extensão) que
  não concordavam entre si nem com o core, e adicionar linguagem exigia editar
  QML. Duas fontes para a mesma verdade divergem por construção.

  Capacidade e disponibilidade são perguntas **distintas**: `format.capabilities`
  responde "existe formatter registrado para esta extensão"; se o binário está no
  `PATH` só se descobre ao rodar `format.text` (`TOOL_NOT_FOUND`). O catálogo não
  muda com o workspace, então a UI pode pedi-lo uma vez por conexão.
- Operação síncrona por ser curta (um buffer); "formatar workspace inteiro"
  viraria job, e fica fora deste contrato.

### Busca de arquivos (`fs.findFiles`)

Implementado no protocolo `0.16.0`. Requer workspace aberto. O core usa
`fd` para busca por nome de arquivo, respeitando ignores do projeto e evitando
um indexador próprio no MVP.

- `fs.findFiles { query }` →
  `{ matches: [{ path, name }], truncated }`.
- `query` não pode ser vazio (`INVALID_PARAMS`).
- `path` é relativo à raiz do workspace; `name` é o nome do arquivo.
- A busca usa `fd --type f --fixed-strings --hidden --color never`, com
  exclusões explícitas para `.git`, `.kinein`, `.idea`, `.cache`, `target`,
  `build` e `node_modules`.
- No máximo 100 arquivos são retornados; `truncated: true` indica que o limite
  cortou resultados.
- O core aceita o binário `fd` e também `fdfind` (nome usado por algumas
  distribuições). Se nenhum existir no `PATH`, retorna `TOOL_NOT_FOUND`.

### Busca no workspace (`fs.search`)

Implementado no protocolo `0.11.0` (Find in Files). Requer workspace aberto.

- `fs.search { query, caseSensitive? }` →
  `{ matches: [{ path, line, column, preview }], truncated }`.
- `query` é texto literal (não regex) e não pode ser vazio
  (`INVALID_PARAMS`). `caseSensitive` é opcional e por padrão `false`
  (comparação case-insensitive apenas ASCII, para manter offsets exatos).
- `path` é relativo à raiz do workspace; `line`/`column` são 1-based
  (coluna em caracteres) para consumo direto do editor.
- A caminhada é determinística (profundidade, nome case-insensitive),
  ignora silenciosamente symlinks, arquivos não UTF-8 ou maiores que 1 MiB e
  os diretórios `.git`, `.kinein`, `.idea`, `.cache`, `target`, `build` e
  `node_modules`.
- No máximo 500 matches no total; `truncated: true` indica que o limite cortou
  resultados. `preview` é a linha com trim, limitada a 200 caracteres.
- **`query` pode conter `\n`** (desde 2026-09-02, protocolo `0.65.0`). A busca
  varre o CONTEÚDO do arquivo, não uma linha de cada vez: `line`/`column`
  apontam para o início do match e o `preview` mostra o trecho inteiro com as
  quebras internas trocadas pela marca ` ⏎ `. Match que atravessa linhas continua
  sendo UM match — é isso que faz o contador do preview bater com o número de
  reescritas.
- **Matches não se sobrepõem**: a varredura retoma no fim do match anterior.
  `aa` em `aaa` são 1 match, não 2, e `fs.replace` reescreve exatamente esse 1.

`fs.replace { query, replacement, caseSensitive? }` foi adicionado no
protocolo `0.48.0` (T4). Ele executa substituição literal confirmada no
projeto, com a mesma política de confinamento/ignores e limites da busca:

- a UI exige confirmação e recusa iniciar enquanto houver editor sujo;
- o core lê e calcula todos os novos conteúdos antes de escrever;
- a gravação multi-arquivo usa a primitiva transacional comum, valida os
  snapshots novamente, escreve de forma atômica e faz rollback se qualquer
  arquivo falhar;
- responde `{ files: ["/abs/..."], replacements: N }`; arquivos binários,
  não UTF-8, grandes demais ou em diretórios ignorados não entram;
- `query` vazia retorna `INVALID_PARAMS`; zero ocorrências é sucesso com
  listas/contador vazios.
- **`query` e `replacement` aceitam `\n`** (desde 2026-09-02, protocolo
  `0.65.0`). De 2026-08-29 até essa data o core RECUSAVA (`INVALID_PARAMS`), e a
  recusa estava certa para o que existia então: a busca casava **linha a linha**,
  então uma query multi-linha era invisível para o preview e ativa para a
  escrita — o usuário via "0 resultados" e arquivos eram reescritos mesmo assim.
  O que mudou não foi a guarda, foi a busca: ela passou a varrer o conteúdo
  inteiro e a devolver preview do trecho completo. **A guarda saiu porque a razão
  dela saiu** — a invariante que ela protegia ("o preview conta o que a escrita
  vai fazer") agora vale sozinha, e está travada pelos testes
  `multiline_search_previews_exactly_what_replace_will_rewrite` e
  `a_multiline_replacement_is_found_by_the_next_search`.
- **A sintaxe `\n` do painel é da UI, não do protocolo.** O campo de busca é um
  `TextInput` de uma linha, então `SearchController.expandLineBreaks()` traduz a
  sequência de dois caracteres `\n` digitada pelo usuário em quebra de verdade
  antes de chamar o core (e `\\n` devolve o literal barra-ene). Essa tradução
  **não pode descer para o core**: `query` é texto LITERAL, e um core que
  interpretasse escapes tornaria impossível procurar por um `\n` de verdade
  dentro do código.
- **`fs.search` reporta TODAS as ocorrências de cada linha** (desde 2026-08-29).
  Antes parava na primeira: "Alpha alpha" aparecia como 1 resultado e virava 2
  substituições. O preview de uma operação destrutiva tem de contar o que ela
  vai fazer.
- **paridade com `fs.search` (garantida desde 2026-08-29):** os dois percorrem o
  mesmo walk (`fsops::walk`), então `fs.replace` nunca toca arquivo que
  `fs.search` não mostrou. Consequência da unificação: um subdiretório ilegível é
  **pulado** (como sempre foi na busca), e não mais aborta a substituição inteira
  — só a raiz ilegível é erro. Travado pelo teste
  `replace_touches_exactly_the_files_search_reports`.

### Execução (`run.start` / `run.script` / `run.stop`)

Implementado no protocolo `0.12.0`; **desde `0.125.0` (2026-09-18) a
execução é uma SESSÃO DE TERMINAL** — decisão do autor no primeiro teste
da IDE polida: "já temos o terminal integrado, não precisamos de mais nada
para executar". `run.start`/`run.script` continuam a resolver O QUE rodar
(a lógica abaixo é a mesma) e abrem o comando num PTY na raiz do
workspace (`sh -lc <command>` para o `start`; argv direto para o
`script`), como o `container.open` e o `serial.monitor` já faziam. A
resposta ganha `terminalId`; a saída chega por `event.terminal.render`
dessa sessão e o fim por `event.terminal.closed { id, exitCode }` — stdin
é `terminal.input`, cores e programas de tela cheia funcionam. Saíram
`run.stdin` e os três `event.run.*` (não há mais processo por pipes).
Requer workspace aberto.

- `run.capabilities {}` → `{ runnable: [ext], debuggable: [ext] }`
  (`0.107.0`). **Não** requer workspace: é o mapa estático do que
  `run.script` aceita (`sh`, `bash`, `zsh`, `py`) e do que `debug.start
  { program }` roteia para um adaptador de linguagem sem o kit (`py`),
  derivado da mesma tabela que `script_interpreter` usa para decidir. A UI
  pede uma vez por conexão; antes do catálogo chegar, nada é executável —
  uma lista escrita à mão em QML divergiu da do core por construção (o
  defeito que o `format.capabilities` corrigiu em 0.61.0 voltou a aparecer
  em dois arquivos da árvore, e este método o fecha).
- `run.start { command?, device? }` → `{ command, terminalId }`. Sem `command`, o core deriva o
  padrão do tipo de projeto: `cargo run` para Rust/Cargo; para CMake, o
  único executável em `.kinein/build` (erro claro se não houver ou houver
  mais de um); **para Python (`0.100.0`, 2026-09-13)**, o ponto de entrada
  por EVIDÊNCIA — `main.py`, `app.py` ou `__main__.py` na raiz (nesta
  ordem); senão UM pacote com `__main__.py` na raiz, depois em `src/`
  (`python -m <pacote>`; dois candidatos = ambiguidade = a IDE não escolhe);
  senão um script de `[project.scripts]` JÁ instalado em `.venv/bin/<nome>`
  — lançado pelo interpretador do projeto ou por `uv run python` (só quando
  o projeto tem `uv.lock` E o uv foi detectado: o uv sincroniza o ambiente
  com o lock antes de rodar, que é o que quem adotou o uv quis). Sem
  interpretador, ou sem entrada, `INVALID_REQUEST` dizendo o que procurou e
  o que fazer (criar o `.venv` pela faixa de saúde; "Executar" num `.py`).
  Outros tipos ainda não têm padrão (`INVALID_REQUEST` com mensagem
  orientando digitar o comando). **`device?` (`0.110.0`)** é a porta serial
  que a tela escolheu para o lançador padrão de um projeto MicroPython
  (`mpremote connect <device> run 'main.py'`); é parâmetro do LANÇADOR
  PADRÃO e por isso é exclusivo com `command` (`INVALID_PARAMS` quando vêm
  os dois — um comando explícito roda como foi escrito e não tem onde
  receber a porta). Uma configuração de execução ATIVA é um comando salvo
  pelo autor: vence o lançador padrão e tampouco recebe a porta; o `command`
  ecoado diz o que rodou. Fora de MicroPython o campo não tem efeito. Vazio
  ou só espaço é `INVALID_PARAMS` (ver `run.script`).
- `run.script { path, device? }` → `{ command, terminalId }` (protocolo `0.55.0`).
  Aceita arquivo regular `.sh`, `.bash`, `.zsh` ou (`0.100.0`) `.py` dentro
  do workspace. O core canonicaliza/confina o caminho e chama `bash`/`zsh` com
  argv explícito (`--`, caminho), sem interpolação por `sh -c`; nomes com
  espaços ou aspas são dados, não sintaxe. Um `.py` roda com o **Python do
  projeto** (`python/env.rs`, o mesmo do `python.status`, do `index.context`
  e do basedpyright — sem medir a versão, que é custo do status) ou com
  `uv run python`, o arquivo como único argumento, cwd no root; `command`
  ecoa `.venv/bin/python 'tools/gera.py'` (interpretador relativo ao root
  quando mora nele) ou `uv run python '…'`. Sem interpretador nenhum,
  `INVALID_REQUEST` orientando a criar o ambiente. **Projeto MicroPython
  (`0.102.0`)**: o `.py` roda **na placa** — `mpremote [connect <device>]
  run <arquivo>` (mpremote 1.29.0, `mpremote run --help`: `run [--follow]
  path`; sem `connect` o mpremote usa a primeira porta serial que acha);
  `device?` é a porta que a tela escolheu (campo ausente ≠ vazio: desde
  `0.110.0` um `device` presente e vazio, só espaço ou com caractere de
  controle é `INVALID_PARAMS` dizendo para omitir o campo — antes virava
  `mpremote connect '' run`, que falhava longe de quem errou). Sem
  mpremote detectado, `INVALID_REQUEST` dizendo `pipx install mpremote` —
  nunca o Python do desktop, que não tem os pinos de `import machine`. O
  mesmo vale para o `run.start` sem comando (`main.py` na placa, mesmo num
  workspace sem `pyproject.toml`); um pacote com `__main__.py` não roda com
  `-m` na placa e o erro o diz. Extensão inválida é `INVALID_PARAMS`.
- `run.stop {}` → `{ status: "ok" }`. Fecha a sessão da ÚLTIMA execução
  aberta; sem uma, `INVALID_REQUEST`. (Fechar a aba pela UI é
  `terminal.close`, como qualquer outra.)
- Fechar o workspace fecha todas as sessões, a da execução inclusa.

A UI (`RuntimeController`) dá à aba o nome do comando (`▶ cargo run`) e,
quando a sessão fecha, **mantém a aba** com o desfecho no nome (`✓` ou
`✗ <código>`) para o autor ler a saída; fechá-la depois é só local.

### Terminal (`terminal.open` / `terminal.input` / `terminal.resize` / `terminal.scroll` / `terminal.mouse` / `terminal.clearScrollback` / `terminal.selectAll` / `terminal.copySelection` / `terminal.close`)

Terminal profissional (reescrito no protocolo `0.41.0`, fatia D2 de
`DocsPublic/roadmaps/24`). Requer workspace aberto. **PTY real** via `portable-pty` (do
wezterm) rodando o `$SHELL` interativo em `TERM=xterm-256color` na raiz.
Um **emulador VT** (`alacritty_terminal`, ADR-0004 — o mesmo motor do Alacritty
e do Zed) no core mantém o GRID (células com cor/atributos), cursor, modos, tela
alternada e scrollback; a UI recebe o grid PRONTO e só desenha — sem interpretar
ANSI. Suporta cores, prompts com `\r`, barra de progresso do cargo e TUIs. Desde
o protocolo `0.44.0` (D2.3), o manager mantém até 12 sessões simultâneas. Cada
`open` cria um id monotônico (`t1`, `t2`, …); todos os comandos e eventos
seguintes carregam esse id.

**Invariante de camada.** O terminal é dono da semântica do input. A UI reporta
gestos crus (tecla, roda, clique) e desenha o grid que recebe; ela **não** decide
o que um gesto significa, porque isso depende do modo VT que só o emulador
conhece. Nenhuma política por programa existe no core: uma CLI de IA recebe o
mesmo tratamento de um `ls`.

- `terminal.open {}` → `{ id, shell }`. Cada chamada cria uma sessão nova;
  erro `INVALID_REQUEST` ao atingir o limite de 12 sessões.
- `terminal.input { id, data }` → `{ status: "ok" }`. Encaminha `data` **cru**
  ao PTY. A UI manda CADA tecla (char-a-char), incl. control chars
  (Enter=`\r`, Backspace=`\x7f`, setas=`\x1b[A..D`, Ctrl+letra, …) — não
  linha+Enter.
- `terminal.resize { id, cols, rows }` → `{ status: "ok" }`. Reflui o PTY e o
  grid. A UI calcula cols/rows do tamanho do painel ÷ métrica da fonte mono.
- `terminal.scroll { id, offset }` → `{ status: "ok" }` (D2.2, `0.42.0`).
  Rolagem **explícita** do histórico: `offset` linhas acima do fundo (0 = ao
  vivo; o core **clampa** ao tamanho real do scrollback). É o que a barra de
  rolagem pede, e o snap-to-bottom ao digitar. **Um gesto de roda não é isto** —
  vai por `terminal.mouse`, porque só o core sabe se a aplicação capturou o
  mouse. O clipboard pertence à UI (singleton `Clipboard`); desde `0.131.0`,
  a cópia da seleção completa lê o texto do emulador por RPC sob demanda.
  **Nunca confie no offset da UI:** até `0.43.0` um offset maior que o
  histórico **derrubava o core** (bug de overflow do `vt100` 0.15 —
  corrigido no 0.16, que satura a subtração; o `alacritty_terminal` clampa por
  conta própria). A verdade do offset volta no render (`scrollback`), não na
  resposta.
- `terminal.clearScrollback { id }` → `{ status: "ok" }` (`0.130.0`). Apaga
  o histórico mantido pelo emulador **só da sessão indicada**, retorna a tela
  ao fundo e emite um render novo (`scrollback = 0`, `scrollbackMax = 0`). Não
  injeta comandos no shell e não apaga as linhas que ainda estão no grid
  visível. “Limpar tela” é outra ação: a UI envia `Ctrl+L` (`\x0c`) por
  `terminal.input`, preservando a semântica do programa em primeiro plano.
- `terminal.selectAll { id, selectionId }` → `{ id, selectionId }`
  (`0.131.0`). Seleciona o buffer ativo retido, incluindo histórico e linhas
  utilizadas da tela, independente da rolagem. Não inclui tela inativa,
  outras sessões, texto descartado nem linhas não utilizadas abaixo do prompt.
  `selectionId` é uma identidade opaca do gesto, não vazia, até 128 bytes.
  Publica a identidade no render sob o lock do buffer; não lê clipboard.
- `terminal.copySelection { id, selectionId }` → `{ id, selectionId, text }`
  (`0.131.0`). Extrai a seleção nativa, preservando Unicode e distinguindo
  wrap visual de quebra real. `text: null` significa identidade obsoleta;
  string vazia significa seleção válida sem texto. Nova saída, resize,
  limpeza e troca de tela invalidam; scroll preserva. Sessão inexistente
  devolve `INVALID_REQUEST`. A UI descarta respostas de outro gesto/sessão
  e não registra o texto copiado no log.
- `terminal.mouse { id, col, row, event, modifiers? }` → `{ status: "ok" }`
  (R4, `0.60.0`). Um gesto de mouse na grade. `col`/`row` são a célula sob o
  ponteiro, **0-based** (mesma origem do cursor no render); o core converte para
  1-based ao montar o relatório, como o xterm especifica. `modifiers` é
  `{ shift?, alt?, ctrl? }`, todos `false` por omissão.

  `event` é marcado por `kind`:

  ```text
  { "kind": "wheel",   "lines": i16 }          implementado
  { "kind": "press",   "button": "left"|"middle"|"right" }   R5 — recusado
  { "kind": "release", "button": ... }                        R5 — recusado
  { "kind": "motion",  "button": ...|null }                   R5 — recusado
  ```

  O contrato de R5 está fixado para a superfície não mudar de novo; o core
  recusa essas variantes com `INVALID_REQUEST` em vez de fingir que funcionam.

  **A decisão é do core**, lendo o modo VT (ordem vinda do `scroll_wheel` do Zed,
  referência MODE-D):

  ```text
  1. shift ligado                      → rola o histórico local
                                         (válvula de escape do xterm)
  2. aplicação capturou o mouse        → relatório à aplicação
     (MOUSE_REPORT_CLICK/DRAG/MOTION)    SGR se ?1006, senão formato legado
  3. ALT_SCREEN + ALTERNATE_SCROLL     → cursor keys (ESC O A / ESC O B)
  4. caso contrário                    → rola o histórico local
  ```

  A ordem entre 2 e 3 é carga estrutural: `ALTERNATE_SCROLL` nasce **ligado**
  (default do emulador, como no xterm), então uma TUI que captura o mouse
  satisfaz os dois. Quem captura tem precedência — inverter faz a aplicação
  receber setas e não rolar.

  `lines` positivo = para cima (histórico mais antigo); negativo = para baixo;
  `0` é no-op. Um relatório é emitido por linha do gesto. No formato legado uma
  coordenada acima de 223 não é representável e o evento é **suprimido**, não
  truncado (um campo truncado viraria clique em outra célula).
- `terminal.close { id }` → `{ status: "ok" }`. Fecha só a sessão indicada;
  fechar/trocar o workspace ou encerrar o core fecha todas.

```text
event.terminal.render {           (throttle ~30fps; substitui event.terminal.data)
  "id": string,                  (0.44.0 — sessão dona deste grid)
  "selectionId": string,         (0.131.0 — vazio se não há seleção completa válida)
  "cols": u16, "rows": u16,
  "cursor": { "row": u16, "col": u16, "visible": bool,
              "shape": "block"|"underline"|"bar",
              "blinking": bool },       (0.57.0 — DECSCUSR da aplicação)
  "alternateScreen": bool,       (0.51.0 — TUI em tela alternativa)
  "applicationCursor": bool,     (0.51.0 — setas SS3 quando solicitado)
  "bracketedPaste": bool,        (0.51.0 — paste delimitado e seguro)
  "scrollback": usize,            (0.43.0 — offset ATUAL, já clampado: a verdade)
  "scrollbackMax": usize,         (0.43.0 — quanto histórico existe; 0 = nenhum)
  "lines": [ [ { "text": str, "cells": u16,
                 "fg"?: idx|"#rrggbb", "bg"?: idx|"#rrggbb",
                 "bold"?: bool, "italic"?: bool, "underline"?: bool,
                 "inverse"?: bool } ... ] ... ]
}
event.terminal.closed  { "id": string, "exitCode": int|null }
```

`fg`/`bg`: índice 0–255 (paleta) ou `#rrggbb` (truecolor); ausência = cor
default do tema. Cada linha é uma lista de SPANS (runs de células de mesmo
estilo). Desde `0.56.0`, `cells` informa a largura autoritativa do span em
colunas VT; ela não deve ser inferida de `text.length` nem da largura em pixels
da fonte, pois glifos largos, combinantes e fallback tipográfico podem divergir.
O core coalesce runs e descarta o espaço final em estilo default.
Desde `0.57.0`, `shape` e `blinking` preservam o estado VT solicitado por
`DECSCUSR` (`CSI Ps SP q`). O core resolve reset/`DefaultUserShape` para a
preferência do terminal Kinein (`bar`) e sempre emite uma forma concreta; forma
explícita e piscagem pertencem ao programa no PTY. A UI não deve inferir a CLI
ativa nem aplicar geometria específica para Claude/Codex.

`scrollback`/`scrollbackMax` (`0.43.0`, B1/B2 de DocsPublic/roadmaps/24) existem porque a UI
**não tem como saber sozinha** se há histórico nem onde a view está: o core é
quem clampa. Sem eles não dá pra desenhar barra de rolagem honesta, e a UI
acabava pedindo offsets impossíveis. A UI trata `scrollback` como fonte da
verdade (reconcilia o estado local a cada render).

Desde `0.51.0`, o render também expõe os modos VT que alteram a tradução de
entrada. A UI respeita application cursor, envolve colagens com bracketed
paste quando a aplicação o pede e continua sem interpretar a interface do
programa. Resize de painel é coalescido antes de `terminal.resize`, evitando
reserializar um grid por pixel durante o arrasto.

### IA externa: sem domínio próprio (removido no `0.59.0`)

O domínio `aiBridge.*` **não existe mais**. Ele expunha `aiBridge.profiles` e
`aiBridge.terminal.open`, que abriam `claude`/`codex` por um caminho especial:
argumentos injetados pelo core (`--no-alt-screen`, `--ax-screen-reader`) e um
filtro que engolia `CSI 3 J` da própria aplicação.

Esse caminho era a IDE interferindo no terminal. Ele tratava um agente de CLI
como se fosse diferente de qualquer outro programa, o que produziu sintomas
atribuídos ao painel quando a causa estava no emulador raso (ver
`DocsPublic/decisoes-adr/ADR-0004-alacritty-terminal-emulator.md`).

O modelo agora é o de qualquer IDE profissional: **uma CLI de IA é um programa
como outro qualquer**. Abrir um terminal (`terminal.open`) e digitar `claude`
ou `codex` usa o mesmo PTY, o mesmo emulador e o mesmo contrato
`terminal.input/resize/scroll/close` de um `ls`. Não há allowlist de perfil,
argumento imposto, filtro de sequência nem preferência persistida.

Consequência de produto: rodar um agente dentro da Kinein passa a ser
indistinguível de rodá-lo fora dela — que era o requisito original. Uma
superfície visual de atalho (Assistente) pode voltar depois **como UI pura**,
abrindo uma sessão de terminal comum, sem regra de negócio própria no core.

### `build.size` — o tamanho do ELF (síncrono)

**Desde 2026-09-12 (pilar 0 do `roadmaps/42`)**, num projeto ESP-IDF a lista
de regiões ganha a partição `app` que o `flasher_args.json` aponta — usado =
tamanho da imagem, capacidade = a partição que **começa** naquele offset. O
`.ld` do IDF não declara a flash; a partição é a flash.

Implementado no protocolo `0.90.0`. **Síncrono**, ao contrário do `build.run`:
o `size` lê um arquivo e volta em milissegundos. `{ "program"? }` — ausente, o
core resolve o ELF como o `debug.start`. Roda `<prefix>size -A` (o prefixo vem
do compilador cross do kit: `arm-none-eabi-gcc` → `arm-none-eabi-size`; sem
cross, o `size` do sistema) e responde `SizeReport { toolAvailable, tool,
program, sections: [{ name, size, addr }], regions: [{ name, used, size }],
rawOutput }`. As `regions` saem do bloco `MEMORY` do único `.ld` do workspace,
com `used` = soma das seções **alocadas** cujo endereço cai na região —
`.comment`/`.ARM.attributes` (metadados do ELF, endereço 0) não contam. Sem
linker script legível, `regions` vem vazio e a UI mostra os totais por seção.

### Build (`build.run` — job assíncrono)

Implementado no protocolo `0.6.0`; migrado para **job assíncrono** (ver seção
Jobs e `DocsPublic/arquitetura/ARCHITECTURE.md` §7). Requer workspace aberto. O core valida de
forma síncrona e responde **na hora** com `{ "jobId": "job_N" }`; o build roda
em background (`cargo build --message-format=json` para Rust/Cargo;
`cmake -S/-B` + `cmake --build` em `.kinein/build` para CMake) e é **cancelável**
via `job.cancel` (mata o processo de build).

Desde `0.55.0`, aceita `{ "buildSystem"?: "cargo"|"cmake"|... }`. Em workspace
híbrido, a seleção explícita precisa existir em
`workspace.capabilities.buildSystems`; sem o campo, o `workspace.kind` primário
preserva o comportamento anterior. Sistema ausente retorna `INVALID_PARAMS`
com a lista disponível, antes de criar job.

Enquanto roda, emite os eventos ricos que a UI consome, agora com `jobId`:

```text
event.build.started     { "jobId", "command": "cargo build" }
event.build.output      { "jobId", "stream": "stdout|stderr", "line": "..." }
event.build.diagnostic  { "jobId", "source": "build", "category": "compiler", "severity": "error|warning|note", "message", "file"?, "line"?, "column"? }
event.build.finished    { "jobId", "success", "exitCode", "diagnostics" }   // ou { "jobId", "success": false, "error" }
```

**Os motores de framework (`0.117.0`, bloco E do `roadmaps/41`).** Antes do
tipo do projeto, o `build.run` pergunta ao `project.model` (`build/engine.rs`)
e, se o framework tem wrapper, é ele quem compila — nada de campo novo:

```text
PlatformIO   pio run                                     (platformio.ini; vence tudo)
ESP-IDF      bash -c '. "$1" >/dev/null || …; shift; exec idf.py "$@"' idf <ativação> build
             ativação = <esp-idf>/export.sh (IDF_PATH ou ~/esp/esp-idf) ou o mais novo
             ~/.espressif/tools/activate_idf_*.sh (EIM, v6) — as duas formas do Get Started
Zephyr       west build -d build [-b <placa>]            placa: CACHED_BOARD do build/CMakeCache.txt,
                                                          senão `west config build.board`
pico-sdk     cmake … -DPICO_SDK_PATH=<sdk>                (o CMake de sempre; o SDK de PICO_SDK_PATH
                                                          ou ~/pico/pico-sdk)
```

Diagnósticos: os três wrappers chamam gcc/clang por baixo e o parser
`arquivo:linha:coluna` casa. O título do job diz o motor (`Build (idf.py)`).
Framework reconhecido e ferramenta ausente (sem `pio`, sem `west`, sem
ativação do IDF) é `TOOL_NOT_FOUND` **antes** do job, com o passo que o
`project.model.sdks` já dava (e, no IDF, o `eim install`). O banner da
ativação vai para `/dev/null`; o erro dela, não. Os builds do `idf.py` e do
`west` ficam em `build/` (a pasta de cada um — é onde o `flasher_args.json` e
o `CMakeCache.txt` são lidos); o do pico-sdk continua em `.kinein/build`.

Além destes, o Job System emite `event.job.created` (ao iniciar),
`event.job.output` (fan-out da saida bruta) e `event.job.finished` (ao
encerrar) para a status bar / lista de jobs. O
resultado do build chega por `event.build.finished`, **não** mais na resposta.
Diagnósticos vêm do JSON do cargo (span primário) ou do formato
`arquivo:linha:coluna: nivel: mensagem` de compiladores/CMake e alimentam o
Problems. Validação síncrona antes de iniciar o job: tipos sem integração de
build retornam `INVALID_REQUEST`; sem workspace, `INVALID_REQUEST`. Falhas do
build (ferramenta ausente, erro de compilação) chegam por
`event.build.finished`/`event.job.finished`, não como erro da resposta.

### Qualidade / lint (`quality.run` — job assíncrono)

Implementado no protocolo `0.18.0`; migrado para **job assíncrono/cancelável**
como o `build.run`. Requer workspace aberto. Responde na hora com `{ "jobId" }`;
para Rust/Cargo roda `cargo clippy --all-targets --message-format=json` em
background, cujo JSON é idêntico ao do `cargo build`, então os lints viram
diagnósticos estruturados sem parser novo. Emite, com `jobId`,
`event.quality.started/output/diagnostic/finished` (mesmos formatos dos
`event.build.*`) + `event.job.*`; a saida bruta tambem faz fan-out para
`event.job.output`. Diagnosticos usam o mesmo payload de build, mas com
`source: "quality"` e `category: "lint"`. A UI adiciona os diagnósticos à aba Problemas
com origem `quality`. `job.cancel` mata o clippy. Validação síncrona: tudo que
não é Rust/Cargo **nem Python** retorna `INVALID_REQUEST` (CMake via clang-tidy
é o próximo passo). Falhas (`cargo` ausente etc.) chegam por
`event.quality.finished`. O parâmetro opcional `buildSystem` de `0.55.0` é
tipado pelo mesmo enum; as capacidades executáveis por `quality.run` são
`cargo`, `python` e, desde `0.119.0`, `cmake`/`make`.

**C/C++ (`0.119.0`, D6 do `roadmaps/41`, 2026-09-17).** Num `CMake` ou num
`Makefile` puro o `quality.run` roda o **clang-tidy do projeto inteiro pela
CDB** (`build/tidy.rs`): `run-clang-tidy -p <pasta da CDB> -quiet` quando o
script do LLVM está detectado (paraleliza e lê a `compile_commands.json`),
senão `clang-tidy -p <pasta> <arquivos>` com os arquivos vindos da própria
CDB (`file` de cada entrada, sem repetir, só os que existem — nunca um
glob). O `.clang-tidy` do projeto é lido pela ferramenta; a IDE não escolhe
checks. Cada `arquivo:linha:coluna: warning: mensagem [check]` vira
`event.quality.diagnostic` pelo parser gcc-like, com o nome do check na
mensagem. Sem CDB (`cdb::status`) o job falha dizendo "configure o CMake ou
compile o Makefile com o bear"; sem `clang-tidy`, nomeia a ferramenta. O
mesmo `.clang-tidy` vale no editor: o clangd sobe com `--clang-tidy`
(`toolchain/arguments.rs`), e os avisos entram nos diagnósticos do arquivo
aberto. O botão de análise da barra aparece também em C/C++ e Python.

**Python (`0.99.0`, 2026-09-13 — fatia 2 da cadeia do `roadmaps/41` bloco
B).** Workspace Python (`buildSystem: "python"`, ou o detectado): roda o
**ruff DETECTADO** (`~/.local/bin` do pipx/uv entra pelo detector) com
`ruff check --output-format concise --no-fix [--select …] .`, cwd no root —
os caminhos dos diagnósticos são relativos ao root e a configuração do
projeto (`ruff.toml`, `.ruff.toml`, `pyproject [tool.ruff*]`) é a que o ruff
acha a partir dali. O `--select` vem do **perfil de rigor** (`settings`)
apenas quando o projeto NÃO declara regras — `strict` = `E,F,W,I,UP,B,N`,
`balanced` = o default do ruff (sem flag), `relaxed` = `E9,F63,F7,F82` (só o
que quebra); projeto que declara vence sempre. Cada linha
`arquivo:linha:coluna: CODIGO [*] mensagem` vira `event.quality.diagnostic`
com `file`/`line`/`column`; severidade `error` para `E9xx` e `SyntaxError`,
`warning` para o resto; o `[*]` do ruff vira o sufixo `(corrigivel: ruff
check --fix)` na mensagem. Resumo (`Found N errors.`), `All checks passed!` e
avisos de configuração não viram diagnóstico. **Sem ruff detectado** o
`event.quality.finished` traz `success: false` e `error` que NOMEIA a
ferramenta e o passo oficial (`pipx install ruff`, o mesmo do painel de
instalação) — não o "tipo de projeto não suportado" de antes. `success` é
`false` quando o ruff achou problemas (exit ≠ 0). Exercitado no gate com o
ruff real (`F401` do `tools/gera.py` do projeto de exercitação).

### Testes (`test.run` / `test.discover` — jobs assíncronos)

**gtest e Catch2 dentro dos binários do ctest (`0.119.0`, D7 do
`roadmaps/41`, 2026-09-17).** Um `add_test` do `CMake` é UM binário com N
casos; o `ctest -N` lista o binário. O `test.discover` de um `CMake` pergunta
ao próprio binário: `ctest --test-dir <build> --show-only=json-v1` (ctest
4.2.3, medido: `tests[].name`, `tests[].command[]`, `WORKING_DIRECTORY`),
depois `<exe> --gtest_list_tests` (gtest: `Suite.` na coluna 0, `  Caso`
indentado; parametrizados `Suite/Inst.Caso/0`) ou `<exe> --list-tests
--verbosity quiet` (Catch2 v3: um nome por linha). Cada caso entra como
`TestCaseInfo { id: "<teste do ctest>::<caso>", name: "<caso>" }` depois das
linhas do ctest; um binário que não responde a nenhuma listagem continua
sendo só a linha do ctest. `test.run { testId: "unit_tests::Math.Adds" }`
roda o binário direto — `--gtest_filter=Math.Adds --gtest_color=no` (cada
`[       OK ]`/`[  FAILED  ]` vira `event.test.case`) ou `"<caso>" -r
compact` no Catch2 (o desfecho do caso é o exit code). Um `testId` sem `::`
é o ctest de sempre (`-R ^nome$`).

Implementado no protocolo `0.17.0`; migrado para **job assíncrono/cancelável**.
Requer workspace aberto. Responde na hora com `{ "jobId" }`; roda o runner do
tipo de projeto em background (`cargo test` para Rust/Cargo; `ctest --test-dir
.kinein/build --output-on-failure` para CMake; **`python -m pytest -v` para
Python** desde `0.100.0`, 2026-09-13) e transmite cada caso conforme sai da
saída do runner. Aceita `{ "filter"? }` (posicional do cargo; `-R` do ctest;
`-k` do pytest) e, desde `0.55.0`, `{ "buildSystem"? }` com a mesma validação
de capacidade do build. `job.cancel` mata o runner.

```text
event.test.started   { "jobId", "command": "cargo test" }
event.test.output    { "jobId", "stream": "stdout|stderr", "line": "..." }
event.test.case      { "jobId", "name": "modulo::caso", "status": "passed|failed|ignored" }
event.test.finished  { "jobId", "success", "exitCode", "passed", "failed", "ignored" }
event.test.finished  { "jobId", "success": false, "error": "…" }   (o runner nem correu)
```

Além destes, `event.job.created`/`event.job.output`/`event.job.finished`. Cada
`event.test.output` tambem gera `event.job.output { "jobId", "line" }` para o
historico generico do job. Os casos são extraídos das linhas
`test <nome> ... ok|FAILED|ignored` (libtest), `... Test #N: <nome> ...
Passed|***Failed` (ctest) e `arquivo::caso PASSED|FAILED|ERROR|SKIPPED|XFAIL|
XPASS [ nn%]` (pytest `-v`: o nome vem antes do estado e tem `::`; `PASSED`/
`XPASS` = passou, `FAILED`/`ERROR` = falhou, `SKIPPED`/`XFAIL` = ignorado; o
resumo curto — `FAILED arquivo::caso - assert …`, estado na frente — não é
caso, senão cada falha contaria duas vezes); a linha de resumo do libtest é
ignorada. Tipos sem integração retornam `INVALID_REQUEST` (síncrono, antes do
job); o resultado vem em `event.test.finished`, não na resposta.

**A árvore antes do run (`test.discover`, `0.106.0`, 2026-09-13).**
`test.discover { buildSystem? }` → `{ jobId }`; o job lista **sem rodar** e
termina em `event.test.discovered { jobId, runner, command, tests:
[TestCaseInfo { id, name, file? }], success, error? }`:

```text
pytest   python -m pytest --collect-only -q    tests/test_a.py::test_x[a b]  (medido, 9.1.1:
         um node id por linha; o resumo "N tests collected" e as linhas vazias
         não contam; só o stdout — o pytest escreve avisos no stderr; exit 5
         = "no tests ran" = lista VAZIA com sucesso, um projeto novo não tem testes)
cargo    cargo test -- --list                  tests::alpha: test  (`: benchmark`
         e o resumo não contam; COMPILA os testes — por isso é job)
ctest    ctest --test-dir .kinein/build -N     "  Test #N: Nome"  (medido, ctest 4.x)
```

O `id` é o que o **mesmo** runner aceita para rodar um só, e é o `name` que
`event.test.case` reporta quando ele roda — a UI pinta a linha da árvore
sem tabela de tradução. `test.run { testId }` (vence `filter`; vazio e
espaços não contam): pytest recebe o node id **posicional** (`pytest
tests/a.py::x` — `-k` casaria por substring), cargo `cargo test <nome> --
--exact` (o `--exact` é do libtest, depois do `--`), ctest `-R ^nome$` com
os metacaracteres escapados (`Broken.Case` não pode casar `BrokenXCase`;
provado contra o ctest real com `Core` e `CoreParsing` lado a lado). Sem
runner para o tipo, `INVALID_REQUEST`; sem interpretador ou sem o módulo
pytest no ambiente, `event.test.discovered { success: false, error }` com o
passo. A UI (`TestsPanel`, que agora recebe o `JobsController` inteiro em
vez de quatro escalares): "Listar testes", a árvore com o ponto de status e
um ▶ por linha; um run novo apaga os status e mantém a árvore.

**Python (`0.100.0`).** O pytest roda com o **interpretador do projeto** (ou
`uv run python` — a mesma regra do `run.script`), cwd no root: é lá que os
pacotes do projeto estão, e um pytest de fora do ambiente testaria outra
coisa. Sem interpretador, `event.test.finished { success: false, error }`
orienta a criar o ambiente; com interpretador mas **sem o módulo pytest
naquele ambiente** (o `No module named pytest` do próprio Python, medido no
3.14 desta máquina pela exercitação do gate), o `error` diz como instalar
NELE: `uv add --dev pytest` (projeto do uv) ou `.venv/bin/python -m pip
install pytest`. O `error` é a forma geral de "o runner nem correu" (o
`emit_run_error` que build e quality já usavam); a UI (`0.100.0`) mostra-o
como resumo do painel Testes em vez de "passou: 0". **O painel Testes passou a
mostrar a saída bruta** (`event.test.started` + `event.test.output`, a metade
de baixo quando há linhas): é onde o pytest explica a falha — o sinal existia
no C++ desde o `test.run` e ninguém ouvia (`roadmaps/41` A6).

> **Nota:** com build/quality/test todos como jobs assíncronos, não existe mais
> caminho de streaming síncrono no core — toda operação longa retorna `jobId` e
> emite eventos pelo canal assíncrono.

### LSP (`lsp.didChange` / `lsp.definition` / `lsp.hover` / `lsp.completion` / `lsp.references` / `lsp.rename`)

Implementado nos protocolos `0.7.0` e `0.8.0`. Requer workspace aberto. A UI envia
`lsp.didChange { "path": "/abs/file", "content": "..." }` depois de debounce
do editor; o core canonicaliza o caminho, confina à raiz do workspace e
sincroniza o texto completo com o servidor LSP gerenciado. A UI nunca fala LSP
diretamente.

O core sobe servidores de linguagem sob demanda quando um arquivo suportado é
aberto, alterado ou salvo:

- C/C++: `clangd --background-index`
- Rust: `rust-analyzer`
- Python (`0.99.0`, 2026-09-13): `basedpyright-langserver --stdio` — o binário
  **detectado** (`~/.local/bin` do pipx/npm entra pelo detector; o `PATH` do
  processo da IDE pode não o ter), `languageId: "python"` para `.py/.pyi/.pyw`.

Se o servidor não estiver instalado ou falhar no `initialize`, o core emite
`event.lsp.status` com `status: "failed"` e mensagem humana. O MVP usa
sincronização de documento inteiro.

**Configuração do servidor (`0.99.0`).** Um `ServerSpec` pode carregar
`settings` (JSON). Quando os há, o core envia `workspace/didChangeConfiguration
{ settings }` **logo após o `initialized`** e responde ao
`workspace/configuration` que o servidor pergunta (LSP 3.17): um valor por
`ConfigurationItem`, navegando `section` com pontos (`python.analysis` →
`settings.python.analysis`); seção inexistente → `null`; item sem `section`
(ou vazia) → a configuração inteira. Servidor sem `settings` (clangd,
rust-analyzer) não recebe configuração alguma — o wire é o mesmo de antes.

Para Python os `settings` são montados pelo core a partir do **interpretador
que o projeto resolve** (`python/env.rs`, a precedência do `roadmaps/29`
§4.1 — o mesmo que `python.status` e `index.context` mostram):
`{ python: { pythonPath, analysis: { autoSearchPaths, diagnosticMode:
"openFilesOnly" } }, basedpyright: { analysis: {…} } }`. Sem interpretador
nenhum (workspace sem `.venv`, sem `python3`) não se empurra configuração — a
IDE não inventa `pythonPath`. `diagnosticMode: "openFilesOnly"` é decisão: o
modo `workspace` faria o basedpyright analisar o projeto inteiro a cada
mudança. O `event.python.finished` com `success: true` (o `.venv` nasceu)
reconfigura e, se o servidor de Python estiver vivo, **reinicia-o** — sai
`event.lsp.restarted { language: "python" }` e a UI reabre os documentos; um
`success: false` não reinicia nada. Sem isto o basedpyright indexaria a stdlib
do Python do `PATH` e o completar mentiria sobre os pacotes do projeto.

**Dois servidores Python (validado em 2026-09-15, sem mudar o IPC).** O
`lsp/registry.rs` mantém o principal `python` (basedpyright) e o companheiro
`python-ruff` (`ruff server`), registrado quando o detector encontra Ruff.
`didOpen`/`didChange`/`didSave`/`didClose` alcançam os dois, com versões
independentes. Navegação, hover e completion continuam no principal; as code
actions consultam ambos. Cada `event.lsp.diagnostics` leva a união das listas
por arquivo, e limpar/remover um servidor preserva a parte do outro. Ruff
ausente não impede o principal; falha de subida do companheiro gera status e
o retira do registro para não repetir a falha a cada sincronização.

`lsp.definition` e `lsp.hover` foram adicionados no protocolo `0.8.0`. Ambos
recebem posição 1-based e o buffer atual para o core sincronizar o documento
antes de consultar o language server:

```json
{
  "path": "/abs/file",
  "content": "texto atual do editor",
  "line": 12,
  "column": 5
}
```

`lsp.definition` responde `{ "path"?, "line"?, "column"? }`; campos ausentes
significam que o servidor não encontrou alvo. `lsp.hover` responde
`{ "content"? }`, com markup LSP achatado em texto.

`lsp.completion`, `lsp.references` e `lsp.rename` foram adicionados no
protocolo `0.10.0` (Fase 5.2). Completion e references usam os mesmos
parâmetros posicionais de `lsp.definition`; rename recebe adicionalmente
`"newName"` (não vazio):

- `lsp.completion` responde `{ "items": [{ "label", "insertText",
  "detail"?, "kind"? }], "isIncomplete": bool }`. O core ordena por `sortText`
  e limita a 100 itens; `kind` é o `CompletionItemKind` numérico do LSP
  achatado em texto (`function`, `variable`, ...). **`isIncomplete`
  (protocolo `0.37.0`)** é `true` quando o servidor marcou a lista incompleta
  OU o core truncou (ex.: `std::c` no clangd, centenas de candidatos): a UI
  então **repede** completion ao digitar mais (prefixo maior → itens que não
  couberam, como `cout`), em vez de filtrar só o cache. Sem isso, itens fora
  dos primeiros N nunca apareciam.
- `lsp.references` responde `{ "references": [{ "path", "line", "column" }] }`
  (1-based, limitado a 200 usos, incluindo a declaração).
- `lsp.rename` consulta o servidor e normaliza o `WorkspaceEdit` para a
  transação descrita abaixo. Renames que criam/renomeiam/apagam arquivos
  (resource operations) ainda não são suportados e retornam erro estruturado
  sem tocar em nada.

`lsp.semanticTokens` foi adicionado no protocolo `0.14.0`. Desde `0.54.0`,
recebe `{ path, content, version }`, sincroniza o buffer
e resolve `textDocument/semanticTokens/full`, decodificando os deltas com a
legend anunciada pelo servidor no `initialize`. Responde
`{ path, version, tokens: [{ line, start, length, kind }] }` com `line` 1-based e
`start`/`length` em unidades UTF-16 (0-based) — os mesmos índices de
`QString`, aplicados direto pelo highlighter da UI. `kind` é o nome da
legend (`variable`, `function`, `parameter`, `class`, ...). Servidores sem
suporte respondem lista vazia. A UI incrementa a versão no instante da edição,
limpa tokens anteriores e só aplica resposta cujo `path`+`version` ainda
corresponde ao buffer ativo; o tempo de chegada do LSP nunca substitui a base
Tree-sitter de uma versão mais nova.

`lsp.codeActions` e `lsp.applyCodeAction` foram adicionados no protocolo
`0.22.0` (fatia M1.3 de `DocsPrivate/diario/18-daily-driver-plan.md`):

- `lsp.codeActions { path, content, line, column }` →
  `{ actions: [{ title, kind? }] }`. O core sincroniza o buffer, envia
  `textDocument/codeAction` com range-ponto no cursor e compõe o `context`
  com os diagnostics que o próprio servidor publicou para a linha (cache da
  thread leitora — a UI não devolve diagnóstico). Só entram na lista ações
  `CodeAction` literais com `edit` inline e sem `disabled`; ações que
  dependem de `workspace/executeCommand` são filtradas (decisão registrada
  em DocsPrivate/diario/18). As ações cruas ficam guardadas como **consulta ativa**,
  junto da chave do servidor que as produziu. A lista reúne principal e
  companheiros; o contexto enviado a cada servidor contém seus próprios
  diagnósticos. O preview valida versões no servidor de origem da ação.
- `lsp.applyCodeAction { path, content, actionIndex }` cria a mesma transação
  confirmável de `lsp.rename`. A consulta é consumida na chamada; índice
  inválido, arquivo diferente ou consulta
  expirada (qualquer didChange real invalida) retornam `INVALID_PARAMS`
  pedindo nova consulta.

**Workspace edits confirmáveis (protocolo `0.47.0`):** rename e code action
respondem primeiro com um preview:

```text
{ transactionId, title, edits,
  files: [{ path, before, after }] }
lsp.workspaceEdit.apply  { transactionId }
  → { transactionId, files, edits }
lsp.workspaceEdit.cancel { transactionId }
  → { transactionId, cancelled: true }
```

O core confina todos os paths, rejeita versões LSP incompatíveis, mantém um
número limitado de planos pendentes e compara cada snapshot do disco outra
vez no `apply`. A escrita multi-arquivo compartilha a primitiva transacional
do `fs.replace`: arquivos atômicos e rollback de todos os já gravados em caso
de falha. Só após sucesso editor, watcher e LSP são ressincronizados. A UI
mostra `before`/`after` lado a lado e nunca aplica edits por conta própria.

### Documentos fechados após um configure (`event.lsp.documentsClosed`)

Nasceu em 2026-09-02 (etapa 4 do `DocsPublic/roadmaps/30-caminho-para-o-mvp.md`).
`{ language, count }`, hoje sempre `language: "cpp"`.

**O que aconteceu:** um `cmake.configure` terminou com sucesso, e o core fechou
(`textDocument/didClose`) os documentos C/C++ que o language server conhecia.

**Por que o core faz isso.** O clangd recarrega a `compile_commands.json`
sozinho desde a v12 — reconfere a cada ~5 s
(<https://reviews.llvm.org/D92663>) —, mas **o documento já aberto fica com a
compilação em cache**. Reiniciar o servidor resolveria e jogaria o índice fora;
a ação certa é reabrir o documento. E reabrir exige **fechar antes**: o
curto-circuito por hash do core torna um `didOpen` repetido inerte, porque
depois do configure o texto não mudou.

**O que a UI deve fazer:** re-sincronizar o arquivo ativo, exatamente como já
faz em `event.lsp.restarted` e no `recovered()`. É esse `didOpen` seguinte que
leva o **buffer do editor** (não o disco) ao servidor, agora com as flags novas.
Sem reagir ao evento, o arquivo ativo fica sem diagnóstico até o usuário digitar.

**Onde a decisão mora, e por quê.** No core, e não na UI: a UI não consegue
fazê-lo sozinha (o `didOpen` seria inerte e o `didClose` não existia até
2026-09-02). O gatilho é o próprio `event.cmake.finished` do job, observado pelo
loop principal antes de ser repassado — o job roda em thread própria e não
alcança o `Core`, mas o evento dele volta ao dono do estado
(`DocsPublic/arquitetura/04-boot-e-comunicacao.md` §3).

### Indentação pela gramática (`syntaxTree.indent`, `0.135.0`)

Recebe `{ path, version, line, column, trigger }` — `line` 1-based, `column`
0-based em unidades UTF-16, `trigger` em `newline` ou `closeDelimiter` — e
responde `{ path, version, language, level, dedentTo? }` ou **`null`**.

Três coisas que o formato diz de propósito:

- **`version` é a da ÁRVORE**, não a que foi pedida. O core responde a partir da
  árvore que tem, que pode ser anterior ao que já foi digitado; a UI descarta a
  resposta quando as duas não batem. É por isso que o `ParsedDocument` guarda
  versão;
- **`level` é um número de níveis**, e o texto de um nível é decisão do editor
  (espaço ou tabulação). Devolver texto obrigaria os dois lados a concordar
  sobre a configuração;
- **`null` não é erro.** Significa "a gramática não sabe aqui" — árvore com
  `ERROR` em volta do cursor, ou arquivo sem gramática. O editor já aplicou o
  fallback local antes de perguntar, e ele continua valendo. Transformar isso em
  falha encheria o log de algo que está funcionando como desenhado.

Nada disso está no caminho síncrono da tecla: o fallback local decide na hora, e
esta resposta só corrige quando diverge.

### Sintaxe incremental (`syntaxTree.update`)

Implementado no protocolo `0.46.0` para C, C++ e Rust. Recebe
`{ path, content, version }` e responde:

```text
{ path, language, version, hasErrors,
  highlights: [{ line, start, length, scope }],
  foldingRanges: [{ startLine, endLine }],
  outline: [{ name, kind, line, column, endLine, children? }],
  locals: [{ kind, name?, line, column, endLine, endColumn }] }
```

`start`/`length` e colunas usam UTF-16 para casar com Qt/LSP. O core usa
Tree-sitter incremental com cache LRU de 32 buffers e limite de 4 MiB; versão
obsoleta é descartada na UI. A composição visual é regex fallback <
Tree-sitter < semantic tokens LSP < diagnósticos/busca. `locals` é índice
sintático local, nunca promovido a referência semântica.

`lsp.documentSymbols` e `lsp.workspaceSymbols` foram adicionados no
protocolo `0.23.0` (fatia M1.4 de `DocsPrivate/diario/18-daily-driver-plan.md`):

- `lsp.documentSymbols { path, content }` →
  `{ symbols: [{ name, kind, path, line, column, container? }] }`. Achata
  os dois shapes do LSP (`DocumentSymbol[]` hierárquico em pré-ordem, com o
  pai como `container`, ou `SymbolInformation[]` plano), preservando a
  ordem do documento; `kind` é o `SymbolKind` nomeado (`struct`, `method`,
  `function`, ...). Cap de 500 símbolos.
- `lsp.workspaceSymbols { path, content, query }` → mesmo shape. `query`
  não vazia (`INVALID_PARAMS` caso contrário), casada server-side via
  `workspace/symbol` **no servidor da linguagem do arquivo ativo** (`path`)
  — consulta multi-servidor está fora do contrato por ora. Cap de 100.
- Posições 1-based; `path` dos símbolos é canônico/absoluto (a UI abre e
  salta direto, como faz com diagnósticos). URIs fora de `file://` são
  ignoradas.
- Na UI, ambos alimentam o Search Everywhere: prefixo `@` lista/filtra a
  estrutura do arquivo atual; `#nome` busca no workspace.

`lsp.switchSourceHeader` foi adicionado no protocolo `0.34.0` (fatia T1
de `DocsPrivate/diario/18-daily-driver-plan.md`, trilha T de `DocsPublic/roadmaps/21`):

- `lsp.switchSourceHeader { path, content }` → `{ path: string | null }`.
  Alterna entre header e source do mesmo componente C/C++ via a
  **extensão do clangd** `textDocument/switchSourceHeader` (params é o
  `TextDocumentIdentifier` PLANO `{ uri }`, sem envelope; resposta é uma
  URI ou `null`). O core sincroniza o documento antes e converte a URI
  de volta ao caminho absoluto.
- Gate de linguagem no core: só o servidor C/C++ tem a extensão —
  arquivo de outra linguagem → `INVALID_PARAMS` (`UnsupportedFile`),
  nunca deixando o rust-analyzer responder "method not found" cru. Sem
  contraparte → `path` ausente (não é erro). clangd ausente →
  `TOOL_NOT_FOUND`. Na UI: atalho `Alt+O` + comando "C/C++: Alternar
  header/source" no Search Everywhere; a UI abre o contraparte reusando
  o caminho de abertura por diagnóstico.

`lsp.restart` foi adicionado no protocolo `0.39.0` (fatia M4.3b — robustez
do LSP travado):

- `lsp.restart { language? }` → `{ restarted: [string] }`. Reinicia o
  principal e os companheiros de UMA linguagem (`"rust"`|`"cpp"`|`"python"`);
  emite um `event.lsp.restarted` por linguagem. Sem `language`, reinicia
  TODOS os vivos. Reiniciar = matar o processo e removê-lo; sobe de novo
  (lazy) no próximo request. `restarted` lista as linguagens que tinham
  servidor rodando.
- **Auto-restart:** o core conta timeouts CONSECUTIVOS por servidor
  (`REQUEST_TIMEOUT` de 4s); ao chegar a 3 seguidos (~12s preso), reinicia
  aquele servidor sozinho. Qualquer resposta zera a contagem.
- Novo evento `event.lsp.restarted { language }` (auto OU comando): a UI
  re-sincroniza o arquivo ativo (o servidor novo não conhece os documentos
  abertos — `refreshSemanticTokens` refaz o `didOpen`, mesmo caminho do
  `recovered()` do crash). Na UI: comando "LSP: Reiniciar servidor" no
  Search Everywhere.

Cancelamento de requests LSP ainda não faz parte deste contrato.

```text
event.lsp.status       { "language": "cpp|rust|python|python-ruff", "status": "running|failed|stopped|exited|restarting", "message"? }
event.lsp.diagnostics  { "path": "/abs/file", "diagnostics": [{ "source": "lsp", "category": "lsp", "severity": "error|warning|note", "message", "line", "column" }] }
event.lsp.log          { "language": "cpp|rust|python|python-ruff", "line": "..." }   (0.111.0)
```

`line` e `column` dos diagnósticos são 1-based para consumo direto da UI. A
UI integra esses eventos à aba Problemas com origem `lsp`.

**`event.lsp.log` (`0.111.0`)** é o stderr do servidor, uma linha por evento,
sem interpretação: o `clangd` anunciando a versão e onde procurou o
`compile_commands.json`, o rust-analyzer contando o índice, o traceback de um
basedpyright que morreu no `import`. Linhas são UTF-8 com perda e cortadas em
4 KiB com marcador; o core guarda as últimas 64 (a cauda). A UI as escreve na
aba IDE. A cauda também vai no **`message`** do `event.lsp.status` quando
`status` é `failed` (dentro do texto do erro, sob `--- stderr do servidor ---`)
ou `exited` (o `message` inteiro; ausente quando o servidor não disse nada).
Um servidor que falha o `initialize` é morto pelo core — antes ficava vivo e
mudo. Sem agrupamento: medido em 2026-09-17 com os servidores reais nesta
máquina, o pico foi 18 linhas/s (clangd indexando), longe de justificar lote.

### CMake service (`cmake.configure` / `cmake.presets.list` / `cmake.targets.list` / `cmake.status`)

Implementado no protocolo `0.25.0` (fatia M2.2 de `DocsPrivate/diario/18`). Todos exigem
workspace aberto com kind `cmake` (`INVALID_PARAMS` para outros kinds). O
diretório de build é único e imutável: `<root>/.kinein/build` — o mesmo de
`build.run` e `run.start`; presets **não** mudam o diretório (o `-B`
explícito tem precedência).

- `cmake.configure { preset? }` → `{ jobId }` (job cancelável). Escreve a
  query `codemodel-v2` do file-api antes de rodar e sempre passa
  `-DCMAKE_EXPORT_COMPILE_COMMANDS=ON`. Eventos: `event.cmake.started
  { jobId, command, preset?, presetSource? }` e `event.cmake.finished { jobId,
  success, exitCode, hasCompileCommands }`; saída linha a linha vai por
  `event.job.output`. **O preset (`0.115.0`, P0):** o do pedido (a ponte C++
  manda o do kit ativo — o último `toolchain.get` que a tela pediu;
  `presetSource: "kit"`, e as ESCOLHAS de ferramenta são as desse kit) ou o
  **padrão do projeto** — `cmake::default_preset`: o primeiro
  `configurePresets` não `hidden` de `CMakeUserPresets.json` (a escolha do
  usuário para esta máquina vence), senão de `CMakePresets.json`, pulando o
  que uma `condition` (`const false`, ou `equals`/`notEquals` sobre
  `${hostSystemName}`) exclui no Linux; `presetSource` é o arquivo; as
  escolhas de ferramenta são as do kit padrão. Sem presets, configure sem
  preset como sempre e `presetSource` ausente. Antes, o configure automático
  de um projeto com presets rodava SEM preset. No sucesso o core anota o
  preset em `<buildDir>/.kinein-preset` (o `CMakeCache.txt` não o guarda) e
  o `cmake.status` o devolve.
- `cmake.presets.list {}` → `{ presets: [{ name, displayName? }] }` —
  configure presets não ocultos de `CMakePresets.json` +
  `CMakeUserPresets.json`, na ordem dos arquivos; JSON inválido → erro
  humano.
- `cmake.targets.list {}` → `{ origin, targets: [{ name, kind, artifacts[],
  sources, generatedSources, languages[], standard?, includes, defines,
  sysroot?, dependencies[], sourceDir? }] }` — lidos da resposta codemodel-v2
  do file-api do último configure (`kind`: `executable`, `staticLibrary`,
  ...; utilitários ficam de fora); vazio antes do primeiro configure. **Desde
  o `0.96.0` (2026-09-12) cada target carrega o seu MODELO** (`cmake/model.rs`):
  `artifacts` absolutos ao build dir (o ELF que o P2 grava), `sources` do
  autor e `generatedSources` (moc/rcc) separados, `languages` na grafia do
  file-api (`C`, `CXX`), `standard` do `languageStandard` do primeiro grupo,
  `includes`/`defines` distintos entre os grupos, `sysroot` quando há
  `CMAKE_SYSROOT`, `dependencies` com os ids resolvidos para nome (um alvo
  importado, fora do codemodel, não entra). Medido neste repositório:
  `kinein-vectis` com 301 fontes + 521 geradas, `CXX` 23, 12 includes, 8
  defines, artefato `.kinein/build/ui/kinein-vectis`. A query passou a pedir
  também a `toolchains-v1` (CMake ≥ 3.20, cmake-file-api(7)): compilador,
  id, versão e includes implícitos por linguagem — vale a partir do próximo
  configure.
- `cmake.status {}` → `{ configured, hasCompileCommands, buildDir, preset? (0.115.0),
  cdbDirectory?, cdbStale?, cdbStaleBecause? }` — stat de
  `CMakeCache.txt`/`compile_commands.json`, mais o **diagnóstico da compilation
  database** (protocolo `0.62.0`).
- **`hasCompileCommands` e `cdbDirectory` não são a mesma pergunta**, e confundi-
  las é a origem de "erro de include sem causa":

  ```text
  hasCompileCommands   existe CDB no build dir DA IDE (.kinein/build)?
  cdbDirectory         existe CDB que o clangd ALCANCA, e onde? Relativa ao
                       root; "." e' a propria raiz. AUSENTE quando nao ha.
  ```

  O clangd procura `compile_commands.json` **nos diretórios pai e em
  subdiretórios `build/`** por conta própria
  (<https://clangd.llvm.org/installation>), então um projeto **Meson ou `bear`
  funciona com `hasCompileCommands: false`**. Só `cdbDirectory` responde se o
  usuário tem inteligência de código ou não.
- `cdbStale` + `cdbStaleBecause`: a CDB alcançável é mais velha que um arquivo
  de build que a define (`CMakeLists.txt`, `CMakePresets.json`, `meson.build`,
  `Makefile`). O clangd então usa flags de um projeto que mudou. Os três campos
  são **omitidos quando não há o que reportar** — projeto sadio não carrega
  ruído, e a UI não precisa distinguir `false` de ausente.
- clangd: quando `compile_commands.json` existe no build dir da IDE, novos
  servidores cpp sobem com `--compile-commands-dir` apontando para ele — o
  `.kinein/build/` **não** é `$SRC/build/`, então o clangd não o acharia sozinho.
- **Recarga de flags, corrigido em 2026-08-30.** O clangd **tem** hot-reload da
  `compile_commands.json` desde a v12 (reconfere a cada ~5 s,
  [D92663](https://reviews.llvm.org/D92663)). O que não atualiza é o **documento
  já aberto**, que fica com a compilação em cache
  ([vscode-clangd #42](https://github.com/clangd/vscode-clangd/issues/42)).
  Logo a ação certa após um configure é **reabrir os documentos abertos**, e não
  reiniciar o servidor — reiniciar joga o índice fora.

### Cargo service (`cargo.metadata` / `cargo.check`)

Implementado no protocolo `0.26.0` (fatia M2.3 de `DocsPrivate/diario/18`). Ambos exigem
workspace aberto com kind `rustCargo` (`INVALID_PARAMS` para outros kinds).

- `cargo.metadata {}` → `{ packages: [{ name, version, features: [...],
  targets: [{ name, kind }] }], workspaceMembers: [...] }`. Síncrono:
  `cargo metadata --format-version 1 --no-deps` é local e rápido (medido
  ~10ms num projeto pequeno); o core resume o JSON gigante do cargo.
  Falha vira erro humano (`INTERNAL_ERROR` com a última linha do stderr).
- `cargo.check {}` → `{ jobId }` (job cancelável). Roda `cargo check
  --workspace --all-targets --message-format=json` e **reusa o pipeline do
  quality**: diagnósticos chegam por `event.quality.diagnostic` e o fim
  por `event.quality.finished` — caem na aba Problemas sem parser novo.
  Aliasing consciente até o Problems 2.0 ter facetas por origem; o título
  do job ("Cargo Check") o distingue do clippy na aba Jobs.

### Run configurations (`runConfig.list` / `runConfig.save` / `runConfig.delete` / `runConfig.setActive` / `runConfig.flashProposal`)

Implementado no protocolo `0.27.0` (fatia M2.4 de `DocsPrivate/diario/18`). Todos exigem
workspace aberto (qualquer kind). Persistência em `.kinein/runconfigs.json`
com `schemaVersion` (arquivo inválido/schema desconhecido = vazio, nunca
quebra). Uma config v1 é `{ id, name, command }` — comando shell executado
na raiz pelo `run.start`.

- Todas as quatro operações respondem o mesmo shape:
  `{ configs: [{ id, name, command }], activeId? }` — a UI nunca calcula
  estado derivado.
- `runConfig.save { id?, name, command }`: cria quando `id` está ausente
  (ids `cfg-N`); criar/editar torna a config a ATIVA. Nome/comando vazios →
  `INVALID_PARAMS`.
- `runConfig.delete { id }`: remove; se era a ativa, volta a "automático".
- `runConfig.setActive { id? }`: `id` ausente = automático; id inexistente →
  `INVALID_PARAMS`.
- `run.start {}` (sem `command`) resolve: comando explícito > config ativa >
  heurística (`cargo run` / executável único do CMake / o ponto de entrada
  Python — na placa, com o `device?` escolhido, quando o projeto é
  MicroPython). Só a heurística recebe `device`.
- **`runConfig.flashProposal { device?, engine?, flashSizeBytes?, firmware? }` →
  `{ name, command, engine, source, warnings }` (`0.113.0`, E4 do
  `integracoes/38` §6; `firmware` em `0.116.0`).** "Gravar" como configuração de execução, na forma
  decidida pelo autor em 2026-09-11 (o Upload do PlatformIO, o Download &
  Run do CLion): **puro** — nada roda, nada é salvo; a resposta é uma
  proposta que a UI mostra como prévia e que vira `runConfig.save` (nome
  `Gravar (<motor>)`) ou `run.start { command }`. `engine` ausente = o
  `target.flashEngine` do modelo; sem ele, uma receita `flasher_args.json`
  no build sugere `esptool` sozinha; sem nada, `INVALID_REQUEST`. As linhas,
  cada uma com a evidência em `source`: **esptool** `--chip <receita|kit>
  --port <device> --baud 460800 --before/--after <receita> [--no-stub]
  write-flash --flash-mode/size/freq <receita> <offset> '<imagem>'…` — tudo
  do `flasher_args.json` que o ESP-IDF escreve (nada adivinhado; `device`
  obrigatório: gravar na placa errada é pior que um clique; sem receita,
  "compile o projeto"); **probe-rs** `download --chip <chip> '<ELF mais
  novo>'` (chip do kit/modelo obrigatório); **picotool** `load -f -x
  '<UF2>'`; **dfu-util** `-a 0 -s 0x08000000:leave -D '<BIN>'` **só para
  STM32** — outra família não tem tabela, e sem tabela não há comando.
  Caminhos entre aspas simples POSIX (a linha roda por `sh -c`).
  `warnings`: imagens marcadas como cifradas (`write-flash` grava em claro;
  acrescente `--encrypt`), e `flashSizeBytes` (a flash que `serial.identify`
  leu) menor que a da receita. Erros: ferramenta ausente → `TOOL_NOT_FOUND`
  com o passo; motor desconhecido → `INVALID_PARAMS`; o resto (sem build,
  sem porta, sem chip, família sem tabela) → `INVALID_REQUEST`. A linha
  salva é editável como qualquer configuração — é assim que um `esptool` v4
  troca `write-flash` por `write_flash`.
- **Os wrappers dos frameworks (`0.117.0`, bloco E):** `engine: "idf.py"`
  → `bash -c '<wrapper>' idf '<ativação>' -p '<porta>' flash` (a linha do
  "Start a Project": `idf.py -p PORT flash`; a porta é exigida, como no
  esptool; sem ativação nesta máquina, `TOOL_NOT_FOUND` com o passo);
  `engine: "west"` → `'<west>' flash -d build` (o **padrão** de um projeto
  Zephyr, pelo `target.flashEngine` do modelo; uma porta escolhida vira
  aviso — o runner do board grava pela sonda/USB); `engine: "platformio"`
  (ou `pio`) → `'<pio>' run -t upload [--upload-port '<porta>']` (o padrão
  de um PlatformIO). Num ESP-IDF o padrão continua sendo o esptool com a
  receita do build; o `idf.py` é escolha explícita.
- **`firmware` (`0.116.0`, C5 do `roadmaps/41` bloco C):** o id de um
  firmware do catálogo (`toolchain.installable`, `kind: firmware`) já
  BAIXADO substitui os artefatos do build. A linha é a da página da placa
  no micropython.org (lida em 2026-09-17): **esptool** `--chip <chip da
  página> --port <device> --baud 460800 --before default-reset --after
  hard-reset write-flash <offset> '<arquivo>'` — `0x1000` no ESP32 clássico,
  `0x0` em C3/S3 —, com `warnings` para a primeira instalação (a página
  manda `erase-flash` antes; a linha pronta vai no aviso, nunca na de
  gravar), para um chip do kit diferente do da página, e para a flash
  identificada menor que o arquivo; **picotool** `load -f -x '<uf2>'` (o
  mesmo que copiar para o drive `RPI-RP2`, como a página manda). O motor é
  o da página: `engine` diferente → `INVALID_PARAMS`; firmware não
  baixado, id de toolchain ou desconhecido → `INVALID_REQUEST` com o passo.
  `name` é `Gravar firmware (<rótulo>)`.

### Configuration Actions (`configAction.list` / `configAction.preview` / `configAction.apply`)

Implementado no protocolo `0.63.0` (etapa 2 de `DocsPublic/roadmaps/30-caminho-para-o-mvp.md`).
Os três exigem workspace aberto. O escopo de cada ação é o **build system
ativo** do workspace (spec 9.2 §1): projeto Cargo não vê ação `CMake`, e pedir
uma fora do escopo é `INVALID_PARAMS`, não silêncio.

**São três métodos porque o ciclo tem três passos**, e a spec 9.1 §10 exige os
três: *explicar, mostrar preview, aplicar com consentimento e validar*. Um
método só seria um botão que edita o `CMakeLists.txt` do usuário sem mostrar o
quê.

- `configAction.list { includeHiddenByScope? }` →
  `{ actions: [...], activeBuildSystems: [...] }`. Cada ação traz
  `{ id, title, description, scope, category, risk, affects, effect, state,
  reason?, params, docs }`. O `state` é medido no workspace agora
  (`available`, `recommended`, `partiallyAvailable`, `unavailable`,
  `hiddenByScope`) e o `reason` diz por quê — "o preset debug já existe",
  "nenhum target declarado". `includeHiddenByScope: true` traz também as ações
  do build system inativo, marcadas (spec 9.2 §25).
- `configAction.preview { id, params? }` →
  `{ id, title, summary, files: [{ path, before?, after }], report, notes }`.
  **Não escreve nada.** `before` ausente significa que o arquivo será CRIADO.
  `report` é o resultado das ações de leitura (o cache do `CMake`); `notes` são
  avisos que o usuário precisa ler antes de aplicar.
- `configAction.apply { id, params?, expected? }` →
  `{ id, message, files, jobId? }`. O `expected` é a lista
  `[{ path, content? }]` que veio do `before` do preview: é a **mesma barreira
  do `fs.write`** (`ARCHITECTURE.md` §7.1) aplicada ao consentimento — se o
  disco mudou entre o preview e o Apply, a resposta é `FILE_CHANGED` e nada é
  escrito. Lista vazia dispensa a comparação.

O `effect` de cada ação diz o que o `apply` faz, e por isso está no contrato:

```text
edit      reescreve um arquivo de texto; o preview mostra o diff exato
inspect   so le: o preview E o resultado, e o apply nao escreve
job       devolve `jobId` reusando um job que ja existe (hoje: cargo.check)
delegate  efeito nao textual dentro do core (remover build dir, salvar run config)
```

As 16 ações do MVP são as da §12 da spec de fechamento: 10 de `CMake`
(`enableCompileCommands`, `createDebugPreset`, `createReleasePreset`,
`addExecutable`, `addStaticLibrary`, `addSourceToTarget`, `addIncludeDirectory`,
`addTargetLinkLibraries`, `inspectCache`, `repairBuildDir`) e 6 de Cargo
(`addDependency`, `addDevDependency`, `addFeature`, `setEdition`, `check`,
`createRunConfig`).

**O que o core RECUSA em vez de adivinhar**, e é o comportamento certo: nome de
target fora do padrão da CMP0037, target inexistente, fonte com `..`,
dependência que já está no manifest, `Cargo.toml` com string multilinha ou
`dependencies` como tabela inline, `edition.workspace` herdada. Corromper o
manifest do usuário não tem desfazer; a recusa vem com o motivo, na lista e no
diálogo.

### Toolchain (`toolchain.get` / `toolchain.set` / `toolchain.setKit` / `toolchain.installable` / `toolchain.install` / `toolchain.inspectSysroot` / `toolchain.importKit`)

Implementado no protocolo `0.64.0` (etapa 5 de
`DocsPublic/roadmaps/30-caminho-para-o-mvp.md`; B2 do TR2 e §5d do `roadmaps/29`).
Ambos exigem workspace aberto. Persistência em `.kinein/toolchain.json` com
`schemaVersion` — arquivo inválido ou de schema desconhecido é tratado como
vazio, e toolchain quebrada nunca impede a IDE de abrir o projeto.

**A pergunta que o domínio responde:** *qual executável cumpre cada papel?*
Até 2026-09-02 todo processo externo do core nascia de `Command::new("<nome>")`
— 28 chamadas, todas resolvidas pelo `PATH` do processo. Trocar de compilador
significava editar `CMakeLists.txt` à mão ou exportar `CC`/`CXX` antes de abrir
a IDE.

Os **papéis** são vocabulário fechado (`cCompiler`, `cxxCompiler`, `generator`,
`cmake`, `cargo`). Papel novo é entrada nova no protocolo e no catálogo do
core, nunca string livre vinda da UI: um papel que o core não sabe usar daria
ao usuário um seletor sem efeito.

- `toolchain.get {}` e `toolchain.set { role, id? }` respondem o **mesmo**
  shape — como as run configs, a UI nunca calcula estado derivado:

```text
{ selections: [{ role, id?, resolvedPath? }],
  candidates: [{ role, id, label, path?, version? }] }
```

- `id` **ausente** em `selections` é AUTOMÁTICO, e é o padrão: nada é fixado e
  o `PATH` continua decidindo — exatamente o comportamento histórico. Quem
  nunca abrir o seletor não vê diferença nenhuma.
- `resolvedPath` no automático é **informação** (o primeiro candidato
  detectado), não fixação. É o que a UI mostra para dizer o que vai acontecer.
- `toolchain.set` com `id` ausente volta para automático. Com um `id` que não
  existe no catálogo, ou que existe mas **não foi detectado nesta máquina**,
  responde `INVALID_PARAMS`: oferecer um compilador ausente é oferecer um
  configure que vai falhar.

**A escolha passou a ser do KIT, não do workspace (protocolo `0.67.0`).** Um kit
é um preset do CMake mais o que ele precisa para compilar: os executáveis por
papel, o `sysroot` e o triple do alvo.

```text
toolchain.get  { preset? }                         -> ToolchainResult
toolchain.set  { role, id?, preset? }              -> ToolchainResult
toolchain.setKit { preset?, sysroot?, targetTriple?, chip?,               NOVO
                   remoteTarget?, debugServer?, toolchainFile?, svdFile? } -> ToolchainResult
```

**Desde o `0.97.0` (2026-09-12, [`integracoes/39`](../integracoes/39-toolchains-por-alvo.md))
o `ToolchainResult` carrega dois campos que só um processo responde:**
`rustTargets` — os alvos Rust INSTALADOS (`rustup target list --installed`,
pelo `rustup` detectado; **ausente** sem rustup, e a UI então não avisa nada;
com o `targetTriple` do kit fora da lista a UI diz o `rustup target add`) — e
`sysrootHint` — quando o compilador C efetivo é `*-linux-gnu*`, o kit não tem
`sysroot` e o sysroot que o próprio compilador declara (`-print-sysroot`) não
tem `usr/include`: é como as distros empacotam o `gcc-aarch64-linux-gnu`
(medido no Fedora 44), compila e não linka programa de usuário nenhum, e a
dica nomeia as três saídas (rsync da placa, Bootlin, SDK Yocto/Buildroot).
Bare metal não entra. Os candidatos de `cCompiler`/`cxxCompiler`/`debugAdapter`
ganharam no mesmo dia os triples da indústria (`riscv-none-elf`,
`xtensa-esp-elf`, `riscv32-esp-elf`, `aarch64-linux-gnu`,
`arm-linux-gnueabihf`, `riscv64-linux-gnu`; `gdb-multiarch` e os
`<triple>-gdb`), e o detector procura além do `PATH` (o store do xpm, o
`~/.espressif/tools`, a pasta da IDE, `/opt/*/bin`).

`preset` ausente é **o kit padrão do workspace** — que é exatamente o que o
schema 1 do `.kinein/toolchain.json` guardava, e por isso a migração é direta:
as seleções antigas viram o kit padrão **na leitura**, não na escrita. Migrar na
leitura é o que impede a IDE de perder a escolha de quem abriu o projeto e não
mexeu na toolchain.

**Em `toolchain.setKit`, campo ausente NÃO é campo vazio.** Ausente preserva o
valor atual; string vazia (ou só de espaços) limpa. Sem essa distinção, mexer no
`sysroot` apagaria o `targetTriple` e o usuário só descobriria no próximo build.

**O kit chega aos language servers** (`handlers/lsp/toolchain.rs`, chamado ao
abrir o workspace e a cada `toolchain.set`/`setKit`): o compilador cross ao
clangd por `--query-driver` (desde 2026-09-11) e, **desde `0.115.0` (P0)**, o
`targetTriple` ao **rust-analyzer** — `rust-analyzer.cargo.target = <triple>`
("Compilation target override (target tuple)", manual do rust-analyzer,
conferido em 2026-09-17), entregue como o basedpyright recebe a dele: secção
`rust-analyzer` respondida no `workspace/configuration` e empurrada no
`workspace/didChangeConfiguration`. O `cargo build` do kit já levava
`--target`; o servidor checando para o host daria diagnóstico do host, que
parece certo e não é. Vale na próxima subida; um servidor de Rust vivo é
**reiniciado** quando o kit muda (é a UI que o sobe de novo ao re-sincronizar
o arquivo ativo, como em qualquer `event.lsp.restarted`); sem triple, o
servidor sobe sem configuração, como sempre.

**O papel `debugAdapter` entrou no protocolo `0.69.0`** (etapa 22 do
`roadmaps/35`). Antes, o adaptador DAP era uma **constante** no core
(`ADAPTER_BINARY = "lldb-dap"`), e por isso não havia debug de embarcado
nenhum: embarcado não debuga com `lldb-dap`.

```text
lldb-dap    desktop; sem argumento          (o PADRAO — nada muda sem escolha)
probe-rs    embarcado; `probe-rs dap-server` fala DAP por stdin/stdout
gdb         `gdb -q -iex ... -i dap`; todo GDB (`gdb-multiarch`, `<triple>-gdb`)
debugpy     Python (0.101.0); NAO vem do kit: o programa e' o INTERPRETADOR do
            projeto e o adaptador e' `-m debugpy.adapter` (DAP por stdin/stdout)
```

**O catálogo da toolchain NÃO sabe os argumentos, e a fronteira é deliberada.**
O cabeçalho dele diz que responde *"quais binários interessam a cada papel"* —
não *"com quais argumentos"*. Que o `probe-rs` precisa do subcomando
`dap-server` é conhecimento de quem **sobe o processo**, e isso mora no domínio
`dap/`.

**O kit ganhou `chip` no protocolo `0.70.0`** — o alvo do adaptador de
embarcado. Ele vai no **`launch` do DAP**, não na linha de comando: verificado
na documentação do probe-rs, onde `chip` é campo da configuração de launch e não
flag do `dap-server`. Supor o contrário daria um processo que sobe e falha no
primeiro request, com a causa longe do sintoma. **Campo ausente não é campo
nulo** — sem chip escolhido, o `launch` não carrega a chave, mesma regra da
condição de breakpoint (`0.66.0`).

**O `launch` do probe-rs tem a forma DELE (`0.118.0`, P3; medido no
`dap-server` 0.32.0 desta máquina e lido em `server/configuration.rs`):**
`program`+`chip` no topo — a forma que a IDE mandava até aqui — falha com
`Serialization error "missing field coreConfigs"`. A forma certa é `{ cwd,
chip?, coreConfigs: [{ coreIndex: 0, programBinary: <ELF>, svdFile?,
rttEnabled: true }] }`; os outros adaptadores continuam com `program`/`cwd`.
**O kit ganhou `svdFile` (`0.118.0`)** — o CMSIS-SVD do chip (do pack do
fabricante ou do `cmsis-svd`, cada um com a sua licença, `35` §5.7), que o
probe-rs transforma no escopo `Peripherals` (`debug.scopes`). Como o
`toolchainFile`: `toolchain.setKit { svdFile? }` ("" limpa, ausente
preserva), `ToolchainResult.svdFile?`, campo **SVD** no painel de Embarcados.

**O kit ganhou `remoteTarget` e `debugServer` no `0.89.0`** — o caminho para
depurar o que não está na máquina. Quando o kit escolhe o adaptador `gdb` (que
fala DAP nativamente desde a v14, fonte `/usr/share/doc/gdb/NEWS`) e declara um
`remoteTarget` (`host:porta`), o `debug.start` faz **`attach`** com esse alvo —
*"passed to the `target remote` command"* (manual do GDB, capítulo Debugger
Adapter Protocol) — em vez de `launch`. O `debugServer` é o comando do servidor
(QEMU, OpenOCD) que a IDE sobe antes de conectar, com `{program}` trocado pelo
ELF; ela o mata com a sessão. **Ambos são declarados, nunca deduzidos** (a IDE
não sabe qual máquina do QEMU é a placa): `roadmaps/35` §5.1. Isso derruba o que
o `integracoes/36` §3 dizia — não há protocolo GDB-remote novo a escrever, é um
segundo candidato do papel `debugAdapter`.

**Cross-compilador NÃO virou papel novo**, e a medição corrigiu o esboço do
`roadmaps/35` §5.3 que dizia que sim: `arm-none-eabi-gcc` escreve a **mesma**
variável que o `gcc` — `CMAKE_C_COMPILER`. Dois papéis apontando para a mesma
variável seria duplicação. O cross é **candidato** dos papéis que já existem.

**Adaptador desconhecido não é recusado:** roda sem argumento, que é a forma da
maioria dos adaptadores DAP. Recusar o que não está na lista impediria o usuário
de apontar um adaptador que a IDE não conhece — e a lista é curta por ser nova,
não por ser completa.

O que o kit vira, no build:

```text
sysroot        -> -DCMAKE_SYSROOT=<caminho>
targetTriple   -> --target <triple>          (cargo: check, clippy e build)
               -> -DCMAKE_SYSTEM_NAME=<...>  quando o triple é inequívoco
               -> -DCMAKE_SYSTEM_PROCESSOR=<arch>
```

**Bare metal ganha `CMAKE_TRY_COMPILE_TARGET_TYPE=STATIC_LIBRARY`, e sem isso o
configure falha SEMPRE.** Medido em 2026-09-03 exercitando
`thumbv7em-none-eabihf` com o `arm-none-eabi-gcc` real: `CMAKE_SYSTEM_NAME=
Generic` sozinho não basta — o CMake ainda tenta **linkar** um executável no
teste de compilador, e bare metal não tem os stubs do newlib
(`undefined reference to _exit`). O projeto nem chega a ser configurado. A doc
do CMake diz que `STATIC_LIBRARY` existe exatamente para *"cross-compiling
toolchains that cannot link without custom flags or linker scripts"*.

Só entra em `Generic`. Em cross para Linux/Windows o link funciona, e forçar o
teste a virar biblioteca **esconderia** um toolchain de verdade quebrado.

`CMAKE_SYSTEM_NAME` é o que faz o CMake entrar em modo cross; o sysroot sozinho
não muda a decisão de compilador. **Triple que o mapa não conhece não vira
palpite:** fica sem `CMAKE_SYSTEM_NAME`, e o caminho oficial para alvos exóticos
continua sendo um `toolchainFile` do preset. O `--target` do cargo vale para
check e clippy também — compilar para o alvo e checar para o host daria
diagnóstico do host, que é pior que não ter porque parece certo.

O resultado carrega `presetToolchainFile` quando o preset declara um
`toolchainFile` (campo do schema de presets desde a versão 3 / CMake 3.21, com
precedência sobre `CMAKE_TOOLCHAIN_FILE`). **É informação, não escolha:** quando
ele existe, o compilador efetivo pode não ser o que o usuário escolheu aqui, e a
IDE diz isso em vez de deixar procurar no lugar errado.
- `candidates` traz só o que existe aqui. `Unix Makefiles` só aparece se o
  `make` existir — por isso ele entrou no `tools.detect` na mesma fatia.

**O que a escolha muda de fato**, e é o que faz dela uma entidade e não um
enfeite:

```text
cmake.configure   -DCMAKE_C_COMPILER / -DCMAKE_CXX_COMPILER / -G, e o
                  EXECUTAVEL do proprio cmake
build.run         o cmake (configure implicito + --build) e o cargo
quality.run       o cargo do clippy
cargo.check       o cargo
cargo.metadata    o cargo
```

Só o que o usuário **fixou** entra na linha de comando. Emitir
`-DCMAKE_CXX_COMPILER` com o que o `PATH` resolveria hoje congelaria no cache do
`CMake` uma escolha que o usuário não fez.

**O que esta fatia NÃO entrega, e está registrado:** sysroot, cross-compilação e
kit por preset. São o resto do B2 do TR2, e ficam para uma fatia própria — o
que existe hoje é a entidade e a rota até o comando.

### O provedor de instalação de toolchain (`toolchain.installable` / `toolchain.install`, `0.103.0`)

A fatia do [`integracoes/39`](../integracoes/39-toolchains-por-alvo.md) §5,
entregue em 2026-09-13 com a regra que a decisão registrada e o pedido do
autor juntos permitem: *"zero-config = DETECTAR + UM CLIQUE com o comando
visível, NUNCA download calado"* (`roadmaps/42` §8).

```text
toolchain.installable {}       -> { installRoot, toolchains: [InstallableToolchain],
                                    projectFamily? }
toolchain.install { id }       -> { jobId }        (job; event.toolchain.installed no fim)

InstallableToolchain   id, label, version, family (cortex-m | riscv | aarch64-linux |
                       arm-linux | riscv64-linux | espressif | rp2040), url, sizeBytes,
                       sha256, license, source, installDir, installed, recommended,
                       kind (toolchain | firmware; 0.116.0),
                       firmware? { board, engine, offset?, chip?, file }
event.toolchain.installed { jobId, id, version, path, success, error? }
```

**Firmware MicroPython no mesmo catálogo (`0.116.0`, C5 do `roadmaps/41`
bloco C).** Cinco entradas `kind: firmware`, v1.29.0 (2026-08-24) do
micropython.org: `ESP32_GENERIC` (esptool, offset `0x1000`, chip `esp32`),
`ESP32_GENERIC_C3` e `ESP32_GENERIC_S3` (esptool, offset `0x0`),
`RPI_PICO` e `RPI_PICO_W` (picotool, `.uf2`). O mesmo provedor baixa
(`.part`, SHA-256 conferido antes de qualquer coisa) e guarda o ARQUIVO
inteiro em `<installRoot>/<id>/<versão>/<arquivo>` (`firmware.file`) — sem
`tar`, sem `bin/`; `installed` é o arquivo existir. **A fonte não publica
checksum** (conferido nas cinco páginas em 2026-09-17): o SHA-256 pinado
foi medido no download desta sessão e o `source` diz isso — protege contra
um download corrompido ou trocado depois da medição, não contra a fonte
ter sido trocada antes. `recommended` só num projeto MicroPython da mesma
família (`espressif`/`rp2040`); a uma app C do ESP-IDF, nunca. Quem grava é
`runConfig.flashProposal { firmware }`.

**O catálogo é PINADO e medido.** Nove entradas (Linux x86_64): Arm GNU
Toolchain 15.2.rel1 (`arm-none-eabi`, `aarch64-none-linux-gnu`,
`arm-none-linux-gnueabihf`), xPack `arm-none-eabi-gcc` 15.2.1-1.1 e
`riscv-none-elf-gcc` 15.2.0-1, Arm Toolchain for Embedded 23.1.0, Bootlin
glibc stable 2026.08-1 (`aarch64`, `armv7-eabihf`, `riscv64-lp64d`). Cada
SHA-256 foi **lido no arquivo que a fonte publica** em 2026-09-13
(`.sha256asc` da Arm, `.sha` do GitHub release da xPack, `.sha256` da ATfE e
da Bootlin) e cada tamanho veio do `Content-Length` de um HEAD no tarball —
`toolchain/install/catalog.rs` guarda a fonte e a data ao lado de cada
número. Não entram: Espressif (o `idf_tools.py` é o instalador oficial e a
IDE já lê `~/.espressif/tools`), Zephyr SDK (precisa do `setup.sh` —
importar kit, P1), e "latest" de qualquer fonte.

**`toolchain.installable` não exige workspace** (catálogo e pasta são desta
máquina); com um, a `family` do `project.model` marca `recommended` —
`stm32`/`rp2040`/`nrf`/`cortex-m` → `cortex-m`, `riscv` → `riscv`;
`espressif` e `linux` não recomendam nada. `installed` é
`<installRoot>/<id>/<version>/bin` existir.

**`toolchain.install` recusa antes de gastar rede:** id fora do catálogo →
`INVALID_PARAMS` apontando o método da lista; já instalada →
`INVALID_REQUEST` dizendo onde está; `tar` ausente → `TOOL_NOT_FOUND`. O job
então: baixa por `ureq` (TLS por rustls; prazo de 60 s **entre bytes**, não
total — um tarball de 400 MB numa rede lenta leva o que levar) para
`<id>/<version>.part/`, calculando o SHA-256 no caminho e reportando
`event.job.progress` por ponto percentual; **confere o digest ANTES de
desempacotar** — divergiu, o erro mostra os dois digests e nada é
desempacotado; `tar -xf … --strip-components=1 -C` (o `tar` do sistema, GPL,
como processo; todos os tarballs do catálogo trazem uma pasta de topo, e é
ela que sai) numa pasta dentro do `.part`; exige `bin/` no resultado; só então
renomeia para `<version>` e apaga o `.part`. Cancelamento, HTTP ≠ 200, disco
e `tar` com erro deixam no máximo um `.part`, que a próxima tentativa apaga.
Provado com as peças REAIS num servidor local: ureq de verdade, SHA-256 de
verdade, `tar` de verdade (`toolchain/install/mod.rs`, testes).

**Depois do sucesso o detector já enxerga** (`tools/search_dirs.rs`,
`installed_bin_dirs`): a pasta da IDE é enumerada a cada busca, não na
construção do detector; o core refaz o registro de ferramentas ao ver
`event.toolchain.installed { success: true }`, e o `toolchain.get` seguinte
lista o compilador novo como candidato — o kit pode fixá-lo. O `PATH` continua
vencendo a pasta da IDE: o que o usuário escolheu vence o que a IDE
encontrou. A UI (`EmbeddedInstallView`) mostra label, versão, tamanho, URL,
sha256 e licença de cada entrada, recomendadas primeiro, e um botão por linha;
uma instalação por vez.

### O gerenciador que lê o disco (`toolchain.inspectSysroot` / `toolchain.importKit`, `0.104.0`)

Os itens (b) e (d) do `roadmaps/42` §8 e o §3 do `integracoes/39`, entregues
em 2026-09-13. Nenhum dos dois exige workspace nem grava nada: leem caminhos
desta máquina e respondem; aplicar é o `toolchain.setKit` de sempre.

```text
toolchain.inspectSysroot { path }  -> SysrootReport { path, exists,
                                      folders { usrInclude, usrLib, lib },
                                      tripleLibDirs[], pkgconfigFiles, libc?, verdict }
toolchain.importKit { path }       -> KitImport { kind (yocto | buildroot | zephyr-sdk | toolchain-dir),
                                      path, evidence[], cCompiler?, cxxCompiler?, gdb?,
                                      sysroot?, targetTriple?, toolchainFile?, hint? }
toolchain.setKit { …, toolchainFile? }   ("" limpa; ausente preserva)
ToolchainResult.toolchainFile?           (o do KIT; presetToolchainFile e' o do preset)
```

**`inspectSysroot` responde "o `--sysroot` aqui vai achar headers e
bibliotecas?"** Lê `usr/include`, `usr/lib`, `lib`, os `usr/lib/<triple>` e
`lib/<triple>` do multiarch (Debian/Raspberry Pi OS), conta os `.pc` onde o
pkg-config procura (`usr/lib/pkgconfig`, `usr/lib64/pkgconfig`,
`usr/share/pkgconfig`, `<triple>/pkgconfig`), e identifica a libc — glibc pela
`__GLIBC__`/`__GLIBC_MINOR__` de `usr/include/features.h`, musl pela
`libc.so`. O veredito é uma frase: *utilizável* (com libc e `.pc`, ou o aviso
"sem .pc — o pkg-config não vai achar bibliotecas de terceiros"), *só
headers*, *só bibliotecas*, ou *vazia para o compilador* — que é o sysroot de
distro do Fedora medido em 2026-09-12 — com o remédio. Caminho relativo →
`INVALID_PARAMS`; pasta inexistente não é erro, é um relatório com
`exists: false`.

**`importKit` propõe, não grava.** Por evidência no caminho:
- **Yocto** — um `environment-setup-*` (o arquivo, ou a pasta do SDK que
  contém UM). O script é **carregado pelo `sh`** — é o que o manual do SDK
  manda fazer (`. environment-setup-…`, docs.yoctoproject.org sdk-manual,
  lido em 2026-09-13) — e as variáveis que ele exporta são lidas: `CC`/`CXX`/
  `GDB` (o `CC` do SDK já traz `--sysroot=$SDKTARGETSYSROOT`; o binário é
  resolvido por `command -v` no `PATH` que o script montou), `SDKTARGETSYSROOT`,
  `OECORE_NATIVE_SYSROOT` (o toolchain file do SDK mora em
  `usr/share/cmake/OEToolchainConfig.cmake`), `TARGET_PREFIX` (o triple). Um
  script que não exporta `CC`, ou que falha ao carregar, é recusa que diz isso;
  dois scripts na pasta = a IDE não escolhe (aponte o arquivo).
- **Buildroot** — `output/`, `output/host` ou a raiz da árvore: a marca é
  `host/share/buildroot/` (onde fica o `toolchainfile.cmake`), mais
  `host/bin/<triple>-gcc` e `host/<triple>/sysroot` (manual do Buildroot,
  "Using the generated toolchain outside Buildroot", 2026-09-13). Os tarballs
  da **Bootlin** entram por aqui — são SDKs do Buildroot, com o arquivo de
  CMake.
- **zephyr-sdk** (2026-09-17) — a raiz de um Zephyr SDK do sdk-ng: a prova
  é `sdk_version` **e** `cmake/Zephyr-sdkConfig.cmake` (o pacote CMake que o
  `setup.sh` registra em `~/.cmake/packages/Zephyr-sdk`). As toolchains GNU
  moram em `gnu/<toolchain>/bin/<toolchain>-gcc` (v1.x — o `setup.sh` extrai
  em `gnu/` e `sdk_gnu_toolchains` lista os nomes) ou na raiz
  (`<toolchain>/`, o layout 0.16/0.17 e os links de "bisectability" que o
  `setup.sh -c` cria); caminho canônico conta uma vez. Lido no
  `scripts/template_setup_posix` e no `cmake/Zephyr-sdkConfig.cmake` do
  sdk-ng em 2026-09-17. A proposta é **uma** toolchain: `arm-zephyr-eabi`
  quando há várias (o Cortex-M é o caso comum), **dita** na `evidence` com a
  lista das outras e a instrução "para outra, importe a pasta dela
  (`gnu/<toolchain>`)" — que cai no `toolchain-dir`. `targetTriple` é o nome
  da toolchain (`arm-zephyr-eabi`, `riscv64-zephyr-elf`,
  `xtensa-espressif_esp32_zephyr-elf`); `sysroot` é o que o gcc declara (a
  libc da toolchain — picolibc/newlib —, não um sistema de destino); `gdb` o
  da toolchain; sem `toolchainFile` (um projeto Zephyr compila pelo `west
  build -b <board>`, que acha o SDK por `ZEPHYR_SDK_INSTALL_DIR` ou pelo
  registro CMake — o `hint` diz isso; nenhum board é inventado). SDK sem
  toolchain nenhuma → proposta vazia com o passo (`./setup.sh -t
  arm-zephyr-eabi`). **Não medido contra um SDK real** (não há nesta máquina;
  a fixture é a forma do `setup.sh`).
- **toolchain-dir** — uma pasta com `bin/<triple>-gcc` (o tarball da Arm, o
  que a IDE instalou): o sysroot é o que o próprio gcc declara
  (`-print-sysroot`, se ele roda aqui e a pasta existe com conteúdo), senão as
  convenções `<triple>/libc` (Arm) e `<triple>/sysroot`; sem arquivo de CMake —
  o `CMAKE_SYSTEM_NAME` sai do triple, como sempre.

Caminho relativo → `INVALID_PARAMS`; nada reconhecido → `INVALID_REQUEST`
dizendo o que se procurou. **Não medido contra um SDK Yocto ou uma árvore
Buildroot reais** — não há nenhum nesta máquina (42 §8 item 5): o que está
provado é o contrato (um `environment-setup` que exporta as variáveis
documentadas; uma `output/host` com a forma documentada), e a exercitação do
gate lê a raiz do projeto como sysroot "vazia".

**O `toolchainFile` do kit** vira `-DCMAKE_TOOLCHAIN_FILE=<arquivo>` no
configure **só quando o preset não declara o seu** — o do `CMakePresets.json`
vence (dois arquivos brigariam), e a tela já dizia isso. Com o arquivo do
Buildroot/Yocto no kit, o configure é o do SDK: compiladores e sysroot vêm
dele. A UI (`EmbeddedKitImportView`): um campo de caminho, "Ler sysroot",
"Importar kit", a proposta em linhas e "Aplicar proposta ao kit" — um `setKit`
com sysroot, alvo e arquivo; o chip fica. O painel, ao aplicar chip/alvo/
sysroot, **preserva** o arquivo (um sinal não carrega `undefined`: o
controller substitui pelo atual).

### Settings (`settings.get` / `settings.set`)

Implementado no protocolo `0.36.0` (fatia M4.1 de `DocsPrivate/diario/18`). Dois níveis:
GLOBAL (`$XDG_CONFIG_HOME` ou `~/.config`, então `kinein-vectis/
settings.json`, vale para todos os workspaces) e por-WORKSPACE
(`.kinein/settings.json`, sobrepõe o global campo a campo). Ambos com
`schemaVersion` (arquivo inválido/schema desconhecido = vazio, nunca
quebra). O efetivo = `default ← global ← workspace`.

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
                 outlineCollapsed?: bool, railExpanded?: bool (0.128.0) }
                                          (campos ausentes = não setados)
EffectiveSettings { formatOnSave, editorFontSize, autoClosePairs, autoSave,
                    rigorProfile, explorerWidth, contextWidth,
                    assistantTerminalWidth,
                    bottomPanelHeight, outlineWidth, outlineCollapsed,
                    outlineCollapsed }
SettingsResult { settings: EffectiveSettings, global: SettingsValues,
                 workspace: SettingsValues }
```

- `settings.get` NÃO exige workspace (o global existe sempre; sem
  workspace, `workspace` vem vazio). Padrão de mutação do repo: `set`
  responde o estado COMPLETO novo.
- `settings.set` faz MERGE parcial de `values` no escopo (campo ausente
  mantém o valor gravado; reverter/limpar override é pós-v1). Defaults:
  `formatOnSave=false`, `editorFontSize=14`, `autoClosePairs=true`,
  `autoSave=true` (0.123.0, decisão do autor na Etapa 2), `rigorProfile="strict"`, larguras `280/360/640/220` (Project, seletor KV,
  terminal KV ativo e Estrutura), painel inferior `260` e Estrutura expandida.
- Erros: `NO_WORKSPACE` (set `scope=workspace` sem workspace);
  `INVALID_PARAMS` (`editorFontSize` fora de 8..=40, ou `rigorProfile`
  fora do enum; largura/altura de painel fora dos limites, inclusive terminal
  KV ativo fora de 300..=720); `INTERNAL_ERROR`
  (falha de escrita).
- Consumidores atuais: `editorFontSize` → `Theme.fontSizeEditor`;
  `autoClosePairs` → auto-close da E1; `formatOnSave` → Ctrl+S formata e
  então salva; `autoSave` → o `EditorPersistenceController` salva o buffer
  sujo sozinho (2 s de pausa; ao trocar de aba; ao perder o foco) pelo
  MESMO caminho do Ctrl+S (`fs.write` com `expectedContent` — o disco
  mudado por fora continua recusado); o rascunho do `seguranca/23` segue
  gravando aos 1,5 s como rede entre um salvar e outro; `rigorProfile` (M4.5) → flags de `quality.run`/`build.run`
  NO PROJETO DO USUÁRIO (clippy pedantic/nursery + `-D warnings` no
  strict; clippy default no balanced; só `clippy::correctness` +
  build sem `RUSTFLAGS` no relaxed). NUNCA regula o gate do repo Kinein.
  As dimensões persistem o layout que o usuário redimensionou;
  `outlineCollapsed` persiste o recolhimento da Estrutura.
  `diffBase` (head|index) fica para uma micro-fatia futura (precisa de
  `base` no `git.fileDiff`).

### Rascunhos / autosave (`draft.save` / `draft.clear`)

Rede de segurança contra perda de dado (protocolo `0.40.0`, fatia S1 de
`DocsPublic/seguranca/23`). O core persiste em **SQLite** (`.kinein/kinein.db`, WAL +
escrita atômica por transação) o buffer NÃO SALVO de arquivos sujos. Se a
UI cair (ou power loss), no próximo `workspace.open` as edições voltam.
Complementa a escrita atômica de `fs.write` (temp + `fsync` + `rename`, que
elimina o arquivo truncado/zerado em crash).

```text
draft.save { path, content } → { savedAt }   (autosave de buffer sujo)
draft.clear { path }          → { ok: true }  (save/close limpo)
workspace.open ... → { ..., drafts?: [{ path, content, savedAt }] }
```

- `path` é absoluto e confinado à raiz (como `fs.*`); a chave é o caminho
  absoluto. A UI autosalva com debounce (~1.5s) só quando o buffer está
  modificado; ao fechar a aba manda `draft.clear`.
- `fs.write` **também** limpa o rascunho do path salvo (o arquivo em disco
  passou a ser a verdade). Então rascunho **só sobrevive a um CRASH**.
- Recuperação: embutida na resposta de `workspace.open` (`drafts`), já
  FILTRADA — rascunho igual ao disco é obsoleto e apagado; só volta o que
  DIFERE. A UI abre a aba, sobrepõe o buffer e a marca MODIFICADA (o
  savedContent segue o disco: salvar/reverter continuam corretos).
- Erros: `NO_WORKSPACE` (sem workspace); `INVALID_PARAMS` (path fora da
  raiz / inexistente); `INTERNAL_ERROR` (store indisponível/falha).

### Debug (`debug.*` — sessao DAP via lldb-dap)

Implementado no protocolo `0.28.0` (fatia M2.5a de `DocsPrivate/diario/18`). O core
orquestra o `lldb-dap` (pacote `lldb`) pelo Debug Adapter Protocol; a UI
nunca fala DAP — recebe eventos `event.debug.*` ja mastigados. Uma sessao
por workspace.

- `debug.start { program?, connect? }` → `{ program, attached }`. Sem ambos, resolve o
  alvo "Automatico" espelhando o run: cargo → unico executavel no topo de
  `target/debug`; cmake → unico executavel de `.kinein/build`; **Python
  (`0.101.0`) → o mesmo ponto de entrada do Executar** (`main.py`/`app.py`/
  `__main__.py` na raiz, ou o script de `[project.scripts]` instalado; um
  pacote com `__main__.py` vira `module: "x"` desde `0.107.0`); zero ou
  varios candidatos → erro claro com a acao a tomar. NAO compila antes
  (build e acao explicita, Ctrl+F9). Erros: `TOOL_NOT_FOUND` (lldb-dap
  ausente), `INVALID_REQUEST` (sem alvo/sessao ja viva), `INVALID_PARAMS`
  (program inexistente), `INTERNAL_ERROR` (falha do adapter).

  **Alvo `.py` — ou um MÓDULO (`0.101.0`/`0.107.0`, cadeia Python,
  2026-09-13).** Seja qual for o tipo do workspace (um CMake com
  `tools/gera.py` inclusive), um `program` terminado em `.py` não passa pelo
  kit; e num workspace Python cujo ponto de entrada é um pacote com
  `__main__.py`, o alvo automático é o módulo — o `launch` leva `module:
  "pacote"` em vez de `program` (medido no debugpy 1.8.21: para no
  breakpoint dentro do pacote; `DebugStartResult.program` = `-m pacote`;
  configurações de servidor/chip do kit nativo não são herdadas): o adaptador é o
  **debugpy do interpretador do projeto** (`python/env.rs`, a precedência do
  `29` §4.1) — `<interpretador> -m debugpy.adapter`, DAP por stdin/stdout,
  `launch { program, cwd }` como o desktop. O `console` fica de fora de
  propósito: o `initialize` desta sessão não declara
  `supportsRunInTerminalRequest`, e o debugpy cai sozinho no
  `internalConsole` (stdout/stderr do programa chegam como `event.debug.
  output`) — medido no 1.8.21 com e sem o campo. Antes de subir, o core
  roda `<interpretador> -I -c "import debugpy"` (isolado: o que o adaptador
  vai ver; prazo de 10 s): sem interpretador → `INVALID_REQUEST` orientando
  a criar o ambiente; sem o módulo → `TOOL_NOT_FOUND` dizendo o passo para
  instalar NELE (`uv add --dev debugpy` ou `.venv/bin/python -m pip install
  debugpy`) — sem isso o adaptador morreria no primeiro request e a falha
  seria um timeout do `initialize`, longe da causa. O debugpy **não sai
  sozinho** no `disconnect` (medido): é o `Drop` da sessão que o mata, como
  já fazia com os outros. Provado ponta a ponta pelo core real em
  `scripts/verificar-python-debug.sh` (breakpoint em `app.py:2`, `soma` no
  topo da pilha, `a=2 b=3` em Locals — não Globals —, `a + b` = `5`,
  `resultado 5` por `event.debug.output`, `exitCode 0`, adaptador morto;
  1,4 s).
  **Attach Python (`0.109.0`).** `connect: { host: "127.0.0.1", port: 5678 }`
  conecta por TCP ao adaptador já criado por `debugpy --listen`/`debugpy.listen`.
  `program` e `connect` juntos são `INVALID_PARAMS`; host vazio ou com espaços,
  controles/URL e porta fora de 1–65535 também. Campos desconhecidos são recusados.
  Não resolve executável automático nem exige interpretador local. O mesmo
  handshake envia `attach { connect }`, reaplica breakpoints e configurationDone.
  Resultado/evento `attached: true`; `program` exibe `debugpy host:porta`.
  Launch devolve `attached: false`. Fechar/trocar workspace limpa a sessão e
  seus breakpoints. Sem SSH, attach por PID ou mapeamento de caminhos nesta fatia.

- `debug.setBreakpoints { file, lines: [int] }` →
  `{ breakpoints: [{ line, verified }] }`. Conjunto COMPLETO por arquivo
  (lines vazio limpa); `file` confinado ao workspace. Sem sessao viva o
  conjunto e guardado (`verified: false`) e replayado no proximo launch;
  com sessao, o adapter responde o que de fato amarrou.
- `debug.continue` / `debug.next` / `debug.stepIn` / `debug.stepOut`
  exigem processo pausado; `debug.pause` pausa o processo em execucao;
  `debug.stop` desconecta educadamente: no attach Python envia
  `terminateDebuggee: false` e fecha o socket, preservando o processo externo;
  em launch encerra o processo e o adaptador filho da IDE. Todos respondem
  `{ status: "ok" }`; exigem apenas o manager (como `run.stop`).

Inspeção (protocolo `0.29.0`, fatia M2.5c), sempre da thread pausada
(`INVALID_REQUEST` fora disso):

- `debug.stackTrace {}` → `{ frames: [{ id, name, file?, line? }] }` —
  topo primeiro, até 20 frames.
- `debug.variables { frameId }` → `{ frameId, variables }` — o core
  resolve os scopes DAP internamente e devolve as variáveis do primeiro
  escopo não-caro (Locals); Globals/Registers ficam pós-M2.
- `debug.variables { ref }` → `{ ref, variables }` — expande uma variável
  estruturada. Variável: `{ name, value, type?, ref }` (`ref` 0 = folha,
  > 0 = expansível). Exatamente um de `frameId`/`ref` → senão
  `INVALID_PARAMS`. A resposta ECOA a chave pedida para a UI correlacionar
  (mesmo padrão do `format.text`).
- `debug.scopes { frameId }` → `{ frameId, scopes: [{ name, ref, expensive }] }`
  (`0.118.0`, P3) — os escopos INTEIROS do frame, na ordem do adaptador:
  `Locals`, `Registers` (probe-rs e GDB), e `Peripherals` quando o kit tem
  `svdFile` e o adaptador é o probe-rs. `expensive` é o que o adaptador diz
  (ler todos os registradores de periférico custa); cada `ref` alimenta o
  MESMO `debug.variables { ref }`. É o D2+D4 do `roadmaps/41`: o que o
  `debug.variables { frameId }` esconde ao escolher o primeiro escopo.
- `debug.readMemory { memoryReference, offset?, count }` → `{ address,
  unreadableBytes?, data? }` (`0.118.0`) — o `readMemory` do DAP, verbatim:
  `memoryReference` é um endereço (`0x3ff00000`) ou a referência que uma
  variável/frame carrega; `data` é base64, como o DAP a manda (a UI mostra
  hexadecimal). `count` 0 ou referência vazia → `INVALID_PARAMS`; sem
  sessão parada → `INVALID_REQUEST`.
- `debug.disassemble { memoryReference, offset?, instructionOffset?,
  instructionCount }` → `{ instructions: [{ address, instruction,
  instructionBytes?, symbol?, file?, line? }] }` (`0.118.0`) — o
  `disassemble` do DAP, verbatim (`location.path` vira `file`). Os dois são
  anunciados pelo `probe-rs dap-server` 0.32 e pelo `gdb -i dap` 17 desta
  máquina (`supportsReadMemoryRequest`/`supportsDisassembleRequest`,
  medidos no `initialize` em 2026-09-17).
- `debug.evaluate { expression, frameId? }` → `{ expression, result, typeName?,
  reference }` (protocolo `0.66.0`) — avalia uma expressão (watch) no frame
  pedido. **Sem `frameId`, o core usa o frame do TOPO** da thread parada: é o
  que a UI quer quando o usuário digita um watch sem ter escolhido um frame.
  `reference > 0` significa expansível e alimenta o MESMO `debug.variables
  { ref }` acima — um watch é uma variável avaliada sob demanda, não uma
  árvore paralela. A resposta ECOA `expression`, e **o erro também a leva em
  `details`**: sem isso a UI não sabe qual watch falhou e marcaria todos.
  Expressão vazia ou só de espaços é `INVALID_PARAMS` no core, antes de chegar
  ao adapter — a mensagem do lldb para string vazia não ajuda ninguém.

**Mudança de contrato em `debug.setBreakpoints` (protocolo `0.66.0`).** O campo
`lines: [u32]` virou `breakpoints: [{ line, condition?, hitCondition? }]`.

```text
ANTES  { file, lines: [3, 7] }
AGORA  { file, breakpoints: [{ "line": 3 },
                             { "line": 7, "condition": "i == 42" }] }
```

`condition` e `hitCondition` mapeiam 1:1 nos campos de mesmo nome do
`SourceBreakpoint` do DAP — *"the breakpoint stops only when this evaluates to
true"* e *"how many times the breakpoint must be hit before stopping"* —, e o
`lldb-dap` anuncia `supportsConditionalBreakpoints` e
`supportsHitConditionalBreakpoints`. **Campo ausente não vira `null` no wire:**
mandar `"condition": null` (ou `"  "`) é pedir para um adapter tratar como
expressão vazia e o breakpoint nunca parar; o core descarta condição em branco
antes de montar o argumento. Array paralelo de condições foi recusado de
propósito — é a forma de linha e condição saírem de sincronia em silêncio.

```text
event.debug.started   { program, attached }
event.debug.output    { category, line, channel?, channelName? }
                                           (stdout|stderr|console|adapter|rtt)
event.debug.stopped   { reason, file?, line?, threadId }  (file/line podem
                        vir null; o core ja enriquece com o frame do topo)
event.debug.continued {}                   (um por retomada, deduplicado)
event.debug.finished  { exitCode? }        (exatamente um por sessao)
```

**`category: "rtt"` (`0.118.0`, D1 do `roadmaps/41`)** é o console da placa
pelo RTT/defmt do probe-rs: cada canal que o adaptador anuncia
(`probe-rs-rtt-channel-config { channelNumber, channelName, dataFormat }`)
vira uma linha `canal RTT <n> \`<nome>\` (<formato>) aberto` com `channel` e
`channelName`, e o core responde na hora o request `rttWindowOpened
{ channelNumber, windowIsOpen: true }` — sem ele o probe-rs NÃO lê o canal
("will delay polling RTT channels until the data window has opened",
`dap_types.rs` do 0.32.0). Os dados (`probe-rs-rtt-data { channelNumber,
data }`) saem linha a linha com `channel`; o defmt já chega decodificado
pelo adaptador quando o canal foi declarado assim (`rttChannelFormats` — hoje
o `launch` só liga `rttEnabled` e deixa o probe-rs detectar os canais). O
`probe-rs-show-message` (o que o adaptador mostraria numa caixa de diálogo)
vira uma linha `console` `probe-rs [<severidade>]: <mensagem>`.

**`category: "adapter"` (`0.111.0`)** é o stderr do PRÓPRIO adaptador que a
IDE criou (`lldb-dap`, `probe-rs dap-server`, `gdb -i dap`, `python -m
debugpy.adapter`), uma linha por evento — as outras três categorias vêm do
evento DAP `output` e falam do programa. Num attach TCP não há `adapter`: o
processo é de outro dono. As últimas 64 linhas ficam na sessão e entram na
mensagem do `debug.start` que falha no handshake (sob `--- stderr do
adaptador ---`): "o adapter nao respondeu a `initialize`" agora vem com o
`ImportError` que o causou. O servidor de debug do kit (`debugServer`) tem
a mesma cauda, sem evento por linha (um QEMU é verboso): ela entra em "o
servidor de debug saiu (…) antes de abrir …" e "nao abriu … em N s". A UI
pinta `adapter` como `stderr`.

### Git (`git.status` e operações diárias)

Implementado no protocolo `0.30.0` (fatia M3.1 de `DocsPrivate/diario/18`). Orquestra o
binario `git` com saida ESTAVEL (`status --porcelain=v2 --branch
--untracked-files=all -z`); deteccao de repo por exit code
(`rev-parse --is-inside-work-tree`), nunca por mensagem (pode vir
localizada). Sincrono e stateless: a UI decide quando consultar
(open/save/fs-ops/manual).

```text
git.status {} → { repo: bool,
                  branch?, detached?, shortSha?,
                  upstream?, ahead?, behind?,
                  entries: [{ path, kind, staged }] }
kind ∈ modified|added|deleted|renamed|untracked|conflicted (o core
consolida o par XY do porcelain; a UI nunca decodifica porcelain).
```

- Paths das entries são relativos ao ROOT do workspace (convertidos do
  toplevel via `rev-parse --show-prefix`); entries fora do workspace e os
  metadados `.kinein/` são filtrados.
- Workspace sem git → `{ repo: false, entries: [] }` (nunca é erro).
  Erros reais: `NO_WORKSPACE`, `TOOL_NOT_FOUND` (git ausente),
  `INTERNAL_ERROR` (git falhou).

`git.fileDiff { path }` (protocolo `0.31.0`, fatia M3.2) → diff do
arquivo contra o HEAD, em duas granularidades na mesma resposta:

```text
git.fileDiff { path } → { path (canônico, ecoado p/ correlação),
                          repo, tracked,
                          hunks: [{ kind: added|modified|removed,
                                    startLine, lineCount }],
                          text }
```

- `hunks` vem de `--unified=0` (ranges exatos para a gutter; `removed` é
  ancorado na linha seguinte à remoção, lineCount 1); `text` vem de
  `--unified=3` (visão de diff). A UI nunca parseia diff.
- Untracked → `tracked: false` + um hunk `added` do arquivo inteiro e
  `text` vazio. `path` confinado ao workspace (`INVALID_PARAMS` fora).
- Diff é do arquivo EM DISCO (buffer não salvo não aparece — a gutter
  atualiza no save/troca de aba; attach de buffer é melhoria futura).

Mutações (protocolo `0.32.0`, fatia M3.3) — todas respondem o MESMO
shape do `git.status` (a UI atualiza tudo de uma vez):

```text
git.stage    { paths: [abs] } → GitStatusResult   (git add)
git.unstage  { paths: [abs] } → GitStatusResult   (git restore --staged)
git.discard  { paths: [abs] } → GitStatusResult   (DESTRUTIVO: restore
                                 p/ tracked, clean -f p/ untracked; a
                                 confirmação é responsabilidade da UI)
git.commit   { message, amend? } → GitStatusResult (commita SÓ o staged; `amend`
                                  reescreve o último commit — 0.126.0)
```

- Guarda do commit por exit code estável (`git diff --cached --quiet`):
  nada staged → `INVALID_REQUEST` com mensagem própria. `message` vazia
  e `paths` vazio/fora do root → `INVALID_PARAMS` (path deletado não
  canonicaliza: vale a checagem lexical dentro do root). Mutação em
  workspace sem git → `INVALID_REQUEST` (leitura responde `repo:false`).
  Falha real do git → `INTERNAL_ERROR` com o stderr.

Leitura de histórico (protocolo `0.33.0`, fatia M3.4) — todas com
formato estável (`--porcelain`, `%x1f`/NUL) e sem parsear saída
localizada:

```text
git.blame { path } → { path (ecoado), repo, tracked, groups: [
    { startLine, lineCount, sha, author, authorTime (epoch),
      summary, committed }] }
git.log { maxCount?, ref? } → { repo, entries: [       (ref: branch/tag, 0.127.0)
    { sha, shortSha, author, authorTime (epoch), summary,
      parents: [sha] (0.126.0; dois num merge), refs: ["HEAD -> main", "origin/main", "tag: v1"] }] }
git.commitDiff { sha } → { sha (ecoado), text (patch unificado) }
```

- `git.blame` usa `git blame --porcelain`; metadado de sha repetido é
  memoizado (o porcelain só o manda uma vez). Linha não commitada tem
  sha zerado → `committed:false` (detecção pelo sha, nunca pela string
  "Not Committed Yet"). Untracked/repo recém-init sem HEAD →
  `tracked:false` com `groups` vazio (não é erro). `path` confinado ao
  root e existente em disco → senão `INVALID_PARAMS`.
- `git.log` usa `--pretty=format:%H%x1f%h%x1f%an%x1f%at%x1f%s -z`;
  `maxCount` default 100, fora de `1..=500` → `INVALID_PARAMS`. Repo
  sem commits (sem HEAD, checado por `rev-parse --verify --quiet`) →
  `entries: []`. Fora de repo → `repo:false`.
- `git.commitDiff` usa `git show --pretty=format: --unified=3`; o `sha`
  é validado (4..=64 hex) ANTES de virar argv (barreira contra string
  arbitrária); merge trivial pode devolver `text` vazio. Fora de repo →
  `INVALID_REQUEST` (a chamada só nasce da lista do `git.log`).

Operações diárias adicionadas no protocolo `0.49.0`:

```text
git.branches {} → { repo, current?, detached, branches: [{ name, current }] }
git.checkout { branch } → GitStatusResult
git.branchCreate { name, checkout? } → GitStatusResult
git.pull {} → { jobId }
git.push {} → { jobId }
git.stash { action: "push|pop", message? } → GitStatusResult
```

- nomes de branch são validados antes de virar argumento; checkout/create e
  stash são recusados pela UI quando existem buffers sujos;
- pull/push são jobs canceláveis e publicam saída/resultado pelo Job System, e
  o desfecho sai em **`event.git.remoteFinished { operation, success, message }`**
  (documentado em 2026-08-29; o evento existia desde o `0.49.0` e era o único
  dos 32 eventos do core que não estava neste contrato). A UI usa `operation`
  para saber se foi `pull` ou `push` sem guardar o `jobId`;
- stash push inclui untracked, limita-se ao workspace e exclui `.kinein`;
  pop restaura o stash mais recente;
- todas as mutações síncronas devolvem o status inteiro para não duplicar
  estado derivado na UI.

### Jobs (`job.list` / `job.cancel`)

Fundação do Job System (ver `DocsPublic/arquitetura/ARCHITECTURE.md` §7). Um **job** é uma
operação longa que o core executa de forma assíncrona: retorna um `id` na hora,
reporta progresso por eventos e pode ser cancelado. A infraestrutura atual
registra ciclo de vida/cancelamento e já é usada por `build.run`, `quality.run`
e `test.run`.

- `job.list` → `{ jobs: [{ id, kind, title, status, progress?, canCancel, risk }] }`.
  `status`: `queued|running|cancelRequested|success|warning|failed|cancelled`;
  `risk`: `low|medium|high|dangerous`. Ordem estável de criação. O core retém
  até 100 jobs em memória, preservando jobs ativos e removendo os finalizados
  mais antigos quando o limite é excedido.
- `job.cancel { jobId }` → `{ jobId, cancelled }`. `cancelled` é `true` só quando
  o job existe, expõe cancelamento e ainda está `running`; ao aceitar o pedido,
  o core muda o status para `cancelRequested` e emite `event.job.progress`.
  `jobId` ausente é `INVALID_PARAMS`. O cancelamento é cooperativo (o trabalho
  verifica o sinal) e o estado terminal chega depois como `cancelled`.

Eventos, todos com `jobId`:

```text
event.job.created   { "id", "kind", "title", "status", "progress"?, "canCancel", "risk" }
event.job.progress  { "jobId", "status": "running|cancelRequested", "progress"?, "message"? }
event.job.output    { "jobId", "line" }
event.job.finished  { "jobId", "status": "success|warning|failed|cancelled" }
```

Regra de UX (specs): `event.job.*` atualizam status bar / tool window; não abrem
pop-up automático. Job `high`/`dangerous` exige confirmação antes de iniciar.

## Os 169 métodos roteados — a lista inteira

> **Refeita por medição em 2026-09-24**, contando os braços `"dominio.metodo"`
> dos roteadores do core com o mesmo código do `verificar-fiacao-ipc.sh`. A
> lista dizia "inteira" e tinha 149 de 167: faltavam o domínio `remote.*`
> completo, `coverage.lines/run`, `debug.disassemble/readMemory/scopes`,
> `python.stubs` e `serial.files`. Lista escrita à mão divergindo em silêncio é
> o que este documento existe para impedir — daí a regeneração.
>
> **2026-09-17:** `serial.identify` (0.112.0), `runConfig.flashProposal`
> (0.113.0) e `serial.access` (0.114.0) entraram; eram 140.

> **Era "Métodos principais implementados", e listava 66 dos 128** — sem dizer
> que era parcial, o que fazia um domínio inteiro parecer inexistente.
> Refeita em 2026-09-06 pelo comando do cabeçalho, agrupada por domínio; os
> `sim.*` saíram dela em 2026-09-12 com o domínio.

```text
build.run
build.size

cargo.check
cargo.metadata

cmake.configure
cmake.presets.list
cmake.status
cmake.targets.list

command.list

configAction.apply
configAction.list
configAction.preview

container.action
container.compose
container.images
container.list
container.open
container.status

core.ping
core.shutdown

coverage.lines
coverage.run

datasource.create
datasource.destroy
datasource.discover
datasource.introspect
datasource.list
datasource.query
datasource.remove
datasource.save
datasource.test

debug.continue
debug.disassemble
debug.evaluate
debug.next
debug.pause
debug.readMemory
debug.scopes
debug.setBreakpoints
debug.stackTrace
debug.start
debug.stepIn
debug.stepOut
debug.stop
debug.variables

draft.clear
draft.save

environment.scan

format.capabilities
format.text

fs.createDirectory
fs.createFile
fs.delete
fs.findFiles
fs.list
fs.read
fs.rename
fs.replace
fs.search
fs.write

git.blame
git.branchCreate
git.branches
git.checkout
git.commit
git.commitDiff
git.discard
git.fileDiff
git.log
git.pull
git.push
git.stage
git.stash
git.status
git.unstage

grafana.forget
grafana.get
grafana.probe
grafana.save

index.context
index.status
index.symbols

job.cancel
job.list

library.list
library.plan

lsp.applyCodeAction
lsp.codeActions
lsp.completion
lsp.definition
lsp.didChange
lsp.documentSymbols
lsp.hover
lsp.references
lsp.rename
lsp.restart
lsp.semanticTokens
lsp.switchSourceHeader
lsp.workspaceEdit.apply
lsp.workspaceEdit.cancel
lsp.workspaceSymbols

probe.list

project.model

python.createEnvironment
python.status
python.stubs

quality.run

remote.command
remote.deploy
remote.discover
remote.list
remote.open
remote.parseCommand
remote.probe
remote.remove
remote.resolve
remote.save
remote.status
remote.sync

run.capabilities
run.script
run.start
run.stop

runConfig.delete
runConfig.flashProposal
runConfig.list
runConfig.save
runConfig.setActive

serial.access
serial.files
serial.identify
serial.list
serial.monitor

settings.get
settings.set

setup.list

syntaxTree.indent
syntaxTree.update

terminal.clearScrollback
terminal.close
terminal.copySelection
terminal.input
terminal.mouse
terminal.open
terminal.resize
terminal.scroll
terminal.selectAll

test.discover
test.run

toolchain.get
toolchain.importKit
toolchain.inspectSysroot
toolchain.install
toolchain.installable
toolchain.set
toolchain.setKit

tools.detect
tools.status

workspace.browse
workspace.close
workspace.createFolder
workspace.createProject
workspace.open
workspace.recent.clear
workspace.recent.list
workspace.recent.pin
workspace.recent.remove
workspace.saveSession
workspace.status
```

## Os 50 eventos emitidos — a lista inteira

> **2026-09-17:** `event.lsp.log` (0.111.0) e `event.serial.identified`
> (0.112.0) entraram; eram 48.

> **Era "Eventos iniciais", e faltavam três** — os dois do `datasource` e o do
> `grafana`. Refeita em 2026-09-06. **Cinco não são literais no código**: eles
> nascem de `format!("event.{domain}.*")` em `handlers/build.rs`, com `domain`
> valendo `build` ou `quality`, e por isso um grep ingênuo não os encontra.

```text
event.build.diagnostic              <- so por format!
event.build.finished
event.build.output                  <- so por format!
event.build.started                 <- so por format!

event.cmake.finished
event.cmake.started

event.datasource.created
event.datasource.destroyed
event.datasource.introspected
event.datasource.queried
event.datasource.tested

event.debug.continued
event.debug.finished
event.debug.output
event.debug.started
event.debug.stopped

event.environment.finished
event.environment.started
event.container.finished

event.environment.tool

event.fs.changed
event.fs.watchError

event.git.remoteFinished

event.grafana.probed

event.index.finished
event.index.progress

event.job.created
event.job.finished
event.job.output
event.job.progress

event.lsp.diagnostics
event.lsp.documentsClosed
event.lsp.log
event.lsp.restarted
event.lsp.status

event.project.changed
event.python.finished
event.quality.diagnostic
event.quality.finished
event.quality.output                <- so por format!
event.quality.started               <- so por format!

event.serial.identified

event.terminal.closed
event.terminal.render

event.test.case
event.test.discovered
event.test.finished
event.test.output
event.test.started

event.toolchain.installed
```

## Regras

1. Todo request deve ter resposta.
2. Eventos não têm `id`.
3. Erros devem ter `code`, `message` e `details`.
4. O protocolo deve ser versionado.
5. Toda alteração no protocolo exige atualização de docs e schemas.
6. A UI não deve interpretar logs brutos quando houver evento estruturado.

## `library.*` — o catálogo curado de bibliotecas C/C++

Domínio novo no protocolo `0.68.0` (etapa 19 do `roadmaps/35`). **A IDE endossa
o que oferece**: cada entrada carrega licença verificada, versão pinada e a
frase que explica o que a biblioteca faz.

```text
library.list {}                  -> { libraries: [LibraryInfo] }
library.plan { id, target }      -> LibraryPlan
```

**Nenhum dos dois exige workspace aberto**, e o domínio é **stateless** — o
catálogo é estático e a disponibilidade se lê do filesystem. Nada depende do
estado do `Core`, e o handler não recebe `self`.

`status` é `detected` | `notDetected`. **`notDetected` não é "não existe":** a
detecção olha os diretórios de config package conhecidos, e o usuário pode ter
um `CMAKE_PREFIX_PATH` próprio. Por isso, quando nada foi achado, o
`LibraryPlan` devolve `searchedPaths` — *"não achei, e olhei aqui"* é acionável;
*"não achei"* manda adivinhar.

O `LibraryInfo` carrega `standardLineage` quando existe o sinal FORTE — a
biblioteca ter sido adotada no padrão ISO, ou passado pela revisão formal do
Boost. **Não há certificação oficial de biblioteca C++:** o WG21 padroniza a
linguagem e a biblioteca padrão, e a Standard C++ Foundation declara que seu
objetivo é reduzir barreiras para *adotar* bibliotecas no próprio Standard — não
certificar as de terceiros. O campo registra o sinal que existe de verdade em
vez de inventar um selo que ninguém emite; hoje só `fmt` o tem
(virou `std::format` no C++20).

**O plano não escreve nada.** Ele devolve passos que nomeiam a Configuration
Action que os executaria, e quem escreve continua sendo o domínio
`configaction`, com o preview e o consentimento que ele já tem. Dois escritores
do mesmo arquivo de build é a forma de eles divergirem em silêncio — e o
critério de aceite do corte é mecânico: `grep -c "CMakeLists"
crates/kinein-core/src/library/` tem que voltar **0**, inclusive em comentário.

**`find_package` ou `FetchContent` sai da MEDIÇÃO, não de um campo do
catálogo.** Se o config package está no sistema, usa-se ele; baixar e compilar o
que já está instalado é desperdício que o usuário paga em tempo de build. A
primeira versão tinha um enum com três variantes para isso e as sete entradas
auditadas saíram todas iguais — os lints reprovaram as variantes nunca
construídas, e estavam certos.

## `probe.*` — as sondas de debug conectadas

Domínio novo no protocolo `0.71.0` (etapa 24 do `roadmaps/35`). É o **"plug"**
do plug and play: a IDE detecta em vez de pedir configuração.

```text
probe.list {} -> { probes, toolAvailable, rawOutput, hint? }
```

Executa `<adaptador> list`, onde o adaptador vem do kit (papel `debugAdapter`).
**Exige workspace**, porque sem kit não há qual ferramenta perguntar — e cair no
`PATH` em silêncio esconderia do usuário quem respondeu.

**O `rawOutput` vai SEMPRE, não só no erro.** O formato do `probe-rs list` foi
levantado de fontes da comunidade e **não** verificado contra o binário
instalado (ele não está nesta máquina, medido em 2026-09-03). Um parser rígido
contra formato não verificado quebraria na primeira mudança de espaçamento — e
quebraria dizendo *"nenhuma sonda"*, que é a pior mentira possível aqui.

Por isso: linha que casa vira sonda, linha que não casa é **ignorada**, e a
saída crua volta junto. *"Não entendi o que a ferramenta respondeu, e aqui está
o que ela disse"* é acionável; *"nenhuma sonda"* com uma plugada viola o item 4
do plug and play (`roadmaps/35` §5.1).

**A `hint` é onde o item 4 vira código.** O caso mais importante é o de **udev**:

```text
permissao negada  -> "falta regra de udev; rodar a IDE como root NAO e a solucao"
resposta vazia    -> "sonda desconectada, ou cabo de dados trocado por um de carga"
nao reconhecida   -> "o formato mudou e o parser precisa acompanhar"
```

Sem regra de udev a ferramenta roda, não acha nada, e o usuário conclui que a
placa está com defeito. **Plug and play morre exatamente aí**, e é a lacuna que
`integracoes/36` §5 já tinha nomeado como a mais subestimada.
## `container.*` — Docker e Podman como domínio nativo

Domínio novo no protocolo `0.92.0` (2026-09-12). A decisão é de 2026-07-17
(`roadmaps/28` §0: *"vão ser cidadãos nativos"*), e as invariantes registradas
lá em §4 são o desenho: **a UI nunca chama `docker`**, **toda ação é Job
cancelável**, **permissão é visível**, e **Podman é motor de primeira** — nesta
máquina `docker` é o shim `podman-docker`, medido em 2026-09-12.

```text
container.status  {}                       -> ContainerStatus
container.list    { all? = true }          -> { containers, engine, rawOutput, hint? }
container.images  {}                       -> { images, engine, rawOutput, hint? }
container.action  { id, action }           -> { jobId }            (job; start|stop|restart|remove)
container.open    { id, mode }             -> { id, command }      (aba de terminal; logs|shell)
container.compose { action, file? }        -> { jobId }            (job; up|down; exige workspace
                                                                     E arquivo de compose, ou `file`)

ContainerStatus   engine? (docker|podman), binary?, version?, emulated, rootless?,
                  socket?, reachable, compose?, composeFile?, hint?, rawOutput
ContainerInfo     id, names[], image, state, status, ports[], created
ImageInfo         id, repository, tag, size, created

event.container.finished { jobId, action, target, ok, message }
```

**A detecção pergunta ao binário, não ao nome.** `docker --version` do shim
escreve *"Emulate Docker CLI using podman"* em stderr; tratar isso como Docker
Engine erraria o formato de `ps` (o Podman devolve um **array** em `--format
json`; o Docker, **um objeto por linha** em `--format '{{json .}}'`) e o
diagnóstico (não há daemon nem grupo `docker` num Podman rootless). Quando o
`docker` é o shim, o core prefere o `podman` real e diz `emulated: true`.

**O parser aceita as duas formas e devolve a saída crua sempre** — a mesma
regra do `probe.list`. A forma do Docker foi lida da documentação e **não**
verificada contra um Docker Engine real (esta máquina não tem um); a do Podman
5.8.4 foi medida, inclusive o detalhe de que `images --format json` não traz
`repository`/`tag` e sim `Names ["repo:tag"]`.

**`status` é a tela de "ativar a ferramenta".** Sem motor: o passo de
instalação da distro. Motor que não responde: `permission denied` no socket →
grupo `docker` e relogar; daemon parado → `systemctl enable --now docker`;
Podman → a saída crua e o `podman.socket` do usuário. A IDE **imprime** o
comando; nunca roda `sudo`.

**`action` e `compose` são jobs** (`JobRisk::Medium`; `remove` é `High`): cada
linha do motor vai para `event.job.output`, cancelar mata o filho, e o
`event.container.finished` leva as últimas linhas como motivo. `rm` vai **sem
`-f`**: remover o que roda é dois gestos (parar, remover), de propósito.
`compose up` é `-d` — um job que nunca termina não é job; a saída viva mora
na aba de logs.

**O compose é do PROJETO, e o core diz se ele existe (`0.108.0`,
2026-09-13).** A ferramenta é da máquina (`compose`); o arquivo é do workspace
aberto: `composeFile` é o primeiro dos nomes que a ferramenta procura sozinha
na raiz (`compose.yaml`, `compose.yml`, `podman-compose.*`,
`docker-compose.yml|yaml`, `container-compose.*` — a ordem do podman-compose
1.6.0, lida em `COMPOSE_DEFAULT_LS`; os `*.override.*` não contam porque
sozinhos não sobem nada), ausente sem workspace ou quando o projeto não tem
nenhum. `container.compose` sem `file` num projeto sem arquivo **recusa com
`INVALID_PARAMS` antes de subir um job** — medido: o podman-compose sai com
255 e *"no compose.yaml, docker-compose.yml or container-compose.yml file
found"*, e um job que só pode falhar não é job. Na UI, "compose up"/"compose
down" só acendem com motor respondendo + ferramenta + arquivo, e a linha do
motor diz o que falta ("abra um projeto" / "o projeto não tem compose.yaml").
A mesma tarde corrigiu a classe visual que escondia isso: um botão primário
DESLIGADO vestia o âmbar a 72% de opacidade e continuava a coisa mais chamativa
do painel — o clique que não fazia nada lia-se como "o Docker não funciona";
agora desligado é um botão comum e apagado (`KvButton`/`KvIconButton`,
provado por `tst_kvbutton_states.qml` e `tst_container_panel.qml`).

**`open` reaproveita o terminal.** `logs -f --tail 200 <id>` e `exec -it <id>
/bin/sh` sobem pelo mesmo `open_command` do domínio `terminal` e voltam como
uma sessão de terminal (`terminal.input`/`resize`/`close` a reconhecem). O
shell dentro do container é a semente do *contexto remoto* do `roadmaps/28`
§4 — o que falta para *dev containers* é o path mapping e o ciclo de vida,
não o transporte.

## `python.*` — o ambiente do projeto Python

Domínio novo no protocolo `0.98.0` (fatia 1 da cadeia Python, 2026-09-12).
O interpretador é o `compile_commands.json` do Python: quem o resolve é
`python::env` (a precedência do `roadmaps/29` §4.1), e é o MESMO que o
`index.context` mostra; as fatias seguintes (basedpyright, run, pytest,
debugpy) leem daqui.

```text
python.status {}                       -> PythonStatus       (exige workspace)
python.createEnvironment { tool? }     -> { jobId }          (job; tool = uv | venv;
                                                              omitido = uv se houver,
                                                              senao venv)
python.stubs { port?, board? }         -> { jobId, package, command, target }
                                                             (job; 0.116.0)
event.python.finished { jobId, success, tool, command, path }
event.python.stubs    { jobId, success, package, command, target }

PythonStatus   interpreter? (PythonEnv: interpreter, version?, origin, warning?),
               nativeModule? (PythonNativeModule: kind, tool, evidence[], buildHint;
               0.102.0),
               hasEnvironment (origin != sistema), environmentTool? (uv|venv:
               o que a IDE usaria), uv? (caminho), projectFiles[] (pyproject.toml,
               requirements.txt, setup.py, uv.lock, poetry.lock, Pipfile),
               hint? (o que falta, com o remedio),
               stubsPath? (<root>/typings quando tem um .pyi; 0.116.0),
               stubsSuggested? (o pacote que o chip do kit/identidade sugere)
```

**`python.stubs` são os STUBS DA PLACA (C4 do `roadmaps/41` bloco C,
`0.116.0`):** o `import machine` completa e o basedpyright para de dizer
que o módulo não existe. Fonte (micropython-stubs.readthedocs.io, "Install
the micropython-stubs", 2026-09-17): `pip install -U
micropython-<port>[-<board>]-stubs --no-user --target ./typings`; aqui o
instalador é o `uv` detectado (`uv pip install -U --link-mode=copy --target
<root>/typings <pacote>`; medido nesta máquina: 2 pacotes em 7 ms) ou, sem
uv, o `pip` do interpretador do projeto. `port`/`board` ausentes = o modelo
do projeto decide pelo chip do kit/identidade, pela lista publicada de
pacotes (`esp32c3` → `micropython-esp32-esp32_generic_c3-stubs`; família
`rp2040` sem chip → `micropython-rp2-stubs`, porque Pico e Pico W têm stubs
diferentes); sem evidência, `INVALID_REQUEST` dizendo o que fixar. `success`
exige exit 0 E um `.pyi` em `typings/`. Ao terminar com sucesso, o
basedpyright é reconfigurado com `basedpyright.analysis.stubPath =
<root>/typings` e `diagnosticSeverityOverrides.reportMissingModuleSource =
none` (a chave do manual do basedpyright e o `pyproject` de exemplo dos
micropython-stubs) e reinicia se está vivo. Na UI: faixa INFORMATIVA de
saúde num projeto MicroPython com sugestão e sem `typings/`, botão
**Instalar stubs**.

**As regras, ditas.** `success` só é `true` quando `.venv/bin/python` existe
depois de a ferramenta sair com 0 — uma ferramenta que "termina bem" sem
criar o interpretador não vira ambiente anunciado. O comando é o da fonte
oficial, mostrado no job (`$ uv venv .venv`), nunca escondido. `uv` sem
`python3` no PATH basta (o uv baixa um Python; docs do uv). Sem uv e sem
python3 o pedido é recusado com o motivo — e o painel de instalação tem os
guias oficiais de `pipx`, `uv`, `ruff` e `basedpyright` (fonte e data em
cada um; família `any` quando a fonte é agnóstica de distro). Na UI: a faixa
de saúde do projeto ganha "o projeto usa o Python do SISTEMA: crie um
ambiente" com o botão **Criar .venv com uv** (ou `python3 -m venv`); um
clique, o job aparece, o status é perguntado de novo. Só em workspace cujo
`buildSystems` inclui `python` — um projeto Cargo/CMake com `pyproject.toml`
dentro conta.

Exercitação: o projeto de exercitação ganhou um `pyproject.toml`; o gate cria
o `.venv` com a ferramenta REAL desta máquina (`python3 -m venv`, 2026-09-12)
e vê o `python.status` passar de `sistema` para `.venv`.

**`nativeModule` (`0.102.0`, fatia 5).** A ponte entre as duas metades da
IDE: um projeto Python com extensão em C++/Rust é TAMBÉM um projeto
CMake/Cargo (o índice e o clangd/rust-analyzer já o leem), mas quem o instala
no `.venv` é o build do Python. Por evidência nos arquivos da raiz —
`pyproject.toml` (`[build-system] requires` com maturin, scikit-build-core,
setuptools-rust, pybind11 ou nanobind; `[tool.maturin]`), `Cargo.toml`
(dependência `pyo3`), `CMakeLists.txt` (`pybind11`/`nanobind`), `setup.py`
(`pybind11`/`nanobind`) — o status diz `kind` (`pybind11`, `nanobind`,
`PyO3`, ou `Rust` para maturin/setuptools-rust sem pyo3 explícito), `tool`
(`maturin`, `scikit-build-core`, `setuptools-rust`, `setuptools`), uma linha
de `evidence` por arquivo, e `buildHint` como a fonte oficial escreve:
`maturin develop` (no ambiente: `uv run maturin develop` ou
`.venv/bin/maturin develop`), `pip install -e .` para scikit-build-core e
setuptools. Projeto Python puro não tem o campo — nem com um `Cargo.toml` sem
pyo3 ou um `CMakeLists.txt` sem binding ao lado.

## `index.*` — o índice do projeto inteiro

Domínio novo no protocolo `0.94.0` (pilar 0 do `roadmaps/42`, 2026-09-12).
**Decisão do autor:** *"a IDE deve ler o projeto inteiro que for aberto… todas
as funções/arquivos/pastas"* — para Python, C, C++ e Rust.

```text
index.status  {}                        -> IndexStats   (responde tambem sem workspace: idle)
index.symbols { query, limit?, kind? }  -> { symbols[], total, state }   (exige workspace)
index.context { path }                  -> FileContext  (exige workspace; 0.95.0)

event.index.progress { files, symbols }     a cada ~200 arquivos
event.index.finished IndexStats             ao fim do build, a cada incremento e
                                            a cada recarga do contexto

IndexStats   state (idle|building|ready|failed), folders, files, sourceFiles,
             lines, bytes, symbols, functions, types, byLanguage[] { language,
             files, lines, symbols }, skipped[], elapsedMs, error?, context?
ContextSummary cdbDirectory?, cdbEntries, cdbStale, cdbStaleBecause?,
             cargoPackages, cargoTargets, cmakeTargets (0.96.0),
             pythonInterpreter?, pythonOrigin?
IndexSymbol  name, kind, path (relativo a raiz), language, line, endLine,
             container?
FileContext  path (absoluto), language (c|cpp|rust|python|other), unit?, crate?,
             python?, targets[] (0.96.0: os targets do CMake que compilam ou
             listam o arquivo), source?, hint?
CompileUnit  compiler, directory, standard?, includes[] (absolutos), defines[],
             output?, arguments[]
CargoUnit    package, target, kind (lib|bin|test|bench|example|custom-build…),
             edition, manifest, srcPath, features[]
PythonEnv    interpreter, version?, origin (VIRTUAL_ENV|.venv|venv|env|poetry|
             sistema), warning?
```

**O contexto de compilador (0.95.0).** O índice diz *o que há* em cada
arquivo; o contexto diz *com que* ele é compilado — e é carregado no mesmo
job, logo depois dos arquivos:

- **C/C++:** a `compile_commands.json` que o `cdb::status` encontra
  (`.kinein/build`, raiz, `build`, `builddir`, `out/build`), nas **duas
  formas** do padrão do clang (`arguments[]` e `command` dividido como um
  shell: aspas agrupam, `\` escapa); `file` relativo é resolvido contra o
  `directory` e a chave é o caminho **real** (`canonicalize`), porque a CDB do
  Ninja/Meson escreve `../src/a.cpp` e o editor pergunta pelo absoluto.
  `-I`/`-isystem`/`-iquote` colados ou separados viram `includes` absolutos;
  `-D` viram `defines`; `-std=` vira `standard`; `output` vence `-o`. O resto
  segue inteiro em `arguments`. Cabeçalho não tem unidade própria e a `hint`
  diz isso (o clangd deduz pela unidade que o inclui); fonte fora da CDB diz
  "nenhum alvo o compila" — ou, se a CDB envelheceu, qual arquivo a envelheceu.
- **A CDB envelhecida por SUBPASTA.** Medido em 2026-09-12 neste repositório:
  a CDB de 04/09 sem cinco fontes que o `ui/CMakeLists.txt` de 12/09
  acrescentou, e o `cdb::status` dizendo "não envelheceu" porque só compara
  com os arquivos de build da **raiz**. O contexto, com as unidades em mãos,
  sobe de cada diretório de fonte até a raiz procurando um `CMakeLists.txt`
  mais novo que a CDB; acha → `cdbStale` com `cdbStaleBecause`
  (`ui/CMakeLists.txt`) e a `hint` da unidade pede reconfigure. A subida
  **para na raiz do workspace**.
- **Rust:** `cargo metadata --format-version 1 --no-deps` com o `cargo` do kit
  efetivo (se houver `Cargo.toml` na raiz e o cargo existir — nos testes o
  PATH é vazio e nada roda). O arquivo pertence ao alvo pelo `src_path`
  **exato**; senão pelo alvo cujo diretório de `src_path` é o prefixo **mais
  longo** do arquivo; `lib` vence o empate (um `src/x.rs` pertence ao lib e
  ao bin do mesmo pacote, como o rust-analyzer prefere). O `build.rs`
  (`custom-build`) só casa exato: mora na raiz do pacote e, por prefixo, seria
  dono de tudo.
- **Python:** a precedência do `roadmaps/29` §4.1 — `$VIRTUAL_ENV` (só se o
  interpretador existir), `.venv/`, `venv/`, `env/`, `poetry.lock` + `poetry
  env info -p`, e por último o `python3` do PATH **com aviso** (instalar
  pacote nele quebra a distro). `version` é o `--version` do interpretador
  achado.
- **O modelo por alvo do `CMake` (0.96.0):** o `codemodel-v2` do build dir da
  IDE (`.kinein/build`) entra no mesmo contexto. Todo arquivo C/C++ ganha
  `targets` — cabeçalho incluso, porque o target o *lista* mesmo sem o
  compilar. E quando **não há** `compile_commands.json`, a unidade vem do
  grupo de compilação do target (includes, defines, `compileCommandFragments`
  inteiros em `arguments`, `-std=` do fragmento antes do `languageStandard`)
  com o compilador da `toolchains-v1` — ou `"(CXX do kit)"` quando ela não
  foi respondida (CMake < 3.20 ou query antiga), dito em vez de inventado;
  `source` diz `file-api codemodel-v2 (target X), sem compile_commands.json`.
  Com a CDB presente, a CDB vence a unidade e os `targets` ficam.
- **Quando recarrega:** `event.cmake.finished` (o configure reescreve a CDB) e
  um `Cargo.toml` em `event.fs.changed` (um alvo novo muda a que pacote cada
  arquivo pertence) recarregam **só o contexto**, num job curto; os arquivos
  ficam. O `event.index.finished` sai de novo com o `context` novo, e a UI
  pergunta de novo pelo arquivo ativo.

Medido neste repositório pelo core real (2026-09-12): 288 unidades na CDB de
`.kinein/build`, 4 pacotes e 6 alvos do cargo, Python do sistema
(`/usr/bin/python3`, 3.14.7). `ui/src/core_client_index.cpp` → `c++`,
`gnu++23`, 12 `-I`, 9 `-D`; `crates/kinein-core/src/index/context.rs` →
`kinein-core`/`kinein_core` (lib, 2024); `crates/kinein-core/src/main.rs` →
`kinein-core` (bin).

**Na UI:** a barra de status mostra o contexto do arquivo ativo —
`contexto: c++ · gnu++23 · 12 -I · 9 -D`, `cargo · kinein-core (lib, 2024)`,
`python · sistema · 3.14.7 ⚠` — e o detalhe (diretório, origem, dica) ao
pairar. Resposta atrasada de outro arquivo é descartada; trocar de arquivo
limpa até a resposta chegar.

**O que ele lê, e como.** Ao abrir o workspace, um job caminha a árvore
inteira com a **mesma lista de pastas ignoradas do watcher** (`.git`,
`.kinein`, `target`, `build`, `node_modules`…, para os dois verem o mesmo
projeto), conta **todo** arquivo, lê os de fonte (C/C++/Rust/Python) e extrai
as declarações com as **gramáticas Tree-sitter do editor** (a mesma query
`tags` oficial de cada gramática, sem cache — `lang/extract.rs`). Python
entrou na fundação em 2026-09-12 à tarde (`tree-sitter-python` 0.25.0, MIT;
bloco B do `roadmaps/41`): a `tags` oficial dá `function` e `class` (o método
vem como `function` com `container`). Arquivo acima de 4 MiB ou ilegível é
contado e **dito** em `skipped`, nunca sumido. Medido em 2026-09-12 à tarde
neste repositório: 1.022 arquivos, 148 pastas, 72.000 linhas, 3.939
declarações (217 delas em 18 `.py`) em ~2,1 s (build de depuração). O
"4.658" da manhã foi medido antes de o dedup do `fn` em `impl` entrar no
mesmo commit — remedido sem Python, o mesmo código dá 3.720.

**O incremento — e o índice segue o disco INTEIRO (2026-09-12 à tarde).**
Os caminhos de `event.fs.changed` são reindexados no loop principal (um
arquivo é milissegundos) e os totais reemitidos. Uma **pasta nova** é
caminhada inteira (com a mesma lista de pastas ignoradas); uma pasta apagada
leva os arquivos e as subpastas dela. E o que fecha o buraco que existia até
então: a cada `event.index.finished` o Core **registra no watcher todas as
pastas que o índice caminhou** — uma a uma, `NonRecursive`, como o ADR-0001
manda —, então um arquivo criado pelo terminal numa pasta que nenhuma tela
listou chega ao índice pelo mesmo caminho. Antes, só as pastas que a UI
expandiu eram observadas e o arquivo ficava fora até o próximo
`workspace.open`, em silêncio. Medido neste repositório: **148 inotify
watches** (as 148 pastas), contra o limite de 186.243 desta máquina
(`fs.inotify.max_user_watches`); num projeto grande o registro **para no
primeiro erro** e o relata uma vez (`event.fs.watchError`). Provado pelo
core real na exercitação: um arquivo nascido em `src/tarde/` depois do índice
pronto aparece no `index.symbols` sem reabrir nada.

**Sem duplicata e sem renomear a gramática.** A `tags` do Rust captura um `fn`
dentro de `impl` duas vezes (`function` e `method`); o índice fica com a mais
específica por (linha, nome). O que a gramática chama de `class` (a `struct`
do Rust) o índice **não** renomeia.

**A busca** ordena exato > prefixo > substring, sem diferenciar caixa, com
filtro por `kind` e `total` antes do limite. Na UI, `#nome` no Search
Everywhere pede ao índice **e** ao LSP: o índice responde primeiro e **sem
arquivo aberto**; o LSP, quando responde, substitui.

**O que este domínio NÃO é:** semântica (tipos, referências, rename continuam
no clangd/rust-analyzer). O contexto de compilador por arquivo entrou no
`0.95.0` (acima); o que ainda falta no pilar 0 está no `roadmaps/42` §3 P0.

## `project.*` — o modelo do projeto embarcado

Domínio novo no protocolo `0.93.0` (pilar 0 do `roadmaps/42`, 2026-09-12).
`workspace.open` diz *que build system* a raiz tem; este domínio diz *o que o
projeto é*.

```text
project.model {} -> ProjectModel        (exige workspace)
event.project.changed  ProjectModel     (ao abrir o workspace; ao fim de
                                         event.cmake.finished e event.build.finished)

ProjectModel   root, embedded, frameworks[], sdks[], artifacts, target, hints[]
FrameworkInfo  framework (espIdf|zephyr|picoSdk|platformIo|stm32Cube|cargoEmbedded|
               microPython|yocto|buildroot), evidence (caminho relativo do
               marcador), detail? (IDF_TARGET, PICO_BOARD, DeviceId do Cube,
               triple do cargo, MACHINE do Yocto, ambientes do PlatformIO)
SdkRequirement id, label, env?, path?, found, hint?
ProjectArtifacts elf[], bin[], hex[], uf2[], map[], flasherArgs?, partitionTable?,
               memoryX?, linkerScripts[], flashRecipe?, partitions?
FlashRecipe    chip?, flashMode?, flashSize?, flashSizeBytes?, flashFreq?, before?,
               after?, stub, files[] { offset, file (absoluto), name?, encrypted }
               — LIDO do flasher_args.json (forma do template
               components/esptool_py/flasher_args.json.in do ESP-IDF)
PartitionTable tableOffset, entries[] { name, kind, subtype, offset, size, flags? },
               end, unreadable[] — LIDA do partitions.csv com os offsets em
               branco resolvidos como o gen_esp32part.py (4 KB; app a 64 KB)
TargetModel    chip?, family?, triple?, flashEngine?, monitor?, debugAdapter?,
               evidence[]  — uma linha por dedução
```

**A detecção desce até 3 níveis e LÊ o marcador**: um `CMakeLists.txt` é
ESP-IDF se inclui o `project.cmake` do `IDF_PATH`, Zephyr se faz
`find_package(Zephyr)`, pico-sdk se chama `pico_sdk_init()`; um `main.py` é
MicroPython se importa `machine`/`board`. Pastas de saída (`build/`, `target/`,
`.pio/`, `.kinein/`) não contam. Entre dois achados do mesmo framework vence o
marcador que **decide** (`.cargo/config.toml` com o triple, não um `memory.x`
solto) e, empatando, o mais raso. Um `CMakeLists.txt` comum **não** é
embarcado; um `main.py` que não importa hardware **não** é MicroPython.

**O alvo tem evidência ou não tem alvo.** O chip do **kit** vence o do
framework (é a palavra do usuário); a família vem do chip ou do triple; os
motores vêm da família — e o ESP32 clássico (sem USB-JTAG) recebe `debugAdapter`
**ausente** com a evidência dizendo que depurar exige ESP-Prog, em vez de um
`probe-rs` que não funcionaria.

**"Achado" só vem de variável, pasta padrão ou binário.** `IDF_PATH` definido
mas apontando para pasta inexistente é `found: false`; a toolchain xtensa fora
do `PATH` mas em `~/.espressif/tools` é `found: true` com o caminho. O alvo
rustup não é medido aqui (é processo) e diz isso no `hint`. Nenhum comando roda.

**O que era só localizado passou a ser LIDO** (segunda fatia, 2026-09-12): a
receita de gravação com os caminhos resolvidos contra o `build/`, e a tabela
de partições — porque a "flash" de um ESP32 não é o `.ld`, é a partição
`app`, e é ela que o tamanho e o "Gravar" vão consumir. Linha de CSV que não
se lê vai em `unreadable`, nunca some.

**Fixtures reais e mínimas** de cada framework moram em
`scripts/fixtures/projetos/`; são elas que os testes leem.

## `serial.*` — as portas seriais USB

Domínio novo no protocolo `0.91.0` (E1 do `integracoes/38` §6, 2026-09-11). A
porta serial é o **canal que toda família bare metal compartilha** — bootloader
de ROM, console e as linhas DTR/RTS de reset — e por isso é a fundação do
monitor UART e do "Gravar".

```text
serial.list     {}                   -> { ports: [SerialPortInfo], hint? }
serial.monitor  { device, baud? }    -> { id, command, tool }   (aba de terminal; 0.92.0)
serial.identify { device, tool? }    -> { jobId, command }      (job; 0.112.0)
serial.access   { device? }          -> { channels: [AccessChannel] }   (0.114.0)
serial.files    { device, action, path?, local?, tool? } -> { jobId, command }   (job; 0.116.0)
                action: list | get | put | rm | mkdir

event.serial.files { jobId, device, action, path, command, success, error?,
                     entries?: [{ name, size, directory }], local?, raw }

AccessChannel   kind (serial|probe|modemManager), device?, ok, detail, problem?,
                fix? { steps: [{ explanation, command }], sourceUrl, checkedOn },
                distroDidIt?

event.serial.identified { jobId, device, command, success, error?,
                          identity?: SerialIdentity, target?: TargetModel, raw }
SerialIdentity  chip? (esp32c3), chipDescription?, features[], crystal?, usbMode?,
                mac?, flashManufacturer?, flashDevice?, flashSize? (4MB), flashSizeBytes?

SerialPortInfo  device, byId?, kind (usbUartBridge|usbCdc), vid, pid,
                manufacturer?, product?, serial?, interface?, driver?, family?,
                access { readableWritable, mode, group?, hint? },
                modemManager? { candidate, ignored, running }
```

**Não exige workspace**, ao contrário do `probe.list`: não há ferramenta vinda
do kit — é o sysfs desta máquina, o mesmo com ou sem projeto aberto.

**Nunca abre a porta.** Abrir um tty aciona DTR/RTS na maioria das pontes
(CP210x, CH340, FTDI) e isso **reseta a placa**; um `serial.list` que
resetasse o firmware a cada abertura do painel seria um defeito. A permissão é
medida com `access(2)`, que honra ACL — é assim que `TAG+="uaccess"` dá acesso
ao usuário da sessão sem grupo nenhum; um `stat` sozinho mentiria nesse caso.

**De onde vem cada campo:** VID:PID, `manufacturer`/`product`/`serial` e
`bInterfaceNumber` subindo de `/sys/class/tty/<n>/device` até o diretório USB
com `idVendor` (ABI documentada do kernel); `driver` do link `device/driver`;
`byId` de `/dev/serial/by-id`; `modemManager` de `udevadm info -q property`
(`ID_MM_CANDIDATE`, `ID_MM_DEVICE_IGNORE`) mais `/proc/*/comm` — e é `null`,
não `false`, quando o `udevadm` não existe. Medido em 2026-09-11: o
ModemManager examinou o ESP32 4 s depois do plug.

**`family` fala do ELO, nunca do chip.** `10c4:ea60` é *"ponte USB-UART
CP210x — o chip do outro lado não se lê pelo USB"*; `303a:1001` é *"Espressif
USB Serial/JTAG — o próprio chip"*. A identidade do chip vem **pelo canal**
(`esptool chip-id`, `probe-rs info`), que é a fatia E5. Fontes da tabela no
`integracoes/38` §5.

**`serial.monitor` é o monitor como PROCESSO numa aba de terminal** (E3 do
`integracoes/38` §6, decisão do autor em 2026-09-11: nunca código serial
nosso). A ferramenta vem do papel novo do kit, `serialMonitor` — candidatos
`tio`, `picocom`, `minicom`, `espflash`, `mpremote` nessa ordem —, com duas
regras a mais que o catálogo não conhece: **projeto MicroPython e `mpremote`
detectado → `mpremote connect <dev> repl`** (`0.102.0`: num firmware
MicroPython o monitor É o REPL — um tio a 115200 mostraria o mesmo texto sem
o raw-paste nem o Ctrl-] de sair; o mpremote é o último do catálogo e nunca
vence sozinho fora de MicroPython); chip Espressif no kit **e** `espflash`
detectado → `espflash monitor --elf <ELF>`, que decodifica o backtrace;
escolha **fixada** pelo autor vence tudo. **Bloco E (`0.117.0`):** antes de tudo que não foi fixado, o wrapper do
framework — num projeto ESP-IDF com ativação nesta máquina, o **IDF
Monitor** (`bash -c '<wrapper>' idf <ativação> -p <dev> monitor`; decodifica
o backtrace com o ELF do build); num PlatformIO com `pio`, `pio device
monitor -p <dev> -b <baud>`. Nenhum dos dois exige monitor no catálogo.
Linhas de comando lidas na fonte: `tio -b`,
`picocom -b`, `minicom -D … -b`, e no espflash é `--monitor-baud` (o `--baud`
dele é o de **gravação**). Baud ausente = 115200. Sem nenhum monitor
instalado, `TOOL_NOT_FOUND` com o que instalar. Exige workspace (a aba nasce
no cwd do projeto) e volta como sessão de terminal, com o comando no título.

**A `hint` de acesso é o passo oficial, nunca um `sudo` que a IDE rodaria:**
*"`/dev/ttyUSB0` é `crw-rw----` do grupo `dialout` e você não está nele …
`sudo usermod -aG dialout $USER` e sair/entrar da sessão. A IDE não roda isso."*

**`serial.identify` é a identidade PELO CANAL (E5 do `integracoes/38` §6,
`0.112.0`)** — o único `serial.*` que ABRE a porta, e por isso um job pedido
por clique: o esptool puxa DTR/RTS e a placa reseta para o bootloader de ROM.
Linha: `esptool --port <device> --chip auto --before default-reset --after
hard-reset flash-id` (`--after hard-reset` devolve a placa ao firmware ao
terminar). A v4 (`esptool.py`, argparse) não conhece `flash-id`: o job vê
"invalid choice"/"No such command" e repete com `flash_id`; o `command` do
evento é o que rodou por último. O parser é **tolerante** como o do
`probe.list` — o formato foi lido no código do esptool 5.4.0 desta máquina
(`Chip type:`, `Features:`, `Crystal frequency:`, `USB mode:`, `MAC:`,
`Manufacturer:`, `Device:`, `Detected flash size:`; `BASE MAC:`/`MAC_EXT:`
não confundem o `MAC:`; `Unknown` não vira tamanho) — linha que não casa é
ignorada e `raw` volta sempre; `success` exige `Chip type:` lido E exit 0.
`chip` é a chave do IDF (`ESP32-C3 (QFN32) (revision v0.4)` → `esp32c3`;
`ESP8685/8686` → `esp32c3`, `ESP8684` → `esp32c2`). **`target`** é o que o
chip sugere pelas MESMAS tabelas de família e motores do `project.model`
(`espressif`; `esptool`/`espflash`/`probe-rs` nos chips com USB-JTAG, sem
depurador no ESP32 clássico), com `evidence` dizendo que veio do
`flash-id`; nada vira kit sem `toolchain.setKit`. Não exige workspace.
Recusas ANTES de abrir a porta: `device` inexistente → `INVALID_PARAMS`;
sem leitura/escrita → `INVALID_REQUEST` com a `hint` de `serial.list`; sem
`esptool`/`esptool.py` no PATH (ou `tool` dado) → `TOOL_NOT_FOUND` com `pipx
install esptool`. Prazo de 30 s no job (o esptool tenta sincronizar várias
vezes): estourou → `success: false` com "não respondeu em 30 s"; cancelado
(`job.cancel`) → mata o esptool e diz "cancelada". `JobRisk::Medium` porque
reseta a placa; nada é escrito nela.

**`serial.files` são os ARQUIVOS NA PLACA de um MicroPython (C2 do
`roadmaps/41` bloco C, `0.116.0`; referência de UX: o "Files on device" do
Thonny)** — `mpremote connect <device> fs <ls|cp|rm|mkdir>` como job, um por
vez (o mpremote prende a porta). O `:` marca o lado da placa e é a IDE que
o põe: `path` vai sem ele (`main.py`, `lib/wifi.py`; ausente só em `list`
= a raiz); `get` é `cp :<path> <local>`, `put` é `cp <local> :<path>`. Um
`local` relativo é sob o workspace (sem workspace, só absoluto); a tela
baixa para `placa/<caminho>` — o espelho da placa, que nunca colide com o
código do projeto e pode ser baixado de novo (o core cria a pasta). O
formato do `ls` foi medido no ESP32 do autor com o mpremote 1.29.0
(`{tamanho:12} {nome}[/]`, uma linha verbosa `ls :<path>` antes) e lido
no código dele (`commands.py::do_filesystem`). **Todo `fs` interrompe o
programa da placa** (raw REPL, soft reset ao sair) — por isso até o `ls`
é gesto explícito e `JobRisk::Medium`; `put`/`rm`/`mkdir` escrevem a
flash e são `High` (a tela pede o segundo clique). **A primeira conexão
pode falhar:** medido com o firmware do autor inundando a UART (1,3 MB de
binário em segundos), o mpremote morreu com `could not enter raw repl`;
a segunda entrou — o job repete UMA vez nesse caso, e as duas saídas
viajam em `raw`. `error` é a linha `mpremote: <cmd>: <caminho>: <motivo>.`
que ele escreve no stderr (sem o prefixo). Recusas antes de tocar a
porta: pedido inválido (`:` na frente, `rm` sem `path`, `get` sem `local`,
`put` de arquivo inexistente) → `INVALID_PARAMS`; porta inexistente ou
sem acesso → como `serial.identify`; sem `mpremote` → `TOOL_NOT_FOUND`.
Prazo de 120 s por execução; cancelar mata o processo. O `put` do mesmo
conteúdo não regrava: o mpremote confere o hash e diz `Up to date`
(medido: o `main.py` do autor reenviado, SHA-256 igual antes e depois).

**`serial.access` é a permissão POR CANAL (E2 do `integracoes/38` §6,
`0.114.0`)** — a fatia 4.3 redesenhada: para CADA canal, o que falta e o
passo oficial. Não exige workspace; não roda nada. Canais: **`serial`**, por
porta — `ok` é o `access(2)` do `serial.list`; `detail` diz o modo, o grupo
dono e se o processo está nele (`/proc/self/status` `Groups:` × `/etc/group`);
quando o acesso vem sem grupo e as propriedades udev trazem `TAGS=…:uaccess:…`,
`distroDidIt` credita a ACL do udev; sem acesso, `fix` é `sudo usermod -a -G
<grupo> $USER` + re-login (fonte: ESP-IDF *Establish Serial Connection*,
2026-09-17) para `dialout|uucp|plugdev|tty`, ou uma regra `TAG+="uaccess"`
para a VID:PID quando o grupo é outro. **`modemManager`**, por porta, só
quando o `udevadm` respondeu — `ok` = não (rodando ∧ candidata ∧ sem regra);
`fix` escreve `ATTRS{idVendor}=="…", ATTRS{idProduct}=="…",
ENV{ID_MM_DEVICE_IGNORE}="1"` em `/etc/udev/rules.d/77-mm-kinein-<vid>-<pid>.rules`
(o 77 corre ANTES do `80-mm-candidate.rules` que marca toda tty; a forma é a
das regras que o pacote instala) e recarrega o udev. **`probe`**, da máquina
— procura `*probe-rs*|*openocd*|*stlink*|*jlink*|*cmsis*.rules` em
`/etc/udev/rules.d`, `/usr/lib/udev/rules.d` e `/lib/udev/rules.d` (pastas
canônicas iguais contam uma vez); achado fora de `/etc` é `distroDidIt`;
nada achado → os três passos do probe.rs *Probe Setup* (baixar
`69-probe-rs.rules`, `udevadm control --reload`, `udevadm trigger`,
citados). Cada `fix` traz `sourceUrl` e `checkedOn`; a UI escreve o
`command` no terminal da IDE pelo mesmo caminho do painel de instalação.

## `command.*` — o catálogo de comandos que a UI mostra

Um método, e ele é a fonte única de **tudo que a IDE oferece por nome**: paleta,
menus, botões e atalhos leem daqui.

```text
command.list {}   ->  { commands: [CommandDescriptor] }
```

```json
{
  "id": "editor.save",
  "title": "Salvar arquivo",
  "category": "Editor",
  "description": "Grava o buffer atual no disco",
  "defaultShortcut": "Ctrl+S",
  "requiresWorkspace": true
}
```

**Não exige workspace aberto** — a paleta existe antes de haver projeto, e é o
`requiresWorkspace` de cada descritor que diz o que fica desabilitado.

**São 76 descritores, medidos em 2026-09-06**, em cinco grupos que são cinco
arquivos em `crates/kinein-core/src/commands/`:

```text
ide.rs      24    editor.rs   19    build.rs    16    git.rs       9    run.rs    8
```

**Por que este domínio existe em vez de a UI ter a lista:** porque o atalho que a
paleta **anuncia** tem de ser o que a IDE **obedece**, e isso é gate desde
2026-09-03 (`scripts/verificar-atalhos.sh`). Com a lista no core, o gate compara
uma fonte com o host; com a lista na UI, ele compararia a UI consigo mesma.

## `coverage.*` — a cobertura dos testes

Domínio novo no protocolo `0.119.0` (D8 do `roadmaps/41`, P5 do `40` §4.1,
2026-09-17). O LCOV é a língua comum: é o que `cargo llvm-cov` e o
`coverage.py` escrevem, e o que a calha do editor lê.

```text
coverage.run   {}          -> { jobId }   (job; event.coverage.finished no fim)
coverage.lines { file }    -> { file, known, covered: [linha], missed: [linha] }

event.coverage.finished { jobId, success, tool, path?, files: [{ file, linesFound,
                          linesHit }], error? }
```

**Por tipo de projeto:** Rust → `cargo llvm-cov --lcov --output-path
.kinein/coverage.lcov` (o `cargo` do kit; `cargo-llvm-cov` 0.9.1 desta
máquina: `--lcov` + `--output-path`; exige `rustup component add
llvm-tools-preview`, e sem o `cargo-llvm-cov` o job falha dizendo os dois
passos); Python → `<python> -m coverage run -m pytest -q` e `<python> -m
coverage lcov -o .kinein/coverage.lcov` pelo interpretador do projeto (sem o
módulo, o passo `uv add --dev coverage pytest`); C/C++ → `INVALID_REQUEST`
antes do job: exige compilar com `--coverage` (gcov/lcov), o que é decisão
do usuário no build — não entra sem ela. O arquivo fica em
`<root>/.kinein/coverage.lcov` e é relido: `files` é um resumo por arquivo
(`SF:`, os `DA:` somados por linha — blocos repetem linhas), e
`coverage.lines` dá as linhas de UM arquivo (caminho exato ou canônico
igual), `known: false` quando o relatório não o tem. Na UI: comando
**Cobertura dos testes** (paleta e menu Build) → job; o `CoverageController`
guarda o resumo e, a cada troca de aba, pede as linhas do arquivo ativo — a
calha pinta uma barra de 3 px ao lado da do diff: verde coberta, vermelha
instrumentada e nunca executada.

## `remote.*` — o alvo Linux por SSH

Domínio novo no protocolo `0.120.0` (P6 fatia 1 do `roadmaps/42`, P6 do `40`
§4.1, 2026-09-17): a Raspberry Pi, a placa com imagem própria, como recurso
do projeto — no molde do `datasource.*`. Transporte é o `ssh`/`rsync`/`scp`
do sistema como **processo** (OpenSSH BSD, rsync GPL-3 — nunca crate).

```text
remote.list    {}                                -> { targets: [RemoteTarget] }
remote.save    { target }                        -> { targets }   (cria/substitui pelo nome)
remote.remove  { name }                          -> { targets }
remote.probe   { name }                          -> { jobId, command }   (job)
remote.deploy  { name, source?, dest? }          -> { jobId, command }   (job)
remote.command { name, kind, program?, port? }   -> { command, remoteTarget?, name, source[] }
               kind: run | debugServer | debugpy | shell | copyId   (PURO: nada roda)
               copyId (0.133.0) -> `ssh-copy-id [-p P] [-i K] [user@]host`
remote.parseCommand { command }                  -> { target: RemoteTarget, source[] }
               (0.134.0; SEM workspace; LE' a linha, nunca a executa; nada e' salvo)
remote.discover {}                               -> { aliases: [{ name, source }], sources[] }
               (0.132.0; SEM workspace; lê arquivo local, não conecta)
remote.resolve  { host }                         -> { host, hostName?, user?, port?,
                                                     identities[], proxyJump?, proxyCommand }
               (0.132.0; SEM workspace; `ssh -G`, que não conecta)

RemoteTarget   name · host · user? · port? (22) · identityFile? · deployDir? (~/kinein/<projeto>)
               — SEM senha, estruturalmente: `deny_unknown_fields`, e um teste reprova
               qualquer chave que pareça segredo no arquivo gravado

event.remote.probed   { jobId, name, success, arch?, kernel?, tools: [{ id, found, path? }],
                        error?, failure?, raw }
                        failure (0.133.0, ausente quando success): authentication |
                        host | network | other — a causa TIPADA, nao a frase
event.remote.deployed { jobId, name, success, source, dest, command, error? }
```

**Persistência:** `.kinein/remotes.json` (`schemaVersion: 1`, ordenado pelo
nome; arquivo inválido = nenhum alvo; porta 22 e campos vazios ficam
ausentes). **Probe:** `ssh -o BatchMode=yes -o ConnectTimeout=5 [-p P] [-i K]
[user@]host 'uname -m; uname -sr; for t in gdbserver python3 rsync; do
printf "%s=" $t; command -v $t || echo; done'` — uma linha por fato, lida
sem adivinhar; `BatchMode` faz a falta de chave falhar em segundos, e o
`error` diz o passo (`ssh-copy-id user@host`; host não resolvido; sem
rota em 5 s). **Deploy:** `rsync -az --delete -e 'ssh [-p P] [-i K]'
<origem> [user@]host:<dest>/` quando há `rsync` nesta máquina, senão `scp
[-P P] [-i K] -r`; a origem padrão é `<root>/build`, o destino o `deployDir`
do alvo ou `~/kinein/<projeto>` (o `~` fica sem aspas para o shell REMOTO
expandir); origem inexistente é recusa síncrona ("compile antes"). **Antes de copiar**, o
job roda `ssh -o BatchMode=yes … 'mkdir -p <dest>'`: medido em 2026-09-24 contra
um alvo real, nem `rsync` nem `scp -r` criam diretório intermediário, e o padrão
`~/kinein/<projeto>` não existe numa placa recém-instalada — o **primeiro deploy
de qualquer alvo novo** falhava com um erro de `rsync` que não dizia o que fazer.
É `mkdir -p` e não `rsync --mkpath` porque o `--mkpath` exige rsync 3.2.3+ dos
dois lados, e o projeto usa o que o sistema tem. Falha no `mkdir` não
interrompe: se for real, a cópia falha em seguida com a mensagem dela.
**Comandos:** `run` → `ssh -tt … '<programa>'`; `debugServer` → `ssh -tt …
'gdbserver :2345 <programa>'` com `remoteTarget = host:2345` (o `gdb -i
dap` faz `attach` a servidor desde 0.89.0); `debugpy` → `ssh -tt … 'python3
-m debugpy --listen 0.0.0.0:5678 --wait-for-client <script>'` com
`remoteTarget = host:5678` (o attach TCP de 0.109.0); `shell` → `ssh …
[user@]host` sem `BatchMode`, para o terminal da IDE. `program` relativo
entra no `deployDir`; absoluto ou `~` vai como dado; ausente vira
`<binario>` com a fonte dizendo "edite". O `-tt` força pty mesmo sem
terminal local (o `debugServer` do kit roda por `sh -c` com stdio fechado):
matar o `ssh` local derruba o processo remoto por SIGHUP — sem isso o
`gdbserver` ficaria órfão na placa. Na UI: painel **Alvo remoto (SSH)** (menu
Ambiente; comando `remote.list`): lista, formulário sem campo de senha,
**Sondar** (primário), **Enviar**, e os quatro botões que levam o resultado
do `remote.command` ao dono certo — `runConfig.save` ("Rodar em pi",
"debugpy em pi"), `toolchain.setKit { remoteTarget, debugServer }` (só os dois
campos; a ponte ganhou `toolchainSetKitRemote`) ou
`runtimeController.submitShellInput`. **Não entrou na fatia 1 (dito):**
workspace remoto, LSP do outro lado, mapeamento de caminhos, `sshd` local
no gate — o `RemoteContext` inteiro do 28 §4 segue no `42` §P6.

### Descobrir e explicar (`0.132.0`, fatia R0.5)

Fonte: [`remote-ssh-ui-hud.md`](../especificacoes/remote-ssh-ui-hud.md) §6.1 e
[`roadmap 48`](../roadmaps/48-arquitetura-executavel-da-serie-0.3.md) §8.1. A
lacuna não era guardar o alias — `RemoteTarget.host` sempre aceitou um alias com
user/port/identity ausentes. Era **descoberta e explicação**.

**`remote.discover {}`** lê `~/.ssh/config` e, no ponto em que aparecem, os
`Include` dele. Só um `Host` **concreto** vira alias selecionável: padrão com
`*`, `?` ou `!` continua valendo na resolução do OpenSSH, mas não é algo que o
usuário possa escolher. `Host "um dois"` é **um** padrão com espaço, recusado —
não dois aliases inventados. Cada alias diz o arquivo que o declarou
(`source`, com `~` no lugar da home), porque um alias de `config.d/` não é a
mesma coisa que um do arquivo principal quando algo destoa. Limites explícitos,
para um `Include` mal escrito não virar varredura de disco: 16 arquivos, 8
níveis, 256 KiB por arquivo, 512 aliases, e `*` casa dentro de **uma** pasta.
Máquina sem `~/.ssh/config` devolve lista vazia — "não há SSH configurado" é
estado do produto, não erro do protocolo.

**`remote.resolve { host }`** roda `ssh -G <host>`, que imprime a configuração
**efetiva** sem abrir sessão, e devolve uma **allowlist**. O dump inteiro não
sai: `localforward`, `sendenv` e o resto são descartados. `proxyJump` é uma
especificação de host, segura de mostrar; de um `ProxyCommand` sai apenas
`proxyCommand: true` — o texto é uma linha de comando arbitrária, que pode
conter segredo, e por isso **nunca** é devolvido. `host` volta ecoado para a UI
descartar resposta atrasada de outro alvo. O core recusa antes de rodar
processo um host vazio, com espaço, com `@`, com controle, longo demais ou
começando por `-` (o `ssh` leria como opção). Como `ssh -G` imprime valores
efetivos, `port` vem resolvido (22 quando ninguém mudou) e `identities` são as
que o `ssh` **tentaria** — não prova que alguma exista. Medido nesta máquina em
2026-09-24: sem `~/.ssh/config`, `ssh -G localhost` imprime 90 linhas e lista
**cinco** `identityfile` padrão, nenhuma presente no disco. Por isso a UI diz
"que o ssh tentaria", e o core não filtra por existência: filtrar trocaria "o
que o OpenSSH diz" por "o que nós achamos que vai funcionar". A UI não persiste um
override que só repete o que o OpenSSH já faria: escolher um alias grava
`{ name, host }` e nada mais.

`ssh -G` roda com `-F` apontando **o mesmo arquivo** que a descoberta leu.
Medido em 2026-09-24 contra um alvo real: o OpenSSH **não honra `$HOME`** para
achar o `~/.ssh/config` — usa a base de senhas do sistema. Sem o `-F`, a
descoberta podia listar aliases de um arquivo enquanto a resolução explicava
outro, e a IDE afirmaria sobre um config que o `ssh` não usaria. Sem arquivo
nenhum vai só `-G`: `ssh -F <inexistente>` é erro, e "não há config" é estado
normal.

**Ressalva dita:** `ssh -G` avalia `Match exec` do config do próprio usuário,
o que executa um comando local. Rodar síncrono (como git, probe e size já
fazem) é aceito porque `ssh -G` é local e instantâneo; um `Match exec` lento
atrasaria o `ssh` do usuário do mesmo jeito.

**Não entrou nesta fatia (dito):** `remote.directories` — a navegação tipada da
pasta remota. A §8.1 do 48 a propôs "se confirmado necessário no teste", e o
teste desta fatia não chegou à escolha de pasta. O campo de caminho continua
sendo o caminho.

**Diferenças frente à proposta da §8.1**, registradas: `discover` devolve também
`sources` (os arquivos lidos, para a UI dizer de onde veio); `resolve` ecoa
`host` e troca o `proxy?` genérico por `proxyJump?` + `proxyCommand: bool`,
porque juntar os dois num campo exigiria devolver o texto do comando.

### A linha colada vira um perfil (`0.134.0`, fatia R0.5)

O segundo caminho da §6.1 da especificação — **configurar servidor** —, e ele
não é um formulário. A ideia vem do `Remote-SSH: Add New SSH Host…` do VS Code
(comparação na §3.1 daquele documento): a pessoa cola o comando que já usa.

Cabe aqui sem quebrar a regra "a UI não monta linha de `ssh`": quem **fornece** a
linha é o usuário, e quem a **interpreta** é o core. E ela é lida, nunca
executada — texto entrando, perfil saindo.

Entende `ssh [-p N] [-i chave] [-l usuário] [-o Port=/User=/IdentityFile=]
[usuário@]host`, aceita só o destino, tolera um `$ ` colado junto do prompt e
respeita aspas. O `name` é **proposto** a partir do host (o primeiro rótulo de um
nome; um IPv4 inteiro vai inteiro, porque `192` não é nome de nada) e a UI deixa
editar.

**O que ele RECUSA, em vez de limpar em silêncio:**

- qualquer `;`, `|`, `&`, crase, `$(`, `>` ou `<` — só fazem sentido para um
  shell, e o que vem depois deles não é assunto de um perfil;
- opção que o perfil não modela (`-J bastion`, `-o ProxyCommand=…`), **dizendo
  qual** e mandando deixá-la no `~/.ssh/config` para escolher o alias aqui;
- um comando remoto depois do destino, porque isso seria um comando, não um
  perfil.

Aceitar e ignorar prometeria um alvo que não se comporta como a linha colada —
e a pessoa não saberia por quê.

### A falha vira um gesto (`0.133.0`, fatia R0.5)

A §3 da [`remote-ssh-ui-hud.md`](../especificacoes/remote-ssh-ui-hud.md) lista
como defeito nº 14: *"a mensagem 'use `ssh-copy-id`' transfere o problema ao
terminal sem guiar"*. O core já separava as causas para escolher a frase — dizê-la
em **tipo** expõe uma decisão que ele já tomava, não inventa dado. A UI escolhe o
gesto por `failure`, nunca casando texto: frase muda de idioma, tipo não.

Só `authentication` tem um gesto que a resolve, e é `ssh-copy-id`. O botão aparece
**onde a causa foi explicada**, no veredito da sonda. Clicar **compõe** a linha e
a mostra; rodar exige um segundo gesto. Isso é a §10 da especificação: *"geração/
cópia de chave nunca ocorre silenciosamente"* e *"o comando é visível antes de
executar"*. A IDE não gera chave, não digita senha — o `ssh` pede no terminal, e
o host key é aceito ali, uma vez.

`copyId` reusa o `-p`/`-i` do perfil: copiar a chave por outra porta que não a do
alvo copiaria para a máquina errada. Com `-i <privada>` o `ssh-copy-id` procura a
`.pub` correspondente. Uma linha armada **não sobrevive à troca de alvo**, porque
ela carrega um host.

### O workspace espelhado (`0.122.0`, P6 fatia 2)

```text
remote.open   { name, path }                       -> { jobId, command, mirror }   (job: pull)
remote.sync   { direction: pull | push, paths? }   -> { jobId, command }           (job)
remote.status {}                                   -> { mirror?: RemoteMirror }
RemoteMirror  name · host · path (no alvo) · mirrorRoot (local)

event.remote.synced { jobId, name, direction, success, command, changed: [caminho],
                      error?, mirror }
workspace.open de um espelho  -> … , remote: RemoteMirror
```

A pergunta era "como abrir a pasta da Pi". O VS Code (Remote-SSH,
proprietário) sobe um servidor no alvo; aqui a resposta é a outra escola
(o "deployment" do PyCharm, o fluxo Zephyr/Yocto): **a pasta remota vira
um espelho local por `rsync`, e a IDE abre o espelho como workspace
comum** — zero mudança em `fs.*`, índice, busca, git e LSP. `remote.open`
puxa a árvore (`rsync -az -i --exclude .kinein -e 'ssh [-p] [-i] -o
ControlMaster=auto -o ControlPath=<cache>/ssh-%C -o ControlPersist=60'
[user@]host:<path>/ <espelho>/`) para `~/.cache/kinein-vectis/remote/
<alvo>/<hash do path>/<basename>` — o basename vira o nome do workspace —,
grava `.kinein/remote-mirror.json` (`schemaVersion`, `name`, `host`,
`path`) e copia o alvo para o `.kinein/remotes.json` **do espelho**, que é
autossuficiente (usuário, porta, chave); o `.kinein` nunca sincroniza,
então nada disso chega ao alvo. Quando o `event.remote.synced { direction:
pull }` chega, a UI abre o espelho pelo `workspace.open` de sempre, cuja
resposta traz `remote`. **Salvar empurra:** um `fs.write` num espelho
dispara o job `Empurrar <arquivo> para <alvo>` só com aquele caminho (o
`ControlMaster` do ssh mantém uma conexão só entre gravações). `remote.sync`
move a árvore inteira ou `paths` relativos (sem `..`, nunca `.kinein`),
**nunca com `--delete`** — apagar do outro lado é gesto explícito, fora
desta fatia. `changed` é a saída `-i` (itemize) do `rsync`, lida linha a
linha. **Dito como não feito:** watcher do lado remoto (o que muda no alvo
só aparece ao Puxar); renomear/apagar não propaga; o LSP resolve contra
ESTA máquina (C/C++ cross usa o sysroot do kit; Python/Rust do alvo não);
edição dos dois lados = o `rsync` mais recente vence.

## `setup.*` — o passo a passo oficial de instalação, por distro

```text
setup.list {}  ->  { distroId, distroName, family, tools: [SetupToolInfo] }
```

```text
SetupToolInfo   id · name · summary · website · installed · guide?
SetupGuide      family · sourceUrl · checkedAt · steps: [SetupStep]
SetupStep       explanation · command
```

**Não exige workspace**: instalar o PostgreSQL não depende de projeto aberto, e
quem está começando abre a IDE antes de ter projeto — que é exatamente quando
este guia serve.

**A regra que governa o domínio inteiro: sem fonte oficial, a IDE não afirma.**
Cada `SetupGuide` carrega `sourceUrl` e `checkedAt`, e uma família de distro sem
fonte oficial **não recebe guia** — o `guide` volta `None` e a tela mostra o site
do projeto dizendo que não tem passo a passo. É a decisão registrada em
`../roadmaps/40` §5, e é por isso que Arch e openSUSE continuam sem passos: a
fonte dos três projetos não cobre essas famílias.

**Embarcados (A5 do `roadmaps/41`, 2026-09-17, sem mudança de contrato):**
o `setup.list` passou a unir dois catálogos — o geral (`setup/catalog.rs`) e
o de embarcados (`setup/catalog_embedded.rs`): `esptool`, `mpremote`,
`espflash`, `probe-rs`, `picotool`, `dfu-util`, `tio`, `picocom`,
`arm-none-eabi` (gcc + gdb), `qemu-embedded` (ARM e RISC-V), `openocd`.
Duas classes de fonte, ditas no arquivo: a página da ferramenta (comando
verbatim — `pip install esptool` num venv, `pipx install mpremote`,
`cargo install espflash --locked`, o instalador do probe-rs, o `BUILDING.md`
do picotool) e o índice de pacotes da distro (a página do pacote prova o
nome; o comando é a forma padrão do gerenciador). Onde o índice não tem o
pacote — `tio` e `picotool` no Arch oficial, `picotool` e `espflash` no
Fedora, `espflash` no Ubuntu 26.04 (a família `debian` cobre Debian e
Ubuntu; um guia que falha em metade da família não entra) — a entrada não
existe. Medido nesta máquina (Ubuntu 26.04): 18 ferramentas, 7 já instaladas
entre as de embarcado.

O `installed` vem do `ToolDetector` — detectar é capacidade, e a política de o
que fazer com a detecção fica na UI (`27-modulos-por-dominio.md` §6).

## `datasource.*` — os perfis de banco, e a senha que não mora em disco

Domínio da etapa 26/27 (`../roadmaps/35` §9). Oito métodos, quatro eventos.

```text
datasource.discover   {}                      -> { candidates: [DataSourceCandidate], containerEngine?, hint? }  (0.124.0, adiado)
datasource.create     { kind: sqliteFile, name, path? } -> { profile }                                       (0.124.0)
                      { kind: containerServer, engine, name, port } -> { jobId, command }  + event.datasource.created
datasource.destroy    { name, data? }         -> { profiles, note? } | { jobId, command } + event.datasource.destroyed  (0.129.0)
datasource.list       {}                      -> { profiles: [DataSourceProfile] }
datasource.save       { profile }             -> DataSourceWriteResult
datasource.remove     { name }                -> DataSourceWriteResult
datasource.test       { name, password? }     -> DataSourceTestAccepted   (job)
datasource.introspect { name, password? }     -> aceite + job
datasource.query      { name, password?, sql, maxRows?, confirmWrite? } -> aceite + job  (0.121.0)
```

**Os cinco últimos exigem workspace aberto**; o perfil mora no projeto.

```text
event.datasource.tested        { jobId, ok, message, ... }
event.datasource.introspected  { jobId, schemas | collections, ... }
event.datasource.queried       { jobId, name, success, columns: [string], rows: [[string | null]],
                                 rowCount, affected?, truncated, elapsedMs, message?, secretRequired }
event.datasource.created       { jobId, success, profile?, message }                          (0.124.0)
event.datasource.destroyed     { jobId, success, message, profiles? }                          (0.129.0)
```

**Descobrir e criar (`0.124.0`).** `DataSourceCandidate { kind: localServer
| container | file, label, detail, running, profile }` — cada achado traz o
perfil que o alcança, pronto para o formulário (adotar não salva; salvar é
do autor). O `create` em container usa autenticação `trust` **só no
loopback** (`-p 127.0.0.1:…`): a IDE não guarda senha, e a porta não sai da
máquina; imagens pinadas (`postgres:16`, `mongo:7`). O MongoDB cria banco na
primeira escrita — não há `create` para ele além do servidor.

**Executar o que o autor escreveu (`0.121.0`, `../roadmaps/35` §7.4).** A
primeira palavra da instrução (comentários iniciais pulados) diz se é
leitura — `SELECT`, `WITH`, `VALUES`, `TABLE`, `SHOW`, `EXPLAIN` — e o **motor
impõe** o que a classificação prometeu: no PostgreSQL a leitura roda em
`BEGIN READ ONLY` (um `WITH … INSERT` disfarçado é recusado pelo servidor),
no SQLite o arquivo abre com `SQLITE_OPEN_READ_ONLY`. O teto (`maxRows`,
padrão 500, máximo 10.000) vem de fora do texto: `SELECT * FROM (<sql>) AS
kinein_q LIMIT n+1` quando é uma instrução só de `SELECT`/`WITH`/`VALUES`/
`TABLE`, o `step` até n+1 no SQLite; `truncated` diz quando cortou. Uma
instrução que **não** é leitura sem `confirmWrite: true` é recusada antes do
job com `WRITE_CONFIRMATION_REQUIRED` (código próprio, `details.name`) — a
UI mostra "esta instrução ESCREVE" e reenvia com o campo; confirmada, roda
como o autor escreveu (`simple_query`/`execute_batch`) e `affected` é o que
o motor contou (o SQLite conta a última instrução). Células são texto: o
protocolo simples do PostgreSQL devolve toda coluna assim, sem mapa de
tipos; `NULL` é `null`; `BLOB` do SQLite vira `<N bytes>`; várias
instruções → o último conjunto de resultados. MongoDB nesta fatia: só
leitura — `<coleção> <filtro JSON>` (filtro ausente = `{}`) → `find` com
`limit`, colunas = união das chaves de primeiro nível (`_id` primeiro),
células = o JSON relaxado do valor; escrever documento fica dito como não
feito. Vazio é `INVALID_PARAMS`; a senha segue a política do perfil
(`SECRET_REQUIRED` síncrono, `secretRequired` no evento).

**TLS no PostgreSQL (`0.121.0`).** `DataSourceProfile.tls: disable |
require` (ausente = `disable`, o de sempre) e `caFile` (PEM em que confiar,
para o servidor autoassinado; ausente = raízes públicas do `webpki-roots`).
`require` é o `verify-full` do libpq — cadeia **e** nome do host conferidos
—; não existe "cifra sem conferir" (o `sslmode=require` do libpq). Só vale
para o motor `postgres`: noutro motor o `save` descarta. Implementação:
`tokio-postgres-rustls` 0.14 (MIT) sobre o `rustls` que o `mongodb` já
trazia — +11 crates medidos em 2026-09-18, `cargo deny` verde
(`../integracoes/37`).

**A senha nunca entra no perfil.** O `DataSourceProfile` guarda motor, host,
porta, banco, usuário e um `SecretSource` — *de onde* o segredo vem —, e o
`password` viaja só no parâmetro do método que precisa dele, por chamada. É a
decisão de `../seguranca/40`, e a UI abre o diálogo de senha por
`secretRequired` no erro, **nunca casando texto de mensagem**.

**Duas formas de resultado, porque há dois tipos de banco.** O relacional
devolve `schemas → tables → columns` lido do `information_schema`; o MongoDB
devolve `collections → fields` com profundidade, tipo **plural** e presença em
%, e a origem do esquema declarada:

```text
DECLARADO   veio do validador `$jsonSchema` da colecao
INFERIDO    veio de `$sample` sobre a colecao
```

**A tela nunca deixa os dois parecidos**, e o custo da leitura vai junto:
quantos documentos foram lidos, e se a amostragem obrigou o servidor a varrer a
coleção inteira (`$sample` varre tudo quando N não é menor que 5% dela). Tetos
de RAM — 2.000 campos, 8 níveis, 10 elementos de array — aparecem como aviso na
coleção em vez de a deixarem com cara de completa.

## `grafana.*` — a observabilidade pela HTTP API, e só

Quatro métodos, um evento.

```text
grafana.get    {}          -> { profile: GrafanaProfile | null }
grafana.save   { profile }
grafana.forget {}
grafana.probe  { token? }  -> GrafanaProbeAccepted   (job)
```

```text
event.grafana.probed  { jobId, datasources, dashboards, matches, ... }
```

**Os quatro exigem workspace aberto.**

**O Grafana nunca é embutido** — licença AGPL, decisão registrada em
`../roadmaps/40` §5. A integração é HTTP, o cliente é o `ureq`, e os dashboards
**abrem no navegador do sistema**. O que justifica o domínio existir é o
`GrafanaMatch`: o cruzamento entre o datasource do Grafana e o perfil de banco
do projeto, que é a pergunta que nenhuma das duas ferramentas responde sozinha.

O token segue a mesma regra da senha: `GrafanaTokenSource` diz de onde ele vem,
e o valor viaja por chamada.

## `core.*` — o handshake e o encerramento

Dois métodos, e eles são os únicos que **nunca** dependem de nada: não exigem
workspace, não tocam o filesystem e respondem mesmo com o resto do core inerte.

```text
core.ping     {}  ->  { protocolVersion, ... }
core.shutdown {}  ->  { message: "shutdown requested" }
```

**O `core.ping` é como a UI descobre com que protocolo está falando** — é o par
do `PROTOCOL_VERSION` em `crates/kinein-protocol/src/lib.rs`, e é o primeiro
pedido que a UI faz depois de subir o processo (`04-boot-e-comunicacao.md`).

**O `core.shutdown` pede, não mata.** Ele responde e deixa o encerramento
acontecer com o drain dos jobs em andamento, que é o que impede um build a meio
caminho de virar processo órfão.
