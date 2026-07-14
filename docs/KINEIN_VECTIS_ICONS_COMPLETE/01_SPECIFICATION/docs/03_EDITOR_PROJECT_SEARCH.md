# Kinein Vectis — Editor, Projeto e Busca

Ícones usados no editor, abas, Project Explorer, refatorações e busca.


## 1. `kv.editor.split_right` — Dividir à direita

**Prioridade:** P0  
**Superfícies:** Editor tabs; layout menu  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Criar grupo de editor à direita.

### Metáfora
Janela dividida verticalmente com seta para a direita.

### Construção geométrica
Retângulo 16×14; divisor central; seta curta apontando à metade direita.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Dividir editor à direita
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 2. `kv.editor.split_down` — Dividir abaixo

**Prioridade:** P0  
**Superfícies:** Editor tabs; layout menu  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Criar grupo de editor abaixo.

### Metáfora
Janela dividida horizontalmente com seta para baixo.

### Construção geométrica
Retângulo 16×14; divisor horizontal; seta curta para a parte inferior.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Dividir editor abaixo
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 3. `kv.editor.pin` — Fixar aba

**Prioridade:** P0  
**Superfícies:** Editor tabs; tool windows  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Impedir que uma aba seja substituída ou fechada por política de preview.

### Metáfora
Alfinete inclinado.

### Construção geométrica
Cabeça pequena, haste diagonal e ponta curta; manter centro de massa equilibrado.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Slash diagonal para unpin.

### Tooltip
```text
Fixar aba
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 4. `kv.editor.preview` — Aba de pré-visualização

**Prioridade:** P0  
**Superfícies:** Editor tabs; status  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Indicar aba temporária que poderá ser substituída.

### Metáfora
Olho dentro de uma aba.

### Construção geométrica
Contorno de olho mínimo; aba/retângulo quase invisível para não poluir.

### Estados
Indicador passivo; não funciona como botão..

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Aba de pré-visualização
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 5. `kv.editor.modified` — Arquivo modificado

**Prioridade:** P0  
**Superfícies:** Editor tabs; Project Explorer  
**Master:** 4/4 px  
**Implementação:** SVG simbólico próprio.

### Função
Indicar alterações ainda não persistidas.

### Metáfora
Ponto sólido.

### Construção geométrica
Círculo de 4 px, sem contorno; nunca usar apenas cor sem tooltip/estado textual.

### Estados
modified, saving, saved e conflict.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Alterações não salvas
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 6. `kv.editor.readonly` — Somente leitura

**Prioridade:** P0  
**Superfícies:** Editor tabs; status; Project  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Indicar documento sem permissão de escrita.

### Metáfora
Cadeado fechado.

### Construção geométrica
Arco largo e corpo quadrado; sem chave interna.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Somente leitura
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 7. `kv.editor.rename` — Renomear símbolo

**Prioridade:** P0  
**Superfícies:** Context menu; quick actions  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Executar rename semântico via LSP.

### Metáfora
Cursor de texto com pequena seta de troca.

### Construção geométrica
I-beam à esquerda; duas setas horizontais curtas à direita.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Renomear símbolo
```

### Atalho
```text
Shift+F6
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 8. `kv.editor.quick_fix` — Correção rápida

**Prioridade:** P0  
**Superfícies:** Gutter; hover; context menu  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Abrir code actions e correções disponíveis.

### Metáfora
Lâmpada simplificada com pequeno brilho.

### Construção geométrica
Bulbo circular simples e base curta; dois raios no máximo.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Ponto para action automática; warning para ação de risco.

### Tooltip
```text
Mostrar correções rápidas
```

### Atalho
```text
Alt+Enter
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 9. `kv.editor.definition` — Ir para definição

**Prioridade:** P0  
**Superfícies:** Context menu; Search Everywhere  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Navegar para a definição do símbolo.

### Metáfora
Alvo circular com seta entrando.

### Construção geométrica
Círculo aberto e pequena seta diagonal para o centro.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Ir para definição
```

### Atalho
```text
Ctrl+B
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 10. `kv.editor.references` — Encontrar usos

