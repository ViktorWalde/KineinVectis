# Kernwerk Studio — Search, Navigation e Indexing Model

> Este documento define busca, navegação e indexação leve do Kernwerk Studio. A IDE deve ser rápida e útil sem criar um indexador pesado cedo demais.

---

## 1. Objetivo

O Kernwerk deve permitir encontrar rapidamente:

```text
arquivos;
texto;
símbolos;
comandos;
ações;
settings;
targets;
run configs;
quality rules;
documentação local.
```

O recurso central deve ser:

```text
Search Everywhere
```

---

## 2. Princípio

Usar ferramentas abertas maduras:

```text
ripgrep para texto;
fd para arquivos;
LSP para símbolos;
CMake File API para targets;
cargo metadata para crates;
Command Registry para ações;
Settings Registry para configurações.
```

Regra:

```text
Não criar indexador semântico próprio no MVP.
```

---

## 3. Tipos de busca

```text
File Search:
buscar arquivo por nome.

Text Search:
buscar texto no projeto.

Symbol Search:
buscar funções/classes/structs via LSP.

Command Search:
buscar comandos da IDE.

Settings Search:
buscar configurações.

Target Search:
buscar targets CMake/Cargo.

Recent Search:
arquivos recentes, comandos recentes, símbolos recentes.
```

---

## 4. Search Everywhere

A UI deve permitir:

```text
Shift Shift
```

Fontes:

```text
arquivos;
comandos;
símbolos;
targets;
settings;
recentes.
```

Resultado deve mostrar:

```text
ícone;
nome;
tipo;
caminho/contexto;
atalho;
ação.
```

Exemplo:

```text
main.cpp                       File       src/main.cpp
build.run                      Command    Build
KernwerkCore                   Symbol     crates/kernwerk-core
debug-strict                   CMake      Preset
Quality Center                 Setting    Tools > Quality
```

Estado MVP atual:

```text
Ctrl+Shift+N / Ctrl+Shift+A abre Search Everywhere inicial;
fontes implementadas: comandos via command.list e arquivos via fs.findFiles/fd;
fontes pendentes: símbolos, targets, settings e recentes.
```

---

## 5. Respeitar ignores

Busca deve respeitar:

```text
.gitignore;
.ignore;
.kernwerkignore futuro;
configurações do usuário.
```

Ignorar por padrão:

```text
.git/
target/
build/
cmake-build-*/
node_modules/
.cache/
.idea/
.vscode/
dist/
out/
```

---

## 6. ripgrep

Text search deve usar `rg`.

Comandos conceituais:

```bash
rg "pattern" --json --hidden --glob '!target' --glob '!build'
```

Vantagens:

```text
rápido;
maduro;
respeita .gitignore;
saída estruturável;
funciona em projetos grandes.
```

---

## 7. fd

File search deve usar `fd`.

Comandos conceituais:

```bash
fd "main" .
```

A IDE pode também manter cache leve de file tree.

---

## 8. Símbolos via LSP

Para C/C++:

```text
clangd workspace/symbol
```

Para Rust:

```text
rust-analyzer workspace/symbol
```

Regra:

```text
Se LSP não estiver pronto, mostrar resultados parciais.
```

---

## 9. Cache leve

Cache permitido:

```text
lista de arquivos;
recent files;
recent commands;
últimos resultados de símbolos;
targets CMake/Cargo;
settings index.
```

Cache proibido no MVP:

```text
index semântico próprio profundo;
banco gigante;
indexação contínua agressiva.
```

---

## 10. File watcher

Mudanças no filesystem devem usar debounce.

Errado:

```text
arquivo mudou → reindexa tudo imediatamente
```

Certo:

```text
arquivo mudou
↓
agrupar eventos
↓
atualizar cache afetado
↓
notificar UI
```

---

## 11. Performance

Regras:

```text
busca cancelável;
não bloquear UI;
limite de resultados;
streaming de resultados;
não carregar arquivo inteiro gigante;
mostrar progresso em busca longa.
```

---

## 12. Interface de busca

Painel Search:

```text
campo de busca;
escopo;
case sensitive;
regex;
whole word;
include globs;
exclude globs;
resultados agrupados por arquivo;
preview;
replace futuro.
```

---

## 13. Replace in Files

Futuro, não MVP.

Regra obrigatória:

```text
Replace em múltiplos arquivos deve mostrar preview/diff antes de aplicar.
```

---

## 14. Critérios de aceite

Busca MVP pronta quando:

```text
busca arquivos;
busca texto com ripgrep;
respeita .gitignore;
ignora target/build;
cancela busca;
mostra resultados com arquivo/linha;
Search Everywhere lista comandos e arquivos;
não bloqueia UI.
```

---

## 15. Decisão final

Busca profissional não precisa começar com indexador próprio.

Ela deve orquestrar bem `ripgrep`, `fd`, LSP e caches leves.
