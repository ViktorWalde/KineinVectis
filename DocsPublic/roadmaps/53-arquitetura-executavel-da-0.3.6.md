# 53 — Arquitetura executável da 0.3.6: a casca da IDE em torno do editor

> **Classe: PLANO / ARQUITETURA.** Escrito em 2026-10-01, a pedido do autor:
> *"estruturar toda a documentação de arquitetura antes de qualquer código"*.
> O [roadmap 49](49-frontend-0.3.6-e-sequencia-0.5.md) diz **o quê** e **por
> quê** (cortes F0–F5); este documento diz **como**, com dono, fluxo, contrato,
> estado, erro e prova de cada parte. Nada aqui está implementado por estar
> escrito. O código e o [roadmap 40](40-estado-e-continuidade.md) prevalecem
> para o estado real; cada sessão relê a seção da fatia antes do código.
>
> **O eixo, igual ao do [roadmap 52](52-arquitetura-executavel-da-0.4.md):**
> evitar código duplicado. A §3 dá, para cada pergunta da casca, **o dono que
> já existe**, o que se acrescenta e o que é proibido criar ao lado.
>
> **Fontes:** [49](49-frontend-0.3.6-e-sequencia-0.5.md) (F0–F5 e o adendo
> §9 da Library), [50 §8](50-biblioteca-e-providers-0.5.md) (o que a 0.5 vai
> pôr na casca), [52 §7](52-arquitetura-executavel-da-0.4.md) (o que a 0.4
> vai pôr na casca), os estudos privados em `DocsPrivate/documentacoes/`, o
> diário `DocsPrivate/uso-diario.md` e a [especificação de frontend](../especificacoes/arquitetura-de-frontend-0.3-em-diante.md).

## 0.1 Princípio do autor (2026-10-01): ruído não se esconde, se elimina

> *"Essa parte não deve ser exibida de forma alguma para o usuário final [...]
> ou essa parte poluída desta forma não deveria de fato existir, e vai ter que
> ser feita uma otimização para resolver esse problema de fato e não apenas
> mascarar."*

Consequências, que valem para toda a série 0.3.6–0.3.9:

1. **A meta é zero mensagem produzida**, não zero mensagem exibida. Cada aviso
   do Qt/QML (`Component is not ready`, binding loop, `wayland-egl`, o que mais
   o passeio da §5.2 achar) é rastreado até a causa e corrigido na causa.
2. **Redirecionar não é correção.** O *message handler* da §5.1 existe só como
   rede de segurança para o que ainda escapar: grava no arquivo de log de
   diagnóstico e **nunca** aparece na interface. Silenciar uma categoria do Qt
   para esconder um aviso é proibido; se uma mensagem é inevitável e inofensiva,
   a fatia prova isso e documenta por quê.
3. **O usuário final não vê internos.** A aba IDE do painel inferior mostra só
   o que é dele (falha de operação que ele pediu, com ação), nunca o stderr do
   Qt. O arquivo `~/.cache/kinein-vectis/logs/` é para relatório de defeito.
4. **O gate garante:** qualquer linha da lista `scripts/avisos-qml.txt` em
   qualquer superfície, no checkout ou no AppImage, reprova.

## 0.2 G0 — gates antes de qualquer código de produto (decisão do autor, 2026-10-01)

> *"Vamos precisar, pelo visto, criar gates rigorosos antes de qualquer linha de
> código."* — depois de achar identificadores em português num código novo,
> contra a regra de [contribuindo/08](../contribuindo/08-convencoes-codigo-testes-commits.md)
> que não tinha gate.

Nenhuma fatia de produto da 0.3.6 (§5.3 em diante) começa antes destes gates
existirem e passarem. Cada gate é provado por mutação (o defeito que ele
promete pegar é introduzido de propósito e ele reprova).

| Gate | O que reprova | Estado |
| --- | --- | --- |
| G0.1 `scripts/check_identifier_language.py` | identificador novo em português em Rust, C++, QML/JS, Python, shell e Python em heredoc; legado em catraca | **feito** 2026-10-01; mutação (`contadorDePassos` reprovou `contador` e `passos` na linha) |
| G0.2 `scripts/verificar-qml-qt64.sh` | parte que o Qt 6.4 do AppImage nunca cria num arquivo com `pragma Bound` | **feito** 2026-10-01; mutação contra o HEAD (as quatro ocorrências) |
| G0.3 `scripts/verificar-qml-logica-qt64.sh` | qualquer harness que passa no Qt do checkout e falha no do pacote; **aviso da lista `avisos-qml.txt` na saída de qualquer harness** (nos dois Qt) | **feito** 2026-10-01. Container Debian 12 próprio (`Containerfile.qml64`, só `qml-qt6` e os módulos usados), 17 s. Novo `tst_list_parts_render` instancia `GitChangesList` e `SymbolResultsList`; mutação: `section.delegate` e `header` inline passam no 6.10 e reprovam no 6.4 com "Component is not ready". O grep de avisos pegou um falso incompleto no `tst_editor_persistence` ("Unable to assign"), já corrigido |
| G0.4 `scripts/run-surface-tour.sh` + `scripts/surface-tour.txt` | aviso da lista `scripts/avisos-qml.txt` em qualquer superfície alcançável por id (nomeando o **passo** dono), id do roteiro que ninguém trata, passeio que não chega ao `@quit` ou IDE que não sai com 0 | **feito** 2026-10-01. 33 passos de 700 ms num projeto com mudanças git em três pastas (as seções do Git existem), em ~25 s; roda no `verificar-binario-abre` (checkout) e no `testar-appimage` (host e Debian mínimo, Qt 6.4). Mutações: `ReferenceError` no cabeçalho de seção do Git (reprovou em `[git.status]`) e id com erro de digitação. Áreas do trilho, abas de baixo e redimensionar entram no roteiro com os comandos de área da F1 (§6.2) |
| G0.5 `scripts/check_terminal_quiet.py` | `kinein <pasta>` num pty que não volte em < 300 ms, que imprima algo (o pai **ou** a IDE desacoplada depois), cuja IDE não chegue ao primeiro frame, ou cujo log de diagnóstico receba qualquer mensagem; `--verbose`/`--wait` que soltem o terminal; erro de caminho e `--help` mudos | **feito** 2026-10-01, no modo completo do `verificar.sh`. XDG isolados num temporário; medido 32 ms. Mutações: filho sem `dup2` do stderr, desacoplamento desligado e um `qWarning` na abertura, cada uma reprovada com a sua mensagem |

