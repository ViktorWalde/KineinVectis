# 41 — O ecossistema aberto de embarcados e Python, em ordem linear

> **Classe: PLANO**, com uma seção de ESTADO (§2) medida em **2026-09-11**.
> Toda licença desta página foi lida no **arquivo de licença do repositório**
> (via `raw.githubusercontent.com` ou o endpoint `/license` da API, que
> devolve o arquivo — não o palpite SPDX). Toda afirmação sobre o VS Code vem
> da documentação oficial em `code.visualstudio.com/docs` ou do README da
> extensão, lidos na mesma data. Número sem data é sobre AGORA.
>
> **A decisão do autor que abre este documento (2026-09-11, noite):**
>
> 1. **Python e MicroPython entram como vertical NATIVA.** Isto **reverte** o
>    "Python fica para depois" de 2026-08-30 ([`29`](29-verticais-de-linguagem.md)
>    §6, ORDEM A) — por decisão explícita do autor, que é como este repositório
>    muda de regra. O que **não** muda: **Pylance continua PROIBIDO** (licença
>    proprietária, só em produtos Microsoft); o motor aberto é o
>    `basedpyright`/`pyright`, como o 29 §3.1 já tinha levantado.
> 2. **O ecossistema aberto de embarcados entra inteiro** — "nada deve ficar de
>    fora" —, organizado em ordem linear, por funcionalidade que **de fato
>    funciona na IDE**, não por aba que anuncia.
> 3. **O critério:** o que o VS Code oferece para Python, C/C++ e Rust é a
>    referência de *funcionalidade*; a *implementação* é a ferramenta aberta
>    que está por trás dela, adotada **pronta** (MODE-A: processo ou protocolo),
>    nunca reescrita. Padrão de mercado, solução pronta, nada do zero.

> **SUCEDIDO na parte de embarcados pelo [`42`](42-trilha-profunda-embarcados.md)
> (2026-09-12).** O autor pediu profundidade em vez de cortes verticais rasos:
> o 42 reorganiza os blocos A–F em oito pilares com critério de pronto por
> família e amplia o alvo para Linux embarcado. O inventário (§3) e o que não
> entra (§4) desta página continuam valendo como fonte; a ORDEM passa a ser a
> do 42 §4.

> **2026-09-12 (tarde):** o autor acrescentou o *critério de experiência* — o
> "efeito JetBrains" (zero-config, indexação visível, Alt+Enter proativo,
> project model antes do LSP, sysroot visual, remote deploy & debug, SVD com
> escrita, sondas visuais) — e a **trilha Python completa** (bare metal → edge
> → backend → banco). Os dois estão mapeados no [`42`](42-trilha-profunda-embarcados.md)
> §8 e §9, com o que já existe medido e o que falta por pilar; este inventário
> continua sendo a lista de ferramentas com licença.

> **2026-09-12 (noite):** as **toolchains por alvo** ganharam documento
> próprio — [`integracoes/39`](../integracoes/39-toolchains-por-alvo.md): o
> catálogo credível (Arm, xPack, Espressif unificada, ATfE, Bootlin, Yocto/
> Buildroot, Zephyr SDK, Rust por `rustup target add`), o problema do sysroot,
> onde a IDE procura além do `PATH`, e a regra do provedor de instalação. Este
> inventário continua sendo a lista das ferramentas de FUNCIONALIDADE.

## 0. Como ler isto

```text
§1  o metodo: de onde vem cada linha, e o que "adotar" significa aqui
§2  o que a IDE JA' TEM, medido — para nao reimplementar o que existe
§3  o inventario, por area: funcionalidade (VS Code) -> ferramenta aberta ->
    licenca -> forma na Kinein -> estado hoje
§4  o que NAO entra, com o motivo (proprietario, sem licenca, decidido contra)
§5  A ORDEM LINEAR — o entregavel deste documento
§6  fontes
```

## 1. O método

**De onde vem a lista de funcionalidades.** Da documentação oficial do VS Code
para Python (tutorial, *Linting*, *Formatting*, *Environments*, *Testing*,
*Debugging*, *Jupyter*), do README do C/C++ (`cpptools`) e do CMake Tools, do
rust-analyzer, e das quatro extensões que definem "embarcado no VS Code" hoje:
**Cortex-Debug** (e as companheiras `mcu-debug`), **probe-rs**, **PlatformIO**,
**ESP-IDF** — mais a do **Raspberry Pi Pico**. Elas são a referência de *o que
o usuário espera*; nenhuma delas roda dentro da Kinein.

**O que "adotar" significa** (`integracoes/README`, modos A–D):

```text
MODE-A  a IDE EXECUTA a ferramenta (processo, DAP, LSP, CLI com saida
        estruturada) — e' a forma de tudo que esta' na §5. GPL entra assim.
MODE-B  a extensao e' REFERENCIA de UX; o codigo e' nosso. E' o caso das
        extensoes MPL-2.0 (MicroPico, pico-vscode) e das proprietarias que
        tem funcionalidade util e codigo fechado (Serial Monitor da Microsoft).
crate   so' permissiva (deny.toml). MPL-2.0 NAO entra — decidido em
        2026-09-11 (integracoes/38 §6).
```

**A regra que o 29 §1 ensinou e vale aqui:** *não anunciar* a vertical antes de
a cadeia inteira funcionar. Realce sem LSP e sem `pytest` é realce, não Python.

## 2. O que a IDE já tem — medido em 2026-09-11

