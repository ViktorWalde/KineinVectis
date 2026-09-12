# Kinein Vectis — Addendum da Parte 5

# Tree-sitter no Editor da Kinein

> Este documento complementa a **Parte 5 — Editor, Language Intelligence, Indexação, Diagnósticos e Refatoração**.  
> A versão original da Parte 5 já cobria `clangd`, `rust-analyzer`, LSP e diagnósticos, mas não deixava Tree-sitter explícito. Para a Kinein, Tree-sitter deve ser uma camada oficial do editor.

---

## 1. Decisão

A Kinein deve usar Tree-sitter como camada local de parsing incremental.

```text
Tree-sitter = estrutura local rápida do arquivo.
LSP = inteligência semântica profunda do projeto.
```

Tree-sitter não substitui `clangd`, `rust-analyzer`, CMake, Cargo ou o compilador.

---

## 2. Por que usar Tree-sitter

Tree-sitter é útil porque permite ao editor entender a estrutura do arquivo aberto sem depender de um servidor de linguagem completamente inicializado.

Isso melhora:

- syntax highlighting estrutural;
- folding;
- bracket matching;
- seleção expandida por escopo;
- breadcrumbs locais;
- outline local;
- navegação rápida em arquivos grandes;
- fallback quando LSP não está pronto;
- experiência offline/local;
- responsividade do editor.

---

## 3. Separação correta

```text
Editor Buffer
  ↓
SyntaxTreeService / Tree-sitter
  ↓
Recursos locais rápidos

Workspace / Project Model
  ↓
LanguageService / LSP
  ↓
Recursos semânticos profundos
```

### Tree-sitter cuida de

```text
estrutura sintática local
realce estrutural
folding
escopos locais
queries por linguagem
outline local
seleção estrutural
```

### LSP cuida de

```text
autocomplete semântico
goto definition real
find references
rename seguro
type information
diagnósticos de projeto
code actions
refatorações profundas
```

---

## 4. Serviços recomendados

```text
SyntaxTreeService
├── loadGrammar(language)
├── parseDocument(documentId)
├── updateTree(documentId, editRange)
├── getNodeAtPosition(documentId, line, column)
├── getFoldingRanges(documentId)
├── getOutline(documentId)
├── getBreadcrumbs(documentId, position)
└── getHighlightCaptures(documentId, visibleRange)
```

```text
LanguageService
├── startLsp(language, workspace)
├── openDocument(document)
├── updateDocument(change)
├── getCompletion(position)
├── getDiagnostics(document)
├── getCodeActions(range)
├── gotoDefinition(position)
├── findReferences(position)
└── renameSymbol(position, newName)
```

---

## 5. Linguagens iniciais

```text
C
C++
Rust
CMake
TOML
JSON
YAML
Shell
Markdown
```

Futuras:

```text
GLSL
QML
Linker scripts
Device Tree
Kconfig
Python auxiliar
```

---

## 6. Integração com Qt/QML

A UI não deve rodar parsers diretamente. A UI pede dados ao core:

```text
UI Editor
  ↓ syntaxTree.highlights
Kinein Core
  ↓ Tree-sitter parser/query
Highlight captures
  ↓
UI renderiza spans no editor
```

O editor deve receber apenas estruturas prontas:

```json
{
  "documentId": "main.cpp",
  "range": { "startLine": 10, "endLine": 80 },
  "captures": [
    { "range": [12, 4, 12, 10], "scope": "function" },
    { "range": [14, 2, 14, 7], "scope": "keyword" }
  ]
}
```

---

## 7. JSON-RPC sugerido

```text
syntaxTree.parse
syntaxTree.update
syntaxTree.highlights
syntaxTree.foldingRanges
syntaxTree.outline
syntaxTree.breadcrumbs
syntaxTree.nodeAtPosition
syntaxTree.status
```

---

## 8. Critérios de aceite

Tree-sitter estará bem integrado quando:

```text
arquivos C/C++/Rust têm realce mesmo antes do LSP iniciar;
folding funciona por blocos reais;
breadcrumbs locais aparecem rapidamente;
outline local aparece sem depender do indexador global;
editar arquivo grande não trava a UI;
quando LSP fica pronto, recursos semânticos complementam Tree-sitter;
Tree-sitter nunca promete refatoração segura sozinho.
```

---

## 9. Regra final

```text
Tree-sitter torna o editor responsivo e estrutural.
clangd/rust-analyzer tornam a IDE semanticamente inteligente.
A Kinein precisa dos dois.
```
