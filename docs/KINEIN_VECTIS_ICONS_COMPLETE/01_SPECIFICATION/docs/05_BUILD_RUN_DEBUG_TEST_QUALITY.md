# Kinein Vectis — Build, Run, Debug, Tests e Quality

Ícones de execução profissional, debugging, testes e qualidade.


## 1. `kv.build.configure` — Configurar

**Prioridade:** P0  
**Superfícies:** Build toolbar; CMake panel  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Executar etapa de configure/generate.

### Metáfora
Engrenagem sobre blueprint.

### Construção geométrica
Pequena engrenagem no canto e duas linhas técnicas no fundo.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Check, x ou spinner.

### Tooltip
```text
Configurar projeto
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 2. `kv.build.compile` — Compilar

**Prioridade:** P0  
**Superfícies:** Build toolbar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Compilar target selecionado.

### Metáfora
Blocos/código convergindo em artefato.

### Construção geométrica
Duas folhas à esquerda, seta curta e cubo/arquivo binário à direita.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Compilar target
```

### Atalho
```text
Ctrl+F9
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 3. `kv.build.rebuild` — Recompilar

**Prioridade:** P0  
**Superfícies:** Build toolbar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Limpar e compilar novamente.

### Metáfora
Ícone de compile com seta circular.

### Construção geométrica
Compile como base; pequeno modifier refresh de 6 px.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Rebuild
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 4. `kv.build.clean` — Limpar build

**Prioridade:** P0  
**Superfícies:** Build toolbar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Remover artefatos de build controlados.

### Metáfora
Vassoura técnica sobre bloco.

### Construção geométrica
Bloco retangular e três cerdas diagonais; evitar lixeira para não sugerir excluir fonte.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Limpar artefatos de build
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 5. `kv.run.start` — Executar

**Prioridade:** P0  
**Superfícies:** Top toolbar; Run panel  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Iniciar configuração de execução.

### Metáfora
Triângulo play.

### Construção geométrica
Triângulo preenchido, centralizado e com área negativa suficiente.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Pequeno target/device quando execução não local.

### Tooltip
```text
Executar
```

### Atalho
```text
Shift+F10
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 6. `kv.run.stop` — Parar

**Prioridade:** P0  
**Superfícies:** Top toolbar; task/process controls  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Encerrar processo ou tarefa.

### Metáfora
Quadrado sólido.

### Construção geométrica
Quadrado de 8–9 px, centralizado; vermelho apenas no estado perigoso/ativo.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Parar
```

### Atalho
```text
Ctrl+F2
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 7. `kv.run.rerun` — Executar novamente

**Prioridade:** P0  
**Superfícies:** Run panel  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Reiniciar a última configuração.

### Metáfora
Play com seta circular.

### Construção geométrica
Triângulo play e arco externo de 180–220°.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Executar novamente
```

### Atalho
```text
Ctrl+F5
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 8. `kv.debug.start` — Iniciar debug

**Prioridade:** P0  
**Superfícies:** Top toolbar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Iniciar configuração sob debugger.

### Metáfora
Inseto técnico com play pequeno.

### Construção geométrica
Base do debug; modifier play de 6–7 px no canto inferior direito.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Depurar
```

### Atalho
```text
Shift+F9
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 9. `kv.debug.attach` — Anexar debugger

**Prioridade:** P0  
**Superfícies:** Debug toolbar; Remote  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Conectar debugger a processo, gdbserver, QEMU ou target.

### Metáfora
Plug conectando-se a inseto/alvo.

### Construção geométrica
Plug simplificado à esquerda e círculo-alvo à direita; cabo curto.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Anexar debugger
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 10. `kv.debug.pause` — Pausar

**Prioridade:** P0  
**Superfícies:** Debug toolbar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Suspender execução do programa.

### Metáfora
Duas barras verticais.

### Construção geométrica
Barras de 2 px com espaço central equivalente; não usar contêiner.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Pausar execução
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 11. `kv.debug.resume` — Continuar

**Prioridade:** P0  
**Superfícies:** Debug toolbar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Continuar programa pausado.

### Metáfora
Play com pequena barra de estado.

### Construção geométrica
Triângulo play; linha vertical curta na origem para diferenciar de Run.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Continuar
```

### Atalho
```text
F9
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 12. `kv.debug.step_over` — Step over

**Prioridade:** P0  
**Superfícies:** Debug toolbar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Executar próxima linha sem entrar na chamada.

### Metáfora
Seta arqueada sobre ponto/linha.