```text
LSP           clangd (C/C++, com --query-driver para cross), rust-analyzer.
              lsp/server.rs e' uma tabela de 2 entradas — o 3o e' um ServerSpec
DAP           lldb-dap, probe-rs dap-server (chip no launch), gdb -i dap com
              attach a servidor (QEMU/OpenOCD) que a IDE sobe e mata
build         cmake (configure/build/targets/presets/kits com sysroot, triple,
              chip, remoteTarget, debugServer), cargo, build.size (regioes do
              linker script), 20 acoes de configuracao do CMake (sanitizers,
              hex/bin, alvo embarcado do cargo, FetchContent pinado...)
teste         cargo test e ctest (test.rs) — a SAIDA ainda nao chega a tela
              (event.test.output sem ouvinte, roadmaps/40 §8)
qualidade     cargo clippy (quality.run). clang-tidy: NAO; cppcheck: NAO
formato       clang-format, rustfmt
sintaxe       Tree-sitter C, C++, Rust. Python: NAO. Desde 2026-09-12 as
              MESMAS gramaticas indexam o projeto INTEIRO (dominio `index`)
embarcado     probe.list, serial.list (sem abrir a porta), serial.monitor
              (processo), project.model (9 frameworks, SDKs, artefatos, alvo),
              build.size (regioes do .ld E a particao app do ESP-IDF),
              fixture bare-metal + QEMU no gate, catalogo com arm-none-eabi
banco/obs     Postgres, TimescaleDB, SQLite, MongoDB, Grafana pela HTTP API
setup         "instalar ferramentas": catalogo com grafana, postgresql,
              timescaledb — NENHUMA ferramenta de embarcado ou Python
terminal      PTY real (portable-pty + alacritty_terminal), multi-aba;
              `open_command` (processo arbitrario num PTY) existe SEM chamador
docker        DECIDIDO nativo (40 §5); IMPLEMENTADO em 2026-09-12 (40 §7.14),
              docker|podman pela mesma CLI. Dev containers: NAO
remoto/SSH    DECIDIDO (40 §4), nao arquitetado
```

## 3. O inventário

Legenda da coluna **forma**: `processo` (MODE-A, filho da IDE), `LSP`/`DAP`
(MODE-A por protocolo), `crate` (linkado; só permissiva), `ref` (MODE-B).
Coluna **hoje**: ✓ existe · ◐ parcial · ✗ não existe.

### 3.1 C/C++ (desktop — a base do embarcado)

| funcionalidade no VS Code | quem faz lá (licença) | ferramenta aberta | licença (arquivo) | forma | hoje |
| --- | --- | --- | --- | --- | --- |
| IntelliSense, navegação, rename, inlay hints | cpptools (MIT **+ binários proprietários**, `LICENSE.md`: *"Additional binary files… governed by the more restrictive proprietary license"*) / clangd ext | **clangd** | Apache-2.0 + LLVM exc. | LSP | ✓ |
| configure/build/CTest/presets/kits | CMake Tools (MIT) | cmake, ninja, ctest | BSD-3 / Apache-2.0 | processo | ✓ (ctest sem saída na tela) |
| depurar | CodeLLDB (MIT), cpptools (`vsdbg` proprietário) | lldb-dap, gdb `-i dap` | Apache-2.0 / GPL-3 | DAP | ✓ |
| formatar | clang-format via clangd/cpptools | clang-format | Apache-2.0 | processo | ✓ |
| análise estática | clang-tidy (cpptools/clangd), cppcheck ext | **clang-tidy**; **cppcheck** | Apache-2.0; **GPL-3** | processo | ✗ (só clippy) |
| sanitizers | tarefas do CMake | `-fsanitize` | — | ação de config | ✓ |
| cobertura | Coverage Gutters (MIT) lê lcov/cobertura | gcov + lcov / llvm-cov | GPL (gcc) / Apache | processo | ✗ |
| test explorer C++ | C++ TestMate (MIT): GoogleTest, Catch2, doctest | gtest/catch2/doctest **por descoberta** (`--gtest_list_tests`) | BSD-3 / BSL-1.0 / MIT | processo | ✗ (só ctest) |
| Makefile | Makefile Tools (MIT) | make | GPL-3 | processo | ✗ |
| pacotes | Conan (MIT), vcpkg (MIT) | conan / vcpkg | MIT | processo | ✗ — a Kinein tem o catálogo `FetchContent` pinado (35 §4) |
| documentação | Doxygen ext | doxygen | GPL-2 | processo | ✗ |
| includes | — | include-what-you-use | LLVM | processo | ✗ |
| profiling | — | perf, Valgrind, Hotspot (ref) | GPL | processo | ✗ (L4) |

### 3.2 Rust

| funcionalidade | quem faz lá | ferramenta | licença | forma | hoje |
| --- | --- | --- | --- | --- | --- |
| LSP completo | rust-analyzer ext (MIT/Apache) | rust-analyzer | MIT OR Apache-2.0 | LSP | ✓ |
| depurar | CodeLLDB | lldb-dap | Apache-2.0 | DAP | ✓ |
| lint / formato | rust-analyzer | clippy / rustfmt | MIT OR Apache-2.0 | processo | ✓ |
| testes | rust-analyzer runnables | cargo test; **cargo-nextest** (MIT OR Apache-2.0) | — | processo | ✓ / ✗ |
| `Cargo.toml` | Even Better TOML (MIT) | **taplo** LSP | MIT | LSP | ✗ |
| versões de crates | Dependi | — | **sem arquivo de licença** publicado (medido 2026-09-11) | — | não entra |
| cobertura | — | cargo-llvm-cov (MIT OR Apache-2.0) | — | processo | ✗ |
| auditoria | — | cargo-deny (já no gate), cargo-audit | MIT OR Apache-2.0 | processo | ◐ (gate, não IDE) |
| embarcado Rust | probe-rs ext | cargo-embed/flash, defmt (MIT/Apache), embassy (MIT/Apache), esp-hal 1.2.1 (MIT/Apache), rp-hal, svd2rust, flip-link, cargo-binutils | MIT OR Apache-2.0 | processo / templates | ◐ (probe-rs sim; templates decididos no 35 §5.7) |

