# Kinein Vectis — Icon System Master


---

# Kinein Vectis — Sistema Profissional de Iconografia

> Especificação visual e técnica para todos os ícones funcionais, abas e estados da primeira arquitetura completa do Kinein Vectis.

## 1. Objetivo

Este pacote impede que a implementação produza uma mistura de ícones retirados de bibliotecas diferentes, com pesos, tamanhos e metáforas incompatíveis.

A regra principal é:

```text
O ID semântico é permanente.
O desenho visual pode evoluir ou ser substituído por tema.
A ação nunca deve depender do caminho do SVG.
```

O sistema foi modelado com base em práticas públicas de IDEs e toolkits profissionais:

- IDs semânticos e temas de product icons;
- versões específicas para 20×20 e 16×16;
- ícones simbólicos, monocromáticos e legíveis em tamanhos pequenos;
- modifiers/badges posicionados consistentemente;
- suporte a temas, fallback e High DPI no Qt;
- não depender apenas de cor para transmitir estado.

## 2. Decisão visual

```text
Tipo: SVG simbólico próprio.
Estilo: outline técnico, compacto e confortável.
Base de Tool Window: 20×20.
Modo compacto e ações: 16×16.
Toolbar confortável: 20×20.
Ações de destaque: até 24×24.
App icon: projeto separado, full color.
```

Não usar um icon font como fonte canônica. SVGs individuais são mais fáceis de auditar, versionar, revisar e substituir por tema.

## 3. Grid e construção

### 3.1 Grid 20×20

```text
Canvas: 20×20 unidades.
Safe area: x/y entre 2 e 18.
Stroke principal: 1.5.
Stroke pesado excepcional: 2.
Cap: square ou round conforme família, nunca misturados no mesmo ícone.
Join: round para formas orgânicas controladas; miter/round para formas técnicas.
```

### 3.2 Grid 16×16

```text
Canvas: 16×16.
Safe area: x/y entre 1.5 e 14.5.
Stroke principal: 1.25–1.5.
Detalhe mínimo: 1.5 px visual.
Espaço negativo mínimo: 1 px.
```

Os masters de 20 e 16 devem ser revisados separadamente. Não confiar apenas em reduzir automaticamente o SVG de 20 para 16.

## 4. Cor

Paleta funcional do Kinein Vectis:

```text
icon.foreground.default   #B9B3A5
icon.foreground.strong    #EAE6E1
icon.foreground.muted     #8F8A7C
icon.accent               #FFBB00
icon.accent.dim           #6E5C01
icon.success              token semântico do tema
icon.warning              token semântico do tema
icon.error                token semântico do tema
icon.info                 token semântico do tema
```

Regras:

```text
1. Ícone neutro por padrão.
2. Accent apenas para selected/active ou ação principal.
3. Error/warning/success precisam também de forma, badge ou texto.
4. Sem gradientes.
5. Sem sombras.
6. Sem glow.
7. Sem preenchimentos multicoloridos em barras densas.
```

## 5. Estados

Cada ícone acionável deve suportar:

```text
default
hover
pressed
selected
focused
disabled
busy
success
warning
error
```

O SVG normalmente permanece o mesmo; a camada Qt/QML altera tokens de cor, fundo, opacidade, badge e animação.

## 6. Modifiers e badges

```text
Tamanho: 6–9 px no master 20×20.
Posição padrão: canto inferior direito.
Distância da forma base: 1–2 px.
Usar canto alternativo quando encobrir a metáfora.
No máximo dois modifiers simultâneos.
```

Modifiers oficiais:

```text
add
remove
check
error
warning
lock
remote
running
paused
sync
dirty
count
```

## 7. Acessibilidade

```text
- Todo botão somente com ícone precisa de tooltip.
- Ícones ambíguos em sidebars e view switchers podem ter label.
- Focus ring pertence ao componente, não ao SVG.
- Estado não pode ser comunicado somente por cor.
- Hit target mínimo recomendado: 28×28 no modo compacto e 32×32 no confortável.
- Tooltip deve descrever a ação, não a aparência.
```

## 8. Naming

IDs:

```text
kv.view.*
kv.action.*
kv.editor.*
kv.project.*
kv.search.*
kv.git.*
kv.build.*
kv.run.*
kv.debug.*
kv.test.*
kv.quality.*
kv.embedded.*
kv.status.*
kv.file.*
kv.modifier.*
```