### Varredura de idioma, em fatias próprias (logo depois do G0)

Medido em 2026-10-01: **17.862 ocorrências** em 4.054 pares arquivo/palavra (o
"cerca de 70" do 08 estava errado). Cada fatia segue o
[glossário](../contribuindo/09-glossario-de-identificadores.md), usa
`scripts/rename_identifiers.py` (troca só em código, nunca em comentário ou
string), passa build, testes e gate, encolhe a linha de base e é um commit:

```text
V-1  C++ de ui/src e ui/tests                 (~880)   a unidade cli_args já foi
V-2  scripts Python e shell dos gates         (~2.400)
V-3  QML de ui/qml e scripts/qml-harness      (~4.600)
V-4  Rust kinein-core, um domínio por commit  (~10.000)
V-5  protocolo: `automatico` (campo serializado: mudança de contrato, com versão)
```

Ao fim, a linha de base fica vazia e a catraca vira proibição pura.

## 0. Como ler

- §1 resultado; §2 invariantes; §3 mapa de donos e regra anti-duplicação.
- §4 o modelo: área, estado de área e layout persistido.
- §5 os fluxos, inclusive **abrir pelo terminal sem ruído** (§5.1) e **zero
  aviso em execução** (§5.2), que o autor pediu junto com esta versão.
- §6 contratos; §7 composição da UI; §8 desempenho; §9 estados e erros;
  §10 provas; §11 ordem; §12 rollback; §13 decisões abertas.
- Nomes marcados *(proposto)* não existem hoje. Os demais foram conferidos no
  checkout em 2026-10-01.

## 1. Resultado ao fim da 0.3.6

```text
R1 SILÊNCIO    `kinein .` devolve o prompt na hora, como `code .`, e a IDE não
               produz mensagem do Qt/QML (a causa de cada uma foi corrigida, §0.1)
R2 LIMPEZA     zero aviso do motor QML em qualquer superfície, no checkout E no
               AppImage (Qt 6.4), provado por um passeio automático por todas
R3 ORIENTAÇÃO  trilho curto (áreas, não ferramentas), painel inferior que mostra
               o que está em uso, header e status com o contexto efetivo
R4 TECLADO     toda área alcançável por teclado e paleta; Esc e "voltar ao
               editor" previsíveis; foco sempre visível
R5 MEMÓRIA     o layout volta como estava, por workspace; presets Codificação,
               Depuração, Revisão e Foco; o modo Foco restaura o anterior
R6 FLUIDEZ     ao menos um custo medido na F0 melhora de forma repetível, e
               nenhum orçamento existente regride
R7 PREPARO     a casca já comporta a área de superfície completa (Library da
               0.5) e os estados de área disponível/detectada/habilitada/
               contextual/fixada (49 §9), sem a 0.5 refazer o trilho
```

## 2. Invariantes

1. **Um shell só.** `Main.qml`, `ShellWorkspaceHost`, `ShellHeaderHost`,
   `ShellStatusHost`, `SideRail`, `BottomPanelHost` e os controllers atuais
   são o ponto de partida (49 §3). Nada de segundo shell, dock graph
   arbitrário ou store universal.
2. **Estado de apresentação mora no `ShellController`; estado persistido mora
   no domínio `settings` do core** (`settings.get/set`, escopo `global` e
   `workspace`). QML não grava arquivo de layout.
3. **Uma ação, um id:** `CommandDispatcher` e `command.list`. Botão, menu,
   atalho e paleta chamam o mesmo id. O gate de atalhos
   (`scripts/verificar-atalhos.sh`) continua dono das colisões.
4. **Fato vem do core.** Disponibilidade de área por fato (há `.git`? há
   framework embarcado? há alvo remoto?) é lida de quem já sabe
   (`project.model`, `git.status`, `remote.status`...), nunca recalculada em QML.
5. **O editor é a superfície padrão.** Toda área que toma o centro devolve o
   centro ao editor por `Esc` e por comando, sem perder abas nem foco.
