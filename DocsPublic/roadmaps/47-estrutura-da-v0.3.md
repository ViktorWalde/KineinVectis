# 47 — Estrutura de produto da série v0.3 até a 0.3.5

> **Classe: PLANO EM EXECUÇÃO.** Este documento organiza a conversa de
> 2026-09-22. Ele ainda não substitui a fila viva do roadmap 40 nem transforma
> toda ideia dos roadmaps 45/46 em bloqueio de release. O autor decidiu que a
> série chega à **0.3.5** e que Grafana faz parte desse fechamento. As demais
> decisões da §13 continuam abertas.
>
> Base medida: aplicação `0.2.0`, protocolo `0.130.0`, `main` em `6b8edc6`,
> um arquivo na catraca (`EditorController.qml`), Remote SSH funcional no core
> mas inviável como fluxo cotidiano, terminal real com ergonomia incompleta.
>
> A arquitetura executável, os contratos e o trem de versões proposto estão no
> [`48-arquitetura-executavel-da-serie-0.3.md`](48-arquitetura-executavel-da-serie-0.3.md).
>
> **Início em 2026-09-23:** a primeira fatia do terminal está implementada e
> provada em automação: atalhos híbridos, menu contextual, seleção visível
> honesta, `terminal.clearScrollback` e confirmação de colagem arriscada. O
> roteiro manual shell/TUI/SSH e acessibilidade permanecem abertos; a 0.3.0
> ainda não está fechada. Pesquisa VS Code/JetBrains e divergências explícitas
> estão na especificação do terminal (§3/§7).
>
> **Complemento do autor, 2026-09-23:** Selecionar Tudo completo e nomes
> reutilizáveis de terminal são básicos ainda pendentes. Launcher/abertura e
> interação completa com pastas entram em 0.3.0–0.3.4, sem depender de Grafana;
> bordas superiores mais naturais entram na 0.3.x. A §4 e o roadmap 48
> incorporam esses requisitos sem declará-los implementados.
>
> **Retomada em 2026-09-24:** Selecionar Tudo e nomes reutilizáveis foram
> implementados no protocolo `0.131.0`. A prova shell/TUI/SSH com pessoa e a
> auditoria de acessibilidade continuam bloqueando o fechamento; ver 40 §7.91.
> Na mesma retomada, o autor substituiu a política híbrida: `Ctrl+C` só
> interrompe e `Ctrl+Shift+C` copia, mesmo quando há seleção.

## 1. Tese da versão

**A série v0.3 deve transformar capacidade técnica em fluxo diário confiável
e fechar esse compromisso na 0.3.5.**

Ela não precisa ganhar outra lista extensa de domínios. Precisa fazer as áreas
centrais — editor, terminal, shell e Remote SSH — parecerem uma única IDE em
vez de funcionalidades corretas alcançadas por caminhos diferentes.

Frase de aceite:

```text
Abrir um projeto local ou um espelho SSH, editar, navegar, executar e usar o
terminal não exige reaprender gestos comuns nem abrir painéis de setup a cada
sessão; a IDE nunca afirma um estado que não mediu.
```

## 2. O que a 0.3 herda sem reimplementar

- core Rust, protocolo tipado, Jobs e eventos;
- bridge C++/Qt e `CoreClient` único;
- editor/renderer, Tree-sitter e LSP atuais;
- shell, header, status bar, painel inferior e trilho atuais;
- `CommandDispatcher.qml` e `command.list` atuais;
- Projeto/Git no slot esquerdo e Símbolos na direita;
- terminal PTY/VT real e múltiplas sessões;
- backend Remote: perfis, sonda, deploy, comandos, espelho, pull/push e
  push-on-save;
- Design System `Kv*`, catracas, harnesses e distribuição AppImage.

Tudo isso é base para adaptar. Nada autoriza um segundo shell, dispatcher,
cliente IPC, terminal ou renderer.

## 3. Cinco resultados de produto

### R1 — Shell coerente e extensível

- commands têm um caminho comum;
- tool windows deixam de exigir ramificação nominal em três hosts;
- abas usam identidade estável;
- Projeto, Git, Remote e Símbolos provam o modelo mínimo;
- nenhuma API pública de plugins ou docking genérico entra.

