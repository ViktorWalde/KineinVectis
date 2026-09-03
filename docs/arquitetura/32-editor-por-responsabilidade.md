# O editor cortado por responsabilidade

> **Classe: ESTADO** (`docs/README.md`). Descreve o que o código faz **hoje**.
> Todo número foi medido em **2026-09-02**, com o gate completo verde. Se
> divergir do código, o código vence e este documento se corrige no mesmo gesto.
>
> **Para que serve:** registrar a etapa 6 do
> [`../roadmaps/30-caminho-para-o-mvp.md`](../roadmaps/30-caminho-para-o-mvp.md)
> — o pagamento do maior débito do repositório — de forma que a próxima sessão
> saiba **onde cada coisa mora e por quê**, e não desfaça o corte por não
> entendê-lo. E registrar, com números, a **decisão que ficou em aberto**.

## 1. O que era, medido

**Medido em 2026-09-02, antes do corte:** `ui/qml/editor/EditorController.qml`
tinha **1.070 linhas contra um limite de 400** — o maior item da catraca, e o
único que bloqueava por área:

> *"O maior débito bloqueia por área, não em geral. `EditorController.qml`
> (1.070/400) bloqueia qualquer feature de editor. Quem toca a área, paga a dela
> antes."* — `LEITURA_TECNICA` §4

Ele **já tinha quatro subcontrollers** (`EditorDocumentController`,
`EditorTextController`, `EditorCompletionController`, `EditorFindController`),
então a leitura fácil — *"é só continuar quebrando"* — não explicava o tamanho.
A leitura correta veio de olhar o que ele **implementava**, e não o que ele
delegava:

```text
23 sinais    12 eram da camada de linguagem (LSP)
97 funcoes   26 eram da camada de linguagem
 6 timers     2 eram de realce, 2 de persistencia, 1 de format
 2 models     os dois eram da camada de linguagem
```

A camada de linguagem inteira — definition, hover, usos, code actions, rename,
WorkspaceEdit, semantic tokens e Tree-sitter — **não tinha dono**. Morava
inline, num arquivo cuja responsabilidade declarada era compor o editor.

## 2. O critério do corte, e por que ele não foi o tamanho

A `ARCHITECTURE.md` §4 regra 9 é explícita:

> *"A catraca mede linhas porque é o que um script consegue medir. Ela é
> detector de fumaça, não o incêndio. Quando dispara, a pergunta certa é 'que
> responsabilidade está misturada aqui?' — nunca 'como corto linhas até
> passar?'."*

E o teste de um corte **não é o número, é o vocabulário**: depois de extrair um
domínio, o arquivo de origem tem de perder o **conceito**, não só as linhas.

Foram quatro cortes, cada um com uma pergunta própria que ninguém mais no
sistema responde:

| Arquivo novo | A pergunta que ele responde |
| --- | --- |
| `EditorLanguageController.qml` | *O que o servidor sabe sobre o **símbolo sob o cursor**, e o que a IDE mostra com isso?* |
| `EditorHighlightController.qml` | *Como o **documento inteiro** fica pintado, e qual resposta ainda vale?* |
| `EditorFormatController.qml` | *Formatar — e o que o format-on-save faz com o **salvar**?* |
| `EditorPersistenceController.qml` | *O que o editor **lembra** quando a IDE fecha?* |

### 2.1 Por que linguagem e realce viraram DOIS arquivos, e não um

O primeiro corte produziu um `EditorLanguageController` de **452 linhas** — e a
catraca cobrou de novo, com razão. A pergunta da regra 9 tinha resposta clara:
havia duas responsabilidades ali dentro, separadas por **ritmo**.

```text
simbolo sob o CURSOR    pontual. Nasce de um GESTO do usuario (F12, Ctrl+Q,
                        Alt+Enter). Responde abrindo popup ou dialogo.
                        -> EditorLanguageController

realce do DOCUMENTO     continuo. Persegue CADA TECLA digitada. Responde
                        repintando o texto inteiro, e precisa descartar
                        resposta obsoleta.
                        -> EditorHighlightController
```

Juntá-las foi o que fez o arquivo passar do limite **mesmo depois** de sair do
`EditorController`. O sinal de que o corte estava certo é que ele resolveu o
tamanho sem ninguém mirar no tamanho.

