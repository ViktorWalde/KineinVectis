# Kinein Vectis — Ícones das Abas e Tool Windows Principais

Catálogo detalhado dos ícones que estruturam a navegação primária da IDE.


## 1. `kv.view.project` — Projeto

**Prioridade:** P0  
**Superfícies:** Barra lateral esquerda; Search Everywhere; menu View  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Abrir ou recolher o Project Explorer e representar o workspace atual.

### Metáfora
Duas folhas/pastas sobrepostas, com a folha frontal parcialmente aberta.

### Construção geométrica
Contorno principal em forma de pasta baixa; aba superior curta; segunda camada deslocada 2 px para sugerir workspace sem virar pilha visual.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Badge opcional: ponto de Git, cadeado read-only ou círculo de sincronização.

### Tooltip
```text
Projeto — arquivos, estrutura, targets e módulos
```

### Atalho
```text
Alt+1
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 2. `kv.view.search` — Buscar

**Prioridade:** P0  
**Superfícies:** Barra lateral; top bar; Command Palette  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Abrir a busca global de arquivos, texto, símbolos e ações.

### Metáfora
Lupa convencional.

### Construção geométrica
Círculo aberto com cabo em 45°; área interna ampla para permanecer legível a 16 px.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Buscar em todo o projeto
```

### Atalho
```text
Ctrl+Shift+F
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 3. `kv.view.git` — Git

**Prioridade:** P0  
**Superfícies:** Barra lateral; painel inferior; status  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Abrir Source Control, mudanças, branches, histórico e conflitos.

### Metáfora
Grafo de três nós conectados.

### Construção geométrica
Três círculos de 2–3 px; haste vertical e ramo diagonal; evitar copiar o logotipo oficial do Git.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Badge numérico para quantidade de alterações; ponto vermelho apenas como apoio à forma.

### Tooltip
```text
Controle de versão
```

### Atalho
```text
Alt+9
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 4. `kv.view.build` — Build

**Prioridade:** P0  
**Superfícies:** Barra lateral; painel inferior; toolbar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Abrir configurações, targets, fases e logs de build.

### Metáfora
Blocos sendo montados por uma pequena ferramenta.

### Construção geométrica
Dois blocos quadrados alinhados e um terceiro incompleto; pequena chave/engrenagem simplificada no canto.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Play pequeno para build ativo; check para sucesso; x para falha.

### Tooltip
```text
Build — configurar, compilar, vincular e empacotar
```

### Atalho
```text
Alt+4
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 5. `kv.view.run` — Executar

**Prioridade:** P0  
**Superfícies:** Barra lateral; toolbar; Run Configurations  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Abrir execuções, processos e configurações de run.

### Metáfora
Triângulo de reprodução dentro de uma moldura de processo.

### Construção geométrica
Triângulo apontando à direita, centralizado dentro de retângulo aberto; não usar apenas um play isolado para diferenciar da ação Run.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Ponto verde para processo ativo; contador quando várias execuções existirem.

### Tooltip
```text
Executar e gerenciar processos
```

### Atalho
```text
Shift+F10
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 6. `kv.view.debug` — Depurar

**Prioridade:** P0  
**Superfícies:** Barra lateral; toolbar; painel inferior  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Abrir sessões, breakpoints, pilhas, variáveis e memória.

### Metáfora
Inseto técnico estilizado com corpo central e quatro pernas.

### Construção geométrica
Corpo vertical simples; cabeça circular; duas pernas por lado; sem aparência orgânica ou infantil.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Play para iniciar; plug para attach; ponto para sessão ativa.

### Tooltip
```text
Depuração
```

