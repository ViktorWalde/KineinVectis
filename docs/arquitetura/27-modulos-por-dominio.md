# 27 — Módulos por domínio: proposta de arquitetura (front + back)

> **Status: PROPOSTA. Nada aqui foi implementado.** Escrita em 2026-07-16 a
> pedido do autor, antes de mover arquivo. Requer aprovação.
>
> Referências de arquitetura: VS Code, Zed e IntelliJ IDEA Community, em
> **MODE-D** (referência apenas) — o padrão é aprendido, o código não entra.
> Ver `docs/roadmaps/KINEIN_VECTIS_OPEN_PLUGIN_ADAPTATION_ROADMAP.md` §2.

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
(embarcados, simulação) traz arquivos novos e a catraca nem percebe — porque
nascem dentro do limite. Ver `ARCHITECTURE.md` §4 regra 8.

### 4.4 Execução da (b): o corte, medido e pronto para rodar

**Só o movimento COMPLETO chega a 400.** Medido em 2026-07-16 — cortes parciais
(só os routers, ou só alguns domínios) param em ~465–490 e somam indireção sem o
pagamento. Meia refatoração é pior que nenhuma: fica a indireção *e* fica o
problema.

```text
ui/qml/app/AppDomains.qml   (EXECUTADO em 0686213; 350 linhas — limite 400,
                             a categoria foi corrigida: composicao nao e visual)
  dono dos 12 controllers + dos 14 routers, mais o ProjectTreeGestures
  (interprete de arraste/teclado da arvore, sem estado proprio) e o
  EditorMarkdownModeController (modo "imagem" dos .md, 2026-07-18).
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

## 6. Horizonte registrado — subsistema opcional (o simulador OpenGL)

> **NÃO É ESCOPO ATUAL. Nada aqui entra em fila de execução.** O autor foi
> explícito em 2026-07-16: o simulador foi **menção de exemplo** para deixar
> clara a preocupação com arquitetura, não pedido de trabalho. A seção existe
> para o achado da 6.2 não se perder — ele é caro de redescobrir e barato de
> registrar.
>
> **Ordem real do projeto, decidida pelo autor:**
>
> ```text
> 1. Solidificar C/C++ e Rust na IDE          <- e aqui que a arquitetura mira AGORA
> 2. Solidificar embarcados, de forma profissional
> 3. So entao simulacao fisica/matematica
> ```
>
> Até o passo 3 vai levar tempo. As frentes 1 e 2 deste documento (UI por
> domínio, catraca no core) servem ao passo 1 e não dependem de nada daqui.

O que o autor descreveu quando chegar a vez: simulador integrado usando OpenGL,
**desativado por padrão**, que o usuário instala/ativa.

O valor de registrar agora é um só: a 6.2 mostra que essa feature **colide com
uma garantia já conquistada** do AppImage. Saber disso desde já evita que as
decisões dos passos 1 e 2 fechem a porta — não obriga a abri-la hoje.

### 6.1 A linha, e ela já tem precedente

```text
DETECTAR o simulador existe?    → CAPACIDADE. ToolDetector, como claude/codex.
ATIVAR   o usuario quer?        → POLITICA. Settings + UI.
EXECUTAR como roda?             → FRONTEIRA. Fora do nucleo.
```

O passo 1 do §0.2f já provou o padrão: o core detecta `claude` como detecta
`cargo`, sem ramo por programa. Simulador é o mesmo caso.

### 6.2 A descoberta que muda o desenho

**O AppImage força `QT_QUICK_BACKEND=software` por padrão, de propósito.**
`packaging/appimage/kinein-portable-graphics-hook.sh` faz
`KINEIN_GRAPHICS_BACKEND:-software`, e a razão está registrada: o driver do host
não criava contexto RHI/OpenGL e a UI abortava antes do primeiro frame. A raster
oficial do Qt Quick desacoplou a abertura de EGL/Mesa/NVIDIA em Wayland e X11. E
isso só é possível porque **a UI hoje é 100% 2D** — não há uma linha de
`ShaderEffect`, `QQuickFramebufferObject` ou OpenGL em `ui/`.

O simulador precisa de GPU. Logo:

> Ativar o simulador **colide de frente com a garantia de abertura do AppImage**.
> Não é detalhe de implementação: é decisão de arquitetura, e tem de ser tomada
> antes da primeira linha de simulador.

### 6.3 As duas saídas reais

**(a) Simulador em processo próprio, com contexto GL próprio — RECOMENDADA.**
A UI da IDE continua raster e continua abrindo em qualquer máquina. O simulador é
um binário separado (`kinein-sim`), detectado como ferramenta, lançado como job,
falando o **mesmo JSON-RPC stdio** que o core já fala. Se a GPU do usuário falhar,
falha o simulador — não a IDE. É a lição do host de extensões do VS Code
(isolamento por processo) sem importar a máquina dele. Pela §6 da
`ARCHITECTURE.md`, nasce como `crates/kinein-core/src/sim/` e vira crate
`kinein-sim` quando ganhar corpo — o nome já fica certo desde o início.

**(b) A UI inteira passa a hardware quando o simulador é ativado.**
`KINEIN_GRAPHICS_BACKEND=hardware` já existe como opt-in reversível, então o
mecanismo está pronto — mas exige reinício e devolve a IDE inteira à dependência
de driver que o AppImage evitou. Um bug de GPU volta a impedir a IDE de abrir,
não só o simulador.

A (a) preserva o que já foi conquistado. A (b) é mais simples de escrever e mais
cara de manter.

### 6.4 Regra do opt-in, independente da saída escolhida

```text
- Desativado por padrao significa CUSTO ZERO: sem instancia, sem thread, sem
  binding. Precedente: TerminalGeometryOverlay atras de Loader (sem a env, o
  Loader nao instancia).
- O nucleo nao pode ter `if simulador_ativo` espalhado. Ou o modulo existe e
  responde, ou nao existe. Fronteira, nao condicional.
- Embarcados e simulacao entram como DOMINIOS (`sim/`, `target/`), no mesmo
  molde dos demais — nao como camada nova nem como excecao.
```

## 7. Nomes explícitos por responsabilidade

Exigência do autor, e ela vira regra verificável:

```text
- Arquivo diz O QUE FAZ, nao onde esta: EditorRequestRouter, nao EditorUtils2.
- Pasta = dominio (editor/, git/, sim/), nunca tipo de arquivo (helpers/, utils/).
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
- **Não** quebrar `terminal.rs` para cumprir métrica.
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
