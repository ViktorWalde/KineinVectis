# 48 — Arquitetura executável da série 0.3 até a 0.3.5

> **Classe: PLANO EM EXECUÇÃO.** A decisão confirmada em 2026-09-22 é:
> Grafana faz parte da série 0.3 e fecha na **0.3.5** com uso prático, simples
> e ergonômico. A distribuição das outras fatias entre 0.3.0–0.3.4 abaixo é
> uma proposta arquitetural, não uma decisão atribuída ao autor.
>
> Este documento aprofunda o
> [`47-estrutura-da-v0.3.md`](47-estrutura-da-v0.3.md). O 47 delimita produto e
> escopo; este define fronteiras, contratos, estado, migração, prova e pontos
> de corte para implementar sem um rewrite.
>
> **Primeira execução, 2026-09-23:** `0.130.0` implementa a fatia local de
> terminal (teclado, menu, seleção visível, limpeza isolada do scrollback e
> confirmação de colagem arriscada). O corte 0.3.0 continua aberto até o
> roteiro manual shell/TUI/SSH e acessibilidade serem provados; testes
> automáticos da política de colagem não substituem essa validação.
> **Complementos de 2026-09-23:** Selecionar Tudo real e nomes de sessão
> reutilizáveis são básicos pendentes, bloqueadores da 0.3.0. Launcher e
> interação completa com pastas entram antes da 0.3.5; bordas/cabeçalho mais
> naturais também entram na série. Nada disso está entregue por constar aqui.
>
> **Retomada em 2026-09-24:** seleção completa e nomes reutilizáveis estão
> implementados em `0.131.0`; desenho e provas na §7.4 e no roadmap 40 §7.91.

## 1. Resultado ao fim da 0.3.5

```text
projeto local ou remoto
→ shell coerente e comandos por uma rota
→ terminal com gestos cotidianos encontráveis
→ Remote simples para começar e silencioso quando já configurado
→ editor sem identidade frágil por índice
→ mínimo de linguagem que não bloqueia a tecla
→ Grafana útil para localizar a observabilidade do projeto
→ artefato distribuído e testado, não só checkout verde
```

A série não precisa concluir a arquitetura futura inteira. Ela precisa criar
seams reais para crescer e provar, em fluxos cotidianos, que as capacidades
existentes ficaram mais simples que seus equivalentes montados no terminal.

## 2. Invariantes

1. um `CoreClient`, um protocolo, um dispatcher, um shell e um terminal;
2. domínio continua dono dos fatos; shell só compõe e navega;
3. tecla/editor nunca espera IPC síncrono;
4. rede e processo longo sempre são Jobs ou operação assíncrona observável;
5. “conectado”, “sincronizado” e “atualizado” exigem medição e idade;
6. índice visual nunca é identidade de documento;
7. perfil não recebe senha ou token;
8. detalhe técnico fica acessível sem dominar o caminho feliz;
9. compatibilidade é por contrato/schema, não pelo número comercial da app;
10. nenhuma generalização entra antes de dois consumidores reais a provarem;
11. telemetria de produto/usuário e assistente de IA não entram;
12. as afirmações medidas da IDE só são alteradas com evidência ou decisão
    explícita do autor.

## 3. Trem de versões proposto

O trem abaixo reduz o raio de falha e permite dogfooding entre marcos. Grafana
na 0.3.5 e launcher/pastas antes dele estão decididos pelo autor. A alocação
exata em 0.3.0–0.3.4 permanece proposta; bordas têm compromisso com a 0.3.x.

