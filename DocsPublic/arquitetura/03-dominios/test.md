# `test.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

### Testes (`test.run` / `test.discover` — jobs assíncronos)

**gtest e Catch2 dentro dos binários do ctest (`0.119.0`, D7 do
`roadmaps/41`, 2026-09-17).** Um `add_test` do `CMake` é UM binário com N
casos; o `ctest -N` lista o binário. O `test.discover` de um `CMake` pergunta
ao próprio binário: `ctest --test-dir <build> --show-only=json-v1` (ctest
4.2.3, medido: `tests[].name`, `tests[].command[]`, `WORKING_DIRECTORY`),
depois `<exe> --gtest_list_tests` (gtest: `Suite.` na coluna 0, `  Caso`
indentado; parametrizados `Suite/Inst.Caso/0`) ou `<exe> --list-tests
--verbosity quiet` (Catch2 v3: um nome por linha). Cada caso entra como
`TestCaseInfo { id: "<teste do ctest>::<caso>", name: "<caso>" }` depois das
linhas do ctest; um binário que não responde a nenhuma listagem continua
sendo só a linha do ctest. `test.run { testId: "unit_tests::Math.Adds" }`
roda o binário direto — `--gtest_filter=Math.Adds --gtest_color=no` (cada
`[       OK ]`/`[  FAILED  ]` vira `event.test.case`) ou `"<caso>" -r
compact` no Catch2 (o desfecho do caso é o exit code). Um `testId` sem `::`
é o ctest de sempre (`-R ^nome$`).

Implementado no protocolo `0.17.0`; migrado para **job assíncrono/cancelável**.
Requer workspace aberto. Responde na hora com `{ "jobId" }`; roda o runner do
tipo de projeto em background (`cargo test` para Rust/Cargo; `ctest --test-dir
.kinein/build --output-on-failure` para CMake; **`python -m pytest -v` para
Python** desde `0.100.0`, 2026-09-13) e transmite cada caso conforme sai da
saída do runner. Aceita `{ "filter"? }` (posicional do cargo; `-R` do ctest;
`-k` do pytest) e, desde `0.55.0`, `{ "buildSystem"? }` com a mesma validação
de capacidade do build. `job.cancel` mata o runner.

```text
event.test.started   { "jobId", "command": "cargo test" }
event.test.output    { "jobId", "stream": "stdout|stderr", "line": "..." }
event.test.case      { "jobId", "name": "modulo::caso", "status": "passed|failed|ignored" }
event.test.finished  { "jobId", "success", "exitCode", "passed", "failed", "ignored" }
event.test.finished  { "jobId", "success": false, "error": "…" }   (o runner nem correu)
```

Além destes, `event.job.created`/`event.job.output`/`event.job.finished`. Cada
`event.test.output` tambem gera `event.job.output { "jobId", "line" }` para o
historico generico do job. Os casos são extraídos das linhas
`test <nome> ... ok|FAILED|ignored` (libtest), `... Test #N: <nome> ...
Passed|***Failed` (ctest) e `arquivo::caso PASSED|FAILED|ERROR|SKIPPED|XFAIL|
XPASS [ nn%]` (pytest `-v`: o nome vem antes do estado e tem `::`; `PASSED`/
`XPASS` = passou, `FAILED`/`ERROR` = falhou, `SKIPPED`/`XFAIL` = ignorado; o
resumo curto — `FAILED arquivo::caso - assert …`, estado na frente — não é
caso, senão cada falha contaria duas vezes); a linha de resumo do libtest é
ignorada. Tipos sem integração retornam `INVALID_REQUEST` (síncrono, antes do
job); o resultado vem em `event.test.finished`, não na resposta.

**A árvore antes do run (`test.discover`, `0.106.0`, 2026-09-13).**
`test.discover { buildSystem? }` → `{ jobId }`; o job lista **sem rodar** e
termina em `event.test.discovered { jobId, runner, command, tests:
[TestCaseInfo { id, name, file? }], success, error? }`:

```text
pytest   python -m pytest --collect-only -q    tests/test_a.py::test_x[a b]  (medido, 9.1.1:
         um node id por linha; o resumo "N tests collected" e as linhas vazias
         não contam; só o stdout — o pytest escreve avisos no stderr; exit 5
         = "no tests ran" = lista VAZIA com sucesso, um projeto novo não tem testes)
cargo    cargo test -- --list                  tests::alpha: test  (`: benchmark`
         e o resumo não contam; COMPILA os testes — por isso é job)
ctest    ctest --test-dir .kinein/build -N     "  Test #N: Nome"  (medido, ctest 4.x)
```

O `id` é o que o **mesmo** runner aceita para rodar um só, e é o `name` que
`event.test.case` reporta quando ele roda — a UI pinta a linha da árvore
sem tabela de tradução. `test.run { testId }` (vence `filter`; vazio e
espaços não contam): pytest recebe o node id **posicional** (`pytest
tests/a.py::x` — `-k` casaria por substring), cargo `cargo test <nome> --
--exact` (o `--exact` é do libtest, depois do `--`), ctest `-R ^nome$` com
os metacaracteres escapados (`Broken.Case` não pode casar `BrokenXCase`;
provado contra o ctest real com `Core` e `CoreParsing` lado a lado). Sem
runner para o tipo, `INVALID_REQUEST`; sem interpretador ou sem o módulo
pytest no ambiente, `event.test.discovered { success: false, error }` com o
passo. A UI (`TestsPanel`, que agora recebe o `JobsController` inteiro em
vez de quatro escalares): "Listar testes", a árvore com o ponto de status e
um ▶ por linha; um run novo apaga os status e mantém a árvore.

**Python (`0.100.0`).** O pytest roda com o **interpretador do projeto** (ou
`uv run python` — a mesma regra do `run.script`), cwd no root: é lá que os
pacotes do projeto estão, e um pytest de fora do ambiente testaria outra
coisa. Sem interpretador, `event.test.finished { success: false, error }`
orienta a criar o ambiente; com interpretador mas **sem o módulo pytest
naquele ambiente** (o `No module named pytest` do próprio Python, medido no
3.14 desta máquina pela exercitação do gate), o `error` diz como instalar
NELE: `uv add --dev pytest` (projeto do uv) ou `.venv/bin/python -m pip
install pytest`. O `error` é a forma geral de "o runner nem correu" (o
`emit_run_error` que build e quality já usavam); a UI (`0.100.0`) mostra-o
como resumo do painel Testes em vez de "passou: 0". **O painel Testes passou a
mostrar a saída bruta** (`event.test.started` + `event.test.output`, a metade
de baixo quando há linhas): é onde o pytest explica a falha — o sinal existia
no C++ desde o `test.run` e ninguém ouvia (`roadmaps/41` A6).

> **Nota:** com build/quality/test todos como jobs assíncronos, não existe mais
> caminho de streaming síncrono no core — toda operação longa retorna `jobId` e
> emite eventos pelo canal assíncrono.
