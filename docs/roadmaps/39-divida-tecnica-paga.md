# 39 — A dívida técnica paga: o que saiu, para onde foi, e o que sobrou

> **Classe: ESTADO** (`docs/README.md`). Descreve o que o código faz **hoje**.
> O trabalho descrito aqui atravessou a meia-noite: começou em **2026-09-03** e
> fechou em **2026-09-04**, com o gate completo verde. As medições individuais
> carregam a data em que foram tomadas. Se divergir do código, o código vence e este documento se corrige no
> mesmo gesto.
>
> **Para que serve:** este documento existe para a **fase de polimento**. Quem
> chegar nela vai precisar saber, sem reabrir a discussão, *por que cada arquivo
> foi partido onde foi partido* e *o que já foi decidido e não se reabre*.
> Ele substitui a §2 do [`38-divida-restante-e-continuidade.md`](38-divida-restante-e-continuidade.md),
> que descrevia a fila anterior.
>
> **Regra zero vale aqui como em tudo:** antes de aceitar qualquer item como
> pendente, MEÇA. Cada seção carrega o comando.

## 1. O antes e o depois, medido

```bash
bash scripts/verificar-arquitetura.sh
cat scripts/arquitetura-baseline.txt
```

Medido em **2026-09-04**, antes e depois do trabalho desta sessão:

```text
arquivo                                       antes  depois  limite
ui/qml/editor/EditorController.qml              791     791     400   congelado (§6)
ui/qml/editor/EditorPane.qml                    538     226     300   SAIU
ui/qml/editor/EditorTextSurface.qml             504     296     300   SAIU
ui/qml/editor/EditorTextController.qml          574     385     400   SAIU
ui/qml/editor/EditorDocumentController.qml      548     385     400   SAIU
ui/qml/editor/EditorFindBar.qml                 329     243     300   SAIU
ui/qml/panels/bottom/SearchPanel.qml            325     251     300   SAIU
ui/qml/panels/bottom/TerminalViewport.qml       319     288     300   SAIU

catraca                                           8       1  arquivos em debito
```

Nenhum arquivo foi cortado para "caber". Cada corte abaixo tem uma pergunta
própria e um **teste de vocabulário** — depois de extrair um domínio, o arquivo
de origem tem de perder o *conceito*, não só as linhas (`ARCHITECTURE.md` §4
regra 9).

## 2. O que a fila anterior errou, e por quê isso importa

O [`38`](38-divida-restante-e-continuidade.md) §2.3 afirmava que três arquivos
— `EditorFindBar` (+29), `SearchPanel` (+25), `TerminalViewport` (+19) — **não
deviam ser cortados**, porque cortar 20 linhas acima do limite seria corte por
TAMANHO, que a §4 regra 9 proíbe.

**A conclusão estava certa e a premissa estava errada.** Ao OLHAR os três, cada
um tinha uma responsabilidade misturada de verdade:

```text
TerminalViewport   traduzia indice ANSI para cor          -> era REGRA na UI
EditorFindBar      declarava dois botoes `component`      -> era WIDGET inline
SearchPanel        continha a barra de substituir         -> era a parte DESTRUTIVA
```

Os cortes que saíram dali não foram por tamanho: foram por responsabilidade, e
os três teriam sido feitos mesmo se estivessem em 200 linhas. **A lição é a
regra zero aplicada a um documento próprio:** o `38` classificou por *distância
do limite*, que é um número, quando o critério é *o que está misturado*, que só
aparece lendo. Fila é hipótese, inclusive quando quem a escreveu foi você.

## 3. Os cortes, um a um

### 3.1 `EditorTextSurface` 504 → 296 — regra saindo de um arquivo que pinta

O arquivo é um `Rectangle` com sarjeta, `Flickable`, `TextEdit`, barra de
rolagem e tooltip. Dentro dele moravam **234 linhas que não desenham nada**:

| Saiu para | O que era |
| --- | --- |
| [`EditorTypingController.qml`](../../ui/qml/editor/EditorTypingController.qml) | auto-close de pares, type-over, envolver seleção, `#include <` → `<>`, backspace de par vazio, **e** todo o roteamento de tecla (qual intenção a tecla representa dado o que está aberto) |
| [`EditorLineHighlights.qml`](../../ui/qml/editor/EditorLineHighlights.qml) | as faixas de linha do cursor e de execução do debugger, com a precedência entre elas |
| [`EditorDiagnosticTooltip.qml`](../../ui/qml/editor/EditorDiagnosticTooltip.qml) | o balão de diagnóstico da sarjeta, com o clamp de largura e a armadilha de binding circular já documentada |