6. **Compatibilidade com o Qt do pacote.** O AppImage usa Qt 6.4 (Debian 12).
   Nenhuma fatia usa recurso de QML que o 6.4 não tenha, e a prova roda nas
   duas versões (lição do [40 §7.148](40-estado-e-continuidade.md)).
7. **Sem regressão de orçamento:** primeiro frame, tecla→frame e RSS medidos
   antes e depois, na mesma máquina e binário equivalente
   (`scripts/medir-performance.sh`).
8. **Nada de telemetria.** Medidas são locais e explícitas.

## 3. Mapa de donos e a regra anti-duplicação

### 3.1 A regra

> Antes de criar componente, propriedade, controller, setting ou comando,
> **nomeie o dono atual**. Se existe, estenda. Componente novo só para uma
> pergunta nova, com o motivo escrito no commit.

### 3.2 O mapa

| Pergunta | Dono hoje | O que a 0.3.6 acrescenta | Proibido criar |
| --- | --- | --- | --- |
| Que áreas existem, com que ícone, ordem e dono? | `shell/ToolWindows.qml` (`entries`, `activate(id)`, `overlayEntries`) | Campos de área (§4.1): `kind`, `defaultPolicy`, `commandId`, `factKey` | Segunda lista de áreas, `switch` por id fora de `activate` |
| Como o trilho desenha? | `shell/SideRail.qml` | Renderiza só a projeção visível (§4.3) e o "Mais" | Lógica de visibilidade no trilho |
| Estado de layout em memória | `shell/ShellController.qml` (larguras, `showBottomPanel`, `bottomTab`, `leftWindow`, `railExpanded`, `outlineCollapsed`, `applyAutomaticLayout`) | Pins/ocultas, preset ativo, snapshot do modo Foco | Outro controller de layout, `property` de largura em host |
| Persistência do layout | core `settings.rs` + protocolo `SettingsValues`/`EffectiveSettings` (`explorer_width`, `bottom_panel_height`, `outline_*`, `rail_expanded`) · `settings/SettingsController.qml` | Campo `layout` versionado (§4.4) no escopo workspace | Arquivo de layout escrito por QML, `localStorage`, segundo store |
| Composição das regiões | `shell/ShellWorkspaceHost.qml`, `ShellEditorHost.qml`, `ShellLeftWindowHost`, `ShellEnvironmentOverlays.qml` | Host de **superfície central** (§5.6) | Segundo host de painel por simetria (`DockHost` universal, 49 F2) |
| Abas de baixo | `panels/bottom/BottomPanelHost.qml`, `shell/BottomTabBar.qml` (terminal, build, problems, tests, jobs, debug, search, tools, logs) | Regra de visibilidade contextual (§5.5) e ordem/pin | Segunda barra de abas, aba que duplica painel de área |
| Header | `shell/ShellHeaderHost.qml`, `TopHeaderBar.qml`, `HeaderRunWidget.qml`, `RunConfigMenu.qml` | Chips de contexto efetivo e orçamento de largura (§5.7) | Valor editável no chip; fato recalculado no header |
| Status | `shell/WorkspaceStatusBar.qml`, `workspace/LspStatusController.qml`, `ProjectHealthController.qml` | Itens com prioridade por largura (§5.7) | Inventário de ferramentas na barra |
| Ações e atalhos | `command/CommandDispatcher.qml`, `command.list`, `shell/GlobalShortcuts.qml`, `verificar-atalhos.sh` | Ids de vista (§6.2) | Atalho fora do gate, ação sem id |
| Busca global | `command/SearchEverywhereDialog.qml`, aba `search` | Decisão medida sobre Busca no trilho (§5.4) | Segunda busca |
| Tela inicial | `workspace/StartScreen.qml`, `RecentWorkspacesController.qml` | Só inventário na F0 (a Welcome é da 0.5, 50 §8) | Welcome nova nesta versão |
| Visual e componentes | `Theme.qml`, `components/` (`KvPanelFrame`, `KvIconButton`, `KvToggleChip`...), `sistema-visual-e-icones.md` | Densidade e foco visível nos componentes base | Estilo local repetido por painel |
| Mensagens de diagnóstico do app | `ui/src/core_client_log.cpp` (`errorLogFile()` → `~/.cache/kinein-vectis/logs/kinein-ui-erros.txt`), `log_redaction.*`, aba `logs` (IDE) | O *message handler* do Qt passa a escrever ali (§5.1) | Segundo arquivo de log, `qDebug` solto |
| Linha de comando | `scripts/kinein.in`, `ui/src/cli_args.*` (com teste C++), `single_instance*` | Desacoplar do terminal (§5.1) | Validação de caminho no script (o binário é o dono) |
| Comandos na abertura | `ui/src/core_client.cpp` `startupCommands()`, `app/StartupCommands.qml` | O passeio por superfícies do smoke (§5.2) | Outro mecanismo de automação |
| Medidas | `scripts/medir-performance.sh`, marcadores `KINEIN_PERF*` | Abrir painel, voltar ao editor (§5.3) | Medidor paralelo |

### 3.3 Checklist por fatia

