# 00 — Comece aqui

> **Para quem:** qualquer pessoa ou IA que vá ler, alterar ou revisar este
> repositório sem ter estado nas conversas anteriores. Escrito em 2026-10-01 a
> pedido do autor: "deixe tudo explícito para qualquer pessoa ou IA que queira
> colaborar". **Este arquivo não contém fatos do projeto; ele diz onde cada fato
> mora.** Se ele e o documento apontado divergirem, o apontado vence.

## 1. O projeto em quatro frases

A Kinein Vectis é uma IDE **Linux nativa** para C, C++, Rust, Python e sistemas
embarcados. Ela **orquestra** ferramentas consolidadas (clangd, rust-analyzer,
CMake, Cargo, GDB, esptool…) em vez de reimplementá-las. A interface é Qt/QML e
o trabalho é de um core Rust, em **processos separados** que conversam por
JSON-RPC em stdin/stdout. Sem telemetria, sem IA embutida, sem runtime de
plugin de terceiros — cada "não" é uma decisão registrada
([`leitura-tecnica.md`](leitura-tecnica.md) §1).

## 2. As seis regras que valem antes de qualquer outra

1. **O código e os gates são a única fonte do que existe.** Nenhum documento
   derruba uma medição. Antes de afirmar que algo falta ou existe, meça
   ([`README.md`](README.md), "Ordem de precedência").
2. **Um fato, um dono.** Nada se duplica — nem código, nem documento. Uma
   segunda implementação divide o lugar onde o bug pode estar; um segundo texto
   sobre o mesmo fato vira mentira quando só um é atualizado
   ([`ARCHITECTURE.md`](arquitetura/ARCHITECTURE.md) §8.1).
3. **Aberto para extensão, fechado para modificação — com responsabilidade.**
   Crescer acrescentando uma entrada de dados ou um módulo novo, não editando o
   mecanismo; mas **mecanismo genérico sem usuário é pior que nenhum**
   (ARCHITECTURE §8). Exemplos vivos: um contexto novo no mapa de módulos é uma
   entrada em `CONTEXTS` (`scripts/module_map.py`); um gate de ambiente novo usa
   o protocolo NÃO PROVADO sem tocar no orquestrador
   ([`contribuindo/07`](contribuindo/07-fluxo-e-responsabilidades-dos-gates.md) §1.1).
