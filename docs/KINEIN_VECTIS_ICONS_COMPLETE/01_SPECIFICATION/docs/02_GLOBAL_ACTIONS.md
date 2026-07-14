# Kinein Vectis — Ícones de Ações Globais

Ações presentes na top bar, menus, welcome screen e navegação global.


## 1. `kv.action.new` — Novo

**Prioridade:** P0  
**Superfícies:** Top bar; menus; Project Explorer  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Criar arquivo, pasta, projeto, target ou configuração conforme contexto.

### Metáfora
Folha/documento com sinal de adição.

### Construção geométrica
Folha com canto dobrado mínimo; plus de 6–7 px no canto inferior direito.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
O plus já é o modificador semântico.

### Tooltip
```text
Novo…
```

### Atalho
```text
Ctrl+N
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 2. `kv.action.open` — Abrir

**Prioridade:** P0  
**Superfícies:** Top bar; welcome screen; menus  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Abrir projeto, arquivo ou workspace.

### Metáfora
Pasta aberta com seta curta entrando.

### Construção geométrica
Pasta aberta; seta horizontal curta para dentro; seta não deve cobrir a aba.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Abrir projeto ou arquivo
```

### Atalho
```text
Ctrl+O
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 3. `kv.action.save` — Salvar

**Prioridade:** P0  
**Superfícies:** Editor toolbar; menus  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Persistir o documento atual.

### Metáfora
Disquete abstrato por convenção.

### Construção geométrica
Quadrado com recorte superior e área inferior vazada; versão muito simples.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Ponto quando pendente; check transitório após salvar.

### Tooltip
```text
Salvar
```

### Atalho
```text
Ctrl+S
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 4. `kv.action.save_all` — Salvar tudo

**Prioridade:** P0  
**Superfícies:** Menus; Command Palette  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Salvar todos os documentos modificados.

### Metáfora
Duas folhas/disquetes sobrepostos.

### Construção geométrica
Ícone de save frontal e segunda silhueta deslocada 2 px.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Salvar todos
```

### Atalho
```text
Ctrl+Shift+S
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 5. `kv.action.back` — Voltar

**Prioridade:** P0  
**Superfícies:** Top bar; editor navigation  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Voltar ao ponto anterior de navegação.

### Metáfora
Seta aberta para a esquerda.

### Construção geométrica
Cabeça aberta de 90° e haste horizontal; peso reduzido para não competir com Build/Run.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Voltar
```

### Atalho
```text
Alt+Left
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 6. `kv.action.forward` — Avançar

**Prioridade:** P0  
**Superfícies:** Top bar; editor navigation  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Avançar no histórico de navegação.

### Metáfora
Seta aberta para a direita.

### Construção geométrica
Espelho exato do ícone Voltar.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Avançar
```

### Atalho
```text
Alt+Right
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 7. `kv.action.refresh` — Atualizar

**Prioridade:** P0  
**Superfícies:** Project, Git, toolchains, remote  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Atualizar estado sem alterar a configuração.

### Metáfora
Seta circular única.

### Construção geométrica
Arco de aproximadamente 270° e cabeça triangular pequena; centro vazio.

### Estados
default, hover, active, disabled e spinning enquanto atualiza.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Atualizar
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 8. `kv.action.sync` — Sincronizar

**Prioridade:** P0  
**Superfícies:** Git; Remote; SDK; settings sync  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Sincronizar duas fontes ou lados.

### Metáfora
Duas setas curvas opostas.

### Construção geométrica
Duas metades circulares, cada uma com uma cabeça; manter separação clara.

### Estados
default, hover, active, disabled, spinning e conflict.

### Modifiers e badges
Warning para conflito; check para sincronizado.

### Tooltip
```text
Sincronizar
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 9. `kv.action.command_palette` — Ações

**Prioridade:** P0  
**Superfícies:** Top bar; menus  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Abrir Find Action/Command Palette.

### Metáfora
Prompt de comando com pequeno brilho.

### Construção geométrica
Chevron > à esquerda, linha curta à direita e pequeno ponto/estrela no topo.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Buscar ações e comandos
```

### Atalho
```text
Ctrl+Shift+A
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 10. `kv.action.search_everywhere` — Buscar em tudo

**Prioridade:** P0  
**Superfícies:** Top bar; shortcut overlay  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Abrir busca unificada de arquivos, símbolos, ações e settings.

### Metáfora
Lupa envolvendo quatro pequenos pontos/categorias.

### Construção geométrica
Lupa convencional; quatro pontos internos em grade 2×2, sem perder legibilidade.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Search Everywhere
```

### Atalho
```text
Shift Shift
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 11. `kv.action.notifications` — Notificações

**Prioridade:** P0  
**Superfícies:** Top bar; status  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Abrir central de notificações não intrusivas.

### Metáfora
Sino simples.

### Construção geométrica
Campânula com topo curto e badalo; sem ondas decorativas.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Ponto para notificações novas; número apenas até 9+.

### Tooltip
```text
Notificações
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 12. `kv.action.layout` — Layout

**Prioridade:** P0  
**Superfícies:** Top bar; View menu  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Trocar, salvar ou restaurar layout de painéis.

### Metáfora
Grade assimétrica de três painéis.

### Construção geométrica
Retângulo externo com coluna esquerda estreita e região direita dividida horizontalmente.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Layout da IDE
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 13. `kv.action.fullscreen` — Tela cheia

**Prioridade:** P0  
**Superfícies:** View menu; top bar optional  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Alternar tela cheia.

### Metáfora
Quatro cantos apontando para fora.

### Construção geométrica
Quatro ângulos retos sem caixa externa.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Alternar tela cheia
```

### Atalho
```text
Ctrl+Shift+F11
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 14. `kv.action.zen` — Modo foco

**Prioridade:** P1  
**Superfícies:** View menu; Command Palette  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Ocultar elementos periféricos e focar no editor.

### Metáfora
Retângulo central com quatro linhas recuando.

### Construção geométrica
Área central sólida/contornada; marcas laterais mínimas sugerindo recolhimento.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Modo foco
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 15. `kv.action.help` — Ajuda

**Prioridade:** P0  
**Superfícies:** Top bar; Help menu  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Abrir documentação, atalhos e diagnósticos.

### Metáfora
Círculo com interrogação.

### Construção geométrica
Círculo fino; interrogação com ponto separado e bom espaço negativo.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Ajuda e documentação
```

### Atalho
```text
F1
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 16. `kv.action.close` — Fechar

**Prioridade:** P0  
**Superfícies:** Tabs; dialogs; panels  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Fechar o elemento atual sem sugerir exclusão.

### Metáfora
X geométrico.

### Construção geométrica
Duas diagonais a 45°; extremidades quadradas; área de clique maior que o glyph.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Fechar
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---
