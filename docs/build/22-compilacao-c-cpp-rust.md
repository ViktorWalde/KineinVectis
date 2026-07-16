# 22 — Comandos de compilação C/C++ e Rust (referência prática)

> **Status:** ativo (criado em 2026-07-11).
> **Para quem:** referência rápida e intuitiva de "o que eu quero fazer →
> qual comando rodar" para C, C++ e Rust no Linux (máquina de referência:
> Arch). Futuramente vira uma UI na IDE; por ora este `.md` já ajuda.
> **Como ler:** cada bloco é "objetivo → comando → o que ele faz". Os
> comandos são os mesmos que o core da IDE orquestra (ver mapeamento no fim).

---

## Rust (cargo) — o dia a dia

O `cargo` é o build system + gerenciador de pacotes. Rode na raiz do projeto
(onde está o `Cargo.toml`).

| Objetivo | Comando | O que faz |
| --- | --- | --- |
| **Compilar** (debug) | `cargo build` | Compila em `target/debug/`. Rápido, sem otimização, com símbolos de debug. |
| **Compilar release** | `cargo build --release` | Otimizado (`-O`), em `target/release/`. Mais lento de compilar, rápido de rodar. |
| **Rodar** | `cargo run` | Compila (se preciso) e executa o binário principal. |
| **Rodar release** | `cargo run --release` | Idem, otimizado. |
| **Rodar passando args** | `cargo run -- --flag valor` | Tudo depois de `--` vai para o SEU programa, não para o cargo. |
| **Checagem rápida** | `cargo check` | Só verifica se COMPILA (erros/tipos), sem gerar binário. Bem mais rápido que `build`. |
| **Testar** | `cargo test` | Compila e roda todos os testes (`#[test]` + doctests). |
| **Testar um só** | `cargo test nome_do_teste` | Filtra pelo nome (substring). |
| **Lint (clippy)** | `cargo clippy` | Análise estática além do compilador (idioms, bugs comuns). |
| **Lint estrito** | `cargo clippy -- -D warnings` | Trata todo warning como erro (o padrão de rigor deste repo). |
| **Formatar** | `cargo fmt` | Reformata o código com o `rustfmt`. |
| **Só checar formato** | `cargo fmt --check` | Não altera; falha se algo estiver fora do padrão (útil no CI/gate). |
| **Documentação** | `cargo doc --open` | Gera e abre a doc HTML do projeto e das dependências. |
| **Limpar** | `cargo clean` | Apaga `target/`. Use quando o cache estiver estranho. |

### Escolhendo O QUE compilar (workspaces, bins, examples)

| Objetivo | Comando |
| --- | --- |
| Um binário específico (vários `[[bin]]`) | `cargo run --bin nome` / `cargo build --bin nome` |
| Um pacote do workspace | `cargo build -p nome_do_pacote` |
| Um example (`examples/foo.rs`) | `cargo run --example foo` |
| Tudo do workspace | `cargo build --workspace` |
| Todos os alvos (libs, bins, tests, examples) | `cargo build --all-targets` |

### Features (compilação condicional)

| Objetivo | Comando |
| --- | --- |
| Ligar features | `cargo build --features "a b"` |
| Todas as features | `cargo build --all-features` |
| Nenhuma feature default | `cargo build --no-default-features` |

### Toolchain (rustup)

| Objetivo | Comando |
| --- | --- |
| Ver toolchain ativa | `rustup show` |
| Atualizar | `rustup update` |
| Usar nightly num comando | `cargo +nightly build` |
| Adicionar um target (cross) | `rustup target add x86_64-unknown-linux-musl` |

---

## C / C++ — compilação direta (gcc / clang)

Para um arquivo ou poucos, chamar o compilador direto já resolve. Convenção:
`gcc`/`g++` (GNU) e `clang`/`clang++` (LLVM) — a máquina de referência tem os
dois. Use `g++`/`clang++` para C++ (linka a stdlib de C++), `gcc`/`clang`
para C.

| Objetivo | Comando | O que faz |
| --- | --- | --- |
| **Compilar 1 arquivo** | `g++ main.cpp -o app` | Compila e linka em `./app`. |
| **Rodar** | `./app` | Executa o binário gerado. |
| **Vários arquivos** | `g++ main.cpp util.cpp -o app` | Compila e linka juntos. |
| **Padrão da linguagem** | `g++ -std=c++23 main.cpp -o app` | Escolhe o dialeto (`c++17/20/23`; C: `-std=c17`). |
| **Warnings** | `g++ -Wall -Wextra main.cpp -o app` | Liga avisos úteis. Rigor: `-Werror` (warning vira erro). |
| **Debug** | `g++ -g main.cpp -o app` | Inclui símbolos p/ o debugger (lldb/gdb). |
| **Otimizado** | `g++ -O2 main.cpp -o app` | Otimização (release). `-O0` = nenhuma (debug). |
| **Só compilar (sem linkar)** | `g++ -c main.cpp -o main.o` | Gera objeto `.o`; útil em builds grandes. |
| **Header em outra pasta** | `g++ -Iinclude main.cpp -o app` | Adiciona `include/` ao caminho de headers. |
| **Linkar biblioteca** | `g++ main.cpp -lpthread -o app` | Linka `libpthread`. `-Lpasta` adiciona onde procurar. |