Arquivos:

```text
resources/icons/20/view/project.svg
resources/icons/16/view/project.svg
resources/icons/20-dark/view/project.svg   # somente se realmente necessário
```

A preferência é colorização por token, evitando duplicar dark/light.

## 9. Contrato com Qt/QML

Cada ícone deve ser resolvido pelo `IconRegistry`, nunca por URL hardcoded.

```qml
KIconButton {
    actionId: "workspace.project.toggle"
    iconId: "kv.view.project"
    tooltip: qsTr("Projeto — arquivos, estrutura, targets e módulos")
}
```

O registry resolve:

```text
ID semântico
→ tema de produto ativo
→ variante de tamanho
→ SVG padrão
→ fallback de theme icon do sistema quando permitido
```

## 10. Critérios de aprovação de um SVG

```text
[ ] reconhecível sem tooltip em contexto comum;
[ ] distinguível dos vizinhos;
[ ] legível em 16 e 20;
[ ] não depende apenas de cor;
[ ] sem detalhes menores que o limite;
[ ] alinhado opticamente;
[ ] strokes consistentes;
[ ] área de clique separada do glyph;
[ ] dark/light testados;
[ ] 100%, 125%, 150%, 200% testados;
[ ] nome e ID estáveis;
[ ] licença/proveniência registradas;
```


---

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


---

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


---

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


---

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


---

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


---

# Kinein Vectis — Embedded, Remote, SSH e QEMU

Ícones especializados para Linux embarcado, bare metal, deploy, serial, QEMU e debug remoto.


## 1. `kv.embedded.target` — Target

**Prioridade:** P0  
**Superfícies:** Embedded panel; top bar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Representar destino de execução selecionado.

### Metáfora
Mira técnica.

### Construção geométrica
Dois círculos concêntricos e ponto central quadrado; quatro marcas cardeais curtas.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Local, SSH, QEMU, MCU ou container como modifier.

### Tooltip
```text
Target ativo
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 2. `kv.embedded.local_linux` — Linux local

**Prioridade:** P0  
**Superfícies:** Target selector  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Representar host Linux local.

### Metáfora
Monitor/terminal com pequeno kernel/pinguim abstrato não marcário.

### Construção geométrica
Monitor simples com prompt; pequeno ponto local no canto inferior.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Linux local
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 3. `kv.embedded.remote_linux` — Linux remoto

**Prioridade:** P0  
**Superfícies:** Target selector  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Representar target Linux acessado por rede.

### Metáfora
Monitor remoto ligado por dois nós.

### Construção geométrica
Monitor à direita e nó/host pequeno à esquerda conectados por linha.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Lock para host verificado; warning para host key.

### Tooltip
```text
Linux remoto
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 4. `kv.embedded.ssh` — SSH

**Prioridade:** P0  
**Superfícies:** Target; terminal; remote toolbar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Abrir ou indicar conexão SSH.

### Metáfora
Terminal com chave.

### Construção geométrica
Janela de terminal; pequena chave de 6–7 px no canto inferior direito.

### Estados
disconnected, connecting, connected, reconnecting e failed.

### Modifiers e badges
Lock/open lock ou warning.

### Tooltip
```text
Conexão SSH
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 5. `kv.embedded.deploy` — Deploy

**Prioridade:** P0  
**Superfícies:** Remote toolbar; Run Config  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Enviar artefatos ao target.

### Metáfora
Pacote/binário com seta para host remoto.

### Construção geométrica
Cubo/arquivo à esquerda, seta horizontal e pequeno target à direita.

### Estados
idle, transferring, succeeded, failed e cancelled.

### Modifiers e badges
Progress ring ou check/x.

### Tooltip
```text
Enviar para o target
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 6. `kv.embedded.remote_run` — Executar remotamente

**Prioridade:** P0  
**Superfícies:** Remote toolbar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Executar comando/binário no target remoto.

### Metáfora
Play dentro de host remoto.

### Construção geométrica
Retângulo de monitor com triângulo central e pequeno nó de rede.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Executar no target remoto
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 7. `kv.embedded.gdbserver` — gdbserver

**Prioridade:** P0  
**Superfícies:** Debug Profile; Remote toolbar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Iniciar ou indicar servidor de debug remoto.

