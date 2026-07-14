# Kinein Vectis

**Kinein Vectis** é uma IDE open source, Linux-first, rígida por padrão e visualmente plug and play, criada para desenvolvimento moderno em C++, Java, Python, backend convencional e sistemas embarcados.

A arquitetura do projeto separa claramente:

- **Frontend visual:** Qt/QML.
- **Core/backend:** Rust.
- **Ferramentas externas:** clangd, jdtls, pyright, CMake, Ninja, Maven, Gradle, uv, Git, GDB, LLDB, QEMU, OpenOCD, Ollama/GPT/Claude CLI.

O Kinein Vectis não tenta reimplementar compiladores, parsers, debugadores ou language servers. Ele atua como uma camada visual profissional, rigorosa e integrada sobre ferramentas open source consolidadas.

## Objetivo

Criar uma IDE com experiência visual familiar para usuários acostumados ao ecossistema JetBrains, mas com filosofia:

- open source;
- sem telemetria obrigatória;
- Linux-first;
- alta performance;
- baixo consumo de memória;
- qualidade máxima por padrão;
- configuração visual em vez de configuração manual excessiva;
- suporte profissional a C++, Java, Python, backend e embarcados;
- integração opcional com IA local e/ou externa.

## Nome

- Nome do produto: **Kinein Vectis**
- Diretório do projeto: `kinein-vectis`
- Binário principal futuro: `kinein-vectis`
- Core Rust/daemon: `kinein-core`
- Nome interno de crate no código Rust: `kinein_core`

## Decisão técnica inicial

O projeto deve começar como **Rust workspace** no CLion.

Motivo:

1. O core/backend é o cérebro da IDE.
2. A UI Qt/QML pode ser adicionada depois como subprojeto `ui/`.
3. O core Rust pode ser testado desde o primeiro dia sem interface gráfica.
4. O protocolo IPC entre UI e core pode ser definido antes da interface definitiva.
5. A arquitetura evita um monólito C++/Qt difícil de manter.

A UI Qt/QML será adicionada posteriormente como processo separado ou como subprojeto CMake que se comunica com o core Rust via IPC local.

## Primeira meta técnica

Antes de editor, LSP, Git ou CMake, o primeiro marco técnico é:

```text
Qt/QML UI ou cliente CLI
        ↓
IPC local
        ↓
Rust Core
        ↓
Resposta: core.pong
```

O MVP inicial pode começar até sem Qt: primeiro um `kinein-core` e um `kinein-cli` para validar protocolo, comandos, settings, logs e strict mode.

## Como executar

Veja **[COMO_EXECUTAR.md](COMO_EXECUTAR.md)**. Resumo: a IDE é offline/local —
o atalho "Kinein Vectis" no menu (ou `./scripts/kinein-vectis`) sobe a UI,
que inicia o core Rust automaticamente.

## Documentação

Toda a documentação está indexada em **[docs/README.md](docs/README.md)**:

- `docs/00–16` — especificações canônicas em vigor (arquitetura, IPC, strict mode…);
- `docs/planning/` — visão e planejamento histórico (masters e blueprint);
- `docs/quality/` — políticas de toolchain, Quality Center e padrões profissionais;
- `docs/subsystems/` — especificações de subsistemas futuros;
- `ContextoIA.md` — estado real do desenvolvimento e sincronização entre agentes de IA.

## Estado atual

MVP 0.1 (core mínimo), MVP 0.2 (tool detection), MVP 0.3 (workspace), MVP 0.4 (UI mínima), Fase 3 (explorer/editor), Fase 4 (build/problemas), criação básica de projeto e as Fases 5/5.1/5.2 de LSP (diagnósticos, navegação, completion, find usages e rename) implementados:

- `kinein-protocol`: tipos JSON-RPC compartilhados, incluindo `ToolStatus`/`ToolInfo`, `ProjectKind`/`WorkspaceInfo`, criação de pasta/projeto, criação/salvamento/renomeação/remoção de arquivos e diretórios, build estruturado, diagnósticos LSP, go to definition, hover, completion, find usages, rename, semantic tokens, busca no workspace, busca de arquivos com `fd`/`fdfind`, build, testes e análise de qualidade estruturados, execução de processos e sessão de terminal, job system assíncrono/cancelável (`job.list`/`job.cancel`) e scan de ambiente (protocolo `0.20.0`).
- `kinein-config`: modelo de configuração strict-by-default.
- `kinein-core`: loop stdin/stdout JSON-RPC com `core.ping`, `core.shutdown`, `command.list`, `tools.detect`, `tools.status`, `environment.scan`, `workspace.*`, `fs.*`, `fs.findFiles`, `fs.search`, `fs.rename`, `fs.delete`, `build.run`, `test.run`, `quality.run` (jobs assíncronos/canceláveis), `job.list`, `job.cancel`, `run.start`/`run.stdin`/`run.stop`, `terminal.open`/`terminal.input`/`terminal.close`, `lsp.didChange`, `lsp.semanticTokens`, `lsp.definition`, `lsp.hover`, `lsp.completion`, `lsp.references` e `lsp.rename`. Lista completa e contrato em `docs/03-ipc-protocol.md`.
- `kinein-cli`: helper mínimo para emitir requests JSON-RPC (`ping`, `list-commands`, `shutdown`, `build`, `tools detect|status`, `workspace open|browse|mkdir|new|status|close`).

A detecção de ferramentas cobre `cargo`, `rustc`, `cmake`, `ninja`, `git`, `clangd`, `ripgrep` e `fd`/`fdfind`: busca no `PATH`, probe de versão via `--version` e sugestão de instalação para CachyOS/Arch quando a ferramenta falta. O core nunca instala nada sozinho.

`workspace.open` identifica o tipo de projeto (Rust/Cargo, CMake, Maven, Gradle, Python ou desconhecido) por marcadores na raiz e persiste `.kinein/workspace.json` (schema em `schemas/workspace.schema.json`).

A UI Qt/QML (`ui/`) implementa janela escura com a paleta do design system, seletor proprio de workspace via `workspace.browse`, criação de pasta/projeto (`empty`, `cppCmake`, `rustCargo`), explorer navegável com criação, renomeação e exclusão de arquivos/pastas (menu de contexto) no Project panel, Search Everywhere inicial para comandos e arquivos (`Ctrl+Shift+N`/`Ctrl+Shift+A`, usando `command.list` e `fd`/`fdfind` via core), abas de editor com indicador de modificação, syntax highlighting com cores semânticas via LSP (variáveis, funções, tipos, parâmetros), completion automático enquanto digita, salvar com Ctrl+S, build com Ctrl+F9, testes com Ctrl+Shift+F9 (aba Testes, verde/vermelho por caso), análise de qualidade (cargo clippy) com Ctrl+Shift+L (lints na aba Problemas), go to definition com Ctrl+B, hover/quick documentation com Ctrl+Q, completion com Ctrl+Space, find usages com Alt+F7, rename com Shift+F6, busca no workspace com Ctrl+Shift+F (aba Busca), botão ▶ Iniciar/■ Parar com Shift+F10/Ctrl+F2 e aba Executar (saída ao vivo com stdin), aba Terminal com o shell real do usuário (`$SHELL` num PTY via `script`, Alt+F12), painel Build/Problemas, painel Ferramentas e KV Context para iniciar Claude ou Codex já instalados em um PTY dedicado. O acesso a arquivos passa inteiro pelo core (`fs.list`/`fs.read`/`fs.createFile`/`fs.createDirectory`/`fs.write`/`fs.rename`/`fs.delete`/`fs.findFiles`, confinados à raiz do workspace). A navegação e criação de pastas/projetos também passam pelo core. Diagnósticos de build e LSP aparecem na aba Problemas. A UI sobe o `kinein-core` como processo filho — nada precisa ser iniciado manualmente.

Exemplos:

```bash
cargo run -p kinein-cli -- tools detect | cargo run -p kinein-core
cargo run -p kinein-cli -- workspace browse ~ | cargo run -p kinein-core
cargo run -p kinein-cli -- workspace new /tmp demo cppCmake | cargo run -p kinein-core
cargo run -p kinein-cli -- workspace open ~/dev/projeto | cargo run -p kinein-core
cargo run -p kinein-cli -- build | cargo run -p kinein-core
```

Para recompilar e verificar sem reler a documentação longa, use a sequência em
[`docs/COMANDOS_BUILD_VERIFICACAO.md`](docs/COMANDOS_BUILD_VERIFICACAO.md).

## Verificação rigorosa

```bash
rustup run stable cargo kw-fmt
rustup run stable cargo kw-check
rustup run stable cargo kw-clippy
rustup run stable cargo kw-test
```

Para C++/Qt/QML, novos alvos devem usar `cmake/KineinStrictOptions.cmake` e os presets em `CMakePresets.json`.