### Construção geométrica
Arco para a direita passando sobre círculo pequeno.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Executar próxima linha
```

### Atalho
```text
F8
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 13. `kv.debug.step_into` — Step into

**Prioridade:** P0  
**Superfícies:** Debug toolbar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Entrar na chamada atual.

### Metáfora
Seta para baixo entrando em bloco.

### Construção geométrica
Bloco inferior aberto e seta vertical apontando para dentro.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Entrar na função
```

### Atalho
```text
F7
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 14. `kv.debug.step_out` — Step out

**Prioridade:** P0  
**Superfícies:** Debug toolbar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Sair da função atual.

### Metáfora
Seta para cima saindo de bloco.

### Construção geométrica
Espelho vertical do step into.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Sair da função
```

### Atalho
```text
Shift+F8
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 15. `kv.debug.breakpoint` — Breakpoint

**Prioridade:** P0  
**Superfícies:** Editor gutter; Breakpoints panel  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Representar breakpoint habilitado.

### Metáfora
Círculo sólido.

### Construção geométrica
Círculo de 8–10 px; forma principal, cor vermelha é secundária.

### Estados
enabled, disabled, pending, verified, invalid e conditional.

### Modifiers e badges
Ponto interno, borda vazada, question mark ou pequena condição.

### Tooltip
```text
Breakpoint
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 16. `kv.debug.memory` — Memória

**Prioridade:** P0  
**Superfícies:** Debug panel  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Abrir visualizador de memória.

### Metáfora
Grade de bytes/chip.

### Construção geométrica
Retângulo de chip com grade 2×2 interna e dois pinos laterais.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Visualizador de memória
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 17. `kv.debug.registers` — Registradores

**Prioridade:** P0  
**Superfícies:** Debug/Embedded panel  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Abrir registradores de CPU/periféricos.

### Metáfora
Três pequenos registradores empilhados.

### Construção geométrica
Três retângulos curtos com bits/pontos à direita.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Registradores
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 18. `kv.test.run_all` — Executar todos os testes

**Prioridade:** P0  
**Superfícies:** Tests toolbar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Executar suite inteira.

### Metáfora
Frasco de testes com play.

### Construção geométrica
Base de tests; modifier play pequeno.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Executar todos os testes
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 19. `kv.test.run_nearest` — Executar teste atual

**Prioridade:** P0  
**Superfícies:** Editor gutter; context menu  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Executar o caso ou suite mais próximo do cursor.

### Metáfora
Pequeno alvo com play.

### Construção geométrica
Círculo alvo de dois anéis; triângulo central.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Executar teste atual
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 20. `kv.test.coverage` — Cobertura

**Prioridade:** P0  
**Superfícies:** Tests toolbar; editor gutter  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Executar/mostrar cobertura de código.

### Metáfora
Gráfico de barras parcialmente preenchidas com check.

### Construção geométrica
Três barras verticais; duas preenchidas, uma vazada; check pequeno.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Executar com cobertura
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 21. `kv.quality.run` — Executar verificações

**Prioridade:** P0  
**Superfícies:** Quality Center  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Rodar profile/gates selecionados.

### Metáfora
Escudo de qualidade com play.

### Construção geométrica
Base de Quality; play como modifier.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Executar verificações de qualidade
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 22. `kv.quality.profile` — Perfil de qualidade

**Prioridade:** P0  
**Superfícies:** Quality Center; top bar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Selecionar strict/balanced/learning/embedded profile.

### Metáfora
Controles deslizantes dentro de escudo.

### Construção geométrica
Escudo simples com duas linhas e pequenos knobs.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Perfil de qualidade ativo
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 23. `kv.quality.static_analysis` — Análise estática

**Prioridade:** P0  
**Superfícies:** Quality Center  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Executar clang-tidy, clippy e ferramentas equivalentes.

### Metáfora
Lupa sobre árvore de código.

### Construção geométrica
Lupa com três ramos internos; não parecer Search Global.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Análise estática
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 24. `kv.quality.sanitizers` — Sanitizers

**Prioridade:** P0  
**Superfícies:** Quality Center; Run Config  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Ativar e representar ASan, UBSan, TSan ou LeakSanitizer.

### Metáfora
Escudo com pequenas ondas/raios de detecção.

### Construção geométrica
Escudo e três marcas radiais internas; sem símbolo médico.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
A/U/T/L como labels textuais adjacentes, não dentro do glyph.

### Tooltip
```text
Sanitizers
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---