### Metáfora
Bug/debug ligado a porta de rede.

### Construção geométrica
Ícone debug à esquerda; tomada/porta circular à direita com linha curta.

### Estados
stopped, starting, listening, attached e failed.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
gdbserver remoto
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 8. `kv.embedded.serial` — Serial Monitor

**Prioridade:** P0  
**Superfícies:** Embedded panel; bottom panel  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Abrir monitor serial.

### Metáfora
Conector serial/USB com linhas de texto.

### Construção geométrica
Plug de 4 pinos simplificado à esquerda; três linhas de log à direita.

### Estados
closed, opening, connected, paused e error.

### Modifiers e badges
Número da porta apenas como texto próximo.

### Tooltip
```text
Monitor serial
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 9. `kv.embedded.qemu` — QEMU

**Prioridade:** P0  
**Superfícies:** Target selector; Embedded panel  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Representar máquina emulada pelo QEMU sem usar marca/logotipo.

### Metáfora
Máquina virtual dentro de uma moldura de chip.

### Construção geométrica
Chip retangular com pequeno monitor/CPU interno; bordas técnicas.

### Estados
stopped, booting, running, paused e failed.

### Modifiers e badges
GDB, serial ou display.

### Tooltip
```text
Target emulado
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 10. `kv.embedded.qemu_display` — Display QEMU

**Prioridade:** P0  
**Superfícies:** QEMU controls  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Abrir a janela/display da máquina emulada.

### Metáfora
Monitor com pequeno chip no canto.

### Construção geométrica
Monitor convencional; chip quadrado de 5–6 px como modifier.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Abrir display da máquina emulada
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 11. `kv.embedded.cross_compile` — Cross-compilation

**Prioridade:** P0  
**Superfícies:** Toolchain selector; Build  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Indicar compilação host → arquitetura alvo.

### Metáfora
Dois chips diferentes ligados por seta.

### Construção geométrica
Chip x86/genérico à esquerda, seta curta, chip alvo à direita; formas levemente diferentes.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Arquitetura deve aparecer em texto: ARM64, ARM, RISC-V.

### Tooltip
```text
Toolchain de cross-compilation
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 12. `kv.embedded.sysroot` — Sysroot

**Prioridade:** P0  
**Superfícies:** SDK/Toolchain settings  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Representar raiz de headers/libs do target.

### Metáfora
Árvore de diretórios dentro de target.

### Construção geométrica
Pasta raiz com dois ramos internos; pequeno alvo no canto.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Sysroot do target
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 13. `kv.embedded.sdk` — SDK

**Prioridade:** P0  
**Superfícies:** SDK profiles; First Run  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Representar SDK Yocto/Buildroot/vendor carregado.

### Metáfora
Caixa de ferramentas com chip.

### Construção geométrica
Maleta baixa; chip quadrado central; alça curta.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Check para ambiente carregado; warning para script inválido.

### Tooltip
```text
SDK
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 14. `kv.embedded.board` — Placa

**Prioridade:** P0  
**Superfícies:** Targets; Project templates  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Representar placa de desenvolvimento.

### Metáfora
PCB retangular com chip central e headers laterais.

### Construção geométrica
Retângulo 16×12; chip 5×5; três pinos em cada lado.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
USB, Wi-Fi, lock ou warning.

### Tooltip
```text
Placa de desenvolvimento
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 15. `kv.embedded.mcu` — Microcontrolador

**Prioridade:** P0  
**Superfícies:** Targets; Memory map  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Representar MCU/bare metal.

### Metáfora
Chip quadrado com núcleo central.

### Construção geométrica
Quadrado de 10–12 px, pinos nos quatro lados e pequeno quadrado interno.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Microcontrolador
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 16. `kv.embedded.probe` — Debug probe

**Prioridade:** P0  
**Superfícies:** Debug Profile; Devices  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Representar ST-Link, J-Link, CMSIS-DAP ou probe-rs sem usar marcas.

### Metáfora
Dongle USB ligado a ponta de probe.

### Construção geométrica
Corpo retangular curto, conector à esquerda e dois fios/pinos à direita.

### Estados
missing, detected, ready, busy e failed.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Probe de debug
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 17. `kv.embedded.flash` — Gravar firmware

**Prioridade:** P0  
**Superfícies:** Bare-metal toolbar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Programar firmware na memória do target.

### Metáfora
Raio/seta entrando em chip.

### Construção geométrica
Chip MCU; seta diagonal preenchida entrando no centro.

### Estados
idle, erasing, writing, verifying, succeeded e failed.

### Modifiers e badges
Progress e check/x.

### Tooltip
```text
Gravar firmware
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 18. `kv.embedded.reset` — Resetar target

