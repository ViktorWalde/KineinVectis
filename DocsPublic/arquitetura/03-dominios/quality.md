# `quality.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

### Qualidade / lint (`quality.run` — job assíncrono)

Implementado no protocolo `0.18.0`; migrado para **job assíncrono/cancelável**
como o `build.run`. Requer workspace aberto. Responde na hora com `{ "jobId" }`;
para Rust/Cargo roda `cargo clippy --all-targets --message-format=json` em
background, cujo JSON é idêntico ao do `cargo build`, então os lints viram
diagnósticos estruturados sem parser novo. Emite, com `jobId`,
`event.quality.started/output/diagnostic/finished` (mesmos formatos dos
`event.build.*`) + `event.job.*`; a saida bruta tambem faz fan-out para
`event.job.output`. Diagnosticos usam o mesmo payload de build, mas com
`source: "quality"` e `category: "lint"`. A UI adiciona os diagnósticos à aba Problemas
com origem `quality`. `job.cancel` mata o clippy. Validação síncrona: tudo que
não é Rust/Cargo **nem Python** retorna `INVALID_REQUEST` (CMake via clang-tidy
é o próximo passo). Falhas (`cargo` ausente etc.) chegam por
`event.quality.finished`. O parâmetro opcional `buildSystem` de `0.55.0` é
tipado pelo mesmo enum; as capacidades executáveis por `quality.run` são
`cargo`, `python` e, desde `0.119.0`, `cmake`/`make`.

**C/C++ (`0.119.0`, D6 do `roadmaps/41`, 2026-09-17).** Num `CMake` ou num
`Makefile` puro o `quality.run` roda o **clang-tidy do projeto inteiro pela
CDB** (`build/tidy.rs`): `run-clang-tidy -p <pasta da CDB> -quiet` quando o
script do LLVM está detectado (paraleliza e lê a `compile_commands.json`),
senão `clang-tidy -p <pasta> <arquivos>` com os arquivos vindos da própria
CDB (`file` de cada entrada, sem repetir, só os que existem — nunca um
glob). O `.clang-tidy` do projeto é lido pela ferramenta; a IDE não escolhe
checks. Cada `arquivo:linha:coluna: warning: mensagem [check]` vira
`event.quality.diagnostic` pelo parser gcc-like, com o nome do check na
mensagem. Sem CDB (`cdb::status`) o job falha dizendo "configure o CMake ou
compile o Makefile com o bear"; sem `clang-tidy`, nomeia a ferramenta. O
mesmo `.clang-tidy` vale no editor: o clangd sobe com `--clang-tidy`
(`toolchain/arguments.rs`), e os avisos entram nos diagnósticos do arquivo
aberto. O botão de análise da barra aparece também em C/C++ e Python.

**Python (`0.99.0`, 2026-09-13 — fatia 2 da cadeia do `roadmaps/41` bloco
B).** Workspace Python (`buildSystem: "python"`, ou o detectado): roda o
**ruff DETECTADO** (`~/.local/bin` do pipx/uv entra pelo detector) com
`ruff check --output-format concise --no-fix [--select …] .`, cwd no root —
os caminhos dos diagnósticos são relativos ao root e a configuração do
projeto (`ruff.toml`, `.ruff.toml`, `pyproject [tool.ruff*]`) é a que o ruff
acha a partir dali. O `--select` vem do **perfil de rigor** (`settings`)
apenas quando o projeto NÃO declara regras — `strict` = `E,F,W,I,UP,B,N`,
`balanced` = o default do ruff (sem flag), `relaxed` = `E9,F63,F7,F82` (só o
que quebra); projeto que declara vence sempre. Cada linha
`arquivo:linha:coluna: CODIGO [*] mensagem` vira `event.quality.diagnostic`
com `file`/`line`/`column`; severidade `error` para `E9xx` e `SyntaxError`,
`warning` para o resto; o `[*]` do ruff vira o sufixo `(corrigivel: ruff
check --fix)` na mensagem. Resumo (`Found N errors.`), `All checks passed!` e
avisos de configuração não viram diagnóstico. **Sem ruff detectado** o
`event.quality.finished` traz `success: false` e `error` que NOMEIA a
ferramenta e o passo oficial (`pipx install ruff`, o mesmo do painel de
instalação) — não o "tipo de projeto não suportado" de antes. `success` é
`false` quando o ruff achou problemas (exit ≠ 0). Exercitado no gate com o
ruff real (`F401` do `tools/gera.py` do projeto de exercitação).
