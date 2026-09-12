# Kinein Vectis — Implementation Plan: UI/UX, Arquitetura e Performance

> **Tipo:** plano de implementação sem código.  
> **Escopo:** UI/UX, arquitetura de software, performance, memória, milestones e critérios de aceite.  
> **Restrição:** este documento não propõe escrever código agora. Ele organiza o caminho para implementar com controle, sem inflar escopo e sem comprometer performance.  
> **Objetivo:** manter tudo que foi definido até agora, mas dentro de uma realidade performática e pragmática.

---

## 1. Intenção central

A Kinein Vectis deve ser:

```text
uma IDE visualmente confortável,
determinística,
não invasiva,
com setup visual poderoso,
mas ainda extremamente performática.
```

A UI/UX pode ser rica, mas não pode virar peso morto.

Regra central:

```text
A Kinein pode ter muitos recursos,
mas poucos devem estar vivos ao mesmo tempo.
```

Isso é o que mantém a IDE rápida.

---

## 2. Limite conceitual deste plano

Este plano cobre:

```text
UI/UX
arquitetura de software
performance
memória
processos
jobs
events
estados
módulos
responsabilidades
ordem de implementação
critérios de aceite
```

Este plano não cobre:

```text
código-fonte
implementação de componentes
detalhes de API final
refatoração de repositório
commits
bibliotecas específicas definitivas
```

---

## 3. Realidade sobre consumo de memória

É realista imaginar que, em uma fase intermediária/pós-MVP, a IDE possa chegar perto de:

```text
1.5 GB a 2.0 GB RAM
```

principalmente com:

```text
Qt/QML
editor
Tree-sitter
clangd
rust-analyzer
terminal integrado
logs
Project Health
Configuration Actions
workspace médio/grande
```

Mas isso não deve ser o alvo para todo cenário.

A meta correta é por perfil de uso.

---

## 4. Budget de memória recomendado

### 4.1 IDE aberta sem projeto

Meta:

```text
300 MB a 600 MB
```

Aceitável:

```text
até 700 MB
```

Não aceitável:

```text
> 1 GB sem projeto aberto
```

---

### 4.2 Projeto pequeno CMake ou Cargo

Meta:

```text
600 MB a 1.0 GB
```

Inclui:

```text
UI
Core
file tree
editor
Tree-sitter
um LSP ativo
terminal
Project Health básico
```

---

### 4.3 Projeto médio

Meta:

```text
900 MB a 1.5 GB
```

Inclui:

```text
LSP indexando
diagnósticos
logs
Configuration Actions
CMake/Cargo jobs
```

---

### 4.4 Projeto grande ou Mixed CMake + Cargo

Meta:

```text
1.3 GB a 2.0 GB
```

Aceitável quando:

```text
clangd e rust-analyzer estão ativos;
workspace é grande;
há múltiplos terminais;
há build logs grandes;
há indexação em andamento.
```

---

### 4.5 Acima de 2 GB

Não deve ser normal.

Se passar disso, a IDE deve ter mecanismos:

```text
reduzir indexação;
compactar logs;
descartar caches de UI;
desligar painéis não usados;
avisar de forma discreta;
permitir modo performance.
```

---

## 5. Performance modes

Além dos Experience Modes, a Kinein deve ter **Performance Profiles**.

```text
Balanced Performance
Low Memory
Large Project
Full Intelligence
```

Esses perfis não são para confundir usuário iniciante.  
Eles podem aparecer em Settings avançadas.

---

## 6. Relação entre Experience Mode e Performance

### 6.1 Guided

Pode mostrar mais UI, mas ainda deve ser lazy.

```text
Configuration Actions visíveis;
Setup Assistant ativo;
Project Health expandido;
mas LSP continua sob demanda.
```

### 6.2 Balanced

Equilíbrio.

```text
Configuration Actions quando relevante;
Project Health compacto;
serviços pesados iniciados conforme contexto.
```

### 6.3 Expert

Mais leve por padrão.

```text
sem painéis guiados abertos;
sem Setup Assistant expandido;
Configuration Actions via Command Palette;
menos sugestões visuais;
menor custo de UI.
```

---

## 7. Regra de performance da UI

```text
UI rica não significa UI sempre montada.
```

Painéis devem ser:

```text
lazy loaded;
reutilizáveis;
desmontáveis;
virtualizados;
atualizados por estado mínimo;
sem recomputar listas inteiras;
sem animações pesadas;
sem sombras/glows exagerados;
sem imagens decorativas grandes.
```

---

## 8. Princípios de arquitetura para performance

### 8.1 UI não executa trabalho pesado

A UI deve apenas:

```text
apresentar estado;
receber interação;
enviar intenção;
renderizar resultado.
```

O Core deve:

```text
validar;
criar jobs;
executar services;
emitir events.
```

---

### 8.2 Tudo que demora vira Job

Exemplos:

```text
environment scan
project fingerprint
toolchain health check
cmake configure
cmake build
cargo metadata
cargo check
LSP startup
docs indexing
AI context generation
Configuration Action preview pesado
```

---

### 8.3 Eventos precisam ser agregados

Não emitir evento de UI para cada microalteração.

```text
diagnostics.updated agrupado
fileTree.updated com debounce
build.output streamado em chunks
terminal.output bufferizado
index.status atualizado em intervalos
```

---

### 8.4 Estado centralizado, UI derivada

A UI não deve guardar lógica duplicada.

```text
Core state → UI model → component rendering
```

Evitar:

```text
cada painel recalculando Project Health por conta própria.
```

---

## 9. Estratégia para LSP/clangd/rust-analyzer

### 9.1 clangd

Iniciar quando:

```text
projeto C/C++ ativo;
arquivo C/C++ aberto;
compile_commands.json disponível ou configuração suficiente;
usuário não desativou Language Intelligence.
```

Não iniciar quando:

```text
projeto Cargo-only;
workspace enorme ainda escaneando;
usuário está em Expert + Low Memory;
não há arquivo C/C++ aberto.
```

---

### 9.2 rust-analyzer

Iniciar quando:

```text
Cargo ativo;
arquivo .rs aberto;
Cargo.toml detectado;
cargo metadata disponível ou em andamento.
```

Não iniciar quando:

```text
projeto CMake-only;
Cargo desativado em Project Settings;
usuário não abriu arquivos Rust;
modo Low Memory ativo.
```

---

### 9.3 Mixed project

Em projeto Mixed:

```text
não iniciar os dois imediatamente sem necessidade.
```

Fluxo recomendado:

```text
abrir workspace
  ↓
detectar CMake + Cargo
  ↓
Tree-sitter disponível para ambos
  ↓
LSP primário inicia primeiro
  ↓
LSP secundário inicia sob demanda
```

---

## 10. Tree-sitter como camada leve

Tree-sitter deve ser usado para:

```text
syntax highlighting;
outline local;
folding;
estrutura de arquivo;
seleção por escopo;
fallback quando LSP ainda não carregou.
```

Não usar Tree-sitter para:

```text
semântica profunda;
rename seguro;
inferência de tipos;
referências reais;
diagnósticos complexos.
```

---

## 11. Configuration Actions e performance

Configuration Actions devem ser leves no uso normal.

### 11.1 Registry local

A lista de ações deve vir de um registry local carregado uma vez.

```text
carregar ids, nomes, descrições curtas, escopo e risco;
não calcular preview de todas as ações;
não validar tudo profundamente ao abrir o painel.
```

### 11.2 Validação progressiva

Ao abrir painel:

```text
mostrar ações por escopo ativo;
estado inicial rápido;
validar disponibilidade em background.
```

Ao selecionar ação:

```text
calcular pré-requisitos;
montar formulário;
preparar preview sob demanda.
```

Ao aplicar:

```text
criar job;
aplicar diff;
validar.
```

---

## 12. Densidade visual sem peso visual

A lista de Configuration Actions pode ser densa, mas deve ser leve.

Regras:

```text
itens compactos;
sem cards enormes;
sem thumbnails;
sem ilustrações;
sem banners;
sem gradientes pesados;
sem animação por item;
virtualização de lista;
preview lateral único;
descrição de uma linha;
docs no preview, não na lista principal.
```

---

## 13. File tree e grandes projetos

A árvore de arquivos deve ser lazy.

Não carregar tudo de uma vez.

```text
expandir diretório sob demanda;
cachear filhos expandidos;
respeitar .gitignore;
ignorar build/, target/, .git/, cache/;
permitir include/exclude nas Settings.
```