### Atalho
```text
Alt+5
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 7. `kv.view.tests` — Testes

**Prioridade:** P0  
**Superfícies:** Barra lateral; painel inferior  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Abrir Test Explorer, suites, casos, cobertura e resultados.

### Metáfora
Frasco de laboratório com marca de verificação.

### Construção geométrica
Gargalo curto e corpo trapezoidal; check interno pequeno; não confundir com Quality Center.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Check, x, pause ou círculo em execução.

### Tooltip
```text
Testes
```

### Atalho
```text
Ctrl+Shift+T
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 8. `kv.view.quality` — Qualidade

**Prioridade:** P0  
**Superfícies:** Barra lateral; painel inferior; toolbar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Abrir Quality Center, perfis rígidos, linters, formatadores e gates.

### Metáfora
Escudo hexagonal com check técnico.

### Construção geométrica
Escudo de lados retos e topo aberto; check central; peso visual equivalente ao debug.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Estrela para perfil recomendado; cadeado para policy obrigatória; x para gate falho.

### Tooltip
```text
Quality Center — perfis, regras e verificações
```

### Atalho
```text
Ctrl+Alt+Q
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 9. `kv.view.embedded` — Embedded & Remote

**Prioridade:** P0  
**Superfícies:** Barra lateral; painel inferior  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Abrir targets, toolchains, deploy, SSH, QEMU, serial e probes.

### Metáfora
Placa eletrônica ligada a um terminal remoto.

### Construção geométrica
Retângulo de PCB com dois pinos laterais e pequeno nó remoto no canto superior; desenho único, não colagem de três ícones.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Wi-Fi/SSH, play, warning, USB ou alvo ativo.

### Tooltip
```text
Embedded & Remote
```

### Atalho
```text
Alt+E
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 10. `kv.view.terminal` — Terminal

**Prioridade:** P0  
**Superfícies:** Painel inferior; top bar; menu View  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Abrir terminais locais, remotos e de SDK.

### Metáfora
Janela de terminal com prompt.

### Construção geométrica
Retângulo com cantos discretos; símbolo >_ simplificado e centralizado.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
SSH para sessão remota; container para Dev Container; SDK para ambiente carregado.

### Tooltip
```text
Terminal integrado
```

### Atalho
```text
Alt+F12
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 11. `kv.view.problems` — Problemas

**Prioridade:** P0  
**Superfícies:** Painel inferior; status bar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Listar erros, warnings, hints e problemas de configuração.

### Metáfora
Triângulo de alerta sobre lista.

### Construção geométrica
Três linhas curtas e um triângulo pequeno à esquerda; evitar triângulo isolado, que representa apenas warning.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Badge numérico; filtro ativo.

### Tooltip
```text
Problemas e diagnósticos
```

### Atalho
```text
Ctrl+Shift+M
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 12. `kv.view.tasks` — Tarefas

**Prioridade:** P0  
**Superfícies:** Painel inferior; status; Command Palette  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Abrir Task Manager, histórico, filas e tarefas ativas.

### Metáfora
Lista de etapas com indicador de execução.

### Construção geométrica
Três linhas horizontais; primeira com play pequeno; segunda com check; terceira neutra.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Spinner, pause, x ou contador.

### Tooltip
```text
Tarefas em background
```

### Atalho
```text
Ctrl+Shift+J
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 13. `kv.view.services` — Serviços

**Prioridade:** P1  
**Superfícies:** Barra lateral; painel inferior  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Representar processos persistentes, containers, servidores e agentes.

### Metáfora
Três módulos conectados a um hub.

### Construção geométrica
Nó central quadrado e três nós pequenos em arco; geometria técnica e estável.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Play, stop, health check ou remote.

### Tooltip
```text
Serviços e processos persistentes
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 14. `kv.view.settings` — Configurações

**Prioridade:** P0  
**Superfícies:** Base da barra lateral; menus  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Abrir configurações globais, do workspace e perfis.

### Metáfora
Engrenagem convencional.

### Construção geométrica
Seis dentes, furo central grande; manter baixo nível de detalhe.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Configurações
```

### Atalho
```text
Ctrl+Alt+S
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---
