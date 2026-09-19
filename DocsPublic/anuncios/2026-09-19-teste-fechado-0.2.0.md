# Anúncio — teste fechado da Kinein Vectis 0.2.0 (Discord, 2026-09-19)

> Texto pronto para colar no Discord. O SHA-256 e o link entram no fim,
> depois do build (`dist/Kinein-Vectis-0.2.0-x86_64.AppImage.sha256`).
> Duas versões: a curta (mensagem fixada) e a longa (post do canal).

---

## Versão curta (mensagem fixada)

**Kinein Vectis 0.2.0 — teste fechado** 🐧

IDE open source, Linux-first, para C, C++, Rust e Python (desktop e embarcados). Interface nativa Qt/QML + core em Rust, orquestrando ferramentas que você já usa (CMake, Cargo, clangd, rust-analyzer, basedpyright, LLDB, gdb, debugpy, Git, docker/podman, esptool, mpremote, probe-rs…). Sem telemetria, sem IA embutida, nada instalado sem você mandar.

📦 **Download**: `Kinein-Vectis-0.2.0-x86_64.AppImage` (+ `.sha256`, `instalar-kinein-vectis.sh`, `Tutorial.md`) — nos anexos/pins deste canal.
✅ Linux x86_64, glibc ≥ 2.36 (Debian 12+, Ubuntu 22.04+, Fedora 37+, Arch…). Não precisa de Rust nem Qt instalados.
▶️ `chmod +x Kinein-Vectis-0.2.0-x86_64.AppImage && ./Kinein-Vectis-0.2.0-x86_64.AppImage` (ou rode o `instalar-kinein-vectis.sh` para ter o ícone no menu).
🐛 Relatos em #kinein-bugs: distro, o que você fez, o que esperava, e o arquivo `~/.cache/kinein-vectis/logs/kinein-ui-erros.txt`.

É **teste fechado**: a IDE está em desenvolvimento ativo e você vai achar arestas. É exatamente isso que a gente quer ouvir.

---

## Versão longa (post do canal)

**Kinein Vectis 0.2.0 — abrindo o teste fechado**

Oi, pessoal. Depois de alguns meses de desenvolvimento, chegou a hora de colocar a Kinein Vectis na mão de mais gente. É uma **IDE open source (MIT/Apache-2.0), Linux-first, para C, C++, Rust e Python** — no desktop e em sistemas embarcados.

**A ideia em uma frase:** uma IDE que **lê o seu projeto e a sua máquina e diz o que leu** — qual toolchain, qual padrão, quais includes, qual interpretador, qual placa está no USB, qual container responde, qual banco existe aqui — e depois **faz o que você mandar, com o que você escolheu**.

Alguns princípios que explicam como ela se comporta:

- **A liberdade de escolha vem na caixa.** Compilador, preset, kit (chip/alvo/sysroot/depurador), language server, formatador, motor de container: tudo é escolha sua, visível e trocável. A IDE sugere com base no que encontrou, mostra a prévia do que vai mudar e só muda com o seu OK.
- **Nada é instalado em silêncio.** Falta uma ferramenta? Ela mostra o passo oficial da sua distro — e você executa. Ela nunca roda `sudo`.
- **Local, sem telemetria, sem IA embutida.** Nada sai da máquina sem uma ação sua. Não tem chat nem assistente: se você usa agentes de linha de comando, eles rodam no terminal integrado como qualquer programa.
- **Ela orquestra, não reimplementa.** Compilador, LSP, build system, depurador e emulador de terminal são as ferramentas maduras de sempre; a IDE integra e mostra de onde cada informação veio.

**O que dá para testar hoje (0.2.0):**