**Prioridade:** P0  
**Superfícies:** Bare-metal/QEMU controls  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Resetar dispositivo ou máquina virtual.

### Metáfora
Seta circular curta sobre chip.

### Construção geométrica
Chip pequeno com arco de 180–220° no topo; não usar refresh genérico isolado.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Resetar target
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 19. `kv.embedded.power` — Ligar/desligar target

**Prioridade:** P0  
**Superfícies:** Devices/QEMU controls  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Controlar energia virtual ou estado de sessão, não alimentação física arbitrária.

### Metáfora
Símbolo universal de power.

### Construção geométrica
Arco quase fechado e haste vertical central.

### Estados
off, on, transitioning e unavailable.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Alternar estado do target
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 20. `kv.embedded.port_forward` — Port forwarding

**Prioridade:** P0  
**Superfícies:** SSH advanced  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Criar túnel/encaminhamento de porta.

### Metáfora
Dois portais/portas ligados por seta.

### Construção geométrica
Dois colchetes retangulares; seta horizontal passando entre eles.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
L/R/D em texto adjacente para local, remote ou dynamic.

### Tooltip
```text
Encaminhamento de porta
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---


---

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


---

# Kinein Vectis — Contrato de Iconografia para o Frontend Qt/QML

## 1. Regra

O frontend não pode referenciar diretamente:

```text
qrc:/icons/project.svg
../../../assets/debug.png
:/dark/search.svg
```

Deve referenciar:

```text
kv.view.project
kv.view.debug
kv.action.search_everywhere
```

## 2. Estruturas previstas

```text
IconRegistry
IconTheme
IconDescriptor
IconState
IconBadge
KIcon
KIconButton
KToolWindowButton
KStatusIcon
KFileIcon
```

## 3. Descriptor

```json
{
  "id": "kv.view.project",
  "variants": {
    "16": "qrc:/icons/16/view/project.svg",
    "20": "qrc:/icons/20/view/project.svg"
  },
  "symbolic": true,
  "recolorable": true,
  "defaultTooltip": "Projeto — arquivos, estrutura, targets e módulos"
}
```

## 4. Exemplo QML futuro

```qml
KToolWindowButton {
    id: projectButton

    actionId: "workspace.project.toggle"
    iconId: "kv.view.project"
    text: qsTr("Projeto")
    shortcutText: "Alt+1"
    checked: WorkspaceUi.projectVisible
    badgeCount: ProjectModel.pendingChanges
}
```

## 5. Separação de responsabilidades

```text
SVG:
geometria.

IconRegistry:
resolução do ID.

Theme:
cor e variantes.

Action Registry:
função, enabled, checked, shortcut e tooltip.

Componente QML:
layout, hover, focus, badge, animação e hit target.
```

## 6. Theme switching

Trocar o tema não deve alterar IDs de ações.

```text
kv.view.project
→ Kinein Default
→ High Contrast
→ System/Breeze fallback opcional
→ tema de produto futuro
```

## 7. Recursos obrigatórios antes do frontend

```text
1. Gerar SVGs 16/20.
2. Validar viewBox.
3. Gerar qrc.
4. Criar IconRegistry.
5. Criar ActionRegistry.
6. Criar tokens do tema.
7. Criar KIcon e KIconButton.
8. Criar storybook/gallery QML interno.
9. Testar escala 100–200%.
10. Testar light, dark e high contrast.
```

## 8. Galeria interna

O frontend deve conter uma tela de desenvolvimento:

```text
Settings > Developer > Icon Gallery
```

Ela deve mostrar:

```text
ID
glyph em 16/20/24
default/hover/selected/disabled
dark/light/high contrast
tooltip
badge
ação vinculada
arquivo de origem
```

## 9. Próxima etapa

O protótipo visual do frontend deve consumir o catálogo JSON e implementar primeiro:

```text
top bar;
activity bar;
Project Explorer;
editor tabs;
painel direito;
painel inferior;
status bar;
Embedded & Remote;
Quality Center.
```
