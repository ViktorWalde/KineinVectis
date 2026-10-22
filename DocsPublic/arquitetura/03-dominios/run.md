# `run.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

### Execução (`run.start` / `run.script` / `run.stop`)

**0.187.0 — execução nativa de gravação (contrato anterior ao produtor):**
`run.start {flash?}` aceita a intenção de runConfig.flashProposal, exclusiva com
command/device; sem parâmetros, a configuração ativa pode conter essa intenção.
Nesse caso, device recebido só é aceito se sua identidade coincide com a porta
vinculada; a seleção visual não redireciona uma configuração salva silenciosamente.
O core regenera o plano, compara programa/argv e snapshot e revalida contexto,
artefatos e identidade da porta. Não interpreta shell nem executa argv arbitrário.
Usa o mesmo PTY/guardião serial; conflito continua SERIAL_PORT_BUSY.
Preparação é adiada e cancelável: run/script concorrente recusa, run.stop cancela
inclusive antes de existir terminal, e qualquer transição de workspace invalida
a continuação. Alterar a ativa durante preparação automática também recusa.
Falha/cancelamento não iniciam processo de gravação nem caem no modo manual.
Seleções/shape em runConfig; desenho no 52 §14.30. Desde a 0.4.1h4b2b3,
header atual permite Parar durante preparação, com resposta correlacionada;
o aceite da parada não conclui a espera antes da recusa ou saída do PTY.
Detalhe da ponte local C++/QML no arquitetura/36 §12, aceites no 52 §14.31.
Outros motores e ações de ocupação seguem pendentes; a jornada maior não fecha aqui.

**0.188.0 — intenção probe-rs:** a mesma regeneração antes do PTY vale para
probe-rs. A reserva usa o registro existente com o canal da sonda (seletor
canônico): conflita com a mesma sonda e com a seleção automática de porta, e
SERIAL_PORT_BUSY descreve a sonda. Um device recebido junto da configuração
ativa não é destino de probe-rs e não altera a intenção. As variáveis PROBE_RS_*
do ambiente do core não chegam ao processo lançado. O diretório corrente
continua a raiz, por isso um `.probe-rs.*` ali recusa já na prévia. 52 §14.34.

Implementado no protocolo `0.12.0`; **desde `0.125.0` (2026-09-18) a
execução é uma SESSÃO DE TERMINAL** — decisão do autor no primeiro teste
da IDE polida: "já temos o terminal integrado, não precisamos de mais nada
para executar". `run.start`/`run.script` continuam a resolver O QUE rodar
(a lógica abaixo é a mesma) e abrem o comando num PTY na raiz do
workspace (`sh -lc <command>` para o `start`; argv direto para o
`script`), como o `container.open` e o `serial.monitor` já faziam. A
resposta ganha `terminalId`; a saída chega por `event.terminal.render`
dessa sessão e o fim por `event.terminal.closed { id, exitCode }` — stdin
é `terminal.input`, cores e programas de tela cheia funcionam. Saíram
`run.stdin` e os três `event.run.*` (não há mais processo por pipes).
Requer workspace aberto.

- `run.capabilities {}` → `{ runnable: [ext], debuggable: [ext] }`
  (`0.107.0`). **Não** requer workspace: é o mapa estático do que
  `run.script` aceita (`sh`, `bash`, `zsh`, `py`) e do que `debug.start
  { program }` roteia para um adaptador de linguagem sem o kit (`py`),
  derivado da mesma tabela que `script_interpreter` usa para decidir. A UI
  pede uma vez por conexão; antes do catálogo chegar, nada é executável —
  uma lista escrita à mão em QML divergiu da do core por construção (o
  defeito que o `format.capabilities` corrigiu em 0.61.0 voltou a aparecer
  em dois arquivos da árvore, e este método o fecha).
- `run.start { command?, device? }` → `{ command, terminalId }`. Sem `command`, o core deriva o
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
  orientando digitar o comando). **`device?` (`0.110.0`)** é a porta serial
  que a tela escolheu para o lançador padrão de um projeto MicroPython
  (`mpremote connect <device> run 'main.py'`); é parâmetro do LANÇADOR
  PADRÃO e por isso é exclusivo com `command` (`INVALID_PARAMS` quando vêm
  os dois — um comando explícito roda como foi escrito e não tem onde
  receber a porta). Uma configuração de execução ATIVA é um comando salvo
  pelo autor: vence o lançador padrão e tampouco recebe a porta; o `command`
  ecoado diz o que rodou. Fora de MicroPython o campo não tem efeito. Vazio
  ou só espaço é `INVALID_PARAMS` (ver `run.script`).
- `run.script { path, device? }` → `{ command, terminalId }` (protocolo `0.55.0`).
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
  `device?` é a porta que a tela escolheu (campo ausente ≠ vazio: desde
  `0.110.0` um `device` presente e vazio, só espaço ou com caractere de
  controle é `INVALID_PARAMS` dizendo para omitir o campo — antes virava
  `mpremote connect '' run`, que falhava longe de quem errou). Sem
  mpremote detectado, `INVALID_REQUEST` dizendo `pipx install mpremote` —
  nunca o Python do desktop, que não tem os pinos de `import machine`. O
  mesmo vale para o `run.start` sem comando (`main.py` na placa, mesmo num
  workspace sem `pyproject.toml`); um pacote com `__main__.py` não roda com
  `-m` na placa e o erro o diz. Extensão inválida é `INVALID_PARAMS`.
- `run.stop {}` → `{ status: "ok" }`. Fecha a sessão da ÚLTIMA execução
  aberta; sem uma, `INVALID_REQUEST`. (Fechar a aba pela UI é
  `terminal.close`, como qualquer outra.)
- Fechar o workspace fecha todas as sessões, a da execução inclusa.

A UI (`RuntimeController`) dá à aba o nome do comando (`▶ cargo run`) e,
quando a sessão fecha, **mantém a aba** com o desfecho no nome (`✓` ou
`✗ <código>`) para o autor ler a saída; fechá-la depois é só local.

**Reserva MicroPython (0.184.0):** nos dois métodos, o lançador Mpremote
reserva o canal até a saída real do PTY. device explícito revalida existência,
acesso e identidade sem abrir tty; seleção automática mantém a escolha do
mpremote e reserva todos os canais gerenciados enquanto não se conhece o alvo.
Conflito é SERIAL_PORT_BUSY; details {scope: device|automatic, device: string|null,
owner: objeto|null}, com owner.automatic indicando seleção ampla. Pare a execução
ou feche sua aba para liberar, após a saída observada. A proteção inclui outra
instância atualizada e ferramenta sobrevivendo ao crash do core. Comando
explícito/configuração manual/host Python não reservam a seleção visual; não
se analisa shell para inferir recursos. Gravação nativa vem na próxima parte.

Na resolução padrão MicroPython, entrada de pacote `-m` ou executável de
`project.scripts` instalado no host é recusada com orientação para executar
um arquivo `.py`; a intenção de placa não cai silenciosamente no CPython.

**0.185.0:** sem command, erro ao ler configurações salvas é INVALID_REQUEST
com caminho e orientação, sem fallback automático. Comando explícito não
precisa ler o arquivo de configurações e permanece disponível para recuperação.
