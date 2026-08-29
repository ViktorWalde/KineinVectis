# Arquitetura de Software e Convenções de Crescimento

> **Classe: CONTRATO** (`docs/README.md`). Só muda por decisão explícita e
> registrada. As §2/§4/§5 são a lei; os inventários e números medidos são a parte
> perecível — e a §1.1 documenta o que aconteceu quando ninguém percebeu a
> diferença.
> **Status:** ativo / contrato de engenharia.
> **LEITURA OBRIGATÓRIA antes de escrever ou propor código novo — humano ou IA.**
> Não é referência de consulta: é contrato. Quem vai propor arquitetura neste
> repositório lê este documento **primeiro** e descobre que a resposta quase
> sempre já está aqui.
> **Função:** codificar como o código é organizado e como crescer sem precisar de
> outra refatoração massiva. A visão-alvo completa (produto/serviços) está em
> `docs/specs/KINEIN_VECTIS_INTERNAL_ARCHITECTURE_CORE_IPC_JOBS.md`; este documento
> é a ponte entre o que **já existe** e como **evoluir** até lá.

## 1. Por que este documento existe

Em 2026-07 o core, o protocolo e o CLI foram quebrados de arquivos-monólito
(`lib.rs` 2967 linhas, `lsp.rs` 1726, `protocol/lib.rs` 1162, …) para módulos por
responsabilidade. Foi uma refatoração grande que **não deveria ter sido
necessária**. Este documento existe para que o crescimento futuro já nasça
modular. A regra:

> Cada peça de código novo entra na camada certa, no módulo certo, com a
> visibilidade certa, e vira pasta/crate **antes** de virar monólito.

### 1.1 Este documento já falhou uma vez. Leia o porquê antes de confiar nele.

Dez dias depois de escrito, medição de 2026-07-16:

```text
lib.rs        505 linhas   a §4 manda: "fino. So o dispatch central e o estado".
handlers/lsp  653 linhas   a §4 manda: "fino. Sem logica pesada".
terminal.rs   955 linhas   1.9x o limite da propria §4.
Main.qml      700 linhas   a §6 afirmava "Estado validado em 2026-07-06: 336 linhas".
```

Sete arquivos do core e vinte da UI violavam as regras deste documento. Ele foi
escrito **exatamente** para impedir a volta do monólito e o monólito voltou. A
causa não foi a regra — a regra é boa. A causa foi mecânica:

> **Regra que mora só em `.md` não segura arquitetura. Ela apodrece em silêncio
> enquanto o gate fica verde.** "Ler antes de escrever código" era recomendação
> sem verificação, e recomendação perde para pressa toda vez.

O que mudou em 2026-07-16: `scripts/verificar-arquitetura.sh` passou a verificar
a §4 (core) e a §6 (UI) dentro do `verificar.sh`. Débito existente congelado em
`scripts/arquitetura-baseline.txt`, e **só pode diminuir**. Agora a regra tem
dente. **Se você está lendo isto para propor arquitetura nova: a proposta
provavelmente já está escrita abaixo, e o que faltava era cumprí-la.**

### 1.2 Manutenção: o que muda e o que não muda

Este documento **precisa** ser atualizado conforme arquivos, módulos e domínios
nascem — um mapa desatualizado engana mais do que a ausência de mapa (a §6 já
afirmou "Main.qml tem 336 linhas" enquanto ele tinha 700).

O que **muda**: inventário, números medidos, exemplos, nomes de módulos.
O que **não muda sem decisão explícita e registrada**: as camadas da §2, a regra
de split da §4/§6, a ordem da §5 e o caminho de crescimento da §6. Esses são os
fundamentos; se um deles atrapalha, a saída é discuti-lo, não contorná-lo em
silêncio.

### 1.3 As três âncoras: contra alucinação, dogmatismo e API imaginada

Decisão técnica aqui se apoia em **três** fontes, sempre juntas, e cada uma
corrige um vício diferente:

```text
1. ESTE DOCUMENTO + o codigo         -> contra ALUCINACAO DE ARQUITETURA.
   O que ja existe, medido, nao imaginado. Antes de propor,
   MEDIR: o problema costuma ser regra nao cumprida, nao regra ausente.

2. IDEs open source consolidadas     -> contra DOGMATISMO.
   Code OSS, IntelliJ IDEA Community, Zed, Lapce, Apache NetBeans.
   O que IDE profissional realmente faz, com revisao citada.
   Impede que "boa pratica" inventada vire lei local.

3. DOCUMENTACAO OFICIAL da linguagem -> contra API IMAGINADA.
   Rust (std/reference/clippy), Qt e QML, C++, CMake, POSIX.
   Comportamento de API se CONSULTA na fonte; nao se deduz do nome
   nem se lembra de cor. Versao/plataforma importam e mudam a resposta.
```