**Teste de vocabulário:**

```bash
grep -c "Qt.Key_" ui/qml/editor/EditorTextSurface.qml   # 0
```

O `EditorTypingController` conversa com a superfície pela **API pública dela**
(`text`, `cursorPosition`, `selectionStart/End`, `remove`, `insert`, `select`).
Isso não é elegância: é o que torna a regra exercitável no harness QML com um
duble de cinco funções, do jeito que o `tst_completion.qml` já faz.

### 3.2 `EditorDocumentController` 548 → 385 — quatro vocabulários, dois saem

O arquivo misturava **abas**, **buffers**, **conflito externo** e **navegação**.
Saíram os dois últimos e mais a lista de recentes:

| Saiu para | A pergunta que ele responde |
| --- | --- |
| [`EditorExternalChangeController.qml`](../../ui/qml/editor/EditorExternalChangeController.qml) | *O disco mudou pelas costas da IDE — e isso é conflito ou não?* |
| [`EditorJumpController.qml`](../../ui/qml/editor/EditorJumpController.qml) | *Levar o cursor até linha L, coluna C do arquivo F — abrindo-o antes se preciso.* |
| [`EditorRecentFiles.qml`](../../ui/qml/editor/EditorRecentFiles.qml) | *Quais arquivos foram abertos há menos tempo?* (MRU, consumido pelo "Buscar em todo lugar") |

A regra central do conflito externo ficou **escrita** no cabeçalho do novo dono,
porque é fácil de errar: *mudança no disco não é conflito; só vira conflito
quando o buffer local também mudou*. As três saídas, nesta ordem — o disco tem o
que a IDE acabou de gravar; o buffer local ainda é igual ao salvo; os dois lados
mudaram.

**Nenhum dos três chama de volta para o dono.** Recebem `filesModel` e
`surfaceBridge`, pedem por sinal (`readFileRequested`, `tabSelectionRequested`).
Não há ciclo entre eles e o `EditorDocumentController`.

**Rede de segurança:** `scripts/qml-harness/tst_external_change.qml` e
`tst_save_recent.qml` já exercitavam exatamente esta API pelo lado de fora. Como
a fachada pública do documento não mudou, os dois testes passaram sem edição —
é o que prova que o corte foi interno.

### 3.3 `EditorTextController` 574 → 385 — pergunta e comando são coisas opostas

| Saiu para | O que era |
| --- | --- |
| [`EditorTextGeometry.qml`](../../ui/qml/editor/EditorTextGeometry.qml) | **PERGUNTAS** que não modificam nada: onde começa a linha, onde termina a palavra, qual a indentação, quais linhas a seleção toca |
| [`EditorSelectionLadder.qml`](../../ui/qml/editor/EditorSelectionLadder.qml) | a escada expandir/encolher seleção — o único pedaço com **MEMÓRIA** |

A geometria precisava sair porque é usada por **três lados** (o próprio
controller de texto, a escada e a completação), e cópia divergente ali produz
cursor no lugar errado — o tipo de bug que ninguém reporta e todo mundo sente.

A escada precisava sair porque tem invariante própria e frágil de propósito: o
histórico só vale enquanto a seleção atual for exatamente a última expansão
registrada **sobre um texto do mesmo tamanho**. Qualquer edição no meio invalida
a pilha em vez de encolher para um lugar que não existe mais. Essa frase agora
mora ao lado do código que a implementa.

### 3.4 `TerminalViewport` 319 → 288 — cor é regra, não desenho

`ansiColor`, `spanFg` e `spanBg` traduziam índice ANSI (as três faixas do padrão
xterm-256) para cor. Isso é **traduzir estado de domínio para token do tema**,
que já tinha dono desde 2026-09-03: o `StatusColors`, onde moram severidade de
diagnóstico e estado de git. Foram para lá como `ansi`, `terminalForeground` e
`terminalBackground`.

`spanCells` virou [`TerminalSpanRules.qml`](../../ui/qml/panels/bottom/TerminalSpanRules.qml),
e essa foi a parte que **não era só arrumação** — ver §5.

### 3.5 `EditorFindBar` 329 → 243 — `component` inline não é reusável