### R2 — Remote SSH melhor que digitar `ssh` para o fluxo suportado

- setup raro fica separado de operação diária;
- alvo, idade da sonda, espelho e sync aparecem sem reabrir formulário;
- save local e push remoto são resultados distintos;
- Run/Debug/Terminal existentes continuam sendo os destinos;
- a versão declara honestamente a limitação de um único escritor enquanto não
  houver detecção de conflito.

### R3 — Editor e terminal respeitam memória muscular

- terminal expõe Copy/Paste/Select All/Clear e usa `Ctrl+C/V` com política
  previsível;
- Markdown alterna entre fonte, preview legível e lado a lado;
- indentação deixa de depender apenas da heurística QML;
- workspace symbols chegam à aba Símbolos pela rota correta;
- assinatura e ocorrências do símbolo entram se a régua de latência permitir;
- nenhuma tecla fica esperando IPC síncrono.

Também inclui abrir o projeto pelo terminal, manipular arquivos/pastas por
mouse e teclado e manter documentos coerentes após operações. A matriz
completa está em
[`projetos-arquivos-e-integracao-desktop-0.3.md`](../especificacoes/projetos-arquivos-e-integracao-desktop-0.3.md);
um drop isolado não atende a esse resultado. O refinamento do cabeçalho e das
bordas está no [sistema de layout §6.5](../especificacoes/sistema-de-layout.md#65-bordas-e-cabeçalho-mais-naturais--compromisso-da-03x).

### R4 — Release verificável

- gate completo, release-hardened e AppImage da versão são provados;
- workspaces/configurações da 0.2 continuam abrindo;
- fluxo local e Remote real são exercitados;
- manual/tutorial descrevem o que a versão realmente faz e suas limitações.

### R5 — Grafana realmente utilizável

- a primeira conexão começa simples e pede autenticação somente se necessária;
- a instância configurada abre no uso diário, não no formulário;
- o cruzamento com bancos do projeto e o acesso aos dashboards são o centro;
- resultados mostram origem/idade e dados antigos não parecem atuais;
- token não é persistido e a política de duração em memória é explícita;
- o fluxo suportado é mais prático que reproduzir chamadas HTTP no terminal.

## 4. Escopo recomendado

### Obrigatório para fechar a série como 0.3.5

```text
V0  baseline, decisões e Grafana/E3-7 registrado para a 0.3.5
V1  ergonomia essencial do terminal (T0–T2)
V2  Remote SSH descobrível e reorganizado (R0.5/R1 da spec Remote)
V3  CommandDispatcher convergido + ToolWindowEntry mínimo
V4  Remote como tool window + HUD honesta (R2 da spec Remote)
V5  identidade estável das abas + preview Markdown M0–M3
V6  L1 workspace symbols + E1 indentação por gramática
V7  Grafana prático G0–G4, incluindo E3-7
P0–P3 launcher + interação completa de projeto, antes da 0.3.5
H0  bordas/cabeçalho mais naturais, previsto na 0.3.x
V8  hardening, dogfooding, migração, AppImage e documentação
```

### Alvo da série, cortável sem mentir sobre a 0.3.5

```text
L2  signature help
L3  document highlight
F7  Símbolos/Structure provando a tool window direita pelo modelo novo
T3  busca no terminal, troca de sessão por teclado e paste seguro completo
C2  superfície "por que este arquivo compila assim" sobre index.context
```

Esses itens entram somente se os obrigatórios estiverem fechados e a latência
da tecla/primeiro frame não regredir. Um alvo cortado é reprogramado para outro
marco da série ou para a 0.4; não é marcado como entregue por estar
parcialmente visível.

### Fora do compromisso mínimo da 0.3.5

- split arbitrário de editores e dock graph; o lado a lado especializado de
  Markdown não inaugura esses recursos gerais. Preview/pin de abas eram
  posteriores no plano inicial; reavaliar com o autor no desenho de P0–P3,
  sem descartá-los diante do pedido de interação completa;
- multi-cursor;
- inlay hints e toda a cauda L4–L9 de uma vez;
- diagnósticos de compilador em todo save (`C1`) antes de medir custo/ruído;
- watcher remoto, resolução de conflito, rename/delete bidirecional (`R3`);
- build geral no alvo, sistema remoto amplo e Yocto/Buildroot profundo;
- stores/DI/registry universais e API pública de plugins;
- Assistente/Chat/AI CLI Bridge ou telemetria de produto/usuário.

Esses itens não foram descartados por dificuldade; têm gatilho e versão
posterior. IA/telemetria são a exceção: estão fora por decisão de produto.

## 5. Ordem por dependência

```text
V0 baseline
 ├─→ V1 terminal ───────────────────────────────┐
 ├─→ V2 Remote reorganizado                     │
 └─→ V6 L1/E1 editor                            │
                                                ↓
V3 commands + ToolWindowEntry → V4 Remote HUD/tool window
                             └→ F7 direita (alvo)

V5 identidade de abas ─────────→ preview tabs/pin/split arbitrário pós-0.3.5
                    └──────────→ preview Markdown seguro

V3 tool windows ───────────────→ V7 Grafana 0.3.5

V5 identidade ─────────────────→ P2/P3 mutações com abas coerentes
P0/P1 projeto ─────────────────→ P2/P3 interação completa antes da 0.3.5

V1..V7 + P0..P3 + H0 + prova real → V8 release
```

V2 vem antes da generalização porque entrega valor e mede o fluxo real usando
os signals atuais. V3 só então extrai o contrato mínimo comprovado por
Projeto/Git/Remote. Isso evita desenhar uma infraestrutura para um painel que
ainda não funciona bem.

V1 vem cedo porque Remote shell, comandos de setup e processos interativos
reutilizam o mesmo terminal. Ergonomia ruim ali contamina todos os domínios.

## 6. Fatias e aceite

### V0 — congelar o contrato da versão

- registrar E3-7/Grafana como obrigatório na 0.3.5, com régua de uso prático;
- capturar baseline em 1024×700 e 1280×800;
- medir digitação, primeiro frame e gate completo;
- executar roteiro local de editor/terminal;
- executar a auditoria de convenções invisíveis da §11.1;
- registrar roteiro Remote com alvo real disponível;
- congelar Must/Target/Out da §4.

Aceite: não há “próxima etapa” divergente entre 40, 45, 46 e 47.

### V1 — terminal ergonômico

> **Concluída em 2026-09-24.** O autor rodou o roteiro real e deu a fatia por
> validada. Era o único item obrigatório da §4 cujo bloqueio não era código.

Fonte: [`terminal-ergonomia-0.3.md`](../especificacoes/terminal-ergonomia-0.3.md).

- menu contextual e ações visíveis;
- política de teclado aprovada: `Ctrl+C` só interrompe, `Ctrl+Shift+C` copia,
  `Ctrl+V` cola (decisão do autor em 2026-09-24);
- limpar tela separado de limpar histórico;
- Selecionar Tudo sobre o buffer completo retido da sessão ativa;
- nomes `terminal`, `terminal1` etc. usam a primeira posição livre, sem
  reutilizar IDs nem conteúdo de sessões encerradas;
- paste arriscado e aliases tradicionais cobertos;
- shell local e SSH real exercitados.

Aceite: uma pessoa encontra Copy/Paste/Clear sem consultar manual e não perde
`SIGINT`, bracketed paste ou AltGr.

Estado em 2026-09-25: seleção completa e nomes reutilizáveis implementados em
`0.131.0`; Bash e Vim reais têm prova automatizada; **o dogfooting do autor e o
SSH real foram feitos** (2026-09-24), e foi isso que fechou a fatia.

O que **continua aberto** são as extensões da §8 da especificação, e elas não
bloqueiam a 0.3.5: busca no scrollback, `Ctrl+PageUp/PageDown` para trocar de
sessão (hoje as teclas são repassadas ao shell, como devem ser quando o foco é
dele), renomear sessão, abrir shell na pasta do arquivo selecionado,
`Ctrl+clique` em paths e URLs, zoom de fonte próprio, copiar-ao-selecionar,
confirmação ao fechar sessão com processo vivo e a indicação **visual** de tela
alternativa (o dado já chega do core; nada o mostra). Dos três alvos que a §8
nomeia para a 0.3 — busca, troca de sessão por teclado e confirmação de paste —
só o terceiro está feito.

Decisão do autor em 2026-09-25: **essas extensões entram depois da V5.** Ações
entram na paleta somente junto da convergência V3, sem dispatcher paralelo.

### V2 — Remote sem formulário monolítico

> **2026-09-24:** as seções, a ação primária por estado e o aceite de 1024×700
> entraram (roadmap 40 §7.98). Continuam pendentes: a escolha da pasta começando
> na home remota, e a tool window lateral/HUD (que são da V4).

- oferecer **usar SSH existente** e listar aliases concretos da configuração
  OpenSSH, sem pedir novamente usuário/porta/chave;
- oferecer **configurar servidor** com teste e terminal guiado para
  confiança/chave, sem guardar senha;
- iniciar escolha da pasta na home remota em vez de exigir caminho decorado;
- Overview, Workspace, Run & Debug, Sistema e Configurar;
- uma ação primária contextual;
- detalhes de comando recolhidos;
- configuração existente editada, não recriada em todo uso;
- 1024×700 mostra contexto e gesto principal sem rolagem de descoberta;
- protocolo/controller preservados salvo lacuna comprovada.

Aceite: se `ssh alias` funciona, o primeiro uso não reconstrói seu perfil; no
segundo uso do mesmo alvo, o usuário não toca nos campos de perfil para
editar/salvar/rodar/abrir shell.

### V3 — espinha do shell

> **Parcial em 2026-09-24 (roadmap 40 §7.99).** Entraram o resultado observavel
> do dispatcher e o `ToolWindowEntry` minimo com o trilho orientado a dado, sem
> mudanca visual. Falta o `componente` da entrada e o slot esquerdo montado a
> partir dela — que so' tem consumidor na V4.

- dispatcher atual recebe resultado observável para ID desconhecido;
- paleta, atalhos, menus e tool windows convergem onde já há command ID;
- `ToolWindowEntry` interno mínimo: id, título, ícone, área, ordem,
  disponibilidade, ativo e componente;
- Projeto/Git provam o slot esquerdo sem mudança visual;
- não criar registry global ou API de plugin.

Aceite: adicionar Remote ao slot não exige novo branching nominal em
`SideRail.qml`, `ShellWorkspaceHost.qml` e `Main.qml` simultaneamente.

### V4 — Remote diário e HUD

> **Parcial em 2026-09-25 (roadmap 40 §7.100).** Entraram a janela no trilho, o
> HUD honesto da barra de status e o caminho para shell, que antes descartava a
> linha quando nao havia sessao aberta. Falta o `componente` da entrada de tool
> window (que segue sem consumidor).

- Remote entra pelo modelo mínimo de tool window;
- workspace espelhado mostra `SSH · alvo · estado de sync` na status bar;
- estado inicial é “não verificado nesta sessão”;
- sonda sempre mostra idade/origem;
- save local bem-sucedido não é confundido com push remoto falho;
- sync/deploy aparece em Jobs e HUD;
- o caminho para shell usa o terminal melhorado da V1.

Aceite: para o fluxo de um único escritor, usar a UI é menos trabalhoso que
montar manualmente `ssh`/`rsync`; fora desse fluxo a limitação aparece antes do
gesto arriscado.

### V5 — identidade das abas

> **Parcial em 2026-09-25 (roadmap 40 §7.101 e §7.103).** A identidade por
> documento entrou inteira: `docId` sintético, `currentTab` derivado, barra de
> abas falando em documento. A prévia de Markdown entrou no corte M1 — modos
> Editar/Preview, buffer não salvo, política de links e HTML desligado. Falta a
> M2: lado a lado e imagens locais exercitadas.

- path/ID estável identifica documento;
- índice fica como detalhe do `ListView`;
- selecionar, fechar, restaurar, renomear e mudança externa operam pelo ID;
- harness reordena/remove e prova que o documento errado não recebe ação;
- não introduzir preview tab, pin ou split arbitrário nesta fatia; o lado a
  lado especializado de Markdown tem escopo e dono próprios.

Aceite: nenhuma operação de domínio depende da posição visual da aba.

Na mesma fundação entra o preview de Markdown, detalhado em
[`markdown-preview-0.3.md`](../especificacoes/markdown-preview-0.3.md):

- modos Editar, Preview e Lado a lado para `.md`/`.markdown`;
- conteúdo não salvo renderizado sem bloquear digitação;
- links relativos abrem no editor e links web no navegador externo;
- HTML executável e recursos remotos ficam bloqueados por padrão;
- imagens locais passam por provider restrito ao workspace;
- debounce identifica documento/versão para nunca renderizar na aba errada.

Aceite adicional: Markdown fica legível sem aplicativo externo, e abrir um
documento não executa HTML nem busca recurso remoto silenciosamente.

### V6 — mínimo da Etapa 4 dentro da 0.3

- L1 separa `workspace/symbol` de `documentSymbol` no C++ e liga a segunda
  fonte da aba Símbolos;
- E1 introduz indentação por gramática com fallback local e sem bloquear tecla;
- latência de digitação medida antes/depois;
- L2/L3 entram apenas depois de L1/E1 verdes;
- `EditorController.qml` não cresce; o corte por responsabilidade acompanha a
  primeira fatia que realmente exigir novo dono.

Aceite: Enter/`}` em C/C++/Rust/Python obedecem casos estruturais medidos, e a
busca de símbolo do workspace não disputa a resposta do outline atual.

### V7 — Grafana prático na 0.3.5

Fonte:
[`grafana-ui-ux-0.3.5.md`](../especificacoes/grafana-ui-ux-0.3.5.md).

- primeira conexão por URL + Conectar, com autenticação sob demanda;
- configuração separada do Overview diário;
- cruzamentos com bancos em primeiro plano;
- filtro e grids navegáveis para fontes/dashboards;
- estado medido com idade, `stale` e próximo passo;
- token nunca persistido e duração em memória decidida explicitamente;
- token vinculado a workspace + URL e invalidado nas fronteiras obrigatórias;
- contribuição ao modelo de tool window, sem estado de domínio no shell;
- teste contra Grafana real, inclusive token ausente/inválido/válido.

Aceite: uma instância já configurada abre direto no estado útil e localizar um
dashboard do projeto é mais simples que consultar a API pelo terminal.

### P0–P3 e H0 — projeto cotidiano e acabamento da janela

- comando `kinein` e abertura de pastas existentes/vazias antes da 0.3.5;
- interação de árvore inteira: foco, teclado, seleção múltipla, menu,
  criar/renomear/excluir, recortar/copiar/colar e arrasto interno/externo;
- distinguir abrir projeto de importar/copiar para a árvore;
- colisão, cancelamento, recuperação e documentos sujos como parte do aceite;
- atualizar abas/árvore/índice/Git pelas rotas existentes após mutações;
- validar integração nativa e artefato, não somente handlers isolados;
- cabeçalho/cantos superiores naturais, sem perder drag/resize/controles.

Aceite: cumprir a matriz da especificação de projetos; não anunciar interação
completa enquanto só abertura/drop estiverem prontos. H0 exige comparação
visual e aprovação do autor. A proposta de marcos está no roadmap 48; defaults
que ampliem contratos são perguntados antes da implementação, não inferidos.

### V8 — fechar e distribuir

- `scripts/verificar.sh` completo;
- debug e release-hardened abrem;
- versão atualizada de forma atômica em Cargo, CMake, AppStream, Sobre e docs;
- AppImage 0.3.5 gerado, checksum e smoke;
- instalação limpa numa máquina sem Qt de desenvolvimento;
- workspace/config/settings da 0.2 reabertos sem perda;
- roteiro local CMake/Cargo/Python;
- roteiro Remote num Linux com `sshd` e `rsync` reais;
- manual, tutorial, changelog/release notes e canal de issues.

Aceite: o artefato distribuído, não apenas o checkout, cumpre a tese da §1.

## 7. Política para Remote na 0.3

A 0.3 pode chamar o Remote de **beta para workspace espelhado de um único
escritor**. Não pode prometer colaboração/bidirecionalidade segura sem R3.

Contrato honesto:

```text
suportado     edição local → push visível; pull explícito; deploy/run/debug/shell
limitado      mudança feita diretamente no alvo; rename/delete; desconexão longa
não prometido dois escritores simultâneos e resolução automática de conflito
```

Se o teste real mostrar que push-on-save pode sobrescrever alteração remota sem
aviso no caminho comum, a saída é bloquear/rebaixar a automação na 0.3, não
maquiar o estado na HUD.

## 8. Política de compatibilidade

- versão do aplicativo (`0.3.5` no fechamento) e versão do protocolo são eixos
  distintos;
- protocolo sobe apenas quando o contrato IPC muda;
- schema `.kinein/workspace.json` não vira `0.3.5` só porque o aplicativo
  mudou; schema sobe somente se o formato mudar;
- mudanças aditivas mantêm defaults seguros;
- formato persistido novo exige teste de leitura do formato 0.2;
- layout novo precisa de versão/fallback e não pode impedir o editor de abrir;
- Remote continua sem senha/token em perfil.

## 9. Orçamento de risco

| Frente | Risco dominante | Trava |
| --- | --- | --- |
| Terminal | quebrar `SIGINT`, TUI ou paste | harness de teclas + shell/TUI/SSH real |
| Remote | UI afirmar conexão/sync falso | estados tipados + horário + Jobs |
| Commands | dois caminhos divergirem | um ID, uma execução, desconhecido recusado |
| Tool windows | big-bang do shell | migração de prova, paridade antes de remover |
| Abas | agir no documento errado | identidade estável e teste de reorder/remove |
| CLI/projeto | mover/perder arquivo ou trocar workspace sem intenção | destino explícito, confinamento, buffers sujos, colisões e cancelamento |
| Bordas/header | perder área clicável ou quebrar resize/composição | estados de janela, escalas e comparação visual |
| E1/LSP | latência ao digitar | nunca esperar IPC; benchmark antes/depois |
| Grafana | formulário ou dado velho parecer operação diária | estados tipados + prova real |
| Release | checkout verde, artefato velho | AppImage gerado/testado depois do gate |

Uma fatia que quebra a régua não avança por estar “quase pronta”; reduz escopo
ou volta para desenho.

## 10. O que pode ocupar marcos intermediários ou seguir após a 0.3.5

Ordem recomendada guiada por dogfooding, sem bloquear o compromisso mínimo:

1. R3 Remote: snapshot, watcher opt-in, conflito, rename/delete;
2. L2/L3 se cortados; L5 formatting por LSP;
3. C2 contexto completo do compilador;
4. C1 diagnóstico do compilador ao salvar, somente com orçamento de ruído e
   CPU definido;
5. preview/pin de abas se não antecipados por decisão em P0–P3; split
   arbitrário de editor permanece uma frente distinta;
6. busca/links/drag-and-drop/zoom do terminal;
7. demais LSP L4/L6–L9 e seleção sintática E4/E5;
8. multi-cursor após pagar o dono restante do editor.

O trem proposto no roadmap 48 distribui o obrigatório entre 0.3.0 e 0.3.5.
Itens desta lista só entram no meio se a prova do marco anterior estiver verde;
do contrário seguem para 0.4 sem serem anunciados como prontos.

### 10.1 Decisão do autor para a 0.4 (registrada em 2026-09-24)

Duas frentes ficam **fora da 0.3** por decisão dele, e não por corte de escopo:

- **os atalhos do trilho da esquerda** precisam de análise de uso e de
  implementação — quais merecem ícone, quais viram só paleta, e o que o ícone
  deve dizer. A remoção do Git do trilho em 2026-09-24 (roadmap 40 §7.99) é o
  primeiro caso dessa conversa, não o fim dela;
- **o ecossistema de embarcados** ganha uma **versão inteira só para ele**:
  layout, atalhos e fluxo. Hoje ele é uma entrada do trilho como as outras, e a
  decisão é que isso não corresponde ao peso que ele tem no produto.

Nada disso bloqueia a 0.3.5. Está aqui para não virar decisão implícita no meio
de outra fatia.

## 11. Métricas que valem

- gestos/cliques do segundo uso de Remote;
- se o gesto principal cabe em 1024×700;
- tempo até feedback de probe/sync/deploy;
- quantidade de estados “desconhecido” tratados honestamente;
- latência da tecla e primeiro frame;
- operações de terminal encontráveis sem manual;
- preview Markdown encontrável, legível e responsivo;
- colisões/atalhos sem acionamento manual;
- regressões de foco, tab order e teclado;
- falhas do roteiro em ambiente real, separadas das cobertas por falsos.

Contar classes, stores ou descriptors não mede sucesso da versão.

### 11.1 Auditoria de convenções invisíveis

Antes de declarar cada superfície pronta, verificar gestos que usuários
normalmente não citam porque esperam que já existam:

```text
clique direito        ações essenciais e estado de habilitação
Ctrl+C/V/X/A          semântica correta para o contexto e aliases conhecidos
Enter/Espaço          ativar o item focado sem mouse
Escape                fechar popup/menu/modal ou limpar seleção transitória
setas/Home/End        navegar lista, grid, tabs e histórico
Tab/Shift+Tab         ordem previsível, foco sempre visível
duplo clique          abrir/confirmar onde a convenção já cria expectativa
Delete/Backspace      efeito claro e confirmação proporcional ao risco
wheel/scrollbar       mesmo alcance, sem perder posição inesperadamente
tooltip               ação curta, atalho e motivo quando desabilitada
foco após ação        voltar ao editor/terminal/item que originou o gesto
empty/loading/error   próximo passo conhecido e retry quando seguro
resize/1024×700       ação principal não desaparece nem exige caça por rolagem
```

O resultado da auditoria vira fatias pequenas ligadas ao fluxo em que foram
encontradas. Não nasce um “framework de ergonomia” e não se acumula uma lista
genérica sem teste. Terminal e Remote são os primeiros consumidores porque já
há dor explícita; editor, explorer, Git, diálogos e painéis seguem o mesmo
roteiro quando forem tocados pela 0.3.

## 12. Definição de pronto da 0.3.5

Todos os itens obrigatórios da §4 estão entregues; os alvos cortados estão
explicitamente reprogramados; não há capacidade parcial anunciada como pronta;
gates e artefato estão verdes; os roteiros reais passam:

```text
LOCAL   abrir → editar → indentar/navegar → terminal → build/run/debug
PROJETO CLI/drop → navegar/selecionar → copiar/mover/renomear → recuperar/cancelar
MARKDOWN editar → preview → lado a lado → link/imagem local
REMOTE  abrir espelho → editar/salvar → ver push → run/shell → falha de rede
GRAFANA conectar → autenticar se necessário → cruzar banco → abrir dashboard
```

O teste Remote precisa provar também o caso de falha: a alteração local
permanece segura e a HUD não diz “sincronizado”.

## 13. Decisões do autor para congelar a versão

1. Confirmar a tese **uso diário confiável** como identidade da 0.3, em vez de
   uma release centrada somente em LSP/compiladores.
2. **Revisado em 2026-09-24:** `Ctrl+C` só interrompe, `Ctrl+Shift+C` copia e
   `Ctrl+Alt+V` para `^V`, conforme a especificação do terminal §11.
3. Remote nasce no slot esquerdo ou direito? A recomendação continua esquerda,
   porque troca contexto de trabalho.
4. A série 0.3 aceita oficialmente o modo Remote de um único escritor, com
   watcher e conflitos depois da 0.3.5?
5. L2/L3 são alvo cortável, como proposto, ou bloqueiam a 0.3.5?
6. C2 (“por que este arquivo compila assim”) é alvo intermediário da série ou
   primeira fatia da 0.4?
7. O trem 0.3.0–0.3.4 proposto no roadmap 48 deve ser congelado ou
   redistribuído?
8. Preview Markdown começa em Editar, como recomendado, e imagens remotas
   permanecem bloqueadas por padrão?

As definições posteriores de Selecionar Tudo, nomes de terminal, launcher e
interação completa de pastas antes do Grafana e bordas na 0.3.x já foram dadas
pelo autor. Não são perguntas reabertas. Defaults e limites adicionais de
projeto estão na §7 da especificação de projetos, para decisão na fatia certa.

As decisões ainda abertas não são preenchidas por inferência. Até a resposta,
este documento organiza a análise e não autoriza remover alternativas existentes.