Nenhuma sozinha basta. Só o documento produz umbiguismo — o projeto repete os
próprios erros achando que são princípios. Só as referências produzem
importação de máquina alheia (DI runtime, host de extensões, Electron) que este
projeto recusou por decisão registrada. E sem a documentação oficial o código
compila e mente.

**A âncora 3 não é teoria.** Casos reais e caros deste repositório, todos
resolvidos por comportamento documentado que ninguém consultou antes:

```text
Qt.exit(256) sai como 0        Codigo de saida POSIX tem 8 bits. 7 dos 14
                               harnesses tinham checks que NUNCA reprovavam.
frameSwapped na render thread   Conexao queued mediria a fila de eventos junto
                               com o frame. Irrelevante em 250 ms, decisivo em 7.
QProcess::start e assincrono    sendRequest DESCARTA em silencio o que chega
                               antes de Running: o pedido nao falhava, sumia.
QQmlContext::objectForName      Existe desde Qt 6.5. Sem consultar, a saida seria
                               sujar o Main.qml com objectName so para medir.
Positioner descarta filho de    Linha vazia virava Row sem largura: o texto subia
largura zero                    e o cursor "parecia" errado. Dois dias de TUI.
`x: x` NAO resolve para o id    A propriedade do PROPRIO alvo vence a do objeto
quando `x` e' propriedade do    raiz do arquivo. `coreClient: coreClient` entregou
raiz (Qt 6.11.1, medido)        null em 15 roteadores: a IDE parou de ler pastas e
                                de criar projetos. Build passa, qmllint diz limpo,
                                o boot vai ao 1o frame e o Qt NAO avisa de loop.
                                (Se `x` E' um id do arquivo, resolve certo.)
Mutar CHAVE de `property var`   `terminalRenders[id] = render` nao notifica; so
nao notifica binding            reatribuir a propriedade notifica. E' por isso que
                                a UI desenha UMA sessao de terminal (a ativa), e
                                por que um painel lateral vivo exigiria uma
                                segunda vista notificante.
qmlcachegen emite `\r` CRU      O escape QML vira byte CR dentro do
dentro do QStringLiteral        `QStringLiteral` gerado; o preprocessador trata
(Qt 6.11.1)                     como fim de linha e o erro sai como "unterminated
                                argument list" 3800 linhas adiante. `\n` e'
                                escapado certo. Bug do Qt; medido no .cpp gerado.
```

Regra prática: ao afirmar que uma API se comporta de tal forma, **cite a fonte
e a versão**. Se a fonte não foi consultada, a frase correta é "não sei ainda" —
e a próxima ação é consultar ou medir, não supor.

A política de referência é obrigatória e tem modos definidos em
`docs/roadmaps/KINEIN_VECTIS_OPEN_PLUGIN_ADAPTATION_ROADMAP.md` §2 — **MODE-A**
(integrar a ferramenta original, preferido) a **MODE-D** (referência apenas).
Arquitetura entra como MODE-D/MODE-B: **aprende-se a regra, não se copia a
máquina**, e a revisão consultada fica registrada. Ver
`docs/arquitetura/27-modulos-por-dominio.md` para o exemplo aplicado.

## 2. Arquitetura em camadas (regra inviolável)

```text
┌─────────────────────────────────────────────┐
│ UI Qt/QML  — apresenta, interage, exibe estado │
└───────────────────────┬─────────────────────┘
                        │ JSON-RPC local (stdin/stdout) + eventos
┌───────────────────────▼─────────────────────┐
│ Rust Core  — roteamento, validação, estado    │
└───────────────────────┬─────────────────────┘
                        │ API interna de serviços
┌───────────────────────▼─────────────────────┐
│ Services   — workspace, fs, lsp, build, run… │
└───────────────────────┬─────────────────────┘
                        │ execução como jobs
┌───────────────────────▼─────────────────────┐
│ External Tools — cargo, cmake, clangd, fd…   │
└─────────────────────────────────────────────┘
```