Os dois botões da barra eram `component ToggleButton` e `component ActionButton`
declarados **dentro** do arquivo. Um componente inline é visível só de dentro de
quem o declara: enquanto ficassem ali, a próxima barra da IDE copiaria em vez de
usar. Viraram [`KvToggleChip.qml`](../../ui/qml/components/KvToggleChip.qml) e
[`KvBarButton.qml`](../../ui/qml/components/KvBarButton.qml), ao lado de
`KvButton`, `KvIcon` e `KvIconButton`.

### 3.6 `SearchPanel` 325 → 251 — a parte destrutiva ganha dono

Substituir em todo o projeto reescreve arquivos que o autor talvez nem tenha
aberto. A guarda contra isso — a confirmação em **dois passos**, com o botão
virando "Confirmar" em vermelho — é regra desta barra, não do painel que a
hospeda. Saiu inteira para
[`SearchReplaceBar.qml`](../../ui/qml/panels/bottom/SearchReplaceBar.qml), com a
invariante documentada: **editar a consulta DESARMA a confirmação**, senão um
clique herdado de uma busca anterior se aplicaria à busca nova.

### 3.7 `EditorPane` 538 → 226 — o repasse era a dívida

Este foi o corte de maior consequência, e o único que mudou fiação fora do
arquivo.

Depois de sair dele a trilha de breadcrumbs
([`EditorBreadcrumbs.qml`](../../ui/qml/editor/EditorBreadcrumbs.qml)) e a alça
de reabrir a Estrutura
([`EditorOutlineHandle.qml`](../../ui/qml/editor/EditorOutlineHandle.qml)), o
painel **quase não desenhava mais nada** — três linhas de cartão (`radius`,
`color`, `border`) e o resto composição. E o volume estava em outro lugar:

```text
~31 propriedades  existiam so' para levar um valor do controller ate' um popup
~21 sinais        existiam so' para levar um clique do popup de volta
```

**Cortar o painel em dois pedaços que continuassem repassando seria o erro que a
§4 regra 9 registra no caso `AppDomains`** — *"13 propriedades de pass-through,
nada ficou mais claro, e 'onde X é ligado' passou a ter duas respostas"*.

A saída que a §4 regra 8 prescreve textualmente para composition root é outra:
**dividir a composição por ÁREA, fazendo a contagem de arquivos crescer**. Foi o
que se fez. Nasceu
[`ShellEditorOverlayHost.qml`](../../ui/qml/shell/ShellEditorOverlayHost.qml)
— barra de Find/Replace, os quatro popups (hover, completação, ações, usos) e os
quatro diálogos (criar, renomear, prévia de workspace edit, ir para linha) —
que lê os controllers **direto**.

```text
EditorPane.qml               538 -> 226   perdeu 31 propriedades e 21 sinais
ShellEditorHost.qml          225 -> 163   perdeu 30 bindings e 21 handlers
ShellEditorOverlayHost.qml   novo, 245    limite 400 (composition host)
```

**As propriedades de travessia não mudaram de arquivo: deixaram de existir.** É
essa a diferença entre este corte e o anti-precedente do `AppDomains`.

**Geometria, para quem for mexer:** o host de overlay cobre exatamente a área do
`EditorPane`. `contentTop` (ligado ao `EditorPane.overlayTop`) desce o que
flutua no topo para baixo da faixa de conflito externo, que vive dentro do
painel. Os popups ancorados ao cursor usam
`editorSurface.cursorPointIn(root)` e não dependem disso; `popupX` e `popupY`
prendem o popup dentro da área visível e o viram para cima quando não cabe
embaixo — sem isso, a completação no fim do arquivo nasce fora da tela.

## 4. As regras que ganharam dono

O autor pediu, em 2026-09-03: *"regra de negócio ou lógica não deve ficar na
UI"*. Três donos novos nasceram disso, e todos são **sem estado e sem pixel**:

| Dono | O que decide |
| --- | --- |
| [`TextRules.qml`](../../ui/qml/editor/TextRules.qml) | o que conta como caractere de palavra; qual fechador casa com qual abridor |
| [`PathRules.qml`](../../ui/qml/editor/PathRules.qml) | nome-base, caminho relativo à raiz, "está debaixo de", reescrita por renomeação |
| [`TerminalSpanRules.qml`](../../ui/qml/panels/bottom/TerminalSpanRules.qml) | quantas células um trecho de linha do terminal ocupa |

Mais o `StatusColors`, que já existia e ganhou a família ANSI.