### 3.3 Python — o que o VS Code documenta, e a ferramenta aberta por trás

O que a documentação oficial lista (páginas *Tutorial*, *Linting*,
*Formatting*, *Environments*, lidas em 2026-09-11):

```text
IntelliSense          Pylance (PROIBIDO) -> pyright (MIT) / basedpyright (MIT)
linting               Microsoft publica Pylint, flake8, mypy; comunidade: Ruff
formatação            Microsoft publica autopep8, Black, isort; comunidade:
                      Ruff (charliermarsh.ruff, "supports import sorting"),
                      yapf. "Format Selection command fails when using Black"
ambientes             "Python: Select Interpreter" e "Create Environment"
                      (venv, conda). A extensao "Python Environments" acha:
                      venv (./**/.venv), sistema (PATH, /usr/bin...), Conda
                      (`conda info --envs`), Pyenv (~/.pyenv/versions),
                      Poetry (.venv do projeto E ~/.cache/pypoetry/virtualenvs),
                      Pipenv (~/.local/share/virtualenvs). "If uv is installed,
                      the extension uses it automatically" (python-envs.alwaysUseUv)
depuração             "Python Debugger extension" = debugpy
testes                pytest / unittest, Test Explorer
REPL / interativo     "Python: Start Terminal REPL", "Python Interactive" (Jupyter)
refactoring, docstrings, Django/Flask/FastAPI (web — fora do escopo desta IDE)
```

A tradução para a Kinein, com a licença lida no arquivo:

| funcionalidade | ferramenta aberta | licença | forma | hoje |
| --- | --- | --- | --- | --- |
| realce + outline | tree-sitter-python 0.25.0 | MIT | crate (gramática) | ✓ 2026-09-12 — realce, outline, folding, locals no editor e as declarações no índice (`40` §7.19) |
| **interpretador** (o `compile_commands.json` do Python) | precedência do 29 §4.1: `$VIRTUAL_ENV` → `.venv/` → `venv/` → `uv.lock` → `poetry.lock` (`poetry env info -p`) → sistema (avisando); **uv** | uv: MIT OR Apache-2.0 | processo | ✓ na base 2026-09-12 — precedência lida (`index.context`, `python.status`); `.venv` de um clique com `uv venv .venv` ou `python3 -m venv .venv` (`40` §7.24); falta `python.select` e o `uv.lock` |
| LSP | **basedpyright** (PyPI, sem Node) / pyright | MIT (`LICENSE.txt`) | LSP | ✓ 2026-09-13 — `basedpyright-langserver --stdio` detectado, com `python.pythonPath` do interpretador do projeto empurrado após o `initialized` e o `workspace/configuration` respondido; reinicia quando o `.venv` nasce (`40` §7.25) |
| lint + formato + imports | **ruff** (`ruff server`) | MIT | LSP | ✓ Validado em 2026-09-15 (`40` §7.36): servidor companheiro do basedpyright, diagnósticos fundidos e ações no Alt+Enter com preview. `ruff format` e `ruff check` continuam nos caminhos existentes. |
| formato alternativo | black | MIT | processo | ✗ |
| tipos | mypy; `ty` (Astral — medir maturidade antes) | MIT | processo | ✗ |
| depurar | **debugpy** — fala DAP; sobe como `python -m debugpy.adapter` | MIT | DAP — **módulo do interpretador do projeto**, não candidato do kit (a medição corrigiu o esboço) | ✓ 2026-09-13 — breakpoint, locais, evaluate, saída, exitCode, adaptador morto com a sessão, contra o debugpy 1.8.21 real (`40` §7.27); módulo feito (§7.33) e attach TCP com host/porta validado em 2026-09-16 (§7.38) |
| testes | pytest (`--collect-only -q` para descobrir; `-q` + `--junitxml` para rodar), unittest | MIT | processo (runner novo em `test.rs`) | ◐ 2026-09-13 — `python -m pytest -v` com o interpretador do projeto, `-k`, casos e saída no painel (`40` §7.26); falta a descoberta como árvore |
| executar | o interpretador do projeto, ou `uv run` quando há `uv.lock` | — | processo (`run.script` de `.py`; `run.start` com ponto de entrada por evidência) | ✓ 2026-09-13 (`40` §7.26) |
| REPL | `python -i` / `ipython` no painel de terminal | — | processo | ✗ |
| notebooks | vscode-jupyter (MIT) sobre jupyterlab (BSD-3) + ipykernel | BSD-3 | processo (kernel via `jupyter_client`) | ✗ — fatia grande e própria (protocolo + renderização); fica no fim |
| ambientes: criar | `uv venv` / `python -m venv` | — | processo | ✗ |
| instalar ferramenta | `uv tool install basedpyright ruff` ou `pipx` | — | passo do `setup` | ✗ |

### 3.4 MicroPython e CircuitPython — Python **no** microcontrolador

