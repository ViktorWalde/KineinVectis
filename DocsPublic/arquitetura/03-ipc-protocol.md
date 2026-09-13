# 03 — Protocolo IPC

> **O `0.108.0` (2026-09-13, fim de tarde) é o "sintoma do Docker" medido e
> fechado até onde a medição alcança:** o core respondeu certo o tempo todo
> (`status`/`list`/`images` reais do Podman 5.8.4, start/stop reais); o que
> estava errado era a PROMESSA da tela — "compose up" primário e aceso sem
> projeto (um botão primário desligado vestia o âmbar), e aceso com projeto
> sem arquivo de compose (o job só podia falhar). `ContainerStatus.composeFile`
> nasce (o arquivo que a ferramenta pegaria na raiz do workspace), o
> `container.compose` sem arquivo recusa antes do job, e os botões da IDE só
> acendem com o que funciona. Campo novo sobe o minor.
>
> **O `0.107.0` (2026-09-13) fecha dois itens do polimento Python:** o
> **módulo como alvo de debug** — um projeto cujo ponto de entrada é um
> pacote com `__main__.py` é depurado como `-m pacote` (o `module` do launch
> do debugpy, medido no 1.8.21 e provado pelo core real no gate: para no
> breakpoint dentro do pacote, `x=21`, `dobro 42`); `DebugStartResult.program`
> mostra `-m pacote` — e **`run.capabilities`**: o core publica o que
> "Executar" e "Depurar" aceitam por extensão (`runnable: sh bash zsh py`,
> `debuggable: py`), a UI pede uma vez por conexão e não mantém lista
> própria (a mesma invariante do `format.capabilities`; a árvore e o
> explorador tinham a lista duplicada em dois QML). Método novo sobe o minor.
>
> **O `0.106.0` (2026-09-13) é a ÁRVORE DE TESTES antes do primeiro run** —
> o que faltou do B6 do [`roadmaps/41`](../roadmaps/41-ecossistema-embarcados-e-python.md):
> `test.discover { buildSystem? }` é um JOB que lista sem rodar (`pytest
> --collect-only -q`, `cargo test -- --list`, `ctest -N`) e termina em
> `event.test.discovered` com um `TestCaseInfo { id, name, file? }` por
> caso, no **id exato** que o mesmo runner aceita; `test.run` ganhou
> `testId?` — pytest recebe o node id **posicional** (não `-k`), cargo
> `<nome> -- --exact`, ctest `-R ^nome$` escapado — e ele vence `filter`. O
> painel Testes mostra a árvore com o status do último run e um "rodar só
> este" por linha; o caso que roda pinta a linha da árvore porque o id é o
> mesmo. Método, evento e campo novos sobem o minor.
>
> **O `0.105.0` (2026-09-13, à tarde) faz o Python APARECER na IDE** — o B8
> do [`roadmaps/41`](../roadmaps/41-ecossistema-embarcados-e-python.md)
> ("só aqui a tela diz Python"), pedido do autor: `workspace.createProject`
> ganhou o template `python` (PEP 621, layout plano, pytest em `[dev]`,
> `ruff`, `main.py` como ponto de entrada — nada é executado; o `.venv` é o
> clique da faixa de saúde), a barra de status mostra o Python do projeto
> (`python: .venv · 3.14.7 · pybind11 (scikit-build-core)`), o `.py` tem
> ícone na árvore (o `>>>` do REPL — não o logotipo, marca da PSF), o menu
> Build ganhou "Testar com pytest" e "Análise (ruff)". Na mesma passada, por
> decisão do autor: o rail ganhou o ícone de **banco de dados** acima de
> Containers e "Ferramentas" fecha a lista; e Logs/Shell de um container sem
> projeto aberto passaram a dizer antes do clique que a aba de terminal é do
> projeto (o core recusava depois). Valor novo de enum no wire sobe o minor.
>
> **O `0.104.0` (2026-09-13) é o GERENCIADOR DE TOOLCHAIN QUE LÊ O DISCO**
> (`roadmaps/42` §8 itens b e d; `integracoes/39` §3): `toolchain.
> inspectSysroot { path }` diz o que uma pasta de sysroot contém (headers,
> bibliotecas, `lib/`, os `usr/lib/<triple>` do multiarch, quantos `.pc`,
> qual libc — e um veredito: utilizável, só headers, só bibliotecas, vazia);
> `toolchain.importKit { path }` lê um SDK Yocto (o `environment-setup-*`
> carregado pelo `sh`), uma árvore Buildroot (`output/host`) ou uma pasta de
> toolchain e devolve uma PROPOSTA de kit — compiladores, gdb, sysroot,
> triple, arquivo de toolchain, evidência — sem gravar nada; o kit ganhou
> `toolchainFile` (`setKit`/`ToolchainResult`), que vira
> `-DCMAKE_TOOLCHAIN_FILE` no configure quando o preset não declara um.
> Métodos e campo novos sobem o minor.
>
> **O `0.103.0` (2026-09-13) é o PROVEDOR DE INSTALAÇÃO de toolchain**
> ([`integracoes/39`](../integracoes/39-toolchains-por-alvo.md) §5, o item
> de cima da fila depois da cadeia Python): `toolchain.installable` publica
> um catálogo PINADO — nove toolchains (Arm GNU 15.2.rel1 ×3, xPack ×2,
> ATfE 23.1.0, Bootlin 2026.08-1 ×3) com URL, tamanho, SHA-256 **lido na
> fonte em 2026-09-13**, licença, onde vai parar e se já está lá, e o que o
> projeto aberto recomenda; `toolchain.install { id }` é um JOB que baixa
> (`ureq`, TLS por rustls) para `~/.local/share/kinein-vectis/toolchains/
> <id>/<versão>`, confere o SHA-256 **antes** de desempacotar, desempacota
> com o `tar` do sistema e termina em `event.toolchain.installed`; o detector
> lê a pasta da IDE a cada busca, então a toolchain vira candidato do kit sem
> reiniciar. Nunca sem clique, nunca no sistema, nunca sem checksum, nunca
> "latest". Métodos e evento novos sobem o minor.
>
> **O `0.102.0` (2026-09-13) fecha a cadeia Python do
> [`roadmaps/41`](../roadmaps/41-ecossistema-embarcados-e-python.md) bloco B
> — a fatia 5, MicroPython e o módulo nativo:** num projeto MicroPython (a
> MESMA evidência do `project.model`: `main.py`/`boot.py` importando
> `machine`/`board`) o `serial.monitor` abre o **REPL da placa** (`mpremote
> connect <porta> repl`, candidato novo — e último — do papel
> `serialMonitor`) e o `run.script`/`run.start` de um `.py` roda o arquivo
> **na placa** (`mpremote [connect <porta>] run <arquivo>`; `run.script`
> ganhou `device?`); sem mpremote o erro diz o que instalar, em vez de rodar
> um `import machine` no Python do desktop — o pytest continua no host. E
> `python.status` ganhou `nativeModule` (pybind11/nanobind/PyO3 e a
> ferramenta que o instala no ambiente: maturin, scikit-build-core,
> setuptools-rust, setuptools — com a evidência e o comando oficial). Campo e
> parâmetro novos sobem o minor.
>
> **O `0.101.0` (2026-09-13) é a fatia 4 da cadeia Python do
> [`roadmaps/41`](../roadmaps/41-ecossistema-embarcados-e-python.md) bloco B —
> depurar Python com o debugpy DO interpretador do projeto:** um alvo `.py`
> em `debug.start` (explícito, ou o ponto de entrada do Executar num
> workspace Python) não passa pelo kit — o adaptador é `<interpretador> -m
> debugpy.adapter`, o mesmo interpretador do status, do índice, do
> basedpyright, do run e do pytest. Antes de subir, o core **pergunta ao
> interpretador** se o módulo existe (`-I -c "import debugpy"`); sem ele,
> `TOOL_NOT_FOUND` com o passo para instalar NO ambiente (nova variante
> `MissingAdapterModule`, daí o minor). O ciclo inteiro — breakpoint,
> locais, evaluate, saída do programa, `exitCode`, adaptador morto com a
> sessão — está provado contra o debugpy 1.8.21 real por
> `scripts/verificar-python-debug.sh` (a 23ª verificação do gate; fica "não
> provado" na máquina sem debugpy). A árvore ganhou "Depurar" para `.py`.
>
> **O `0.100.0` (2026-09-13) é a fatia 3 da cadeia Python do
> [`roadmaps/41`](../roadmaps/41-ecossistema-embarcados-e-python.md) bloco B —
> executar e testar Python com o interpretador DO PROJETO:** `run.script`
> aceita `.py` (o interpretador da precedência do `29` §4.1, ou `uv run
> python` quando o projeto tem `uv.lock` e o uv existe; sem shell, cwd no
> root); `run.start` sem comando acha o **ponto de entrada por evidência**
> (`main.py`/`app.py`/`__main__.py` na raiz, UM pacote com `__main__.py`, um
> script de `[project.scripts]` instalado) e diz o que procurou quando não
> acha; `test.run` aceita workspace Python — `python -m pytest -v` no mesmo
> interpretador, `-k` com o filtro, cada linha `-v` um `event.test.case`, e
> **sem o módulo pytest naquele ambiente** o erro diz como instalar NELE.
> `event.test.finished` ganhou `error` quando o runner nem correu (campo novo
> no wire, daí o minor). E a saída bruta dos testes finalmente chega à tela
> (`41` A6): o painel Testes mostra `event.test.started`/`output`.
>
> **O `0.99.0` (2026-09-13) é a fatia 2 da cadeia Python do
> [`roadmaps/41`](../roadmaps/41-ecossistema-embarcados-e-python.md) bloco B —
> três contratos existentes ganharam Python, sem método novo:** o servidor de
> Python (`basedpyright-langserver --stdio`, o binário DETECTADO) sobe **com o
> interpretador do projeto** — o core empurra `workspace/didChangeConfiguration
> { python.pythonPath, … }` logo após o `initialized` e responde ao
> `workspace/configuration` que o servidor pergunta, seção a seção; o
> `event.python.finished` com sucesso reinicia esse servidor com o ambiente
> novo. `format.text` formata `.py`/`.pyi` com `ruff format` (o `format.
> capabilities` publica o id `ruff` — valor novo no catálogo, daí o minor). E
> `quality.run` aceita workspace Python: `ruff check --output-format concise
> --no-fix` no root, cada linha um `event.quality.diagnostic`, e sem ruff o
> erro **nomeia a ferramenta e o passo oficial**, não "tipo não suportado".
>
> **O `0.98.0` (2026-09-12, noite) acrescentou o domínio `python`** — a fatia
> 1 da cadeia Python do [`roadmaps/41`](../roadmaps/41-ecossistema-embarcados-e-python.md)
> bloco B: `python.status` (o interpretador que o projeto resolve, se é
> ambiente próprio ou o Python do sistema, com que ferramenta a IDE criaria um
> `.venv`, os arquivos de projeto, a dica) e `python.createEnvironment` (`uv
> venv .venv` ou `python3 -m venv .venv` como JOB, com `event.python.finished`
> ao fim — e o `index.context` recarregado, para o interpretador do projeto
> passar a ser o do ambiente novo). Domínio novo sobe o minor.
>
> **O `0.97.0` (2026-09-12, noite) acrescentou ao `ToolchainResult` o que só
> um processo responde:** `rustTargets` (os alvos Rust instalados, pelo
> `rustup` detectado) e `sysrootHint` (o compilador cross de distro sem o
> sistema alvo, medido com `-print-sysroot`) — a base do gerenciador de
> toolchains do [`integracoes/39`](../integracoes/39-toolchains-por-alvo.md),
> que no mesmo dia pôs no catálogo os triples da indústria e ensinou o
> detector a procurar além do `PATH`. Campos novos sobem o minor.
>
> **O `0.96.0` (2026-09-12, fim de tarde) acrescentou o MODELO POR ALVO do
> `CMake`:** `cmake.targets.list` passa a trazer, por target, artefatos,
> fontes (geradas à parte), linguagens, padrão, includes/defines, sysroot,
> dependências e pasta de fonte — lidos do `codemodel-v2` do file-api, com a
> `toolchains-v1` (CMake ≥ 3.20) pedida junto; e `index.context` ganha
> `targets` (que targets compilam ou listam o arquivo — o inverso) e, quando
> não há `compile_commands.json`, a **unidade vinda do file-api** (grupo de
> compilação + compilador da `toolchains-v1`) — a "CDB em memória" do
> `roadmaps/42` §8. Campos novos sobem o minor.
>
> **O `0.95.0` (2026-09-12) acrescentou o CONTEXTO DE COMPILADOR por arquivo
> ao domínio `index`:** `index.context { path }` diz COM QUE cada arquivo é
> compilado ou executado — a unidade da `compile_commands.json` (compilador,
> `-std`, `-I`, `-D`, diretório) para C/C++, o pacote e alvo do
> `cargo metadata` para Rust, o interpretador resolvido para Python — e
> `IndexStats.context` resume o que o índice carregou (`cdbEntries`,
> `cdbStale`/`cdbStaleBecause`, `cargoTargets`, `pythonInterpreter`). É a
> segunda metade da exigência do autor ("integração profunda de leitura do
> contexto do código/compilador"). Campo e método novos sobem o minor.
>
> **O `0.94.0` (2026-09-12) acrescentou o domínio `index`:** a IDE passa a
> ler o projeto INTEIRO que abre — todas as pastas, arquivos e declarações
> (funções, tipos) de C, C++, Rust e Python — por decisão do autor no mesmo
> dia, sem esperar language server. `index.status` dá os totais, `index.symbols`
> busca por nome, `event.index.progress`/`finished` acompanham o job que o
> `workspace.open` sobe. É a primeira forma concreta do "entender o projeto
> inteiro" da especificação do KSWE, com as gramáticas Tree-sitter do editor.
> Domínio novo sobe o minor.
>
> **O `0.93.0` (2026-09-12) acrescentou o domínio `project`:** o MODELO do
> projeto embarcado (pilar 0 do `roadmaps/42`). `project.model` diz o que o
> projeto É — framework com o arquivo que o prova, SDKs exigidos e se estão
> aqui, artefatos do último build, alvo deduzido com evidência — e
> `event.project.changed` o reemite ao abrir o workspace e ao fim de
> configure/build. Nada é adivinhado calado; o que não se decide vira `hint`.
> No mesmo minor, `serial.monitor` e o papel `serialMonitor` do kit.
>
> **O `0.92.0` (2026-09-12) acrescentou o domínio `container`:** Docker e
> Podman como domínio NATIVO (decisão do autor de 2026-07-17, `roadmaps/28`
> §0, priorizada em 2026-09-12). `container.status` é a tela de "ativar a
> ferramenta" (motor, versão, rootless, socket, responde, compose, passo
> oficial); `list`/`images` leem `ps`/`images` em JSON dos dois motores;
> `action` e `compose` são JOBS com `event.container.finished`; `open` abre
> logs ou um shell numa aba de terminal. A UI nunca chama `docker`. Domínio
> novo sobe o minor.
>
> **O `0.91.0` (2026-09-11) acrescentou o domínio `serial`:** `serial.list`
> enumera as portas seriais USB desta máquina pelo sysfs — `ttyUSB*` (ponte)
> e `ttyACM*` (CDC) — com VID:PID, driver, o que o VID:PID diz do **elo** (nunca
> do chip atrás de uma ponte), a permissão **medida** com `access(2)` e o estado
> do ModemManager via `udevadm`. Nunca abre a porta: abrir aciona DTR/RTS e
> reseta a placa. Domínio novo sobe o minor. É a E1 do
> [`../integracoes/38`](../integracoes/38-conectividade-bare-metal.md) §6.
>
> **O `0.90.0` (2026-09-11) acrescentou `build.size`:** o tamanho do ELF
> medido por `<prefix>size` do kit, com a fração usada de cada região do
> linker script (`BuildSizeParams` → `SizeReport`). Método e tipos novos sobem
> o minor. Detalhe na seção Build; a medição está no
> [`../roadmaps/35`](../roadmaps/35-ambiente-cpp-e-embarcados.md) §5.7.
>
> **O `0.89.0` (2026-09-11) acrescentou o ALVO REMOTO do depurador:**
> `remoteTarget` e `debugServer` no `toolchain.setKit` e no `ToolchainResult`.
> Com eles o adaptador `gdb` (que fala DAP desde a v14) faz `attach` a um
> servidor GDB — QEMU, OpenOCD — que a IDE sobe e mata com a sessao, em vez de
> `launch`. Campos novos no contrato sobem o minor. A medicao esta' no
> [`../roadmaps/35`](../roadmaps/35-ambiente-cpp-e-embarcados.md) §5.7
> e o ciclo completo e' provado no QEMU pelo `scripts/verificar-embarcado.sh`.
>
> **Os `0.83.0`–`0.88.0` (2026-09-05 → 2026-09-10) foram o domínio `sim`** —
> a simulação por conceito, que **saiu do produto em 2026-09-12** por decisão
> do autor (métodos `sim.*` e tipos `Sim*`). As notas destas
> versões estão íntegras em `DocsPrivate/historico/simulacao/`; o número do
> protocolo não volta atrás: versão é história, não inventário.
>
> Antes disso, a sincronização de 2026-09-06 — e **os comentários dentro dos
> comandos, que ainda diziam 128/130**. Comentário dentro de comando envelhece
> igual a número solto; a diferença é que o gate não o vê.
>
> **Escopo, SINCRONIZADO em 2026-09-06 — e a dívida que este cabeçalho
> declarava foi paga.** Em 2026-09-05 ele foi corrigido para parar de afirmar
> cobertura que não tinha: cinco domínios estavam roteados pelo core e ausentes
> daqui. **Os cinco ganharam seção**: `command.*`, `setup.*`, `datasource.*`,
> `grafana.*` e `sim.*` (este último removido com o domínio em 2026-09-12).
>
> **E a sincronização achou dois números errados — nos comandos que os provam.**
> Ambos pela mesma causa: eles grepam literais sem saber o que os literais são.
>
> ```bash
> # METODOS: o braco de despacho nunca comeca com `event.`. Sem o filtro, a
> # contagem inclui dois nomes de EVENTO que aparecem num `match` dentro de
> # `#[cfg(test)]` em handlers/build.rs — e da' 132 em vez de 130.
> grep -rhoE '"[a-z][a-zA-Z]*\.[a-zA-Z][a-zA-Z.]*"\s*(\||=>)' \
>      crates/kinein-core/src/handlers/ crates/kinein-core/src/lib.rs \
>   | grep -oE '"[a-z][a-zA-Z]*\.[a-zA-Z][a-zA-Z.]*"' | tr -d '"' \
>   | grep -v '^event\.' | sort -u | wc -l          # 130
>
> # EVENTOS: cinco sao construidos por `format!("event.{domain}.*")` em
> # handlers/build.rs, com `domain` em {build, quality}. Um grep de literal
> # NAO OS VE, e da' 36 em vez de 41.
> { grep -rhoE '"event\.[a-zA-Z.]+"' crates/kinein-core/src/ | tr -d '"'
>   for d in build quality; do
>     for s in started output diagnostic finished; do echo "event.$d.$s"; done
>   done
> } | sort -u | wc -l                                # 41
> ```
>
> ```text
> protocolo   0.88.0
> metodos     131 roteados
> eventos     41 (36 literais + 5 construidos por format!)
> dominios    30, e os 30 tem secao neste documento
> ```
>
> **Os cinco que só existem por `format!`:** `event.build.started`,
> `event.build.output`, `event.build.diagnostic`, `event.quality.started` e
> `event.quality.output`. Os outros três da mesma família aparecem como literal
> em algum ponto do código e por isso o grep os via.
>
> O protocolo-**alvo** completo está em
> `DocsPublic/especificacoes/arquitetura-interna-core-ipc-jobs.md`. Onde
> divergir, vale o que está implementado no código + `DocsPrivate/ContextoIA.md`.

## Objetivo

O protocolo IPC permite comunicação entre:

```text
Kinein Vectis UI  ←→  Kinein Vectis Core
```

A UI deve mandar comandos e receber respostas/eventos. O core deve executar lógica, chamar ferramentas externas e emitir eventos de estado.

## Transporte inicial

Para o MVP:

```text
stdin/stdout JSON-RPC
```

Depois:

```text
Unix domain socket
```

Futuro:

```text
gRPC/local socket
```

## Formato base

### Request

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "core.ping",
  "params": {}
}
```

### Response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "status": "ok",
    "message": "pong"
  }
}
```

### Error

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": "TOOL_NOT_FOUND",
    "message": "clangd não foi encontrado",
    "details": {
      "tool": "clangd",
      "suggestedCommand": "sudo pacman -S clang"
    }
  }
}
```