```text
[ ] dono nomeado para cada pergunta (linha da §3.2); `rg` citado
[ ] nenhum fato recalculado em QML; nenhum valor persistido fora de settings
[ ] comando com id + atalho no gate + entrada na paleta
[ ] harness QML do controller (estado, intenção, resposta velha, null)
[ ] passeio por superfícies sem aviso (§5.2) no checkout e no AppImage
[ ] screenshots 1024×700, 1366×768 e largo (§10)
[ ] medida antes/depois (§8)
[ ] registro em 40 §7; contrato em 03-ipc-protocol.md quando settings mudar
```

### 3.4 Sinais de alerta na revisão

Um `property bool xVisible` novo num host; um `if (id === "...")` fora do
`ToolWindows.activate`; um painel que repete Problems, Jobs ou Environment; um
chip de header com estado próprio; um `Timer` para "esperar o layout"; um
`console.log` deixado; qualquer saída nova em stderr.

## 4. O modelo

### 4.1 Área (extensão da entrada do `ToolWindows`)

Hoje cada entrada tem `id, label, icon, tooltip, area ("left"), order,
available, active` e, quando é painel de ambiente, `panel`. A 0.3.6 acrescenta:

```text
kind           dock-left | dock-right | bottom | surface | overlay   (proposto)
               — onde a área abre. Hoje: explorer/git = dock-left; símbolos =
                 dock-right; ambiente = overlay; tools = bottom. surface nasce
                 vazio (Library 0.5) mas o host existe (§5.6)
defaultPolicy  pinned | contextual | hidden                          (proposto)
               — o padrão de fábrica (49 F1): Projeto, Ambiente pinned;
                 Git, Embarcados, Remote, Banco, Containers, Observabilidade
                 contextual; Busca decidida por medida (§5.4)
factKey        nome do fato do core que torna a área relevante       (proposto)
               (git.repo, project.embedded, remote.mirror, datasource.any...)
commandId      id do comando que abre/foca a área                   (proposto)
shortcut       o atalho que já existe, sem mudar (49 F1: manter ids e atalhos)
```

### 4.2 Estados de área (49 §9.1)

```text
disponível   a IDE tem a área (sempre verdade para as entradas da lista)
detectada    o fato do core existe (factKey verdadeiro)
habilitada   a área funciona neste workspace (na 0.3.6 = detectada ou sem fato;
             habilitar explicitamente é da 0.5)
contextual   aparece no trilho porque detectada e a política permite
fixada       o usuário fixou (aparece sempre)
oculta       o usuário ocultou neste workspace (some do trilho, continua no
             "Mais" e na paleta)
```

Regras: **habilitar não fixa; fixar não instala; ocultar não desabilita.** O
estado de UI (fixada/oculta/ordem) é do usuário e vai para `settings`; o estado
de fato (detectada) vem do core e nunca é persistido pela UI.

### 4.3 Projeção do trilho (função pura, testável)

```text
entradaVisível(e) =
    estado.ocultas ∌ e.id
  ∧ ( estado.fixadas ∋ e.id
    ∨ (e.defaultPolicy = pinned ∧ estado.desfixadas ∌ e.id)
    ∨ (e.defaultPolicy = contextual ∧ fato(e.factKey)) )

trilho = ordenar(visíveis, por ordemDoUsuário ou e.order); máximo 7 (49 §9.3);
excedentes e não visíveis → menu "Mais" (com estado e atalho de cada uma)
```

Mora no `ToolWindows` como função (ex.: `visibleEntries(state, facts)`
*(proposto)*); o `SideRail` só desenha o resultado. O harness testa a função
sem janela.

### 4.4 Layout persistido (extensão de `SettingsValues`)

```text
layout (proposto, escopo workspace; global guarda só preferência do usuário)
  schemaVersion   1
  leftWindow      "explorer" | "git" | ...
  widths          { explorer, context, outline, bottom }   (os que já existem
                                                             migram para cá)
  bottom          { visible, tab, order[], pinned[] }
  rail            { expanded, pinned[], unpinned[], hidden[], order[] }
  preset          "coding" | "debugging" | "review" | "focus" | ""
  focusRestore    snapshot do layout antes do modo Foco (ou vazio)
```

Regras: campos atuais (`explorer_width`, `bottom_panel_height`,
`outline_width`, `outline_collapsed`, `rail_expanded`) continuam lidos por um
ciclo como fallback; `layout` desconhecido ou de schema maior abre o padrão
seguro, sem apagar o arquivo; nunca se restaura processo de terminal (49 F2);
valores fora de faixa passam pelo `clamp` que o `ShellController` já faz.

## 5. Fluxos

### 5.1 Abrir pelo terminal sem ruído (R1)

**Problema medido em 2026-10-01:** `kinein` faz `exec` do binário em primeiro
plano (`scripts/kinein.in`). O terminal fica preso, e todo stderr do Qt
aparece nele (`wayland-egl`, avisos QML). O comportamento de referência é o
`code .`: devolve o prompt e não imprime nada em caso de sucesso.

