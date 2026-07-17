import QtQuick

// Assistente: o conceito INTEIRO mora aqui — o que e um agente, qual foi
// escolhido, o que e digitado, quando a sessao abre, como a aba se chama e
// quando o icone do rail acende.
//
// O RuntimeController mantem sessoes de terminal e NAO sabe que "Assistente"
// existe: ele so carimba na aba o rotulo e o `kind` que lhe pedem, e devolve o
// `kind` em `terminalOpened`. Essa e a fronteira — manter sessao de terminal
// nao e saber o que uma delas significa.
//
// A LINHA, e ela e fina (§0.2f):
//
//   DETECTAR  claude/codex existem no PATH?  -> CAPACIDADE. Vive no core, e o
//             core ja faz: sao entradas do KNOWN_TOOLS como cargo ou clangd,
//             sem nenhum ramo por programa.
//   EXECUTAR  como o agente e rodado?        -> POLITICA. Vive AQUI. Foi
//             politica por programa no core que o 0.59.0 removeu, e ela nao
//             volta.
//
// Saber que `claude` e `codex` sao agentes e politica, e por isso a lista mora
// nesta camada: o core nao conhece o conceito de "agente de IA". Ele so responde
// "existe no PATH? onde?".
//
// Este controller nao abre PTY, nao roda nada e nao renderiza. Escolhido o
// agente, o seletor some e o que roda e o terminal normal — mesmo `terminal.open`,
// mesmo renderer, mesma grade, mesmo cursor — e o comando entra pelo mesmo
// `terminal.input` de quando o usuario digita.
Item {
    id: root

    // Vem do `tools.detect` do core, via workspaceController.
    property var toolsList: []
    // O dono das sessoes de terminal. Este controller PEDE sessao; nunca abre.
    property var runtimeController: null
    property bool selectorVisible: false
    property alias agentsModel: agentsListModel
    // Comando a mandar quando a sessao abrir. Vem do `path` DETECTADO, nunca de
    // string literal — senao a politica so teria migrado de camada.
    property string pendingCommand: ""
    // Numeracao propria das abas de contexto. NUNCA decrementa, pela mesma razao
    // do `terminalSeq`: fechar a 2 de 3 faria a proxima nascer "Assistente 3".
    property int assistantSeq: 0

    // A tag que o RuntimeController carimba na aba por nossa conta. Opaca para
    // ele; o significado dela e este arquivo.
    readonly property string assistantKind: "assistant"

    // Quais ferramentas sao agentes. E o unico lugar do projeto que sabe disso,
    // e e UI de proposito.
    readonly property var agentIds: ["claude", "codex"]

    /// `true` quando a aba ATIVA do terminal e uma sessao de contexto.
    readonly property bool activeTerminalIsAssistant:
        root.runtimeController !== null
        && root.runtimeController.activeTerminalKind === root.assistantKind

    /// Estado aceso do icone no rail. Nao ha painel proprio de Assistente para
    /// alternar: o icone acende quando o painel do terminal esta na frente E a
    /// sessao ativa e a do Assistente. A regra e POLITICA do Assistente e mora
    /// aqui — montada num host visual, ela ficava espalhada por tres fontes.
    readonly property bool assistantSessionVisible:
        root.runtimeController !== null
        && root.runtimeController.terminalPanelVisible
        && root.activeTerminalIsAssistant

    visible: false

    ListModel {
        id: agentsListModel
    }

    // O RuntimeController avisa TODA sessao aberta; so as nossas interessam. A
    // escuta mora aqui, junto da dependencia declarada, e nao no composition
    // root: sinal escutado no lugar errado nao falha no build — para de
    // funcionar em silencio (PONTO_ATUAL §0.2j).
    Connections {
        target: root.runtimeController

        function onTerminalOpened(id, kind) {
            root.handleTerminalOpened(id, kind);
        }
    }

    // "Redetectar" so tem efeito visivel por aqui: a lista e montada em
    // `refresh()`, entao o `tools.detect` voltando com outro resultado precisa
    // remonta-la enquanto o seletor esta aberto.
    onToolsListChanged: {
        if (root.selectorVisible) {
            refresh();
        }
    }

    function refresh() {
        agentsListModel.clear();
        const tools = toolsList !== undefined && toolsList !== null ? toolsList : [];
        for (let i = 0; i < agentIds.length; i++) {
            const id = agentIds[i];
            let achado = null;
            for (let j = 0; j < tools.length; j++) {
                if (tools[j].id === id) {
                    achado = tools[j];
                    break;
                }
            }
            if (achado === null) {
                continue;
            }
            const disponivel = achado.status === "detected";
            agentsListModel.append({
                agentId: id,
                displayName: achado.displayName !== undefined ? achado.displayName : id,
                // Detectado: mostra ONDE esta. Ausente: mostra como instalar, e
                // o core nunca roda essa sugestao — e texto, nao acao.
                detail: disponivel
                        ? (achado.path !== undefined && achado.path !== null ? achado.path : id)
                        : (achado.suggestedInstall !== undefined
                           && achado.suggestedInstall !== null ? achado.suggestedInstall : ""),
                command: disponivel && achado.path !== undefined && achado.path !== null
                         ? achado.path : "",
                available: disponivel
            });
        }
    }

    /// O gesto do rail/menu.
    ///
    /// Se ja existe uma sessao de contexto viva, foca ela em vez de acumular
    /// abas — o ponto e melhorar o fluxo, nao multiplicar terminal. Sem sessao
    /// viva, a escolha do agente vem ANTES de abrir o PTY: o seletor so existe
    /// enquanto nao ha sessao.
    function openAssistant() {
        if (root.runtimeController === null
                || root.runtimeController.workspaceRoot === "") {
            return;
        }
        const existente = root.runtimeController.firstTerminalOfKind(root.assistantKind);
        if (existente !== "") {
            root.runtimeController.selectTerminal(existente);
            return;
        }
        refresh();
        selectorVisible = true;
    }

    function dismiss() {
        selectorVisible = false;
    }

    /// A qual aba do painel inferior a sessao ATIVA pertence.
    ///
    /// O RuntimeController so sabe pedir "terminal" — para ele nao existe aba de
    /// Assistente, e nao deve existir: ele nao conhece o conceito. Quem traduz e
    /// este arquivo, e a traducao e aplicada na composicao (AppDomains), onde o
    /// pedido dele vira `shellController.showTab`.
    function tabFor(tab) {
        if (tab !== "terminal" || root.runtimeController === null) {
            return tab;
        }
        // `pendingKind` cobre a janela entre pedir a sessao e o core devolve-la:
        // sem ele o painel piscaria na aba Terminal ate o PTY abrir.
        const nossa = root.activeTerminalIsAssistant
                      || root.runtimeController.pendingKind === root.assistantKind;
        return nossa ? "assistant" : tab;
    }

    /// O usuario clicou numa aba do painel inferior.
    ///
    /// Terminal e Assistente desenham a MESMA sessao ativa, cada um sob o seu
    /// rotulo — trocar de aba tem que trocar a sessao junto, senao uma mostra o
    /// conteudo da outra. Essa regra so existe porque a aba do Assistente
    /// existe, entao ela e nossa, e nao do host visual.
    ///
    /// Devolve `true` quando ja resolveu — quem seleciona a sessao ja pede a aba
    /// certa de volta pelo `tabFor`. `false` quando a aba nao tem nada a ver
    /// conosco e o shell deve trata-la como sempre.
    function handleBottomTabClick(tab) {
        if (root.runtimeController === null) {
            return false;
        }
        if (tab === "assistant") {
            openAssistant();
            return true;
        }
        if (tab !== "terminal" || !root.activeTerminalIsAssistant) {
            return false;
        }
        // Sair para o Terminal com o agente ativo: leva para uma sessao comum,
        // ou materializa uma. Sem isto a aba Terminal mostraria a grade do
        // agente sob o rotulo errado.
        const comum = root.runtimeController.firstTerminalOfKind("");
        if (comum !== "") {
            root.runtimeController.selectTerminal(comum);
        } else {
            root.runtimeController.newTerminal();
        }
        return true;
    }

    /// Escolhido um agente disponivel: guarda o comando, fecha o seletor e pede
    /// a sessao. Agente ausente nao e escolhivel — o seletor mostra a sugestao
    /// de instalacao como texto e nada roda.
    function choose(agentId) {
        for (let i = 0; i < agentsListModel.count; i++) {
            const item = agentsListModel.get(i);
            if (item.agentId !== agentId || !item.available || item.command === "") {
                continue;
            }
            // Caminho com espaco quebraria a linha de comando do shell.
            root.pendingCommand = item.command.indexOf(" ") >= 0
                    ? '"' + item.command + '"' : item.command;
            selectorVisible = false;
            openSession();
            return;
        }
    }

    /// Pede a sessao rotulada. O numero e PROVISORIO: `terminal.open` pode
    /// falhar, e `assistantSeq` so e commitado quando a aba existe de fato —
    /// senao uma falha queimaria um numero.
    function openSession() {
        if (root.runtimeController === null) {
            return;
        }
        root.runtimeController.openLabeledTerminal(
            qsTr("Assistente %1").arg(root.assistantSeq + 1), root.assistantKind);
    }

    /// A sessao abriu. So agora o numero e commitado e o comando e digitado,
    /// uma vez so — sem zerar, a proxima aba de contexto herdaria o comando da
    /// anterior.
    function handleTerminalOpened(id, kind) {
        if (kind !== root.assistantKind) {
            return;
        }
        root.assistantSeq += 1;
        const comando = root.pendingCommand;
        root.pendingCommand = "";
        if (comando !== "") {
            root.runtimeController.sendTerminalInput(id, comando + "\n");
        }
    }

    /// Workspace fechou ou o core caiu: o RuntimeController descarta as abas,
    /// entao a numeracao e a escolha pendente morrem junto.
    function clear() {
        agentsListModel.clear();
        selectorVisible = false;
        root.pendingCommand = "";
        root.assistantSeq = 0;
    }
}
