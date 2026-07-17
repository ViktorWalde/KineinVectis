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

## Leitura obrigatória: `docs/arquitetura/ARCHITECTURE.md`

**Antes de propor arquitetura, split, módulo, pasta ou "plano" de organização,
leia `docs/arquitetura/ARCHITECTURE.md` inteiro.** Ele é contrato de engenharia,
não referência de consulta: define camadas (§2), organização do core (§4), onde
colocar código novo (§5) e o caminho de crescimento `função → arquivo → pasta →
crate` (§6), com os nomes dos crates futuros já escolhidos.

**Por que isto está no topo e não numa lista.** Já estava escrito "obrigatório",
e não segurou. Medição de 2026-07-16: 7 arquivos do core e 20 da UI violavam as
regras desse documento; `lib.rs` voltou a 505 linhas e `terminal.rs` chegou a
955, num documento que existe **explicitamente** para impedir a volta do
monólito. A §6 chegou a afirmar "Main.qml tem 336 linhas" enquanto ele tinha 700.

> Regra que mora só em `.md` não segura arquitetura: apodrece calada enquanto o
> gate fica verde. Por isso a §4 e a §6 hoje são verificadas por catraca
> (`scripts/verificar-arquitetura.sh`, dentro do `verificar.sh`), com débito
> congelado em `scripts/arquitetura-baseline.txt` que **só pode diminuir**.

**A consequência prática para quem for propor arquitetura:** meça antes de
propor. O padrão observado é que o problema é **regra não cumprida**, não regra
ausente — duas vezes seguidas (§0.2g na UI, doc 27 no core) a resposta certa foi
"o projeto já tem arquitetura, é boa, e não era aplicada". Propor desenho novo
sem medir é o erro mais caro possível aqui.

**Três âncoras, sempre juntas** (ver ARCHITECTURE.md §1.3):

- **este documento + o código medido** — contra **alucinação de arquitetura**;
- **IDEs open source consolidadas** (Code OSS, IntelliJ IDEA Community, Zed,
  Lapce, NetBeans), com revisão citada — contra **dogmatismo**;
- **documentação oficial da linguagem/tecnologia** (Rust std/reference/clippy,
  Qt e QML, C++, CMake, POSIX) — contra **API imaginada**.

Só o documento produz umbiguismo; só as referências produzem importação de
máquina alheia que este projeto recusou (Node, Electron, WebView, host de
extensões); e sem a documentação oficial o código compila e mente. Arquitetura
entra como MODE-D/MODE-B: aprende-se a regra, não se copia a máquina.

**Sobre a âncora 3, que é a mais fácil de pular:** comportamento de API se
consulta na fonte, não se deduz do nome nem se lembra de cor — e versão e
plataforma mudam a resposta. Este repositório já pagou caro por isso:
`Qt.exit(256)` sai como 0 (8 bits do código de saída POSIX) e deixou 7 dos 14
harnesses com checks que nunca reprovavam; `frameSwapped` é emitido na render
thread; `QProcess::start` é assíncrono e o `sendRequest` descarta em silêncio o
que chega antes de `Running`. Ao afirmar que uma API se comporta de tal forma,
**cite fonte e versão**. Sem fonte consultada, a frase correta é "não sei ainda",
e a próxima ação é consultar ou medir — nunca supor.

**Manter o documento vivo é parte da tarefa.** Ao criar/renomear/remover módulo,
domínio, router ou controller, atualizar o inventário e os números da
`ARCHITECTURE.md` — mapa desatualizado engana mais que a ausência de mapa. Os
fundamentos (camadas §2, regra de split §4/§6, ordem §5, crescimento §6) não
mudam sem decisão explícita e registrada.

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
3. Ler `docs/arquitetura/ARCHITECTURE.md` **inteiro** (camadas, convenções de
   módulo, regra de split, caminho de crescimento). Obrigatório e verificado por
   catraca — ver a seção "Leitura obrigatória" no topo deste arquivo. Se a
   tarefa é propor arquitetura: **medir antes de propor**.
4. Ler `docs/arquitetura/02-repository-structure.md` e `docs/arquitetura/03-ipc-protocol.md` (estrutura e contrato IPC atual).
5. Ler `docs/arquitetura/06-strict-mode.md` (rigor Rust/C++).
6. Consultar `docs/specs/` para a visão-alvo do que está sendo construído (entrada: `SPEC_INDEX`).
7. Verificar se a tarefa pertence ao core, UI, tooling, docs ou protocolo — e ao domínio certo dentro do core.
8. Se a tarefa criar ou alterar uma funcionalidade de IDE, seguir a política
   obrigatória de referência profissional de
   `docs/roadmaps/KINEIN_VECTIS_OPEN_PLUGIN_ADAPTATION_ROADMAP.md`: estudar a implementação
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
`ContextoIA.md` + `docs/specs/` + `docs/arquitetura/ARCHITECTURE.md` + código. Não existe
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