```text
dono     cli_args (contrato) · single_instance (encaminhar) ·
         core_client_log (destino das mensagens) · kinein.in (só lança)

1  binário  cli_args lê os argumentos ANTES de qualquer janela (já é assim).
            --help, --version e erro de caminho → imprime no terminal e sai
            com o código certo (é a única saída permitida no terminal)
2  binário  single_instance: há janela para essa pasta → encaminha, sai 0
            (já existe)
3  binário  caso novo e sem --wait/--verbose: DESACOPLA (proposto)
            — relança a si mesmo detached (QProcess::startDetached com os
              mesmos argumentos e KINEIN_DETACHED=1), stdin/stdout/stderr
              ligados a /dev/null; o processo do terminal sai 0 assim que o
              filho foi criado
            — alternativa medida na fatia: fork + setsid no próprio processo;
              escolher a que não duplica o parse de argumentos
4  binário  rede de segurança (§0.1), não correção: qInstallMessageHandler
            (proposto) grava o que escapar no log de diagnóstico (mesma redação
            do log_redaction); nunca na interface; nunca em stderr, salvo com
            --verbose ou KINEIN_LOG=stderr. O gate (§5.2) garante que, em uso
            normal, ele não recebe nada
5  binário  `wayland-egl` no modo portátil é CAUSA a corrigir, não aviso
            esperado: o AppImage pede software (QT_QUICK_BACKEND=software) mas
            o plugin Wayland ainda tenta a integração EGL. A fatia mede e
            escolhe a forma correta de não carregá-la quando o renderer é
            software (ex.: QT_WAYLAND_CLIENT_BUFFER_INTEGRATION no hook
            portátil, ou não empacotar a integração quando ela não é usada),
            provando que a IDE desenha igual e o aviso deixa de ser produzido;
            o trecho do tutorial (entregue com a 0.3.5) que o chama de esperado é
            reescrito junto com a correção
6  script   kinein.in continua só resolvendo o binário e repassando argumentos
flags    --wait     (proposto) fica preso até a janela fechar, como `code -w`
         --verbose  (proposto) mantém stderr no terminal para diagnóstico
falha    o filho não sobe → o processo do terminal espera até 2 s pelo sinal
         de vida (socket do single_instance ou marcador) e, se falhar, imprime
         UMA linha com o caminho do log e sai com código ≠ 0
prova    teste C++ do cli_args (flags novas); teste de integração em Xvfb:
         `kinein /tmp/x` retorna em < 300 ms com stdout/stderr vazios e a
         janela abre; `--verbose` mostra; caminho inválido mostra o erro
```

### 5.2 Zero aviso em execução (R2)

**Medido em 2026-10-01** no AppImage instalado, numa sessão do autor:
centenas de `QQmlComponent: Component is not ready` e `GitWindow.qml:50:13:
Binding loop detected for property "width"`. Nenhum dos dois apareceu ao abrir
o pacote no projeto da IDE e no projeto do site, nem com o Git e o menu de
toolchain abertos. O gatilho depende da interação e ainda não está reproduzido.

```text
dono     a lista scripts/avisos-qml.txt (já usada pelo verificar_binario_abre.py
         e pelo testar-appimage.sh) + o passeio por superfícies (proposto)

1  lista    acrescentar "Component is not ready" (e "QQmlComponent:" como
            prefixo) à lista única
2  passeio  KINEIN_STARTUP_COMMANDS já roda comandos por id na abertura
            (StartupCommands.qml). O smoke ganha um ROTEIRO (proposto,
            scripts/surface-tour.txt) que abre e fecha cada área, cada
            aba de baixo, cada overlay, a paleta, a busca, o menu e o Git,
            redimensiona para 1024×700, e sai
3  onde     o mesmo roteiro roda: (a) no checkout (Qt do sistema), no
            verificar-binario-abre; (b) no AppImage (Qt 6.4), no
            testar-appimage.sh; (c) no Debian mínimo
4  reprova  qualquer linha da lista no log reprova, com a linha e o comando do
            roteiro em que apareceu (o handler da §5.1 carimba o id do comando
            corrente no log)
5  achar    "Component is not ready": com o carimbo do item 4, o primeiro
            comando que o produz é o dono; a correção vai à CAUSA (§0.1),
            seguindo a regra da §3 (provável Loader/Repeater cujo delegate não
            compila no Qt 6.4). Esconder a mensagem não fecha a fatia
6  GitWindow `width: visible ? implicitWidth : -parent.spacing` numa Row
            realimenta a largura no Qt 6.4: medir e trocar por um desenho
            sem largura negativa (a fatia mostra o antes/depois nas duas)
prova    mutação: um Loader com componente quebrado no roteiro reprova
```

#### 5.2.1 Causas encontradas em 2026-10-01 (antes de qualquer correção)

O passeio (`@passo=<ms>` no `KINEIN_STARTUP_COMMANDS`, cada passo marcado no
stderr) rodou no checkout (Qt 6.10) e no AppImage (Qt 6.4), em X11 e Wayland.
O fluxo do autor (editar e **salvar** o `.gitignore` do site) foi refeito com
mouse e teclado reais. A pilha de cada mensagem (`QT_MESSAGE_PATTERN` com
`%{backtrace}`) e casos mínimos rodados no Qt 6.4 do builder deram as causas:

