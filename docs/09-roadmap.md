# 09 — Roadmap

## Fase 0 — Fundamento Rust

Objetivo: criar base robusta antes da UI.

- Rust workspace.
- Crates iniciais:
  - `kernwerk-core`
  - `kernwerk-protocol`
  - `kernwerk-config`
  - `kernwerk-cli`
- Strict mode do próprio projeto.
- Logs.
- Settings básicos.
- IPC simulado ou stdin/stdout.
- Comando `core.ping`.
- Testes iniciais.

## Fase 1 — Core + CLI

Objetivo: validar arquitetura sem interface gráfica.

- `kernwerk-core` roda como processo.
- `kernwerk-cli ping`.
- `kernwerk-cli tools detect`.
- `kernwerk-cli workspace open <path>`.
- JSON-RPC básico.
- Schemas iniciais.

## Fase 2 — UI Qt/QML mínima

Objetivo: janela conecta no core.

- Janela escura.
- Splash/local status.
- IPC client.
- Status: Core conectado.
- Painel de logs.
- Settings básico.

## Fase 3 — Workspace e Explorer

- Abrir pasta/projeto.
- Explorer visual.
- Abas.
- Abrir/salvar arquivo.
- Estado de layout.
- Arquivos recentes.

## Fase 4 — Build CMake

- Detectar CMake.
- Detectar CMakePresets.
- Configure.
- Build.
- Run.
- Painel Build.
- Erros estruturados.

## Fase 5 — Base LSP C/C++ e Rust

- Iniciar clangd.
- Enviar abertura de documento.
- Diagnostics.
- Hover.
- Go to definition.
- Completion simples.
- Quick fixes.
- Suporte equivalente para rust-analyzer quando disponível.
- Semantic tokens, references e rename básicos.

Critério de saída: a Fase 5 não precisa virar uma implementação completa de
IDE JetBrains. Ela deve fornecer uma base prática de LSP para C/C++ e Rust.
Code actions avançados, refatorações profundas e indexação persistente ficam
para polimento pós-V1 ou fases específicas futuras.

## Fase 6 — Git

- Status.
- Diff.
- Stage/unstage.
- Commit.
- Branch atual.
- Histórico básico.

## Fase 7 — Assistente como terminal dedicado

Objetivo: manter a integração de IA simples e útil no curto prazo, sem criar
um sistema próprio de providers antes da IDE amadurecer.

- Aba/painel visualmente separado para chat de IA.
- Terminal dedicado para Claude/Codex/GPT CLI ou ferramenta equivalente.
- Terminal separado do terminal geral de comandos do projeto.
- Persistência visual mínima da sessão quando possível.
- Política clara: nada é enviado para IA externa sem ação explícita do usuário.
- Sem tentar implementar provider próprio, contexto automático profundo ou
  revisão inteligente de diff nesta fase.

Integrações mais profundas de IA ficam para depois que Git, build, debug e
fluxos C/C++/Rust estiverem sólidos.

## Fase 8 — C/C++ profissional

Objetivo: tornar o Kernwerk Studio forte para desenvolvimento C/C++ moderno em
Linux antes de expandir para outras linguagens.

- CMakePresets e configuração/build por perfil.
- clangd com compile_commands confiável.
- clang-format no arquivo/projeto.
- clang-tidy/quality para CMake.
- CTest integrado ao painel de testes.
- Run/debug de executáveis CMake.
- GDB/LLDB via DAP ou integração equivalente madura.
- Toolchains CMake e perfis de compilador.
- Melhor UX para erros de compilação, warnings e navegação até diagnóstico.

## Fase 9 — Rust profissional

Objetivo: tornar o Kernwerk Studio excelente para desenvolver o próprio core
Rust e projetos Rust reais.

- rust-analyzer como LSP de primeira classe.
- cargo check/build/test/run por workspace/package.
- cargo clippy e rustfmt integrados como quality/format.
- Testes Rust com filtro e saída estruturada.
- Navegação por diagnósticos e lints.
- Suporte a workspaces com múltiplos crates.
- Comandos de verificação do próprio Kernwerk como tarefas de projeto.
- Preparação para dogfooding pesado do `kernwerk-core`.

## Fase 10 — Embedded

- Toolchain profiles.
- CMake toolchain files.
- QEMU.
- Serial monitor.
- GDB remote.
- Flash profiles.
- Yocto/Buildroot SDK support.

## Pós-V1 — Java e Python

Java e Python continuam no escopo de produto, mas não são prioridade de curto
prazo. Eles devem voltar ao roadmap depois que C/C++, Rust, Git, debug,
quality e o fluxo de dogfooding estiverem maduros.

- Java: JDK, Maven/Gradle, jdtls, JUnit, Spring Boot e Services.
- Python: uv/venv, Pyright, Ruff, pytest, FastAPI/Django e Services.

## Pós-V1 — Modos de compilador e loja de funções

Depois da V1.0, o Kernwerk Studio deve estudar e implementar uma camada visual
para ativar modos de compilador e funções de qualidade para C/C++ e Rust.

- Modos como Strict ISO, Strict, Balanced, Relaxed, Debug Sanitized, Release
  Hardened e Embedded Strict.
- Catálogo visual de flags, regras, linters, formatadores, sanitizers e
  quality gates.
- Janela de opções para ambiente do projeto, com nome humano da função,
  explicação do que ela faz, risco, fonte e prévia das mudanças.
- Ordem de confiança: ISO/Standard primeiro em C/C++; Rust oficial primeiro em
  Rust; depois diagnósticos oficiais, ferramentas maduras, presets Kernwerk,
  regras locais e opções experimentais.
- Tudo deve gerar configuração real, auditável e reversível.

Detalhes em `docs/16-compiler-modes-and-function-store.md`.

## Regra do roadmap

Não avançar para features avançadas antes de validar o contrato UI ↔ Core e o sistema de comandos.
