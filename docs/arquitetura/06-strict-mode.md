# 06 — Strict Mode

## Filosofia

Kinein Vectis é rígido por padrão.

Projetos novos devem nascer com qualidade profissional, e o usuário pode relaxar regras depois se quiser.

## Regra geral

```text
Strict primeiro.
Relaxado apenas por decisão explícita.
```

## Rust — core do Kinein Vectis

Regras:

- `unsafe_code = "forbid"`.
- warnings como erro.
- clippy agressivo.
- sem `unwrap`, `expect`, `panic` fora de testes.
- `cargo fmt --check`.
- `cargo test`.
- `cargo clippy --all-targets --all-features -- -D warnings`.
- auditoria de dependências com `cargo-deny`/`cargo-audit` futuramente.

Comandos:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo check --workspace --all-targets --all-features
```

## C++ Strict

Padrão para projetos C++ gerados:

- C++23;
- CMakePresets;
- Ninja;
- `CMAKE_EXPORT_COMPILE_COMMANDS=ON`;
- clangd;
- clang-format;
- clang-tidy;
- warnings as errors;
- sanitizers em Debug;
- LTO em Release Hardened;
- CTest;
- Catch2 ou GoogleTest.

Flags sugeridas para Clang/GCC:

```text
-Wall
-Wextra
-Wpedantic
-Werror
-Wconversion
-Wsign-conversion
-Wshadow
-Wnon-virtual-dtor
-Wold-style-cast
-Wcast-align
-Wunused
-Woverloaded-virtual
-Wnull-dereference
-Wdouble-promotion
-Wformat=2
-Wimplicit-fallthrough
-Wundef
```

Sanitizers Debug:

```text
-fsanitize=address,undefined
-fno-omit-frame-pointer
```

No frontend da própria Kinein, warnings-as-errors permanecem obrigatórios para
todo `ui/src/*`. Arquivos gerados pelo Qt 6 (MOC, registro QML e cache AOT)
podem disparar novos warnings do compilador sem haver fonte autoral corrigível;
as únicas supressões aceitas ficam por `COMPILE_OPTIONS` nos caminhos gerados
específicos em `ui/CMakeLists.txt`. Não adicionar `-Wno-error` global para
resolver warning de MOC/QML AOT.

## QML Strict (UI do Kinein Vectis)

Desde 2026-07-08 o QML da UI tem gate automatizado, no mesmo espírito de
Rust/C++ (zerado primeiro, ligado depois):

- `scripts/verificar-qml.sh` roda `qmllint -W 0` (zero warnings) com o
  contexto real do módulo (`--bare`, import paths, qmldir e resources do
  response file gerado pelo `qt_add_qml_module` no build debug);
- quando mais de um build existe, o response file do preset
  `linux-clang-debug-strict` tem precedência sobre `dev-local`, evitando lint
  contra cópias QML/qmltypes antigas;
- faz parte do `scripts/verificar.sh` nos modos completo e rápido;
- padrões do repositório: `pragma ComponentBehavior: Bound` em arquivos com
  delegates, `required property` para roles de model, acesso qualificado
  (id explícito) em vez de resolução implícita de escopo ou `parent.parent`;
- `import KineinVectis` explícito quando o arquivo usa tipos do módulo
  (ex.: `Theme`), mesmo que o import implícito resolvesse.

Degraus futuros (ordem em `docs/diario/18-daily-driver-plan.md`): `qmlformat
--check` após reformatar o tree numa fatia dedicada e testes Qt Quick Test
para controllers não visuais.

## Java Strict

Padrão para projetos Java:

- Java 25 LTS como alvo desejado do usuário;
- Maven ou Gradle;
- JUnit;
- Checkstyle;
- SpotBugs;
- PMD;
- Error Prone opcional;
- logs estruturados;
- profile dev/prod;
- testes por padrão.

## Python Strict

Padrão para projetos Python:

- uv;
- `.venv` obrigatório;
- Ruff lint;
- Ruff format;
- Pyright/basedpyright strict;
- pytest;
- mypy opcional;
- `.env.example`;
- estrutura clara para FastAPI/Django/CLI.

## Backend Strict

Padrão:

- healthcheck;
- logs estruturados;
- `.env.example`;
- Docker/Podman Compose opcional;
- OpenAPI quando aplicável;
- testes de endpoint;
- configuração dev/prod separada.

## Embedded Strict

Padrão:

- target explícito;
- toolchain explícita;
- sysroot explícito quando aplicável;
- CMake toolchain file;
- debug profile;
- flash profile;
- serial monitor;
- QEMU quando aplicável;
- logs de build e deploy.

## Níveis futuros

```text
Strict       padrão
Balanced     menos agressivo
Relaxed      para legado
```

Mesmo que o autor use apenas Strict, os níveis devem existir no modelo para futura distribuição.