| Mensagem | Causa provada | Correção (na causa) |
| --- | --- | --- |
| `QQmlComponent: Component is not ready` (centenas) | No Qt 6.4, um `section.delegate` declarado num arquivo com `pragma ComponentBehavior: Bound` **nunca é criado**; cada cabeçalho de seção vira essa mensagem. Na prática, os cabeçalhos de pasta da lista do Git não apareciam no AppImage. Caso mínimo: com o pragma falha (com ou sem `required property section`, e também como tipo de arquivo próprio); sem o pragma, funciona. O mesmo vale, também provado, para `header`, `footer` e `highlight` do `ListView` e para `Loader.sourceComponent` (inline ou por id do próprio arquivo); só `delegate:` funciona. Ocorrências no código: as seções de `git/GitChangesList.qml` e `editor/SymbolResultsList.qml`, o `header` desta e o `Loader` de depuração de `panels/bottom/TerminalViewport.qml` | O `Component` da seção nasce num arquivo **sem** o pragma (um `QtObject` com `property Component`), e o delegate é um tipo próprio que acha a lista por `ListView.view`, tipado como `var` (tipar com a própria lista cria ciclo de tipos que trava o carregador do 6.4). Provado no 6.4 (seções criadas, sem aviso) e no `qmllint -W 0` das duas versões. Gate `scripts/verificar-qml-qt64.sh`: num arquivo com o pragma, essas chaves só aceitam caminho de membro (`root.parts.section`); provado por mutação contra o HEAD, que reprova nas quatro ocorrências |
| `GitWindow.qml:50:13: Binding loop ... "width"` | `width: visible ? implicitWidth : -parent.spacing` num `Text` dentro de `Row`: largura amarrada ao próprio `implicitWidth` (aparece também no Qt 6.10, no passo `git.log`) | Sem largura explícita (a `Row` já pula filho invisível) e o espaçador conta o título só quando visível. Passeio do checkout: 24 passos, zero aviso |
| `qt.qpa.wayland: Failed to load client buffer integration: "wayland-egl"` | Os grupos de plugins Wayland são copiados à mão pelo empacotador **sem `rpath`**; `libqt-plugin-wayland-egl.so` não acha `libQt6WaylandEglClientHwIntegration.so.6`, que está em `usr/lib` do pacote. A checagem de dependências rodava o `ldd` com `LD_LIBRARY_PATH` apontando para `usr/lib`, um ambiente que o AppImage não tem em execução, e escondeu o defeito | Gravar `rpath $ORIGIN/../../lib` nos plugins copiados (o mesmo que o linuxdeploy grava nos dele) e validar **sem** `LD_LIBRARY_PATH`, como o carregador vê |
| `xkbcommon: ERROR: .../Compose: unrecognized keysym "dead_hamza"` | O pacote leva a `libxkbcommon` do Debian 12, que lê os arquivos `Compose` **mais novos** do sistema e não conhece símbolos recentes | A medir na fatia: dados de compose coerentes com a biblioteca empacotada (ex.: `XLOCALEDIR` para a cópia do Debian 12 no pacote), ou usar a biblioteca do sistema; o critério é a mensagem deixar de ser produzida sem perder composição de acentos |

**Prevenção, para não depender de achar no uso:** o Qt do AppImage passa a
rodar os harnesses QML e o passeio no gate (o builder ganha o executor `qml`
do Qt 6.4), e a lista `scripts/avisos-qml.txt` ganha `Component is not ready`,
`xkbcommon: ERROR` e `Failed to load client buffer integration`.

### 5.3 F0 — inventário e linha de base

```text
dono     este documento (tabela) + medir-performance.sh + uso-diario.md
1  listar cada elemento visível: trilho, header, status, abas de baixo,
   overlays, menus, StartScreen, diálogos
2  para cada um, a tabela do 49 F0: superfície e dono · frequência e propósito ·
   duplicação · decisão (manter/unir/mover/contextual/remover) · alcance
   (mouse, teclado, paleta, volta ao editor)
3  screenshots em Xvfb (larguras exatas, sem notificações): 1024×700,
   1366×768, 1920×1080; e na tela real do autor para conferência
4  medir: primeiro frame, RSS inicial, tecla→frame (mediana/p95/pior), abrir
   cada painel, voltar ao editor; mesma máquina e binário
5  cruzar com o diário de uso: cada "atrito" vira linha com contagem de gestos
saída    registro no 40 §7 + a tabela anexada aqui (§F0 abaixo, a preencher)
```

### 5.4 F1 — trilho por áreas

```text
dono     ToolWindows (dados e projeção §4.3) · SideRail (desenho) ·
         ShellController (estado) · settings (persistência)
1  core   fatos de área já disponíveis (git.status, project.model,
          remote.status, datasource/container listas) → QML recebe por eventos
          existentes
2  QML    ToolWindows.visibleEntries(estado, fatos) → SideRail
3  QML    "Mais" lista todas as não visíveis com estado e atalho; dali fixar,
          ocultar, restaurar padrão
4  QML    menu de contexto do ícone: Abrir · Fixar/Desafixar · Ocultar neste
          workspace · Restaurar trilho padrão
5  core   settings.set(workspace, layout.rail) com debounce (o
          persistLayoutSoon que já existe)
Busca    decisão por medida (49 F1): contar gestos de "achar arquivo/símbolo"
         com e sem a entrada, com o atalho e o header; a entrada só fica se
         reduzir gestos sem repetir o botão do header
falha    fato indisponível (core ainda carregando) → entrada contextual não
         some e volta: aparece quando o fato chega, sem piscar (estado
         "desconhecido" = mantém a última projeção)
```

### 5.5 F2 — painéis e painel inferior contextuais

