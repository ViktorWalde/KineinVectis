# 27 — Módulos por domínio: proposta de arquitetura (front + back)

> **Status revisto em 2026-08-29 — o cabeçalho anterior mentia.** Ele dizia
> "PROPOSTA, nada aqui foi implementado" enquanto a §4.3 do MESMO arquivo
> registrava *"APROVADA pelo autor em 2026-07-16: (b)"* e a catraca do core já
> estava no gate. Documento que se contradiz no topo não é proposta pendente: é
> mapa desatualizado, e mapa desatualizado engana mais que a ausência de mapa
> (`AGENTS.md`).
>
> ```text
> ENTREGUE   Item 1 — a catraca cobre o core. `verificar-arquitetura.sh` varre
>            `crates/` desde 2026-07-16; sao 7 arquivos do core na baseline.
> APROVADO   §4.3, opcao (b), pelo autor em 2026-07-16: dividir a composicao
>            por AREA, fazendo crescer a CONTAGEM de arquivos e nunca o limite.
>            Virou a regra 8 da ARCHITECTURE §4.
> ABERTO     Frente 1 (§4) — modulo por dominio na UI: um `<X>Domain` que
>            instancia controller + EventRouter + RequestRouter. VERIFICADO
>            ABERTO em 2026-08-29: nao existe `EditorDomain.qml` e o
>            `AppDomains.qml` nao usa o padrao. E' o caminho para o `Main.qml`
>            sair do debito, e e' fatia PROPRIA.
> ```
>
> Escrito em 2026-07-16 a pedido do autor, antes de mover arquivo.
>
> Referências de arquitetura: VS Code, Zed e IntelliJ IDEA Community, em
> **MODE-D** (referência apenas) — o padrão é aprendido, o código não entra.
> Ver `DocsPublic/roadmaps/adaptacao-de-plugins-abertos.md` §2.

## 1. A conclusão primeiro

**O projeto não precisa de arquitetura nova.** Ele já tem uma, ela é boa, e está
escrita na `ARCHITECTURE.md` §2 (camadas), §4 (organização do core), §5 (onde
colocar código novo) e §6 (crescimento `função → arquivo → pasta → crate`, com os
nomes dos crates futuros já definidos).

É a mesma conclusão que a §0.2g tirou da UI, agora confirmada no core. O que
falta não é desenho — são três buracos concretos:

```text
1. O core nao tem catraca.   A regra §4 existe e e violada em 7 arquivos.
2. A UI nao tem padrao de composition root. O Main.qml nao chega a 400
   por router; o resto e composicao legitima e precisa de outro corte.
3. Nao existe fronteira para SUBSISTEMA OPCIONAL. O simulador OpenGL
   exigido pelo autor nao tem onde morar sem contaminar o nucleo.
```

Este documento ataca os três. Não propõe reescrita.

## 2. Evidência medida (2026-07-16)

