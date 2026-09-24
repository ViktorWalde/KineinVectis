# Kinein Vectis — Parte 10: Resource-on-Demand Performance Strategy

> **Revisão vinculante de 2026-09-22:** métricas de desempenho são locais e
> explícitas; não há telemetria de produto/usuário nem envio à Kinein. Não há
> Assistente/Chat de IA embutido. Ver
> `arquitetura-de-frontend-0.3-em-diante.md`.

> **Tipo:** versão corrigida e definitiva da Parte 10 de performance.  
> **Escopo:** arquitetura orientada à demanda, performance, memória, UI responsiva, LSP sob demanda, serviços ociosos, logs, painéis, Configuration Actions, Project Health e modos de performance.  
> **Correção importante:** não existe fronteira rígida de RAM como “2 GB”. Memória é observada, explicada e otimizada, mas não usada como bloqueio artificial.

---

## 1. Decisão central

A Kinein Vectis deve ser performática por arquitetura, não por limitar recursos.

```text
Kinein não limita poder.
Kinein evita desperdício.
```

A IDE pode usar mais CPU/RAM quando houver valor real:

```text
projeto grande;
LSP indexando;
refatoração profunda;
debug ativo;
múltiplos terminais;
Mixed CMake + Cargo;
Full Intelligence Profile;
logs de build em execução.
```

Mas ela não deve manter recursos caros vivos sem necessidade.

---

## 2. Sem hard cap de memória

A Kinein não deve ter regra do tipo:

```text
passou de 2 GB → cortar recurso
```

Isso não deve existir.

O que deve existir:

```text
métricas locais de desempenho;
observabilidade;
background services visíveis;
estratégias de suspensão;
cache em disco;
limpeza de recursos antigos;
modo responsivo para hardware mais fraco;
controle explícito do usuário.
```

2 GB, 3 GB ou mais podem ser aceitáveis dependendo do projeto e do uso.  
O problema não é usar memória. O problema é usar memória sem motivo claro.

---

## 3. Princípio definitivo

```text
A Kinein deve ser rica por disponibilidade,
não por presença constante.
```

Ou seja:

```text
os recursos existem;
mas não precisam estar todos ativos;
não precisam estar todos renderizados;
não precisam estar todos indexando;
não precisam estar todos na memória.
```

---

## 4. Resource-on-Demand como padrão

Resource-on-Demand não é um modo opcional.  
É o comportamento natural da IDE.

```text
abrir rápido;
detectar o essencial;
ativar profundidade quando o contexto pede;
reduzir atividade quando o recurso fica ocioso;
liberar memória de recursos antigos;
manter UI responsiva.
```

---

## 5. Ciclo de vida de recursos

Todo recurso pesado deve ter ciclo de vida explícito.

```text
Not Loaded
Warm
Active
Background
Sleeping
Evicted
Failed
```

### 5.1 Not Loaded

Recurso ainda não foi carregado.

Exemplo:

```text
rust-analyzer em projeto CMake-only.
```

---

### 5.2 Warm

Metadados leves foram carregados, mas o recurso pesado ainda não iniciou.

Exemplo:

```text
Configuration Actions registry carregado,
mas previews e diffs não calculados.
```

---

### 5.3 Active

Recurso está em uso direto.

Exemplo:

```text
clangd analisando arquivo C++ aberto.
```

---

### 5.4 Background

Recurso trabalha sem bloquear UI.

Exemplo:

```text
Project Health atualizando;
cargo metadata rodando;
CMake configure em job.
```

---

### 5.5 Sleeping

Recurso fica pausado/reduzido.

Exemplo:

```text
painel fechado;
terminal inativo;
LSP secundário em Mixed project sem arquivo aberto.
```

---

### 5.6 Evicted

Recurso foi liberado da memória e pode ser recriado depois.

Exemplo:

```text
preview antigo de Configuration Action;
contexto temporário de AI Terminal;
cache visual de painel fechado.
```