---

## 14. Logs e terminal

Logs grandes podem destruir performance.

Regras:

```text
streaming em chunks;
limite de linhas em memória;
arquivo de log em disco;
UI mostra janela deslizante;
botão para abrir log completo;
busca em log sob demanda;
não renderizar 100 mil linhas ao mesmo tempo.
```

Terminal:

```text
buffer limitado por sessão;
scrollback configurável;
AI Terminal separado;
terminal inativo pode reduzir atualização visual.
```

---

## 15. Project Health sem pesar

Project Health deve ser incremental.

```text
estado imediato:
- tipo de projeto;
- build system detectado;
- arquivos principais.

estado em background:
- toolchain health;
- LSP status;
- CMake cache;
- diagnostics;
- docs index.
```

Visual:

```text
mostrar "updating..." discreto;
nunca bloquear abertura do projeto.
```

---

## 16. Onboarding e performance

Primeira abertura deve ser rápida.

Não fazer scan pesado antes da UI aparecer.

Fluxo:

```text
abrir UI
mostrar tela inicial
iniciar environment scan em background
mostrar resultados conforme chegam
```

Evitar:

```text
splash screen travada;
scan completo antes da janela;
bloqueio por falta de ferramenta.
```

---

## 17. Settings e performance

Settings pode ser grande, mas não deve carregar tudo.

```text
categorias lazy;
busca indexada leve;
detalhes carregados sob demanda;
validações pesadas em botão Health Check;
não rodar health check ao abrir Settings inteira.
```

---

## 18. Arquitetura de memória

### 18.1 UI

Manter na UI:

```text
estado visual atual;
listas visíveis;
buffers visuais;
seleção;
layout.
```

Não manter na UI:

```text
logs completos;
índices completos;
estado duplicado do Core;
árvore inteira renderizada;
respostas antigas desnecessárias.
```

---

### 18.2 Core

Manter no Core:

```text
workspace state;
job state;
toolchain state;
settings;
project fingerprint;
cache leve;
handles de processos.
```

Não manter indefinidamente:

```text
logs enormes;
contextos temporários antigos;
diagnósticos obsoletos;
previews descartados;
file snapshots desnecessários.
```

---

### 18.3 Disco

Usar disco para:

```text
logs completos;
cache de docs;
context files temporários;
toolchain scan cache;
workspace cache;
support bundles.
```

---

## 19. Performance budget por componente

### 19.1 UI Shell

```text
baixo custo;
sempre carregado;
sem lógica pesada.
```

### 19.2 Editor

```text
custo médio;
carregar arquivos sob demanda;
limite para arquivos gigantes;
avisar em arquivos muito grandes.
```

### 19.3 LSP

```text
custo alto;
sob demanda;
controlado por escopo.
```

### 19.4 Tree-sitter

```text
custo baixo/médio;
pode carregar cedo;
incremental.
```

### 19.5 Configuration Actions

```text
custo baixo na listagem;
custo médio no preview;
custo controlado no apply.
```

### 19.6 Project Health

```text
custo baixo inicial;
custo médio em background.
```

### 19.7 Terminal

```text
custo variável;
scrollback limitado;
logs em disco.
```

---

## 20. UX não invasiva e performance

UX não invasiva também melhora performance.

```text
painéis não abertos não precisam renderizar;
sugestões recolhidas não precisam atualizar constantemente;
Expert Mode reduz componentes vivos;
Command Palette sob demanda reduz UI fixa.
```

A arquitetura visual deve seguir:

```text
mostrar estado;
não abrir tudo;
não atualizar tudo;
não animar tudo.
```

---

## 21. Milestones de implementação sem código

### 21.1 Milestone A — Arquitetura congelada

Entregáveis:

```text
diagrama UI/Core/Services;
contratos principais;
lista de services;
regras de Jobs/Events;
performance budgets;
memory budgets.
```

Critério:

```text
ninguém implementa tela antes de saber onde o estado mora.
```

---

### 21.2 Milestone B — UX skeleton

Entregáveis:

```text
AppShell;
top bar;
activity bar;
project panel;
editor placeholder;
bottom tool window;
status bar;
command palette placeholder.
```

Critério:

```text
UI navega sem services reais.
```

---