## 3. Os dois relógios — a invariante mais fácil de quebrar em silêncio

`syntaxVersion` e `semanticVersion` sobem a **cada pedido**, e toda resposta é
descartada se a versão não bater — ou se o caminho do documento mudou.

Isso existe porque as duas camadas respondem em tempos diferentes sobre o mesmo
buffer: o Tree-sitter é local e reparseia em milissegundos; o servidor LSP pode
levar segundos. Aplicar *"a última que chegou"* pintaria realce de um texto que
já não está na tela.

> **Não é precedência por tempo de chegada. É identidade de documento/versão.**

A composição visual é outra coisa, e vive no highlighter C++:
`regex fallback < Tree-sitter < semantic tokens < diagnósticos/busca`.

**Por que isso é perigoso:** quebrar o relógio não quebra build, não acende
`qmllint` e não derruba teste nenhum que existisse antes. Aparece como *cor
errada* — o defeito que o usuário não sabe reportar. É a definição de falha
silenciosa da `ARCHITECTURE.md` §4 regra 11, e é por isso que a etapa entregou
harness junto com o corte.

## 4. Format-on-save: estado pendente precisa de dono

O gesto do usuário é um só (`Ctrl+S`), mas o caminho tem **duas pernas**:

```text
Ctrl+S ─► formata ─► resolved ─► salva
             └────► failed   ─► salva ASSIM MESMO
```

A segunda perna é a que importa. Se o formatter não existe, morreu ou deu
timeout, o `format.text` volta em `requestFailed` — e o `Ctrl+S` do usuário
**não pode ficar preso** esperando um servidor que não vem.

> Formatar é conveniência. Salvar é dado.

`pendingSaveAfterFormat` é consumido nos **dois** caminhos, e é justamente por
isso que precisa de um dono que os enxergue juntos: espalhados por dois `if` no
meio do salvar, os dois braços divergem na primeira manutenção.

O "salvar tudo" com format-on-save é uma **fila**, não um laço: cada
`format.text` é um round-trip ao core e a resposta chega por sinal. Por isso
`saveCurrentFile`/`saveAllFiles` recusam começar com fila em andamento —
reentrar no meio duplicaria escrita.

## 5. Sessão e rascunho: parecem a mesma coisa e não são

```text
SESSAO     quais arquivos estavam abertos, e qual estava ativo.
           Sobrevive a um fechamento NORMAL. E conveniencia.
           -> .kinein/session.json, via `workspace.saveSession`

RASCUNHO   o TEXTO nao salvo de um buffer sujo.
           Sobrevive a um CRASH — e SO a um crash: salvar limpa o rascunho,
           porque o disco passou a ser a verdade. E rede de seguranca de DADO
           (docs/seguranca/23, pilar 2).
           -> .kinein/kinein.db (SQLite/WAL), via `draft.save`
```

O que as une é o **gesto**: as duas gravam depois que o usuário para, por
debounce, e as duas são relidas na abertura do workspace. Por isso um dono só.

**A ordem do restore é carga estrutural.** Os arquivos são pedidos com o
**ativo por último**, porque o core responde na ordem em que recebe
(`04-boot-e-comunicacao.md` §6, a única garantia de ordem que existe) e cada
`fileLoaded` seleciona a própria aba. Inverter isso reabre o projeto na aba
errada, e nada reclama — o usuário só nota que *"a IDE nunca lembra onde eu
estava"*.

## 6. Onde os roteadores IPC passaram a escutar

Os sinais de pedido **saíram** do `EditorController` junto com a implementação.
Escutar no objeto errado não quebra o build: o handler simplesmente nunca
dispara — a falha silenciosa que a `ARCHITECTURE.md` §8 registra como tendo
custado três fatias a este repositório.

Por isso o `EditorRequestRouter` tem agora **três blocos, e três é a
informação**:

```text
target: editorController            arquivo, rascunho, format, sessao,
                                    notificacao de mudanca, completion
target: editorController.language   definition, hover, references, rename,
                                    code actions, WorkspaceEdit, switch header
target: editorController.highlight  semantic tokens, arvore sintatica
```