É o cruzamento das duas decisões (Python + embarcado), e o que o ESP32 clássico
da mesa roda hoje sem toolchain nenhum: grava-se o firmware `.bin` oficial pelo
`esptool` e o chip vira um REPL na serial.

| funcionalidade | ferramenta aberta | licença | forma | hoje |
| --- | --- | --- | --- | --- |
| falar com a placa: REPL, `run`, `exec`, `fs cp/ls/cat/rm/mkdir/tree`, `mount`, `mip install`, `edit`, `reset`, `bootloader` | **mpremote** (oficial, `pip install mpremote`; atalhos `a0`/`u0`, `id:<serial>`) | MIT (repo micropython) | processo — o REPL numa aba de terminal, o `fs` como comandos | ◐ 2026-09-13 — REPL pelo `serial.monitor` e `run` pelo Executar (`40` §7.28); `fs`/`mip`/`mount` ainda não |
| completar/tipos do `machine`, `network`… | **micropython-stubs** (`pip install micropython-esp32-stubs --target typings`) + basedpyright com `typingsPath` | MIT (`LICENSE.md`) | passo por projeto | ✗ |
| gerar stubs de uma placa | micropython-stubber | MIT | processo | ✗ |
| firmware | `.bin`/`.uf2` oficiais; `esptool write-flash`, UF2 no Pico | MIT (firmware) | motor de gravar (E4) | ✗ |
| compilar `.mpy` | mpy-cross | MIT | processo | ✗ |
| CircuitPython | drive `CIRCUITPY` + **circup** (bibliotecas) | MIT | processo / cópia de arquivo | ✗ |
| referência de UX | **Thonny** (MIT): painel "Files on device", REPL, "Run current script on device" | MIT | ref | — |
| referência de UX | MicroPico (VS Code) | **MPL-2.0** | ref apenas | — |

### 3.5 Embarcados — depurar e VER (o que o Cortex-Debug e o probe-rs mostram)