```text
dono     BottomPanelHost/BottomTabBar (abas) · ShellController (estado) ·
         settings (ordem/pin)
regra    sempre visíveis: Terminal e Problemas
         contextuais: Build (há build em curso ou terminado nesta sessão),
         Testes (há descoberta ou execução), Debug (sessão de debug),
         Jobs (há job vivo ou falho não visto), Busca (há resultado), IDE/logs
         (há aviso não lido)
         fixadas pelo usuário: sempre
         toda aba alcançável pelo comando/paleta mesmo escondida
Tools    auditar o que a aba e a entrada "Ferramentas" fazem; se coberto por
         Ambiente/Setup, migrar gestos e atalhos ANTES de remover (49 F2)
presets  Codificação, Depuração, Revisão, Foco = conjuntos de layout (§4.4)
         aplicados sobre os mesmos hosts; trocar guarda o anterior em
         focusRestore/um "voltar" (proposto); layout desconhecido = padrão
falha    aba removida por migração com pin salvo → ignora o id e registra no
         log (não quebra a barra)
```

### 5.6 Host de superfície central (R7)

```text
dono     ShellWorkspaceHost (composição) · ShellController (área central ativa)
1  estado centralArea = "editor" | <id de área kind=surface>   (proposto)
2  abrir  área surface troca o centro; o editor fica montado e oculto (não
          destrói abas nem perde texto/cursor)
3  voltar Esc (quando o foco está na superfície), comando view.returnToEditor
          (proposto) e o clique na aba do editor devolvem o centro
4  0.3.6  nenhuma área nova é surface ainda; o host nasce com um harness e uma
          área de teste. A Library da 0.5 é o primeiro uso real (50 §8)
falha    área surface sem conteúdo → volta ao editor e registra no log
```

### 5.7 F3 — header e status com contexto efetivo

```text
dono     header: ShellHeaderHost/TopHeaderBar/HeaderRunWidget/RunConfigMenu;
         status: WorkspaceStatusBar; fatos: project.model, toolchain.get,
         runConfig.list, remote.status, index.status
chips    projeto · perfil/kit · alvo · toolchain efetiva · configuração de
         execução · dispositivo · remoto · operação em curso — só os
         PERTINENTES ao projeto
largura  orçamento por prioridade: abaixo de N px some primeiro o de menor
         prioridade, que vai para um "…" com a lista; o editor nunca encolhe
         para caber chip (49 F4)
clique   abre o DONO da configuração (ToolchainController, RunConfigMenu,
         Remote...), nunca um editor no chip
estados  configurado / detectado / selecionado / efetivo quando o core
         distinguir; dado sem origem não parece medida atual (49 F3)
```

### 5.8 F4 — foco, teclado, densidade e modo Foco

```text
foco     grafo explícito: trilho → dock esquerdo → editor → dock direito →
         painel de baixo → header → status; Ctrl+F6 (proposto, gate) percorre;
         Esc fecha overlay/popup e devolve ao editor; "voltar ao editor"
         sempre disponível
visível  todo componente focável mostra foco (anel do Theme); nada depende só
         de cor; texto alternativo em ícones
estreito abaixo de 1024 px: recolhe dock direito, compacta header, esconde
         rótulos do status — nessa ordem — antes de reduzir o editor
movimento animações respeitam preferência de movimento reduzido (Theme)
Foco     modo Foco recolhe esquerda/direita/baixo e compacta header/status;
         guarda focusRestore; sair restaura EXATAMENTE o anterior (49 F4);
         atalho e comando descobríveis
carga    medir antes: hoje os overlays de ambiente são criados na abertura
         (ShellEnvironmentOverlays, createObject uma vez). Criar no primeiro
         uso só se a medida mostrar custo; preservar o estado que precisa
         voltar (token do Grafana tem política própria)
```

### 5.9 F5 — prova da versão

Comparação antes/depois na mesma máquina; tarefa real cronometrada e com
contagem de gestos: abrir projeto, achar arquivo, alternar painel, compilar,
entender um problema, voltar ao editor. Critério de ganho: repetível acima da
variação das amostras no custo escolhido na F0; nenhum outro orçamento piora.

## 6. Contratos

### 6.1 IPC e settings

| Contrato | Tipo | Dono | Nota |
| --- | --- | --- | --- |
| `SettingsValues.layout` / `EffectiveSettings.layout` | campo novo | core `settings.rs` + protocolo | §4.4; sobe `PROTOCOL_VERSION`; escopo workspace |
| Campos antigos de largura | mantidos 1 ciclo | idem | fallback e migração |
| Fatos de área | **nenhum método novo** | domínios existentes | a fatia lista de qual evento cada `factKey` lê |

### 6.2 Comandos *(propostos)*

`view.returnToEditor`, `view.focusMode`, `view.preset.coding`,
`view.preset.debugging`, `view.preset.review`, `view.preset.focus`,
`view.preset.restorePrevious`, `rail.pin`, `rail.unpin`, `rail.hide`,
`rail.reset`, `bottom.pin`, `view.cycleFocus`. Todos pelo `CommandDispatcher`,
com atalho validado no gate.

### 6.3 Linha de comando *(proposta)*

