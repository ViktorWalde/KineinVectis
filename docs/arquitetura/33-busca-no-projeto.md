# 33 — Busca no projeto: os três buscadores, o casamento multi-linha e a sintaxe `\n`

> **Classe: ESTADO.** Tudo aqui é verificável contra o disco. Se divergir do
> código, o código vence e este documento se corrige no mesmo gesto
> (`docs/README.md`, ordem de precedência).
>
> Escrito em 2026-09-02, na etapa 9 do
> [roadmaps/30](../roadmaps/30-caminho-para-o-mvp.md), quando `fs.search` deixou
> de casar linha a linha.

## 1. Três coisas chamadas "busca", e confundi-las é o defeito clássico

A palavra é a mesma e os donos são três. Quem procura "a busca" e abre o
primeiro arquivo com `Search` no nome tem dois terços de chance de mexer no
lugar errado.

```text
BUSCA NO ARQUIVO        Ctrl+F. UI pura, ZERO RPC — o buffer ja esta na
EditorFindController    memoria. Realce em cima do texto aberto, next/prev,
ui/qml/editor/          substituir no buffer. Nunca toca disco.

BUSCA NO PROJETO        Ctrl+Shift+F (Ctrl+Shift+H ja' no modo substituir).
SearchController        Painel de baixo, PERSISTENTE, resultado navegavel. Tem
ui/qml/search/          `fs.replace` atras dela: e' a unica das tres que
                        ESCREVE, e escreve em varios arquivos.
                        -> `fs.search` / `fs.replace` no core

SEARCH EVERYWHERE       Ctrl+Shift+A ou Ctrl+Shift+N; Ctrl+E abre em
                        RECENTES. Caixa MODAL,
SearchEverywhere-       efemera, teclado-primeiro, sem escrita nenhuma. Acha
Controller              ARQUIVO, SIMBOLO (`@` documento, `#` workspace) e
ui/qml/search/          COMANDO.
                        -> `fs.findFiles`, `lsp.*Symbols`, `command.list`
```

**Regra prática:** se a pergunta tem a palavra "substituir", é a do meio. Se tem
"abrir rápido", é a de baixo. Se não sai da tela do editor, é a de cima.

### Por que as duas de baixo moravam no mesmo arquivo, e por que não moram mais

Até 2026-09-02 `SearchController.qml` tinha 420 linhas e era as duas. Elas
dividiam **o nome e nada mais**:

| | painel de baixo | caixa modal |
| --- | --- | --- |
| vida | persistente, fica aberto | efêmera, some no Esc |
| escrita | **destrutiva** (`fs.replace`) | nenhuma |
| entrada | campo do painel, com foco | teclado, primeiro caractere é modo |
| estado | resultados navegáveis | lista descartável + índice |
| erro | vira texto no painel | vira texto na caixa |

O corte veio pela `ARCHITECTURE.md` §4 regra 9 (**responsabilidade, não
tamanho**) e o gatilho foi concreto: a etapa 9 precisava acrescentar a tradução
de `\n` num arquivo que já estava em 420/400. Resultado medido em 2026-09-02:
`SearchController.qml` 185 linhas, `SearchEverywhereController.qml` 352 — e o
arquivo **saiu da catraca**, de 19 para 18 arquivos em débito.

A prova de que o corte foi por vocabulário e não por tesoura: `grep -i
everywhere ui/qml/search/SearchController.qml` volta **vazio**.

## 2. O casamento no CONTEÚDO (core, `fsops/search.rs`)

Antes: `for linha in content.lines()` — o que torna uma query com `\n`
**impossível de casar por construção**, não difícil.

Agora a varredura é sobre o conteúdo inteiro, com um cursor:

```text
cursor = 0
enquanto find_literal(content[cursor..], query) devolver offset:
    start  = cursor + offset
    linha  = avanca line_number/line_start ate' `start`   (INCREMENTAL)
    column = caracteres de line_start ate' start, +1
    end    = start + query.len()
    empurra o match
    cursor = end          <- retoma NO FIM, nao em start+1
```

Três decisões estão nesse bloco, e cada uma tem consequência visível:

**a) O relógio de linha nunca volta atrás.** `line_number` e `line_start` são
avançados a partir de onde pararam, não recontados do começo do arquivo. Recontar
seria quadrático num arquivo com muitas ocorrências — e arquivo com muitas
ocorrências é exatamente o caso em que a busca precisa ser rápida.

**b) `cursor = end` — matches não se sobrepõem.** `aa` em `aaa` é **1** match,
não 2. Isso não é detalhe de contagem: é o que faz o número que o preview mostra
ser o número que a escrita vai fazer, porque `replace` também consome o casamento
inteiro.

**c) `end` cai em fronteira de caractere.** `find_literal` compara byte a byte
(o case-insensitive é ASCII-only, de propósito, para manter os offsets exatos),
então o casamento tem o mesmo comprimento da query e `start + query.len()` nunca
parte um caractere UTF-8 ao meio.

### O preview: uma linha na lista, mesmo quando o casamento tem três

```text
casamento de UMA linha    -> mostra a LINHA INTEIRA (trim, 200 caracteres)
                             contexto ajuda a reconhecer o lugar
