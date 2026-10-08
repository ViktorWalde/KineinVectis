# 60 — Windows: adaptar a IDE, fechar a 0.3.9 e a 0.4.0, e a edição especial

> **Classe: PLANO** (`DocsPublic/README.md`). Decisão do autor em 2026-10-08,
> à noite. Cada fatia ainda tem desenho próprio antes do código, no documento
> dono, como sempre.

## 1. A decisão

O autor está migrando o sistema operacional de desenvolvimento para Windows,
por necessidade de um projeto que pode se tornar fonte de renda. Nas palavras
dele: "nesse meio tempo podemos adaptar a IDE para multiplataforma e usá-la no
Windows, finalizando a 0.3.9, vamos para a 0.4.0 e, finalizando ela, vamos ter
uma 'special edition' onde vamos buscar resolver o problema/atrito do Windows
e WSL2 com a IDE, para parte de embarcados e comunicação USB".

A ordem:

1. **Adaptar a IDE para rodar no Windows**, o suficiente para o autor
   continuar desenvolvendo a própria IDE nela, em paralelo à migração.
2. **Fechar a 0.3.9**: 9a.3 (o campo "Adaptador" no formulário), a fatia dos
   containers (16b), as provas finais (15) e o pente fino (16)
   ([59](59-fechamento-da-0.3.9.md) §2). O corte de escopo de 2026-10-08 já
   tirou da 0.3.9 o 9b/9c, o 10 e o 11 (59 §2.3).
3. **A 0.4.0**: embarcados ([52](52-arquitetura-executavel-da-0.4.md)).
4. **A edição especial**, depois da 0.4.0: resolver o atrito do Windows com o
   WSL2 na IDE, para embarcados e comunicação USB.

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
  - O terminal usa `portable-pty` (que tem ConPTY no Windows) e
    `alacritty_terminal`.
- **Estado e segurança:** XDG (`settings.rs`, o histórico do Banco em
  `$XDG_STATE_HOME`), permissões `0700`/`0600` (histórico, consoles), escrita
  atômica com `rename` e grupos de processo para encerrar descendentes. No
  Windows: `%APPDATA%`/`%LOCALAPPDATA%`, ACL e Job Objects, a desenhar.
- **UI (C++/Qt):** `single_instance.cpp` e `main.cpp` têm código de Unix.
- **Gates e provas:** 42 scripts `bash` em `scripts/`; provas de tela com
  Xvfb e `xdotool`; empacotamento AppImage em `packaging/appimage`.
- **Ambiente:** hoje Arch com Qt 6.12, clang 23 e GCC 16. No Windows: Qt para
  MSVC ou MinGW e Rust com o alvo MSVC, a decidir na primeira fatia.

## 3. A primeira fatia no Windows (a desenhar lá)

Medir antes de mudar, como no resto do projeto:

1. Clonar e compilar o core (`cargo build`, `cargo test`) e a UI no Windows.
   O que quebra vira a lista de fatias, por dono (core, UI, gates).
2. Decidir como os gates rodam lá: Git Bash, WSL ou reescrita dos scripts
   críticos, e qual é o equivalente da prova de tela (o Xvfb não existe no
   Windows).
3. O mínimo para o autor usar a IDE no Windows: abrir projeto, editar,
   terminal, build, Git. Banco, containers e embarcados vêm depois, na ordem
   do §1.

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