- Projetos **C++/CMake, Rust/Cargo e Python**: abrir, criar, editor com abas, realce/estrutura por Tree-sitter, completion, diagnósticos, navegação, rename e quick fixes com prévia (clangd, rust-analyzer, basedpyright).
- **Aba Símbolos** (borda direita, `Alt+7`): estrutura do arquivo e busca de funções/tipos por nome no projeto inteiro.
- **Build, testes, análise, cobertura, executar e depurar** (LLDB, `gdb -i dap`, debugpy). A execução roda numa aba do terminal integrado (PTY real, várias sessões).
- **Git como janela em pé à esquerda** (Commit e Log com grafo e refs, amend, commit-e-push, stash); o diff e o commit abrem no editor.
- **Banco de dados** (PostgreSQL/TimescaleDB, SQLite, MongoDB): descobre o que responde na sua máquina, cria um SQLite ou sobe um servidor em container, consulta e mostra o esquema, remove o que criou. Senha nunca vai para o disco.
- **Containers** (Docker ou Podman): motor, ciclo de vida, logs e shell numa aba de terminal, compose do projeto.
- **Embarcados** (painel Placa · Projeto · Gravar · Kit): portas seriais sem abri-las, identidade do chip, monitor serial, ESP-IDF/Zephyr/pico-sdk/PlatformIO/STM32Cube/Rust bare metal/MicroPython/Yocto/Buildroot reconhecidos, gravar por esptool/probe-rs/picotool/dfu-util/`idf.py`/`west`/`pio`, QEMU + gdb.
- **Python** de ponta a ponta: interpretador do projeto, `.venv` de um clique com `uv`, ruff, pytest, debugpy, MicroPython pelo `mpremote`.
- **Grafana** pela HTTP API (o que ele já observa deste projeto).

**O que ainda NÃO está polido (para você não perder tempo):**

- O painel do Grafana ainda está com a cara antiga (o redesenho é a próxima fatia).
- Edição "inteligente" (indentação automática por gramática, assinatura enquanto digita, inlay hints, formatação ao salvar pelo LSP, multi-cursor) está começando agora — é o foco da próxima etapa. O Enter hoje repete a indentação da linha e continua comentários; só isso.
- Windows e ARM64 não são suportados.
- É teste fechado: a interface pode mudar entre versões e não há atualização automática.

**Como instalar (2 minutos):**

1. Baixe os quatro arquivos fixados: `Kinein-Vectis-0.2.0-x86_64.AppImage`, `Kinein-Vectis-0.2.0-x86_64.AppImage.sha256`, `instalar-kinein-vectis.sh`, `Tutorial.md`.
2. Confira o download: `sha256sum -c Kinein-Vectis-0.2.0-x86_64.AppImage.sha256` (tem que terminar em `OK`).
3. Rode: `chmod +x Kinein-Vectis-0.2.0-x86_64.AppImage && ./Kinein-Vectis-0.2.0-x86_64.AppImage`. Ou `bash instalar-kinein-vectis.sh` para ter o ícone no menu.
4. Deu erro de FUSE? `APPIMAGE_EXTRACT_AND_RUN=1 ./Kinein-Vectis-0.2.0-x86_64.AppImage`.

Requisitos: Linux x86_64, glibc 2.36+ (Debian 12, Ubuntu 22.04, Fedora 37, Arch e mais novos), sessão Wayland ou X11. Não precisa de Rust, Qt, CMake nem GPU. As ferramentas dos **seus** projetos (compiladores, clangd, rust-analyzer, docker…) continuam sendo as suas — a IDE diz o que falta e como instalar.

**O que a gente mais quer que você teste:**

- Abra um projeto **seu** (CMake, Cargo ou Python) e conte: o que a IDE entendeu certo e errado sobre ele?
- O fluxo diário: editar → compilar → rodar → depurar → commitar. Onde travou, onde ficou lento, o que faltou?
- Se você tem uma placa (ESP32, Pico, STM32…): o painel Embarcados achou? Identificou? Gravou?
- Se você usa Docker/Podman ou Postgres/Mongo local: os painéis acharam o que existe?

**Como relatar (em #kinein-bugs):**

- Distro e versão; Wayland ou X11.
- O que você fez (passo a passo curto), o que esperava, o que aconteceu. Print ajuda muito.
- O arquivo `~/.cache/kinein-vectis/logs/kinein-ui-erros.txt` e, se a janela nem abriu, a saída do terminal ao rodar o AppImage.
- Ideias e "no VS Code/CLion isso funciona assim" também valem — é a régua que a gente usa.

Obrigado por testar. Cada relato vira uma fatia com teste e prova no repositório, e o changelog da próxima versão vai citar o que veio daqui. 🙏

---

## Checklist antes de postar

- [ ] `dist/Kinein-Vectis-0.2.0-x86_64.AppImage` gerado e `sha256sum -c` OK
- [ ] `bash scripts/testar-appimage.sh` e `testar-appimage-portatil.sh` verdes
- [ ] os quatro arquivos anexados/fixados (Discord limita anexos por tamanho — se o AppImage não couber, hospede e cole o link + o texto do hash na própria mensagem)
- [ ] o hash SHA-256 colado na mensagem (segundo canal além do `.sha256`)
- [ ] um canal de bugs criado (#kinein-bugs) e outro de ideias