| versão | corte proposto | prova que libera o corte seguinte |
| --- | --- | --- |
| 0.3.0 | baseline + terminal T0–T2, Selecionar Tudo e nomes reutilizáveis | buffer completo e isolamento por sessão; teclas, TUI e SSH sem regressão |
| 0.3.1 | Remote R0.5/R1 + P0 launcher/abertura de pasta | pasta vazia/preenchida pela CLI; SSH existente e novo; segundo uso sem formulário |
| 0.3.2 | commands + tool windows mínimas + Remote R2 + P1 seleção/ações da árvore | mesma intenção por teclado/menu; Projeto/Git/Remote e HUD honesta |
| 0.3.3 | identidade estável das abas + Markdown M0–M3 + P2 clipboard/arrasto | documentos certos após mover/renomear; preview seguro/responsivo |
| 0.3.4 | P3 interação completa de projeto + bordas/cabeçalho + L1/E1 | matriz de arquivos e desktop completa; geometria, latência e correção medidas |
| 0.3.5 | Grafana G0–G4 + integração/hardening | Grafana e Remote reais, migração, AppImage e docs |

Uma fatia pode mudar de marco sem mudar o destino final. A versão seguinte não
herda uma base vermelha; item incompleto não é renomeado como entregue.
P0 pode antecipar para 0.3.0 se não atrasar seus bloqueios. As adições ampliam
o trabalho da série: não são um prazo estimado nem autorização para esconder
recursos faltantes sob um único drop funcionando. Ver §13.1.

## 4. Topologia de responsabilidades

Desenho-alvo, não inventário de tipos implementados. Os nomes lógicos abaixo
não autorizam criar controllers duplicados: o terminal usa hoje
`RuntimeController`, `TerminalInputController`, `TerminalSelectionController`
e `TerminalScrollController`; a convergência deve estender esses donos.

```text
Main / AppDomains
  ├─ ShellController             visibilidade, foco e layout
  ├─ CommandDispatcher           rota única de intenção
  ├─ ToolWindowCatalog (QML)     descriptors internos, sem estado de domínio
  ├─ EditorDocumentController    identidade e ciclo de documentos
  ├─ TerminalController          sessões e transporte terminal
  ├─ RemoteController            fachada do domínio Remote
  └─ GrafanaController           fachada do domínio Grafana
             │ signals tipados
          CoreClient
             │ JSON-RPC
          core Rust
```

O catálogo de tool windows não vira service locator. Ele responde apenas:
“qual superfície existe, em qual área, em qual ordem e qual componente montar”.
Controllers de domínio continuam instanciados/ligados como hoje.

## 5. Commands: uma intenção, uma execução

### 5.1 Estado atual preservado

`command.list` fornece metadados e `CommandDispatcher.qml` executa a intenção.
O dispatcher atual é condicional e pequeno; uma registry universal agora
troca clareza por indirection sem resolver um problema medido.

### 5.2 Contrato interno

```text
CommandDescriptor
  id · title · category · description · defaultShortcut · requiresWorkspace

dispatch(id, context?)
  → accepted
  → rejected(reason)
  → unknown(id)
```

- header, menu, paleta, atalho e contexto chamam o mesmo `id`;
- desconhecido é observável em desenvolvimento/teste;
- habilitação vem do dono do estado, não de cópia no menu;
- argumentos internos só entram quando houver consumidor real e tipado;
- o hack textual `id=arg` não deve crescer como protocolo paralelo;
- comandos puramente visuais não viram método IPC.

### 5.3 Migração

1. inventariar IDs já existentes e seus chamadores;
2. convergir ações duplicadas sem mudar aparência;
3. acrescentar resultado observável;
4. migrar tool windows;
5. remover somente o caminho antigo que tiver paridade coberta.

## 6. Tool windows: composição mínima

### 6.1 Descriptor interno

```text
ToolWindowEntry
  id
  title
  icon
  area            left | right | bottom
  order
  available
  active
  component
```

Não entram nesta série: API pública de plugin, docking arbitrário, drag entre
áreas, múltiplas instâncias genéricas ou árvore universal de layout.

### 6.2 Donos do estado

