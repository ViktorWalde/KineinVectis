# Kernwerk Studio — Refactoring e Code Actions

> Este documento define como o Kernwerk deve tratar refatorações, quick fixes e ações de código. A regra principal é segurança: qualquer alteração em arquivos deve ser previsível, revisável e reversível quando possível.

---

## 1. Objetivo

O Kernwerk deve oferecer refatoração e code actions de forma profissional, usando:

```text
LSP;
ferramentas oficiais;
ações próprias do Kernwerk;
preview/diff;
transações;
confirmação;
testes.
```

---

## 2. Fontes de ações

Ações podem vir de:

```text
clangd;
rust-analyzer;
Quality Center;
CMake integration;
Cargo integration;
Qt/GTK templates;
Embedded tooling;
IA sob demanda;
ações próprias do Kernwerk.
```

---

## 3. Tipos de ação

```text
Navigation Action:
go to definition, find references.

Quick Fix:
corrigir include, aplicar sugestão do LSP.

Refactoring:
rename, move file, extract, create module.

Project Action:
adicionar target CMake, criar crate, atualizar Cargo.

Generated Change:
aplicar profile strict, gerar .clang-tidy.

AI Suggested Action:
patch sugerido por IA, sempre com preview.
```

---

## 4. Regra principal

```text
Toda ação que altera arquivos deve mostrar preview/diff antes de aplicar,
exceto edições explícitas feitas diretamente pelo usuário no editor.
```

---

## 5. Code Actions via LSP

C/C++ via clangd:

```text
rename symbol;
organize includes;
fix include;
apply quick fix;
go to definition;
find references;
hover;
code action.
```

Rust via rust-analyzer:

```text
rename;
extract variable/function quando disponível;
add missing import;
qualify path;
generate impl;
module actions;
go to definition;
find references.
```

---

## 6. Refatorações próprias do Kernwerk

### 6.1 C/C++

```text
criar par .hpp/.cpp;
mover arquivo e atualizar includes;
renomear arquivo e atualizar includes;
adicionar arquivo ao target CMake;
criar novo target CMake;
criar teste Catch2/GoogleTest;
adicionar clang-format;
adicionar clang-tidy;
adicionar sanitizers;
converter projeto para CMakePresets.
```

### 6.2 Rust

```text
criar módulo;
renomear módulo e atualizar mod/use;
criar crate no workspace;
criar bin/lib/test/example;
adicionar feature;
adicionar teste;
adicionar cargo deny;
adicionar cargo audit;
```

### 6.3 Qt/GTK

```text
criar QML component;
criar .qrc;
adicionar Qt module;
criar janela Qt Widgets;
criar GTK window;
adicionar pkg-config ao CMake;
adicionar gtk-rs ao Cargo.
```

---

## 7. Refactoring Transaction

Toda refatoração deve virar uma transação.

Modelo:

```json
{
  "id": "refactor-001",
  "title": "Move C++ file",
  "files": [
    {
      "path": "src/old.cpp",
      "operation": "rename",
      "newPath": "src/core/new.cpp"
    },
    {
      "path": "CMakeLists.txt",
      "operation": "modify"
    }
  ],
  "requiresConfirmation": true
}
```

Transação deve permitir:

```text
preview;
apply;
cancel;
rollback quando possível;
log.
```

---

## 8. Preview/Diff

Diff deve mostrar:

```text
arquivo;
linhas removidas;
linhas adicionadas;
motivo da alteração;
ferramenta que sugeriu;
risco;
ações alternativas.
```

Para múltiplos arquivos:

```text
árvore de arquivos afetados;
checkbox por arquivo;
aplicar tudo;
aplicar parcialmente;
cancelar.
```

---

## 9. IA e refatoração

IA pode sugerir patches, mas não aplicar sozinha.

Regras:

```text
mostrar contexto enviado;
mostrar patch;
mostrar arquivos afetados;
exigir confirmação;
permitir aplicar parcialmente;
nunca executar comando destrutivo automaticamente.
```

---

## 10. Segurança

Ações perigosas:

```text
delete file;
delete directory;
git reset;
git clean;
overwrite generated files;
mass replace;
move many files;
remote changes.
```

Exigem confirmação explícita.

---

## 11. Testes

Refatorações devem ter testes.

Tipos:

```text
unit tests;
golden tests;
fixture projects;
diff expected;
rollback tests quando possível.
```

Exemplo:

```text
move header/source pair
↓
atualiza include
↓
atualiza CMake
↓
resultado comparado com expected
```

---

## 12. Critérios de aceite

Refactoring MVP pronto quando:

```text
executa rename via LSP;
aplica quick fix via LSP;
mostra preview para alterações multi-file;
tem transação básica;
não sobrescreve sem confirmação;
tem golden test para pelo menos uma refatoração própria.
```

---

## 13. Decisão final

Refatoração no Kernwerk deve ser conservadora, explícita e auditável.

Melhor ter poucas refatorações confiáveis do que muitas ações perigosas.
