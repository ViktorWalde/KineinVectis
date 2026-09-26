# V6 — símbolos e indentação sem bloquear a tecla

> **Classe: ARQUITETURA / PLANO.** Escrito em 2026-09-25, antes de qualquer
> código, a partir de medição do que já existe. Fonte do escopo: roadmap 47 §V6
> e roadmap 48 §11.

## 1. O que foi medido antes de desenhar

Três achados mudam o desenho em relação ao que os roadmaps supunham.

**A ponte C++ funde as duas respostas de símbolo num sinal só.** Em
`ui/src/core_client_dispatch_lsp.cpp`, `lsp.documentSymbols` e
`lsp.workspaceSymbols` caem no mesmo `lspSymbolsResolved(symbols)`, sem nada que
diga qual pedido está respondendo. O caminho de FALHA, no mesmo fluxo, preserva
o método (`handleRequestFailed(method, message)`). A assimetria é o defeito: o
sucesso perde a informação que o fracasso mantém.

**A aba Símbolos não consome LSP nenhum.** `SymbolsController.qml` usa só
`index.symbols` — o índice próprio, que responde sem arquivo aberto. Quem usa
LSP é a paleta `SearchEverywhereController`: `@nome` pede `documentSymbols` e
`#nome` pede `workspaceSymbols`. Então "ligar a segunda fonte da aba Símbolos"
(roadmap 47) é trabalho novo, e não religar um fio existente.

**O core já mantém a árvore Tree-sitter viva por documento.** O
`SyntaxTreeService` guarda `ParsedDocument { language, content, tree, touched }`
e reparsa incrementalmente a cada `syntaxTree.update`. A indentação por gramática
**não precisa de parser novo** — precisa de uma consulta nova sobre a árvore que
já está lá.

**Mas o `ParsedDocument` não guarda a versão.** O `update` recebe `version` e
não a persiste. Sem isso o core não tem como dizer "respondi sobre a versão N", e
a regra do E1 — "aplica correção somente se documento, versão e cursor ainda
coincidirem" — não teria como ser cumprida com honestidade. **É a primeira coisa
a mudar.**

## 2. L1 — a fonte do símbolo deixa de ser anônima

### 2.1 O que muda na ponte

Um sinal por origem, em vez de um sinal para as duas:

```text
lsp.documentSymbols   → lspDocumentSymbolsResolved(path, symbols)
lsp.workspaceSymbols  → lspWorkspaceSymbolsResolved(query, symbols)
```

O `path` e a `query` não são enfeite: são a identidade do pedido. Sem eles, uma
resposta atrasada de `@foo` pinta a lista de `#foo` — o mesmo defeito que a V5
consertou nas abas, uma camada adiante.

### 2.2 O que muda na aba Símbolos

Hoje ela tem UMA fonte (índice). Passa a ter duas, e a regra da §11.1 do
roadmap 48 é explícita: **deduplicar por arquivo/posição** e **manter a fonte
visível** quando isso ajudar a explicar divergência.

A regra de mesclagem é pura e vai para um `SymbolMergeRules.qml`, com harness:

```text
mesma posição (arquivo + linha + coluna)  → um item, fonte "lsp"
só no índice                              → item com fonte "índice"
só no LSP                                 → item com fonte "lsp"
índice respondeu, LSP ainda não           → mostra índice, sem espera
LSP respondeu para OUTRO arquivo          → descartado
```

Por que o LSP vence no empate: ele tem tipo, escopo e assinatura; o índice tem
nome e posição. Por que o índice aparece sozinho: ele responde desde o primeiro
segundo e sem arquivo aberto, e é isso que faz a aba ser útil antes de o servidor
subir (pilar 0 do roadmap 42).

### 2.3 O que NÃO muda

Nenhum método novo de LSP. O core já pede os dois; o que falta é a ponte dizer
qual voltou.

## 3. E1 — indentação por gramática, fora do caminho da tecla

### 3.1 O que existe hoje, medido

`EditorTextController.insertNewline()` é uma heurística **local e sem
linguagem**:

```text
linha começa com //        → continua o comentário
dentro de /* */            → continua com " * "
termina em { e há } à frente→ abre bloco com a linha do meio
termina em { ( [ :         → indenta um nível
resto                      → repete a indentação da linha
```

O que ela erra, e é medível: `}` digitado **não dedenta** (não há tratador para
`Key_BraceRight`); `return`/`pass`/`break` em Python não dedentam a próxima;
`case:`/`public:` em C++ e braços de `match` em Rust seguem a regra genérica do
`:` e do `{`; e um `{` **dentro de string ou comentário** indenta como se fosse
estrutura.

### 3.2 A forma, e ela é a mesma da V5

```text
Enter ou }
 → aplica o fallback LOCAL imediatamente        (a tecla nunca espera)
 → pede syntaxTree.indent { path, version, offset, trigger }
 → resposta chega
 → aplica a correção SOMENTE se path, version e cursor ainda coincidem
```