```text
ToolWindowCatalog   quais entradas compõem cada área
ShellController     ativa/oculta, largura/altura, último foco
Domain controller   conteúdo, loading, erro e ações
Host genérico       renderiza entry; não conhece Remote/Grafana/Git
```

Persistência, quando entrar, usa IDs estáveis e versão própria. ID desconhecido
é ignorado; área inválida usa default; layout corrompido não impede o editor de
abrir.

### 6.3 Ordem de prova

- Projeto/Git adaptam o slot esquerdo sem mudança visual;
- Remote prova uma terceira entrada e estados cotidianos;
- Símbolos prova a direita;
- Grafana 0.3.5 usa o mesmo modelo;
- branching nominal antigo só sai depois da paridade.

## 7. Terminal: ação ergonômica sobre o emulador existente

Fonte detalhada:
[`terminal-ergonomia-0.3.md`](../especificacoes/terminal-ergonomia-0.3.md).

### 7.1 Camadas

```text
menu/atalho/context menu
       ↓
Terminal actions          copy · paste · select · clear · session
       ↓
TerminalPanel             compõe os donos existentes, não outro controller
  input/selection/scroll  estado local e política de interação
  RuntimeController      sessão ativa e roteamento
       ↓
terminal.* IPC            somente o que pertence ao PTY/grid
       ↓
alacritty_terminal/PTy
```

Copy/Paste e menu não justificam outro emulador. Por decisão do autor em
2026-09-24, `Ctrl+C` sempre envia `0x03`, com ou sem seleção; `Ctrl+Shift+C`
copia sem consumir a seleção. `Ctrl+V` cola e `Ctrl+Shift+V` permanece.

### 7.2 Limpar e selecionar honestamente

- **Limpar tela** envia o gesto equivalente ao shell/terminal;
- **Limpar histórico** é ação explícita sobre o grid e não apaga arquivo/log;
- o core atual já permite limpar history no emulador; um método tipado pequeno
  é preferível a simular sequências frágeis;
- a seleção por arrasto QML alcança o grid renderizado. Desde `0.131.0`,
  **Selecionar Tudo** usa a seleção nativa no core e lê o buffer sob demanda;
  **Selecionar área visível** permanece uma ação separada;
- seleção completa usa o buffer retido da sessão ativa; sessão nova começa
  limpa mesmo com o mesmo rótulo. Não duplicar o scrollback em QML;
- nomes usam a primeira posição livre: `terminal`, `terminal1` etc. O
  `RuntimeController` existente continua dono das abas; IDs técnicos não são
  reutilizados. Busca é um alvo separado desses requisitos básicos.

### 7.3 Segurança do paste

Bracketed paste continua respeitado. A confirmação implementada ocorre antes
de enviar bytes, mostra preview limitado e oferece Cancelar/Colar em uma
linha/Colar. Estado pertence ao input existente; a superfície compõe
`KvPanelFrame`/`KvButton`. Detalhes e diferenças frente a VS Code estão na
especificação §7. AltGr, política de paste e roda têm cobertura automática;
TUI/SSH reais e captura completa de cliques/arrasto não estão provados.

### 7.4 Retomada da seleção completa — desenho de 2026-09-24

A árvore da sessão interrompida já contém o núcleo de `0.131.0`, ainda sem
validação integrada/documentação. Concluir essa fatia antes de Remote:

- `TerminalState` usa a seleção nativa do emulador sobre o buffer ativo
  retido; `session/selection.rs` publica a identidade sob o mesmo lock do
  primeiro frame e extrai texto somente ao copiar. Saída, resize e limpeza
  invalidam; scroll do viewport preserva. Não há segundo histórico.
- Contrato: `terminal.selectAll { id, selectionId } -> { id, selectionId }`
  e `terminal.copySelection { id, selectionId } -> { id, selectionId, text }`.
  `text: null` indica seleção obsoleta; `event.terminal.render.selectionId`
  confirma/invalida a marca. Texto não entra no log da bridge.