**Prioridade:** P0  
**Superfícies:** Context menu; Problems/Usages  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Listar referências ao símbolo.

### Metáfora
Nó central conectado a três marcas.

### Construção geométrica
Círculo central pequeno, três nós externos e linhas curtas.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Encontrar usos
```

### Atalho
```text
Alt+F7
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 11. `kv.editor.format` — Formatar

**Prioridade:** P0  
**Superfícies:** Toolbar; context menu; Quality  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Executar formatter no arquivo ou seleção.

### Metáfora
Linhas de código alinhadas por uma régua.

### Construção geométrica
Três linhas horizontais de comprimentos distintos; barra vertical lateral indicando alinhamento.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Check para formatado; ponto para format on save.

### Tooltip
```text
Formatar código
```

### Atalho
```text
Ctrl+Alt+L
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 12. `kv.editor.wrap` — Quebra de linha

**Prioridade:** P0  
**Superfícies:** Editor toolbar; status  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Alternar word wrap.

### Metáfora
Linha longa curvando para a linha seguinte.

### Construção geométrica
Duas linhas; seta curva no fim da superior.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Alternar quebra de linha
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 13. `kv.project.new_file` — Novo arquivo

**Prioridade:** P0  
**Superfícies:** Project Explorer  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Criar arquivo no diretório selecionado.

### Metáfora
Documento com plus.

### Construção geométrica
Reutiliza base kv.action.new com geometria de arquivo explícita.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Novo arquivo
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 14. `kv.project.new_folder` — Nova pasta

**Prioridade:** P0  
**Superfícies:** Project Explorer  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Criar diretório.

### Metáfora
Pasta com plus.

### Construção geométrica
Pasta baixa; plus no canto inferior direito, separado 1–2 px.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Nova pasta
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 15. `kv.project.collapse_all` — Recolher tudo

**Prioridade:** P0  
**Superfícies:** Project Explorer toolbar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Recolher todos os nós expandidos.

### Metáfora
Duas setas apontando para centro sobre árvore.

### Construção geométrica
Três linhas de árvore e duas setas verticais para dentro.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Recolher tudo
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 16. `kv.project.follow_active` — Seguir arquivo ativo

**Prioridade:** P0  
**Superfícies:** Project Explorer toolbar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Selecionar automaticamente no Project Explorer o arquivo ativo.

### Metáfora
Mira pequena sobre arquivo.

### Construção geométrica
Documento simples com círculo-alvo no centro inferior.

### Estados
off, on e temporarily suspended.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Sempre selecionar arquivo aberto
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 17. `kv.search.regex` — Regex

**Prioridade:** P0  
**Superfícies:** Search toolbar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Alternar busca por expressão regular.

### Metáfora
Ponto e asterisco por convenção.

### Construção geométrica
Glyph .* com fonte geométrica própria convertida em paths.

### Estados
off e on.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Usar expressão regular
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 18. `kv.search.case_sensitive` — Diferenciar maiúsculas

**Prioridade:** P0  
**Superfícies:** Search toolbar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Alternar case-sensitive.

### Metáfora
A maiúsculo e a minúsculo.

### Construção geométrica
Aa em paths próprios, sem depender da fonte da UI.

### Estados
off e on.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Diferenciar maiúsculas e minúsculas
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 19. `kv.search.whole_word` — Palavra inteira

**Prioridade:** P0  
**Superfícies:** Search toolbar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Restringir busca a palavras inteiras.

### Metáfora
Texto 'ab' entre barras de limite.

### Construção geométrica
|ab| estilizado, com barras curtas e glyphs vetoriais.

### Estados
off e on.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Palavra inteira
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 20. `kv.search.replace` — Substituir

**Prioridade:** P0  
**Superfícies:** Search panel  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Alternar ou executar substituição.

### Metáfora
Duas linhas de texto ligadas por seta para baixo.

### Construção geométrica
Linha superior curta, seta vertical, linha inferior; sem usar recycle/sync.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Preview eye para substituição em massa.

### Tooltip
```text
Substituir
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---