**Core — 7 arquivos violam a regra §4** ("quebre em pasta quando misturar
responsabilidade OU passar de ~400–500 linhas de código fora dos testes").
Contagem exclui `#[cfg(test)]` e `tests/`, como a própria regra manda:

```text
955 (1.9x)  crates/kinein-core/src/terminal.rs
732 (1.5x)  crates/kinein-core/src/lsp/manager.rs
696 (1.4x)  crates/kinein-core/src/commands.rs
672 (1.3x)  crates/kinein-core/src/dap/session.rs
653 (1.3x)  crates/kinein-core/src/handlers/lsp.rs
598 (1.2x)  crates/kinein-core/src/lsp/parse.rs
505 (1.0x)  crates/kinein-core/src/lib.rs
```

Dois deles não violam só o tamanho — violam a **responsabilidade que a §4 lhes
atribui por escrito**:

- `lib.rs` (505): a §4 diz *"`lib.rs` é fino. Só o dispatch central e o estado.
  Lógica de domínio nunca entra aqui."*
- `handlers/lsp.rs` (653): a §4 diz *"`handlers/<dominio>.rs` é fino. Roteia,
  valida params, delega, formata a resposta. Sem lógica pesada."*

`scripts/verificar-arquitetura.sh` varre **só `ui/qml` e `ui/src`**. O core nunca
foi verificado. É o mesmo diagnóstico da §0.2g — regra boa apodrecendo calada —
só que aqui ninguém tinha olhado ainda.

> **Este parágrafo e a lista acima são registro de 2026-07-16 e NÃO valem mais.**
> A catraca varre `crates/` desde aquele mesmo dia (é o item ENTREGUE do
> cabeçalho), e **4 dos 7 arquivos foram cortados** desde então: `terminal.rs`
> (955 → pasta), `lsp/manager.rs` (732 → `lsp/session.rs`), `commands.rs`
> (696 → pasta `commands/`) e `lib.rs` (505). Sobram `dap/session.rs`,
> `handlers/lsp.rs` e `lsp/parse.rs`. **Fonte viva:**
> `cat scripts/arquitetura-baseline.txt` — nunca esta lista.

**UI — o composition root não fecha por router.** O split em andando
(`<X>RequestRouter`) levou `Main.qml` de 700 a 610. O que resta é
`GitController` (58), `ShellWorkspaceHost` (50), `SearchController` (45),
`ProjectTreeController` (39), `ShellHeaderHost` (36). Extrair os três controllers
restantes chega a **~515**, ainda acima de 400. O resto é binding de propriedade
e bloco de host: **é** trabalho de composition root e não sai por router.

## 3. O que as três referências resolvem (MODE-D)

O padrão comum às três, e é ele que responde ao `Main.qml`:

> **A raiz não conhece as features. As features se registram.**

- **VS Code** — serviços com injeção de dependência e *contribution points*: uma
  feature declara o que contribui, e a raiz nunca a instancia à mão. O host de
  extensões roda em **processo separado**, com *activation events* (a extensão só
  acorda quando é usada).
- **Zed** — crate por domínio, com fronteira de compilação real. O que é opcional
  é extensão, não `if` espalhado no núcleo.
- **IntelliJ IDEA Community** — *extension points* declarativos; plugin é
  opcional, carregado sob demanda, e o núcleo não sabe quem existe.

**Adaptação honesta ao Kinein.** As três resolvem o problema com máquinas que o
projeto recusou por decisão registrada (Node, Electron, WebView, host de
extensões). Não se importa a máquina — importa-se a **regra**:

```text
o que a raiz precisa saber = quais dominios existem
o que a raiz NAO precisa saber = como cada dominio se liga por dentro
```

Isso é implementável em QML puro com componentes, e em Rust com pasta-módulo. Já
existe precedente dos dois lados.

## 4. Frente 1 — UI: módulo por domínio

O par `<X>EventRouter` (core → controller) + `<X>RequestRouter` (controller →
core) já isola o IPC. Falta o dono: um componente por domínio que instancia os
três e expõe só o controller.

```text
ui/qml/domains/EditorDomain.qml     <- dono: EditorController + os 2 routers
ui/qml/domains/GitDomain.qml
ui/qml/domains/RuntimeDomain.qml
ui/qml/domains/DebugDomain.qml
ui/qml/domains/SearchDomain.qml
ui/qml/domains/ProjectDomain.qml
```

Forma de cada um (explícito por responsabilidade, sem mágica):

```qml
// EditorDomain.qml — tudo do editor que a raiz nao precisa ver.
Item {
    property var coreClient: null
    readonly property alias controller: editorController   // a unica saida

    EditorController { id: editorController; ... }
    EditorEventRouter { coreClient: parent.coreClient; editorController: editorController }
    EditorRequestRouter { coreClient: parent.coreClient; editorController: editorController }
}
```

E o `Main.qml` volta a ser o que o nome promete:

```qml
EditorDomain { id: editorDomain; coreClient: coreClient }
GitDomain    { id: gitDomain;    coreClient: coreClient }
```

**Regra que impede o domínio de virar god de novo:** domínio **não conhece
domínio**. Onde dois se cruzam de verdade — o guard de git que consulta
`editorController.hasModifiedFiles()` antes de checkout — a fiação fica
**explícita no `Main.qml`**. Isso não é dívida: é composição, e é exatamente o
que um composition root deve conter. O que ele não deve conter é o *como* de cada
domínio.

### 4.1 CORREÇÃO da estimativa (medido em 2026-07-16, após executar)

A versão original desta seção afirmava que os módulos por domínio "fecham abaixo
de 400 com folga". **Isso estava errado, e a correção fica registrada** — foi uma
afirmação desatualizada ("Main.qml tem 336 linhas") que deixou a §6 apodrecer;
repetir o vício aqui seria imperdoável.

Estado real depois de extrair **todos** os `RequestRouter` (Editor 19 pedidos,
Runtime 13, Debug 11, Git 15, Search 7, ProjectTree 6):

```text
Main.qml   700 -> 537   (-163 linhas; TODA a fiacao de IPC saiu)

O que sobrou, medido:
  179  controllers   bindings de propriedade + fiacao cross-domain/host
  120  hosts         ShellWorkspaceHost, ShellHeaderHost, ShellOverlays...
   64  routers       14 instanciacoes (vao para dentro dos Domain)
  ~170 resto         Window, Connections, dialogs, dispatcher, onCompleted
```

Os módulos por domínio absorvem os 64 dos routers e parte dos 179. **Piso
estimado: ~440–470. Ainda acima de 400.**

### 4.2 A razão estrutural, e ela não é preguiça

Sem registro/contribuição, **um composition root cresce linearmente com o número
de domínios**. É isso que as três referências resolvem — e resolvem com máquina
que este projeto recusou por decisão registrada (DI runtime, extension host,
registry dinâmico). Removida a máquina, sobra a linearidade.

Ou seja: as 537 linhas de hoje não são dívida no sentido da §0.2g. São o trabalho
que um composition root de uma IDE com ~12 domínios de fato tem. O limite de 400
para `Main.qml` não foi calibrado para isso: ele vem da regra genérica
"Controller/Host/Main.qml → 400", herdada de quando o arquivo tinha 336 linhas e
não continha a IDE inteira.

**Decisão em aberto — é do autor, e não deve ser tomada por conveniência:**

```text
(a) Modulos por dominio + limite proprio para composition root (ex.: 500),
    explicito e justificado no gate, com a catraca continuando a impedir
    crescimento. Pragmatico; risco: "aumentar o limite quando incomoda" e
    exatamente como a regra apodrece.

(b) Modulos por dominio + dividir a COMPOSICAO em arquivos por area
    (dominios / hosts / atalhos), com o Main.qml so montando as pecas.
    Fecha abaixo de 400 sem mexer no limite; custo: indirecao a mais.

(c) Registro minimo em QML (as features se registram). Resolve de verdade e
    e o que as referencias fazem — mas e a maquina que o projeto recusou, e
    o ganho nao paga a complexidade nesta escala.
```

**APROVADA pelo autor em 2026-07-16: (b).** Mantém a catraca intacta (nunca se
gira o limite para o lado confortável), e o corte por área é honesto — hosts e
domínios são coisas diferentes.

A (b) também responde à exigência do autor de que **os limites acompanhem o
crescimento do projeto sem que a realidade de hoje trave o amanhã**. E responde
da única forma que não apodrece: dividida por área, cada arquivo fica pequeno e
o que cresce é a **contagem** de arquivos, não o tamanho de cada um. Domínio novo
(embarcados, containers) traz arquivos novos e a catraca nem percebe — porque
nascem dentro do limite. Ver `ARCHITECTURE.md` §4 regra 8.

### 4.4 Execução da (b): o corte, medido e pronto para rodar

**Só o movimento COMPLETO chega a 400.** Medido em 2026-07-16 — cortes parciais
(só os routers, ou só alguns domínios) param em ~465–490 e somam indireção sem o
pagamento. Meia refatoração é pior que nenhuma: fica a indireção *e* fica o
problema.

```text
ui/qml/app/AppDomains.qml   (EXECUTADO em 0686213 — limite 400, e a categoria
                             foi corrigida: composicao nao e visual)
  dono dos controllers + dos routers de dominio; cresce com a CONTAGEM de
  dominios, que e o desenho (§4 regra 8). Tinha 335 linhas em 2026-07-17 e
  356 em 2026-09-02, quando as Configuration Actions nasceram.
  recebe:  coreClient, shellController, workspaceHost, shellOverlays
  expoe:   readonly property alias <x>Controller  (um por dominio)

ui/qml/Main.qml   537 -> ~300
  fica com: Window, hosts visuais, dialogs, atalhos, Component.onCompleted
  ganha:    AppDomains { id: domains; ... }   (~6 linhas)
```

Por que fecha: os controllers somam **179** linhas e os routers **64** = 243
saem; entram ~6. E dentro do `AppDomains` as referências cruzadas entre
controllers continuam sendo ids simples do mesmo arquivo — não viram
pass-through.

**Custo medido: 204 referências** no `Main.qml` viram `domains.<x>` (editor 34,
shell 21, runtime 19, search 19, git 17, settings 17, debug 15, projectTree 15,
jobs 13, diagnostics 10, workspace 9, recentWorkspaces 8, projectHealth 5,
dispatcher 2). Só as que **ficam** no `Main.qml` precisam do prefixo — as que
moram dentro dos blocos que migram continuam ids simples.

**Ordem segura de execução** (o refactor é largo; cada passo com gate verde):

```text
1. Criar AppDomains.qml com os 12 controllers + 14 routers e os aliases.
2. Main.qml: instanciar AppDomains e prefixar as referencias que sobraram.
3. Build dev-local E linux-clang-debug-strict.
   ATENCAO: verificar-qml.sh le o response file do STRICT primeiro; reconstruir
   so o dev-local faz o qmllint acusar tipo novo como inexistente.
4. qmllint estrito + catraca + logica QML.
5. Atualizar a baseline (Main.qml sai do debito) no mesmo commit.
```

Depois disso, `RuntimeController.qml` (494/400) ainda bloqueia o seletor: separar
run configs de terminais é fatia própria.

### 4.3 O seletor continua bloqueado — os routers não resolveram isso

Registro de um erro de raciocínio, para ninguém repetir: os `RequestRouter`
tiraram linhas do **`Main.qml`** (o bloco de instanciação, onde os
`on*Requested` eram declarados), **não dos arquivos dos controllers**. São coisas
diferentes.

```text
Main.qml                    700 -> 537   (o bloco do RuntimeController: 53 -> 20)
RuntimeController.qml       494 -> 494   (INTACTO. O router nao o tocou.)
```

Logo os **dois** bloqueios do seletor (§0.2f) seguem de pé: `Main.qml` acima de
400 e `RuntimeController.qml` acima de 400. Destravar exige, além da decisão da
§4.1, separar terminais de run configs dentro do `RuntimeController` — a costura
já identificada na §0.2g (ele mistura terminais + Assistente + run configs).

## 5. Frente 2 — Core: dar dente à regra que já existe

**Estender a catraca a `crates/`**, contando linhas fora de testes, como a §4
define. Mesmo mecanismo da UI: baseline congela os 7 atuais, só pode diminuir,
arquivo novo acima do limite reprova.

Ordem de pagamento pelo critério da própria §4 — **quem mistura responsabilidade
primeiro, não quem é maior**:

```text
1. lib.rs (505)          A §4 ja diz o que ele deveria ser: dispatch + estado.
                         O excedente e logica de dominio no lugar errado.
2. handlers/lsp.rs (653) A §4 ja diz: handler e fino. 653 nao e fino.
3. commands.rs (696)     Descritores: provavelmente dados, nao logica.
4. lsp/manager.rs (732)  Ja e pasta; continuar dividindo por responsabilidade.
5. dap/session.rs (672)  Idem.
```

**`terminal.rs` (955) fica por último, e de propósito.** É o maior violador e o
mais perigoso: é onde moram a grade VT, o cursor/DECSCUSR e o `wheel_action`, e
foi o que custou dois dias de estabilização ao autor. A catraca **congela** o
arquivo (não pode crescer) sem exigir quebrá-lo agora. Quebrar terminal para
"cumprir métrica" seria trocar risco real por número bonito.

### PAGO em 2026-08-30 — e o que destravou primeiro foi a rede, não a coragem

O corte só aconteceu depois de duas coisas que **não** eram refatoração:
`tests/terminal.rs` (2026-08-29, 8 testes de integração, incluindo o contrato de
`event.terminal.render` que trava os nomes de campo lidos pelo QML) e o aceite
visual do autor no terminal rodando. Foi isso que tornou o risco mensurável — a
condição que faltava era evidência, não disposição.

```text
terminal/mod.rs       49   submodulos, re-export, EventSender e MAX_SESSIONS
terminal/session.rs  471   PTY, threads de ciclo de vida, teto de sessoes
terminal/render.rs   246   snapshot do grid -> event.terminal.render
terminal/input.rs    135   o que a roda significa; formato de fio do mouse
terminal/state.rs     83   grid do emulador e dimensoes do viewport
terminal/error.rs     46   vocabulario de erro, que o rpc.rs mapeia
```

O `error.rs` não estava no corte planejado em quatro: apareceu porque
`session.rs` fechou em 509 linhas, nove acima do limite. **Os dois reflexos
errados ali seriam cortar nove linhas quaisquer ou subir o baseline** — a §4
regra 9 chama os dois de trapaça. O que havia de fato era uma quinta
responsabilidade que o próprio repositório já nomeia em `fsops/error.rs` e
`workspace/error.rs`: o vocabulário de erro atravessa a fronteira, porque o
`rpc.rs` mapeia cada variante para um código JSON-RPC.

Aceite verificado, os dois juntos: os mesmos 26 testes verdes antes e depois
(8 de integração + 18 unitários, nomes idênticos), e o teste de VOCABULÁRIO da
regra 9 — `grep -i span` em `session.rs` não devolve nada. Esse segundo critério
mudou uma decisão de projeto: os helpers de teste que decodificam o evento de
render (`render_text`/`render_contains`) ficaram em `render.rs`, expostos aos
irmãos por `pub(in crate::terminal)`, em vez de renomear a variável `span` no
teste de sessão para escapar do `grep`. Um teste de sessão afirma que a saída
apareceu; ele não conhece a forma do contrato.

## 6. Horizonte registrado — subsistema opcional (REMOVIDO em 2026-09-12)

> Esta seção descrevia o simulador OpenGL como subsistema opcional e a colisão
> dele com o renderer por software do AppImage (§6.2). A simulação **saiu do
> produto em 2026-09-12** por decisão do autor; o texto está íntegro em
> `DocsPrivate/historico/simulacao/`. O que dela continua valendo para
> qualquer subsistema futuro que peça GPU: o AppImage força renderer por
> software, e é isso que faz a IDE abrir em qualquer máquina — um subsistema
> com GPU nasce como processo separado, opt-in, nunca dentro do processo da UI.

## 7. Nomes explícitos por responsabilidade

Exigência do autor, e ela vira regra verificável:

```text
- Arquivo diz O QUE FAZ, nao onde esta: EditorRequestRouter, nao EditorUtils2.
- Pasta = dominio (editor/, git/, serial/), nunca tipo de arquivo (helpers/, utils/).
- Sufixo carrega a responsabilidade e o limite da catraca depende dele:
    *Controller  estado + decisao   (400)
    *Router      transporte IPC     (300)
    *Panel/*View render             (300)
    *Domain      composicao         (300)
- Proibidos: utils, helpers, common, misc, manager sem dominio. Nome que nao
  cabe em nenhuma categoria e sinal de que a responsabilidade nao foi decidida.
```

## 8. O que NÃO fazer

- **Não** criar um crate por serviço agora. A §6 já é explícita: pasta-módulo
  primeiro, crate só quando houver ganho real.
- **Não** big-bang. Cada frente é fatia própria, com gate verde e catraca
  atualizada no mesmo commit.
- **Não** quebrar um arquivo para cumprir métrica. O `terminal.rs` era o caso
  citado aqui e foi pago em 2026-08-30 — por responsabilidade e com rede de
  teste antes, não por causa do número. A regra que ele exemplifica continua.
- **Não** importar DI, host de extensões ou registry dinâmico das referências. O
  padrão é MODE-D: aprende-se a regra, não se copia a máquina.

## 9. Ordem proposta

Tudo aqui serve ao passo 1 da ordem do autor (solidificar C/C++ e Rust). Nada
depende do §6.

```text
1. Catraca no core (crates/), baseline congelando os 7.
   Dente antes de divida nova. Barato e sem risco.
2. Modulos por dominio na UI  ->  Main.qml < 400  ->  DESTRAVA o seletor (§0.2f).
3. Seletor do Assistente + rename (§0.2f passos 2-4, §0.2d-2).
4. Pagar a §4 no core por RESPONSABILIDADE: lib.rs, handlers/lsp.rs, commands.rs.
```

Fora desta fila, sem prazo: a fronteira do simulador (§6), que só é decidida
quando embarcados estiver solidificado — e a decisão 6.3 (a) vs (b) precisa vir
antes da primeira linha de simulador, porque ela determina se a IDE continua
abrindo em máquina sem GPU.
