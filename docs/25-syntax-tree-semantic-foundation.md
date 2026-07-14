# 25 — Fundação sintática incremental e composição semântica

Status: implementado em D3 (F1/F2), protocolo 0.46.0  
Data: 2026-07-14  
Escopo inicial: C, C++ e Rust

## 1. Objetivo

Adicionar uma camada estrutural local, rápida e sempre disponível sem duplicar
o trabalho dos language servers:

```text
buffer aberto
  ├─ SyntaxTreeService / Tree-sitter → highlight, folds, outline e locals
  ├─ clangd ou rust-analyzer         → semântica, tipos e refatorações
  └─ build/linters/testes            → verdade executável do projeto
```

Tree-sitter nunca decide tipos, resolução de overload, borrow checking,
referências globais ou rename. clangd e rust-analyzer continuam sendo as únicas
autoridades semânticas interativas.

## 2. Adoção do componente

Modo A do roadmap de componentes abertos, somente no `kinein-core`:

| Componente | Pin | Licença | Função |
|---|---:|---|---|
| `tree-sitter` | `=0.26.11` | MIT | runtime/parser incremental |
| `tree-sitter-c` | `=0.24.2` | MIT | gramática e queries C |
| `tree-sitter-cpp` | `=0.23.4` | MIT | gramática e queries C++ |
| `tree-sitter-rust` | `=0.24.2` | MIT | gramática e queries Rust |

As crates de gramática carregam o parser C gerado pelo upstream. O código do
Kinein permanece `unsafe`-free; FFI e `unsafe` ficam encapsulados nas
dependências auditadas. Versões são exatas no manifest e reproduzidas pelo
`Cargo.lock`.

## 3. Camadas e ownership

```text
crates/kinein-core/src/lang/
  registry.rs       extensão → linguagem, gramática e queries oficiais
  service.rs        cache incremental limitado dos documentos abertos
  positions.rs      conversão byte UTF-8 → linha/coluna UTF-16 do Qt
  outline.rs        tags sintáticas → árvore local
  folding.rs        ranges estruturais multi-linha

crates/kinein-protocol/src/syntax.rs
  payloads IPC; nenhuma estrutura Tree-sitter atravessa o protocolo

ui/src/editor_highlighter.*
  composição visual: regex fallback < Tree-sitter < semantic tokens LSP

ui/qml/editor/
  debounce, outline e controle de folding; nenhum parser na UI
```

O `SyntaxTreeService` pertence ao `Core`, é descartado ao trocar/fechar o
workspace e mantém no máximo 32 documentos por LRU. Só buffers alcançados pelo
editor são analisados; não há varredura recursiva do workspace nesta fase.

## 4. Contrato incremental

`syntaxTree.update` recebe:

```json
{
  "path": "/workspace/src/main.cpp",
  "content": "...",
  "version": 18
}
```

O core:

1. confina o path ao workspace;
2. detecta C, C++ ou Rust pelo registry;
3. calcula a edição contígua mínima entre o snapshot anterior e o novo;
4. aplica `InputEdit` à árvore anterior;
5. faz parse incremental com a árvore editada;
6. produz uma resposta versionada com captures, folds, outline e locals.

Resultados de versão antiga são descartados pela UI. Arquivos não suportados
retornam resultado vazio tipado, não erro. Conteúdo acima do limite documentado
retorna erro explícito e preserva o último snapshot válido.

## 5. Payload de saída

`SyntaxTreeSnapshotResult` contém:

- `path`, `language`, `version` e `hasErrors`;
- `highlights`: linha 1-based e colunas/comprimento UTF-16;
- `foldingRanges`: linhas 1-based, fechamento exclusivo na UI;
- `outline`: nós aninhados com nome, kind e range;
- `locals`: scopes, definições e referências sintáticas, sempre marcadas como
  fonte `syntax` e nunca promovidas a referência semântica.

Highlights usam as queries oficiais das gramáticas. Captures são normalizados
por prefixo (`function.call` → `function`, `type.builtin` → `type`) apenas na
paleta; o scope original continua no protocolo.