> A UI apresenta, o Core decide, os Services executam, os Jobs registram, os
> Events notificam.

Proibições que sustentam a arquitetura (não negociáveis):

- a UI **nunca** chama ferramenta externa (cargo/cmake/clangd/fd) diretamente;
- a UI **nunca** manipula CMakePresets/Cargo.toml/arquivos sem passar pelo Core;
- a UI **não** contém regra de negócio nem parsing de saída de ferramenta;
- todo dado entre UI e Core passa por **tipos versionados** do `kinein-protocol`
  (nada de JSON solto montado à mão).

### 2.1 Referência externa não altera as camadas

Funcionalidades de IDE devem estudar implementações profissionais atuais
conforme a seção 2.1 de
`docs/roadmaps/KINEIN_VECTIS_OPEN_PLUGIN_ADAPTATION_ROADMAP.md`. Esse estudo importa
invariantes, decisões, modos de falha e estratégias de teste; não importa
código nem a arquitetura do host.

Uma solução observada em Code OSS, IntelliJ IDEA Community, Zed, Lapce ou
Apache NetBeans precisa ser redesenhada no fluxo nativo acima. Electron/Node,
Extension Host, IntelliJ Platform/Swing, GPUI, Floem e modelos internos dessas
IDEs não atravessam a fronteira como dependência implícita. Copiar função ou
fazer tradução mecânica também não é adaptação. O documento do domínio deve
registrar fonte/revisão, lições e a tradução **arquitetural** — em termos de
responsabilidade e contrato, nunca de texto de implementação — para as camadas
da Kinein.

## 3. Mapa de crates atual

| Crate | Responsabilidade |
| --- | --- |
| `kinein-protocol` | contratos UI↔Core: requests, responses, eventos, erros. Um módulo por domínio, re-exportado flat (`kinein_protocol::TipoX`). |
| `kinein-core` | dispatch JSON-RPC, estado da sessão e os serviços de domínio. É o cérebro; é o mais testável. |
| `kinein-config` | modelo de configuração strict-by-default. |
| `kinein-cli` | lib (`kinein_cli`) que gera requests JSON-RPC + binário fino. |
| `ui/` (C++/Qt) | frontend Qt/QML; sobe o `kinein-core` como processo filho via `CoreClient`. |

Esta é a base mínima. A visão-alvo (specs) prevê ~18 crates; a Seção 6 descreve
como chegar lá **sem big-bang**.

## 3.1. Organização da UI Qt/QML

A UI segue a mesma regra anti-monólito do core. A fase descrita em
`docs/arquitetura/17-architecture-hygiene-plan.md` eliminou as concentrações conhecidas em
2026-07-06; a regra permanente é não aceitar "dívida pequena" quando ela já é
uma concentração conhecida.

Fluxo obrigatório:

```text
QML visual -> controller/store QML -> CoreClient facade -> handler IPC interno -> Rust core
```

Regras:

1. **`Main.qml` é composition root.** Ele instancia janela, `CoreClient`,
   controllers, roteadores e layout. Não recebe `ListModel`, `Connections`,
   timers, parsing, estado de domínio nem helpers que conheçam domínio.
2. **Componente visual é burro.** Recebe dados por `property`, expõe ações por
   `signal` e não chama ferramenta externa, filesystem ou core diretamente.
3. **Controller/store QML guarda estado de UI.** Ele pode coordenar modelos,
   timers e intenção de IPC, mas não executa regra de negócio nem parsing de
   saída de ferramenta.
4. **Roteador IPC QML só despacha evento.** Eventos de `CoreClient` ficam em
   `ui/qml/ipc/<Dominio>EventRouter.qml` e chamam o controller certo.
5. **`CoreClient` é fachada única.** Não criar `CoreClient2` nem clientes QML
   paralelos. A implementação C++ deve ser dividida internamente por domínio
   quando crescer.
6. **Regra de split da UI.** Arquivo QML visual acima de ~300 linhas,
   controller/store acima de ~400 linhas ou qualquer arquivo que combine
   renderização + estado + IPC deve ser quebrado antes de nova feature crescer
   em cima dele.

   **Verificada pelo gate desde 2026-07-16** (`scripts/verificar-arquitetura.sh`,
   dentro do `verificar.sh`). Regra que mora só em `.md` não segura
   arquitetura: esta existia, era boa, e ninguém a checava — em dez dias o
   `Main.qml` foi de 336 para 700 linhas e 20 arquivos passaram do limite.
   A verificação é **catraca**, não limite duro: o débito existente fica
   congelado em `scripts/arquitetura-baseline.txt` e só pode diminuir.
   Arquivo novo acima do limite reprova; arquivo em débito que cresce
   reprova; encolher é sempre aceito. Falhar nos 20 de uma vez só ensinaria
   a desligar o script.

