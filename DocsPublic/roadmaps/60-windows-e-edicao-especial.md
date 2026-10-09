# 60 — Windows: adaptar a IDE, fechar a 0.3.9 e a 0.4.0, e a edição especial

> **Classe: PLANO** (`DocsPublic/README.md`). Decisão do autor em 2026-10-08,
> à noite. Cada fatia ainda tem desenho próprio antes do código, no documento
> dono, como sempre.

> **EM VIGOR de novo desde 2026-10-09** (40.7 §7.260). No mesmo dia o autor
> o suspendeu ("não vou mais para ambiente Windows no momento", §7.258) e o
> retomou: "por eu tocar um projeto de integração TA/TI vou precisar estar
> usando Windows 11 Pro". O ambiente de desenvolvimento passa a ser o
> **Windows 11 Pro**. Entre as duas decisões, o 9a.3 foi feito no Arch
> (§7.259); o §1 e o §2 abaixo já contam com ele.

> **Ordem revista em 2026-10-09, à noite** (40.7 §7.261): primeiro o porte,
> depois a organização da documentação (e da arquitetura, se a medição
> pedir), e só então o fechamento da 0.3.9. O §1 já está na ordem nova. O §2.1
> traz a medição feita no Windows; o §3, as decisões D1–D4 e as fatias
> W1–W5; o §6, o ponto de partida da organização.

## 1. A decisão

O autor está migrando o sistema operacional de desenvolvimento para Windows,
por necessidade de um projeto que pode se tornar fonte de renda. Nas palavras
dele: "nesse meio tempo podemos adaptar a IDE para multiplataforma e usá-la no
Windows, finalizando a 0.3.9, vamos para a 0.4.0 e, finalizando ela, vamos ter
uma 'special edition' onde vamos buscar resolver o problema/atrito do Windows
e WSL2 com a IDE, para parte de embarcados e comunicação USB".

A ordem:

1. **Adaptar a IDE para rodar no Windows** (o porte, §3), o suficiente para o
   autor continuar desenvolvendo a própria IDE nela, sem perder o Linux: o
   mesmo código compila e roda nos dois, e o gate do Linux continua verde.
2. **Organizar a documentação por contexto e, se a medição pedir, a
   arquitetura** (§6), para que uma pessoa ou um agente de IA leia só o que
   vai mexer.
3. **Fechar a 0.3.9**: a fatia dos containers (16b), as provas finais (15) e
   o pente fino (16) ([59](59-fechamento-da-0.3.9.md) §2). O 9a.3 foi feito
   no Arch em 2026-10-09 (40.7 §7.259). O corte de escopo de 2026-10-08 já
   tirou da 0.3.9 o 9b/9c, o 10 e o 11 (59 §2.3).
4. **A 0.4.0**: embarcados ([52](52-arquitetura-executavel-da-0.4.md)).
5. **A edição especial**, depois da 0.4.0: resolver o atrito do Windows com o
   WSL2 na IDE, para embarcados e comunicação USB.

