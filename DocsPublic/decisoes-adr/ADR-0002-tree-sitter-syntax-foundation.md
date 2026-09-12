# ADR-0002 — Tree-sitter para a fundação sintática local

- Status: aceito
- Data: 2026-07-14
- Escopo: D3 / protocolo 0.46.0
- Nota datada (2026-09-12): a fundação ganhou a **quarta gramática, Python**
  (`tree-sitter-python` 0.25.0, MIT — LICENSE lido no repositório), pela
  decisão do autor de 2026-09-11 (Python entra como vertical nativa,
  `roadmaps/41` bloco B). Mesma forma: highlights e tags oficiais da
  gramática, `locals` mínimo escrito aqui. O que vale para C/C++/Rust neste
  ADR vale para Python; o índice do projeto inteiro (`roadmaps/42` P0) passou
  a ver as declarações dos `.py` no mesmo gesto.

## Contexto

O fallback regex do editor é rápido, mas não representa estrutura real e não
consegue alimentar folding ou outline. clangd e rust-analyzer têm semântica
profunda, porém dependem de inicialização, configuração de projeto e processos
externos. As specs exigem uma camada estrutural local que continue útil durante
startup, crash ou ausência do LSP.

Escrever parsers de C, C++ ou Rust violaria a regra central do projeto. O
roadmap de adaptação determina auditoria e integração direta de componentes
abertos maduros.

## Decisão

Adotar o runtime Tree-sitter `=0.26.11` e as gramáticas oficiais C `=0.24.2`,
C++ `=0.23.4` e Rust `=0.24.2`, em Modo A e somente no `kinein-core`.

O `SyntaxTreeService` mantém árvores incrementais de buffers abertos, calcula
uma edição contígua mínima entre versões e limita o cache a 32 documentos de
até 4 MiB. Queries oficiais fornecem highlights e tags; queries locais pequenas
e testadas fornecem scopes/definitions/references sintáticos. Folding deriva da
árvore concreta.

O protocolo retorna apenas posições UTF-16 e payloads neutros. A UI não recebe
objetos Tree-sitter nem executa parsers. Semantic tokens do LSP prevalecem sobre
captures sintáticos; refatorações continuam exclusivas de clangd e
rust-analyzer.

## Auditoria

- upstreams oficiais da organização Tree-sitter;
- licença MIT em runtime e gramáticas;
- runtime MSRV 1.77, abaixo do MSRV 1.85 do workspace;
- sem rede, telemetria, shell, segredos ou upload de conteúdo;
- código nativo limitado ao runtime e aos parsers gerados dos upstreams;
- Kinein continua com `unsafe` proibido;
- pins exatos no manifest e lockfile;
- transitivos e verificações registrados em
  `DocsPublic/integracoes/registro-de-componentes-abertos.json`.

## Consequências

Positivas: highlight estrutural independe do LSP, folding e outline compartilham
a mesma árvore, Unicode é normalizado no core e o parser incremental evita
reprocessar todo o buffer a cada edição.

Custos: três parsers C gerados aumentam build e binário; queries upstream podem
mudar junto das gramáticas; a primeira abertura compila queries lazy. Os pins e
testes tornam essas mudanças explícitas.

## Alternativas rejeitadas

- ampliar o highlighter regex: não produz estrutura confiável;
- usar somente semantic tokens LSP: deixa startup/falha do servidor sem base;
- rodar Tree-sitter no QML/C++: duplica estado e move lógica ao frontend;
- usar AST de clang/rustc diretamente: custo, tolerância a código incompleto e
  acoplamento inadequados para feedback por tecla;
- indexar todo o workspace já nesta fatia: contraria o orçamento on-demand.