- `CoreClient` e routers Runtime transportam os contratos; o controller de
  seleção filtra sessão/gesto/resposta atrasada. `TerminalActionsController`
  concentra as ações que estavam no painel, mantendo input/scroll/seleção
  nos donos existentes.
- `RuntimeController` escolhe o primeiro rótulo livre, mantendo IDs únicos e
  títulos de Run/container. Harness prova fechar/criar sem herdar saída.
- Teclado usa o menu contextual (`Shift+F10`/Menu, setas e Enter). `Ctrl+A`
  continua no shell e `Ctrl+Shift+A` na paleta. Auditar a precedência do
  `Shift+F10` global (Executar) com foco no terminal: teste de controller
  isolado não prova entrega real do evento Qt.
- Provas: despacho Rust de duas sessões, Unicode/wrap/tela alternativa,
  invalidação e retenção limitada; harness de composição/teclado e respostas
  atrasadas; gate integrado, build e abertura debug/release. Registrar
  separadamente o que depender de pessoa e de um SSH real.

Referências: [VS Code Terminal Basics](https://code.visualstudio.com/docs/terminal/basics)
não reserva Select All no Linux para evitar conflito com o shell;
[Qt Keys](https://doc.qt.io/qt-6/qml-qtquick-keys.html#shortcutOverride-signal)
define a precedência de `shortcutOverride`. Consultadas em 2026-09-24;
adaptação sobre os componentes atuais, sem código transplantado.

## 8. Remote: da descoberta ao uso diário

Fonte detalhada:
[`remote-ssh-ui-hud.md`](../especificacoes/remote-ssh-ui-hud.md).

### 8.1 Dois inícios explícitos

```text
SSH já funciona                  SSH ainda não funciona
listar aliases concretos        host + usuário
→ escolher                      → testar
→ testar                        → terminal guiado para confiança/chave
→ escolher pasta                → retestar
```

O modelo atual já permite guardar o alias em `RemoteTarget.host` e deixar
user/port/identity ausentes. A lacuna é descoberta e explicação.

Proposta de contratos, a validar na fatia:

```text
remote.discover {}
  → aliases [{ name, source }]

remote.resolve { host }
  → { hostName?, user?, port?, identities[], proxy? }

remote.directories { name, path? }
  → { path, parent?, entries [{ name, path }] }
```

- descoberta lê apenas config local autorizada e não varre rede;
- somente `Host` concreto aparece; padrões continuam aplicados pelo OpenSSH;
- resolução deve preferir `ssh -G` e retornar allowlist segura, nunca o dump
  inteiro nem comandos arbitrários;
- navegador remoto lista diretórios por contrato tipado e escaping no core;
- senha/passphrase/host key continuam no terminal do OpenSSH;
- novo contrato só entra após teste provar que a alternativa não atende.

### 8.2 Máquina de estados

```text
discovery       idle | loading | ready | failed
setup           incomplete | testing | ready | action_required
availability    unknown | probing | reachable | unreachable
workspace       local | mirror
sync            idle | pulling | pushing | synced | failed | attention
deployment      idle | running | success | failed
```

Cada transição registra o alvo e a operação que a originaram. Resposta atrasada
de alvo A não altera o alvo B. “reachable” é última sonda desta sessão com
horário, não conexão persistente.

### 8.3 Fachada e possível divisão futura

`RemoteController` permanece fachada enquanto couber na catraca. Se a máquina
de estados realmente crescer, filhos possíveis são:

```text
RemoteSetupController
RemoteHealthController
RemoteWorkspaceController
RemoteExecutionController
```

Eles não nascem preventivamente. A view nunca chama o `CoreClient` direto, e o
shell não absorve estado Remote.

### 8.4 Superfícies

- tool window: Overview, Workspace, Run & Debug, Sistema, Configurar;
- status bar: somente workspace espelhado, sync/job e saúde medida;
- header: configuração de Run/Debug mostra `Local` ou `SSH: alvo`;
- terminal: sessão existente nomeada `SSH · alvo`;
- Jobs/Problems: progresso e falha com próximo passo.

O segundo uso não mostra formulário. Se `ssh alias` já funciona, o primeiro
uso não pede seus detalhes novamente.

## 9. Abas e identidade de documento

### 9.1 Problema

Hoje várias operações usam índice. Reordenar, fechar ou receber resposta
assíncrona pode fazer uma intenção atingir outro documento.

### 9.2 Modelo alvo

```text
Document
  documentId      identidade runtime imutável
  path            atributo mutável e persistível
  displayName
  content/version/dirty/readonly/...
```

- `EditorDocumentController` gera o ID ao abrir/restaurar;
- sinais de domínio carregam `documentId`; a view traduz ID para índice;
- rename muda path, não identidade;
- sessão persistida pode continuar guardando paths e gerar IDs novos no
  restore; persistir o ID só entra se um layout futuro precisar;
- respostas assíncronas carregam documento + versão e são descartadas quando
  obsoletas;
- preview/pin de abas e split arbitrário ficam para depois dessa fundação; o
  lado a lado especializado de Markdown não altera o modelo de abas.

Migração é incremental: acrescentar ID ao model, criar lookup, converter ações
de maior risco, cobrir reorder/remove e só então proibir operações de domínio
por índice.

## 10. Markdown legível sem um navegador embutido

Fonte detalhada:
[`markdown-preview-0.3.md`](../especificacoes/markdown-preview-0.3.md).

```text
documentId + path + bufferVersion + conteúdo
                   ↓ debounce/cancelamento
MarkdownPreviewPane (QML)
                   ↓
QTextDocument MarkdownNoHTML + stylesheet + resource provider restrito
```

- a versão mínima Qt 6.4 já possui renderer Markdown; WebEngine não entra;
- Editar, Preview e Lado a lado são modos da mesma aba/documento;
- o buffer não salvo alimenta o preview sem novo IPC;
- identidade/versão impedem debounce atrasado de atualizar outra aba;
- links relativos usam a abertura de arquivo existente e URLs web abrem no
  navegador externo;
- HTML/script e imagens remotas ficam bloqueados por padrão;
- imagens locais são resolvidas pela pasta do documento e aceitas somente no
  escopo permitido do workspace;
- documento oculto não renderiza; documento grande possui limite/ação manual se
  a medição mostrar risco para a latência da tecla.

O split do Markdown é uma composição especializada e não inaugura split
arbitrário do editor. Preview/pin e múltiplos grupos continuam posteriores.

## 11. Linguagem e indentação sem bloquear digitação

### 11.1 L1 — workspace symbols

O core já pede `workspace/symbol`; a bridge C++ precisa separar a resposta de
`documentSymbol`. Não há método novo de LSP. A aba Símbolos deduplica por
arquivo/posição e mantém a fonte visível ao usuário quando isso ajudar a
explicar divergência.

### 11.2 E1 — indentação

```text
tecla Enter/}
→ UI aplica fallback local imediatamente
→ solicita lang.indent(documentId/path, bufferVersion, posição)
→ resposta chega
→ aplica correção somente se documento, versão e cursor ainda coincidirem
```

Não há chamada síncrona no caminho da tecla. Tree-sitter/core fornece regra
estrutural; QML mantém fallback para linguagem/estado sem árvore. Casos medidos:
C, C++, Rust, Python, comentários, pares vazios, dedent e paste.

L2/L3 seguem o mesmo padrão: debounce/cancelamento, identidade + versão e
resposta obsoleta descartada. Não entram juntos só porque compartilham “LSP”.

## 12. Grafana obrigatório na 0.3.5

Fonte detalhada:
[`grafana-ui-ux-0.3.5.md`](../especificacoes/grafana-ui-ux-0.3.5.md).

### 12.1 Fronteira

```text
tool window
  ├─ conexão curta, só quando ausente/editada
  └─ overview diário: estado → matches → filtro → dashboards
             ↓
      GrafanaController
             ↓
      grafana.* existente
             ↓
      HTTP API do Grafana externo
```

O backend atual continua. A 0.3.5 não embute Grafana, não cria dashboard e não
persiste token. “Polimento” só estará feito quando o fluxo diário for simples,
não quando componentes tiverem sido renomeados.

### 12.2 Estado

```text
setup       absent | editing | saved
probe       unknown | probing | reachable | failed
auth        not_required | required | authenticated | failed
content     unknown | loading | ready | empty | stale
```

Resultado pertence à URL/workspace/requisição que o produziu. Trocar URL
invalida os dados; falha pode manter o último conteúdo apenas como `stale`.
Horário/origem impedem resultado antigo de parecer medição atual.

### 12.3 Fluxo

- primeira conexão começa com URL e **Conectar**;
- token aparece somente se necessário;
- perfil salvo diz o que persiste e o que não persiste;
- uso diário abre no overview, não no formulário;
- cruzamento entre banco do projeto e datasource tem prioridade;
- dashboards/fontes usam grid, filtro, seleção e teclado;
- dashboard abre no navegador;
- token continua efêmero e redigido.

Decisão de 2026-09-22: fechar a tool window mantém o token somente em memória;
trocar/fechar workspace, confirmar outra URL, esquecer credencial/instância,
receber rejeição de autenticação ou encerrar a aplicação o invalida. O token é
vinculado ao par workspace + URL confirmada, nunca dispara refresh sozinho e
permanece fora de perfil/log/view. A limitação de apagamento de strings QML e o
alvo de remover a custódia duradoura do QML para estado privado da bridge C++
estão documentados na spec, sem promessa falsa de zeroização perfeita. O
protocolo atual permanece; a bridge só inclui o token quando o contexto confere.

## 13. Persistência, versões e migração

- aplicativo chega a `0.3.5`; protocolo sobe apenas por método/evento novo;
- schema de workspace só sobe quando o formato persistido mudar;
- campos aditivos possuem default seguro;
- layout do shell tem versão própria e fallback;
- `RemoteTarget.host` existente continua válido; alias é uso compatível;
- IDs runtime de documento não precisam migrar;
- perfil Grafana mantém URL/política e nunca ganha token;
- leitura de workspace/configuração 0.2 é teste obrigatório;
- escrita nova não ocorre apenas por abrir um workspace antigo.

### 13.1 Integração com pastas/desktop e refinamento do chrome

Fonte de P0–P3:
[`projetos-arquivos-e-integracao-desktop-0.3.md`](../especificacoes/projetos-arquivos-e-integracao-desktop-0.3.md).
O launcher já encaminha argumentos e a bridge já abre uma pasta existente.
`ProjectTreeController` e `fsops` já criam/renomeiam/excluem. Estender esses
donos: nenhuma dessas frentes justifica outro workspace, explorer, dispatcher
ou motor de filesystem.

CLI e drop de abertura convergem no mesmo fluxo de workspace. Recortar/colar,
menu e arrasto na árvore convergem na mesma operação de arquivos, com destino
visível, proteção de buffers sujos e validação no core. Importar de fora não
autoriza remover confinamento. Lote longo usa Jobs; resultado parcial não vira
sucesso global. Identidade estável das abas precede mutações que dependam dela.

A especificação registra a matriz completa e decisões ainda abertas de
janela, arquivo avulso/multi-root, preview/pin e recuperação. Não eliminar
esses casos por serem mais difíceis nem tratar apenas P0 como paridade.

**H0 — bordas/cabeçalho:** previsto para a 0.3.x, proposto na 0.3.4. O contrato
está no [sistema de layout §6.5](../especificacoes/sistema-de-layout.md#65-bordas-e-cabeçalho-mais-naturais--compromisso-da-03x).
Reutilizar Theme/header/chrome, medir janela restaurada/maximizada e escalas,
preservar controles, drag/resize e orçamento de primeiro frame. O pedido de
“rodapé superior” foi registrado como interpretação provisória do cabeçalho,
a confirmar antes de fixar geometria, não como mudança já feita.

## 14. Prova por marco

| frente | prova automatizada | prova real/manual |
| --- | --- | --- |
| commands | ID conhecido/desconhecido, habilitação e rota única | menu/paleta/atalho fazem o mesmo |
| tool windows | catálogo, alternância, fallback de layout | 1024×700 e 1280×800 |
| terminal | teclas, seleção, clear, paste, sessões | shell, TUI e SSH |
| Remote | máquina de estados, resposta obsoleta, sync falho | alias existente + servidor do zero + espelho real |
| abas | reorder/remove/rename/version | edição e restauração reais |
| CLI/projetos P0–P3 | argumentos, destino, seleção, colisões, confinamento, buffers sujos e falha parcial | pasta vazia/preenchida, clipboard de arquivos e drag-and-drop no desktop/artefato |
| bordas H0 | estados/geometria e regressão de hit targets onde automatizável | antes/depois, restaurar/maximizar, escalas e X11/Wayland suportados |
| Markdown | links/recursos, documento/versão, arquivo grande | leitura, split e imagens locais no AppImage |
| L1/E1 | fonte correta, casos de indent, latência | C/C++/Rust/Python reais |
| Grafana | estados, token redigido, filtro/grid | instância real: sem token, inválido e válido |
| release | gates, migração e smoke | AppImage em máquina sem Qt de desenvolvimento |

Cada marco mede latência de digitação e primeiro frame quando tocar shell/editor.
O gate completo roda antes do artefato; o artefato também é exercitado.

## 15. Seams de rollback

- commands: adaptador mantém chamador antigo até paridade;
- tool windows: host antigo pode coexistir por uma fatia, nunca indefinidamente;
- terminal: novos atalhos podem ser desligados sem trocar o emulador;
- Remote: reorganização R1 preserva signals; contratos de descoberta são
  aditivos;
- abas: path continua disponível durante migração para ID;
- Markdown: Editar permanece disponível; preview não altera conteúdo/disco;
- E1: fallback local permanece se resposta estrutural falhar/atrasar;
- Grafana: core e perfil existentes permanecem; overview novo pode voltar à
  view anterior sem perda de dados.

Rollback não significa dois donos permanentes. A remoção do caminho anterior é
parte do aceite da fatia seguinte.

## 16. Decisões ainda necessárias

1. confirmar ou redistribuir o trem 0.3.0–0.3.4 da §3;
2. **revisado pelo autor em 2026-09-24:** `Ctrl+C` só interrompe,
   `Ctrl+Shift+C` copia; `Ctrl+Alt+V` envia `^V`;
3. confirmar o momento de iniciar o primeiro shell (projeto ou painel);
   alcance de Selecionar Tudo já está decidido: buffer inteiro da sessão;
4. Remote inicia no slot esquerdo, como recomendado, ou direito;
5. confirmar Remote de um único escritor na série 0.3;
6. escolher a precedência de alvo por workspace/configuração de execução;
7. dizer se L2/L3 e C2 entram antes da 0.3.5 ou seguem para 0.4.
8. confirmar o modo inicial de Markdown e se imagens remotas continuam
   bloqueadas por padrão.

Grafana na 0.3.5, launcher/pastas antes dele e os requisitos básicos do terminal
já não são perguntas. CLI, arquivos avulsos/multi-root, preview de abas e
recuperação de arquivos têm decisões específicas na §13.1; a região exata das
bordas superiores será confirmada antes da implementação visual. As propostas
não são atribuídas ao autor como se já estivessem aprovadas.