---

### 5.7 Failed

Recurso falhou e deve mostrar estado claro.

Exemplo:

```text
clangd crashou;
cargo metadata falhou;
CMake configure falhou.
```

---

## 6. Resource Manager, não Resource Limiter

A Kinein deve ter um conceito interno:

```text
Resource Manager
```

Ele não é um limitador agressivo.  
Ele é um coordenador.

Responsabilidades:

```text
saber quais serviços estão ativos;
saber por que estão ativos;
saber quando podem dormir;
saber o que pode ser descartado;
emitir status para UI;
ajudar perfis de performance;
evitar desperdício.
```

Não deve:

```text
matar LSP necessário;
bloquear projeto grande;
impedir uso avançado;
cortar recurso sem explicar;
tomar decisão destrutiva sem controle.
```

---

## 7. Background Services visíveis

A Kinein deve ter uma visão discreta de serviços em segundo plano.

Exemplo:

```text
Background Services
├── Tree-sitter: active
├── clangd: running
├── rust-analyzer: sleeping
├── Project Health: updated
├── CMake cache: stale
├── Configuration Actions: warm
└── Jobs: 1 running
```

Isso dá confiança ao usuário.

---

## 8. Performance Profiles revisados

Os perfis não servem para “capar” a IDE.  
Eles definem quando ativar profundidade.

```text
Responsive
Balanced
Full Intelligence
Large Workspace
```

---

### 8.1 Responsive

Para hardware mais fraco ou usuário que quer máxima fluidez.

```text
LSP mais sob demanda;
menos indexação inicial;
Project Health compacto;
painéis guiados mais recolhidos;
logs com scrollback menor;
menos serviços em background.
```

Não significa:

```text
sem LSP;
sem diagnóstico;
sem recurso avançado.
```

Significa:

```text
ativar quando solicitado.
```

---

### 8.2 Balanced

Perfil padrão recomendado.

```text
bom equilíbrio;
LSP inicia quando contexto aparece;
Project Health incremental;
Configuration Actions sob demanda;
logs controlados;
UI completa, mas lazy.
```

---

### 8.3 Full Intelligence

Para máquinas mais fortes ou projetos onde o usuário quer análise profunda.

```text
LSP inicia mais cedo;
indexação semântica mais ativa;
diagnósticos mais abrangentes;
Project Context mais profundo;
melhor para refatoração e navegação intensiva.
```

Esse modo pode consumir mais CPU/RAM e isso é aceitável.

---

### 8.4 Large Workspace

Para monorepos e projetos grandes.

```text
file tree muito lazy;
watchers reduzidos;
indexação controlada;
diagnósticos agregados;
exclusões por padrão;
Project Health por módulo/target;
evitar abrir tudo de uma vez.
```

---

## 9. Relação com Experience Modes

Experience Modes e Performance Profiles são coisas diferentes.

```text
Experience Mode = quanto auxílio visual aparece.
Performance Profile = quando serviços pesados são ativados.
```

Exemplos:

```text
Guided + Responsive
Balanced + Balanced
Expert + Full Intelligence
Expert + Large Workspace
```

Um usuário avançado pode querer Full Intelligence.  
Um iniciante em hardware fraco pode querer Guided + Responsive.

---

## 10. UI/UX performática

A UI deve ser rica, mas barata.

Regras:

```text
painéis lazy;
listas virtualizadas;
componentes desmontáveis;
sem animação por item;
sem banners pesados;
sem thumbnails;
sem gradiente desnecessário;
sem shadow/glow exagerado;
preview lateral único;
estado mínimo na UI;
dados grandes no Core/disco.
```

---

## 11. Painéis

Painel fechado não deve continuar caro.

```text
Project panel:
árvore lazy.

Configuration Actions:
registry leve, preview sob demanda.

Problems:
renderiza itens visíveis, agrupa diagnostics.

Project Health:
compacto por padrão, expandido sob demanda.

Settings:
categorias lazy.

AI Terminal:
contextos temporários limpos quando não usados.
```