### 21.3 Milestone C — Estado e eventos

Entregáveis:

```text
modelo visual de workspace;
modelo visual de jobs;
modelo visual de status;
eventos fake;
painéis reagindo a estado.
```

Critério:

```text
UI prova que não depende de trabalho síncrono.
```

---

### 21.4 Milestone D — Project detection UX

Entregáveis:

```text
Project Health visual;
Build System scope;
CMake-only/Cargo-only/Mixed states;
empty states;
warnings discretos.
```

Critério:

```text
a IDE mostra estado sem invadir.
```

---

### 21.5 Milestone E — Configuration Actions UX

Entregáveis:

```text
painel de actions;
escopo ativo;
busca;
categorias;
lista compacta;
preview lateral;
documentation placeholder;
risk badge.
```

Critério:

```text
lista densa, mas confortável.
```

---

### 21.6 Milestone F — Performance review

Entregáveis:

```text
orçamento de memória por painel;
quais painéis são lazy;
quais listas são virtualizadas;
regras de logs;
regras de LSP;
regras de terminal.
```

Critério:

```text
nenhum recurso pesado fica vivo sem necessidade.
```

---

## 22. Polimento antes de implementar pesado

Antes de entrar em features profundas, polir:

```text
hierarquia visual;
texto dos modos;
texto das actions;
estados vazios;
status bar;
Project Health compacto;
Settings de Experience Mode;
Settings de Performance Profile.
```

---

## 23. Riscos principais

### 23.1 Escopo visual crescer demais

Mitigação:

```text
MVP só com painéis essenciais.
Configuration Actions limitadas.
Embedded avançado fora do MVP.
```

### 23.2 LSP pesar muito

Mitigação:

```text
lazy start;
escopo por Build System ativo;
limites para projeto grande;
status visível;
modo Low Memory.
```

### 23.3 QML ficar pesado

Mitigação:

```text
componentes simples;
listas virtualizadas;
pouca animação;
painéis lazy;
sem efeitos visuais caros.
```

### 23.4 Core virar monólito caótico

Mitigação:

```text
services claros;
protocol crate;
jobs/events;
storage separado;
testabilidade.
```

### 23.5 UX guiada irritar experts

Mitigação:

```text
Expert Mode;
Command Palette;
desabilitar sugestões;
usar existing config;
sem modal automático.
```

---

## 24. Regras de corte de escopo

Se o projeto começar a pesar, cortar primeiro:

```text
QEMU visual;
embedded flash avançado;
docs cacheadas;
plugin system;
actions avançadas;
Project Health expandido;
animações;
painéis secundários.
```

Não cortar:

```text
performance;
editor;
build;
toolchain;
Configuration Actions MVP;
modo Expert;
preview obrigatório;
Core/UI separation.
```

---

## 25. Definição de “performática” para a Kinein

A Kinein será considerada performática se:

```text
abre rápido;
não trava UI;
não inicia serviços sem necessidade;
mostra status claro;
permite cancelar jobs;
não renderiza listas gigantes sem virtualização;
não guarda logs infinitos na memória;
não ativa LSP fora de escopo;
fica útil antes de terminar indexação;
permite modo Expert/Low Memory.
```

---

## 26. Critérios de aceite deste plano

```text
[ ] Performance budget documentado.
[ ] Memory budget documentado.
[ ] LSP sob demanda definido.
[ ] Tree-sitter como camada leve definido.
[ ] Configuration Actions não dependem de LSP.
[ ] UI lazy definida.
[ ] Logs/terminal com limites definidos.
[ ] Project Health incremental definido.
[ ] Experience Mode conectado à performance.
[ ] Performance Profiles definidos.
[ ] Riscos e cortes de escopo definidos.
[ ] Milestones sem código definidos.
```

---

## 27. Resumo executivo

A Kinein pode manter toda a proposta visual e ainda ser performática se seguir esta regra:

```text
não carregar tudo;
não iniciar tudo;
não renderizar tudo;
não indexar tudo;
não sugerir tudo;
não manter tudo na memória.
```

A IDE deve ser rica por disponibilidade, não por presença constante.

Frase final:

```text
Kinein deve oferecer poder sob demanda.
O usuário vê o que precisa,
quando precisa,
sem pagar o custo de tudo ao mesmo tempo.
```