O mesmo vale para o diálogo de rename, que é aberto pela **camada de
linguagem** (o gesto nasce de um `textDocument/rename`) e não pelo editor — o
`AppDomains.qml` escuta lá, explicitamente.

## 7. O resultado, medido

```text
                                    ANTES    DEPOIS
ui/qml/editor/EditorController.qml   1070       791
ui/qml/editor/EditorLanguageController.qml  —   376   (novo)
ui/qml/editor/EditorHighlightController.qml —   161   (novo)
ui/qml/editor/EditorFormatController.qml    —   203   (novo)
ui/qml/editor/EditorPersistenceController.qml — 138   (novo)
```

**279 linhas de implementação** saíram para donos com fronteira escrita. Os
quatro arquivos novos nascem abaixo do limite e com teste.

### 7.1 As provas, e o que elas custaram para existir

Nenhum dos quatro tinha teste — e refatoração sem teste é mudança com os olhos
fechados. Dois harnesses novos, **provados por mutação**:

```text
tst_editor_language.qml
  relogio semantico para de descartar resposta obsoleta  -> cai
  sem documento, o realce anterior NAO e apagado         -> cai
  rename deixa de checar as outras abas sujas            -> cai

tst_editor_persistence.qml
  o restore pede o ATIVO primeiro                        -> cai
  rascunho passa a persistir buffer LIMPO                -> cai
```

**A quinta mutação não caiu na primeira tentativa, e isso é o achado do dia.**
O harness checava a ausência do rascunho aos **700 ms** — antes de o debounce de
**1.500 ms** poder disparar. A checagem acontecia cedo demais para poder falhar:
verde por construção, exatamente o que a §4 regra 11 chama de *"gate sem
evidência"*. Corrigido o prazo para 1.900 ms, a mutação cai.

> Checagem que acontece antes de o efeito ser possível não é checagem. É ruído
> verde.

## 8. A decisão que estava em aberto — RESPONDIDA em 2026-09-03

> **DECIDIDA pelo autor em 2026-09-03: saída (a), a terceira fatia.** Corta-se
> o `ShellWorkspaceHost.qml` PRIMEIRO; só depois se reavalia a fachada. Nenhum
> limite foi levantado — a §4 regra 8 não foi acionada. O registro da decisão,
> com a medição que a sustenta, está na §8.4.
>
> Esta era a etapa 11 de
> [`roadmaps/34-depois-do-mvp.md`](../roadmaps/34-depois-do-mvp.md) §3.2 — a
> primeira da fila do pós-MVP, porque era pergunta ao autor e bloqueava qualquer
> fatia que tocasse o editor. **Deixou de bloquear.**

`EditorController.qml` continua na catraca, em **791/400**. Isto não é omissão —
é uma decisão que não é da sessão, e a §4 regra 8 é clara: *"subir um limite é
decisão explícita, registrada e justificada, nunca silenciosa"*.

### 8.1 O que sobrou, medido

```text
97 funcoes no arquivo
64 delas sao DELEGACAO PURA de uma linha (documents.x(), language.y(), ...)
```

Depois dos quatro cortes, o que resta é **o composition root do editor mais uma
fachada única**: ele instancia os sete subcontrollers, orquestra o que cada um
faz a cada tecla digitada, e **re-exporta tudo** para que o resto da aplicação
tenha um interlocutor só.

O teste de vocabulário, aplicado com honestidade, **falha** para este arquivo:
ele já não *implementa* hover, realce, format ou sessão, mas continua *nomeando*
os quatro — porque é fachada.

### 8.2 As duas saídas, com o custo de cada uma

**(a) Quebrar a fachada.** Expor os subcontrollers por nome
(`editorController.find.openFind()`, `editorController.language.requestHover()`)
e reescrever os chamadores. Custo medido:

```text
~180 call sites em 13 arquivos
 92 deles em ui/qml/shell/ShellWorkspaceHost.qml — que esta ELE PROPRIO na
    catraca, em 576/400, e ficaria mais longo
```