**Inventário: parte perecível deste documento.** A separação de camadas acima é
CONTRATO e não muda por conveniência. Os números abaixo são ESTADO e envelhecem —
a fonte viva é `scripts/arquitetura-baseline.txt`, mantido pela catraca. Este
bloco já mentiu duas vezes (ver §1.1 e o `arquitetura/17`); ele existe para dar
forma, não para ser citado como verdade de hoje.

Medição de 2026-07-17 (o que a catraca cobrou e o que foi pago):

```text
PAGO desde 2026-07-16, e cada um por RESPONSABILIDADE, nao por tamanho:
  Main.qml                700 -> 270   dominios sairam para ui/qml/app/AppDomains.qml
  BottomPanelHost.qml     541 -> 383   a barra de chips virou TerminalSessionTabs.qml
  RuntimeController.qml   494 -> 309   run configs sairam; o conceito de IA saiu
  AppMenuBar.qml          303 -> 292   o icone saiu de dentro da IDE

EM ABERTO — 23 arquivos no baseline. Os maiores bloqueiam a propria area:
  EditorController.qml   1070 (400)    ja tem subcontrollers: continuar movendo
  terminal.rs             955 (500)
  editor_highlighter.cpp  910 (500)
  core_client_dispatch.cpp 804 (500)   a §5 ja manda dividir por dominio
  ShellWorkspaceHost.qml  576 (400)    composition host: cortar por area
```

O `CoreClient` preserva a API QML única, com a implementação C++ fatiada em
processo, requests, dispatch, estado e logs; o editor divide documentos, texto e
completion em subcontrollers. Isso continua valendo e é o alvo.

## 4. Organização interna do `kinein-core` (o que impede o monólito)

```text
kinein-core/src/
├── lib.rs          # SÓ: struct Core, dispatch, RequestOutcome, erro do loop
├── runtime.rs      # loop stdio JSON-RPC
├── rpc.rs          # helpers de response/erro/parse de params
├── commands.rs     # descriptors de command.list
├── handlers/       # roteadores por domínio (impl Core) — finos: parse + delega
├── <dominio>.rs    # lógica de domínio pequena (arquivo único)
├── <dominio>/      # lógica de domínio grande (pasta: mod.rs + submódulos)
└── tests/          # testes de integração, um arquivo por domínio
```

Regras que mantêm isso saudável:

1. **`lib.rs` é fino.** Só o dispatch central e o estado. Lógica de domínio nunca
   entra aqui.
2. **`handlers/<dominio>.rs` é fino.** Roteia `<dominio>.*`, valida params,
   delega para o serviço de domínio, formata a resposta. Sem lógica pesada.
3. **A lógica vive no módulo de domínio.** Começa como arquivo `<dominio>.rs`.
4. **Regra de split (a mais importante).** Quando um arquivo passa a **misturar
   mais de uma responsabilidade** OU cresce além de **~400–500 linhas de código
   fora dos testes**, quebre em pasta `<dominio>/` com `mod.rs` + um submódulo
   por responsabilidade. Precedente já no código:
   - `lsp/` → `types`, `manager`, `server`, `framing`, `parse`, `edit`, `uri`;
   - `fsops/` → `error`, `confine`, `ops`, `search`, `find`;
   - `workspace/` → `error`, `detect`, `open`, `create`.
   O `mod.rs` só declara submódulos, re-exporta a API pública e guarda os
   aliases/constantes compartilhadas.
5. **Visibilidade.** Dentro de uma pasta-módulo: itens internos usados entre
   submódulos irmãos usam `pub(super)` (não `pub(crate)`, que o clippy `nursery`
   rejeita como redundante; não `pub`, que o `unreachable_pub` rejeita). Só a API
   real re-exportada pelo `mod.rs` é `pub`. Em módulo público, `pub(crate)` é ok.
   Um binário que precisa de módulos internos ganha um **lib target** (ver
   `kinein-cli`).