**Por que `TextRules` e `PathRules` moram em `ui/qml/editor/` e não numa pasta
`rules/` própria.** O módulo QML da IDE é **plano**: cada arquivo é registrado
com `QT_RESOURCE_ALIAS` reduzido ao nome, então no app todos são irmãos e um
tipo compartilhado é visível de qualquer pasta. Já o harness de lógica
(`scripts/qml-harness`) carrega os controllers por **caminho relativo real**,
onde só resolve o que está na MESMA pasta. Enquanto os dois modelos coexistirem,
um tipo compartilhado precisa morar junto de quem o harness exercita. Nenhum
deles é `pragma Singleton` pelo mesmo motivo: singleton QML só existe através do
módulo `KineinVectis`, e importar o módulo tornaria as regras inexercitáveis
fora do app. São sem estado — várias instâncias custam nada, e o que importa é
haver uma única **definição**.

## 5. As duplicações que morreram, e o que elas já tinham quebrado

Isto não é limpeza estética. Cada uma abaixo foi **medida**, e duas já
divergiam:

### 5.1 `isWordChar`, duas implementações diferentes da mesma regra

```text
EditorTextController   (ch >= "a" && ch <= "z") || (ch >= "A" && ch <= "Z") ...
EditorTextSurface      /[A-Za-z0-9_]/.test(character)
```

As duas concordavam — **por sorte, não por construção**. Bastava uma delas
ganhar `$` (comum em identificador gerado) para o autocomplete e o auto-close
discordarem sobre onde começa uma palavra, sem nada reclamar. É a mesma forma da
divergência de cor de severidade que virou o 16º gate
([`38`](38-divida-restante-e-continuidade.md) §3).

### 5.2 `spanCells`, e as cópias JÁ divergiam

```text
TerminalViewport             `span.text` ausente -> conta como ""
TerminalSelectionController  `span.text` ausente -> ramo diferente
```

Largura de célula é a base de **todo** o mapeamento pixel→célula. Renderer e
seleção contando diferente significa que o texto copiado não é o texto
destacado — e nenhum gate reclamaria, porque os dois lados "funcionam".

### 5.3 A derivação "este caminho está debaixo daquele"

`p === a || p.indexOf(a + "/") === 0` aparecia **três vezes dentro do
`EditorDocumentController`**, duas delas lado a lado (renomear abas e renomear a
lista de recentes). O `+ "/"` não é detalhe: sem ele, renomear `src/app` também
pegaria `src/application`, e o bug só apareceria com dois diretórios de nome
parecido no mesmo projeto.

### 5.4 O que NÃO deu para consolidar, e o que desbloqueia

Medido em **2026-09-03**:

```bash
grep -rn "function relativeToRoot\|function baseName" --include=*.qml ui
```

```text
relativeToRoot   6 copias   editor, jobs, shell, search (x2), project
baseName         3 copias   editor, search, project
```

Só a cópia do editor foi consolidada. As outras cinco vivem em `ui/qml/jobs/`,
`ui/qml/shell/`, `ui/qml/search/` e `ui/qml/project/`, e o harness carrega
controllers dessas pastas por caminho relativo real — um tipo em `editor/` não
resolveria lá. **O desbloqueio é único e conhecido:** fazer o `.qrc` espelhar a
árvore real em vez de achatá-la com `QT_RESOURCE_ALIAS`, e então
`import "../rules"` passa a valer nos dois modelos. Isso mexe em ~153 registros
do `ui/CMakeLists.txt` e em toda referência implícita entre pastas do app — é
uma fatia própria, não um detalhe deste corte.

## 6. O que sobrou: `EditorController`, 791/400

**Um arquivo, e ele tem decisão registrada do autor.** Não corte sem falar com
ele. O registro está em
[`../arquitetura/32-editor-por-responsabilidade.md`](../arquitetura/32-editor-por-responsabilidade.md)
§8.4: a saída **(b)** — limite próprio para fachada — foi **descartada**; a
**(c)** — deixar como está — foi descartada como destino final, não como estado
transitório. Até a fatia existir, o arquivo fica congelado e só pode diminuir.

O que este arquivo é, medido em **2026-09-03**:

```text
791 linhas  ~65 aliases + ~120 funcoes de delegacao sobre 8 subcontrollers
```

Ele implementa muito pouco: é a **fachada única do domínio editor**. E o
tamanho dela é função de quantos membros os consumidores pedem:

```bash
grep -rho "editorController\.[a-zA-Z]*" --include=*.qml ui | sort -u | wc -l
grep -rc "editorController\." --include=*.qml ui | grep -v ":0"
```

```text
123 membros DISTINTOS, consumidos por 15 arquivos
```