O ganho é duvidoso: `ShellWorkspaceHost` é um **host visual**, e trocar
`editorController.openFind()` por `editorController.find.openFind()` ali é mais
digitação sem mais entendimento. A `ARCHITECTURE.md` §4 regra 9 avisa
exatamente contra isso — *"split que não deixa mais claro não é split, é
cerimônia"* —, e o precedente do `AppDomains.qml` mostra o estrago: espremê-lo
gerou "13 propriedades de pass-through" e fez *"onde X é ligado"* passar a ter
duas respostas.

**(b) Corrigir a categoria.** Reconhecer que este arquivo é um **composition
root**, cujo tamanho é função de *quantas preocupações o editor tem*, e não da
qualidade do código — o mesmo diagnóstico que corrigiu a categoria do
`ui/qml/app/` na catraca. O contra é honesto e forte: `AppDomains` estava em
325 contra 300 (um quase-acerto); este está em **791 contra 400**, quase o
dobro. Nenhum limite defensável cobre isso.

### 8.3 A recomendação desta sessão, e ela não decide nada

Nem (a) nem (b) puros. O que a medição sugere é uma **terceira fatia própria**,
que não é a etapa 6:

> **O problema real não é o `EditorController` re-exportar; é o
> `ShellWorkspaceHost` precisar de 92 propriedades do editor.** Um host visual
> que lê 92 coisas de um controller não está compondo — está reimplementando a
> composição. Reduzir isso encolhe os DOIS arquivos em débito de uma vez, e aí a
> fachada restante fica pequena o bastante para a pergunta (a)/(b) mudar de
> forma.

Enquanto essa fatia não existir, o `EditorController` fica na catraca em 791 —
**congelado e só podendo diminuir**, que é exatamente o que a catraca existe
para garantir.

### 8.4 O registro da decisão (2026-09-03)

**Escolha do autor: (a), a terceira fatia.** `EditorController.qml` continua em
791/400 na catraca — congelado e só podendo diminuir — e **não recebe limite
próprio**. O trabalho vai para o `ShellWorkspaceHost.qml` antes.

O que mudou entre a §8.3 (recomendação) e esta decisão foi **uma medição nova**,
feita em 2026-09-03. A §8.3 dizia que as 92 leituras "viram um punhado de
propriedades agregadas" sem mostrar que isso era possível. Era a parte fraca do
argumento: 92 leituras podem ser 92 conceitos distintos, e nesse caso agregar não
reduz nada — só muda o nome do acoplamento. Medido:

```text
92 pontos de leitura  ->  73 membros DISTINTOS   (quase nao ha' repeticao)
```

**73 membros distintos derrubariam a saída (a)** se não caíssem em lugar nenhum.
Eles caem — e caem nos donos que a etapa 6 já criou:

```text
find/replace ......... ~20 membros  -> EditorFindController
actions (code action) .. 7          -> EditorLanguageController
workspaceEdit .......... 7          -> EditorLanguageController
texto (indent/newline) . 7          -> EditorTextController
abas/arquivos .......... 6          -> EditorDocumentController
completion ............. 5          -> EditorCompletionController
externo (conflito) ..... 5          -> EditorDocumentController
goToLine / rename ...... 6
hover / usages / watch . 5          -> EditorLanguageController
```