`kinein [--wait] [--verbose] [pasta]` · `KINEIN_LOG=stderr|debug` ·
`KINEIN_DETACHED=1` (interno, marca o filho). O contrato e os testes ficam em
`cli_args`.

## 7. Composição

```text
┌ header: projeto · kit/alvo · run · (chips por prioridade) ───────────────┐
├─trilho─┬─dock esquerdo──┬────────── centro ────────────┬─dock direito─────┤
│ áreas  │ explorer | git │ editor  (ou superfície §5.6) │ símbolos         │
│ + Mais │                │                              │                  │
├────────┴────────────────┴── painel de baixo (abas §5.5) ┴─────────────────┤
└ status: posição · saúde · trabalho em curso (prioridade por largura) ─────┘
overlays de ambiente: sobre o centro (como hoje), até a 0.4/0.5 decidir
```

## 8. Desempenho

- Linha de base na F0 e régua do [21](21-long-horizon-roadmap.md):
  primeiro frame, RSS, tecla→frame, abrir painel, voltar ao editor.
- Projeção do trilho e da barra de baixo: funções puras sobre listas curtas,
  sem binding que leia `ListModel` (regra do `EditorOpenDocuments`).
- Persistência com debounce (o `layoutSaveTimer` que já existe).
- Overlays sob demanda só com medida (§5.8).

## 9. Estados e erros

| Situação | Comportamento |
| --- | --- |
| `layout` corrompido ou de schema maior | abre o padrão seguro; o arquivo fica; aviso no log |
| Fato de área ainda desconhecido | mantém a última projeção; não pisca |
| Área oculta pedida por atalho | abre normalmente; continua oculta no trilho |
| Janela muito estreita | regra de recolhimento da §5.8, nesta ordem |
| Filho do `kinein` não sobe | uma linha com o caminho do log e código ≠ 0 |
| Aviso do motor QML | reprova o passeio no gate; em produção vai para o log e para a aba IDE |

## 10. Provas

```text
unidade    projeção do trilho e da barra de baixo (funções puras), migração
           do layout, cli_args (flags novas)
harness    ShellController, ToolWindows, SideRail, BottomTabBar, presets e
           modo Foco (entrar/sair restaura exato), superfície central
passeio    §5.2 no checkout, no AppImage (Qt 6.4) e no Debian mínimo
telas      Xvfb a 1024×700, 1366×768, 1920×1080 (sem notificações, reprodutível);
           conferência na tela real do autor
teclado    roteiro: percorrer o grafo de foco, Esc, voltar ao editor, abrir
           cada área pelo atalho
terminal   `kinein` retorna rápido e mudo; `--verbose` fala; erro de caminho fala
medida     medir-performance.sh antes/depois, mesma máquina
mutação    cada guarda nova provada removendo-a
```

## 11. Ordem e trem de versões (decisão do autor, 2026-10-01)

A série 0.3.0–0.3.5 está encerrada e divulgada. A reorganização da casca vai
da **0.3.6 à 0.3.9**; cada versão é publicável sozinha, com notas de
atualização no site, e só fecha com o gate completo, o passeio sem aviso no
AppImage e as telas nas três larguras.

```text
0.3.6  LIMPEZA E BASE
       §5.1 terminal mudo (desacoplar como `code .`, --wait/--verbose)
       §5.2 passeio por superfícies + causa de cada aviso corrigida
            (Component is not ready, GitWindow, wayland-egl)
       §5.3 F0: inventário, telas em Xvfb, linha de base medida
       §4.4 layout versionado no settings (contrato, migração)
0.3.7  NAVEGAÇÃO
       §5.4 F1 trilho por áreas (projeção, Mais, fixar/ocultar)
       §5.5 F2 painel de baixo contextual, Tools migrado, presets
0.3.8  CENTRO E CONTEXTO
       §5.6 host de superfície central (com volta ao editor)
       §5.7 F3 header e status com contexto efetivo
0.3.9  TECLADO, FLUIDEZ E PROVA
       §5.8 F4 foco, teclado, densidade, modo Foco, carga sob demanda medida
       §5.9 F5 prova antes/depois e fechamento da série
```

## 12. Rollback

- Message handler atrás de `KINEIN_LOG=stderr` (volta ao comportamento antigo).
- Desacoplamento: `--wait` é o comportamento antigo, e um setting global
  *(proposto)* pode fixá-lo.
- `layout` novo ignorável: apagar o campo volta aos campos antigos.
- Projeção do trilho com política "todas fixadas" reproduz o trilho de hoje.

## 13. Decisões do autor

1. Busca no trilho: decidir pela medida da §5.4 (proposta: fora, se o atalho e
   o header cobrirem com menos gestos).
2. Abas sempre visíveis: Terminal e Problemas (proposta) — confirmar.
3. Overlays de ambiente continuam overlays na 0.3.6 (proposta) e viram área
   de dock ou superfície na 0.4/0.5, por fluxo real?
4. `--wait` como padrão em algum caso (ex.: `git config core.editor`)?
5. Atalho para percorrer o foco entre regiões (proposta: Ctrl+F6, como em
   IDEs JetBrains) — confirmar no gate de atalhos.

## F0 — inventário (a preencher na fatia 3)

| Elemento | Host/dono | Frequência/propósito | Duplicação | Decisão | Alcance |
| --- | --- | --- | --- | --- | --- |
| | | | | | |