**As duas saídas reais, e nenhuma é "cortar linhas":**

**(1) Dissolver a fachada.** Expor os subcontrollers como o `language` e o
`highlight` já são expostos (`readonly property alias`), e deixar cada
consumidor falar com o dono: `editorController.find.next()` em vez de
`editorController.findNext()`. Isso apaga ~110 linhas só na área de find, e a
fachada inteira se ~120 funções virarem 8 aliases. **Custo medido: ~190 pontos
de chamada em 15 arquivos.** É trabalho real, com risco real, e é o caminho que
a §8.4 chama de "fatia própria".

**(2) Corrigir a categoria.** Reconhecer que fachada de domínio não é
"controller com lógica", que é o que o limite de 400 mede. O contra continua
sendo o que a §8.2 já registrou: `AppDomains` estava em 325 contra 300 (um
quase-acerto); este está em 791 contra 400, quase o dobro.

**Esta sessão não escolheu nenhuma das duas**, e é deliberado: a §4 regra 8 diz
que mexer em limite é **decisão explícita, registrada e justificada** do autor,
nunca silenciosa.

## 7. Onde cada coisa mora agora

```text
ui/qml/editor/
├── EditorController.qml              composition root + fachada do editor
├── EditorSurfaceBridge.qml           ponte fina para a superficie C++
│
│   ── os donos de DADO
├── EditorDocumentController.qml      que arquivo esta aberto, o que esta sujo
├── EditorExternalChangeController.qml o disco mudou pelas costas          [novo]
├── EditorJumpController.qml          ir ate' linha/coluna, abrindo antes  [novo]
├── EditorRecentFiles.qml             o MRU de arquivos abertos            [novo]
├── EditorPersistenceController.qml   sessao e rascunho
│
│   ── os donos de TEXTO
├── EditorTextController.qml          transformar linhas e blocos
├── EditorTextGeometry.qml            PERGUNTAS sobre o texto              [novo]
├── EditorSelectionLadder.qml         expandir/encolher selecao (memoria)  [novo]
├── EditorTypingController.qml        o que uma TECLA significa            [novo]
├── TextRules.qml                     regras puras de texto de codigo      [novo]
├── PathRules.qml                     regras puras de caminho              [novo]
│
│   ── os donos de LINGUAGEM
├── EditorCompletionController.qml    a lista de completion e o filtro local
├── EditorLanguageController.qml      o simbolo sob o cursor (LSP)
├── EditorHighlightController.qml     o realce do documento (TS + LSP)
├── EditorFormatController.qml        formatar e format-on-save
├── EditorFindController.qml          busca e substituicao NO ARQUIVO
│
│   ── o que DESENHA
├── EditorPane.qml                    compoe o painel (226 linhas)
├── EditorTextSurface.qml             sarjeta + rolagem + TextEdit
├── EditorLineHighlights.qml          faixas de linha do cursor/execucao   [novo]
├── EditorDiagnosticTooltip.qml       o balao da sarjeta                   [novo]
├── EditorBreadcrumbs.qml             a trilha "src › lsp › manager.rs"    [novo]
├── EditorOutlineHandle.qml           a alca de reabrir a Estrutura        [novo]
└── (popups e dialogos)               instanciados pelo host de overlay

ui/qml/shell/
├── ShellEditorHost.qml               liga o EditorPane aos controllers
└── ShellEditorOverlayHost.qml        liga o que FLUTUA aos controllers    [novo]

ui/qml/components/
├── KvToggleChip.qml                  botao liga/desliga de rotulo curto   [novo]
└── KvBarButton.qml                   botao pequeno de barra               [novo]

ui/qml/panels/bottom/
├── TerminalSpanRules.qml             largura em CELULAS de um trecho      [novo]
└── SearchReplaceBar.qml              substituir no projeto (2 passos)     [novo]
```

**A regra para quem for acrescentar algo:** a função entra no dono da pergunta
que ela responde. Se nenhum dono responde aquela pergunta, o certo é um dono
novo — não uma função a mais na fachada. Foi assim que o `EditorController`
chegou a 1.070 linhas (medição de 2026-09-02).

## 8. Como verificar tudo isto

```bash
bash scripts/verificar.sh                     # o gate inteiro, 17 verificacoes
bash scripts/verificar-arquitetura.sh         # a catraca
bash scripts/verificar-qml-propriedades.sh    # binding orfao
bash scripts/verificar-qml-duplicacao.sh      # derivacao duplicada
bash scripts/verificar-atalhos.sh             # atalho que a paleta anuncia
bash scripts/verificar-qml-logica.sh          # os harnesses, sem GUI
```

