[# Kinein Vectis

Kinein Vectis é uma IDE open source, Linux-first, para **C, C++, Rust e
Python** — no desktop e em sistemas embarcados. A interface nativa em Qt/QML
conversa por JSON-RPC local com um core em Rust, e o core **orquestra
ferramentas maduras** — CMake, Cargo, clangd, rust-analyzer, basedpyright,
LLDB, debugpy, gdb, Git, docker/podman, esptool, mpremote, probe-rs… — em vez
de reimplementar compiladores, servidores de linguagem, build systems ou
depuradores.

O projeto está em desenvolvimento ativo, em **teste fechado**. A versão de
teste atual é a `0.1.0`, distribuída como AppImage para Linux x86_64.

## A ideia

Uma IDE que **lê o seu projeto e diz o que leu** — qual toolchain, qual
padrão, quais includes, qual interpretador, qual placa está no USB, qual
container responde, qual banco existe nesta máquina — e depois **faz o que
você mandar, com o que você escolheu**. Ela não decide por você:

- **A liberdade de escolha vem na caixa.** Compilador, preset, kit (chip,
  alvo, sysroot, SVD, depurador), language server, formatador, motor de
  container, versão de toolchain: tudo é escolha sua, visível e trocável.
  A IDE sugere com o que encontrou na sua máquina e no seu projeto, mostra
  a prévia do que vai mudar, e só muda com o seu consentimento.
- **Nada é instalado em silêncio.** Falta uma ferramenta? A IDE mostra o
  passo oficial da sua distro (ou o instalador do fabricante), e você
  executa. Toolchains que ela mesma baixa vão para a pasta dela, com o
  SHA-256 conferido antes de desempacotar. Ela nunca roda `sudo`.
- **Rigor é um preset, não uma imposição.** O próprio projeto compila com
  `-Werror`, clippy pedante, sanitizers e clang-tidy nos presets
  `*-strict`/`*-hardened` — e oferece os `dev-local` sem sanitizers para o
  dia a dia. Para os **seus** projetos, os presets, os flags e as análises
  são os do seu `CMakePresets.json`/`Cargo.toml`; a IDE os lê, não os
  impõe.
- **Local e sem telemetria.** Nada sai da máquina sem uma ação sua.
- **Sem IA embutida.** Não há chat nem painel de assistente: agentes de
  linha de comando (Claude Code, Codex, …) rodam no terminal integrado
  como qualquer outro programa, se você quiser — e o projeto documenta
  como colaborar com ou sem eles (ver *Contribuir*).

## O que já funciona

- projetos C++/CMake, Rust/Cargo e Python, com workspaces recentes; o
  **projeto inteiro lido** por um índice próprio (pastas, arquivos,
  declarações) com o contexto de compilador de cada arquivo;
- editor com múltiplas abas, recuperação de rascunhos, Tree-sitter
  incremental (realce, folding, estrutura), completion, diagnósticos,
  navegação, rename e quick fixes com prévia — pelo clangd, rust-analyzer
  e basedpyright; a aba **Símbolos** (estrutura do arquivo + busca de
  declarações por nome no projeto);
- build, testes (gtest/Catch2, `cargo test`, `pytest`), análise (clippy,
  ruff, clang-tidy), cobertura (LCOV pintada na calha), execução numa aba
  de terminal e depuração por DAP (LLDB, `gdb -i dap`, debugpy);
- terminal PTY real com múltiplas sessões;
- **Git como janela em pé** à esquerda (Commit e Log, grafo, refs, amend,
  commit-e-push), o diff e o commit abrindo no editor, blame na calha;
- **ambiente do projeto**: catálogo de bibliotecas C/C++, ações de
  configuração de CMake/Cargo com prévia e consentimento, kits por alvo,
  passo oficial de instalação por distro;
- **banco de dados** nativo (PostgreSQL/TimescaleDB, SQLite, MongoDB):
  descobre o que responde nesta máquina, cria um SQLite ou um servidor em
  container, remove o que criou; perfis sem senha em disco; consultas e
  esquema;
- **containers** (Docker ou Podman): motor, ciclo de vida como jobs, logs
  e shell numa aba de terminal, compose do projeto;
- **observabilidade**: o Grafana pela HTTP API (o que ele já observa deste
  projeto);
- **embarcados** (painel em abas Placa · Projeto · Gravar · Kit): portas
  seriais sem abri-las, identidade do chip, monitor serial, o modelo do
  projeto (ESP-IDF, Zephyr, pico-sdk, PlatformIO, STM32Cube, Rust bare
  metal, MicroPython, Yocto, Buildroot), gravar por esptool/probe-rs/
  picotool/dfu-util/`idf.py`/`west`/`pio`, permissões medidas com o passo
  oficial, QEMU + `gdb -i dap`;
- **Python** como vertical nativa: interpretador do projeto, `.venv` de um
  clique com `uv`, ruff, pytest, debugpy, MicroPython pelo `mpremote`
  (arquivos da placa, firmware oficial com checksum, stubs);
- **toolchains por alvo**: catálogo conferido, instalação com SHA-256,
  sysroot lido, kit importado de SDK.

## Testar ou instalar

O AppImage inclui a interface, o `kinein-core`, o runtime Qt/QML, plugins,
licenças e o manual. Rust e Qt não precisam estar instalados para abrir a IDE;
as ferramentas usadas pelos projetos continuam opcionais e externas.

- Instalar, verificar o SHA-256, atualizar, gerar uma versão:
  [DocsPublic/tutorial.md](DocsPublic/tutorial.md).
- Usar a IDE (recursos, fluxos, atalhos): [DocsPublic/manual.md](DocsPublic/manual.md).
- Compilar pelo código-fonte: [DocsPublic/build/como-executar.md](DocsPublic/build/como-executar.md).

## Arquitetura

```text
Qt/QML Frontend ─ apresenta e recebe ações
       ↕ JSON-RPC local, tipado (kinein-protocol)
Rust Core ─ valida, mantém estado, chama as ferramentas; operações longas
            são jobs canceláveis; respostas lentas saem do laço
       ↕
CMake · Cargo · clangd · rust-analyzer · basedpyright · ruff · LLDB · debugpy
· gdb · Git · docker/podman · esptool/mpremote/probe-rs · toolchains
```

O contrato entre as duas metades é público e versionado:
[DocsPublic/arquitetura/03-ipc-protocol.md](DocsPublic/arquitetura/03-ipc-protocol.md).
As regras que sustentam o desenho (e que o gate verifica) estão em
[DocsPublic/arquitetura/ARCHITECTURE.md](DocsPublic/arquitetura/ARCHITECTURE.md).

## Contribuir

Quem chega — para corrigir uma linha ou para uma etapa inteira, **com ou
sem um agente de IA** — começa por
[DocsPublic/contribuindo/](DocsPublic/contribuindo/README.md): o que o
projeto é e recusa ser, como preparar o ambiente, o ritual de uma
mudança (desenhar → contrato → core → UI → provar → documentar), os gates
que dizem não, e como trabalhar com um agente sem que ele quebre as
regras. O índice de toda a documentação é
[DocsPublic/README.md](DocsPublic/README.md).

## Plataforma

- Linux x86_64;
- baseline do AppImage: glibc 2.36, equivalente ao Debian 12 ou posterior;
- desktop Linux com pilha gráfica e fontes normais;
- Windows ainda não é suportado.

## Privacidade do repositório e distribuição do código

O repositório de desenvolvimento permanece privado durante o teste fechado.
Se o código for entregue a terceiros, será usada uma cópia sanitizada, sem o
histórico Git privado, com o código do projeto e a documentação pública
(`README.md`, `DocsPublic/`). Registros internos de sessão e evidências
(`DocsPrivate/`) não fazem parte dessa cópia.

## Licença

Kinein Vectis é disponibilizado sob licença dupla MIT ou Apache-2.0. Consulte
`LICENSE-MIT.txt` e `LICENSE-APACHE-2.0.txt`.
](https://github.com/ViktorWalde/KineinVectis)
