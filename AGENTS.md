# AGENTS.md — Instruções para GPT/Claude no terminal

Este arquivo orienta agentes de IA trabalhando no repositório **Kinein Vectis**.

## Identidade do projeto

Kinein Vectis é uma IDE open source, Linux-first, rígida por padrão, visualmente plug and play e orientada a performance.

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

1. Ler `ContextoIA.md` (estado real e decisões vigentes).
2. Ler `GUIAIA.md` para localizar o domínio, as conexões e os documentos
   específicos da tarefa. Ele é um mapa, não substitui as fontes seguintes.
3. Ler `docs/ARCHITECTURE.md` (camadas, convenções de módulo e regra de split — obrigatório).
4. Ler `docs/02-repository-structure.md` e `docs/03-ipc-protocol.md` (estrutura e contrato IPC atual).
5. Ler `docs/06-strict-mode.md` (rigor Rust/C++).
6. Consultar `docs/specs/` para a visão-alvo do que está sendo construído (entrada: `SPEC_INDEX`).
7. Verificar se a tarefa pertence ao core, UI, tooling, docs ou protocolo — e ao domínio certo dentro do core.
8. Se a tarefa criar ou alterar uma funcionalidade de IDE, seguir a política
   obrigatória de referência profissional de
   `KINEIN_VECTIS_OPEN_PLUGIN_ADAPTATION_ROADMAP.md`: estudar a implementação
   atual e oficial relevante em Code OSS, IntelliJ IDEA Community, Zed, Lapce
   e/ou Apache NetBeans; registrar revisão, invariantes e adaptação; escrever a
   solução nativa da Kinein sem copiar função, classe ou módulo e sem fazer
   tradução mecânica entre linguagens/frameworks.

Ao propor implementação:

- explicar arquivos que serão criados/alterados;
- manter escopo pequeno;
- não misturar muitas camadas;
- escrever testes quando aplicável;
- atualizar docs se mudar contrato ou arquitetura;
- registrar no documento do domínio quais referências profissionais atuais
  foram consultadas, o que foi aprendido e como isso foi adaptado às camadas
  Qt/QML → CoreClient → protocolo → Rust Core; a referência não autoriza
  dependência, port ou cópia de código;
- atualizar `GUIAIA.md` se criar/renomear/remover módulo, domínio, router,
  controller, gate ou direção de dependência.

## Política de leitura de documentação

Use a ordem de precedência definida em `docs/README.md`:

1. `ContextoIA.md` — estado real e decisões vigentes do repositório.
2. `docs/specs/` — especificação canônica da Kinein Vectis (visão-alvo).
3. `docs/00–16` (numerados) — contrato e estado do que já está implementado.

A **UI/UX segue `docs/specs/`**; a fonte de verdade do que já existe é
`ContextoIA.md` + `docs/specs/` + `docs/ARCHITECTURE.md` + código. Não existe
mais pasta de arquivo morto (`docs/archive/` foi removida em 2026-07-05 — ver
`docs/README.md`); não recriar uma só para guardar material descontinuado.
`GUIAIA.md` apenas encurta a navegação entre essas fontes e o código.

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

- Produto: `Kinein Vectis`
- Repositório/pasta: `kinein-vectis`
- Core daemon/package: `kinein-core`
- Crate importável: `kinein_core`
- CLI futura: `kinein-cli`
- Protocolo: `kinein-protocol`
- Configuração: `kinein-config`

## Não fazer

- Não transformar o core Rust em código dependente de Qt.
- Não colocar lógica de negócio na UI.
- Não chamar ferramentas externas diretamente da UI.
- Não criar formato de configuração sem schema/documentação.
- Não adicionar telemetria.
- Não enviar código do usuário para IA externa sem confirmação explícita.
- Não relaxar strict mode sem registrar motivo.
- Não copiar e colar implementação pronta de outra IDE, nem escolher revisão
  antiga/abandonada apenas porque contém uma função conveniente.