Outline usa as `tags.scm` oficiais. Folding deriva da árvore concreta para
blocos, declarações compostas e comentários multi-linha. Locals usam queries
pequenas, próprias e testadas por linguagem porque as três crates iniciais não
publicam uma query `locals.scm` uniforme.

## 6. Composição no editor

Ordem de pintura por bloco:

1. regex atual, como fallback para linguagens ainda não registradas;
2. captures Tree-sitter para C/C++/Rust;
3. semantic tokens do LSP, que têm precedência;
4. diagnósticos e ocorrências de busca, que acrescentam decoração.

Tree-sitter não é apagado quando o LSP reinicia. Semantic tokens obsoletos são
limpos na troca de documento; a base estrutural permanece.

O outline é uma árvore recolhível local e continua disponível sem clangd ou
rust-analyzer. Um clique navega para a posição do nó. Folding esconde blocos
no `QTextDocument`; a gutter usa a lista de blocos visíveis para preservar a
numeração lógica.

## 7. Concorrência e orçamento

- a UI nunca bloqueia esperando resposta: JSON-RPC continua assíncrono;
- requests têm debounce de 180 ms para estrutura e 600 ms para LSP;
- cache máximo de 32 documentos;
- limite inicial de 4 MiB por buffer;
- somente o documento ativo recebe updates por tecla;
- toda resposta carrega versão para rejeição de resultado obsoleto;
- indexação sintática global, injections e grammars adicionais ficam para
  fatias próprias e só entram com orçamento medido.

## 8. Relação com workspace edits

Tree-sitter pode enriquecer preview e navegação, mas não autoriza alterações.
Rename e code actions continuam vindo do LSP e passam por uma transação única:

```text
WorkspaceEdit LSP
  → plano normalizado e versionado
  → preview
  → validação de snapshots do disco
  → escrita atômica de todos os arquivos
  → rollback em falha
  → resync de watcher, SyntaxTreeService e LSP
```

O executor vive no core. A UI aprova ou cancela planos; ela não aplica edits
nem mantém uma implementação concorrente da transação.

## 9. Critérios de aceite

- C, C++ e Rust recebem highlight antes do LSP;
- uma segunda versão do mesmo documento reutiliza a árvore anterior;
- posições com Unicode chegam corretas ao renderer UTF-16;
- folds correspondem a blocos reais e podem ser alternados pela gutter;
- outline funciona com o LSP indisponível;
- semantic tokens continuam prevalecendo onde existem;
- cache é limitado e limpo ao fechar workspace;
- protocolo, core, C++ e QML têm testes;
- `fmt`, Clippy estrito, testes Rust, qmllint, harness QML e checks C++ passam.

## 10. Fora do escopo desta fatia

- substituir clangd/rust-analyzer;
- AST semântica própria;
- índice sintático recursivo de todo o workspace;
- injections, breadcrumbs estruturais e seleção expansível;
- CMake/TOML/JSON/YAML/Shell/Markdown;
- múltiplos contextos incompatíveis de clangd;
- semantic token delta e scheduler LSP completo.

Esses itens permanecem evoluções incrementais sobre este contrato, não motivo
para alargar o primeiro merge de D3.

## 11. Resultado implementado

O contrato acima foi materializado em 2026-07-14 sem parser paralelo na UI:

- registry e service incremental C/C++/Rust em `kinein-core/src/lang/`;
- queries oficiais para highlight/tags e queries locais pequenas para locals;
- resposta versionada `syntaxTree.update`, stale-drop e debounce no editor;
- highlighter C++ compõe Tree-sitter e semantic tokens na precedência definida;
- folding na gutter e Outline hierárquico continuam disponíveis sem LSP;
- testes cobrem Unicode/UTF-16, incremento, limites/cache, estruturas das três
  linguagens e dispatch; harness QML cobre outline/folding/stale-drop.

Workspace edits conservadores da seção 8 também foram entregues no protocolo
0.47.0, reutilizando uma única primitiva transacional do filesystem.