**É isto que torna a (a) um corte por responsabilidade e não por tamanho** (§4
regra 9): a agregação não inventa camada nova nem cria pass-through — ela liga o
host aos **sete subcontrollers que já existem**. O anti-precedente do
`AppDomains.qml` citado na §8.2 (espremer gerou "13 propriedades de
pass-through") não se aplica, porque ali não havia dono para onde apontar; aqui
há.

### 8.5 A fatia foi feita no mesmo dia (2026-09-03)

`ui/qml/shell/ShellEditorHost.qml` — 225 linhas, novo — recebeu a fiação inteira
do painel do editor. O `ShellWorkspaceHost` mantém o **layout** (largura, altura,
visibilidade), que depende dos irmãos (banner de saúde, painel inferior) e por
isso não desce.

```text
ShellWorkspaceHost.qml   576 -> 407   (-29%)
leituras editorController 92 -> 2     as duas restantes sao `openDiagnostic`,
                                      de OUTROS paineis: outro consumidor
ShellEditorHost.qml      novo, 225    limite 400
catraca                  18 arquivos, nenhum novo
```

**O corte é por responsabilidade, e o teste de vocabulário passa:** o conceito
"propriedade do editor" saiu do `ShellWorkspaceHost` — o que sobrou não é
fiação do painel, é um consumidor diferente. O remédio é o que a
`ARCHITECTURE.md` §4 regra 8 prescreve **textualmente** para composition root:
*"dividir a composição por área (`domínios/`, `hosts/`, `atalhos/`), fazendo a
contagem de arquivos crescer em vez do tamanho — nunca subir o limite"*. O
precedente interno é o `ShellHeaderHost.qml`, host que recebe 10 controllers e
nunca entrou na catraca.

**O que NÃO foi feito, de propósito:** o `ShellWorkspaceHost` ficou em **407/400**
e continua na catraca. Faltam 7 linhas, e cortá-las seria corte por TAMANHO — o
que a §4 regra 9 proíbe. O que resta ali (`SideRail`, `ProjectExplorer`,
`ProjectHealthBanner`, `StartScreen`, `ShellEditorHost`, `BottomPanelHost`) é
composição de workspace legítima, e nenhuma peça é de outro dono.

### 8.6 A fatia produziu um gate, porque produziu uma falha silenciosa

Mover 85 bindings levantou a pergunta: **se um nome estivesse errado, quem
reclamaria?** Medido por mutação, `findQuery:` → `findQeury:`:

```text
cmake --build --preset dev-local     EXIT=0   nem um aviso
verificar-qml-fiacao.sh              EXIT=0
verificar-qml-logica.sh              EXIT=0
verificar-arquitetura.sh             EXIT=0
```

O gate inteiro verde com o Find quebrado. O QML só reclama de propriedade
inexistente ao **instanciar** o componente, e nenhum teste do harness instancia
os hosts do shell — eles só nascem na IDE de verdade. É a condição exata da §4
regra 11, e por isso nasceu `scripts/verificar-qml-propriedades.sh` (o 15º do
gate): todo `nome:` dentro de um bloco de componente deste repositório tem que
ser propriedade declarada, alias, `on<Sinal>` ou propriedade do tipo raiz.

Provado por mutação em três formas — propriedade com typo, handler de sinal
inexistente, e propriedade renomeada no alvo deixando o binding órfão. As três
reprovam; restaurado, passa limpo em 137 componentes.

**Critério de aceite da fatia, quando ela vier** — o mesmo da §4 regra 9, e é o
que a distingue de cerimônia:

```text
1. grep -c "editorController\." ui/qml/shell/ShellWorkspaceHost.qml  CAI
2. ShellWorkspaceHost.qml sai de 576/400 ou encolhe de verdade
3. nenhum membro agregado vira pass-through de um so' consumidor
4. a catraca aceita sem --atualizar-baseline para CRESCER
```

**O que esta decisão NÃO autoriza:** quebrar a fachada do `EditorController`
agora. A saída (b) — limite próprio para fachada — foi **descartada**, e a (c)
— deixar como está — foi descartada como destino final, não como estado
transitório: até a fatia existir, o arquivo fica em 791, congelado.

## 9. Onde cada coisa mora agora

```text
ui/qml/editor/
├── EditorController.qml            composition root + fachada unica do editor
├── EditorDocumentController.qml    que arquivo esta aberto, o que esta sujo
├── EditorTextController.qml        o que se faz com o TEXTO (cursor, linhas)
├── EditorCompletionController.qml  a lista de completion e o filtro local
├── EditorFindController.qml        busca e substituicao NO ARQUIVO
├── EditorLanguageController.qml    o simbolo sob o cursor (LSP)          [novo]
├── EditorHighlightController.qml   o realce do documento (TS + LSP)      [novo]
├── EditorFormatController.qml      formatar e format-on-save             [novo]
├── EditorPersistenceController.qml sessao e rascunho                     [novo]
└── EditorSurfaceBridge.qml         a ponte fina para a superficie C++
```

**A regra para quem for acrescentar algo ao editor:** a função entra no dono da
pergunta que ela responde. Se nenhum dono responde aquela pergunta, o certo é um
dono novo — não uma função a mais no `EditorController`. Foi assim que ele
chegou a 1.070.