4. **Decisão do autor não se reabre por conta própria.** Ela está registrada,
   com data, no dono (a lista está no [`40`](roadmaps/40-estado-e-continuidade.md) §5).
   Proposta não é decisão: o [mapa de versões](roadmaps/57-mapa-de-versoes-ate-a-1.0.md)
   separa as duas. Para os rascunhos de HUD/UX de 2026-10-21, imagens não
   são design aprovado: a direção visual depende da aprovação do autor
   sobre a UI atual ([frontend §1.1](especificacoes/arquitetura-de-frontend-0.3-em-diante.md#11-aprovação-visual-e-referências--decisão-de-2026-10-21)).
5. **Toda mudança passa pelo gate**, `scripts/verificar.sh`, e todo gate novo é
   provado por mutação ([`contribuindo/04`](contribuindo/04-os-gates-que-dizem-nao.md)).
6. **Bloqueador real fura a fila.** Perda de dados, crash e bloqueio do uso
   diário vêm antes de qualquer item planejado
   ([`roadmaps/34`](roadmaps/34-depois-do-mvp.md) §7).

## 3. A arquitetura da documentação

A documentação pública tem três eixos, e todos estão descritos por inteiro no
índice, [`README.md`](README.md):

- **Pasta = assunto.** `arquitetura/` (como o código é), `roadmaps/` (estado,
  registro, versões e planos), `especificacoes/` (o alvo do produto),
  `contribuindo/` (como trabalhar), `build/`, `integracoes/`, `seguranca/`,
  `decisoes-adr/`, `pesquisa/`, `iconografia/` — a árvore com o que cada uma
  guarda está em [`README.md`](README.md) §Estrutura, e cada pasta tem o
  próprio índice, que cita **todo** documento dela pelo nome.
- **Classe = o que acontece se o documento envelhecer.** CONTRATO (regra; não
  muda sem decisão), ESTADO (tem de ser verdade hoje), PLANO (alvo; pode
  divergir), LOG (registro datado; nunca se reescreve) e PESQUISA (hipótese;
  não autoriza nada) — [`README.md`](README.md) §"As quatro classes".
- **Precedência = quem vence num conflito.** O código e os gates, depois o
  contrato, o estado, o plano, o log e a pesquisa —
  [`README.md`](README.md) §"Ordem de precedência".

Como os documentos se apoiam uns nos outros:

```mermaid
flowchart TD
  porta["00-comece-aqui.md<br/>a porta"] --> indice["README.md<br/>índice: pastas, classes, precedência"]
  porta --> contribuindo["contribuindo/<br/>como trabalhar; 04 = os gates"]
  indice --> pastas["índice de cada pasta<br/>(todo documento pelo nome)"]
  porta -->|"o que está aberto"| estado
  porta -->|"para onde vai"| versoes
  porta -->|"quem fala com quem"| mapa
  codigo[("código")] -->|"gerado e conferido por gate"| mapa["arquitetura/01<br/>mapa de módulos"]
  codigo -->|"conferido por gate"| protocolo["arquitetura/03<br/>protocolo IPC"]
  contrato["arquitetura/ARCHITECTURE.md<br/>CONTRATO"] -.->|"regras que o código segue"| codigo
  versoes["roadmaps/57<br/>versões até a 1.0"] -->|"dono do detalhe"| plano["roadmap da versão<br/>(53, 52, 49/50…)"]
  versoes -->|"onde começa no código"| inicio["roadmaps/58<br/>arquivos e 1ª fatia"]
  inicio -->|"caminhos conferidos por gate"| codigo
  plano -->|"alvo de cada área"| specs["especificacoes/<br/>PLANO"]
  estado["roadmaps/40<br/>ESTADO: a fila"] -->|"a fatia feita vira"| registro["roadmaps/40.7<br/>LOG: o registro"]
  plano -->|"a fatia em curso entra na"| estado
  adr["decisoes-adr/<br/>o porquê"] -.-> contrato
```

Duas regras mantêm isso de pé, e as duas têm gate: **um fato mora num
documento só** (os outros apontam para ele) e **a documentação pública só cita
documentação pública** (§6).

## 4. Onde mora cada fato

| Pergunta | Dono (leia este) |
| --- | --- |
| O que o projeto é, medido? | [`leitura-tecnica.md`](leitura-tecnica.md) |
| Quais pastas existem e o que há em cada uma? | [`arquitetura/02-estrutura-do-repositorio.md`](arquitetura/02-estrutura-do-repositorio.md) |
| **Quem fala com quem, por quê e como?** | [`arquitetura/01-mapa-de-modulos.md`](arquitetura/01-mapa-de-modulos.md) — gerado do código, com um diagrama por contexto |
| Quais são as regras de engenharia (camadas, onde pôr código novo, anti-padrões)? | [`arquitetura/ARCHITECTURE.md`](arquitetura/ARCHITECTURE.md) |
| Qual a forma de cada mensagem UI ↔ core? | [`arquitetura/03-protocolo-ipc.md`](arquitetura/03-protocolo-ipc.md) |
| Como o processo sobe e as mensagens circulam? | [`arquitetura/04-boot-e-comunicacao.md`](arquitetura/04-boot-e-comunicacao.md) |
| O que está aberto agora, e as decisões que não se reabrem? | [`roadmaps/40-estado-e-continuidade.md`](roadmaps/40-estado-e-continuidade.md) |
| O que foi feito, quando, e com que prova? | [`roadmaps/40.7-registro-das-entregas.md`](roadmaps/40.7-registro-das-entregas.md): o índice das partes e a fase aberta |
| **O que ler para mexer num contexto, e o que dispensar?** | §5.1 abaixo |
| **Quais versões existem até a 1.0, e o que é decidido ou proposta?** | [`roadmaps/57-mapa-de-versoes-ate-a-1.0.md`](roadmaps/57-mapa-de-versoes-ate-a-1.0.md) |
| **Onde, no código, cada versão começa — arquivos, mecanismo a estender, primeira fatia?** | [`roadmaps/58-onde-cada-versao-comeca-no-codigo.md`](roadmaps/58-onde-cada-versao-comeca-no-codigo.md) — caminhos conferidos por gate |
| O plano da versão em curso? | o que o [mapa de versões](roadmaps/57-mapa-de-versoes-ate-a-1.0.md) marca EM CURSO; hoje (desde 2026-10-16), a 0.4, [`roadmaps/52-arquitetura-executavel-da-0.4.md`](roadmaps/52-arquitetura-executavel-da-0.4.md) §11 |
| A visão-alvo de cada área do produto? | [`especificacoes/README.md`](especificacoes/README.md) — alvo, não estado |
| Por que uma decisão de arquitetura foi tomada? | [`decisoes-adr/`](decisoes-adr/README.md) |
| Como montar o ambiente e compilar? | [`contribuindo/02-preparar-o-ambiente.md`](contribuindo/02-preparar-o-ambiente.md), [`build/comandos-de-build-e-verificacao.md`](build/comandos-de-build-e-verificacao.md) |
| O que cada gate mede e como ler o "não"? | [`contribuindo/04-os-gates-que-dizem-nao.md`](contribuindo/04-os-gates-que-dizem-nao.md) |
| Convenções de código, testes e commits? Nomes de identificador? | [`contribuindo/08`](contribuindo/08-convencoes-codigo-testes-commits.md), [`contribuindo/09`](contribuindo/09-glossario-de-identificadores.md) |
| Licenciamento, distribuição e procedência das contribuições? | [`contribuindo/12`](contribuindo/12-licenciamento-e-procedencia.md); colaboração institucional no [`13`](contribuindo/13-colaboracao-institucional.md) |
| Como usar a IDE (usuário final)? | [`manual.md`](manual.md), [`tutorial.md`](tutorial.md) |

## 5. Por onde começar, conforme o que você veio fazer

**Entender o projeto (30 minutos).** `leitura-tecnica.md` §1–§3 → o nível 1 e o
caminho de um pedido em `arquitetura/01-mapa-de-modulos.md` → o mapa de versões.

**Fazer uma mudança (uma fatia).**

1. Leia o estado: o handoff no topo do `roadmaps/40` e o §4 (o que está
   aberto, com o dono de cada ordem).
2. Ache o contexto da mudança na tabela abaixo e leia **só a linha dele**; o
   resto da documentação fica para quando a linha mandar. Depois, o "onde
   colocar código novo" do `ARCHITECTURE.md` §5. **Se já existe um mecanismo
   para isso, estenda-o** (ARCHITECTURE §8.1).
3. Siga o ritual de [`contribuindo/03-o-ritual-de-uma-fatia.md`](contribuindo/03-o-ritual-de-uma-fatia.md).
4. Rode `scripts/verificar.sh` (antes de release, com `--estrito`).
5. Registre: estado no `40`, entrega datada no `40.7`, decisão nova no dono dela.

### 5.1 O que cada contexto lê (desde 2026-10-15)

Pedido do autor: "separar a documentação por contexto", para que ninguém leia
o que não vai mexer ([`roadmaps/61`](roadmaps/61-organizacao-da-documentacao.md)).
Os contextos de código são os do mapa de módulos (`CONTEXTS` em
`scripts/module_map.py`); cada um tem uma seção "Contextos" no
[`arquitetura/01`](arquitetura/01-mapa-de-modulos.md), que é sempre a primeira
leitura dele. Do protocolo, leia o [`arquitetura/03`](arquitetura/03-protocolo-ipc.md)
(o formato base, as listas e as regras) e só os arquivos dos domínios da
linha, em [`arquitetura/03-dominios/`](arquitetura/03-dominios/README.md)
(um por domínio desde 2026-10-16).

| Contexto | Leia, além do 01 | Domínios no 03 |
| --- | --- | --- |
| Workspace e árvore | [`especificacoes/projetos-arquivos-e-integracao-desktop-0.3.md`](especificacoes/projetos-arquivos-e-integracao-desktop-0.3.md) | [`workspace.*`](arquitetura/03-dominios/workspace.md), [`fs.*`](arquitetura/03-dominios/fs.md), [`draft.*`](arquitetura/03-dominios/draft.md) |
| Editor e LSP | [`arquitetura/32`](arquitetura/32-editor-por-responsabilidade.md), [`especificacoes/editor-e-inteligencia-de-linguagem.md`](especificacoes/editor-e-inteligencia-de-linguagem.md); o motor semântico só se for ele: [`roadmaps/56`](roadmaps/56-motor-semantico-profundo-cpp-rust.md) | [`lsp.*`](arquitetura/03-dominios/lsp.md), [`syntaxTree.*`](arquitetura/03-dominios/syntaxTree.md), [`format.*`](arquitetura/03-dominios/format.md) |
| Busca, índice e símbolos | [`arquitetura/33`](arquitetura/33-busca-no-projeto.md) | [`fs.search`](arquitetura/03-dominios/fs-busca.md), [`fs.findFiles`](arquitetura/03-dominios/fs-busca.md), [`index.*`](arquitetura/03-dominios/index.md) |
| Git | — | [`git.*`](arquitetura/03-dominios/git.md) |
| Executar, terminal, build e testes | [`especificacoes/fluxos-de-produto-build-run-debug.md`](especificacoes/fluxos-de-produto-build-run-debug.md), [`especificacoes/terminal-ergonomia-0.3.md`](especificacoes/terminal-ergonomia-0.3.md) | [`run.*`](arquitetura/03-dominios/run.md), [`terminal.*`](arquitetura/03-dominios/terminal.md), [`build.*`](arquitetura/03-dominios/build.md), [`quality.*`](arquitetura/03-dominios/quality.md), [`test.*`](arquitetura/03-dominios/test.md), [`job.*`](arquitetura/03-dominios/job.md), [`coverage.*`](arquitetura/03-dominios/coverage.md) |
| Depuração (DAP) | [`especificacoes/fluxos-de-produto-build-run-debug.md`](especificacoes/fluxos-de-produto-build-run-debug.md) | [`debug.*`](arquitetura/03-dominios/debug.md) |
| Embarcados | [`roadmaps/52`](roadmaps/52-arquitetura-executavel-da-0.4.md) (a 0.4), [`roadmaps/42`](roadmaps/42-trilha-profunda-embarcados.md), [`especificacoes/embarcados-targets-flash-serial-qemu.md`](especificacoes/embarcados-targets-flash-serial-qemu.md), [`integracoes/36`](integracoes/36-ferramentas-de-embarcados.md) | [`probe.*`](arquitetura/03-dominios/probe.md), [`dfu.*`](arquitetura/03-dominios/dfu.md), [`picotool.*`](arquitetura/03-dominios/picotool.md), [`serial.*`](arquitetura/03-dominios/serial.md), [`project.*`](arquitetura/03-dominios/project.md) |
| Toolchains e kits | [`integracoes/39`](integracoes/39-toolchains-por-alvo.md), [`build/22`](build/22-compilacao-c-cpp-rust.md) | [`toolchain.*`](arquitetura/03-dominios/toolchain.md), [`cmake.*`](arquitetura/03-dominios/cmake.md), [`cargo.*`](arquitetura/03-dominios/cargo.md), [`runConfig.*`](arquitetura/03-dominios/runConfig.md) |
| Remote SSH | [`especificacoes/remote-ssh-ui-hud.md`](especificacoes/remote-ssh-ui-hud.md) | [`remote.*`](arquitetura/03-dominios/remote.md) |
| Banco e observabilidade | [`arquitetura/37`](arquitetura/37-banco-de-dados.md), [`38`](arquitetura/38-provedores-de-banco-e-linguagem.md), [`39`](arquitetura/39-drivers-externos-e-compatibilidade.md), [`40`](arquitetura/40-contrato-operacional-de-drivers.md); os adiados pelo autor no [`roadmaps/59`](roadmaps/59-fechamento-da-0.3.9.md) §2.3 (registro) | [`datasource.*`](arquitetura/03-dominios/datasource.md), [`grafana.*`](arquitetura/03-dominios/grafana.md) |
| Containers | [`roadmaps/59`](roadmaps/59-fechamento-da-0.3.9.md) §2.5 (o 16b) e §3 | [`container.*`](arquitetura/03-dominios/container.md) |
| Python | [`roadmaps/42`](roadmaps/42-trilha-profunda-embarcados.md) §9 | [`python.*`](arquitetura/03-dominios/python.md) |
| Configuração, biblioteca, setup | [`especificacoes/fluxo-duplo-e-acoes-de-configuracao.md`](especificacoes/fluxo-duplo-e-acoes-de-configuracao.md); para Setup, [`Instalar Ferramentas`](especificacoes/instalar-ferramentas.md) | [`configAction.*`](arquitetura/03-dominios/configAction.md), [`library.*`](arquitetura/03-dominios/library.md), [`setup.*`](arquitetura/03-dominios/setup.md), [`settings.*`](arquitetura/03-dominios/settings.md), [`command.*`](arquitetura/03-dominios/command.md) |
| Quality Center e gates locais | [`especificacoes/quality-center-e-gates.md`](especificacoes/quality-center-e-gates.md); ordem no 52 §11.1 | [`build.*`](arquitetura/03-dominios/build.md), [`quality.*`](arquitetura/03-dominios/quality.md), [`test.*`](arquitetura/03-dominios/test.md), [`job.*`](arquitetura/03-dominios/job.md), [`settings.*`](arquitetura/03-dominios/settings.md), [`configAction.*`](arquitetura/03-dominios/configAction.md) |

Os contextos que não são domínio do core:

| Contexto | Leia |
| --- | --- |
| A casca da UI (QML/C++) | [`arquitetura/36`](arquitetura/36-casca-da-ide.md), [`especificacoes/arquitetura-de-frontend-0.3-em-diante.md`](especificacoes/arquitetura-de-frontend-0.3-em-diante.md), [`especificacoes/sistema-de-componentes-de-ui.md`](especificacoes/sistema-de-componentes-de-ui.md) |
| Um contrato IPC novo | a seção do domínio no [`arquitetura/03`](arquitetura/03-protocolo-ipc.md) e a nota no topo do [`03.1`](arquitetura/03.1-notas-por-versao-do-protocolo.md) |
| Plataforma (Linux e Windows) | [`roadmaps/60`](roadmaps/60-windows-e-edicao-especial.md) §2.1 e §3 |
| Gates e build | [`contribuindo/04`](contribuindo/04-os-gates-que-dizem-nao.md), [`contribuindo/07`](contribuindo/07-fluxo-e-responsabilidades-dos-gates.md), [`build/comandos-de-build-e-verificacao.md`](build/comandos-de-build-e-verificacao.md) |
| A documentação | [`README.md`](README.md) (classes e precedência), [`roadmaps/61`](roadmaps/61-organizacao-da-documentacao.md) |
| A fila e as versões | [`roadmaps/40`](roadmaps/40-estado-e-continuidade.md) §4, [`roadmaps/57`](roadmaps/57-mapa-de-versoes-ate-a-1.0.md), [`roadmaps/58`](roadmaps/58-onde-cada-versao-comeca-no-codigo.md) |

**O que nenhum contexto lê de saída:** o registro
([`roadmaps/40.7`](roadmaps/40.7-registro-das-entregas.md) e
[`roadmaps/registro/`](roadmaps/registro/README.md)), que se lê pela entrada
citada, para responder "por que isto é assim?"; o
[`03.1`](arquitetura/03.1-notas-por-versao-do-protocolo.md), se a seção do
domínio bastar; os roadmaps de etapas fechadas (o índice de
[`roadmaps/`](roadmaps/README.md) marca quais são registro); e as
especificações de outras áreas.

**Se você é uma IA.** Tudo acima, e mais: leia
[`contribuindo/05-com-um-agente-de-ia.md`](contribuindo/05-com-um-agente-de-ia.md).
Três erros já custaram fatias inteiras a este projeto, e estão documentados para
não se repetirem: implementar a partir de uma especificação **cancelada**
(ADR-0005), confiar num número de documento sem medir (`leitura-tecnica.md`,
nota do topo), e ligar um sinal no lugar errado — o build passa e a função
deixa de funcionar em silêncio (ARCHITECTURE §8).

## 6. Só o que é público

Tudo o que é preciso para colaborar está neste repositório. O autor guarda
esboços fora dele, e **nenhum documento versionado aponta para eles**: a
documentação pública só se refere a documentação pública (decisão do autor,
2026-10-01; o `scripts/verificar-links-docs.sh` reprova citação de caminho
interno). Se um texto parecer depender de algo que você não encontra aqui, é
defeito de documentação — abra um issue.