casamento MULTI-LINHA     -> mostra o TEXTO CASADO, com as quebras trocadas
                             pela marca " ⏎ " (trim, 200 caracteres)
```

Mostrar só a primeira linha de um casamento de três — que é o que o Code OSS faz
— faria o usuário aprovar uma reescrita maior do que a que viu. Aqui a operação é
destrutiva e multi-arquivo, então o preview mostra **tudo o que vai ser
reescrito**. A marca ` ⏎ ` existe porque um `\n` cru quebraria a linha da lista e
desalinharia a tabela de resultados.

### Limites (inalterados pela etapa 9)

```text
500 matches no total       -> `truncated: true` diz que o limite cortou
1 MiB por arquivo          -> arquivo maior e' pulado em silencio
UTF-8 apenas               -> binario e' pulado em silencio
.git .kinein .idea .cache target build node_modules -> nao entram
symlink                    -> nao segue
walk deterministico        -> profundidade, nome case-insensitive
```

## 3. A guarda que saiu, e por que ela estava certa enquanto existiu

De 2026-08-29 a 2026-09-02, `handlers/fs.rs` respondia `INVALID_PARAMS` para
`query` ou `replacement` com `\n`. **Não era limitação preguiçosa — era a única
resposta honesta que aquela busca permitia:**

> a busca casava linha a linha e não achava nada; o replace casava no conteúdo e
> achava tudo. O usuário via **"0 resultados"** e os arquivos eram reescritos
> mesmo assim.

Transação, snapshot e rollback protegem contra falha de **escrita**. Não protegem
contra alguém aprovar o que não viu.

**Por isso a etapa 9 começou pela busca, não pela guarda.** Tirar a guarda
primeiro teria devolvido exatamente o defeito que ela cobria. A guarda saiu
quando a razão dela saiu, e no lugar dela ficou um comentário dizendo isso — para
que a próxima sessão não a "restaure" achando que faltava.

A invariante que ela protegia continua de pé, agora sozinha:

> **O número de resultados da busca é o número de substituições do replace.**

Travada por três testes que falham se ela cair:

| teste | onde | o que quebra se sumir |
| --- | --- | --- |
| `multiline_search_and_replace_agree_on_the_count` | `fsops/search.rs` | contagem divergente volta a ser possível |
| `multiline_search_previews_exactly_what_replace_will_rewrite` | `tests/fs.rs` | o preview volta a mentir sobre o tamanho |
| `a_multiline_replacement_is_found_by_the_next_search` | `tests/fs.rs` | o ciclo não fecha: escreve algo que a busca não acha |
| `replace_touches_exactly_the_files_search_reports` | `tests/fs.rs` | busca e replace divergem no **walk** (desde 2026-08-29) |

O terceiro é o que fecha o ciclo e o mais fácil de esquecer: **o texto que a
substituição escreveu tem de ser achável pela busca seguinte.** Sem ele, seria
possível ter uma busca e um replace que concordam entre si e discordam da
realidade.

## 4. A sintaxe `\n` mora na UI, e isso é decisão, não acidente

O campo do painel é um `TextInput` — **uma linha por construção**. O usuário não
consegue digitar nem colar uma quebra. Sem um caminho de entrada, a capacidade
nova existiria sem ninguém alcançá-la: o anti-padrão "mecanismo sem usuário" da
`ARCHITECTURE.md` §8.

`SearchController.expandLineBreaks()` traduz, numa passada da esquerda para a
direita:

```text
o usuario digita   vira            para que serve
\n                 quebra real     buscar/substituir atravessando linhas
\\n                \n literal      procurar a sequencia de escape NO CODIGO
\t                 \t  (intocado)  nao e' sintaxe da caixa; nao inventamos escape
\ no fim           \   (intocado)  nao some, nao estoura o indice
```

A passada única é o ponto sutil: a barra que acabou de ser resolvida **não pode**
ser reinterpretada junto com o `n` seguinte, senão `\\n` viraria quebra.

### Por que a tradução NÃO pode descer para o core

Porque `query` é, no contrato, **texto literal** — e é isso que o torna útil:

> Se o core interpretasse escapes, procurar por um `\n` de verdade dentro do
> código — o que qualquer pessoa mexendo em C ou Rust faz o tempo todo — passaria
> a ser impossível. A caixa ganharia uma capacidade e perderia outra.

Na UI a decisão é **reversível e local**: outro campo (um `TextArea`, um dia)
manda o texto cru e não passa por tradução nenhuma. No core seria contrato, e
contrato não se desfaz sem bump de versão.

O `placeholder` dos dois campos do painel diz `\n quebra linha` — é onde a
sintaxe é descoberta, e é por isso que ela está lá e não só aqui.

## 5. O caminho completo de um `fs.replace` multi-linha

```text
usuario digita "a\nb" no painel
  SearchPanel.qml            campo de uma linha, placeholder anuncia o \n
  SearchController           expandLineBreaks -> "a<LF>b"
                             replaceInFilesRequested(query, replacement, case)
  SearchRequestRouter        recusa se houver aba SUJA (nao ha' preview de um
                             buffer que o disco nao tem) -> coreClient.replaceInFiles
  core_client_requests.cpp   monta o JSON-RPC
  handlers/fs.rs             valida params (query vazia -> INVALID_PARAMS)
  fsops::replace             walk unico + find_literal, calcula TODOS os
                             conteudos novos ANTES de escrever
  fsops (transacao)          revalida snapshots, escreve atomico, rollback
                             total se qualquer arquivo falhar
  <- { files: [...], replacements: N }
  SearchEventRouter          onFilesReplaced -> searchController
  SearchController           replaceSummary vira texto no painel
