# Kinein Vectis — Source Control e Git

Ícones para o fluxo Git sem copiar o logotipo do Git como metáfora de ação.


## 1. `kv.git.commit` — Commit

**Prioridade:** P0  
**Superfícies:** Git toolbar; top bar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Criar commit das alterações preparadas.

### Metáfora
Check inserido em nó de histórico.

### Construção geométrica
Círculo na linha vertical do histórico; check central.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Commit alterações
```

### Atalho
```text
Ctrl+K
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 2. `kv.git.push` — Push

**Prioridade:** P0  
**Superfícies:** Git toolbar; branch menu  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Enviar commits ao remoto.

### Metáfora
Seta para cima saindo de branch.

### Construção geométrica
Haste vertical com ramo curto e seta aberta para cima.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Número de commits pendentes.

### Tooltip
```text
Push
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 3. `kv.git.pull` — Pull

**Prioridade:** P0  
**Superfícies:** Git toolbar; branch menu  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Trazer e integrar alterações remotas.

### Metáfora
Seta para baixo entrando em branch.

### Construção geométrica
Espelho vertical do push; não usar download genérico.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Warning quando houver conflito.

### Tooltip
```text
Pull
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 4. `kv.git.fetch` — Fetch

**Prioridade:** P0  
**Superfícies:** Git toolbar; branch menu  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Atualizar referências remotas sem integrar.

### Metáfora
Nuvem/remoto com seta curta para baixo e sem branch merge.

### Construção geométrica
Pequeno nó remoto no topo, seta para cache/local abaixo.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Fetch
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 5. `kv.git.branch` — Branch

**Prioridade:** P0  
**Superfícies:** Top bar; Git panel  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Selecionar ou gerenciar branch.

### Metáfora
Linha com bifurcação.

### Construção geométrica
Dois nós e uma bifurcação ascendente; forma diferente de kv.view.git.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Branch atual
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 6. `kv.git.merge` — Merge

**Prioridade:** P0  
**Superfícies:** Git toolbar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Mesclar uma branch em outra.

### Metáfora
Duas linhas convergindo em um nó.

### Construção geométrica
Duas hastes inferiores curvas convergindo em círculo superior.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Merge
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 7. `kv.git.rebase` — Rebase

**Prioridade:** P0  
**Superfícies:** Git toolbar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Reposicionar commits sobre outra base.

### Metáfora
Sequência de nós mudando de trilho.

### Construção geométrica
Duas linhas verticais; dois nós com seta diagonal entre trilhos.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Rebase
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 8. `kv.git.stash` — Stash

**Prioridade:** P0  
**Superfícies:** Git toolbar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Guardar mudanças temporariamente.

### Metáfora
Caixa/arquivo com seta entrando.

### Construção geométrica
Caixa aberta no topo; seta curta para dentro.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Guardar alterações no stash
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 9. `kv.git.stage` — Stage

**Prioridade:** P0  
**Superfícies:** Changes list  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Adicionar arquivo ou hunk ao index.

### Metáfora
Plus dentro de bandeja Git.

### Construção geométrica
Linha de mudança com plus à direita; base horizontal curta.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Check quando já staged.

### Tooltip
```text
Adicionar ao stage
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 10. `kv.git.unstage` — Unstage

**Prioridade:** P0  
**Superfícies:** Staged changes list  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Remover arquivo ou hunk do index sem descartar conteúdo.

### Metáfora
Minus dentro de bandeja.

### Construção geométrica
Mesmo corpo de stage com traço horizontal.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Remover do stage
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 11. `kv.git.conflict` — Conflito

**Prioridade:** P0  
**Superfícies:** Project tree; Problems; Git  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Indicar conflito de merge/rebase.

### Metáfora
Duas setas convergentes interrompidas por losango.

### Construção geométrica
Linhas diagonais; losango central vazado; deve diferir de warning.

### Estados
unresolved, resolved e partially resolved.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Conflito de versionamento
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 12. `kv.git.blame` — Blame

**Prioridade:** P0  
**Superfícies:** Editor gutter; context menu  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Mostrar autoria e commit da linha.

### Metáfora
Pessoa mínima ao lado de linha de histórico.

### Construção geométrica
Cabeça/corpo simples e linha vertical com um nó.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Anotar autoria da linha
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---
