# AGENTS.md — Instruções para GPT/Claude no terminal

Este arquivo orienta agentes de IA trabalhando no repositório **Kernwerk Studio**.

## Identidade do projeto

Kernwerk Studio é uma IDE open source, Linux-first, rígida por padrão, visualmente plug and play e orientada a performance.

Arquitetura principal:

```text
Qt/QML Frontend  ← IPC/JSON-RPC local →  Rust Core
```

O projeto deve evitar reimplementar ferramentas já existentes. Sempre que possível, integrar ferramentas consolidadas:

- C++: clangd, CMake, Ninja, clang-format, clang-tidy, GDB/LLDB.
- Java: JDK 25 LTS, jdtls, Maven, Gradle, JUnit, Checkstyle, SpotBugs, PMD.
- Python: uv, venv, Pyright/basedpyright, Ruff, pytest, mypy opcional.
- Embedded: CMake toolchains, QEMU, OpenOCD, pyOCD, GDB remote, serial monitor, Yocto/Buildroot SDKs.
- Backend: Docker/Podman Compose, HTTP client, OpenAPI, PostgreSQL, Redis, logs.
- IA: GPT CLI, Claude CLI, Ollama, OpenAI/Anthropic/OpenRouter via providers configuráveis.

## Regra principal

Não criar soluções improvisadas quando existir ferramenta aberta, madura e gratuita que resolva o problema.

A IDE deve orquestrar ferramentas, não substituir compiladores, servidores LSP ou debugadores.

## Rigor obrigatório

Todo código Rust deve seguir o máximo rigor possível:

- `unsafe` proibido por padrão.
- Warnings devem quebrar build.
- `cargo fmt --check` obrigatório.
- `cargo clippy --all-targets --all-features -- -D warnings` obrigatório.
- Testes devem ser criados para módulos de core.
- Erros devem usar tipos explícitos e contexto.
- Evitar `unwrap`, `expect` e panics fora de testes.
- Preferir APIs pequenas, testáveis e desacopladas.
- Evitar acoplamento entre UI e core.
- Não bloquear UI com trabalho pesado.
- Não adicionar dependências sem justificativa.

## Como agir ao implementar

Antes de criar código:

1. Ler `docs/00-product-vision.md`.
2. Ler `docs/01-architecture.md`.
3. Ler `docs/02-repository-structure.md`.
4. Ler `docs/03-ipc-protocol.md`.
5. Ler `docs/06-strict-mode.md`.
6. Verificar se a tarefa pertence ao core, UI, tooling, docs ou protocolo.

Ao propor implementação:

- explicar arquivos que serão criados/alterados;
- manter escopo pequeno;
- não misturar muitas camadas;
- escrever testes quando aplicável;
- atualizar docs se mudar contrato ou arquitetura.

## Padrão de commits sugerido

Usar Conventional Commits:

```text
feat: adiciona protocolo inicial de IPC
fix: corrige serialização de eventos do core
docs: documenta strict mode
refactor: separa workspace service do command registry
test: cobre validação de comandos
chore: atualiza configuração de lint
```

## Nomes

- Produto: `Kernwerk Studio`
- Repositório/pasta: `kernwerk-studio`
- Core daemon/package: `kernwerk-core`
- Crate importável: `kernwerk_core`
- CLI futura: `kernwerk-cli`
- Protocolo: `kernwerk-protocol`
- Configuração: `kernwerk-config`

## Não fazer

- Não transformar o core Rust em código dependente de Qt.
- Não colocar lógica de negócio na UI.
- Não chamar ferramentas externas diretamente da UI.
- Não criar formato de configuração sem schema/documentação.
- Não adicionar telemetria.
- Não enviar código do usuário para IA externa sem confirmação explícita.
- Não relaxar strict mode sem registrar motivo.