6. **Testes.** Unitários co-localizados (`#[cfg(test)] mod tests` no próprio
   arquivo). Integração em `tests/<dominio>.rs`, com helper compartilhado no
   `tests/mod.rs`.
7. **Lints estritos são inegociáveis** (`unsafe` forbid, warnings/pedantic/nursery
   deny, sem `unwrap/expect/panic` fora de teste). Eles são parte do design.
8. **Como os limites acompanham o crescimento do projeto** (decisão do autor,
   2026-07-16: a catraca não pode impedir o projeto de crescer).

   > O limite é **por responsabilidade** e **não cresce**. O projeto cresce
   > somando unidades, não engordando unidades.

   Um controller que faz uma coisa não precisa de mais linhas porque o projeto
   ficou maior — precisam existir **mais** controllers. Um projeto com 12
   domínios tem 12 arquivos de domínio, não um arquivo de 1000 linhas. É isso
   que faz a regra escalar: o que cresce é a **contagem** de módulos, não o
   tamanho de cada um. Quando um domínio novo nasce (embarcados, simulação), ele
   traz arquivos novos e a catraca nem percebe — porque nascem dentro do limite.

   **A exceção legítima é o composition root**, cujo tamanho é função do número
   de domínios, não da qualidade do código. Aí a saída é dividir a composição
   por área (`domínios/`, `hosts/`, `atalhos/`), fazendo a contagem de arquivos
   crescer em vez do tamanho — nunca subir o limite. Ver
   `docs/arquitetura/27-modulos-por-dominio.md` §4.1.

   **Subir um limite é permitido — e é decisão explícita, registrada e
   justificada, nunca silenciosa e nunca "porque incomodou hoje".** Se um limite
   está errado, o caminho é discuti-lo e registrar o porquê no ADR/documento do
   domínio; contorná-lo em silêncio é o vício que a §1.1 documenta.