**Dois harnesses nasceram junto com os cortes**, e é isso que torna a extração
mais que arrumação — a regra saiu de arquivos que nenhum teste instancia e caiu
em arquivos que o gate exercita:

```text
scripts/qml-harness/tst_editor_typing.qml     12 casos, mask ate' 2^26
scripts/qml-harness/tst_selection_ladder.qml  10 casos, mask ate' 2^14
```

Os dois foram **provados por mutação**, como o contrato exige de todo gate:

```text
EditorTypingController  cursor do auto-close +1 -> +2     bitmask=2      reprova
EditorSelectionLadder   guarda do historico perde o
                        `lastExpansion.length !== text.length`  bitmask=2048  reprova
```

O segundo é o mais valioso: a guarda de tamanho do texto é o que impede o
`Ctrl+Shift+W` de encolher para offsets que uma edição já deslocou, e **até
2026-09-03 ela não tinha teste nenhum** — quebrá-la não produzia erro, produzia
uma seleção no lugar errado.

**O que o teste da escada documentou de comportamento não óbvio:** com o cursor
DENTRO da indentação, o primeiro degrau não é "a linha sem indentação" e sim a
linha inteira — aquele candidato *começa depois do cursor*, então não o contém.
Não é defeito; é consequência direta de "o menor candidato que CONTÉM a
seleção".

## 8.1 O 17º gate, e ele nasceu de um relato de uso (2026-09-04)

O autor disse: *"eu ainda não consigo acessar as bibliotecas na versão de
desenvolvimento"*. **A causa não era o painel — era o atalho.**

O core declara um `default_shortcut` por comando, a paleta mostra esse texto ao
lado do comando, e **nada verificava que a UI honrava a promessa**. Medido no
mesmo dia, a falha não era uma:

```text
Ctrl+Alt+L   library.list      a UI FORMATAVA o arquivo — `format.text`
                               declarava o mesmo atalho, e era esse que a UI
                               ligava
Ctrl+Alt+D   datasource.list   a UI iniciava o DEBUG (alias de `debug.start`)
Ctrl+O       workspace.open    nenhum `Shortcut`: apertar nao fazia nada
Alt+Enter    lsp.codeActions   a UI liga Alt+Return; no Qt sao teclas
                               DIFERENTES (`Key_Enter` e' o do numerico)
```

Nenhuma produzia erro. O build passava e os dezesseis gates passavam. **A única
forma de descobrir era apertar a tecla** — a definição exata de falha silenciosa
da §4 regra 11, e por isso nasceu
[`scripts/verificar-atalhos.sh`](../../scripts/verificar-atalhos.sh).

**A costura é uma anotação**, e ela é deliberada: sem um elo explícito, nenhuma
máquina sabe que `onActivated: root.libraryController.open()` implementa
`library.list`.

```qml
Shortcut {
    // comando: library.list
    sequence: "Ctrl+Alt+K"
    onActivated: root.libraryController.open()
}
```

**Provado por mutação, três formas** — e a primeira **reproduz o bug original**:

```text
library.list volta a Ctrl+Alt+L     reprova (a sequencia do Shortcut diverge)
a anotacao some                     reprova (comando sem Shortcut anotado)
a sequencia do Shortcut muda        reprova (promessa != realidade)
```

**O que este gate diz sobre o resto:** o registro de saídas do dogfooding
continua sendo o item mais barato e mais valioso da lista. Uma frase do autor
achou quatro defeitos que dezesseis gates não achavam.

## 9. O que continua aberto, e não é dívida

Segue valendo o que o [`38`](38-divida-restante-e-continuidade.md) §4 registra,
sem alteração desta sessão:

```text
25  handshake DAP com probe-rs         precisa de sonda ou alvo QEMU
26  banco: COFRE DE CREDENCIAL ANTES   §7.3 do roadmaps/35
27  TimescaleDB e Grafana por HTTP     Grafana e' AGPL: API, nunca embutido
28  simulacao: CALCULO sem tela        §5.1 ja respondida
--  UI de embarcado                    probe.list responde por IPC, sem painel
```

E a lacuna que não é técnica continua sendo a mais cara: **o registro de saídas
do dogfooding está VAZIO** (`docs-privada/diario/19-registro-de-saidas.md`).
Enquanto estiver, a ordem da frente C é palpite.