O item 2 é decisão do autor em 2026-10-09, à noite: "depois de fazer a IDE
rodar no windows, vamos também já melhorar a arquitetura do projeto
atualmente, ainda mais no quesito documentação, para os agentes de IA,
inclusive você, não ficar lendo partes desnecessárias onde não vai mexer", e
"separar a documentação por contexto, deixar tudo explícito e bem separado".
Ele vinha depois da 0.3.9; na mesma noite o autor aceitou trazê-lo para antes,
porque as fatias que faltam na 0.3.9 também pagam o custo de leitura ("pode
seguir assim com essa rota eficiente que me apontou e recomendou também").
Sem prazo: "mesmo que atrase não precisamos ter pressa. Não fiz nenhuma
promessa de entrega para 0.4".

Isto revê a decisão antiga de "alvo Linux nativo"
([47](47-estrutura-da-v0.3.md) §10.2, [57](57-mapa-de-versoes-ate-a-1.0.md)).
Rust, Qt, o ritual e os gates continuam (decisão do autor de 2026-10-08,
40.7 §7.241).

## 2. O que hoje é Linux, medido no código em 2026-10-08

Contagem por `grep` no checkout; serve de ponto de partida, não de lista
fechada.

- **Core (Rust):**
  - `std::os::unix` em 12 arquivos fora de testes: `run.rs`, `format.rs`,
    `fsops/{walk,ops,replace,transfer_batch,search}.rs`,
    `datasource/console.rs`, `handlers/remote_trust.rs`, `serial/mod.rs`,
    `tools/mod.rs` e `toolchain/import.rs`.
  - `rustix` em `owned_child.rs` (grupo de processo e sinais, a base do
    adaptador e do LSP), `datasource/console*.rs`,
    `fsops/{copy_ops,publish}.rs` e `serial/mod.rs`.
  - 31 trechos com `cfg(unix)` ou `cfg(target_os)` fora de testes.
  - Caminhos `/dev/tty*` em `build/engine.rs`, `dap/server.rs`,
    `flash/{mod,frameworks}.rs` e `handlers/run.rs`.
  - O adaptador da IDE é procurado como `kinein-adapter-<motor>` ao lado do
    core (`datasource/external.rs`), sem `std::env::consts::EXE_SUFFIX`: no
    Windows o `.exe` não seria achado (conferido em 2026-10-09).
  - O terminal usa `portable-pty` (que tem ConPTY no Windows) e
    `alacritty_terminal`.
- **Estado e segurança:** XDG (`settings.rs`, o histórico do Banco em
  `$XDG_STATE_HOME`), permissões `0700`/`0600` (histórico, consoles), escrita
  atômica com `rename` e grupos de processo para encerrar descendentes. No
  Windows: `%APPDATA%`/`%LOCALAPPDATA%`, ACL e Job Objects, a desenhar.
- **UI (C++/Qt):** `single_instance.cpp` e `main.cpp` têm código de Unix.
- **Gates e provas:** 42 scripts `bash` em `scripts/`; provas de tela com
  Xvfb e `xdotool`; empacotamento AppImage em `packaging/appimage`. O passo
  release do `verificar.sh` compila também o `kinein-adapter-sqlite`
  (2026-10-09), e o launcher do checkout (`scripts/kinein-vectis`) é `sh`.
- **Ambiente:** hoje Arch com Qt 6.12, clang 23 e GCC 16. No Windows: Qt para
  MSVC ou MinGW e Rust com o alvo MSVC, a decidir na primeira fatia.

### 2.1 Medido no Windows em 2026-10-09, no `668d890`

**Ambiente.** Windows 11 Pro 26300; Visual Studio Community 2026 (MSVC 14.51,
e o CMake 4.3.1 e o Ninja 1.13.2 que vêm com ele; Windows SDK 10.0.26100);
Rust 1.96.1 `x86_64-pc-windows-msvc`; Qt 6.12.0 `msvc2022_64` em `C:\Qt`. O
Qt veio pelo aqtinstall do commit `076e165` do repositório oficial: a 3.3.0 do
PyPI não acha o 6.11 em diante, porque a Qt mudou a estrutura do repositório
de download. O clone fica em `C:\dev\KineinVectis`, com `core.autocrlf=false`
local: o repositório não tem regra de fim de linha e o Git para Windows
converteria os scripts `bash` para CRLF. O Linux de referência passa a ser o
Fedora 44 no WSL2. Lá o `instalar-ambiente.sh --extras` fecha (com o
`fmt-devel` à parte, ver D1), o core, o adaptador e a UI compilam e a UI sobe
sem aviso de QML no Qt 6.11.2.

**O que quebra.**

- `cargo check --workspace --all-targets`: só o `kinein-core` falha, com 29
  erros na biblioteca (85 contando os testes). Os 29, por causa:
  - permissão `0600`/`0700`: `fsops/mod.rs`, `format.rs`,
    `datasource/history.rs` e `fsops/copy_ops.rs`, que copia o modo;
  - `rustix::fs` (publicar sem sobrescrever, `Errno`): `fsops/publish.rs`,
    `fsops/copy_ops.rs` e `datasource/console_fs.rs`;
  - grupo de processo e sinais: `owned_child.rs`;
  - permissão do dispositivo serial (`mode`, `gid`): `serial/mod.rs`;
  - nome de arquivo em bytes: `fsops/copy_ops.rs`;
  - PostgreSQL por socket Unix (`host_path`): `datasource/connection.rs`.
- CMake da UI: acha o Qt e o MSVC e para só no `find_package(fmt)` (D1). Com um
  `fmt` vazio, só para medir, o build para no `ui/src/single_instance.cpp`
  (`sys/socket.h`). Ele está na `kinein-ui-puro`, de que todo o resto depende:
  13 de 925 passos.

**O que compila, mas faria a coisa errada no Windows.**

- As pastas do usuário vêm de `HOME` e do XDG, e o Windows não define `HOME`:
  `settings.rs`, `datasource/history.rs`, `remote/mirror.rs`,
  `handlers/remote_mirror.rs`, `lib.rs`, `project/mod.rs`,
  `workspace/places.rs`, `tools/mod.rs` e `handlers/toolchain_install.rs`.
- O shell do terminal é o `SHELL`, senão `/bin/bash` (`terminal/session.rs`).
- A UI procura o `kinein-core` sem `.exe` (`core_client_process.cpp`), e o
  `main.cpp` solta o terminal com `fork` e `setsid`.

## 3. O porte (desenho de 2026-10-09)

Os dois primeiros itens do plano de 2026-10-08 estão resolvidos: a medição é o
§2.1, e o modo dos gates é a D2. O terceiro é o critério de pronto do porte: o
autor abre um projeto, edita, usa o terminal, compila e usa o Git na IDE, no
Windows. Banco, containers e embarcados vêm depois, na ordem do §1. No porte,
eles compilam e dizem "não suportado no Windows" onde dependem de Unix.

### 3.1 As decisões do autor (2026-10-09, respondidas como perguntas)

- **D1, o CMake.** Saem as linhas 20–38 do `CMakeLists.txt` da raiz: o
  `FetchContent` do `sqlite3` e do `asio` e o `find_package(fmt)`, que aparece
  duas vezes. Nenhum alvo as usa. Elas vieram dos commits `84cf382`, `fa6d2c1`,
  `fe2e5b4` e `424c574` (2026-09-04, configaction).
- **D2, os gates.** O `scripts/verificar.sh` completo continua no Linux, o
  Fedora 44 do WSL2, e é a referência. No Windows entra um gate curto,
  `scripts/verificar-windows.ps1` (W4).
- **D3, a privacidade no Windows.** O papel do `0600`/`0700` fica com a ACL da
  pasta do usuário: `%APPDATA%` e `%LOCALAPPDATA%` só dão acesso ao usuário, ao
  SYSTEM e aos Administradores. Não há ACL por arquivo.
- **D4, a instância única no Windows.** Usa um named pipe, pelo `QLocalServer`
  e o `QLocalSocket` (Qt Network, ligado só no Windows), com o mesmo protocolo
  de texto.

### 3.2 As fatias

A regra vale para todas: o comportamento no Linux não muda, e os testes do
Linux são a prova disso. O que difere por sistema mora num lugar só. No core
é `crates/kinein-core/src/platform/`, com a API no `mod.rs` e um arquivo por
sistema, e os domínios chamam o `platform`, sem `cfg` espalhado. Na UI vale o
mesmo: um arquivo por sistema atrás do cabeçalho que já existe.

**W1, o core compila no Windows.** Revisto ao implementar (2026-10-09): o
workspace tem `unsafe_code = "forbid"` e o core, `#![forbid(unsafe_code)]`.
Toda chamada direta à API do Windows (Job Object, `MoveFileExW`) é `unsafe`,
então o Windows usa só a biblioteca padrão. Não entra `windows-sys` e nenhum
contrato é relaxado. Um Job Object continua possível se o autor decidir um
crate de FFI próprio.

- `Cargo.toml`: o `rustix` passa para `[target.'cfg(unix)'.dependencies]`.
- `platform` (`mod.rs` com a API; `unix.rs` com o código de antes, movido;
  `windows.rs`):
  - publicar sem sobrescrever: no Linux, o `renameat2` com `NOREPLACE` de
    hoje. No Windows, o arquivo vai por `hard_link`, que falha se o destino
    existe, seguido da remoção da origem. A pasta vai por `rename`, depois de
    conferir que o destino não existe. Limite: um arquivo criado no destino
    entre a conferência e o `rename` de uma pasta seria substituído;
  - abrir sem seguir link (o motor de cópia): no Linux, o `openat` com
    `O_NOFOLLOW` a partir de `/`. No Windows, cada nó é aberto com
    `FILE_FLAG_OPEN_REPARSE_POINT` e conferido no próprio handle, que recusa
    link, junção e qualquer ponto de reparse. Limite: uma pasta já conferida
    pode ser trocada por junção antes de o filho ser aberto;
  - arquivo e pasta privados: no Linux, `0600`/`0700`; no Windows, a D3;
  - escrita para o dono, ao limpar uma cópia interrompida: no Linux, `u+rwx`;
    no Windows, sem o somente-leitura;
  - o grupo do processo: no Linux, o `process_group(0)` e o sinal no grupo de
    hoje. No Windows, a árvore de processos, encerrada por `taskkill /T /F`, e
    o filho sem janela de console. Não há `SIGTERM` para um processo sem
    janela, então o encerramento vai do EOF no stdin direto ao `taskkill`
    (`Ending::Killed`). Limite: um descendente cujo pai já saiu não está mais
    na árvore e escapa.
- O ambiente permitido (`ALLOWED_ENV`) passa a depender do sistema. No Windows
  ele leva `PATH`, `SystemRoot`, `windir`, `TEMP`, `TMP`, `USERPROFILE`,
  `APPDATA`, `LOCALAPPDATA`, `ComSpec` e `PATHEXT`; sem `SystemRoot`, muito
  programa do Windows não sobe.
- `serial`: a checagem de permissão do dispositivo é de Unix. No Windows o
  domínio responde "não suportado no Windows" até a fatia de embarcados.
- Os consoles do Banco (`datasource/console_fs.rs`) dependem do `openat` para
  não seguir link. No Windows eles respondem "não suportado" até a fatia do
  Banco, em vez de existirem com uma garantia mais fraca.
- PostgreSQL por socket Unix: só no Unix; no Windows, TCP.
- Prova: `cargo clippy --workspace --all-targets` e `cargo test --workspace`
  verdes no Linux (WSL) e no Windows. Os testes que só fazem sentido no Unix
  ganham `#[cfg(unix)]` e ficam listados no 40.7.

**W2, o core acerta o Windows.**

- `platform::dirs`. A home vem do `HOME`, e no Windows do `USERPROFILE`. Config
  e estado seguem o XDG no Linux; no Windows ficam em `%APPDATA%` e
  `%LOCALAPPDATA%`. O cache segue o XDG no Linux; no Windows fica em
  `%LOCALAPPDATA%\kinein-vectis\cache`. Os lugares do §2.1 passam a chamar
  este módulo.
- Executáveis com `std::env::consts::EXE_SUFFIX`: o adaptador
  (`datasource/external.rs`) e a busca de ferramentas no `PATH` (medir o
  `tools/mod.rs` contra o `PATHEXT`).
- O shell do terminal: no Unix, o `SHELL`, senão `/bin/bash`. No Windows, o
  `pwsh.exe`, senão o `powershell.exe`, senão o `%ComSpec%`; o `portable-pty`
  usa o ConPTY.
- Achados dos testes da W1 no Windows (40.7 §7.262), que entram aqui:
  - **o `canonicalize` devolve `\\?\C:\...`**, e nesse formato `/` não é
    separador. A raiz do projeto sai assim do `workspace.open`, e todo
    caminho que a UI manda com `/` cai fora do projeto. A correção é um
    `platform::canonicalize` que tira o prefixo quando o caminho cabe sem ele;
  - **caminho relativo gravado com `\`**: a sessão (`workspace/session.rs`)
    gravaria `src\main.rs`, e o mesmo projeto aberto no Linux não acharia o
    arquivo. Caminho relativo em formato persistido e no protocolo usa `/`
    nos dois sistemas;
  - o `owned_child` no Windows ganha teste próprio (um processo com filho,
    encerrado pelo `taskkill`). Os de hoje usam processos falsos em `sh`;
  - os testes que falham no Windows por infraestrutura passam a rodar ou
    ficam `#[cfg(unix)]` com o motivo: ferramenta falsa em `sh`, `python3`
    ausente ou asserção que compara caminho com `/`.
- Prova: teste de unidade por sistema para cada função do `platform`, e o
  `cargo test --workspace` verde no Windows.

**W3, a UI compila e abre no Windows.**

- A D1 no `CMakeLists.txt`, e os presets `windows-msvc-debug` e
  `windows-msvc-release` no `CMakePresets.json` (Ninja, `cl`, condição
  `hostSystemName == Windows`).
- `single_instance`: o transporte vira um arquivo por sistema. O
  `single_instance_unix.cpp` é o de hoje; o `single_instance_windows.cpp` é a
  D4. O protocolo de texto fica onde está.
- `main.cpp`: o `fork`/`setsid` que solta o terminal fica só no Unix. No
  Windows o executável é de subsistema gráfico (`WIN32_EXECUTABLE`), e o
  terminal já devolve o prompt.
- A UI procura o `kinein-core` com `.exe`.
- Um lançador, `scripts/kinein-vectis.ps1`, par do `scripts/kinein-vectis`: põe
  o `bin` do Qt no `PATH` do processo e acha o core. Não há `windeployqt` no
  checkout.
- Prova: o smoke sem janela (`QT_QPA_PLATFORM=offscreen`, 20 s, sem aviso de
  QML) e a foto da janela real. Na foto, o agente opera a tela com o mouse e o
  teclado do autor, que autorizou isso em 2026-10-09.

**W4, o gate do Windows (D2).** O `scripts/verificar-windows.ps1` roda:

- `cargo fmt --check`;
- `cargo clippy --workspace --all-targets -D warnings`;
- `cargo test --workspace`;
- o build da UI com o preset `windows-msvc-debug`;
- o smoke da W3.

O `verificar.sh` completo continua no WSL, no mesmo commit.

**W5, o critério de pronto.** O autor abre um projeto, edita, usa o terminal,
compila e usa o Git na IDE no Windows. A partir daí, os aceites pendentes (7,
7b, 14a, 14b, 13a, 13b, 9a.3) podem ser feitos lá. Nessa hora o ambiente do
Windows sai do §2.1 e vai para o `contribuindo/02`, para ter um dono só.

Cada fatia é um commit no branch `porte-windows`, com o gate do Linux verde no
WSL e o build do Windows medido. Os commits não sobem para o GitHub sem
pedido do autor.

## 4. A edição especial (depois da 0.4.0)

O tema é o atrito real de quem desenvolve embarcados no Windows. A lista abaixo
é ponto de partida, sem desenho; o desenho é feito lá, com as ferramentas
medidas.

- O projeto e as toolchains podem morar no WSL2 com a IDE no Windows. A IDE
  já tem Remote SSH com escritor único (série 0.3); avaliar se o mesmo modelo
  serve ao WSL.
- USB e serial: placa como `COMx` no Windows, como `/dev/ttyACM*` no WSL2
  (com `usbipd-win` para anexar o dispositivo). Gravação e depuração
  (probe-rs, OpenOCD, esptool) nativas ou no WSL2.
- Containers: Docker Desktop ou Podman no Windows, com a fatia dos
  containers (59, 16b) como base.

## 5. O que não muda

Contrato e desenho no documento dono antes do código; uma fatia por commit com
o gate verde; registro datado no 40.7 dizendo o que provou e o que não fez; o
aceite com mouse e teclado do autor.

## 6. A organização da documentação (depois do porte, antes da 0.3.9)

**Classe: PLANO, sem desenho ainda.** O desenho é feito depois do porte, num
documento próprio, que passa a ser o dono desta seção. Aqui ficam o objetivo e
o ponto de partida medido.

**O objetivo do autor** (§1, item 2): separar a documentação por contexto,
explícita e bem separada, para que uma pessoa ou um agente de IA leia só o que
vai mexer. Se a medição pedir, a arquitetura também.

**O ponto de partida, medido em 2026-10-09 no `668d890`** (arquivos versionados):

```text
codigo, sem Markdown     206752 linhas
  Rust (crates/)         110412 linhas em 414 arquivos
  QML (ui/ e harness)     71982 linhas em 548 arquivos
  scripts .sh e .py       13917 linhas em 85 arquivos
  C++ (ui/)               10441 linhas em 67 arquivos
Markdown                  96008 linhas em 153 arquivos
  40.7, o registro        12007 linhas, 710 KB
  03, o protocolo IPC      4914 linhas, 275 KB
  59, a 0.3.9              1805 linhas
  40, o estado             1581 linhas, 415 delas no cabecalho, antes do §1
```

O autor estimava "quase 190 mil linhas de código"; a medida, sem Markdown, dá
206752.

**O que se viu nesta sessão** (o agente que fez o porte, 2026-10-09):

- Para saber onde o projeto está, a ordem de leitura do `contribuindo/05`
  passa por 00, o cabeçalho do 40, o 40.7 recente, o roadmap da versão e o
  `ARCHITECTURE.md` antes do primeiro arquivo de código.
- O 40.7 é grande demais para um agente ler de uma vez, e só se consegue lê-lo
  por trechos.
- A ordem das versões aparecia, com pequenas diferenças, no cabeçalho do 40, no
  57, no 59, no 47 e no 60.

**O que o desenho precisa responder** (nada decidido):

- Quais são os contextos: plataforma, core por domínio, UI, banco, embarcados,
  gates.
- O que cada contexto obriga a ler, e o que dispensa.
- Como o 40.7 se divide sem reescrever o LOG (por exemplo, por versão), e o
  cabeçalho do 40 se reduz ao handoff.
- Que gate prova a separação: o `verificar-docs.sh` e o
  `verificar-links-docs.sh` já existem e seriam estendidos, não duplicados.