9. **O critério é RESPONSABILIDADE. Tamanho é sintoma, não regra**
   (decisão do autor, 2026-07-16).

   > A catraca mede linhas porque é o que um script consegue medir. Ela é
   > **detector de fumaça, não o incêndio.** Quando dispara, a pergunta certa é
   > *"que responsabilidade está misturada aqui?"* — nunca *"como corto linhas
   > até passar?"*.

   O "OU" da regra 4 ("mistura responsabilidade **OU** passa de ~400–500
   linhas") não faz do tamanho um critério independente: ele é o gatilho
   automatizável que manda **ir olhar**. Quem decide o corte é a
   responsabilidade, e o corte é **pragmático** — se não deixa o código mais
   claro para quem vai ler, não é split, é cerimônia.

   Os três casos medidos em 2026-07-16/17 — a catraca disparou nos três, e o
   diagnóstico certo foi diferente em cada um:

   ```text
   RuntimeController 414   catraca CERTA, e o alvo era o ARQUIVO. O tamanho
                           apontava mistura real: terminais + Assistente + run
                           configs. Corte por responsabilidade (494 -> 397).

   AppDomains 325          catraca ERRADA: erro de CATEGORIA. O arquivo faz UMA
                           coisa (compor dominios). Quebra-lo por tamanho gerou
                           13 propriedades de pass-through — nada ficou mais
                           claro e "onde X e ligado" passou a ter duas
                           respostas. 300 e limite de QML visual, e composicao
                           nao e visual. Corrigiu-se a categoria, nao o arquivo.

   ShellWorkspaceHost 583  catraca CERTA, e o alvo era a MUDANCA. Ela disparou
                           por +1 linha de fiacao legitima num arquivo gordo por
                           motivos antigos. O defeito nao estava no arquivo (e
                           composition host: split e fatia propria) nem na
                           categoria — estava no diff, que punha politica de KV
                           Context num host visual. Devolvida ao dono, o arquivo
                           caiu para 579 sem ninguem "cortar linhas".
   ```

   **Quando a catraca dispara, há três suspeitos, nesta ordem: a sua mudança, a
   categoria, o arquivo.** O reflexo é olhar só o terceiro — e foi o terceiro que
   errou em dois dos três casos. Antes de quebrar nada, pergunte se o que você
   está *acrescentando* pertence ali.

   **O teste de um corte por responsabilidade não é o número — é o vocabulário.**
   Depois de mover o Assistente para fora, `grep -i assistant` no
   `RuntimeController.qml` não devolve nada — nem `grep -i context`, como o
   domínio se chamava até 2026-07-17. O arquivo perdeu o **conceito**, não só as
   linhas: ele nem sequer tem uma palavra para nomeá-lo, porque o que recebeu no
   lugar foi um mecanismo genérico (`openLabeledTerminal(label, kind)`, com o
   `kind` opaco). Se o arquivo ainda nomeia o domínio que você diz ter extraído,
   você moveu código e manteve a responsabilidade — o número desceu e o
   acoplamento ficou. Um corte que sobrevive a esse teste quase nunca precisa de
   justificativa de tamanho.

   **Consequência prática.** Arquivo acima do limite que faz **uma coisa só** não
   deve ser quebrado: ou a categoria está errada (corrija-a, explicitamente), ou
   o débito fica congelado até existir um corte que melhore a leitura. Ficar
   acima do limite fazendo uma coisa é melhor do que ficar abaixo fazendo
   ginástica. E as duas saídas fáceis de um débito que cresce são trapaça:
   **cortar uma linha qualquer para caber** e **subir o baseline** — a primeira é
   cerimônia, a segunda é a §1.1 se repetindo. **Os dois extremos se evitam:**
   monólito que cresce calado, e desacoplamento inútil para satisfazer um número.

10. **A regra 4 é verificada por catraca desde 2026-07-16.**
   `scripts/verificar-arquitetura.sh` (dentro do `verificar.sh`) conta as linhas
   **fora dos testes** — o corte é o `#[cfg(test)]`, para não punir quem testa
   junto — e reprova arquivo novo acima de 500 ou arquivo em débito que cresça.
   Não é limite duro: o débito existente fica congelado em
   `scripts/arquitetura-baseline.txt` e **só pode diminuir**.

   Até essa data a catraca varria só a UI, e esta seção nunca havia sido
   verificada por ninguém: **7 arquivos já a violavam**, dois deles contrariando
   não o tamanho, mas a responsabilidade que as regras 1 e 2 lhes atribuem por
   escrito (`lib.rs` com 505 linhas; `handlers/lsp.rs` com 653). Regra que mora
   só em `.md` não segura arquitetura — apodrece em silêncio enquanto o gate
   fica verde. Plano de pagamento e ordem em
   `docs/arquitetura/27-modulos-por-dominio.md`.

11. **Todo gate deste projeto nasceu de uma falha SILENCIOSA, e essa é a regra
    para criar o próximo.** Não se cria gate por gosto de rigor: cria-se quando
    uma classe de erro passa verde por todos os checks existentes. O critério é
    esse — *"o que pode quebrar sem nada reclamar?"*.

    ```text
    verificar-arquitetura.sh   2026-07-16  a regra de split morava so em .md.
      (catraca)                            20 arquivos da UI e 7 do core a
                                           violavam com o gate verde.
    verificar-qml-fiacao.sh    2026-07-17  `coreClient: coreClient` entregou null
      (binding auto-referente)             em 15 roteadores. Build passa, qmllint
                                           limpo, boot vai ao 1o frame, o Qt nao
                                           avisa. A IDE parou de ler pastas.
    verificar-docs.sh          2026-07-17  numero sem data que mente. O
      (veracidade dos .md)                 arquitetura/17 dizia "concluido para o
                                           estado atual" com numeros 3,4x errados
                                           — o MESMO vicio da §1.1, dez dias
                                           depois de a §1.1 ser escrita sobre ele.
    verificar-qml-logica.sh    (anterior)  harness headless dos controllers. Em
                                           2026-07-16 descobriu-se que 7 dos 14
                                           NAO conseguiam reprovar: `Qt.exit()`
                                           trunca em 8 bits (§1.3).
    verificar-transicao-        2026-08-29  estado por-workspace do `Core`
      workspace.sh                          trocado em 3 caminhos, cada copia
                                            esquecendo uma peca diferente. O
                                            `workspace.createProject` nao trocava
                                            `self.drafts`: o autosave do projeto
                                            NOVO ia para o banco do ANTERIOR.
                                            Build, clippy e 271 testes verdes.
    ```

    **Um gate que nunca reprovou não está provado — está sem evidência.** Ao
    criar ou mexer em um, MUTE o produto e confirme que ele cai. Em 2026-07-17 a
    primeira versão do `verificar-docs.sh` deixava passar "42 linhas" porque o
    regex exigia 3 dígitos: cega para arquivo pequeno, e só o teste de mutação
    mostrou. Vale para teste também — a §0.2i do `PONTO_ATUAL` existe porque uma
    suíte inteira passava verde com o bug presente.

    **E gate que grita falso é pior que gate nenhum: ensina a ignorar.** Por isso
    o `verificar-docs.sh` entende que uma seção "## Resultado 2026-07-06" data
    tudo dentro dela, e a catraca corta no `#[cfg(test)]`.

O `kinein-protocol` segue a mesma ideia: **um módulo por domínio** (`rpc`,
`workspace`, `fs`, `lsp`, `build`, …) re-exportado flat pelo `lib.rs`. Um tipo
novo entra no módulo do seu domínio, não num arquivo gigante.

## 5. Onde colocar código novo (guia de decisão)

Ao adicionar um comando/feature, siga sempre esta ordem:

```text
1. Tipo(s) de contrato → kinein-protocol/src/<dominio>.rs (params + result + evento)
2. Roteamento          → kinein-core/src/handlers/<dominio>.rs (parse + delega)
3. Lógica              → kinein-core/src/<dominio>.rs  (ou .../<dominio>/ se já for grande)
4. Testes              → unit no módulo + integração em tests/<dominio>.rs
5. Se for operação longa → vira JOB (ver Seção 7), não handler síncrono
6. Doc                 → atualizar docs/arquitetura/03-ipc-protocol.md (contrato) e ContextoIA.md (estado)
```

Se o domínio ainda não existe, crie o par `handlers/<dominio>.rs` +
`<dominio>.rs`. Não pendure método novo num domínio que não é o dele.

## 6. Caminho de crescimento até a arquitetura-alvo (sem big-bang)

Os specs preveem serviços dedicados (workspace, project, toolchain, cmake, cargo,
language, build, run, debug, terminal, target, settings, storage — o
`ai-bridge` saiu: fora de escopo desde 2026-07-17) e,
eventualmente, um crate por serviço (`kinein-cmake`, `kinein-cargo`,
`kinein-language`, …) + `apps/kinein-ui` e `apps/kinein-core-daemon`.

**Não** criar essa estrutura toda agora. A progressão é sempre incremental e
mecânica (baixo risco, como foi o rename):

```text
função  →  arquivo <dominio>.rs  →  pasta <dominio>/  →  crate kinein-<dominio>
```

- Um serviço novo **nasce como pasta-módulo** em `kinein-core/src/` (ex.:
  `src/cmake/`), já dividido por responsabilidade.
- Só vira **crate próprio** (`kinein-cmake`) quando (a) fica grande, (b) é
  independente o suficiente e (c) há ganho real (reuso, tempo de compilação,
  fronteira clara). A extração de crate é uma passada mecânica — nunca um
  big-bang.
- O nome do crate futuro já é conhecido (specs), então **nomeie a pasta-módulo
  igual** desde o início (`src/cmake/`, `src/toolchain/`, `src/language/`), para
  a extração ser trivial.

## 7. Jobs, Events e Risk — construir cedo (ponto crítico anti-refatoração)

Os specs exigem um **Job System**: toda operação longa (configure, build,
index, scan, flash, debug, geração de contexto de IA) é assíncrona, cancelável e
reporta progresso por eventos; nunca bloqueia a UI.

> **Armadilha a evitar:** se continuarmos adicionando handlers **síncronos** para
> operações longas, introduzir jobs depois será exatamente a refatoração massiva
> que este documento existe para prevenir.

Portanto:

- a abstração de **Job/Event** deve ser generalizada **antes** de adicionar os
  próximos serviços longos (cmake configure/build, cargo, debug, targets). O
  `build.run`/`test.run`/`quality.run` (que já fazem streaming via `emit`) são o
  embrião — generalizar em vez de duplicar por comando;
- todo job tem `id`, estado (`queued/running/…/success/failed/cancelled`),
  eventos e logs; job cancelável expõe cancel;
- **eventos não viram pop-up automático** — atualizam status bar, Problems, tool
  window; painel só abre se o usuário pedir;
- todo comando é **classificado por risco** (`low/medium/high/dangerous`);
  `high` exige confirmação, `dangerous` é bloqueado ou exige confirmação muito
  explícita; ação vinda de IA nunca é aplicada automaticamente;
- erros são **estruturados** (código estável + mensagem + detalhes), como já são
  hoje no protocolo.

### 7.1. Estado do disco e buffers do editor

O disco é uma fonte externa concorrente; a UI nunca presume que o snapshot de
uma aba ainda é atual. O domínio `fswatch` observa apenas a raiz e diretórios
alcançados por `fs.list`/`fs.read`, sem varredura recursiva global. Ele publica
eventos tipados e debounced; routers de UI os distribuem para editor, árvore e
Git sem colocar lógica de conflito no `CoreClient`.

O watcher é aviso antecipado, não a barreira final. Todo `fs.write` de usuário
leva `expectedContent`, e `fsops` compara com o disco antes da escrita atômica.
Se divergir, o core retorna `FILE_CHANGED` e não escreve. Só uma decisão
explícita da UI atualiza o snapshot esperado para permitir sobrescrever a versão
externa. Operações de workspace edit do LSP devem convergir para o mesmo modelo
transacional quando ganharem preview/rollback.

## 8. Anti-padrões (o que causou a dívida — proibido repetir)

```text
- lib.rs / arquivo de domínio virando monólito multi-responsabilidade;
- God object no lado Qt (CoreClient / Main.qml acumulando tudo);
- UI chamando ferramenta externa ou mexendo em arquivos direto;
- regra de negócio ou parsing de saída de ferramenta na UI;
- operação longa rodando síncrona no handler (deveria ser job);
- JSON montado à mão em vez de tipo do kinein-protocol;
- método pendurado no domínio errado;
- relaxar strict mode sem registrar motivo;
- mecanismo GENÉRICO sem usuário: pior que nenhum. Quando o único chamador morre,
  o mecanismo morre junto (2026-07-17: com a IA fora de escopo, o
  `openLabeledTerminal`/`kind` do RuntimeController ficou órfão e saiu);
- doc de ESTADO afirmando número sem data — vira mentira em silêncio (§1.1, e o
  `verificar-docs.sh` agora reprova);
- ligar/escutar no lugar errado: não falha no build, **deixa de funcionar em
  silêncio**. Já custou três fatias (`runConfigController` null, `onAgentChosen`
  no controller errado, `coreClient: coreClient` em 15 roteadores).
```

## 9. Critérios de aceite (checklist arquitetural)

Uma mudança está arquiteturalmente saudável quando:

```text
[ ] UI não executa ferramenta externa direto; tudo passa por JSON-RPC tipado.
[ ] lib.rs e handlers/ continuam finos; a lógica ficou no serviço de domínio.
[ ] arquivo que cresceu/misturou responsabilidade virou pasta-módulo.
[ ] visibilidade correta (pub(super) interno; pub só na API re-exportada).
[ ] operação longa é job assíncrono, cancelável, com eventos.
[ ] comando classificado por risco; high/dangerous confirmam.
[ ] erro estruturado; nada de unwrap/expect/panic fora de teste.
[ ] testes unit co-localizados + integração por domínio.
[ ] o teste/gate novo REPROVA de verdade: mutei o produto e ele caiu (regra 11).
[ ] contrato novo documentado em docs/arquitetura/03; decisão registrada no
    ContextoIA.md (que e' LOG datado, nao o estado).
[ ] GUIAIA.md atualizado se módulo/domínio/router nasceu, mudou de nome ou morreu
    — mapa desatualizado engana mais que ausência de mapa (§1.2).
[ ] mexeu na UI? `cmake --build --preset release-hardened` ANTES de pedir
    validação: o atalho de desenvolvimento roda o release, não o dev-local.
[ ] pasta-módulo de serviço já nomeada como o crate-alvo dos specs.
[ ] funcionalidade de IDE registra referência oficial atual, invariantes e
    adaptação própria; nenhum código/runtime do host foi transplantado.
```

E o critério que nenhum checklist pega, aprendido caro em 2026-07-17: **rigor
arquitetural não torna útil uma feature que não se paga.** O seletor de agente de
IA passou em todo item desta lista — camadas certas, zero política no core,
mutação, gesto real — e foi removido no mesmo dia porque o usuário já podia
digitar `claude` no terminal. Antes da fatia, pergunte o que ela substitui e
quanto isso custava. Só o uso responde; responder cedo é barato.

Referência da visão completa: `docs/specs/` (o ALVO, que diverge por natureza).
**O que existe hoje se mede no código** — nenhum documento derruba uma medição.
Classes de volatilidade dos documentos: `docs/README.md`.
