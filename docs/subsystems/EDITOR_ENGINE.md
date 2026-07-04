# Kernwerk Studio — Editor Engine

> Este documento define como o editor de texto/código do Kernwerk deve ser tratado. O editor é o coração visual da IDE e deve evoluir com cuidado, sem tentar ser perfeito antes do Core estar sólido.

---

## 1. Objetivo

O editor do Kernwerk deve permitir editar código com segurança, integrar LSP, mostrar diagnósticos, oferecer ações de código e manter performance.

Regra inicial:

```text
No MVP, editor simples.
No futuro, editor robusto.
Não tentar criar um editor perfeito antes do Core, Commands, Tooling e Quality funcionarem.
```

---

## 2. Escopo do MVP

O editor inicial deve suportar:

```text
abrir arquivo;
editar texto;
salvar arquivo;
salvar como;
indentação básica com Tab/Shift+Tab;
autoindent ao pressionar Enter;
número de linha;
posição linha/coluna;
abas de arquivo;
detecção de alteração externa básica;
diagnóstico visual simples;
atalhos principais;
integração mínima com comandos.
```

Fora do MVP:

```text
multi-cursor;
minimap;
semantic highlighting completo;
folding avançado;
refatoração avançada;
editor de diff avançado;
renderização otimizada para arquivos gigantes.
```

---

## 3. Modelo de documento

Todo arquivo aberto deve virar um `EditorDocument`.

Campos conceituais:

```json
{
  "id": "doc-1",
  "path": "src/main.cpp",
  "language": "cpp",
  "encoding": "utf-8",
  "lineEnding": "LF",
  "isDirty": true,
  "isReadOnly": false,
  "version": 12,
  "lastSavedVersion": 10
}
```

Regra:

```text
Todo documento deve ter versão incremental para integração com LSP.
```

---

## 4. Encoding e line endings

A IDE deve lidar com:

```text
UTF-8;
UTF-8 com BOM;
line endings LF;
line endings CRLF;
arquivo sem newline final;
arquivo read-only.
```

Comportamento profissional:

```text
não converter line endings silenciosamente;
mostrar status na barra;
permitir converter explicitamente;
preservar padrão existente quando possível.
```

---

## 5. Arquivos grandes

A IDE deve detectar arquivos grandes.

Exemplos de limites iniciais:

```text
> 1 MB: avisar sobre recursos pesados.
> 5 MB: desativar semantic highlighting.
> 10 MB: abrir em Large File Mode.
```

Large File Mode:

```text
sem semantic highlighting;
sem diagnostics em tempo real;
sem minimap;
busca simples;
edição básica;
salvar seguro.
```

Regra:

```text
Arquivo grande nunca deve travar a UI.
```

---

## 6. Undo/Redo

Undo/redo deve ser confiável.

Regras:

```text
cada documento tem sua própria stack;
salvar não deve limpar undo;
ações automáticas devem ser agrupadas;
refatoração deve ser uma transação reversível quando possível.
```

---

## 7. Integração com LSP

LSPs iniciais:

```text
clangd
rust-analyzer
```

Recursos via LSP:

```text
diagnostics;
completion;
hover;
go to definition;
find references;
rename;
code actions;
semantic tokens;
inlay hints.
```

Regra:

```text
O editor não deve implementar inteligência semântica própria de C++/Rust no início.
Ele deve consumir LSP.
```

---

## 8. Diagnostics inline

Diagnósticos devem aparecer de forma clara:

```text
sublinhado no código;
ícone na gutter;
mensagem no hover;
entrada no Problems panel;
atalho para próximo erro;
ação de quick fix quando disponível.
```

Severidades:

```text
Error
Warning
Info
Hint
```

---

## 9. Autocomplete

Autocomplete deve:

```text
ser acionado por LSP;
não bloquear digitação;
mostrar origem da sugestão;
ter navegação por teclado;
aceitar com Enter/Tab configurável;
fechar com Escape;
não abrir agressivamente em arquivo grande.
```

---

## 10. Hover

Hover deve mostrar:

```text
tipo;
documentação;
mensagem de diagnóstico;
origem da ferramenta;
ações disponíveis.
```

Regra:

```text
Hover deve ser útil, não intrusivo.
```

---

## 11. Code Actions

Code actions podem vir de:

```text
LSP;
Quality Center;
Kernwerk refactoring;
IA sob demanda.
```

Toda action que modifica arquivos deve passar pelo sistema de refatoração/diff.

---

## 12. Syntax Highlighting

MVP:

```text
highlight básico por linguagem;
fallback para plain text.
```

Futuro:

```text
Tree-sitter;
semantic tokens via LSP;
temas configuráveis;
injeção de linguagem;
QML/CMake/TOML/JSON/YAML.
```

---

## 13. Salvamento

Regras:

```text
salvar deve ser atômico quando possível;
não sobrescrever alteração externa sem avisar;
mostrar conflito de arquivo alterado fora da IDE;
permitir reload from disk;
permitir compare before overwrite.
```

---

## 14. Autosave

Autosave deve ser opcional.

Modos:

```text
off;
on focus lost;
after delay;
before build/run.
```

Regra:

```text
Autosave nunca deve esconder alterações críticas sem indicação visual.
```

---

## 15. Crash Recovery

O editor deve salvar estado de recuperação:

```text
arquivos abertos;
conteúdo não salvo;
posição do cursor;
abas;
layout.
```

Ao reiniciar após crash:

```text
Kernwerk detectou sessão não finalizada.
[Restaurar] [Descartar] [Comparar]
```

---

## 16. Diff Editor

Futuro necessário para:

```text
preview de refatoração;
generated files policy;
AI patches;
Git diff;
format changes;
Quality Center apply profile.
```

MVP pode começar com diff textual simples.

---

## 17. Critérios de aceite

Editor MVP pronto quando:

```text
abre/salva arquivo;
mostra abas;
marca dirty;
mostra linha/coluna;
preserva encoding/line ending básico;
integra diagnostics simples;
não trava em arquivo comum;
detecta alteração externa básica;
passa por command system.
```

---

## 18. Decisão final

O editor deve crescer por etapas.

Não sacrificar arquitetura do Kernwerk tentando criar editor perfeito cedo demais.
