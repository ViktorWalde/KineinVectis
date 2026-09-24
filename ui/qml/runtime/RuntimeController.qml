import QtQuick

Item {
    id: root

    property string workspaceRoot: ""
    property bool running: false
    property bool terminalActive: false
    property bool terminalPanelVisible: false
    // D2 (DocsPublic/roadmaps/24): grid do terminal (cores/cursor/spans) vindo do core. Com a
    // D2.3 há VÁRIAS sessões: este é o grid da ABA ATIVA. Os demais ficam em
    // `terminalRenders` e voltam intactos ao trocar de aba.
    property var terminalRender: ({})
    // D2.3 (DocsPublic/roadmaps/24): multi-terminal. O core dá um `id` por sessão; a UI guarda
    // a lista (pras abas), o render de cada uma e quem está ativa.
    property alias terminalsModel: terminalsListModel
    property string activeTerminalId: ""
    property var terminalRenders: ({})
    // ListModel nao notifica mudanca de conteudo para funcoes; este contador
    // e o gatilho de reavaliacao dos bindings que dependem da lista.
    property int terminalsRevision: 0
    property string terminalText: ""
    // A EXECUCAO e' uma aba de terminal como as outras (2026-09-18, pedido do
    // autor: "ja' temos o terminal integrado"): o core abre a sessao e devolve
    // o id; a aba ganha o nome do comando. Aqui fica qual sessao e' a da
    // execucao, e o desfecho da ultima.
    property string runTerminalId: ""
    property string lastRunMessage: ""
    // A porta serial que o Executar leva ao core como `device` (run.start /
    // run.script, 0.110.0): num projeto MicroPython o arquivo roda na placa
    // por `mpremote connect <porta> run`. Vazio = campo ausente = o mpremote
    // escolhe. E' ESCOLHA feita no painel de Embarcados (EmbeddedController.
    // selectedPort) e chega aqui por binding na composicao; este controller
    // nao a guarda nem a valida — so' a repassa no gesto de executar.
    property string serialDevice: ""

    signal showTabRequested(string tab)
    signal terminalOpenRequested()
    signal terminalInputRequested(string id, string data)
    signal terminalResizeRequested(string id, int cols, int rows)
    signal terminalScrollRequested(string id, int offset)
    signal terminalClearScrollbackRequested(string id)
    signal terminalSelectAllRequested(string id, string selectionId)
    signal terminalCopySelectionRequested(string id, string selectionId)
    signal terminalSelectionCopied(string id, string selectionId, string text, bool valid)
    signal terminalSelectionFailed()
    signal terminalWheelRequested(string id, int col, int row, int lines,
                                  int modifiers)
    signal terminalCloseRequested(string id)
    signal runStartRequested(string command, string device)
    signal runScriptRequested(string path, string device)
    signal runStopRequested()
    signal focusTerminalInputRequested()
    signal clearTerminalInputRequested()

    visible: false

    // D2.3: uma linha por terminal aberto — { termId, title }.
    ListModel {
        id: terminalsListModel
    }

    function clearTerminals() {
        terminalsListModel.clear();
        root.terminalsRevision += 1;
        root.activeTerminalId = "";
        root.terminalRender = ({});
        root.terminalRenders = ({});
        root.runTerminalId = "";
        root.finishedRuns = ({});
    }

    function clear() {
        clearTerminals();
        terminalText = "";
        lastRunMessage = "";
    }

    // Um crash do core não produz `event.terminal.closed` para cada shell.
    // `CoreClient` zera terminalActive ao desconectar (e tambem quando a
    // ULTIMA sessao viva fecha); descarte aqui as abas que apontariam para
    // ids vivos — as de execucao ja' terminada ficam: o render e' da UI e
    // fecha-las e' local.
    onTerminalActiveChanged: {
        if (root.terminalActive) {
            return;
        }
        for (let i = terminalsListModel.count - 1; i >= 0; i--) {
            const id = terminalsListModel.get(i).termId;
            if (!isFinishedRun(id)) {
                removeTerminalTab(id);
            }
        }
    }

    function appendTerminalData(data) {
        terminalText += data;
        if (terminalText.length > 100000) {
            terminalText = terminalText.substring(terminalText.length - 80000);
        }
    }

    // ---- D2.3: multi-terminal --------------------------------------------

    /// O core criou a sessão: vira aba e assume o foco.
    ///
    /// Para o core toda sessão é um `$SHELL` no PTY, igual — o número da aba é
    /// decisão da UI e por isso não é parâmetro do protocolo.
    /// `title` e' opcional: quem abre um PROCESSO numa aba (logs de um
    /// container, um monitor serial) nomeia a aba pelo que roda nela; o shell
    /// usa o primeiro rotulo livre; identidade continua sendo o id do core.
    function nextTerminalTitle() {
        const used = ({});
        for (let i = 0; i < terminalsListModel.count; i++) {
            used[terminalsListModel.get(i).title] = true;
        }
        let number = 0;
        while (used[number === 0 ? "terminal" : "terminal" + number] === true) number++;
        return number === 0 ? "terminal" : "terminal" + number;
    }

    function handleTerminalOpened(id, shell, title) {
        root.terminalRenders[id] = ({});
        terminalsListModel.append({
            "termId": id,
            "title": title !== undefined && title !== "" ? title
                                                        : nextTerminalTitle()
        });
        root.terminalsRevision += 1;
        selectTerminal(id);
    }

    /// Troca a aba ativa. O grid da sessão volta INTACTO (o core mantém o
    /// emulador vivo de cada uma; a UI só guarda o último render).
    function selectTerminal(id) {
        root.activeTerminalId = id;
        root.terminalRender = root.terminalRenders[id] || ({});
        showTabRequested("terminal");
        focusTerminalInputRequested();
    }

    function newTerminal() {
        terminalOpenRequested();
    }

    function closeTerminal(id) {
        if (isFinishedRun(id)) {
            removeTerminalTab(id);
            return;
        }
        terminalCloseRequested(id);
    }

    function indexOfTerminal(id) {
        for (let i = 0; i < terminalsListModel.count; i++) {
            if (terminalsListModel.get(i).termId === id) {
                return i;
            }
        }
        return -1;
    }

    function openTerminalPanel() {
        if (workspaceRoot === "") {
            return;
        }
        // Sem nenhuma sessão viva, Alt+F12 tem que MATERIALIZAR um terminal.
        // UM dono para isso: o TerminalPanel nao abre sessao sozinho ao
        // aparecer — quando abria, um gesto criava DUAS (foto de 2026-09-18,
        // "Terminal 1" e "Terminal 2").
        showTabRequested("terminal");
        if (root.activeTerminalId === "") {
            terminalOpenRequested();
        }
        focusTerminalInputRequested();
    }

    function submitShellInput(text) {
        if (root.activeTerminalId === "") {
            terminalOpenRequested();
            return;
        }
        terminalInputRequested(root.activeTerminalId, text + "\n");
        clearTerminalInputRequested();
    }

    // A porta so' acompanha o LANCADOR PADRAO (command vazio): um comando
    // digitado roda como foi escrito, e o core recusa os dois juntos.
    function startRun(command) {
        if (workspaceRoot === "" || running) {
            return;
        }
        lastRunMessage = "";
        runStartRequested(command, command === "" ? serialDevice : "");
    }

    function startScript(path) {
        if (workspaceRoot === "" || running || path === "") {
            return;
        }
        lastRunMessage = "";
        runScriptRequested(path, serialDevice);
    }

    function stopRun() {
        if (running) {
            runStopRequested();
        }
    }

    // O nome da aba da execucao: "▶ " + o comando ate' caber (o caminho
    // longo de um script vira o nome do arquivo).
    function runTabTitle(command) {
        const partes = command.trim().split(/\s+/);
        let base = partes.length > 0 ? partes[0] : command;
        if (base.indexOf("/") >= 0) {
            base = base.substring(base.lastIndexOf("/") + 1).replace(/'$/, "");
        }
        const resto = partes.slice(1).join(" ");
        const titulo = resto === "" ? base : base + " " + resto;
        return "▶ " + (titulo.length > 28 ? titulo.substring(0, 27) + "…" : titulo);
    }

    /// Roteia o render pra sessão dona. Só a ABA ATIVA vira `terminalRender`
    /// (o que a tela desenha); as outras ficam guardadas e voltam na troca.
    function handleTerminalRender(render) {
        if (!render || render.id === undefined) {
            return;
        }
        // O reader do PTY ainda pode entregar o render final depois do
        // `closed`. IDs nunca são reutilizados, então um render sem aba dona
        // é atrasado e deve ser descartado.
        if (indexOfTerminal(render.id) < 0) {
            return;
        }
        root.terminalRenders[render.id] = render;
        if (render.id === root.activeTerminalId) {
            terminalRender = render;
        }
    }

    // D2: teclas cruas (char-a-char) do painel de shell → PTY. Se a sessão
    // não está ativa, uma tecla reabre o terminal.
    function sendTerminalKey(data) {
        if (root.activeTerminalId === "") {
            terminalOpenRequested();
            return;
        }
        terminalInputRequested(root.activeTerminalId, data);
    }

    function resizeTerminal(cols, rows) {
        if (root.activeTerminalId !== "") {
            terminalResizeRequested(root.activeTerminalId, cols, rows);
        }
    }

    function scrollTerminal(offset) {
        if (root.activeTerminalId !== "") {
            terminalScrollRequested(root.activeTerminalId, offset);
        }
    }

    function clearTerminalScrollback() {
        if (root.activeTerminalId !== "" && !isFinishedRun(root.activeTerminalId)) {
            terminalClearScrollbackRequested(root.activeTerminalId);
        }
    }

    // Repassa o gesto cru; nao interpreta. Ver TerminalScrollController.
    function wheelTerminal(col, row, lines, modifiers) {
        if (root.activeTerminalId !== "") {
            terminalWheelRequested(root.activeTerminalId, col, row, lines,
                                   modifiers);
        }
    }

    // As execucoes que ja' terminaram e cuja aba FICA (o autor le a saida):
    // id -> exitCode. Fechar a aba de uma delas e' so' local — o core nao
    // tem mais a sessao.
    property var finishedRuns: ({})

    function isFinishedRun(id) {
        return root.finishedRuns[id] !== undefined;
    }

    /// Uma sessão morreu (shell saiu ou fechamos a aba): tira da lista e, se
    /// era a ativa, cai pra vizinha. Sem sessão, a aba fica vazia e a próxima
    /// tecla reabre. A aba de uma EXECUCAO fica, com o desfecho no nome.
    function handleTerminalClosed(id, exitCode) {
        if (id === root.runTerminalId && !isFinishedRun(id)) {
            const codigo = exitCode === undefined ? -1 : exitCode;
            root.finishedRuns[id] = codigo;
            const index = indexOfTerminal(id);
            if (index >= 0) {
                const titulo = terminalsListModel.get(index).title;
                terminalsListModel.setProperty(index, "title",
                    titulo + (codigo === 0 ? " ✓" : " ✗ " + codigo));
                root.terminalsRevision += 1;
            }
            return;
        }
        removeTerminalTab(id);
    }

    function removeTerminalTab(id) {
        delete root.finishedRuns[id];
        const index = indexOfTerminal(id);
        if (index >= 0) {
            terminalsListModel.remove(index);
            root.terminalsRevision += 1;
        }
        delete root.terminalRenders[id];
        if (root.activeTerminalId !== id) {
            return;
        }
        if (terminalsListModel.count === 0) {
            root.activeTerminalId = "";
            root.terminalRender = ({});
            return;
        }
        const fallback = Math.min(index < 0 ? 0 : index, terminalsListModel.count - 1);
        const next = terminalsListModel.get(Math.max(0, fallback)).termId;
        root.activeTerminalId = next;
        root.terminalRender = root.terminalRenders[next] || ({});
    }

    /// A execucao abriu: vira aba de terminal com o nome do comando e ganha
    /// o foco. Sem `terminalId` (core antigo) nao ha' o que mostrar.
    function handleRunStarted(command, terminalId) {
        if (terminalId === undefined || terminalId === "") {
            return;
        }
        runTerminalId = terminalId;
        handleTerminalOpened(terminalId, "", runTabTitle(command));
    }

    /// A sessao da execucao fechou: a aba fica (o autor le a saida) e o
    /// desfecho e' dito numa linha.
    function handleRunFinished(success, exitCode) {
        lastRunMessage = success
                         ? qsTr("execução terminou (código %1)").arg(exitCode)
                         : qsTr("execução falhou (código %1)").arg(exitCode);
        // `runTerminalId` fica ate' o `terminalClosed` da sessao marcar a aba.
    }

    function handleRequestFailed(method, message) {
        if (method === "run.start" || method === "run.script" || method === "run.stop") {
            lastRunMessage = message;
            showTabRequested("terminal");
        }
    }
}