Nada síncrono no caminho da tecla. O nome do método é `syntaxTree.indent`, e não
`lang.indent` como o roadmap 48 sugeriu: o serviço já se chama `syntaxTree.*` no
fio, e inventar um segundo prefixo para o mesmo dono seria dívida de nome.

### 3.3 Por que a versão é obrigatória, e não recomendada

O core responde a partir da árvore que tem. Se o autor digitou entre o pedido e a
resposta, a árvore é de outra versão e a correção cairia no lugar errado. Então:

1. `ParsedDocument` passa a guardar `version`;
2. `syntaxTree.indent` **devolve** a versão da árvore que usou;
3. a UI descarta a resposta se a versão devolvida não for a que ela pediu.

Três travas, e a terceira existe porque as duas primeiras dependem do core estar
correto — a UI não confia, confere.

### 3.4 O que a resposta carrega

```text
{ version, indent: "<texto>", dedentTo: <coluna|null>, source: "grammar" }
```

`indent` é TEXTO, não número de níveis: quem decide espaço ou tabulação é a
configuração do editor, e devolver "dois níveis" obrigaria os dois lados a
concordar sobre o que é um nível. `dedentTo` serve ao `}` e ao `case:`, que não
indentam a linha nova — corrigem a atual.

### 3.5 O fallback não é plano B, é o caminho normal

Ele roda **sempre**, e a resposta estrutural só corrige quando diverge. Com isso:

- arquivo sem gramática (Markdown, texto, `.log`) funciona como hoje;
- árvore com erro de sintaxe — que é o estado normal enquanto se digita — não
  trava a tecla;
- core lento ou morto degrada para o comportamento de hoje, sem tela de erro.

### 3.6 Medição obrigatória

O aceite do V6 pede latência medida antes e depois. O projeto já tem
`typing_perf_harness` em C++. A medição é a mesma régua da V1: **p95 da latência
da tecla**, com e sem a correção estrutural ligada, no mesmo corpus.

Se a correção estrutural custar tecla, ela sai — a régua não é "a indentação
ficou melhor", é "a indentação ficou melhor E a tecla não piorou".

## 4. Onde o código mora

```text
crates/kinein-core/src/lang/indent.rs      regra por gramática (pura, testável)
crates/kinein-core/src/lang/service.rs     guarda `version`; responde indent
crates/kinein-core/src/handlers/…          roteia syntaxTree.indent
ui/src/core_client_dispatch_lsp.cpp        dois sinais, um por origem
ui/qml/editor/SymbolMergeRules.qml         mescla índice + LSP (harness)
ui/qml/editor/EditorIndentRules.qml        o fallback local, extraído e testável
ui/qml/editor/EditorTextController.qml     aplica; não decide mais sozinho
```

**`EditorController.qml` não cresce** — é exigência escrita do V6. Ele está em
791 linhas contra um limite de 400, e é o único arquivo em débito declarado na
catraca. Nenhuma linha nova entra nele; o que precisar de dono novo ganha dono
novo.

O fallback sai do `EditorTextController` para um `EditorIndentRules.qml` puro
pelo mesmo motivo que a regra do Markdown saiu do painel: **o harness carrega um
espelho plano das fontes, sem os tipos C++** — regra separada do widget é regra
testável.

## 5. Ordem, e o que corta

```text
1. version no ParsedDocument + syntaxTree.indent devolvendo a versão
2. EditorIndentRules extraído do EditorTextController, com harness do que já existe
3. indent.rs com C e Rust primeiro (gramáticas mais estáveis)
4. a UI aplica a correção, com as três travas
5. medição p95 antes/depois
6. L1: dois sinais + SymbolMergeRules + aba Símbolos com duas fontes
7. C++ e Python no indent.rs
```

L1 vem depois de E1 de propósito, embora o roadmap os liste na ordem inversa: E1
toca o caminho da tecla, que é a régua da versão, e deixar a medição para o fim
da fatia seria descobrir tarde demais que ela não passa.

**L2/L3 não entram aqui**, e o roadmap já diz por quê: "não entram juntos só
porque compartilham LSP".

## 6. Riscos ditos

- **A árvore está quase sempre com erro** enquanto se digita. A regra de
  indentação tem de funcionar sobre árvore com `ERROR`, e não presumir árvore
  limpa. É o caso comum, não a exceção.
- **`}` digitado altera a linha atual**, não a próxima — é o único caso em que a
  correção estrutural mexe em texto que o autor acabou de escrever. Se a resposta
  atrasar e o cursor tiver andado, ela é descartada, não aplicada adiante.
- **Dedupe por posição supõe que as duas fontes concordam sobre coluna.** O
  índice conta bytes; o LSP conta UTF-16. A mesclagem tem de normalizar antes de
  comparar, ou acento vira símbolo duplicado.