```

**Onde a guarda de aba suja mora e por quê:** no router, não no controller e não
no core. É uma regra sobre o estado da UI (existe buffer não salvo?), então quem
a conhece é quem fala com a UI. O core não sabe o que é uma aba.

## 6. O que o plano de 2026-08-29 previu, e o que foi construído

`PONTO_ATUAL.md` §0.2m desenhou a fatia com antecedência e acertou o diagnóstico
inteiro. Duas coisas saíram diferentes, e as duas merecem registro:

**a) `endLine` + `endColumn` no protocolo: NÃO foram adicionados.** O plano
previa estender `FsSearchMatch` com o fim do casamento. Ao construir, o fim
mostrou-se **derivável e não consumido**: quem navega até o resultado usa o
INÍCIO (`line`/`column`), e quem precisa ver a extensão está lendo o `preview`,
que já mostra o casamento inteiro. Campo de protocolo que ninguém lê é contrato
que nunca mais se remove. Ficou de fora — e volta no dia em que houver um
consumidor real (realce do trecho casado no editor, por exemplo).

**b) `lang/positions.rs` não foi usado.** O plano apontava para ele como quem já
sabe traduzir offset → (linha, coluna). O caminho incremental dentro do próprio
laço é O(n) sobre o arquivo e não cria dependência do `fsops` para o domínio de
linguagem — que é uma direção de dependência que este repositório não tem.

**A pergunta de design que o plano deixou em aberto** — *"como mostrar um
casamento de 3 linhas numa linha da lista?"* — foi respondida pela marca ` ⏎ `
com o casamento inteiro, e não pela primeira linha mais reticências do Code OSS.
A razão é a §2: aqui o preview autoriza uma escrita.

## 7. O que trava isto no gate

```text
cargo test                       433 testes (2026-09-02), incluindo os quatro
                                 da §3
scripts/verificar-qml-logica.sh  tst_search_multiline.qml — a traducao e o
                                 caminho ate' o core
                                 tst_save_recent.qml — recentes, ja' na caixa
scripts/verificar-qml.sh         qmllint estrito
scripts/verificar-qml-fiacao.sh  binding auto-referente
scripts/verificar-transicao-     estado por-workspace com UM dono: os dois
  workspace.sh                   controllers tem `clear()` e os dois sao
                                 chamados pelo WorkspaceUiResetter
scripts/verificar-arquitetura.sh catraca: SearchController saiu do debito
```

**Mutações que provaram `tst_search_multiline.qml`** (um teste que não reprova
não é rede — `ARCHITECTURE.md` §4 regra 11):

| mutação | resultado |
| --- | --- |
| `expandLineBreaks` vira identidade | ✗ reprova (bitmask ≠ 0) |
| remover o ramo do escape `\\` | ✗ reprova |
| mandar `replacement` sem tradução | ✗ reprova |

**Mutações do lado do core**, feitas na própria etapa 9: truncar a query no
primeiro `\n` passou a reprovar só depois que o fixture ganhou uma **isca** — uma
segunda linha `"primeira"` que faz a query truncada casar MAIS vezes que a
inteira. Fixture em que a mutação dá o mesmo número por coincidência é fixture
que não testa nada.

## 8. O que continua em aberto

**a) `SearchPanel.qml` está em 325/300, na catraca.** O campo continua sendo um
`TextInput` de uma linha; a sintaxe `\n` é a saída barata e honesta, não a
definitiva. Um `TextArea` de duas ou três linhas no modo replace resolveria a
entrada de verdade — e é fatia própria, porque o painel já está em débito e a §4
regra 9 pede o corte por responsabilidade antes do acréscimo.

**b) O resultado multi-linha navega para o INÍCIO do casamento.** É o
comportamento correto e o único que o protocolo permite hoje (ver §6a). Realçar
no editor o trecho casado inteiro exige o fim no protocolo — e um consumidor que
o peça.

**c) `MAX_SEARCH_MATCHES = 500` conta casamentos, não arquivos.** Uma query
multi-linha curta e frequente esgota o teto mais rápido do que a intuição sugere.
`truncated: true` continua dizendo a verdade, e a UI continua mostrando o aviso —
mas ninguém mediu ainda se 500 é o número certo para busca multi-linha.