O README do Cortex-Debug (MIT, `LICENSE` de Marcel Ball) lista: *"Disassembly
of source code… instruction level breakpoints and stepping"*, registradores do
core, *"SWO Decoding"* com *"Live graphing of decoded ITM data"*,
*"Semi-hosting"*, *"RTT using OpenOCD and J-Link"*, *"RTOS Thread Support in
CALL STACK window (J-Link, OpenOCD, pyOCD)"*, *"Live Watch"*, multi-core; e as
companheiras `mcu-debug`: **Peripheral (SVD) Viewer**, **Memory Viewer**,
**RTOS Views** (todas MIT). O debugger do probe-rs (doc lida em 2026-09-11):
flash no launch (bin/hex/elf/**idf**), RTT com defmt *"in the VSCode Integrated
Terminal"*, *"Navigate and monitor SVD Peripheral registers"*, disassembly,
memória, REPL — e os limites: *"Supports a single thread, for a single core"*,
*"pre-production/Alpha stage"*.

| funcionalidade | ferramenta / mecanismo | licença | forma | hoje |
| --- | --- | --- | --- | --- |
| launch/attach, breakpoints, step, variáveis | probe-rs, gdb `-i dap` | — | DAP | ✓ |
| gravar no launch | probe-rs `flashingEnabled`; gdb `load` | — | DAP | ◐ (decidido 35 §5.7) |
| **RTT / defmt** no console | probe-rs (`rttEnabled`, canais; defmt no canal 0) | — | DAP + evento | ✗ |
| **registradores de periférico (SVD)** | probe-rs `svdFile`; parser **cmsis-svd** (Apache-2.0) para gdb | arquivos SVD do fabricante: **licença por arquivo** (35 §5.7) | DAP / processo | ✗ |
| memória | DAP `readMemory`/`writeMemory` (gdb e probe-rs) | — | DAP | ✗ (tela) |
| **disassembly** | DAP `disassemble` (gdb ≥ 14, probe-rs) | — | DAP | ✗ (tela) |
| registradores do core | escopo `Registers` (a Kinein hoje o **esconde** de propósito, 40 §7.9) | — | DAP | ◐ (existe, não se mostra) |
| RTOS: threads | OpenOCD `rtos` (FreeRTOS, Zephyr, ChibiOS…) via gdb; probe-rs **não** | GPL-2 (processo) | DAP | ✗ |
| SWO / ITM | OpenOCD `tpiu`/`itm`; **orbuculum** (BSD-3) | — | processo | ✗ (tardio) |
| semihosting | OpenOCD `arm semihosting enable`; probe-rs | — | DAP | ✗ |
| core dump / post-mortem | ESP-IDF `espcoredump` (Apache-2.0) | — | processo | ✗ |
| tamanho | `size` (✓ build.size); **bloaty** (Apache-2.0) para "o que cresceu" | — | processo | ◐ |
| uso de pilha | `-fstack-usage` + **puncover** (MIT) | — | processo | ✗ |
| map file | `-Wl,-Map` + parser próprio ou puncover | — | — | ✗ |

### 3.6 Embarcados — gravar, rodar, monitorar

Os motores de gravação são um por família e a IDE **orquestra**, nunca
implementa o protocolo do bootloader (`integracoes/38` §5):

| família | gravar | monitor | licença | hoje |
| --- | --- | --- | --- | --- |
| Espressif | **esptool** 5.4 (`write-flash @build/flash_project_args`), **espflash** 4.6 | `espflash monitor --elf` (decodifica backtrace, defmt), `idf.py monitor` | GPL-2 / MIT+Apache | ✗ (E4/E3) |
| ARM via sonda (STM32, nRF, RP2040…) | **probe-rs** `download`/`run` (grava, reseta, RTT) | RTT | MIT+Apache | ◐ |
| RP2040/RP2350 sem sonda | **picotool** 2.3.1 `load -x`, `reboot -f`; UF2 = copiar arquivo | CDC do firmware | BSD-3 | ✗ |
| STM32 sem sonda | **dfu-util** 0.11 (`0483:df11`), **stm32flash** (AN3155) | VCP | GPL-2 | ✗ |
| OpenOCD (quem já tem `.cfg`) | `openocd -c "program x.elf verify reset exit"` | — | GPL-2 | ✗ |
| Zephyr | `west flash` (runners: openocd, pyocd, jlink, esp32, picotool, dfu-util…) | `west debug`, RTT | Apache-2.0 | ✗ |
| PlatformIO | `pio run -t upload` | `pio device monitor` com filtros (`esp32_exception_decoder`, `time`, `log2file`, `hexlify`, `colorize`…) | Apache-2.0 (Python) | ✗ |
| genérico | — | **tio** 3.9 (dnf), picocom, minicom | GPL-2 | ✗ (E3) |
| AVR/Arduino | avrdude (GPL-2), arduino-cli (GPL-3) | — | — | **decidido fora** (35 §5.7) |
| Nordic | probe-rs cobre nRF; `nrfutil` é licença Nordic (não OSI) | — | — | probe-rs |
| SAM (Arduino Due/Zero) | bossac (BSD-3) | — | — | ✗ (junto com AVR: fora) |
| Zephyr DFU | mcumgr / smpmgr (Apache-2.0) | — | — | ✗ (tardio) |

**O que os filtros do `pio device monitor` ensinam** sobre o monitor da Kinein:
`time` (carimbo), `log2file`, `hexlify`, `nocontrol`/`printable`, e o
`esp32_exception_decoder` — este último é o que o `espflash monitor --elf` faz
nativamente. É a lista de recursos que uma aba de monitor "de mercado" tem, e a
E3 entrega o processo; os filtros vêm depois, se forem pedidos por dor real.

### 3.7 Frameworks e SDKs — a IDE detecta e configura; o código é do usuário

| framework | build | licença | o que a IDE precisa saber | hoje |
| --- | --- | --- | --- | --- |
| **ESP-IDF** 6.1 | CMake + Ninja (`idf.py` é wrapper; `cmake -G Ninja -DIDF_TARGET=`) | Apache-2.0 | `IDF_PATH`, toolchain `xtensa-esp-elf`/`riscv32-esp-elf` no PATH, `build/flasher_args.json`, `sdkconfig` (menuconfig = `idf.py menuconfig` no terminal) | ✗ |
| **Zephyr** | `west build -b <board>` (CMake + Kconfig + devicetree) | Apache-2.0 (west também) | workspace `west`, Zephyr SDK, `board`, runners de `west flash/debug` | ✗ |
| **pico-sdk** | CMake (`PICO_SDK_PATH`, `pico_sdk_import.cmake`) | BSD-3 | SDK, toolchain arm, picotool, UF2 em `build/` | ✗ |
| STM32Cube HAL / CMSIS | CMake (CubeMX gera `CMakeLists`; CubeMX é proprietário) | BSD-3 / Apache-2.0 | linker script, startup, `-DSTM32F4xx` | ◐ (kit genérico) |
| **PlatformIO** | `pio run` (Python; `platformio.ini`) | Apache-2.0 | `pio` como motor de build/upload/monitor/test/check para projetos que já são PlatformIO | ✗ |
| CMSIS-Pack / csolution | `cbuild` (Open-CMSIS-Pack) | Apache-2.0 | `.csolution.yml` | ✗ (tardio) |
| FreeRTOS, TinyUSB, lvgl | fontes no projeto | MIT | nada — entram pelo catálogo de bibliotecas se pedidos | catálogo |
| Arduino | arduino-cli (GPL-3) | — | — | **fora** (decisão 35 §5.7) |

### 3.8 Provar sem placa — simulação e teste de host

| ferramenta | o que dá | licença | forma | hoje |
| --- | --- | --- | --- | --- |
| **QEMU** | Cortex-M (18 máquinas aqui), RISC-V; servidor GDB | GPL-2 | processo (`debugServer`) | ✓ (gate) |
| **Renode** 1.17.0 (2026-09-07) | placas inteiras (ARM M/A/R, RISC-V, x86, Xtensa, SPARC, POWER; 40+ na tabela), servidor GDB, Robot Framework, redes multi-nó | MIT | processo (`debugServer`) | ✗ |
| **Unity** + Ceedling | testes de unidade C no host e no alvo | MIT | processo (runner) | ✗ |
| GoogleTest / Catch2 / doctest | testes C++ no host | BSD-3 / BSL-1.0 / MIT | processo (descoberta) | ✗ |
| ESP-IDF unit test app / Zephyr twister | testes no alvo | Apache-2.0 | processo | ✗ (tardio) |
| Wokwi | simulador — **proprietário** | — | fora | — |

### 3.9 Ambiente: instalar, permitir, achar

| item | ferramenta / fonte oficial | forma | hoje |
| --- | --- | --- | --- |
| portas seriais | sysfs (`serial.list`) | core | ✓ |
| permissão USB por canal | `dialout`; `69-probe-rs.rules` (`uaccess`+plugdev); `60-openocd.rules` do Fedora (só `uaccess`); `ID_MM_DEVICE_IGNORE` | diagnóstico + comando impresso (E2) | ✗ |
| instalar ferramentas de embarcado | dnf: `openocd`, `dfu-util`, `stlink`, `tio`, `arm-none-eabi-gcc-cs`, `qemu-system-arm`; pipx: `esptool`, `platformio`, `mpremote`; script oficial: `probe-rs`; release: `espflash`; git: `pico-sdk`, `picotool`; `install.sh`: ESP-IDF; Zephyr SDK | catálogo do domínio `setup` (hoje só banco/Grafana) | ✗ |
| instalar ferramentas Python | `uv` (script oficial / pipx), `uv tool install basedpyright ruff`, `debugpy`/`pytest` no ambiente do projeto | `setup` | ✗ |
| Docker | decidido nativo, não implementado | — | ✗ |
| SSH remoto | decidido, não arquitetado (40 §4) | — | ✗ |

## 4. O que NÃO entra, e por quê — para ninguém reabrir sem saber

```text
Pylance                       proprietario (so' em produtos Microsoft). basedpyright
Serial Monitor (Microsoft)    codigo FECHADO: o repositorio "contains no source code
                              for the extension itself" (README, 2026-09-11). E'
                              referencia de UX, e so'
Embedded Tools (Microsoft)    DEPRECADO — "no longer be available on the marketplace
                              as of 1/15/2026"; tambem sem fonte
cpptools (binarios)           o motor de IntelliSense e o vsdbg sao proprietarios
                              (LICENSE.md). A parte util ja' e' o clangd
nRF Connect for VS Code       licenca Nordic, nao OSI. nRF entra pelo probe-rs
STM32CubeIDE/MX/Programmer    proprietarios. STM32 entra por probe-rs/OpenOCD/dfu-util
J-Link Software               proprietario; a SONDA J-Link funciona via probe-rs/OpenOCD
Wokwi                         proprietario. Sem placa e' QEMU/Renode
Dependi (VS Code)             sem arquivo de licenca publicado — nao se adota o que
                              nao se pode auditar
MicroPico, pico-vscode        MPL-2.0: referencia de UX (MODE-B), nunca codigo — a
                              decisao de 2026-09-11 e' "MPL nao entra" e vale para
                              crate; extensao nao se linka de todo modo
AVR / Arduino                 decisao do autor em 2026-09-11 (35 §5.7): "outro
                              ecossistema, nao entra". Reabrir e' decisao dele; se
                              reabrir, e' arduino-cli (GPL-3) e avrdude (GPL-2)
                              como processo, e o probe-rs tem ZERO AVR
symbolica                     PROIBIDO (40 §5), categoria Pylance
```

## 5. A ORDEM LINEAR

Critérios da ordem, nesta prioridade: (1) o que **já está em andamento** fecha
antes de abrir frente nova; (2) cada item é **medível na mesa hoje** (ESP32
clássico + o C3/C6 que vai chegar) ou no gate sem placa; (3) capacidade antes
de ferramenta (`integracoes/README`); (4) a vertical Python entra **inteira e
sem anúncio parcial**; (5) o que é grande e próprio (Jupyter, SSH, Docker) vai
ao fim, com data só quando começar.

Cada item nasce com gate (falha silenciosa + mutação), exercitação contra a
ferramenta REAL, e a saída "o que faltou e onde procurei" quando não der.

```text
BLOCO A — fechar o canal serial e o ciclo Espressif (ja' decidido, 38 §6)
 A1  E3 monitor serial como PROCESSO       FEITO 2026-09-12 (40 §7.15): papel
     numa aba de terminal (open_command)    `serialMonitor`, serial.monitor, picocom
                                            exercitado no ESP32
 A2  E5 identidade Espressif                FEITO 2026-09-17 (40 §7.41, 0.112.0):
                                            serial.identify roda `esptool flash-id`
                                            como job, le chip/features/MAC/flash e
                                            SUGERE o kit (tabelas do project.model);
                                            "Usar chip no kit" e' clique. Provado com
                                            esptool falso — sem placa nesta maquina
 A3  E4 gravar como CONFIGURACAO DE         FEITO 2026-09-17 (40 §7.42, 0.113.0):
     EXECUCAO                               runConfig.flashProposal (puro) monta a linha
                                            esptool/probe-rs/picotool/dfu-util da receita
                                            e da porta; "Gravar agora" = run.start,
                                            "Salvar" = runConfig.save. Provado com esptool
                                            falso pelo ciclo real; sem placa
 A4  E2 permissao por canal                 FEITO 2026-09-17 (40 §7.43, 0.114.0):
                                            serial.access mede grupo/ACL uaccess,
                                            ModemManager e regra de sonda; passo
                                            oficial datado escrito no terminal da IDE,
                                            nunca sudo. MEDIDO no ESP32 real
 A5  setup: ferramentas de embarcado        o catalogo do `setup` ganha tio, esptool,
                                            espflash, probe-rs, openocd, picotool, dfu-util,
                                            arm-none-eabi, qemu — comando oficial por distro
 A6  a saida do teste chega a tela          FEITO 2026-09-13 (40 §7.26, com a fatia 3 da
                                            cadeia Python): o painel Testes mostra a saida
                                            bruta e o `error` de um runner que nem correu

BLOCO B — Python, a vertical inteira (reverte o adiamento; sem anuncio parcial)
 B1  Tree-sitter Python                     FEITO 2026-09-12 (40 §7.19): realce + outline +
                                            indice; zero dependencia externa
 B2  interpretador                          FEITO na base 2026-09-12 (40 §7.18 e §7.24):
                                            precedencia do 29 §4.1 resolvida UMA vez
                                            (python::env), lida pelo index.context e pelo
                                            python.status; `.venv` de um clique com o uv
                                            (ou venv) pela faixa de saude; guias oficiais
                                            de pipx/uv/ruff/basedpyright no painel de
                                            instalacao. FALTA `python.select` (escolher
                                            entre varios) e ler o uv.lock
 B3  basedpyright como 3o ServerSpec        FEITO 2026-09-13 (40 §7.25): sobe com o
                                            interpretador do B2 por didChangeConfiguration
                                            + workspace/configuration; reinicia quando o
                                            .venv nasce; binario detectado (~/.local/bin)
 B4  ruff server                            FEITO, validado em 2026-09-15 (40 §7.36):
                                            companheiro do basedpyright, sincronizacao
                                            nos dois, diagnosticos fundidos e acoes no
                                            Alt+Enter. Preview valida a versao do Ruff;
                                            aplica pela transacao existente. Provado
                                            com Ruff 0.16.7 e basedpyright 1.40.1 reais
 B5  debugpy como adaptador                 FEITO 2026-09-13 (40 §7.27): NAO e' candidato
                                            do kit — e' modulo do interpretador do projeto;
                                            todo alvo .py sobe `<interp> -m debugpy.adapter`,
                                            com a sonda `import debugpy` antes e o passo
                                            para instalar no ambiente; "Depurar" na arvore;
                                            ciclo provado contra o debugpy 1.8.21 real
                                            (verificar-python-debug.sh). O `-m pacote`
                                            (launch por `module`) FEITO 2026-09-13 (40
                                            §7.33); attach TCP FEITO em 2026-09-16
                                            (40 §7.38), com campos de host/porta e
                                            desconexao preservando o processo externo
 B6  pytest no test.rs                      FEITO 2026-09-13 (40 §7.26): `python -m pytest
                                            -v` com o interpretador do projeto (ou `uv run`),
                                            `-k` como filtro, casos pelo `-v`, saida no
                                            painel do A6; sem pytest no ambiente, o passo
                                            para instalar NELE. A descoberta (`--collect-only
                                            -q`) como ARVORE antes de rodar, com "rodar so'
                                            este": FEITA 2026-09-13 (40 §7.32, 0.106.0)
 B6b executar Python                        FEITO 2026-09-13 (40 §7.26): "Executar" num
                                            `.py` e o botao Executar (ponto de entrada por
                                            evidencia: main.py/app.py/__main__.py, pacote
                                            com __main__.py, [project.scripts] instalado)
 B7  setup Python                           FEITO 2026-09-12 (40 §7.24, com o B2): pipx,
                                            uv, ruff e basedpyright no catalogo do setup
                                            com o guia oficial (a fonte e' agnostica de
                                            distro; PEP 668 respeitada)
 B8  ANUNCIAR Python                        FEITO 2026-09-13 (40 §7.31), a pedido do autor:
                                            template "Python" no Novo projeto, o resumo
                                            Python na barra de status, o icone do .py na
                                            arvore, "Testar com pytest"/"Analise (ruff)" no
                                            menu. Depois vieram a arvore do pytest (§7.32),
                                            o `-m pacote` e o run.capabilities (§7.33), a
                                            porta escolhida do MicroPython (§7.39). O
                                            que falta e' polimento: stderr DAP/LSP
                                            (40 §4.1). A
                                            REFORMULACAO da tela para o Python e' etapa
                                            PROPRIA, depois do backend (40 §5, 2026-09-13)

BLOCO C — MicroPython / CircuitPython (Python + serial: precisa de A e B)
 C1  mpremote no terminal                   FEITO 2026-09-13 (40 §7.28): num projeto
                                            MicroPython o serial.monitor abre `mpremote
                                            connect <porta> repl` (porta do serial.list;
                                            `u0`/`a0` nunca adivinhados); o autor pode
                                            fixar outro monitor
 C2  arquivos no dispositivo                `mpremote fs ls/cp/rm/mkdir/tree` como painel
                                            (referencia: Thonny "Files on device")
 C3  rodar o arquivo atual na placa         FEITO 2026-09-13 no core (40 §7.28) e
                                            2026-09-17 na tela (40 §7.39, 0.110.0):
                                            "Executar" num .py e o botao Executar
                                            (main.py) rodam `mpremote [connect <porta>]
                                            run <arquivo>`; a porta se escolhe no painel
                                            de Embarcados (chip "Executar" por porta) e
                                            vai como `device` em run.script E run.start.
                                            Sem escolha, a primeira que o mpremote acha.
                                            Provado com mpremote falso: sem placa nesta
                                            maquina
 C4  stubs por placa                        micropython-<port>-stubs em typings/ +
                                            typingsPath no basedpyright do projeto
 C5  firmware MicroPython                   gravar o .bin/.uf2 oficial pelo motor do A3
 C6  CircuitPython                          drive CIRCUITPY (copia) + circup

BLOCO D — embarcado em profundidade: o que o Cortex-Debug/probe-rs MOSTRAM
 D1  console RTT/defmt                      probe-rs: canais no launch, saida em evento
                                            no painel (o C3/C6 da mesa prova)
 D2  registradores de periferico (SVD)      probe-rs svdFile; cmsis-svd para o caminho gdb;
                                            cada SVD auditado por licenca (35 §5.7)
 D3  memoria e disassembly                  DAP readMemory/disassemble -> duas vistas
 D4  registradores do core                  mostrar o escopo que hoje se esconde, sob pedido
 D5  RTOS threads                           OpenOCD `rtos` pelo gdb -i dap (FreeRTOS, Zephyr)
 D6  clang-tidy e cppcheck no quality.run   C/C++ deixa de ter so' clippy
 D7  test explorer C/C++                    gtest/catch2/doctest por descoberta
 D8  cobertura                              gcov/lcov e llvm-cov -> gutters no editor
 D9  uso de pilha e "o que cresceu"         -fstack-usage + puncover; bloaty
 D10 Renode como debugServer                a 2a maquina sem placa, com placas reais
                                            (nRF52840, STM32F4) que o QEMU nao tem

BLOCO E — frameworks: a IDE reconhece o projeto e configura, sem editar a mao
 E1  ESP-IDF                                detectar (project.cmake), IDF_PATH, IDF_TARGET
                                            do A2, toolchain no PATH, menuconfig no terminal,
                                            flasher_args -> A3, size do idf (idf.py size)
 E2  pico-sdk                               PICO_SDK_PATH, UF2 -> A3 (picotool/BOOTSEL),
                                            Debug Probe -> probe-rs
 E3  Zephyr                                 workspace west, `west build -b`, runners de
                                            flash/debug, Kconfig/devicetree como texto
 E4  PlatformIO                             `pio` como motor para projeto com platformio.ini:
                                            run/upload/monitor/test/check
 E5  templates curados                      cortex-m-quickstart, startup+linker CMSIS por
                                            familia (ja' decidido 35 §5.7), esp-hal, rp-hal
 E6  Unity/Ceedling                         testes C no host como runner do test.rs

BLOCO F — grande e proprio; entra sem data, um de cada vez
 F1  Jupyter / janela interativa            kernel via jupyter_client, protocolo e render
 F2  SSH remoto                             ja' na fila (40 §4)
 F3  Docker                                 FEITO em 2026-09-12 (40 §7.14): dominio
                                            `container` — docker|podman, status/list/
                                            images/action/open/compose, icone no rail,
                                            Ctrl+Alt+W. Em 2026-09-13 (40 §7.34) o compose
                                            passou a prometer so' o que funciona
                                            (composeFile). Falta o CONTEXTO REMOTO (dev
                                            containers), que nasce junto com o F2
 F4  SWO/ITM, semihosting, core dump        quando houver placa e dor
 F5  taplo (Cargo.toml), cargo-nextest,     polimento Rust
     cargo-llvm-cov
```

**O que a ordem NÃO promete:** data. Cada item entra pela porta de sempre —
medir, decidir, gate, mutação, exercitação, doc no mesmo gesto — e o
`roadmaps/40` §4 é onde o "próximo" mora. Este documento é o mapa; a fila
continua sendo o 40.

## 6. Fontes (lidas em 2026-09-11)

```text
VS Code Python   code.visualstudio.com/docs/python/{python-tutorial,linting,formatting,environments}
cpptools         github.com/microsoft/vscode-cpptools (LICENSE.md: binarios proprietarios)
Cortex-Debug     github.com/Marus/cortex-debug (README, LICENSE) + mcu-debug/{peripheral-viewer,
                 memview,rtos-views} (MIT) + eclipse-cdt-cloud/vscode-memory-inspector (EPL-2.0)
probe-rs         probe.rs/docs/tools/debugger/ ; github.com/probe-rs/vscode (MIT/Apache)
PlatformIO       docs.platformio.org (what-is-platformio, core/userguide/device/cmd_monitor);
                 platformio-core e platformio-vscode-ide: Apache-2.0
ESP-IDF ext      github.com/espressif/vscode-esp-idf-extension (README, LICENSE Apache-2.0)
Pico ext         github.com/raspberrypi/pico-vscode (README; LICENSE MPL-2.0)
MicroPico        github.com/paulober/MicroPico (LICENSE.txt MPL-2.0)
Serial Monitor   github.com/microsoft/vscode-serial-monitor (README: sem codigo)
Embedded Tools   github.com/microsoft/vscode-embedded-tools (README: deprecado 2026-01-15)
mpremote         docs.micropython.org/en/latest/reference/mpremote.html
micropython-stubs github.com/Josverl/micropython-stubs (README, LICENSE.md MIT)
Renode           github.com/renode/renode (LICENSE MIT; release v1.17.0 2026-09-07);
                 renode.readthedocs.io (supported-boards)
Zephyr / west    github.com/zephyrproject-rtos/{zephyr,west} (Apache-2.0); mylonics/zephyr-ide (Apache-2.0)
licencas lidas   pyright, basedpyright, ruff, uv, mypy, pytest, black, debugpy, vscode-python,
                 vscode-jupyter (MIT); jupyterlab, pico-sdk, picotool (BSD-3); cmsis-svd, bloaty,
                 CMSIS_6 (Apache-2.0); cppcheck (GPL-3); avrdude (GPL-2); arduino-cli (GPL-3);
                 iwyu (LLVM); conan, vcpkg, coverage-gutters, taplo, puncover, Unity, Ceedling,
                 tinyusb, FreeRTOS-Kernel, thonny, circup, micropython (MIT); embassy, defmt,
                 esp-hal (MIT/Apache); CodeLLDB, CMake Tools, TestMate (MIT); Dependi: sem arquivo
ferramentas      integracoes/38 §4 (esptool, espflash, openocd-esp32, esp-gdb, picotool, dfu-util,
                 stm32flash, tio — licenca no arquivo, 2026-09-11)
```