---

## 12. File Tree

A árvore de arquivos deve seguir:

```text
expandir sob demanda;
respeitar .gitignore;
ignorar build/;
ignorar target/;
ignorar .git/;
ignorar cache/;
não renderizar árvore inteira;
não observar diretórios desnecessários.
```

---

## 13. Logs e terminal

Logs grandes devem ir para disco.

Na memória:

```text
janela recente;
scrollback limitado;
índice leve;
estado da sessão.
```

No disco:

```text
log completo;
build output completo;
job logs;
support bundle futuro.
```

A UI nunca deve tentar renderizar 100 mil linhas de uma vez.

---

## 14. LSP sob demanda, mas sem medo de profundidade

A Kinein deve usar `clangd` e `rust-analyzer` de forma inteligente.

```text
não iniciar fora de contexto;
não iniciar linguagem não usada;
não duplicar servidores sem necessidade;
não reiniciar em loop;
não bloquear UI esperando LSP;
mostrar status.
```

Mas quando o usuário precisar:

```text
usar navegação profunda;
find references;
rename;
hover;
diagnósticos;
refatoração;
```

a IDE deve permitir o LSP usar poder total.

---

## 15. Tree-sitter como resposta imediata

Tree-sitter deve dar sensação de velocidade.

```text
syntax highlighting imediato;
outline local;
folding;
seleção estrutural;
breadcrumbs básicos;
fallback enquanto LSP carrega.
```

Isso evita a IDE parecer “morta” antes do LSP terminar.

---

## 16. Configuration Actions e performance

Configuration Actions devem ser orientadas por escopo e sob demanda.

```text
CMake ativo → ações CMake.
Cargo ativo → ações Cargo.
Mixed → ambas, separadas.
```

O painel deve carregar:

```text
id;
nome;
descrição curta;
escopo;
categoria;
risco;
docs metadata.
```

Não carregar inicialmente:

```text
preview;
diff;
validação pesada;
análise profunda do CMake inteiro;
docs completas.
```

Preview só quando o usuário seleciona a ação.

---

## 17. Project Health incremental

Project Health deve ter camadas.

### 17.1 Imediato

```text
tipo de projeto;
build systems detectados;
arquivos principais;
workspace aberto;
```

### 17.2 Background rápido

```text
toolchain scan;
CMake/Cargo status;
compile_commands;
presets;
```

### 17.3 Background profundo

```text
diagnósticos agregados;
LSP status;
CMake cache;
target graph;
Project Context Model.
```

A UI deve mostrar progresso discreto.

---

## 18. Inspections e tempo real

A Kinein deve avisar em tempo real, mas sem invadir.

Fontes:

```text
clangd;
rust-analyzer;
compiler;
linker;
cmake;
cargo;
project-health;
setup-intelligence.
```

Visual:

```text
gutter;
underline;
Problems;
status bar;
Project Health;
quick action sob demanda.
```

Evitar:

```text
modal automático;
toast repetitivo;
painel abrindo sozinho;
animações agressivas;
IA se oferecendo sozinha.
```

---

## 19. Unified Diagnostics Model

Diagnósticos devem ser unificados.

```text
fonte;
severidade;
arquivo;
range;
target;
build profile;
ação sugerida;
docs;
related diagnostics;
```

Isso permite:

```text
Problems panel forte;
Project Health coerente;
Configuration Actions como correção;
refatoração conectada a erro real.
```

---

## 20. Project Context Model

A Kinein deve manter um contexto técnico do projeto.

```text
workspace;
build systems;
targets;
toolchains;
presets;
run configs;
debug configs;
source files;
include paths;
dependencies;
diagnostics;
symbols;
build history;
test history;
Configuration Actions disponíveis.
```

Esse contexto é determinístico.