### Event

```json
{
  "jsonrpc": "2.0",
  "method": "event.diagnostics.updated",
  "params": {
    "uri": "file:///home/vitor/dev/projeto/src/main.cpp",
    "errors": 1,
    "warnings": 0
  }
}
```

### Diagnóstico comum (`Diagnostic`)

Implementado no protocolo `0.20.0` como modelo comum para Problems. Os eventos
continuam sendo por domínio (`event.build.diagnostic`,
`event.quality.diagnostic`, `event.project.changed

event.lsp.diagnostics`), mas os itens de
diagnóstico usam campos comuns:

```text
Diagnostic {
  id?,
  source: "build|quality|lsp|toolchain",
  severity: "error|warning|note",
  category?,
  message,
  file?,
  line?,        // 1-based, início do range
  column?,      // 1-based, início do range
  endLine?,     // 1-based, fim do range (T6, para o sublinhado)
  endColumn?,   // 1-based, fim do range
  code?,        // código/regra: "E0425", "unused_variables", ...
  jobId?,
  command?,
  target?,
  logRef?
}
```

Estado atual: build usa `source: "build"` / `category: "compiler"`;
quality usa `source: "quality"` / `category: "lint"`; LSP usa
`source: "lsp"` / `category: "lsp"`. `id`, `command`, `target` e `logRef` já
existem no tipo de protocolo, mas ainda só aparecem quando um produtor tiver
dado real para preencher.

`endLine`/`endColumn`/`code` entraram no protocolo `0.35.0` (fatia T6):
`event.lsp.diagnostics` passa a mandar o range completo (o core deriva de
`textDocument/publishDiagnostics`; `end` ausente cai de volta ao início) e o
`code` do servidor (string ou número → sempre string). A UI usa o range para
o sublinhado ondulado no editor e a marca na gutter, e o `code` no detalhe da
aba Problemas. Build/quality ainda não preenchem range (sublinhado deles fica
para o Problems 2.0).

### Resultado de `tools.detect` / `tools.status`

Implementado no protocolo `0.2.0`. `tools.detect` sempre executa a detecção e
atualiza o registro interno; `tools.status` responde com o último resultado
conhecido (detectando na primeira chamada).

Inventário atual: cargo, rustc, rustup, rust-analyzer, cmake, ninja, git,
clangd, clang, clang++ (`id: clangxx`), gcc, g++ (`id: gxx`), gdb, lldb,
ripgrep (`rg`) e fd/fdfind.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "tools": [
      {
        "id": "cargo",
        "displayName": "Cargo",
        "status": "detected",
        "path": "/home/user/.cargo/bin/cargo",
        "version": "cargo 1.96.1"
      },
      {
        "id": "clangd",
        "displayName": "clangd",
        "status": "missing",
        "suggestedInstall": "sudo pacman -S clang",
        "message": "clangd nao foi encontrado no PATH."
      }
    ]
  }
}
```

Estados possíveis de ferramenta (`DocsPublic/07-tooling-lifecycle.md`):
`notConfigured`, `missing`, `detected`, `ready`, `running`, `failed`,
`disabled`. A detecção usa `missing`, `detected` e `failed`; os demais são
reservados para o gerenciamento de processos.

### Scan de ambiente (`environment.scan` — job assíncrono)

Implementado no protocolo `0.20.0`. Roda a mesma detecção de ferramentas de
`tools.detect`, mas como job assíncrono para o fluxo de First Run / Toolchain
Settings. Responde na hora com `{ "jobId" }`, não requer workspace aberto e,
ao concluir, atualiza o mesmo registry consultado por `tools.status`.

```text
event.environment.started   { "jobId", "tools" }
event.environment.tool      { "jobId", "tool": ToolInfo }
event.environment.finished  { "jobId", "success", "total", "detected", "missing", "failed", "tools": [ToolInfo] }
```

Além destes, emite `event.job.created/progress/output/finished`. A UI deve usar
`event.environment.*` para preencher telas de ambiente/toolchain e
`event.job.*` para status bar/lista de jobs. O core nunca instala ferramentas;
`suggestedInstall` continua sendo apenas uma sugestão para ação explícita do
usuário.

### Workspace (`workspace.browse` / `workspace.createFolder` / `workspace.createProject` / `workspace.open` / `workspace.recent.*`)

`workspace.open` foi implementado no protocolo `0.3.0`. `workspace.browse`
foi adicionado no protocolo `0.5.0` para o seletor proprio de workspace da UI.
`workspace.createFolder` e `workspace.createProject` foram adicionados no
protocolo `0.9.0` para permitir o fluxo JetBrains-like de criar pasta/projeto
sem sair da IDE.

`workspace.open` recebe
`{ "path": "/dir" }`, canonicaliza o caminho, identifica o tipo de projeto por
marcadores (precedência: `Cargo.toml` > `CMakeLists.txt` > `pom.xml` >
Gradle > Python) e persiste `.kinein/workspace.json`
(`schemas/workspace.schema.json`).

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "name": "meu-projeto",
    "root": "/home/user/dev/meu-projeto",
    "kind": "rustCargo",
    "markers": ["Cargo.toml", "CMakeLists.txt"],
    "capabilities": { "buildSystems": ["cargo", "cmake"] }
  }
}
```

Desde o protocolo `0.55.0`, `kind` continua sendo a classificação primária
compatível pela precedência de marcadores, enquanto `capabilities.buildSystems`
contém todos os sistemas reconhecidos na mesma varredura. Assim, um repositório
híbrido pode ser `rustCargo` e oferecer Cargo+CMake simultaneamente. O metadata
persistido usa `schemas/workspace.schema.json` 0.2.0; a UI consome o snapshot e
não repete detecção por arquivo.

`workspace.browse` recebe `{ "path": "/dir" }`, canonicaliza o diretorio e
retorna apenas subdiretorios para a UI navegar sem depender de dialogo nativo do
desktop. A UI continua proibida de listar o filesystem diretamente.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "path": "/home/user",
    "parent": "/home",
    "entries": [
      {
        "name": "dev",
        "path": "/home/user/dev"
      }
    ]
  }
}
```

`workspace.createFolder` recebe `{ "parent": "/dir", "name": "modulo" }`.
O core canonicaliza `parent`, valida que `name` é apenas um segmento de caminho
e cria o diretório filho. A resposta é `{ "path": "/dir/modulo" }`.

`workspace.createProject` recebe
`{ "parent": "/dir", "name": "demo", "template": "empty|cppCmake|rustCargo|python" }`.
O core cria o diretório do projeto, aplica o template e abre o projeto como
workspace, retornando o mesmo payload de `workspace.open`.

- `empty`: cria diretório vazio e persiste `.kinein/workspace.json`.
- `cppCmake`: cria projeto C++23/CMake strict e target-based com
  `CMakeLists.txt`, presets Ninja Debug/Release, `src/main.cpp`, diretórios
  `include/` e `tests/`, `.gitignore` e `README.md`.
- `rustCargo`: usa `cargo new --bin --vcs none`; se `cargo` não existir,
  retorna `TOOL_NOT_FOUND`.
- `python` (`0.105.0`, 2026-09-13): geração interna, sem ferramenta —
  `pyproject.toml` como a PEP 621 escreve (`requires-python = ">=3.12"`,
  `[project.optional-dependencies] dev = ["pytest"]`, `[tool.ruff]`,
  `[tool.pytest.ini_options]` com `testpaths` e `pythonpath = ["."]`),
  `main.py` (o ponto de entrada que o botão Executar procura primeiro), o
  pacote `<nome_com_underscores>/__init__.py` **na raiz** — layout plano, para
  `python main.py` e `python -m pytest` acharem o pacote sem `pip install -e .`
  antes do primeiro clique —, `tests/test_main.py`, `.gitignore` com o `.venv`
  e `README.md`. O workspace nasce como `python`; o ambiente é o clique da
  faixa de saúde. Medido em 2026-09-13: o projeto gerado roda, passa no pytest
  e no `ruff check`/`format --check` sem editar nada.

`workspace.status` responde `{ "workspace": <objeto acima> | null }`.
`workspace.close` responde `{ "status": "ok", "closed": <root | null> }`.

**Sessão por workspace** (protocolo `0.24.0`, fatia M1.5 de `DocsPrivate/diario/18`): a
resposta de `workspace.open` ganha o campo opcional
`session: { openFiles: ["/abs/..."], activeFile? }`, presente apenas quando
`.kinein/session.json` existe, tem `schemaVersion` conhecida (1) e ao menos
um arquivo ainda válido — arquivos apagados/fora do root são filtrados no
carregamento e schema desconhecido é ignorado sem erro. A escrita é feita por
`workspace.saveSession { openFiles: ["/abs/..."], activeFile? }` →
`{ files: N }` (N = entradas efetivamente salvas; caminhos inválidos são
pulados, nunca falham a sessão inteira). Em disco os caminhos são RELATIVOS
ao root (mover a pasta do projeto preserva a sessão); no contrato IPC são
sempre absolutos canônicos. A UI salva com debounce (~1.2s) a cada mudança de
abas/aba ativa e restaura pedindo `fs.read` na ordem da sessão, com a aba
ativa por último.
Caminho inexistente ou sem `path` nos params retorna `INVALID_PARAMS`; falha de
IO ao persistir, listar ou criar diretorios retorna `INTERNAL_ERROR`.

**Workspaces recentes globais** (protocolo `0.53.0`, fatia A1 de `DocsPrivate/diario/18`):
somente `workspace.open` e `workspace.createProject` concluídos com sucesso
registram a raiz canônica. O core persiste até 12 entradas em
`$XDG_CONFIG_HOME/kinein-vectis/recent-workspaces.json` (ou
`~/.config/kinein-vectis/`), conforme
`schemas/recent-workspaces.schema.json`; a UI nunca consulta o filesystem.

```text
workspace.recent.list {} → { workspaces: [RecentWorkspace] }
workspace.recent.pin { root, pinned } → mesmo snapshot completo
workspace.recent.remove { root } → mesmo snapshot completo
workspace.recent.clear {} → { workspaces: [] }

