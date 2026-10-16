# `python.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

## `python.*` — o ambiente do projeto Python

Domínio novo no protocolo `0.98.0` (fatia 1 da cadeia Python, 2026-09-12).
O interpretador é o `compile_commands.json` do Python: quem o resolve é
`python::env` (a precedência do `roadmaps/29` §4.1), e é o MESMO que o
`index.context` mostra; as fatias seguintes (basedpyright, run, pytest,
debugpy) leem daqui.

```text
python.status {}                       -> PythonStatus       (exige workspace)
python.createEnvironment { tool? }     -> { jobId }          (job; tool = uv | venv;
                                                              omitido = uv se houver,
                                                              senao venv)
python.stubs { port?, board? }         -> { jobId, package, command, target }
                                                             (job; 0.116.0)
event.python.finished { jobId, success, tool, command, path }
event.python.stubs    { jobId, success, package, command, target }

PythonStatus   interpreter? (PythonEnv: interpreter, version?, origin, warning?),
               nativeModule? (PythonNativeModule: kind, tool, evidence[], buildHint;
               0.102.0),
               hasEnvironment (origin != sistema), environmentTool? (uv|venv:
               o que a IDE usaria), uv? (caminho), projectFiles[] (pyproject.toml,
               requirements.txt, setup.py, uv.lock, poetry.lock, Pipfile),
               hint? (o que falta, com o remedio),
               stubsPath? (<root>/typings quando tem um .pyi; 0.116.0),
               stubsSuggested? (o pacote que o chip do kit/identidade sugere)
```

**`python.stubs` são os STUBS DA PLACA (C4 do `roadmaps/41` bloco C,
`0.116.0`):** o `import machine` completa e o basedpyright para de dizer
que o módulo não existe. Fonte (micropython-stubs.readthedocs.io, "Install
the micropython-stubs", 2026-09-17): `pip install -U
micropython-<port>[-<board>]-stubs --no-user --target ./typings`; aqui o
instalador é o `uv` detectado (`uv pip install -U --link-mode=copy --target
<root>/typings <pacote>`; medido nesta máquina: 2 pacotes em 7 ms) ou, sem
uv, o `pip` do interpretador do projeto. `port`/`board` ausentes = o modelo
do projeto decide pelo chip do kit/identidade, pela lista publicada de
pacotes (`esp32c3` → `micropython-esp32-esp32_generic_c3-stubs`; família
`rp2040` sem chip → `micropython-rp2-stubs`, porque Pico e Pico W têm stubs
diferentes); sem evidência, `INVALID_REQUEST` dizendo o que fixar. `success`
exige exit 0 E um `.pyi` em `typings/`. Ao terminar com sucesso, o
basedpyright é reconfigurado com `basedpyright.analysis.stubPath =
<root>/typings` e `diagnosticSeverityOverrides.reportMissingModuleSource =
none` (a chave do manual do basedpyright e o `pyproject` de exemplo dos
micropython-stubs) e reinicia se está vivo. Na UI: faixa INFORMATIVA de
saúde num projeto MicroPython com sugestão e sem `typings/`, botão
**Instalar stubs**.

**As regras, ditas.** `success` só é `true` quando `.venv/bin/python` existe
depois de a ferramenta sair com 0 — uma ferramenta que "termina bem" sem
criar o interpretador não vira ambiente anunciado. O comando é o da fonte
oficial, mostrado no job (`$ uv venv .venv`), nunca escondido. `uv` sem
`python3` no PATH basta (o uv baixa um Python; docs do uv). Sem uv e sem
python3 o pedido é recusado com o motivo — e o painel de instalação tem os
guias oficiais de `pipx`, `uv`, `ruff` e `basedpyright` (fonte e data em
cada um; família `any` quando a fonte é agnóstica de distro). Na UI: a faixa
de saúde do projeto ganha "o projeto usa o Python do SISTEMA: crie um
ambiente" com o botão **Criar .venv com uv** (ou `python3 -m venv`); um
clique, o job aparece, o status é perguntado de novo. Só em workspace cujo
`buildSystems` inclui `python` — um projeto Cargo/CMake com `pyproject.toml`
dentro conta.

Exercitação: o projeto de exercitação ganhou um `pyproject.toml`; o gate cria
o `.venv` com a ferramenta REAL desta máquina (`python3 -m venv`, 2026-09-12)
e vê o `python.status` passar de `sistema` para `.venv`.

**`nativeModule` (`0.102.0`, fatia 5).** A ponte entre as duas metades da
IDE: um projeto Python com extensão em C++/Rust é TAMBÉM um projeto
CMake/Cargo (o índice e o clangd/rust-analyzer já o leem), mas quem o instala
no `.venv` é o build do Python. Por evidência nos arquivos da raiz —
`pyproject.toml` (`[build-system] requires` com maturin, scikit-build-core,
setuptools-rust, pybind11 ou nanobind; `[tool.maturin]`), `Cargo.toml`
(dependência `pyo3`), `CMakeLists.txt` (`pybind11`/`nanobind`), `setup.py`
(`pybind11`/`nanobind`) — o status diz `kind` (`pybind11`, `nanobind`,
`PyO3`, ou `Rust` para maturin/setuptools-rust sem pyo3 explícito), `tool`
(`maturin`, `scikit-build-core`, `setuptools-rust`, `setuptools`), uma linha
de `evidence` por arquivo, e `buildHint` como a fonte oficial escreve:
`maturin develop` (no ambiente: `uv run maturin develop` ou
`.venv/bin/maturin develop`), `pip install -e .` para scikit-build-core e
setuptools. Projeto Python puro não tem o campo — nem com um `Cargo.toml` sem
pyo3 ou um `CMakeLists.txt` sem binding ao lado.