```text
não é IA;
não é chat;
não é agente;
não é adivinhação.
```

---

## 21. Refatoração futura e performance

Refatorações profundas devem ser chamadas sob demanda.

Níveis:

```text
1. LSP-native
2. IDE-assisted
3. Build-aware
4. Workspace-aware
```

Nada disso deve rodar constantemente em background.

A análise profunda acontece quando o usuário chama a refatoração.

---

## 22. Resource cleanup

A Kinein deve limpar:

```text
previews antigos;
diffs descartados;
contextos temporários;
logs antigos em memória;
painéis fechados;
diagnósticos obsoletos;
jobs finalizados antigos;
watchers não necessários.
```

Mas deve preservar:

```text
settings;
cache útil;
logs em disco;
recent projects;
estado necessário para reabrir workspace.
```

---

## 23. Observabilidade local

A IDE deve conseguir explicar:

```text
o que está rodando;
por que está rodando;
quanto tempo está rodando;
qual job iniciou;
qual serviço consome mais;
qual LSP está ativo;
qual projeto/target disparou a ação.
```

Sem expor isso o tempo todo.  
Mas deve estar acessível.

---

## 24. Quando usar mais recursos é correto

Uso alto é correto quando:

```text
o usuário pediu refatoração;
o projeto é grande;
Full Intelligence está ativo;
LSP está indexando;
debug está rodando;
build está emitindo logs;
Mixed project está ativo;
múltiplos terminais estão abertos.
```

Uso alto é ruim quando:

```text
nada está visível;
nada está sendo usado;
serviço ficou preso;
logs antigos acumulam;
painel fechado continua renderizando;
LSP de linguagem inativa segue rodando;
watcher observa diretórios inúteis.
```

---

## 25. Roadmap desta estratégia

### 25.1 MVP

```text
UI lazy básica;
Tree-sitter rápido;
LSP por escopo;
Project Health incremental;
Configuration Actions sob demanda;
logs com limite em memória;
Background Jobs menu;
Performance Profile: Responsive/Balanced/Full Intelligence;
sem hard cap.
```

### 25.2 Pós-MVP

```text
Background Services detalhado;
Large Workspace Profile;
Project Context profundo;
Inspections ricas;
refatoração IDE-assisted;
melhor cleanup automático;
cache de docs;
support bundle sanitizado.
```

### 25.3 Futuro

```text
build-aware refactoring;
workspace-aware refactoring;
graph visual de targets;
análise de dependências;
modo monorepo;
perfil de performance por workspace;
observabilidade avançada.
```

---

## 26. Critérios de aceite

```text
[ ] Não existe hard cap de memória.
[ ] Resource-on-Demand é padrão da arquitetura.
[ ] UI abre antes de análise profunda terminar.
[ ] Painéis são lazy.
[ ] Listas densas são virtualizadas.
[ ] Logs completos ficam em disco, não todos em RAM.
[ ] LSP inicia por escopo e contexto.
[ ] LSP pode usar força total quando necessário.
[ ] Tree-sitter dá resposta imediata.
[ ] Configuration Actions não calculam tudo antes de serem chamadas.
[ ] Project Health é incremental.
[ ] Inspections aparecem em tempo real sem intrusão.
[ ] Background Services são observáveis.
[ ] Recursos ociosos podem dormir ou ser descartados.
```

---

## 27. Resumo executivo

A Kinein deve ser performática sem virar limitada.

```text
Sem hard cap.
Sem desperdício.
Sem tudo ativo sempre.
Sem UI travada.
Sem análise profunda inútil.
```

A arquitetura correta é:

```text
Resource-on-Demand por padrão.
Full Intelligence quando necessário.
Responsive quando desejado.
Observabilidade sempre.
Controle do usuário acima de tudo.
```

Frase final:

```text
A Kinein não deve economizar poder quando o usuário precisa dele.
Ela deve economizar desperdício quando ninguém está usando.
```