RecentWorkspace {
  name: string,
  root: string,
  lastOpenedAt: inteiro (Unix epoch em milissegundos),
  pinned: bool,
  available: bool
}
```

- entradas fixadas vêm primeiro; cada grupo é ordenado por
  `lastOpenedAt` decrescente e uma raiz canônica nunca é duplicada;
- `available` é recalculado pelo core em cada snapshot. Uma pasta removida
  continua listada, desabilitada e removível, sem ser aberta pela UI;
- abrir uma entrada reutiliza `workspace.open` e, portanto, restaura a sessão
  por workspace já existente;
- o formato legado `schemaVersion: 0` sem `pinned` é aceito e promovido na
  próxima escrita. JSON inválido ou schema futuro é tratado como lista vazia;
- pin/remover exigem uma raiz absoluta já registrada e retornam
  `INVALID_PARAMS` caso contrário. Erro de escrita retorna `INTERNAL_ERROR`;
  uma falha ao atualizar o histórico não desfaz uma abertura válida;
- o arquivo guarda apenas nome, raiz, último acesso e fixação: nunca conteúdo,
  credenciais ou contexto de IA.

### Arquivos (`fs.list` / `fs.read` / `fs.createFile` / `fs.createDirectory` / `fs.write` / `fs.rename` / `fs.delete` / `fs.replace`)

Implementado no protocolo `0.4.0`. Todos exigem workspace aberto
(`INVALID_REQUEST` caso contrário) e todo caminho é canonicalizado e
confinado à raiz do workspace (`INVALID_PARAMS` se escapar).

- `fs.list { path }` → `{ path, entries: [{ name, kind: file|directory|other, size? }] }`,
  ordenado diretórios primeiro, depois nome case-insensitive.
- `fs.read { path }` → `{ path, content }`. Limites: arquivo regular, até
  1 MiB, UTF-8 válido (senão `INVALID_PARAMS` com mensagem humana).
- `fs.createFile { path, content? }` → `{ path, bytesWritten }`. Criado no
  protocolo `0.15.0`; o diretório pai precisa existir dentro do workspace e a
  operação falha se o arquivo já existir.
- `fs.createDirectory { path }` → `{ path }`. Criado no protocolo `0.15.0`;
  o diretório pai precisa existir dentro do workspace e a operação falha se o
  caminho já existir.
- `fs.write { path, content, expectedContent }` → `{ path, bytesWritten }`.
  Desde o protocolo `0.45.0`, sobrescreve apenas arquivos existentes cujo
  conteúdo atual ainda seja idêntico a `expectedContent` (o último snapshot
  lido pela UI). Divergência retorna `FILE_CHANGED` com `details.path` e não
  toca o disco. A substituição aceita continua atômica (temp irmão + `fsync` +
  `rename`).
- `fs.rename { from, to }` → `{ from, to }`. Criado no protocolo `0.19.0`;
  renomeia ou move um arquivo ou diretório dentro do workspace. `from` precisa
  existir; `to` não pode já existir e seu diretório pai precisa existir dentro
  do workspace. A raiz do workspace não pode ser renomeada (`INVALID_PARAMS`).
- `fs.delete { path }` → `{ path }`. Criado no protocolo `0.19.0`; remove um
  arquivo ou diretório (recursivo para diretórios) dentro do workspace. A raiz
  do workspace não pode ser removida (`INVALID_PARAMS`). **Desde 2026-09-02**,
  se o arquivo estava aberto num language server, o core manda
  `textDocument/didClose`: documento apagado que continua aberto deixa
  diagnóstico de um arquivo que não existe mais na aba Problemas. A forma da
  mensagem IPC não mudou.

**Mudanças externas (protocolo `0.45.0`, T2):** ao abrir o workspace, o core
inicia `notify` com backend nativo (`inotify` no Linux) e fallback por polling.
O registro é lazy e não recursivo: raiz, diretórios expandidos por `fs.list` e
diretórios de arquivos abertos por `fs.read`. Build/caches e temporários de
save atômico são ignorados. Eventos brutos são deduplicados por uma janela de
180 ms:

```text
event.fs.changed {
  changes: [{ path: "/abs/file", kind: "created|modified|deleted" }]
}
event.fs.watchError { message }
```

A UI recarrega automaticamente uma aba limpa; se houver edição local, preserva
o buffer e exige a escolha explícita entre recarregar o disco ou manter o
local. `event.fs.watchError` é visível, mas a proteção compare-before-save
continua ativa mesmo sem watcher.

### Formatação de buffer (`format.text` / `format.capabilities`)

Implementado no protocolo `0.21.0` (fatia M1.1 de
`DocsPrivate/diario/18-daily-driver-plan.md`). Requer workspace aberto. Formata o conteúdo
do editor com a ferramenta do projeto via stdin/stdout, sem tocar o disco —
salvar continua sendo decisão do usuário. O formatter roda com cwd na raiz do
workspace, então `rustfmt.toml`/`.clang-format` do projeto valem.

- `format.text { path, text }` →
  `{ path, text, changed, formatter }`.
- `path` precisa existir e estar confinado ao workspace (`INVALID_PARAMS` se
  escapar); é ecoado canonicalizado na resposta para a UI descartar respostas
  de uma aba que já mudou.
- Formatter por extensão: `.rs` → `rustfmt` (`--emit stdout`; acrescenta
  `--edition 2021` apenas quando o root não tem `rustfmt.toml`/
  `.rustfmt.toml`); `.c/.cc/.cpp/.cxx/.h/.hh/.hpp/.hxx` → `clang-format`
  (`--assume-filename=<path> --style=file --fallback-style=LLVM`);
  `.py/.pyi` → `ruff` (`0.99.0`, 2026-09-13: `ruff format --stdin-filename
  <path>` — o nome do arquivo decide qual `ruff.toml`/`pyproject [tool.ruff]`
  vale; com `--stdin-filename` e sem caminhos o ruff lê stdin, medido no
  0.16.4). Extensão sem formatter → `INVALID_PARAMS`.
- `changed: false` quando a saída é idêntica ao texto enviado.
- **O binário é o DETECTADO** (`0.99.0`): o handler pergunta ao detector de
  ferramentas — que procura no `PATH` e em `~/.local/bin`, onde pipx/uv põem o
  ruff e o `PATH` do processo da IDE pode não alcançar — e só cai no nome nu
  quando o detector não acha. Binário ausente → `TOOL_NOT_FOUND` (com
  `data.tool`); formatter com exit ≠ 0 → `INTERNAL_ERROR` com o stderr na
  mensagem.
- `format.capabilities {}` → `{ formatters: [ { id, extensions[] } ] }`
  (`0.61.0`). **Não** requer workspace: é o mapa estático de extensões, derivado
  da mesma constante que `formatter_for_path` usa para decidir — o que a UI
  recebe é, por construção, o que o `format.text` vai aceitar.

  **Invariante de camada.** Quem decide o que é formatável é o core; a UI
  consome. Até `0.60.0` o `EditorController.qml` mantinha duas listas escritas à
  mão (`formattableLanguage` por linguagem, `formattablePath` por extensão) que
  não concordavam entre si nem com o core, e adicionar linguagem exigia editar
  QML. Duas fontes para a mesma verdade divergem por construção.

  Capacidade e disponibilidade são perguntas **distintas**: `format.capabilities`
  responde "existe formatter registrado para esta extensão"; se o binário está no
  `PATH` só se descobre ao rodar `format.text` (`TOOL_NOT_FOUND`). O catálogo não
  muda com o workspace, então a UI pode pedi-lo uma vez por conexão.
- Operação síncrona por ser curta (um buffer); "formatar workspace inteiro"
  viraria job, e fica fora deste contrato.

### Busca de arquivos (`fs.findFiles`)

Implementado no protocolo `0.16.0`. Requer workspace aberto. O core usa
`fd` para busca por nome de arquivo, respeitando ignores do projeto e evitando
um indexador próprio no MVP.

- `fs.findFiles { query }` →
  `{ matches: [{ path, name }], truncated }`.
- `query` não pode ser vazio (`INVALID_PARAMS`).
- `path` é relativo à raiz do workspace; `name` é o nome do arquivo.
- A busca usa `fd --type f --fixed-strings --hidden --color never`, com
  exclusões explícitas para `.git`, `.kinein`, `.idea`, `.cache`, `target`,
  `build` e `node_modules`.
- No máximo 100 arquivos são retornados; `truncated: true` indica que o limite
  cortou resultados.
- O core aceita o binário `fd` e também `fdfind` (nome usado por algumas
  distribuições). Se nenhum existir no `PATH`, retorna `TOOL_NOT_FOUND`.

### Busca no workspace (`fs.search`)

Implementado no protocolo `0.11.0` (Find in Files). Requer workspace aberto.

- `fs.search { query, caseSensitive? }` →
  `{ matches: [{ path, line, column, preview }], truncated }`.
- `query` é texto literal (não regex) e não pode ser vazio
  (`INVALID_PARAMS`). `caseSensitive` é opcional e por padrão `false`
  (comparação case-insensitive apenas ASCII, para manter offsets exatos).
- `path` é relativo à raiz do workspace; `line`/`column` são 1-based
  (coluna em caracteres) para consumo direto do editor.
- A caminhada é determinística (profundidade, nome case-insensitive),
  ignora silenciosamente symlinks, arquivos não UTF-8 ou maiores que 1 MiB e
  os diretórios `.git`, `.kinein`, `.idea`, `.cache`, `target`, `build` e
  `node_modules`.
- No máximo 500 matches no total; `truncated: true` indica que o limite cortou
  resultados. `preview` é a linha com trim, limitada a 200 caracteres.
- **`query` pode conter `\n`** (desde 2026-09-02, protocolo `0.65.0`). A busca
  varre o CONTEÚDO do arquivo, não uma linha de cada vez: `line`/`column`
  apontam para o início do match e o `preview` mostra o trecho inteiro com as
  quebras internas trocadas pela marca ` ⏎ `. Match que atravessa linhas continua
  sendo UM match — é isso que faz o contador do preview bater com o número de
  reescritas.
- **Matches não se sobrepõem**: a varredura retoma no fim do match anterior.
  `aa` em `aaa` são 1 match, não 2, e `fs.replace` reescreve exatamente esse 1.

`fs.replace { query, replacement, caseSensitive? }` foi adicionado no
protocolo `0.48.0` (T4). Ele executa substituição literal confirmada no
projeto, com a mesma política de confinamento/ignores e limites da busca:

- a UI exige confirmação e recusa iniciar enquanto houver editor sujo;
- o core lê e calcula todos os novos conteúdos antes de escrever;
- a gravação multi-arquivo usa a primitiva transacional comum, valida os
  snapshots novamente, escreve de forma atômica e faz rollback se qualquer
  arquivo falhar;
- responde `{ files: ["/abs/..."], replacements: N }`; arquivos binários,
  não UTF-8, grandes demais ou em diretórios ignorados não entram;
- `query` vazia retorna `INVALID_PARAMS`; zero ocorrências é sucesso com
  listas/contador vazios.
- **`query` e `replacement` aceitam `\n`** (desde 2026-09-02, protocolo
  `0.65.0`). De 2026-08-29 até essa data o core RECUSAVA (`INVALID_PARAMS`), e a
  recusa estava certa para o que existia então: a busca casava **linha a linha**,
  então uma query multi-linha era invisível para o preview e ativa para a
  escrita — o usuário via "0 resultados" e arquivos eram reescritos mesmo assim.
  O que mudou não foi a guarda, foi a busca: ela passou a varrer o conteúdo
  inteiro e a devolver preview do trecho completo. **A guarda saiu porque a razão
  dela saiu** — a invariante que ela protegia ("o preview conta o que a escrita
  vai fazer") agora vale sozinha, e está travada pelos testes
  `multiline_search_previews_exactly_what_replace_will_rewrite` e
  `a_multiline_replacement_is_found_by_the_next_search`.
- **A sintaxe `\n` do painel é da UI, não do protocolo.** O campo de busca é um
  `TextInput` de uma linha, então `SearchController.expandLineBreaks()` traduz a
  sequência de dois caracteres `\n` digitada pelo usuário em quebra de verdade
  antes de chamar o core (e `\\n` devolve o literal barra-ene). Essa tradução
  **não pode descer para o core**: `query` é texto LITERAL, e um core que
  interpretasse escapes tornaria impossível procurar por um `\n` de verdade
  dentro do código.
- **`fs.search` reporta TODAS as ocorrências de cada linha** (desde 2026-08-29).
  Antes parava na primeira: "Alpha alpha" aparecia como 1 resultado e virava 2
  substituições. O preview de uma operação destrutiva tem de contar o que ela
  vai fazer.
- **paridade com `fs.search` (garantida desde 2026-08-29):** os dois percorrem o
  mesmo walk (`fsops::walk`), então `fs.replace` nunca toca arquivo que
  `fs.search` não mostrou. Consequência da unificação: um subdiretório ilegível é
  **pulado** (como sempre foi na busca), e não mais aborta a substituição inteira
  — só a raiz ilegível é erro. Travado pelo teste
  `replace_touches_exactly_the_files_search_reports`.

### Execução (`run.start` / `run.script` / `run.stdin` / `run.stop`)

Implementado no protocolo `0.12.0`. Requer workspace aberto. O core executa
um comando via `sh -c` na raiz do workspace SEM bloquear o loop de IPC: a
saída chega como notificações assíncronas (mesmo canal dos eventos LSP) e o
processo aceita stdin e cancelamento enquanto roda. Um processo por vez.

- `run.capabilities {}` → `{ runnable: [ext], debuggable: [ext] }`
  (`0.107.0`). **Não** requer workspace: é o mapa estático do que
  `run.script` aceita (`sh`, `bash`, `zsh`, `py`) e do que `debug.start
  { program }` roteia para um adaptador de linguagem sem o kit (`py`),
  derivado da mesma tabela que `script_interpreter` usa para decidir. A UI
  pede uma vez por conexão; antes do catálogo chegar, nada é executável —
  uma lista escrita à mão em QML divergiu da do core por construção (o
  defeito que o `format.capabilities` corrigiu em 0.61.0 voltou a aparecer
  em dois arquivos da árvore, e este método o fecha).
- `run.start { command? }` → `{ command }`. Sem `command`, o core deriva o
  padrão do tipo de projeto: `cargo run` para Rust/Cargo; para CMake, o
  único executável em `.kinein/build` (erro claro se não houver ou houver
  mais de um); **para Python (`0.100.0`, 2026-09-13)**, o ponto de entrada
  por EVIDÊNCIA — `main.py`, `app.py` ou `__main__.py` na raiz (nesta
  ordem); senão UM pacote com `__main__.py` na raiz, depois em `src/`
  (`python -m <pacote>`; dois candidatos = ambiguidade = a IDE não escolhe);
  senão um script de `[project.scripts]` JÁ instalado em `.venv/bin/<nome>`
  — lançado pelo interpretador do projeto ou por `uv run python` (só quando
  o projeto tem `uv.lock` E o uv foi detectado: o uv sincroniza o ambiente
  com o lock antes de rodar, que é o que quem adotou o uv quis). Sem
  interpretador, ou sem entrada, `INVALID_REQUEST` dizendo o que procurou e
  o que fazer (criar o `.venv` pela faixa de saúde; "Executar" num `.py`).
  Outros tipos ainda não têm padrão (`INVALID_REQUEST` com mensagem
  orientando digitar o comando).
- `run.script { path, device? }` → `{ command }` (protocolo `0.55.0`).
  Aceita arquivo regular `.sh`, `.bash`, `.zsh` ou (`0.100.0`) `.py` dentro
  do workspace. O core canonicaliza/confina o caminho e chama `bash`/`zsh` com
  argv explícito (`--`, caminho), sem interpolação por `sh -c`; nomes com
  espaços ou aspas são dados, não sintaxe. Um `.py` roda com o **Python do
  projeto** (`python/env.rs`, o mesmo do `python.status`, do `index.context`
  e do basedpyright — sem medir a versão, que é custo do status) ou com
  `uv run python`, o arquivo como único argumento, cwd no root; `command`
  ecoa `.venv/bin/python 'tools/gera.py'` (interpretador relativo ao root
  quando mora nele) ou `uv run python '…'`. Sem interpretador nenhum,
  `INVALID_REQUEST` orientando a criar o ambiente. **Projeto MicroPython
  (`0.102.0`)**: o `.py` roda **na placa** — `mpremote [connect <device>]
  run <arquivo>` (mpremote 1.29.0, `mpremote run --help`: `run [--follow]
  path`; sem `connect` o mpremote usa a primeira porta serial que acha);
  `device?` é a porta que a tela escolheu (campo ausente ≠ vazio). Sem
  mpremote detectado, `INVALID_REQUEST` dizendo `pipx install mpremote` —
  nunca o Python do desktop, que não tem os pinos de `import machine`. O
  mesmo vale para o `run.start` sem comando (`main.py` na placa, mesmo num
  workspace sem `pyproject.toml`); um pacote com `__main__.py` não roda com
  `-m` na placa e o erro o diz. Reutiliza os mesmos eventos e a mesma sessão
  única de `run.start`; extensão inválida é `INVALID_PARAMS`.
- `run.stdin { data }` → `{ status: "ok" }`. Encaminha `data` cru ao stdin
  do processo (a UI acrescenta o `\n`).
- `run.stop {}` → `{ status: "ok" }`. Mata o processo; o término é
  reportado pelo evento `finished`.
- Fechar o workspace mata o processo em execução automaticamente.

```text
event.run.started   { "command": "cargo run" }
event.run.output    { "stream": "stdout|stderr", "line": "..." }
event.run.finished  { "success": bool, "exitCode": int|null }
```

OBS: `run.*` não tem TTY (pipes simples) — é o executor do botão Run. Para
shell interativo com PTY, ver `terminal.*` abaixo.

### Terminal (`terminal.open` / `terminal.input` / `terminal.resize` / `terminal.scroll` / `terminal.mouse` / `terminal.close`)

Terminal profissional (reescrito no protocolo `0.41.0`, fatia D2 de
`DocsPublic/roadmaps/24`). Requer workspace aberto. **PTY real** via `portable-pty` (do
wezterm) rodando o `$SHELL` interativo em `TERM=xterm-256color` na raiz.
Um **emulador VT** (`alacritty_terminal`, ADR-0004 — o mesmo motor do Alacritty
e do Zed) no core mantém o GRID (células com cor/atributos), cursor, modos, tela
alternada e scrollback; a UI recebe o grid PRONTO e só desenha — sem interpretar
ANSI. Suporta cores, prompts com `\r`, barra de progresso do cargo e TUIs. Desde
o protocolo `0.44.0` (D2.3), o manager mantém até 12 sessões simultâneas. Cada
`open` cria um id monotônico (`t1`, `t2`, …); todos os comandos e eventos
seguintes carregam esse id.

**Invariante de camada.** O terminal é dono da semântica do input. A UI reporta
gestos crus (tecla, roda, clique) e desenha o grid que recebe; ela **não** decide
o que um gesto significa, porque isso depende do modo VT que só o emulador
conhece. Nenhuma política por programa existe no core: uma CLI de IA recebe o
mesmo tratamento de um `ls`.

- `terminal.open {}` → `{ id, shell }`. Cada chamada cria uma sessão nova;
  erro `INVALID_REQUEST` ao atingir o limite de 12 sessões.
- `terminal.input { id, data }` → `{ status: "ok" }`. Encaminha `data` **cru**
  ao PTY. A UI manda CADA tecla (char-a-char), incl. control chars
  (Enter=`\r`, Backspace=`\x7f`, setas=`\x1b[A..D`, Ctrl+letra, …) — não
  linha+Enter.
- `terminal.resize { id, cols, rows }` → `{ status: "ok" }`. Reflui o PTY e o
  grid. A UI calcula cols/rows do tamanho do painel ÷ métrica da fonte mono.
- `terminal.scroll { id, offset }` → `{ status: "ok" }` (D2.2, `0.42.0`).
  Rolagem **explícita** do histórico: `offset` linhas acima do fundo (0 = ao
  vivo; o core **clampa** ao tamanho real do scrollback). É o que a barra de
  rolagem pede, e o snap-to-bottom ao digitar. **Um gesto de roda não é isto** —
  vai por `terminal.mouse`, porque só o core sabe se a aplicação capturou o
  mouse. Copiar/colar são 100% UI (singleton `Clipboard`), sem RPC.
  **Nunca confie no offset da UI:** até `0.43.0` um offset maior que o
  histórico **derrubava o core** (bug de overflow do `vt100` 0.15 —
  corrigido no 0.16, que satura a subtração; o `alacritty_terminal` clampa por
  conta própria). A verdade do offset volta no render (`scrollback`), não na
  resposta.
- `terminal.mouse { id, col, row, event, modifiers? }` → `{ status: "ok" }`
  (R4, `0.60.0`). Um gesto de mouse na grade. `col`/`row` são a célula sob o
  ponteiro, **0-based** (mesma origem do cursor no render); o core converte para
  1-based ao montar o relatório, como o xterm especifica. `modifiers` é
  `{ shift?, alt?, ctrl? }`, todos `false` por omissão.

  `event` é marcado por `kind`:

  ```text
  { "kind": "wheel",   "lines": i16 }          implementado
  { "kind": "press",   "button": "left"|"middle"|"right" }   R5 — recusado
  { "kind": "release", "button": ... }                        R5 — recusado
  { "kind": "motion",  "button": ...|null }                   R5 — recusado
  ```

  O contrato de R5 está fixado para a superfície não mudar de novo; o core
  recusa essas variantes com `INVALID_REQUEST` em vez de fingir que funcionam.

  **A decisão é do core**, lendo o modo VT (ordem vinda do `scroll_wheel` do Zed,
  referência MODE-D):

  ```text
  1. shift ligado                      → rola o histórico local
                                         (válvula de escape do xterm)
  2. aplicação capturou o mouse        → relatório à aplicação
     (MOUSE_REPORT_CLICK/DRAG/MOTION)    SGR se ?1006, senão formato legado
  3. ALT_SCREEN + ALTERNATE_SCROLL     → cursor keys (ESC O A / ESC O B)
  4. caso contrário                    → rola o histórico local
  ```

  A ordem entre 2 e 3 é carga estrutural: `ALTERNATE_SCROLL` nasce **ligado**
  (default do emulador, como no xterm), então uma TUI que captura o mouse
  satisfaz os dois. Quem captura tem precedência — inverter faz a aplicação
  receber setas e não rolar.

  `lines` positivo = para cima (histórico mais antigo); negativo = para baixo;
  `0` é no-op. Um relatório é emitido por linha do gesto. No formato legado uma
  coordenada acima de 223 não é representável e o evento é **suprimido**, não
  truncado (um campo truncado viraria clique em outra célula).
- `terminal.close { id }` → `{ status: "ok" }`. Fecha só a sessão indicada;
  fechar/trocar o workspace ou encerrar o core fecha todas.

```text
event.terminal.render {           (throttle ~30fps; substitui event.terminal.data)
  "id": string,                  (0.44.0 — sessão dona deste grid)
  "cols": u16, "rows": u16,
  "cursor": { "row": u16, "col": u16, "visible": bool,
              "shape": "block"|"underline"|"bar",
              "blinking": bool },       (0.57.0 — DECSCUSR da aplicação)
  "alternateScreen": bool,       (0.51.0 — TUI em tela alternativa)
  "applicationCursor": bool,     (0.51.0 — setas SS3 quando solicitado)
  "bracketedPaste": bool,        (0.51.0 — paste delimitado e seguro)
  "scrollback": usize,            (0.43.0 — offset ATUAL, já clampado: a verdade)
  "scrollbackMax": usize,         (0.43.0 — quanto histórico existe; 0 = nenhum)
  "lines": [ [ { "text": str, "cells": u16,
                 "fg"?: idx|"#rrggbb", "bg"?: idx|"#rrggbb",
                 "bold"?: bool, "italic"?: bool, "underline"?: bool,
                 "inverse"?: bool } ... ] ... ]
}
event.terminal.closed  { "id": string, "exitCode": int|null }
```

`fg`/`bg`: índice 0–255 (paleta) ou `#rrggbb` (truecolor); ausência = cor
default do tema. Cada linha é uma lista de SPANS (runs de células de mesmo
estilo). Desde `0.56.0`, `cells` informa a largura autoritativa do span em
colunas VT; ela não deve ser inferida de `text.length` nem da largura em pixels
da fonte, pois glifos largos, combinantes e fallback tipográfico podem divergir.
O core coalesce runs e descarta o espaço final em estilo default.
Desde `0.57.0`, `shape` e `blinking` preservam o estado VT solicitado por
`DECSCUSR` (`CSI Ps SP q`). O core resolve reset/`DefaultUserShape` para a
preferência do terminal Kinein (`bar`) e sempre emite uma forma concreta; forma
explícita e piscagem pertencem ao programa no PTY. A UI não deve inferir a CLI
ativa nem aplicar geometria específica para Claude/Codex.

`scrollback`/`scrollbackMax` (`0.43.0`, B1/B2 de DocsPublic/roadmaps/24) existem porque a UI
**não tem como saber sozinha** se há histórico nem onde a view está: o core é
quem clampa. Sem eles não dá pra desenhar barra de rolagem honesta, e a UI
acabava pedindo offsets impossíveis. A UI trata `scrollback` como fonte da
verdade (reconcilia o estado local a cada render).

Desde `0.51.0`, o render também expõe os modos VT que alteram a tradução de
entrada. A UI respeita application cursor, envolve colagens com bracketed
paste quando a aplicação o pede e continua sem interpretar a interface do
programa. Resize de painel é coalescido antes de `terminal.resize`, evitando
reserializar um grid por pixel durante o arrasto.

### IA externa: sem domínio próprio (removido no `0.59.0`)

O domínio `aiBridge.*` **não existe mais**. Ele expunha `aiBridge.profiles` e
`aiBridge.terminal.open`, que abriam `claude`/`codex` por um caminho especial:
argumentos injetados pelo core (`--no-alt-screen`, `--ax-screen-reader`) e um
filtro que engolia `CSI 3 J` da própria aplicação.

Esse caminho era a IDE interferindo no terminal. Ele tratava um agente de CLI
como se fosse diferente de qualquer outro programa, o que produziu sintomas
atribuídos ao painel quando a causa estava no emulador raso (ver
`DocsPublic/decisoes-adr/ADR-0004-alacritty-terminal-emulator.md`).

O modelo agora é o de qualquer IDE profissional: **uma CLI de IA é um programa
como outro qualquer**. Abrir um terminal (`terminal.open`) e digitar `claude`
ou `codex` usa o mesmo PTY, o mesmo emulador e o mesmo contrato
`terminal.input/resize/scroll/close` de um `ls`. Não há allowlist de perfil,
argumento imposto, filtro de sequência nem preferência persistida.

Consequência de produto: rodar um agente dentro da Kinein passa a ser
indistinguível de rodá-lo fora dela — que era o requisito original. Uma
superfície visual de atalho (Assistente) pode voltar depois **como UI pura**,
abrindo uma sessão de terminal comum, sem regra de negócio própria no core.

### `build.size` — o tamanho do ELF (síncrono)

**Desde 2026-09-12 (pilar 0 do `roadmaps/42`)**, num projeto ESP-IDF a lista
de regiões ganha a partição `app` que o `flasher_args.json` aponta — usado =
tamanho da imagem, capacidade = a partição que **começa** naquele offset. O
`.ld` do IDF não declara a flash; a partição é a flash.

Implementado no protocolo `0.90.0`. **Síncrono**, ao contrário do `build.run`:
o `size` lê um arquivo e volta em milissegundos. `{ "program"? }` — ausente, o
core resolve o ELF como o `debug.start`. Roda `<prefix>size -A` (o prefixo vem
do compilador cross do kit: `arm-none-eabi-gcc` → `arm-none-eabi-size`; sem
cross, o `size` do sistema) e responde `SizeReport { toolAvailable, tool,
program, sections: [{ name, size, addr }], regions: [{ name, used, size }],
rawOutput }`. As `regions` saem do bloco `MEMORY` do único `.ld` do workspace,
com `used` = soma das seções **alocadas** cujo endereço cai na região —
`.comment`/`.ARM.attributes` (metadados do ELF, endereço 0) não contam. Sem
linker script legível, `regions` vem vazio e a UI mostra os totais por seção.

### Build (`build.run` — job assíncrono)

Implementado no protocolo `0.6.0`; migrado para **job assíncrono** (ver seção
Jobs e `DocsPublic/arquitetura/ARCHITECTURE.md` §7). Requer workspace aberto. O core valida de
forma síncrona e responde **na hora** com `{ "jobId": "job_N" }`; o build roda
em background (`cargo build --message-format=json` para Rust/Cargo;
`cmake -S/-B` + `cmake --build` em `.kinein/build` para CMake) e é **cancelável**
via `job.cancel` (mata o processo de build).

Desde `0.55.0`, aceita `{ "buildSystem"?: "cargo"|"cmake"|... }`. Em workspace
híbrido, a seleção explícita precisa existir em
`workspace.capabilities.buildSystems`; sem o campo, o `workspace.kind` primário
preserva o comportamento anterior. Sistema ausente retorna `INVALID_PARAMS`
com a lista disponível, antes de criar job.

Enquanto roda, emite os eventos ricos que a UI consome, agora com `jobId`:

```text
event.build.started     { "jobId", "command": "cargo build" }
event.build.output      { "jobId", "stream": "stdout|stderr", "line": "..." }
event.build.diagnostic  { "jobId", "source": "build", "category": "compiler", "severity": "error|warning|note", "message", "file"?, "line"?, "column"? }
event.build.finished    { "jobId", "success", "exitCode", "diagnostics" }   // ou { "jobId", "success": false, "error" }
```

Além destes, o Job System emite `event.job.created` (ao iniciar),
`event.job.output` (fan-out da saida bruta) e `event.job.finished` (ao
encerrar) para a status bar / lista de jobs. O
resultado do build chega por `event.build.finished`, **não** mais na resposta.
Diagnósticos vêm do JSON do cargo (span primário) ou do formato
`arquivo:linha:coluna: nivel: mensagem` de compiladores/CMake e alimentam o
Problems. Validação síncrona antes de iniciar o job: tipos sem integração de
build retornam `INVALID_REQUEST`; sem workspace, `INVALID_REQUEST`. Falhas do
build (ferramenta ausente, erro de compilação) chegam por
`event.build.finished`/`event.job.finished`, não como erro da resposta.

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
`cargo` e `python`.

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

### Testes (`test.run` / `test.discover` — jobs assíncronos)

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

### LSP (`lsp.didChange` / `lsp.definition` / `lsp.hover` / `lsp.completion` / `lsp.references` / `lsp.rename`)

Implementado nos protocolos `0.7.0` e `0.8.0`. Requer workspace aberto. A UI envia
`lsp.didChange { "path": "/abs/file", "content": "..." }` depois de debounce
do editor; o core canonicaliza o caminho, confina à raiz do workspace e
sincroniza o texto completo com o servidor LSP gerenciado. A UI nunca fala LSP
diretamente.

O core sobe servidores de linguagem sob demanda quando um arquivo suportado é
aberto, alterado ou salvo:

- C/C++: `clangd --background-index`
- Rust: `rust-analyzer`
- Python (`0.99.0`, 2026-09-13): `basedpyright-langserver --stdio` — o binário
  **detectado** (`~/.local/bin` do pipx/npm entra pelo detector; o `PATH` do
  processo da IDE pode não o ter), `languageId: "python"` para `.py/.pyi/.pyw`.

Se o servidor não estiver instalado ou falhar no `initialize`, o core emite
`event.lsp.status` com `status: "failed"` e mensagem humana. O MVP usa
sincronização de documento inteiro.

**Configuração do servidor (`0.99.0`).** Um `ServerSpec` pode carregar
`settings` (JSON). Quando os há, o core envia `workspace/didChangeConfiguration
{ settings }` **logo após o `initialized`** e responde ao
`workspace/configuration` que o servidor pergunta (LSP 3.17): um valor por
`ConfigurationItem`, navegando `section` com pontos (`python.analysis` →
`settings.python.analysis`); seção inexistente → `null`; item sem `section`
(ou vazia) → a configuração inteira. Servidor sem `settings` (clangd,
rust-analyzer) não recebe configuração alguma — o wire é o mesmo de antes.

Para Python os `settings` são montados pelo core a partir do **interpretador
que o projeto resolve** (`python/env.rs`, a precedência do `roadmaps/29`
§4.1 — o mesmo que `python.status` e `index.context` mostram):
`{ python: { pythonPath, analysis: { autoSearchPaths, diagnosticMode:
"openFilesOnly" } }, basedpyright: { analysis: {…} } }`. Sem interpretador
nenhum (workspace sem `.venv`, sem `python3`) não se empurra configuração — a
IDE não inventa `pythonPath`. `diagnosticMode: "openFilesOnly"` é decisão: o
modo `workspace` faria o basedpyright analisar o projeto inteiro a cada
mudança. O `event.python.finished` com `success: true` (o `.venv` nasceu)
reconfigura e, se o servidor de Python estiver vivo, **reinicia-o** — sai
`event.lsp.restarted { language: "python" }` e a UI reabre os documentos; um
`success: false` não reinicia nada. Sem isto o basedpyright indexaria a stdlib
do Python do `PATH` e o completar mentiria sobre os pacotes do projeto.

**Dívida registrada:** o ruff como *servidor* LSP (code actions "organizar
imports"/"corrigir F401" no Alt+Enter) exige **mais de um servidor por
linguagem** em `lsp/session.rs` — hoje a tabela é `language → um spec`. Fica
no `roadmaps/40` §4; o ruff entrou por `format.text` e `quality.run`.

`lsp.definition` e `lsp.hover` foram adicionados no protocolo `0.8.0`. Ambos
recebem posição 1-based e o buffer atual para o core sincronizar o documento
antes de consultar o language server:

```json
{
  "path": "/abs/file",
  "content": "texto atual do editor",
  "line": 12,
  "column": 5
}
```

`lsp.definition` responde `{ "path"?, "line"?, "column"? }`; campos ausentes
significam que o servidor não encontrou alvo. `lsp.hover` responde
`{ "content"? }`, com markup LSP achatado em texto.

`lsp.completion`, `lsp.references` e `lsp.rename` foram adicionados no
protocolo `0.10.0` (Fase 5.2). Completion e references usam os mesmos
parâmetros posicionais de `lsp.definition`; rename recebe adicionalmente
`"newName"` (não vazio):

- `lsp.completion` responde `{ "items": [{ "label", "insertText",
  "detail"?, "kind"? }], "isIncomplete": bool }`. O core ordena por `sortText`
  e limita a 100 itens; `kind` é o `CompletionItemKind` numérico do LSP
  achatado em texto (`function`, `variable`, ...). **`isIncomplete`
  (protocolo `0.37.0`)** é `true` quando o servidor marcou a lista incompleta
  OU o core truncou (ex.: `std::c` no clangd, centenas de candidatos): a UI
  então **repede** completion ao digitar mais (prefixo maior → itens que não
  couberam, como `cout`), em vez de filtrar só o cache. Sem isso, itens fora
  dos primeiros N nunca apareciam.
- `lsp.references` responde `{ "references": [{ "path", "line", "column" }] }`
  (1-based, limitado a 200 usos, incluindo a declaração).
- `lsp.rename` consulta o servidor e normaliza o `WorkspaceEdit` para a
  transação descrita abaixo. Renames que criam/renomeiam/apagam arquivos
  (resource operations) ainda não são suportados e retornam erro estruturado
  sem tocar em nada.

`lsp.semanticTokens` foi adicionado no protocolo `0.14.0`. Desde `0.54.0`,
recebe `{ path, content, version }`, sincroniza o buffer
e resolve `textDocument/semanticTokens/full`, decodificando os deltas com a
legend anunciada pelo servidor no `initialize`. Responde
`{ path, version, tokens: [{ line, start, length, kind }] }` com `line` 1-based e
`start`/`length` em unidades UTF-16 (0-based) — os mesmos índices de
`QString`, aplicados direto pelo highlighter da UI. `kind` é o nome da
legend (`variable`, `function`, `parameter`, `class`, ...). Servidores sem
suporte respondem lista vazia. A UI incrementa a versão no instante da edição,
limpa tokens anteriores e só aplica resposta cujo `path`+`version` ainda
corresponde ao buffer ativo; o tempo de chegada do LSP nunca substitui a base
Tree-sitter de uma versão mais nova.

`lsp.codeActions` e `lsp.applyCodeAction` foram adicionados no protocolo
`0.22.0` (fatia M1.3 de `DocsPrivate/diario/18-daily-driver-plan.md`):

- `lsp.codeActions { path, content, line, column }` →
  `{ actions: [{ title, kind? }] }`. O core sincroniza o buffer, envia
  `textDocument/codeAction` com range-ponto no cursor e compõe o `context`
  com os diagnostics que o próprio servidor publicou para a linha (cache da
  thread leitora — a UI não devolve diagnóstico). Só entram na lista ações
  `CodeAction` literais com `edit` inline e sem `disabled`; ações que
  dependem de `workspace/executeCommand` são filtradas (decisão registrada
  em DocsPrivate/diario/18). As ações cruas ficam guardadas como **consulta ativa**.
- `lsp.applyCodeAction { path, content, actionIndex }` cria a mesma transação
  confirmável de `lsp.rename`. A consulta é consumida na chamada; índice
  inválido, arquivo diferente ou consulta
  expirada (qualquer didChange real invalida) retornam `INVALID_PARAMS`
  pedindo nova consulta.

**Workspace edits confirmáveis (protocolo `0.47.0`):** rename e code action
respondem primeiro com um preview:

```text
{ transactionId, title, edits,
  files: [{ path, before, after }] }
lsp.workspaceEdit.apply  { transactionId }
  → { transactionId, files, edits }
lsp.workspaceEdit.cancel { transactionId }
  → { transactionId, cancelled: true }
```

O core confina todos os paths, rejeita versões LSP incompatíveis, mantém um
número limitado de planos pendentes e compara cada snapshot do disco outra
vez no `apply`. A escrita multi-arquivo compartilha a primitiva transacional
do `fs.replace`: arquivos atômicos e rollback de todos os já gravados em caso
de falha. Só após sucesso editor, watcher e LSP são ressincronizados. A UI
mostra `before`/`after` lado a lado e nunca aplica edits por conta própria.

### Documentos fechados após um configure (`event.lsp.documentsClosed`)

Nasceu em 2026-09-02 (etapa 4 do `DocsPublic/roadmaps/30-caminho-para-o-mvp.md`).
`{ language, count }`, hoje sempre `language: "cpp"`.

**O que aconteceu:** um `cmake.configure` terminou com sucesso, e o core fechou
(`textDocument/didClose`) os documentos C/C++ que o language server conhecia.

**Por que o core faz isso.** O clangd recarrega a `compile_commands.json`
sozinho desde a v12 — reconfere a cada ~5 s
(<https://reviews.llvm.org/D92663>) —, mas **o documento já aberto fica com a
compilação em cache**. Reiniciar o servidor resolveria e jogaria o índice fora;
a ação certa é reabrir o documento. E reabrir exige **fechar antes**: o
curto-circuito por hash do core torna um `didOpen` repetido inerte, porque
depois do configure o texto não mudou.

**O que a UI deve fazer:** re-sincronizar o arquivo ativo, exatamente como já
faz em `event.lsp.restarted` e no `recovered()`. É esse `didOpen` seguinte que
leva o **buffer do editor** (não o disco) ao servidor, agora com as flags novas.
Sem reagir ao evento, o arquivo ativo fica sem diagnóstico até o usuário digitar.

**Onde a decisão mora, e por quê.** No core, e não na UI: a UI não consegue
fazê-lo sozinha (o `didOpen` seria inerte e o `didClose` não existia até
2026-09-02). O gatilho é o próprio `event.cmake.finished` do job, observado pelo
loop principal antes de ser repassado — o job roda em thread própria e não
alcança o `Core`, mas o evento dele volta ao dono do estado
(`DocsPublic/arquitetura/04-boot-e-comunicacao.md` §3).

### Sintaxe incremental (`syntaxTree.update`)

Implementado no protocolo `0.46.0` para C, C++ e Rust. Recebe
`{ path, content, version }` e responde:

```text
{ path, language, version, hasErrors,
  highlights: [{ line, start, length, scope }],
  foldingRanges: [{ startLine, endLine }],
  outline: [{ name, kind, line, column, endLine, children? }],
  locals: [{ kind, name?, line, column, endLine, endColumn }] }
```

`start`/`length` e colunas usam UTF-16 para casar com Qt/LSP. O core usa
Tree-sitter incremental com cache LRU de 32 buffers e limite de 4 MiB; versão
obsoleta é descartada na UI. A composição visual é regex fallback <
Tree-sitter < semantic tokens LSP < diagnósticos/busca. `locals` é índice
sintático local, nunca promovido a referência semântica.

`lsp.documentSymbols` e `lsp.workspaceSymbols` foram adicionados no
protocolo `0.23.0` (fatia M1.4 de `DocsPrivate/diario/18-daily-driver-plan.md`):

- `lsp.documentSymbols { path, content }` →
  `{ symbols: [{ name, kind, path, line, column, container? }] }`. Achata
  os dois shapes do LSP (`DocumentSymbol[]` hierárquico em pré-ordem, com o
  pai como `container`, ou `SymbolInformation[]` plano), preservando a
  ordem do documento; `kind` é o `SymbolKind` nomeado (`struct`, `method`,
  `function`, ...). Cap de 500 símbolos.
- `lsp.workspaceSymbols { path, content, query }` → mesmo shape. `query`
  não vazia (`INVALID_PARAMS` caso contrário), casada server-side via
  `workspace/symbol` **no servidor da linguagem do arquivo ativo** (`path`)
  — consulta multi-servidor está fora do contrato por ora. Cap de 100.
- Posições 1-based; `path` dos símbolos é canônico/absoluto (a UI abre e
  salta direto, como faz com diagnósticos). URIs fora de `file://` são
  ignoradas.
- Na UI, ambos alimentam o Search Everywhere: prefixo `@` lista/filtra a
  estrutura do arquivo atual; `#nome` busca no workspace.

`lsp.switchSourceHeader` foi adicionado no protocolo `0.34.0` (fatia T1
de `DocsPrivate/diario/18-daily-driver-plan.md`, trilha T de `DocsPublic/roadmaps/21`):

- `lsp.switchSourceHeader { path, content }` → `{ path: string | null }`.
  Alterna entre header e source do mesmo componente C/C++ via a
  **extensão do clangd** `textDocument/switchSourceHeader` (params é o
  `TextDocumentIdentifier` PLANO `{ uri }`, sem envelope; resposta é uma
  URI ou `null`). O core sincroniza o documento antes e converte a URI
  de volta ao caminho absoluto.
- Gate de linguagem no core: só o servidor C/C++ tem a extensão —
  arquivo de outra linguagem → `INVALID_PARAMS` (`UnsupportedFile`),
  nunca deixando o rust-analyzer responder "method not found" cru. Sem
  contraparte → `path` ausente (não é erro). clangd ausente →
  `TOOL_NOT_FOUND`. Na UI: atalho `Alt+O` + comando "C/C++: Alternar
  header/source" no Search Everywhere; a UI abre o contraparte reusando
  o caminho de abertura por diagnóstico.

`lsp.restart` foi adicionado no protocolo `0.39.0` (fatia M4.3b — robustez
do LSP travado):

- `lsp.restart { language? }` → `{ restarted: [string] }`. Reinicia o
  servidor de UMA linguagem (`"rust"`|`"cpp"`); sem `language`, reinicia
  TODOS os vivos. Reiniciar = matar o processo e removê-lo; sobe de novo
  (lazy) no próximo request. `restarted` lista as linguagens que tinham
  servidor rodando.
- **Auto-restart:** o core conta timeouts CONSECUTIVOS por servidor
  (`REQUEST_TIMEOUT` de 4s); ao chegar a 3 seguidos (~12s preso), reinicia
  aquele servidor sozinho. Qualquer resposta zera a contagem.
- Novo evento `event.lsp.restarted { language }` (auto OU comando): a UI
  re-sincroniza o arquivo ativo (o servidor novo não conhece os documentos
  abertos — `refreshSemanticTokens` refaz o `didOpen`, mesmo caminho do
  `recovered()` do crash). Na UI: comando "LSP: Reiniciar servidor" no
  Search Everywhere.

Restart/cancelamento de requests LSP ainda não fazem parte deste contrato.

```text
event.lsp.status       { "language": "cpp|rust", "status": "running|failed|stopped|exited", "message"? }
event.lsp.diagnostics  { "path": "/abs/file", "diagnostics": [{ "source": "lsp", "category": "lsp", "severity": "error|warning|note", "message", "line", "column" }] }
```

`line` e `column` dos diagnósticos são 1-based para consumo direto da UI. A
UI integra esses eventos à aba Problemas com origem `lsp`.

### CMake service (`cmake.configure` / `cmake.presets.list` / `cmake.targets.list` / `cmake.status`)

Implementado no protocolo `0.25.0` (fatia M2.2 de `DocsPrivate/diario/18`). Todos exigem
workspace aberto com kind `cmake` (`INVALID_PARAMS` para outros kinds). O
diretório de build é único e imutável: `<root>/.kinein/build` — o mesmo de
`build.run` e `run.start`; presets **não** mudam o diretório (o `-B`
explícito tem precedência).

- `cmake.configure { preset? }` → `{ jobId }` (job cancelável). Escreve a
  query `codemodel-v2` do file-api antes de rodar e sempre passa
  `-DCMAKE_EXPORT_COMPILE_COMMANDS=ON`. Eventos: `event.cmake.started
  { jobId, command }` e `event.cmake.finished { jobId, success, exitCode,
  hasCompileCommands }`; saída linha a linha vai por `event.job.output`.
- `cmake.presets.list {}` → `{ presets: [{ name, displayName? }] }` —
  configure presets não ocultos de `CMakePresets.json` +
  `CMakeUserPresets.json`, na ordem dos arquivos; JSON inválido → erro
  humano.
- `cmake.targets.list {}` → `{ origin, targets: [{ name, kind, artifacts[],
  sources, generatedSources, languages[], standard?, includes, defines,
  sysroot?, dependencies[], sourceDir? }] }` — lidos da resposta codemodel-v2
  do file-api do último configure (`kind`: `executable`, `staticLibrary`,
  ...; utilitários ficam de fora); vazio antes do primeiro configure. **Desde
  o `0.96.0` (2026-09-12) cada target carrega o seu MODELO** (`cmake/model.rs`):
  `artifacts` absolutos ao build dir (o ELF que o P2 grava), `sources` do
  autor e `generatedSources` (moc/rcc) separados, `languages` na grafia do
  file-api (`C`, `CXX`), `standard` do `languageStandard` do primeiro grupo,
  `includes`/`defines` distintos entre os grupos, `sysroot` quando há
  `CMAKE_SYSROOT`, `dependencies` com os ids resolvidos para nome (um alvo
  importado, fora do codemodel, não entra). Medido neste repositório:
  `kinein-vectis` com 301 fontes + 521 geradas, `CXX` 23, 12 includes, 8
  defines, artefato `.kinein/build/ui/kinein-vectis`. A query passou a pedir
  também a `toolchains-v1` (CMake ≥ 3.20, cmake-file-api(7)): compilador,
  id, versão e includes implícitos por linguagem — vale a partir do próximo
  configure.
- `cmake.status {}` → `{ configured, hasCompileCommands, buildDir,
  cdbDirectory?, cdbStale?, cdbStaleBecause? }` — stat de
  `CMakeCache.txt`/`compile_commands.json`, mais o **diagnóstico da compilation
  database** (protocolo `0.62.0`).
- **`hasCompileCommands` e `cdbDirectory` não são a mesma pergunta**, e confundi-
  las é a origem de "erro de include sem causa":

  ```text
  hasCompileCommands   existe CDB no build dir DA IDE (.kinein/build)?
  cdbDirectory         existe CDB que o clangd ALCANCA, e onde? Relativa ao
                       root; "." e' a propria raiz. AUSENTE quando nao ha.
  ```

  O clangd procura `compile_commands.json` **nos diretórios pai e em
  subdiretórios `build/`** por conta própria
  (<https://clangd.llvm.org/installation>), então um projeto **Meson ou `bear`
  funciona com `hasCompileCommands: false`**. Só `cdbDirectory` responde se o
  usuário tem inteligência de código ou não.
- `cdbStale` + `cdbStaleBecause`: a CDB alcançável é mais velha que um arquivo
  de build que a define (`CMakeLists.txt`, `CMakePresets.json`, `meson.build`,
  `Makefile`). O clangd então usa flags de um projeto que mudou. Os três campos
  são **omitidos quando não há o que reportar** — projeto sadio não carrega
  ruído, e a UI não precisa distinguir `false` de ausente.
- clangd: quando `compile_commands.json` existe no build dir da IDE, novos
  servidores cpp sobem com `--compile-commands-dir` apontando para ele — o
  `.kinein/build/` **não** é `$SRC/build/`, então o clangd não o acharia sozinho.
- **Recarga de flags, corrigido em 2026-08-30.** O clangd **tem** hot-reload da
  `compile_commands.json` desde a v12 (reconfere a cada ~5 s,
  [D92663](https://reviews.llvm.org/D92663)). O que não atualiza é o **documento
  já aberto**, que fica com a compilação em cache
  ([vscode-clangd #42](https://github.com/clangd/vscode-clangd/issues/42)).
  Logo a ação certa após um configure é **reabrir os documentos abertos**, e não
  reiniciar o servidor — reiniciar joga o índice fora.

### Cargo service (`cargo.metadata` / `cargo.check`)

Implementado no protocolo `0.26.0` (fatia M2.3 de `DocsPrivate/diario/18`). Ambos exigem
workspace aberto com kind `rustCargo` (`INVALID_PARAMS` para outros kinds).

- `cargo.metadata {}` → `{ packages: [{ name, version, features: [...],
  targets: [{ name, kind }] }], workspaceMembers: [...] }`. Síncrono:
  `cargo metadata --format-version 1 --no-deps` é local e rápido (medido
  ~10ms num projeto pequeno); o core resume o JSON gigante do cargo.
  Falha vira erro humano (`INTERNAL_ERROR` com a última linha do stderr).
- `cargo.check {}` → `{ jobId }` (job cancelável). Roda `cargo check
  --workspace --all-targets --message-format=json` e **reusa o pipeline do
  quality**: diagnósticos chegam por `event.quality.diagnostic` e o fim
  por `event.quality.finished` — caem na aba Problemas sem parser novo.
  Aliasing consciente até o Problems 2.0 ter facetas por origem; o título
  do job ("Cargo Check") o distingue do clippy na aba Jobs.

### Run configurations (`runConfig.list` / `runConfig.save` / `runConfig.delete` / `runConfig.setActive`)

Implementado no protocolo `0.27.0` (fatia M2.4 de `DocsPrivate/diario/18`). Todos exigem
workspace aberto (qualquer kind). Persistência em `.kinein/runconfigs.json`
com `schemaVersion` (arquivo inválido/schema desconhecido = vazio, nunca
quebra). Uma config v1 é `{ id, name, command }` — comando shell executado
na raiz pelo `run.start`.

- Todas as quatro operações respondem o mesmo shape:
  `{ configs: [{ id, name, command }], activeId? }` — a UI nunca calcula
  estado derivado.
- `runConfig.save { id?, name, command }`: cria quando `id` está ausente
  (ids `cfg-N`); criar/editar torna a config a ATIVA. Nome/comando vazios →
  `INVALID_PARAMS`.
- `runConfig.delete { id }`: remove; se era a ativa, volta a "automático".
- `runConfig.setActive { id? }`: `id` ausente = automático; id inexistente →
  `INVALID_PARAMS`.
- `run.start {}` (sem `command`) resolve: comando explícito > config ativa >
  heurística (`cargo run` / executável único do CMake).

### Configuration Actions (`configAction.list` / `configAction.preview` / `configAction.apply`)

Implementado no protocolo `0.63.0` (etapa 2 de `DocsPublic/roadmaps/30-caminho-para-o-mvp.md`).
Os três exigem workspace aberto. O escopo de cada ação é o **build system
ativo** do workspace (spec 9.2 §1): projeto Cargo não vê ação `CMake`, e pedir
uma fora do escopo é `INVALID_PARAMS`, não silêncio.

**São três métodos porque o ciclo tem três passos**, e a spec 9.1 §10 exige os
três: *explicar, mostrar preview, aplicar com consentimento e validar*. Um
método só seria um botão que edita o `CMakeLists.txt` do usuário sem mostrar o
quê.

- `configAction.list { includeHiddenByScope? }` →
  `{ actions: [...], activeBuildSystems: [...] }`. Cada ação traz
  `{ id, title, description, scope, category, risk, affects, effect, state,
  reason?, params, docs }`. O `state` é medido no workspace agora
  (`available`, `recommended`, `partiallyAvailable`, `unavailable`,
  `hiddenByScope`) e o `reason` diz por quê — "o preset debug já existe",
  "nenhum target declarado". `includeHiddenByScope: true` traz também as ações
  do build system inativo, marcadas (spec 9.2 §25).
- `configAction.preview { id, params? }` →
  `{ id, title, summary, files: [{ path, before?, after }], report, notes }`.
  **Não escreve nada.** `before` ausente significa que o arquivo será CRIADO.
  `report` é o resultado das ações de leitura (o cache do `CMake`); `notes` são
  avisos que o usuário precisa ler antes de aplicar.
- `configAction.apply { id, params?, expected? }` →
  `{ id, message, files, jobId? }`. O `expected` é a lista
  `[{ path, content? }]` que veio do `before` do preview: é a **mesma barreira
  do `fs.write`** (`ARCHITECTURE.md` §7.1) aplicada ao consentimento — se o
  disco mudou entre o preview e o Apply, a resposta é `FILE_CHANGED` e nada é
  escrito. Lista vazia dispensa a comparação.

O `effect` de cada ação diz o que o `apply` faz, e por isso está no contrato:

```text
edit      reescreve um arquivo de texto; o preview mostra o diff exato
inspect   so le: o preview E o resultado, e o apply nao escreve
job       devolve `jobId` reusando um job que ja existe (hoje: cargo.check)
delegate  efeito nao textual dentro do core (remover build dir, salvar run config)
```

As 16 ações do MVP são as da §12 da spec de fechamento: 10 de `CMake`
(`enableCompileCommands`, `createDebugPreset`, `createReleasePreset`,
`addExecutable`, `addStaticLibrary`, `addSourceToTarget`, `addIncludeDirectory`,
`addTargetLinkLibraries`, `inspectCache`, `repairBuildDir`) e 6 de Cargo
(`addDependency`, `addDevDependency`, `addFeature`, `setEdition`, `check`,
`createRunConfig`).

**O que o core RECUSA em vez de adivinhar**, e é o comportamento certo: nome de
target fora do padrão da CMP0037, target inexistente, fonte com `..`,
dependência que já está no manifest, `Cargo.toml` com string multilinha ou
`dependencies` como tabela inline, `edition.workspace` herdada. Corromper o
manifest do usuário não tem desfazer; a recusa vem com o motivo, na lista e no
diálogo.

### Toolchain (`toolchain.get` / `toolchain.set` / `toolchain.setKit` / `toolchain.installable` / `toolchain.install` / `toolchain.inspectSysroot` / `toolchain.importKit`)

Implementado no protocolo `0.64.0` (etapa 5 de
`DocsPublic/roadmaps/30-caminho-para-o-mvp.md`; B2 do TR2 e §5d do `roadmaps/29`).
Ambos exigem workspace aberto. Persistência em `.kinein/toolchain.json` com
`schemaVersion` — arquivo inválido ou de schema desconhecido é tratado como
vazio, e toolchain quebrada nunca impede a IDE de abrir o projeto.

**A pergunta que o domínio responde:** *qual executável cumpre cada papel?*
Até 2026-09-02 todo processo externo do core nascia de `Command::new("<nome>")`
— 28 chamadas, todas resolvidas pelo `PATH` do processo. Trocar de compilador
significava editar `CMakeLists.txt` à mão ou exportar `CC`/`CXX` antes de abrir
a IDE.

Os **papéis** são vocabulário fechado (`cCompiler`, `cxxCompiler`, `generator`,
`cmake`, `cargo`). Papel novo é entrada nova no protocolo e no catálogo do
core, nunca string livre vinda da UI: um papel que o core não sabe usar daria
ao usuário um seletor sem efeito.

- `toolchain.get {}` e `toolchain.set { role, id? }` respondem o **mesmo**
  shape — como as run configs, a UI nunca calcula estado derivado:

```text
{ selections: [{ role, id?, resolvedPath? }],
  candidates: [{ role, id, label, path?, version? }] }
```

- `id` **ausente** em `selections` é AUTOMÁTICO, e é o padrão: nada é fixado e
  o `PATH` continua decidindo — exatamente o comportamento histórico. Quem
  nunca abrir o seletor não vê diferença nenhuma.
- `resolvedPath` no automático é **informação** (o primeiro candidato
  detectado), não fixação. É o que a UI mostra para dizer o que vai acontecer.
- `toolchain.set` com `id` ausente volta para automático. Com um `id` que não
  existe no catálogo, ou que existe mas **não foi detectado nesta máquina**,
  responde `INVALID_PARAMS`: oferecer um compilador ausente é oferecer um
  configure que vai falhar.

**A escolha passou a ser do KIT, não do workspace (protocolo `0.67.0`).** Um kit
é um preset do CMake mais o que ele precisa para compilar: os executáveis por
papel, o `sysroot` e o triple do alvo.

```text
toolchain.get  { preset? }                         -> ToolchainResult
toolchain.set  { role, id?, preset? }              -> ToolchainResult
toolchain.setKit { preset?, sysroot?, targetTriple?, chip?,               NOVO
                   remoteTarget?, debugServer? } -> ToolchainResult
```

**Desde o `0.97.0` (2026-09-12, [`integracoes/39`](../integracoes/39-toolchains-por-alvo.md))
o `ToolchainResult` carrega dois campos que só um processo responde:**
`rustTargets` — os alvos Rust INSTALADOS (`rustup target list --installed`,
pelo `rustup` detectado; **ausente** sem rustup, e a UI então não avisa nada;
com o `targetTriple` do kit fora da lista a UI diz o `rustup target add`) — e
`sysrootHint` — quando o compilador C efetivo é `*-linux-gnu*`, o kit não tem
`sysroot` e o sysroot que o próprio compilador declara (`-print-sysroot`) não
tem `usr/include`: é como as distros empacotam o `gcc-aarch64-linux-gnu`
(medido no Fedora 44), compila e não linka programa de usuário nenhum, e a
dica nomeia as três saídas (rsync da placa, Bootlin, SDK Yocto/Buildroot).
Bare metal não entra. Os candidatos de `cCompiler`/`cxxCompiler`/`debugAdapter`
ganharam no mesmo dia os triples da indústria (`riscv-none-elf`,
`xtensa-esp-elf`, `riscv32-esp-elf`, `aarch64-linux-gnu`,
`arm-linux-gnueabihf`, `riscv64-linux-gnu`; `gdb-multiarch` e os
`<triple>-gdb`), e o detector procura além do `PATH` (o store do xpm, o
`~/.espressif/tools`, a pasta da IDE, `/opt/*/bin`).

`preset` ausente é **o kit padrão do workspace** — que é exatamente o que o
schema 1 do `.kinein/toolchain.json` guardava, e por isso a migração é direta:
as seleções antigas viram o kit padrão **na leitura**, não na escrita. Migrar na
leitura é o que impede a IDE de perder a escolha de quem abriu o projeto e não
mexeu na toolchain.

**Em `toolchain.setKit`, campo ausente NÃO é campo vazio.** Ausente preserva o
valor atual; string vazia (ou só de espaços) limpa. Sem essa distinção, mexer no
`sysroot` apagaria o `targetTriple` e o usuário só descobriria no próximo build.

**O papel `debugAdapter` entrou no protocolo `0.69.0`** (etapa 22 do
`roadmaps/35`). Antes, o adaptador DAP era uma **constante** no core
(`ADAPTER_BINARY = "lldb-dap"`), e por isso não havia debug de embarcado
nenhum: embarcado não debuga com `lldb-dap`.

```text
lldb-dap    desktop; sem argumento          (o PADRAO — nada muda sem escolha)
probe-rs    embarcado; `probe-rs dap-server` fala DAP por stdin/stdout
gdb         `gdb -q -iex ... -i dap`; todo GDB (`gdb-multiarch`, `<triple>-gdb`)
debugpy     Python (0.101.0); NAO vem do kit: o programa e' o INTERPRETADOR do
            projeto e o adaptador e' `-m debugpy.adapter` (DAP por stdin/stdout)
```

**O catálogo da toolchain NÃO sabe os argumentos, e a fronteira é deliberada.**
O cabeçalho dele diz que responde *"quais binários interessam a cada papel"* —
não *"com quais argumentos"*. Que o `probe-rs` precisa do subcomando
`dap-server` é conhecimento de quem **sobe o processo**, e isso mora no domínio
`dap/`.

**O kit ganhou `chip` no protocolo `0.70.0`** — o alvo do adaptador de
embarcado. Ele vai no **`launch` do DAP**, não na linha de comando: verificado
na documentação do probe-rs, onde `chip` é campo da configuração de launch e não
flag do `dap-server`. Supor o contrário daria um processo que sobe e falha no
primeiro request, com a causa longe do sintoma. **Campo ausente não é campo
nulo** — sem chip escolhido, o `launch` não carrega a chave, mesma regra da
condição de breakpoint (`0.66.0`).

**O kit ganhou `remoteTarget` e `debugServer` no `0.89.0`** — o caminho para
depurar o que não está na máquina. Quando o kit escolhe o adaptador `gdb` (que
fala DAP nativamente desde a v14, fonte `/usr/share/doc/gdb/NEWS`) e declara um
`remoteTarget` (`host:porta`), o `debug.start` faz **`attach`** com esse alvo —
*"passed to the `target remote` command"* (manual do GDB, capítulo Debugger
Adapter Protocol) — em vez de `launch`. O `debugServer` é o comando do servidor
(QEMU, OpenOCD) que a IDE sobe antes de conectar, com `{program}` trocado pelo
ELF; ela o mata com a sessão. **Ambos são declarados, nunca deduzidos** (a IDE
não sabe qual máquina do QEMU é a placa): `roadmaps/35` §5.1. Isso derruba o que
o `integracoes/36` §3 dizia — não há protocolo GDB-remote novo a escrever, é um
segundo candidato do papel `debugAdapter`.

**Cross-compilador NÃO virou papel novo**, e a medição corrigiu o esboço do
`roadmaps/35` §5.3 que dizia que sim: `arm-none-eabi-gcc` escreve a **mesma**
variável que o `gcc` — `CMAKE_C_COMPILER`. Dois papéis apontando para a mesma
variável seria duplicação. O cross é **candidato** dos papéis que já existem.

**Adaptador desconhecido não é recusado:** roda sem argumento, que é a forma da
maioria dos adaptadores DAP. Recusar o que não está na lista impediria o usuário
de apontar um adaptador que a IDE não conhece — e a lista é curta por ser nova,
não por ser completa.

O que o kit vira, no build:

```text
sysroot        -> -DCMAKE_SYSROOT=<caminho>
targetTriple   -> --target <triple>          (cargo: check, clippy e build)
               -> -DCMAKE_SYSTEM_NAME=<...>  quando o triple é inequívoco
               -> -DCMAKE_SYSTEM_PROCESSOR=<arch>
```

**Bare metal ganha `CMAKE_TRY_COMPILE_TARGET_TYPE=STATIC_LIBRARY`, e sem isso o
configure falha SEMPRE.** Medido em 2026-09-03 exercitando
`thumbv7em-none-eabihf` com o `arm-none-eabi-gcc` real: `CMAKE_SYSTEM_NAME=
Generic` sozinho não basta — o CMake ainda tenta **linkar** um executável no
teste de compilador, e bare metal não tem os stubs do newlib
(`undefined reference to _exit`). O projeto nem chega a ser configurado. A doc
do CMake diz que `STATIC_LIBRARY` existe exatamente para *"cross-compiling
toolchains that cannot link without custom flags or linker scripts"*.

Só entra em `Generic`. Em cross para Linux/Windows o link funciona, e forçar o
teste a virar biblioteca **esconderia** um toolchain de verdade quebrado.

`CMAKE_SYSTEM_NAME` é o que faz o CMake entrar em modo cross; o sysroot sozinho
não muda a decisão de compilador. **Triple que o mapa não conhece não vira
palpite:** fica sem `CMAKE_SYSTEM_NAME`, e o caminho oficial para alvos exóticos
continua sendo um `toolchainFile` do preset. O `--target` do cargo vale para
check e clippy também — compilar para o alvo e checar para o host daria
diagnóstico do host, que é pior que não ter porque parece certo.

O resultado carrega `presetToolchainFile` quando o preset declara um
`toolchainFile` (campo do schema de presets desde a versão 3 / CMake 3.21, com
precedência sobre `CMAKE_TOOLCHAIN_FILE`). **É informação, não escolha:** quando
ele existe, o compilador efetivo pode não ser o que o usuário escolheu aqui, e a
IDE diz isso em vez de deixar procurar no lugar errado.
- `candidates` traz só o que existe aqui. `Unix Makefiles` só aparece se o
  `make` existir — por isso ele entrou no `tools.detect` na mesma fatia.

**O que a escolha muda de fato**, e é o que faz dela uma entidade e não um
enfeite:

```text
cmake.configure   -DCMAKE_C_COMPILER / -DCMAKE_CXX_COMPILER / -G, e o
                  EXECUTAVEL do proprio cmake
build.run         o cmake (configure implicito + --build) e o cargo
quality.run       o cargo do clippy
cargo.check       o cargo
cargo.metadata    o cargo
```

Só o que o usuário **fixou** entra na linha de comando. Emitir
`-DCMAKE_CXX_COMPILER` com o que o `PATH` resolveria hoje congelaria no cache do
`CMake` uma escolha que o usuário não fez.

**O que esta fatia NÃO entrega, e está registrado:** sysroot, cross-compilação e
kit por preset. São o resto do B2 do TR2, e ficam para uma fatia própria — o
que existe hoje é a entidade e a rota até o comando.

### O provedor de instalação de toolchain (`toolchain.installable` / `toolchain.install`, `0.103.0`)

A fatia do [`integracoes/39`](../integracoes/39-toolchains-por-alvo.md) §5,
entregue em 2026-09-13 com a regra que a decisão registrada e o pedido do
autor juntos permitem: *"zero-config = DETECTAR + UM CLIQUE com o comando
visível, NUNCA download calado"* (`roadmaps/42` §8).

```text
toolchain.installable {}       -> { installRoot, toolchains: [InstallableToolchain],
                                    projectFamily? }
toolchain.install { id }       -> { jobId }        (job; event.toolchain.installed no fim)

InstallableToolchain   id, label, version, family (cortex-m | riscv | aarch64-linux |
                       arm-linux | riscv64-linux), url, sizeBytes, sha256, license,
                       source, installDir, installed, recommended
event.toolchain.installed { jobId, id, version, path, success, error? }
```

**O catálogo é PINADO e medido.** Nove entradas (Linux x86_64): Arm GNU
Toolchain 15.2.rel1 (`arm-none-eabi`, `aarch64-none-linux-gnu`,
`arm-none-linux-gnueabihf`), xPack `arm-none-eabi-gcc` 15.2.1-1.1 e
`riscv-none-elf-gcc` 15.2.0-1, Arm Toolchain for Embedded 23.1.0, Bootlin
glibc stable 2026.08-1 (`aarch64`, `armv7-eabihf`, `riscv64-lp64d`). Cada
SHA-256 foi **lido no arquivo que a fonte publica** em 2026-09-13
(`.sha256asc` da Arm, `.sha` do GitHub release da xPack, `.sha256` da ATfE e
da Bootlin) e cada tamanho veio do `Content-Length` de um HEAD no tarball —
`toolchain/install/catalog.rs` guarda a fonte e a data ao lado de cada
número. Não entram: Espressif (o `idf_tools.py` é o instalador oficial e a
IDE já lê `~/.espressif/tools`), Zephyr SDK (precisa do `setup.sh` —
importar kit, P1), e "latest" de qualquer fonte.

**`toolchain.installable` não exige workspace** (catálogo e pasta são desta
máquina); com um, a `family` do `project.model` marca `recommended` —
`stm32`/`rp2040`/`nrf`/`cortex-m` → `cortex-m`, `riscv` → `riscv`;
`espressif` e `linux` não recomendam nada. `installed` é
`<installRoot>/<id>/<version>/bin` existir.

**`toolchain.install` recusa antes de gastar rede:** id fora do catálogo →
`INVALID_PARAMS` apontando o método da lista; já instalada →
`INVALID_REQUEST` dizendo onde está; `tar` ausente → `TOOL_NOT_FOUND`. O job
então: baixa por `ureq` (TLS por rustls; prazo de 60 s **entre bytes**, não
total — um tarball de 400 MB numa rede lenta leva o que levar) para
`<id>/<version>.part/`, calculando o SHA-256 no caminho e reportando
`event.job.progress` por ponto percentual; **confere o digest ANTES de
desempacotar** — divergiu, o erro mostra os dois digests e nada é
desempacotado; `tar -xf … --strip-components=1 -C` (o `tar` do sistema, GPL,
como processo; todos os tarballs do catálogo trazem uma pasta de topo, e é
ela que sai) numa pasta dentro do `.part`; exige `bin/` no resultado; só então
renomeia para `<version>` e apaga o `.part`. Cancelamento, HTTP ≠ 200, disco
e `tar` com erro deixam no máximo um `.part`, que a próxima tentativa apaga.
Provado com as peças REAIS num servidor local: ureq de verdade, SHA-256 de
verdade, `tar` de verdade (`toolchain/install/mod.rs`, testes).

**Depois do sucesso o detector já enxerga** (`tools/search_dirs.rs`,
`installed_bin_dirs`): a pasta da IDE é enumerada a cada busca, não na
construção do detector; o core refaz o registro de ferramentas ao ver
`event.toolchain.installed { success: true }`, e o `toolchain.get` seguinte
lista o compilador novo como candidato — o kit pode fixá-lo. O `PATH` continua
vencendo a pasta da IDE: o que o usuário escolheu vence o que a IDE
encontrou. A UI (`EmbeddedInstallView`) mostra label, versão, tamanho, URL,
sha256 e licença de cada entrada, recomendadas primeiro, e um botão por linha;
uma instalação por vez.

### O gerenciador que lê o disco (`toolchain.inspectSysroot` / `toolchain.importKit`, `0.104.0`)

Os itens (b) e (d) do `roadmaps/42` §8 e o §3 do `integracoes/39`, entregues
em 2026-09-13. Nenhum dos dois exige workspace nem grava nada: leem caminhos
desta máquina e respondem; aplicar é o `toolchain.setKit` de sempre.

```text
toolchain.inspectSysroot { path }  -> SysrootReport { path, exists,
                                      folders { usrInclude, usrLib, lib },
                                      tripleLibDirs[], pkgconfigFiles, libc?, verdict }
toolchain.importKit { path }       -> KitImport { kind (yocto | buildroot | toolchain-dir),
                                      path, evidence[], cCompiler?, cxxCompiler?, gdb?,
                                      sysroot?, targetTriple?, toolchainFile?, hint? }
toolchain.setKit { …, toolchainFile? }   ("" limpa; ausente preserva)
ToolchainResult.toolchainFile?           (o do KIT; presetToolchainFile e' o do preset)
```

**`inspectSysroot` responde "o `--sysroot` aqui vai achar headers e
bibliotecas?"** Lê `usr/include`, `usr/lib`, `lib`, os `usr/lib/<triple>` e
`lib/<triple>` do multiarch (Debian/Raspberry Pi OS), conta os `.pc` onde o
pkg-config procura (`usr/lib/pkgconfig`, `usr/lib64/pkgconfig`,
`usr/share/pkgconfig`, `<triple>/pkgconfig`), e identifica a libc — glibc pela
`__GLIBC__`/`__GLIBC_MINOR__` de `usr/include/features.h`, musl pela
`libc.so`. O veredito é uma frase: *utilizável* (com libc e `.pc`, ou o aviso
"sem .pc — o pkg-config não vai achar bibliotecas de terceiros"), *só
headers*, *só bibliotecas*, ou *vazia para o compilador* — que é o sysroot de
distro do Fedora medido em 2026-09-12 — com o remédio. Caminho relativo →
`INVALID_PARAMS`; pasta inexistente não é erro, é um relatório com
`exists: false`.

**`importKit` propõe, não grava.** Por evidência no caminho:
- **Yocto** — um `environment-setup-*` (o arquivo, ou a pasta do SDK que
  contém UM). O script é **carregado pelo `sh`** — é o que o manual do SDK
  manda fazer (`. environment-setup-…`, docs.yoctoproject.org sdk-manual,
  lido em 2026-09-13) — e as variáveis que ele exporta são lidas: `CC`/`CXX`/
  `GDB` (o `CC` do SDK já traz `--sysroot=$SDKTARGETSYSROOT`; o binário é
  resolvido por `command -v` no `PATH` que o script montou), `SDKTARGETSYSROOT`,
  `OECORE_NATIVE_SYSROOT` (o toolchain file do SDK mora em
  `usr/share/cmake/OEToolchainConfig.cmake`), `TARGET_PREFIX` (o triple). Um
  script que não exporta `CC`, ou que falha ao carregar, é recusa que diz isso;
  dois scripts na pasta = a IDE não escolhe (aponte o arquivo).
- **Buildroot** — `output/`, `output/host` ou a raiz da árvore: a marca é
  `host/share/buildroot/` (onde fica o `toolchainfile.cmake`), mais
  `host/bin/<triple>-gcc` e `host/<triple>/sysroot` (manual do Buildroot,
  "Using the generated toolchain outside Buildroot", 2026-09-13). Os tarballs
  da **Bootlin** entram por aqui — são SDKs do Buildroot, com o arquivo de
  CMake.
- **toolchain-dir** — uma pasta com `bin/<triple>-gcc` (o tarball da Arm, o
  que a IDE instalou): o sysroot é o que o próprio gcc declara
  (`-print-sysroot`, se ele roda aqui e a pasta existe com conteúdo), senão as
  convenções `<triple>/libc` (Arm) e `<triple>/sysroot`; sem arquivo de CMake —
  o `CMAKE_SYSTEM_NAME` sai do triple, como sempre.

Caminho relativo → `INVALID_PARAMS`; nada reconhecido → `INVALID_REQUEST`
dizendo o que se procurou. **Não medido contra um SDK Yocto ou uma árvore
Buildroot reais** — não há nenhum nesta máquina (42 §8 item 5): o que está
provado é o contrato (um `environment-setup` que exporta as variáveis
documentadas; uma `output/host` com a forma documentada), e a exercitação do
gate lê a raiz do projeto como sysroot "vazia".

**O `toolchainFile` do kit** vira `-DCMAKE_TOOLCHAIN_FILE=<arquivo>` no
configure **só quando o preset não declara o seu** — o do `CMakePresets.json`
vence (dois arquivos brigariam), e a tela já dizia isso. Com o arquivo do
Buildroot/Yocto no kit, o configure é o do SDK: compiladores e sysroot vêm
dele. A UI (`EmbeddedKitImportView`): um campo de caminho, "Ler sysroot",
"Importar kit", a proposta em linhas e "Aplicar proposta ao kit" — um `setKit`
com sysroot, alvo e arquivo; o chip fica. O painel, ao aplicar chip/alvo/
sysroot, **preserva** o arquivo (um sinal não carrega `undefined`: o
controller substitui pelo atual).

### Settings (`settings.get` / `settings.set`)

Implementado no protocolo `0.36.0` (fatia M4.1 de `DocsPrivate/diario/18`). Dois níveis:
GLOBAL (`$XDG_CONFIG_HOME` ou `~/.config`, então `kinein-vectis/
settings.json`, vale para todos os workspaces) e por-WORKSPACE
(`.kinein/settings.json`, sobrepõe o global campo a campo). Ambos com
`schemaVersion` (arquivo inválido/schema desconhecido = vazio, nunca
quebra). O efetivo = `default ← global ← workspace`.

```text
settings.get {} → SettingsResult
settings.set { scope: "global"|"workspace", values: SettingsValues }
             → SettingsResult
SettingsValues { formatOnSave?: bool, editorFontSize?: u32,
                 autoClosePairs?: bool,
                 rigorProfile?: "strict"|"balanced"|"relaxed",
                 explorerWidth?: u32, contextWidth?: u32,
                 assistantTerminalWidth?: u32,
                 bottomPanelHeight?: u32, outlineWidth?: u32,
                 outlineCollapsed?: bool,
                 outlineCollapsed?: bool }
                                          (campos ausentes = não setados)
EffectiveSettings { formatOnSave, editorFontSize, autoClosePairs,
                    rigorProfile, explorerWidth, contextWidth,
                    assistantTerminalWidth,
                    bottomPanelHeight, outlineWidth, outlineCollapsed,
                    outlineCollapsed }
SettingsResult { settings: EffectiveSettings, global: SettingsValues,
                 workspace: SettingsValues }
```

- `settings.get` NÃO exige workspace (o global existe sempre; sem
  workspace, `workspace` vem vazio). Padrão de mutação do repo: `set`
  responde o estado COMPLETO novo.
- `settings.set` faz MERGE parcial de `values` no escopo (campo ausente
  mantém o valor gravado; reverter/limpar override é pós-v1). Defaults:
  `formatOnSave=false`, `editorFontSize=14`, `autoClosePairs=true`,
  `rigorProfile="strict"`, larguras `280/360/640/220` (Project, seletor KV,
  terminal KV ativo e Estrutura), painel inferior `260` e Estrutura expandida.
- Erros: `NO_WORKSPACE` (set `scope=workspace` sem workspace);
  `INVALID_PARAMS` (`editorFontSize` fora de 8..=40, ou `rigorProfile`
  fora do enum; largura/altura de painel fora dos limites, inclusive terminal
  KV ativo fora de 300..=720); `INTERNAL_ERROR`
  (falha de escrita).
- Consumidores atuais: `editorFontSize` → `Theme.fontSizeEditor`;
  `autoClosePairs` → auto-close da E1; `formatOnSave` → Ctrl+S formata e
  então salva; `rigorProfile` (M4.5) → flags de `quality.run`/`build.run`
  NO PROJETO DO USUÁRIO (clippy pedantic/nursery + `-D warnings` no
  strict; clippy default no balanced; só `clippy::correctness` +
  build sem `RUSTFLAGS` no relaxed). NUNCA regula o gate do repo Kinein.
  As dimensões persistem o layout que o usuário redimensionou;
  `outlineCollapsed` persiste o recolhimento da Estrutura.
  `diffBase` (head|index) fica para uma micro-fatia futura (precisa de
  `base` no `git.fileDiff`).

### Rascunhos / autosave (`draft.save` / `draft.clear`)

Rede de segurança contra perda de dado (protocolo `0.40.0`, fatia S1 de
`DocsPublic/seguranca/23`). O core persiste em **SQLite** (`.kinein/kinein.db`, WAL +
escrita atômica por transação) o buffer NÃO SALVO de arquivos sujos. Se a
UI cair (ou power loss), no próximo `workspace.open` as edições voltam.
Complementa a escrita atômica de `fs.write` (temp + `fsync` + `rename`, que
elimina o arquivo truncado/zerado em crash).

```text
draft.save { path, content } → { savedAt }   (autosave de buffer sujo)
draft.clear { path }          → { ok: true }  (save/close limpo)
workspace.open ... → { ..., drafts?: [{ path, content, savedAt }] }
```

- `path` é absoluto e confinado à raiz (como `fs.*`); a chave é o caminho
  absoluto. A UI autosalva com debounce (~1.5s) só quando o buffer está
  modificado; ao fechar a aba manda `draft.clear`.
- `fs.write` **também** limpa o rascunho do path salvo (o arquivo em disco
  passou a ser a verdade). Então rascunho **só sobrevive a um CRASH**.
- Recuperação: embutida na resposta de `workspace.open` (`drafts`), já
  FILTRADA — rascunho igual ao disco é obsoleto e apagado; só volta o que
  DIFERE. A UI abre a aba, sobrepõe o buffer e a marca MODIFICADA (o
  savedContent segue o disco: salvar/reverter continuam corretos).
- Erros: `NO_WORKSPACE` (sem workspace); `INVALID_PARAMS` (path fora da
  raiz / inexistente); `INTERNAL_ERROR` (store indisponível/falha).

### Debug (`debug.*` — sessao DAP via lldb-dap)

Implementado no protocolo `0.28.0` (fatia M2.5a de `DocsPrivate/diario/18`). O core
orquestra o `lldb-dap` (pacote `lldb`) pelo Debug Adapter Protocol; a UI
nunca fala DAP — recebe eventos `event.debug.*` ja mastigados. Uma sessao
por workspace.

- `debug.start { program? }` → `{ program }`. Sem `program`, resolve o
  alvo "Automatico" espelhando o run: cargo → unico executavel no topo de
  `target/debug`; cmake → unico executavel de `.kinein/build`; **Python
  (`0.101.0`) → o mesmo ponto de entrada do Executar** (`main.py`/`app.py`/
  `__main__.py` na raiz, ou o script de `[project.scripts]` instalado; um
  pacote `-m x` não é arquivo — o erro aponta o `x/__main__.py`); zero ou
  varios candidatos → erro claro com a acao a tomar. NAO compila antes
  (build e acao explicita, Ctrl+F9). Erros: `TOOL_NOT_FOUND` (lldb-dap
  ausente), `INVALID_REQUEST` (sem alvo/sessao ja viva), `INVALID_PARAMS`
  (program inexistente), `INTERNAL_ERROR` (falha do adapter).

  **Alvo `.py` — ou um MÓDULO (`0.101.0`/`0.107.0`, cadeia Python,
  2026-09-13).** Seja qual for o tipo do workspace (um CMake com
  `tools/gera.py` inclusive), um `program` terminado em `.py` não passa pelo
  kit; e num workspace Python cujo ponto de entrada é um pacote com
  `__main__.py`, o alvo automático é o módulo — o `launch` leva `module:
  "pacote"` em vez de `program` (medido no debugpy 1.8.21: para no
  breakpoint dentro do pacote; `DebugStartResult.program` = `-m pacote`;
  um servidor de debug do kit não aceita módulo, e diz isso): o adaptador é o
  **debugpy do interpretador do projeto** (`python/env.rs`, a precedência do
  `29` §4.1) — `<interpretador> -m debugpy.adapter`, DAP por stdin/stdout,
  `launch { program, cwd }` como o desktop. O `console` fica de fora de
  propósito: o `initialize` desta sessão não declara
  `supportsRunInTerminalRequest`, e o debugpy cai sozinho no
  `internalConsole` (stdout/stderr do programa chegam como `event.debug.
  output`) — medido no 1.8.21 com e sem o campo. Antes de subir, o core
  roda `<interpretador> -I -c "import debugpy"` (isolado: o que o adaptador
  vai ver; prazo de 10 s): sem interpretador → `INVALID_REQUEST` orientando
  a criar o ambiente; sem o módulo → `TOOL_NOT_FOUND` dizendo o passo para
  instalar NELE (`uv add --dev debugpy` ou `.venv/bin/python -m pip install
  debugpy`) — sem isso o adaptador morreria no primeiro request e a falha
  seria um timeout do `initialize`, longe da causa. O debugpy **não sai
  sozinho** no `disconnect` (medido): é o `Drop` da sessão que o mata, como
  já fazia com os outros. Provado ponta a ponta pelo core real em
  `scripts/verificar-python-debug.sh` (breakpoint em `app.py:2`, `soma` no
  topo da pilha, `a=2 b=3` em Locals — não Globals —, `a + b` = `5`,
  `resultado 5` por `event.debug.output`, `exitCode 0`, adaptador morto;
  1,4 s).
- `debug.setBreakpoints { file, lines: [int] }` →
  `{ breakpoints: [{ line, verified }] }`. Conjunto COMPLETO por arquivo
  (lines vazio limpa); `file` confinado ao workspace. Sem sessao viva o
  conjunto e guardado (`verified: false`) e replayado no proximo launch;
  com sessao, o adapter responde o que de fato amarrou.
- `debug.continue` / `debug.next` / `debug.stepIn` / `debug.stepOut`
  exigem processo pausado; `debug.pause` pausa o processo em execucao;
  `debug.stop` desconecta educadamente e mata o adapter. Todos respondem
  `{ status: "ok" }`; exigem apenas o manager (como `run.stop`).

Inspeção (protocolo `0.29.0`, fatia M2.5c), sempre da thread pausada
(`INVALID_REQUEST` fora disso):

- `debug.stackTrace {}` → `{ frames: [{ id, name, file?, line? }] }` —
  topo primeiro, até 20 frames.
- `debug.variables { frameId }` → `{ frameId, variables }` — o core
  resolve os scopes DAP internamente e devolve as variáveis do primeiro
  escopo não-caro (Locals); Globals/Registers ficam pós-M2.
- `debug.variables { ref }` → `{ ref, variables }` — expande uma variável
  estruturada. Variável: `{ name, value, type?, ref }` (`ref` 0 = folha,
  > 0 = expansível). Exatamente um de `frameId`/`ref` → senão
  `INVALID_PARAMS`. A resposta ECOA a chave pedida para a UI correlacionar
  (mesmo padrão do `format.text`).
- `debug.evaluate { expression, frameId? }` → `{ expression, result, typeName?,
  reference }` (protocolo `0.66.0`) — avalia uma expressão (watch) no frame
  pedido. **Sem `frameId`, o core usa o frame do TOPO** da thread parada: é o
  que a UI quer quando o usuário digita um watch sem ter escolhido um frame.
  `reference > 0` significa expansível e alimenta o MESMO `debug.variables
  { ref }` acima — um watch é uma variável avaliada sob demanda, não uma
  árvore paralela. A resposta ECOA `expression`, e **o erro também a leva em
  `details`**: sem isso a UI não sabe qual watch falhou e marcaria todos.
  Expressão vazia ou só de espaços é `INVALID_PARAMS` no core, antes de chegar
  ao adapter — a mensagem do lldb para string vazia não ajuda ninguém.

**Mudança de contrato em `debug.setBreakpoints` (protocolo `0.66.0`).** O campo
`lines: [u32]` virou `breakpoints: [{ line, condition?, hitCondition? }]`.

```text
ANTES  { file, lines: [3, 7] }
AGORA  { file, breakpoints: [{ "line": 3 },
                             { "line": 7, "condition": "i == 42" }] }
```

`condition` e `hitCondition` mapeiam 1:1 nos campos de mesmo nome do
`SourceBreakpoint` do DAP — *"the breakpoint stops only when this evaluates to
true"* e *"how many times the breakpoint must be hit before stopping"* —, e o
`lldb-dap` anuncia `supportsConditionalBreakpoints` e
`supportsHitConditionalBreakpoints`. **Campo ausente não vira `null` no wire:**
mandar `"condition": null` (ou `"  "`) é pedir para um adapter tratar como
expressão vazia e o breakpoint nunca parar; o core descarta condição em branco
antes de montar o argumento. Array paralelo de condições foi recusado de
propósito — é a forma de linha e condição saírem de sincronia em silêncio.

```text
event.debug.started   { program }
event.debug.output    { category, line }   (stdout|stderr|console)
event.debug.stopped   { reason, file?, line?, threadId }  (file/line podem
                        vir null; o core ja enriquece com o frame do topo)
event.debug.continued {}                   (um por retomada, deduplicado)
event.debug.finished  { exitCode? }        (exatamente um por sessao)
```

### Git (`git.status` e operações diárias)

Implementado no protocolo `0.30.0` (fatia M3.1 de `DocsPrivate/diario/18`). Orquestra o
binario `git` com saida ESTAVEL (`status --porcelain=v2 --branch
--untracked-files=all -z`); deteccao de repo por exit code
(`rev-parse --is-inside-work-tree`), nunca por mensagem (pode vir
localizada). Sincrono e stateless: a UI decide quando consultar
(open/save/fs-ops/manual).

```text
git.status {} → { repo: bool,
                  branch?, detached?, shortSha?,
                  upstream?, ahead?, behind?,
                  entries: [{ path, kind, staged }] }
kind ∈ modified|added|deleted|renamed|untracked|conflicted (o core
consolida o par XY do porcelain; a UI nunca decodifica porcelain).
```

- Paths das entries são relativos ao ROOT do workspace (convertidos do
  toplevel via `rev-parse --show-prefix`); entries fora do workspace e os
  metadados `.kinein/` são filtrados.
- Workspace sem git → `{ repo: false, entries: [] }` (nunca é erro).
  Erros reais: `NO_WORKSPACE`, `TOOL_NOT_FOUND` (git ausente),
  `INTERNAL_ERROR` (git falhou).

`git.fileDiff { path }` (protocolo `0.31.0`, fatia M3.2) → diff do
arquivo contra o HEAD, em duas granularidades na mesma resposta:

```text
git.fileDiff { path } → { path (canônico, ecoado p/ correlação),
                          repo, tracked,
                          hunks: [{ kind: added|modified|removed,
                                    startLine, lineCount }],
                          text }
```

- `hunks` vem de `--unified=0` (ranges exatos para a gutter; `removed` é
  ancorado na linha seguinte à remoção, lineCount 1); `text` vem de
  `--unified=3` (visão de diff). A UI nunca parseia diff.
- Untracked → `tracked: false` + um hunk `added` do arquivo inteiro e
  `text` vazio. `path` confinado ao workspace (`INVALID_PARAMS` fora).
- Diff é do arquivo EM DISCO (buffer não salvo não aparece — a gutter
  atualiza no save/troca de aba; attach de buffer é melhoria futura).

Mutações (protocolo `0.32.0`, fatia M3.3) — todas respondem o MESMO
shape do `git.status` (a UI atualiza tudo de uma vez):

```text
git.stage    { paths: [abs] } → GitStatusResult   (git add)
git.unstage  { paths: [abs] } → GitStatusResult   (git restore --staged)
git.discard  { paths: [abs] } → GitStatusResult   (DESTRUTIVO: restore
                                 p/ tracked, clean -f p/ untracked; a
                                 confirmação é responsabilidade da UI)
git.commit   { message }      → GitStatusResult   (commita SÓ o staged)
```

- Guarda do commit por exit code estável (`git diff --cached --quiet`):
  nada staged → `INVALID_REQUEST` com mensagem própria. `message` vazia
  e `paths` vazio/fora do root → `INVALID_PARAMS` (path deletado não
  canonicaliza: vale a checagem lexical dentro do root). Mutação em
  workspace sem git → `INVALID_REQUEST` (leitura responde `repo:false`).
  Falha real do git → `INTERNAL_ERROR` com o stderr.

Leitura de histórico (protocolo `0.33.0`, fatia M3.4) — todas com
formato estável (`--porcelain`, `%x1f`/NUL) e sem parsear saída
localizada:

```text
git.blame { path } → { path (ecoado), repo, tracked, groups: [
    { startLine, lineCount, sha, author, authorTime (epoch),
      summary, committed }] }
git.log { maxCount? } → { repo, entries: [
    { sha, shortSha, author, authorTime (epoch), summary }] }
git.commitDiff { sha } → { sha (ecoado), text (patch unificado) }
```

- `git.blame` usa `git blame --porcelain`; metadado de sha repetido é
  memoizado (o porcelain só o manda uma vez). Linha não commitada tem
  sha zerado → `committed:false` (detecção pelo sha, nunca pela string
  "Not Committed Yet"). Untracked/repo recém-init sem HEAD →
  `tracked:false` com `groups` vazio (não é erro). `path` confinado ao
  root e existente em disco → senão `INVALID_PARAMS`.
- `git.log` usa `--pretty=format:%H%x1f%h%x1f%an%x1f%at%x1f%s -z`;
  `maxCount` default 100, fora de `1..=500` → `INVALID_PARAMS`. Repo
  sem commits (sem HEAD, checado por `rev-parse --verify --quiet`) →
  `entries: []`. Fora de repo → `repo:false`.
- `git.commitDiff` usa `git show --pretty=format: --unified=3`; o `sha`
  é validado (4..=64 hex) ANTES de virar argv (barreira contra string
  arbitrária); merge trivial pode devolver `text` vazio. Fora de repo →
  `INVALID_REQUEST` (a chamada só nasce da lista do `git.log`).

Operações diárias adicionadas no protocolo `0.49.0`:

```text
git.branches {} → { repo, current?, detached, branches: [{ name, current }] }
git.checkout { branch } → GitStatusResult
git.branchCreate { name, checkout? } → GitStatusResult
git.pull {} → { jobId }
git.push {} → { jobId }
git.stash { action: "push|pop", message? } → GitStatusResult
```

- nomes de branch são validados antes de virar argumento; checkout/create e
  stash são recusados pela UI quando existem buffers sujos;
- pull/push são jobs canceláveis e publicam saída/resultado pelo Job System, e
  o desfecho sai em **`event.git.remoteFinished { operation, success, message }`**
  (documentado em 2026-08-29; o evento existia desde o `0.49.0` e era o único
  dos 32 eventos do core que não estava neste contrato). A UI usa `operation`
  para saber se foi `pull` ou `push` sem guardar o `jobId`;
- stash push inclui untracked, limita-se ao workspace e exclui `.kinein`;
  pop restaura o stash mais recente;
- todas as mutações síncronas devolvem o status inteiro para não duplicar
  estado derivado na UI.

### Jobs (`job.list` / `job.cancel`)

Fundação do Job System (ver `DocsPublic/arquitetura/ARCHITECTURE.md` §7). Um **job** é uma
operação longa que o core executa de forma assíncrona: retorna um `id` na hora,
reporta progresso por eventos e pode ser cancelado. A infraestrutura atual
registra ciclo de vida/cancelamento e já é usada por `build.run`, `quality.run`
e `test.run`.

- `job.list` → `{ jobs: [{ id, kind, title, status, progress?, canCancel, risk }] }`.
  `status`: `queued|running|cancelRequested|success|warning|failed|cancelled`;
  `risk`: `low|medium|high|dangerous`. Ordem estável de criação. O core retém
  até 100 jobs em memória, preservando jobs ativos e removendo os finalizados
  mais antigos quando o limite é excedido.
- `job.cancel { jobId }` → `{ jobId, cancelled }`. `cancelled` é `true` só quando
  o job existe, expõe cancelamento e ainda está `running`; ao aceitar o pedido,
  o core muda o status para `cancelRequested` e emite `event.job.progress`.
  `jobId` ausente é `INVALID_PARAMS`. O cancelamento é cooperativo (o trabalho
  verifica o sinal) e o estado terminal chega depois como `cancelled`.

Eventos, todos com `jobId`:

```text
event.job.created   { "id", "kind", "title", "status", "progress"?, "canCancel", "risk" }
event.job.progress  { "jobId", "status": "running|cancelRequested", "progress"?, "message"? }
event.job.output    { "jobId", "line" }
event.job.finished  { "jobId", "status": "success|warning|failed|cancelled" }
```

Regra de UX (specs): `event.job.*` atualizam status bar / tool window; não abrem
pop-up automático. Job `high`/`dangerous` exige confirmação antes de iniciar.

## Os 140 métodos roteados — a lista inteira

> **Era "Métodos principais implementados", e listava 66 dos 128** — sem dizer
> que era parcial, o que fazia um domínio inteiro parecer inexistente.
> Refeita em 2026-09-06 pelo comando do cabeçalho, agrupada por domínio; os
> `sim.*` saíram dela em 2026-09-12 com o domínio.

```text
build.run
build.size

cargo.check
cargo.metadata

cmake.configure
cmake.presets.list
cmake.status
cmake.targets.list

command.list

configAction.apply
configAction.list
configAction.preview

container.action
container.compose
container.images
container.list
container.open
container.status

core.ping
core.shutdown

datasource.introspect
datasource.list
datasource.remove
datasource.save
datasource.test

debug.continue
debug.evaluate
debug.next
debug.pause
debug.setBreakpoints
debug.stackTrace
debug.start
debug.stepIn
debug.stepOut
debug.stop
debug.variables

draft.clear
draft.save

environment.scan

format.capabilities
format.text

fs.createDirectory
fs.createFile
fs.delete
fs.findFiles
fs.list
fs.read
fs.rename
fs.replace
fs.search
fs.write

git.blame
git.branchCreate
git.branches
git.checkout
git.commit
git.commitDiff
git.discard
git.fileDiff
git.log
git.pull
git.push
git.stage
git.stash
git.status
git.unstage

grafana.forget
grafana.get
grafana.probe

index.context
index.status
index.symbols
grafana.save

job.cancel
job.list

library.list
library.plan

lsp.applyCodeAction
lsp.codeActions
lsp.completion
lsp.definition
lsp.didChange
lsp.documentSymbols
lsp.hover
lsp.references
lsp.rename
lsp.restart
lsp.semanticTokens
lsp.switchSourceHeader
lsp.workspaceEdit.apply
lsp.workspaceEdit.cancel
lsp.workspaceSymbols

probe.list

project.model
python.createEnvironment
python.status

quality.run

run.capabilities
run.script
run.start
run.stdin
run.stop

runConfig.delete
runConfig.list
runConfig.save
runConfig.setActive

serial.list
serial.monitor

settings.get
settings.set

setup.list


syntaxTree.update

terminal.close
terminal.input
terminal.mouse
terminal.open
terminal.resize
terminal.scroll

test.discover
test.run

toolchain.get
toolchain.importKit
toolchain.inspectSysroot
toolchain.install
toolchain.installable
toolchain.set
toolchain.setKit

tools.detect
tools.status

workspace.browse
workspace.close
workspace.createFolder
workspace.createProject
workspace.open
workspace.recent.clear
workspace.recent.list
workspace.recent.pin
workspace.recent.remove
workspace.saveSession
workspace.status
```

## Os 48 eventos emitidos — a lista inteira

> **Era "Eventos iniciais", e faltavam três** — os dois do `datasource` e o do
> `grafana`. Refeita em 2026-09-06. **Cinco não são literais no código**: eles
> nascem de `format!("event.{domain}.*")` em `handlers/build.rs`, com `domain`
> valendo `build` ou `quality`, e por isso um grep ingênuo não os encontra.

```text
event.build.diagnostic              <- so por format!
event.build.finished
event.build.output                  <- so por format!
event.build.started                 <- so por format!

event.cmake.finished
event.cmake.started

event.datasource.introspected
event.datasource.tested

event.debug.continued
event.debug.finished
event.debug.output
event.debug.started
event.debug.stopped

event.environment.finished
event.environment.started
event.container.finished

event.environment.tool

event.fs.changed
event.fs.watchError

event.git.remoteFinished

event.grafana.probed

event.index.finished
event.index.progress

event.job.created
event.job.finished
event.job.output
event.job.progress

event.lsp.diagnostics
event.lsp.documentsClosed
event.lsp.restarted
event.lsp.status

event.project.changed
event.python.finished
event.quality.diagnostic
event.quality.finished
event.quality.output                <- so por format!
event.quality.started               <- so por format!

event.run.finished
event.run.output
event.run.started

event.terminal.closed
event.terminal.render

event.test.case
event.test.discovered
event.test.finished
event.test.output
event.test.started

event.toolchain.installed
```

## Regras

1. Todo request deve ter resposta.
2. Eventos não têm `id`.
3. Erros devem ter `code`, `message` e `details`.
4. O protocolo deve ser versionado.
5. Toda alteração no protocolo exige atualização de docs e schemas.
6. A UI não deve interpretar logs brutos quando houver evento estruturado.

## `library.*` — o catálogo curado de bibliotecas C/C++

Domínio novo no protocolo `0.68.0` (etapa 19 do `roadmaps/35`). **A IDE endossa
o que oferece**: cada entrada carrega licença verificada, versão pinada e a
frase que explica o que a biblioteca faz.

```text
library.list {}                  -> { libraries: [LibraryInfo] }
library.plan { id, target }      -> LibraryPlan
```

**Nenhum dos dois exige workspace aberto**, e o domínio é **stateless** — o
catálogo é estático e a disponibilidade se lê do filesystem. Nada depende do
estado do `Core`, e o handler não recebe `self`.

`status` é `detected` | `notDetected`. **`notDetected` não é "não existe":** a
detecção olha os diretórios de config package conhecidos, e o usuário pode ter
um `CMAKE_PREFIX_PATH` próprio. Por isso, quando nada foi achado, o
`LibraryPlan` devolve `searchedPaths` — *"não achei, e olhei aqui"* é acionável;
*"não achei"* manda adivinhar.

O `LibraryInfo` carrega `standardLineage` quando existe o sinal FORTE — a
biblioteca ter sido adotada no padrão ISO, ou passado pela revisão formal do
Boost. **Não há certificação oficial de biblioteca C++:** o WG21 padroniza a
linguagem e a biblioteca padrão, e a Standard C++ Foundation declara que seu
objetivo é reduzir barreiras para *adotar* bibliotecas no próprio Standard — não
certificar as de terceiros. O campo registra o sinal que existe de verdade em
vez de inventar um selo que ninguém emite; hoje só `fmt` o tem
(virou `std::format` no C++20).

**O plano não escreve nada.** Ele devolve passos que nomeiam a Configuration
Action que os executaria, e quem escreve continua sendo o domínio
`configaction`, com o preview e o consentimento que ele já tem. Dois escritores
do mesmo arquivo de build é a forma de eles divergirem em silêncio — e o
critério de aceite do corte é mecânico: `grep -c "CMakeLists"
crates/kinein-core/src/library/` tem que voltar **0**, inclusive em comentário.

**`find_package` ou `FetchContent` sai da MEDIÇÃO, não de um campo do
catálogo.** Se o config package está no sistema, usa-se ele; baixar e compilar o
que já está instalado é desperdício que o usuário paga em tempo de build. A
primeira versão tinha um enum com três variantes para isso e as sete entradas
auditadas saíram todas iguais — os lints reprovaram as variantes nunca
construídas, e estavam certos.

## `probe.*` — as sondas de debug conectadas

Domínio novo no protocolo `0.71.0` (etapa 24 do `roadmaps/35`). É o **"plug"**
do plug and play: a IDE detecta em vez de pedir configuração.

```text
probe.list {} -> { probes, toolAvailable, rawOutput, hint? }
```

Executa `<adaptador> list`, onde o adaptador vem do kit (papel `debugAdapter`).
**Exige workspace**, porque sem kit não há qual ferramenta perguntar — e cair no
`PATH` em silêncio esconderia do usuário quem respondeu.

**O `rawOutput` vai SEMPRE, não só no erro.** O formato do `probe-rs list` foi
levantado de fontes da comunidade e **não** verificado contra o binário
instalado (ele não está nesta máquina, medido em 2026-09-03). Um parser rígido
contra formato não verificado quebraria na primeira mudança de espaçamento — e
quebraria dizendo *"nenhuma sonda"*, que é a pior mentira possível aqui.

Por isso: linha que casa vira sonda, linha que não casa é **ignorada**, e a
saída crua volta junto. *"Não entendi o que a ferramenta respondeu, e aqui está
o que ela disse"* é acionável; *"nenhuma sonda"* com uma plugada viola o item 4
do plug and play (`roadmaps/35` §5.1).

**A `hint` é onde o item 4 vira código.** O caso mais importante é o de **udev**:

```text
permissao negada  -> "falta regra de udev; rodar a IDE como root NAO e a solucao"
resposta vazia    -> "sonda desconectada, ou cabo de dados trocado por um de carga"
nao reconhecida   -> "o formato mudou e o parser precisa acompanhar"
```

Sem regra de udev a ferramenta roda, não acha nada, e o usuário conclui que a
placa está com defeito. **Plug and play morre exatamente aí**, e é a lacuna que
`integracoes/36` §5 já tinha nomeado como a mais subestimada.
## `container.*` — Docker e Podman como domínio nativo

Domínio novo no protocolo `0.92.0` (2026-09-12). A decisão é de 2026-07-17
(`roadmaps/28` §0: *"vão ser cidadãos nativos"*), e as invariantes registradas
lá em §4 são o desenho: **a UI nunca chama `docker`**, **toda ação é Job
cancelável**, **permissão é visível**, e **Podman é motor de primeira** — nesta
máquina `docker` é o shim `podman-docker`, medido em 2026-09-12.

```text
container.status  {}                       -> ContainerStatus
container.list    { all? = true }          -> { containers, engine, rawOutput, hint? }
container.images  {}                       -> { images, engine, rawOutput, hint? }
container.action  { id, action }           -> { jobId }            (job; start|stop|restart|remove)
container.open    { id, mode }             -> { id, command }      (aba de terminal; logs|shell)
container.compose { action, file? }        -> { jobId }            (job; up|down; exige workspace
                                                                     E arquivo de compose, ou `file`)

ContainerStatus   engine? (docker|podman), binary?, version?, emulated, rootless?,
                  socket?, reachable, compose?, composeFile?, hint?, rawOutput
ContainerInfo     id, names[], image, state, status, ports[], created
ImageInfo         id, repository, tag, size, created

event.container.finished { jobId, action, target, ok, message }
```

**A detecção pergunta ao binário, não ao nome.** `docker --version` do shim
escreve *"Emulate Docker CLI using podman"* em stderr; tratar isso como Docker
Engine erraria o formato de `ps` (o Podman devolve um **array** em `--format
json`; o Docker, **um objeto por linha** em `--format '{{json .}}'`) e o
diagnóstico (não há daemon nem grupo `docker` num Podman rootless). Quando o
`docker` é o shim, o core prefere o `podman` real e diz `emulated: true`.

**O parser aceita as duas formas e devolve a saída crua sempre** — a mesma
regra do `probe.list`. A forma do Docker foi lida da documentação e **não**
verificada contra um Docker Engine real (esta máquina não tem um); a do Podman
5.8.4 foi medida, inclusive o detalhe de que `images --format json` não traz
`repository`/`tag` e sim `Names ["repo:tag"]`.

**`status` é a tela de "ativar a ferramenta".** Sem motor: o passo de
instalação da distro. Motor que não responde: `permission denied` no socket →
grupo `docker` e relogar; daemon parado → `systemctl enable --now docker`;
Podman → a saída crua e o `podman.socket` do usuário. A IDE **imprime** o
comando; nunca roda `sudo`.

**`action` e `compose` são jobs** (`JobRisk::Medium`; `remove` é `High`): cada
linha do motor vai para `event.job.output`, cancelar mata o filho, e o
`event.container.finished` leva as últimas linhas como motivo. `rm` vai **sem
`-f`**: remover o que roda é dois gestos (parar, remover), de propósito.
`compose up` é `-d` — um job que nunca termina não é job; a saída viva mora
na aba de logs.

**O compose é do PROJETO, e o core diz se ele existe (`0.108.0`,
2026-09-13).** A ferramenta é da máquina (`compose`); o arquivo é do workspace
aberto: `composeFile` é o primeiro dos nomes que a ferramenta procura sozinha
na raiz (`compose.yaml`, `compose.yml`, `podman-compose.*`,
`docker-compose.yml|yaml`, `container-compose.*` — a ordem do podman-compose
1.6.0, lida em `COMPOSE_DEFAULT_LS`; os `*.override.*` não contam porque
sozinhos não sobem nada), ausente sem workspace ou quando o projeto não tem
nenhum. `container.compose` sem `file` num projeto sem arquivo **recusa com
`INVALID_PARAMS` antes de subir um job** — medido: o podman-compose sai com
255 e *"no compose.yaml, docker-compose.yml or container-compose.yml file
found"*, e um job que só pode falhar não é job. Na UI, "compose up"/"compose
down" só acendem com motor respondendo + ferramenta + arquivo, e a linha do
motor diz o que falta ("abra um projeto" / "o projeto não tem compose.yaml").
A mesma tarde corrigiu a classe visual que escondia isso: um botão primário
DESLIGADO vestia o âmbar a 72% de opacidade e continuava a coisa mais chamativa
do painel — o clique que não fazia nada lia-se como "o Docker não funciona";
agora desligado é um botão comum e apagado (`KvButton`/`KvIconButton`,
provado por `tst_kvbutton_states.qml` e `tst_container_panel.qml`).

**`open` reaproveita o terminal.** `logs -f --tail 200 <id>` e `exec -it <id>
/bin/sh` sobem pelo mesmo `open_command` do domínio `terminal` e voltam como
uma sessão de terminal (`terminal.input`/`resize`/`close` a reconhecem). O
shell dentro do container é a semente do *contexto remoto* do `roadmaps/28`
§4 — o que falta para *dev containers* é o path mapping e o ciclo de vida,
não o transporte.

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
event.python.finished { jobId, success, tool, command, path }

PythonStatus   interpreter? (PythonEnv: interpreter, version?, origin, warning?),
               nativeModule? (PythonNativeModule: kind, tool, evidence[], buildHint;
               0.102.0),
               hasEnvironment (origin != sistema), environmentTool? (uv|venv:
               o que a IDE usaria), uv? (caminho), projectFiles[] (pyproject.toml,
               requirements.txt, setup.py, uv.lock, poetry.lock, Pipfile),
               hint? (o que falta, com o remedio)
```

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

## `index.*` — o índice do projeto inteiro

Domínio novo no protocolo `0.94.0` (pilar 0 do `roadmaps/42`, 2026-09-12).
**Decisão do autor:** *"a IDE deve ler o projeto inteiro que for aberto… todas
as funções/arquivos/pastas"* — para Python, C, C++ e Rust.

```text
index.status  {}                        -> IndexStats   (responde tambem sem workspace: idle)
index.symbols { query, limit?, kind? }  -> { symbols[], total, state }   (exige workspace)
index.context { path }                  -> FileContext  (exige workspace; 0.95.0)

event.index.progress { files, symbols }     a cada ~200 arquivos
event.index.finished IndexStats             ao fim do build, a cada incremento e
                                            a cada recarga do contexto

IndexStats   state (idle|building|ready|failed), folders, files, sourceFiles,
             lines, bytes, symbols, functions, types, byLanguage[] { language,
             files, lines, symbols }, skipped[], elapsedMs, error?, context?
ContextSummary cdbDirectory?, cdbEntries, cdbStale, cdbStaleBecause?,
             cargoPackages, cargoTargets, cmakeTargets (0.96.0),
             pythonInterpreter?, pythonOrigin?
IndexSymbol  name, kind, path (relativo a raiz), language, line, endLine,
             container?
FileContext  path (absoluto), language (c|cpp|rust|python|other), unit?, crate?,
             python?, targets[] (0.96.0: os targets do CMake que compilam ou
             listam o arquivo), source?, hint?
CompileUnit  compiler, directory, standard?, includes[] (absolutos), defines[],
             output?, arguments[]
CargoUnit    package, target, kind (lib|bin|test|bench|example|custom-build…),
             edition, manifest, srcPath, features[]
PythonEnv    interpreter, version?, origin (VIRTUAL_ENV|.venv|venv|env|poetry|
             sistema), warning?
```

**O contexto de compilador (0.95.0).** O índice diz *o que há* em cada
arquivo; o contexto diz *com que* ele é compilado — e é carregado no mesmo
job, logo depois dos arquivos:

- **C/C++:** a `compile_commands.json` que o `cdb::status` encontra
  (`.kinein/build`, raiz, `build`, `builddir`, `out/build`), nas **duas
  formas** do padrão do clang (`arguments[]` e `command` dividido como um
  shell: aspas agrupam, `\` escapa); `file` relativo é resolvido contra o
  `directory` e a chave é o caminho **real** (`canonicalize`), porque a CDB do
  Ninja/Meson escreve `../src/a.cpp` e o editor pergunta pelo absoluto.
  `-I`/`-isystem`/`-iquote` colados ou separados viram `includes` absolutos;
  `-D` viram `defines`; `-std=` vira `standard`; `output` vence `-o`. O resto
  segue inteiro em `arguments`. Cabeçalho não tem unidade própria e a `hint`
  diz isso (o clangd deduz pela unidade que o inclui); fonte fora da CDB diz
  "nenhum alvo o compila" — ou, se a CDB envelheceu, qual arquivo a envelheceu.
- **A CDB envelhecida por SUBPASTA.** Medido em 2026-09-12 neste repositório:
  a CDB de 04/09 sem cinco fontes que o `ui/CMakeLists.txt` de 12/09
  acrescentou, e o `cdb::status` dizendo "não envelheceu" porque só compara
  com os arquivos de build da **raiz**. O contexto, com as unidades em mãos,
  sobe de cada diretório de fonte até a raiz procurando um `CMakeLists.txt`
  mais novo que a CDB; acha → `cdbStale` com `cdbStaleBecause`
  (`ui/CMakeLists.txt`) e a `hint` da unidade pede reconfigure. A subida
  **para na raiz do workspace**.
- **Rust:** `cargo metadata --format-version 1 --no-deps` com o `cargo` do kit
  efetivo (se houver `Cargo.toml` na raiz e o cargo existir — nos testes o
  PATH é vazio e nada roda). O arquivo pertence ao alvo pelo `src_path`
  **exato**; senão pelo alvo cujo diretório de `src_path` é o prefixo **mais
  longo** do arquivo; `lib` vence o empate (um `src/x.rs` pertence ao lib e
  ao bin do mesmo pacote, como o rust-analyzer prefere). O `build.rs`
  (`custom-build`) só casa exato: mora na raiz do pacote e, por prefixo, seria
  dono de tudo.
- **Python:** a precedência do `roadmaps/29` §4.1 — `$VIRTUAL_ENV` (só se o
  interpretador existir), `.venv/`, `venv/`, `env/`, `poetry.lock` + `poetry
  env info -p`, e por último o `python3` do PATH **com aviso** (instalar
  pacote nele quebra a distro). `version` é o `--version` do interpretador
  achado.
- **O modelo por alvo do `CMake` (0.96.0):** o `codemodel-v2` do build dir da
  IDE (`.kinein/build`) entra no mesmo contexto. Todo arquivo C/C++ ganha
  `targets` — cabeçalho incluso, porque o target o *lista* mesmo sem o
  compilar. E quando **não há** `compile_commands.json`, a unidade vem do
  grupo de compilação do target (includes, defines, `compileCommandFragments`
  inteiros em `arguments`, `-std=` do fragmento antes do `languageStandard`)
  com o compilador da `toolchains-v1` — ou `"(CXX do kit)"` quando ela não
  foi respondida (CMake < 3.20 ou query antiga), dito em vez de inventado;
  `source` diz `file-api codemodel-v2 (target X), sem compile_commands.json`.
  Com a CDB presente, a CDB vence a unidade e os `targets` ficam.
- **Quando recarrega:** `event.cmake.finished` (o configure reescreve a CDB) e
  um `Cargo.toml` em `event.fs.changed` (um alvo novo muda a que pacote cada
  arquivo pertence) recarregam **só o contexto**, num job curto; os arquivos
  ficam. O `event.index.finished` sai de novo com o `context` novo, e a UI
  pergunta de novo pelo arquivo ativo.

Medido neste repositório pelo core real (2026-09-12): 288 unidades na CDB de
`.kinein/build`, 4 pacotes e 6 alvos do cargo, Python do sistema
(`/usr/bin/python3`, 3.14.7). `ui/src/core_client_index.cpp` → `c++`,
`gnu++23`, 12 `-I`, 9 `-D`; `crates/kinein-core/src/index/context.rs` →
`kinein-core`/`kinein_core` (lib, 2024); `crates/kinein-core/src/main.rs` →
`kinein-core` (bin).

**Na UI:** a barra de status mostra o contexto do arquivo ativo —
`contexto: c++ · gnu++23 · 12 -I · 9 -D`, `cargo · kinein-core (lib, 2024)`,
`python · sistema · 3.14.7 ⚠` — e o detalhe (diretório, origem, dica) ao
pairar. Resposta atrasada de outro arquivo é descartada; trocar de arquivo
limpa até a resposta chegar.

**O que ele lê, e como.** Ao abrir o workspace, um job caminha a árvore
inteira com a **mesma lista de pastas ignoradas do watcher** (`.git`,
`.kinein`, `target`, `build`, `node_modules`…, para os dois verem o mesmo
projeto), conta **todo** arquivo, lê os de fonte (C/C++/Rust/Python) e extrai
as declarações com as **gramáticas Tree-sitter do editor** (a mesma query
`tags` oficial de cada gramática, sem cache — `lang/extract.rs`). Python
entrou na fundação em 2026-09-12 à tarde (`tree-sitter-python` 0.25.0, MIT;
bloco B do `roadmaps/41`): a `tags` oficial dá `function` e `class` (o método
vem como `function` com `container`). Arquivo acima de 4 MiB ou ilegível é
contado e **dito** em `skipped`, nunca sumido. Medido em 2026-09-12 à tarde
neste repositório: 1.022 arquivos, 148 pastas, 72.000 linhas, 3.939
declarações (217 delas em 18 `.py`) em ~2,1 s (build de depuração). O
"4.658" da manhã foi medido antes de o dedup do `fn` em `impl` entrar no
mesmo commit — remedido sem Python, o mesmo código dá 3.720.

**O incremento — e o índice segue o disco INTEIRO (2026-09-12 à tarde).**
Os caminhos de `event.fs.changed` são reindexados no loop principal (um
arquivo é milissegundos) e os totais reemitidos. Uma **pasta nova** é
caminhada inteira (com a mesma lista de pastas ignoradas); uma pasta apagada
leva os arquivos e as subpastas dela. E o que fecha o buraco que existia até
então: a cada `event.index.finished` o Core **registra no watcher todas as
pastas que o índice caminhou** — uma a uma, `NonRecursive`, como o ADR-0001
manda —, então um arquivo criado pelo terminal numa pasta que nenhuma tela
listou chega ao índice pelo mesmo caminho. Antes, só as pastas que a UI
expandiu eram observadas e o arquivo ficava fora até o próximo
`workspace.open`, em silêncio. Medido neste repositório: **148 inotify
watches** (as 148 pastas), contra o limite de 186.243 desta máquina
(`fs.inotify.max_user_watches`); num projeto grande o registro **para no
primeiro erro** e o relata uma vez (`event.fs.watchError`). Provado pelo
core real na exercitação: um arquivo nascido em `src/tarde/` depois do índice
pronto aparece no `index.symbols` sem reabrir nada.

**Sem duplicata e sem renomear a gramática.** A `tags` do Rust captura um `fn`
dentro de `impl` duas vezes (`function` e `method`); o índice fica com a mais
específica por (linha, nome). O que a gramática chama de `class` (a `struct`
do Rust) o índice **não** renomeia.

**A busca** ordena exato > prefixo > substring, sem diferenciar caixa, com
filtro por `kind` e `total` antes do limite. Na UI, `#nome` no Search
Everywhere pede ao índice **e** ao LSP: o índice responde primeiro e **sem
arquivo aberto**; o LSP, quando responde, substitui.

**O que este domínio NÃO é:** semântica (tipos, referências, rename continuam
no clangd/rust-analyzer). O contexto de compilador por arquivo entrou no
`0.95.0` (acima); o que ainda falta no pilar 0 está no `roadmaps/42` §3 P0.

## `project.*` — o modelo do projeto embarcado

Domínio novo no protocolo `0.93.0` (pilar 0 do `roadmaps/42`, 2026-09-12).
`workspace.open` diz *que build system* a raiz tem; este domínio diz *o que o
projeto é*.

```text
project.model {} -> ProjectModel        (exige workspace)
event.project.changed  ProjectModel     (ao abrir o workspace; ao fim de
                                         event.cmake.finished e event.build.finished)

ProjectModel   root, embedded, frameworks[], sdks[], artifacts, target, hints[]
FrameworkInfo  framework (espIdf|zephyr|picoSdk|platformIo|stm32Cube|cargoEmbedded|
               microPython|yocto|buildroot), evidence (caminho relativo do
               marcador), detail? (IDF_TARGET, PICO_BOARD, DeviceId do Cube,
               triple do cargo, MACHINE do Yocto, ambientes do PlatformIO)
SdkRequirement id, label, env?, path?, found, hint?
ProjectArtifacts elf[], bin[], hex[], uf2[], map[], flasherArgs?, partitionTable?,
               memoryX?, linkerScripts[], flashRecipe?, partitions?
FlashRecipe    chip?, flashMode?, flashSize?, flashSizeBytes?, flashFreq?, before?,
               after?, stub, files[] { offset, file (absoluto), name?, encrypted }
               — LIDO do flasher_args.json (forma do template
               components/esptool_py/flasher_args.json.in do ESP-IDF)
PartitionTable tableOffset, entries[] { name, kind, subtype, offset, size, flags? },
               end, unreadable[] — LIDA do partitions.csv com os offsets em
               branco resolvidos como o gen_esp32part.py (4 KB; app a 64 KB)
TargetModel    chip?, family?, triple?, flashEngine?, monitor?, debugAdapter?,
               evidence[]  — uma linha por dedução
```

**A detecção desce até 3 níveis e LÊ o marcador**: um `CMakeLists.txt` é
ESP-IDF se inclui o `project.cmake` do `IDF_PATH`, Zephyr se faz
`find_package(Zephyr)`, pico-sdk se chama `pico_sdk_init()`; um `main.py` é
MicroPython se importa `machine`/`board`. Pastas de saída (`build/`, `target/`,
`.pio/`, `.kinein/`) não contam. Entre dois achados do mesmo framework vence o
marcador que **decide** (`.cargo/config.toml` com o triple, não um `memory.x`
solto) e, empatando, o mais raso. Um `CMakeLists.txt` comum **não** é
embarcado; um `main.py` que não importa hardware **não** é MicroPython.

**O alvo tem evidência ou não tem alvo.** O chip do **kit** vence o do
framework (é a palavra do usuário); a família vem do chip ou do triple; os
motores vêm da família — e o ESP32 clássico (sem USB-JTAG) recebe `debugAdapter`
**ausente** com a evidência dizendo que depurar exige ESP-Prog, em vez de um
`probe-rs` que não funcionaria.

**"Achado" só vem de variável, pasta padrão ou binário.** `IDF_PATH` definido
mas apontando para pasta inexistente é `found: false`; a toolchain xtensa fora
do `PATH` mas em `~/.espressif/tools` é `found: true` com o caminho. O alvo
rustup não é medido aqui (é processo) e diz isso no `hint`. Nenhum comando roda.

**O que era só localizado passou a ser LIDO** (segunda fatia, 2026-09-12): a
receita de gravação com os caminhos resolvidos contra o `build/`, e a tabela
de partições — porque a "flash" de um ESP32 não é o `.ld`, é a partição
`app`, e é ela que o tamanho e o "Gravar" vão consumir. Linha de CSV que não
se lê vai em `unreadable`, nunca some.

**Fixtures reais e mínimas** de cada framework moram em
`scripts/fixtures/projetos/`; são elas que os testes leem.

## `serial.*` — as portas seriais USB

Domínio novo no protocolo `0.91.0` (E1 do `integracoes/38` §6, 2026-09-11). A
porta serial é o **canal que toda família bare metal compartilha** — bootloader
de ROM, console e as linhas DTR/RTS de reset — e por isso é a fundação do
monitor UART e do "Gravar".

```text
serial.list    {}                   -> { ports: [SerialPortInfo], hint? }
serial.monitor { device, baud? }    -> { id, command, tool }   (aba de terminal; 0.92.0)

SerialPortInfo  device, byId?, kind (usbUartBridge|usbCdc), vid, pid,
                manufacturer?, product?, serial?, interface?, driver?, family?,
                access { readableWritable, mode, group?, hint? },
                modemManager? { candidate, ignored, running }
```

**Não exige workspace**, ao contrário do `probe.list`: não há ferramenta vinda
do kit — é o sysfs desta máquina, o mesmo com ou sem projeto aberto.

**Nunca abre a porta.** Abrir um tty aciona DTR/RTS na maioria das pontes
(CP210x, CH340, FTDI) e isso **reseta a placa**; um `serial.list` que
resetasse o firmware a cada abertura do painel seria um defeito. A permissão é
medida com `access(2)`, que honra ACL — é assim que `TAG+="uaccess"` dá acesso
ao usuário da sessão sem grupo nenhum; um `stat` sozinho mentiria nesse caso.

**De onde vem cada campo:** VID:PID, `manufacturer`/`product`/`serial` e
`bInterfaceNumber` subindo de `/sys/class/tty/<n>/device` até o diretório USB
com `idVendor` (ABI documentada do kernel); `driver` do link `device/driver`;
`byId` de `/dev/serial/by-id`; `modemManager` de `udevadm info -q property`
(`ID_MM_CANDIDATE`, `ID_MM_DEVICE_IGNORE`) mais `/proc/*/comm` — e é `null`,
não `false`, quando o `udevadm` não existe. Medido em 2026-09-11: o
ModemManager examinou o ESP32 4 s depois do plug.

**`family` fala do ELO, nunca do chip.** `10c4:ea60` é *"ponte USB-UART
CP210x — o chip do outro lado não se lê pelo USB"*; `303a:1001` é *"Espressif
USB Serial/JTAG — o próprio chip"*. A identidade do chip vem **pelo canal**
(`esptool chip-id`, `probe-rs info`), que é a fatia E5. Fontes da tabela no
`integracoes/38` §5.

**`serial.monitor` é o monitor como PROCESSO numa aba de terminal** (E3 do
`integracoes/38` §6, decisão do autor em 2026-09-11: nunca código serial
nosso). A ferramenta vem do papel novo do kit, `serialMonitor` — candidatos
`tio`, `picocom`, `minicom`, `espflash`, `mpremote` nessa ordem —, com duas
regras a mais que o catálogo não conhece: **projeto MicroPython e `mpremote`
detectado → `mpremote connect <dev> repl`** (`0.102.0`: num firmware
MicroPython o monitor É o REPL — um tio a 115200 mostraria o mesmo texto sem
o raw-paste nem o Ctrl-] de sair; o mpremote é o último do catálogo e nunca
vence sozinho fora de MicroPython); chip Espressif no kit **e** `espflash`
detectado → `espflash monitor --elf <ELF>`, que decodifica o backtrace;
escolha **fixada** pelo autor vence tudo. Linhas de comando lidas na fonte: `tio -b`,
`picocom -b`, `minicom -D … -b`, e no espflash é `--monitor-baud` (o `--baud`
dele é o de **gravação**). Baud ausente = 115200. Sem nenhum monitor
instalado, `TOOL_NOT_FOUND` com o que instalar. Exige workspace (a aba nasce
no cwd do projeto) e volta como sessão de terminal, com o comando no título.

**A `hint` de acesso é o passo oficial, nunca um `sudo` que a IDE rodaria:**
*"`/dev/ttyUSB0` é `crw-rw----` do grupo `dialout` e você não está nele …
`sudo usermod -aG dialout $USER` e sair/entrar da sessão. A IDE não roda isso."*

## `command.*` — o catálogo de comandos que a UI mostra

Um método, e ele é a fonte única de **tudo que a IDE oferece por nome**: paleta,
menus, botões e atalhos leem daqui.

```text
command.list {}   ->  { commands: [CommandDescriptor] }
```

```json
{
  "id": "editor.save",
  "title": "Salvar arquivo",
  "category": "Editor",
  "description": "Grava o buffer atual no disco",
  "defaultShortcut": "Ctrl+S",
  "requiresWorkspace": true
}
```

**Não exige workspace aberto** — a paleta existe antes de haver projeto, e é o
`requiresWorkspace` de cada descritor que diz o que fica desabilitado.

**São 76 descritores, medidos em 2026-09-06**, em cinco grupos que são cinco
arquivos em `crates/kinein-core/src/commands/`:

```text
ide.rs      24    editor.rs   19    build.rs    16    git.rs       9    run.rs    8
```

**Por que este domínio existe em vez de a UI ter a lista:** porque o atalho que a
paleta **anuncia** tem de ser o que a IDE **obedece**, e isso é gate desde
2026-09-03 (`scripts/verificar-atalhos.sh`). Com a lista no core, o gate compara
uma fonte com o host; com a lista na UI, ele compararia a UI consigo mesma.

## `setup.*` — o passo a passo oficial de instalação, por distro

```text
setup.list {}  ->  { distroId, distroName, family, tools: [SetupToolInfo] }
```

```text
SetupToolInfo   id · name · summary · website · installed · guide?
SetupGuide      family · sourceUrl · checkedAt · steps: [SetupStep]
SetupStep       explanation · command
```

**Não exige workspace**: instalar o PostgreSQL não depende de projeto aberto, e
quem está começando abre a IDE antes de ter projeto — que é exatamente quando
este guia serve.

**A regra que governa o domínio inteiro: sem fonte oficial, a IDE não afirma.**
Cada `SetupGuide` carrega `sourceUrl` e `checkedAt`, e uma família de distro sem
fonte oficial **não recebe guia** — o `guide` volta `None` e a tela mostra o site
do projeto dizendo que não tem passo a passo. É a decisão registrada em
`../roadmaps/40` §5, e é por isso que Arch e openSUSE continuam sem passos: a
fonte dos três projetos não cobre essas famílias.

O `installed` vem do `ToolDetector` — detectar é capacidade, e a política de o
que fazer com a detecção fica na UI (`27-modulos-por-dominio.md` §6).

## `datasource.*` — os perfis de banco, e a senha que não mora em disco

Domínio da etapa 26/27 (`../roadmaps/35` §9). Cinco métodos, dois eventos.

```text
datasource.list       {}                      -> { profiles: [DataSourceProfile] }
datasource.save       { profile }             -> DataSourceWriteResult
datasource.remove     { name }                -> DataSourceWriteResult
datasource.test       { name, password? }     -> DataSourceTestAccepted   (job)
datasource.introspect { name, password? }     -> aceite + job
```

**Os quatro últimos exigem workspace aberto**; o perfil mora no projeto.

```text
event.datasource.tested        { jobId, ok, message, ... }
event.datasource.introspected  { jobId, schemas | collections, ... }
```

**A senha nunca entra no perfil.** O `DataSourceProfile` guarda motor, host,
porta, banco, usuário e um `SecretSource` — *de onde* o segredo vem —, e o
`password` viaja só no parâmetro do método que precisa dele, por chamada. É a
decisão de `../seguranca/40`, e a UI abre o diálogo de senha por
`secretRequired` no erro, **nunca casando texto de mensagem**.

**Duas formas de resultado, porque há dois tipos de banco.** O relacional
devolve `schemas → tables → columns` lido do `information_schema`; o MongoDB
devolve `collections → fields` com profundidade, tipo **plural** e presença em
%, e a origem do esquema declarada:

```text
DECLARADO   veio do validador `$jsonSchema` da colecao
INFERIDO    veio de `$sample` sobre a colecao
```

**A tela nunca deixa os dois parecidos**, e o custo da leitura vai junto:
quantos documentos foram lidos, e se a amostragem obrigou o servidor a varrer a
coleção inteira (`$sample` varre tudo quando N não é menor que 5% dela). Tetos
de RAM — 2.000 campos, 8 níveis, 10 elementos de array — aparecem como aviso na
coleção em vez de a deixarem com cara de completa.

## `grafana.*` — a observabilidade pela HTTP API, e só

Quatro métodos, um evento.

```text
grafana.get    {}          -> { profile: GrafanaProfile | null }
grafana.save   { profile }
grafana.forget {}
grafana.probe  { token? }  -> GrafanaProbeAccepted   (job)
```

```text
event.grafana.probed  { jobId, datasources, dashboards, matches, ... }
```

**Os quatro exigem workspace aberto.**

**O Grafana nunca é embutido** — licença AGPL, decisão registrada em
`../roadmaps/40` §5. A integração é HTTP, o cliente é o `ureq`, e os dashboards
**abrem no navegador do sistema**. O que justifica o domínio existir é o
`GrafanaMatch`: o cruzamento entre o datasource do Grafana e o perfil de banco
do projeto, que é a pergunta que nenhuma das duas ferramentas responde sozinha.

O token segue a mesma regra da senha: `GrafanaTokenSource` diz de onde ele vem,
e o valor viaja por chamada.

## `core.*` — o handshake e o encerramento

Dois métodos, e eles são os únicos que **nunca** dependem de nada: não exigem
workspace, não tocam o filesystem e respondem mesmo com o resto do core inerte.

```text
core.ping     {}  ->  { protocolVersion, ... }
core.shutdown {}  ->  { message: "shutdown requested" }
```

**O `core.ping` é como a UI descobre com que protocolo está falando** — é o par
do `PROTOCOL_VERSION` em `crates/kinein-protocol/src/lib.rs`, e é o primeiro
pedido que a UI faz depois de subir o processo (`04-boot-e-comunicacao.md`).

**O `core.shutdown` pede, não mata.** Ele responde e deixa o encerramento
acontecer com o drain dos jobs em andamento, que é o que impede um build a meio
caminho de virar processo órfão.
