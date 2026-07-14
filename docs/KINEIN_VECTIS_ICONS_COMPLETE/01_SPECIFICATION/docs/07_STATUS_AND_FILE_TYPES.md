# Kinein Vectis — Status e Tipos de Arquivo

Ícones compactos da status bar e do Project Explorer.


## 1. `kv.status.lsp` — LSP

**Prioridade:** P0  
**Superfícies:** Status bar; tool status  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Mostrar saúde e atividade do language server.

### Metáfora
Símbolo de linguagem/conexão em nós.

### Construção geométrica
Três nós em triângulo com pequeno cursor central.

### Estados
starting, ready, indexing, degraded, failed e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Language Server
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 2. `kv.status.toolchain` — Toolchain

**Prioridade:** P0  
**Superfícies:** Status bar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Mostrar compilador/toolchain ativo.

### Metáfora
Martelo técnico cruzando chip.

### Construção geométrica
Chip pequeno ao fundo e ferramenta linear à frente; sem estética de construção civil.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Toolchain ativo
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 3. `kv.status.encoding` — Encoding

**Prioridade:** P0  
**Superfícies:** Status bar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Abrir seleção de encoding do arquivo.

### Metáfora
Documento com caracteres binários/letra.

### Construção geométrica
Folha simples e dois glyphs vetoriais 'A' e '0' pequenos.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Codificação do arquivo
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 4. `kv.status.line_endings` — Fim de linha

**Prioridade:** P0  
**Superfícies:** Status bar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Selecionar LF/CRLF.

### Metáfora
Seta de retorno de linha.

### Construção geométrica
Seta em L, similar a return, com ponta aberta.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Fim de linha
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 5. `kv.status.lock` — Seguro/verificado

**Prioridade:** P0  
**Superfícies:** Status; SSH; workspace trust  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Indicar confiança, lock ou política protegida.

### Metáfora
Cadeado fechado.

### Construção geométrica
Mesmo cadeado readonly, mas uso contextual e tooltip obrigatório.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Conexão ou configuração protegida
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 6. `kv.file.cpp` — Arquivo C++

**Prioridade:** P0  
**Superfícies:** Project Explorer; tabs  
**Master:** 16 px  
**Implementação:** SVG simbólico próprio.

### Função
Identificar source C++.

### Metáfora
Folha com C++ estilizado.

### Construção geométrica
Documento com glyphs C e dois plus vetoriais; accent discreto permitido.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Arquivo-fonte C++
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 7. `kv.file.c` — Arquivo C

**Prioridade:** P0  
**Superfícies:** Project Explorer; tabs  
**Master:** 16 px  
**Implementação:** SVG simbólico próprio.

### Função
Identificar source C.

### Metáfora
Folha com C.

### Construção geométrica
Documento com C vetorial central.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Arquivo-fonte C
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 8. `kv.file.header` — Header

**Prioridade:** P0  
**Superfícies:** Project Explorer; tabs  
**Master:** 16 px  
**Implementação:** SVG simbólico próprio.

### Função
Identificar .h/.hpp.

### Metáfora
Folha com H e colchetes.

### Construção geométrica
Documento com H vetorial; pequenos colchetes laterais opcionais.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Header C/C++
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 9. `kv.file.rust` — Arquivo Rust

**Prioridade:** P0  
**Superfícies:** Project Explorer; tabs  
**Master:** 16 px  
**Implementação:** SVG simbólico próprio.

### Função
Identificar arquivo Rust sem copiar marca de terceiros.

### Metáfora
Folha com engrenagem mínima e R.

### Construção geométrica
Documento; R vetorial e três dentes discretos no contorno inferior.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Arquivo Rust
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 10. `kv.file.cmake` — CMake

**Prioridade:** P0  
**Superfícies:** Project Explorer; tabs  
**Master:** 16 px  
**Implementação:** SVG simbólico próprio.

### Função
Identificar CMakeLists/preset/toolchain.

### Metáfora
Folha com triângulo de build e C.

### Construção geométrica
Documento; triângulo contornado e C pequeno; não copiar logo oficial.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Arquivo CMake
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 11. `kv.file.qml` — QML

**Prioridade:** P0  
**Superfícies:** Project Explorer; tabs  
**Master:** 16 px  
**Implementação:** SVG simbólico próprio.

### Função
Identificar QML.

### Metáfora
Folha com Q e dois blocos de UI.

### Construção geométrica
Documento; Q vetorial e dois retângulos pequenos em grade.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Arquivo QML
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 12. `kv.file.toml` — TOML/Cargo

**Prioridade:** P0  
**Superfícies:** Project Explorer; tabs  
**Master:** 16 px  
**Implementação:** SVG simbólico próprio.

### Função
Identificar TOML e manifestos Cargo.

### Metáfora
Folha com chave/valor.

### Construção geométrica
Documento; duas linhas 'key = value' abstratas.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Arquivo TOML
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 13. `kv.file.device_tree` — Device Tree

**Prioridade:** P0  
**Superfícies:** Project Explorer; tabs  
**Master:** 16 px  
**Implementação:** SVG simbólico próprio.

### Função
Identificar DTS/DTSI.

### Metáfora
Árvore hierárquica dentro de folha.

### Construção geométrica
Documento com nó raiz e três ramos.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Device Tree
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 14. `kv.file.linker_script` — Linker script

**Prioridade:** P0  
**Superfícies:** Project Explorer; tabs  
**Master:** 16 px  
**Implementação:** SVG simbólico próprio.

### Função
Identificar scripts de link e mapa de memória.

### Metáfora
Folha com blocos de memória ligados.

### Construção geométrica
Documento com três blocos horizontais e setas/linhas de ligação.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Linker script
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---
