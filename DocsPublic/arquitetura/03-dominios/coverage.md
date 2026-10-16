# `coverage.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

## `coverage.*` — a cobertura dos testes

Domínio novo no protocolo `0.119.0` (D8 do `roadmaps/41`, P5 do `40` §4.1,
2026-09-17). O LCOV é a língua comum: é o que `cargo llvm-cov` e o
`coverage.py` escrevem, e o que a calha do editor lê.

```text
coverage.run   {}          -> { jobId }   (job; event.coverage.finished no fim)
coverage.lines { file }    -> { file, known, covered: [linha], missed: [linha] }

event.coverage.finished { jobId, success, tool, path?, files: [{ file, linesFound,
                          linesHit }], error? }
```

**Por tipo de projeto:** Rust → `cargo llvm-cov --lcov --output-path
.kinein/coverage.lcov` (o `cargo` do kit; `cargo-llvm-cov` 0.9.1 desta
máquina: `--lcov` + `--output-path`; exige `rustup component add
llvm-tools-preview`, e sem o `cargo-llvm-cov` o job falha dizendo os dois
passos); Python → `<python> -m coverage run -m pytest -q` e `<python> -m
coverage lcov -o .kinein/coverage.lcov` pelo interpretador do projeto (sem o
módulo, o passo `uv add --dev coverage pytest`); C/C++ → `INVALID_REQUEST`
antes do job: exige compilar com `--coverage` (gcov/lcov), o que é decisão
do usuário no build — não entra sem ela. O arquivo fica em
`<root>/.kinein/coverage.lcov` e é relido: `files` é um resumo por arquivo
(`SF:`, os `DA:` somados por linha — blocos repetem linhas), e
`coverage.lines` dá as linhas de UM arquivo (caminho exato ou canônico
igual), `known: false` quando o relatório não o tem. Na UI: comando
**Cobertura dos testes** (paleta e menu Build) → job; o `CoverageController`
guarda o resumo e, a cada troca de aba, pede as linhas do arquivo ativo — a
calha pinta uma barra de 3 px ao lado da do diff: verde coberta, vermelha
instrumentada e nunca executada.