### Sanitizers (achar bugs de memória/UB) — C e C++

| Objetivo | Comando |
| --- | --- |
| Address (buffer overflow, use-after-free) | `g++ -fsanitize=address -g main.cpp -o app` |
| Undefined Behavior | `g++ -fsanitize=undefined -g main.cpp -o app` |
| Thread (data races) | `g++ -fsanitize=thread -g main.cpp -o app` |

> Combine `-fsanitize=address,undefined`. Rode o binário normalmente; o
> sanitizer aborta com um relatório quando detecta o problema.

---

## C / C++ — projetos com CMake

Para projetos de verdade (múltiplos arquivos, libs, dependências), o CMake é
o padrão. Ele **gera** o build (Ninja/Make) a partir do `CMakeLists.txt`.

| Objetivo | Comando | O que faz |
| --- | --- | --- |
| **Configurar** (via preset) | `cmake --preset dev` | Lê `CMakePresets.json`, gera o build dir. Exporta `compile_commands.json`. |
| **Configurar** (manual) | `cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Debug` | `-S` fonte, `-B` build dir, `-G` gerador, `-D` variáveis. |
| **Compilar** (via preset) | `cmake --build --preset dev` | Compila o que o configure preparou. |
| **Compilar** (build dir) | `cmake --build build` | Idem, apontando o build dir. |
| **Compilar 1 alvo** | `cmake --build build --target nome` | Só aquele executável/lib. |
| **Rodar** | `./build/nome_do_executavel` | O CMake não "roda"; execute o binário gerado. |
| **Instalar** | `cmake --install build` | Copia artefatos p/ o prefixo de instalação. |
| **Limpar** | `rm -rf build` | Apaga o build dir. Reconfigure do zero se o cache ficar stale (ver ContextoIA). |

### CTest — testes de projetos CMake

| Objetivo | Comando |
| --- | --- |
| Rodar todos os testes | `ctest --test-dir build` |
| Saída detalhada | `ctest --test-dir build -V` |
| Um teste por nome/regex | `ctest --test-dir build -R nome_do_caso` |
| Parar no primeiro erro | `ctest --test-dir build --stop-on-failure` |

### compile_commands.json (essencial p/ o clangd/IDE)

O `clangd` (autocomplete, erros em tempo real, ir-para-definição) precisa
saber COM QUE FLAGS cada arquivo compila. O CMake exporta isso:

- Configure com `-DCMAKE_EXPORT_COMPILE_COMMANDS=ON` (os presets deste repo já
  fazem). Gera `build/compile_commands.json`.
- **Sem esse arquivo, o clangd não resolve os includes** (ex.: `std::cout`
  não completa). Na IDE: rode **"CMake: Configure"** para gerar/atualizar.

### Qualidade C/C++

| Objetivo | Comando |
| --- | --- |
| Formatar | `clang-format -i arquivo.cpp` (`-i` = edita no lugar) |
| Análise estática | `clang-tidy arquivo.cpp -p build` (`-p` = onde está o compile_commands) |

---

## Como isso aparece na IDE Kinein Vectis

A IDE **orquestra** exatamente esses comandos (não reimplementa). Mapeamento
rápido dos atalhos/comandos para o que roda por baixo:

| Na IDE | Roda por baixo |
| --- | --- |
| Build (`Ctrl+F9`) | `cargo build` / `cmake --build` |
| Testes (`Ctrl+Shift+F9`) | `cargo test` / `ctest` |
| Análise (`Ctrl+Shift+L`) | `cargo clippy` / (C++: clang-tidy, evoluindo) |
| Formatar (`Ctrl+Alt+L`) | `rustfmt` / `clang-format` |
| "CMake: Configure" (palette) | `cmake --preset ...` + exporta `compile_commands.json` |
| "Cargo: Check" (palette) | `cargo check` |
| Erros em tempo real no editor | clangd / rust-analyzer (LSP); o rust-analyzer roda `cargo check` no save sozinho (flycheck) |
| Debug (`Shift+F9`) | `lldb-dap` (DAP) no binário compilado |

**Perfil de rigor (`Ctrl+Alt+S` → Configurações):** regula as flags que o
Build e a Análise passam **ao seu projeto** (Rust, v1) — não muda o gate do
próprio Kinein:

| Perfil | `cargo clippy` (Análise) | `cargo build` (Build) |
| --- | --- | --- |
| **Estrito** (padrão) | `-- -W clippy::pedantic -W clippy::nursery -D warnings` | `RUSTFLAGS="-D warnings"` |
| **Equilibrado** | clippy default (sem flags extras) | sem `RUSTFLAGS` |
| **Relaxado** | `-- -A clippy::all -W clippy::correctness` | sem `RUSTFLAGS` |

Detalhe do contrato IPC de cada um: `docs/arquitetura/03-ipc-protocol.md`.
