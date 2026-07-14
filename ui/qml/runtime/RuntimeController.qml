import QtQuick

Item {
    id: root

    property string workspaceRoot: ""
    property bool running: false
    property bool terminalActive: false
    property bool terminalPanelVisible: false
    // D2 (docs/24): grid do terminal (cores/cursor/spans) vindo do core. Com a
    // D2.3 há VÁRIAS sessões: este é o grid da ABA ATIVA. Os demais ficam em
    // `terminalRenders` e voltam intactos ao trocar de aba.
    property var terminalRender: ({})
    // D2.3 (docs/24): multi-terminal. O core dá um `id` por sessão; a UI guarda
    // a lista (pras abas), o render de cada uma e quem está ativa.
    property alias terminalsModel: terminalsListModel
    property string activeTerminalId: ""
    property var terminalRenders: ({})
    // Numeração das abas: NUNCA decrementa. Se decrementasse, fechar a 2 de 3
    // faria a próxima nascer "Terminal 3" de novo — dois com o mesmo nome.
    property int terminalSeq: 0
    property string terminalText: ""
    property alias runModel: runItemsModel
    // Sessao visivel dentro da aba Terminal (fatia M2.1): o shell PTY e o
    // processo controlado (run.*) dividem a mesma aba, mas nunca o backend.
    property string terminalSession: "shell"
    property alias runConfigsModel: runConfigsListModel
    property string activeConfigId: ""
    property string activeConfigName: ""
    property bool runConfigDialogVisible: false
    property string editingConfigId: ""
    property bool configMenuVisible: false
    property real configMenuX: 0
    property real configMenuY: 0

    signal showTabRequested(string tab)
    signal terminalOpenRequested()
    signal terminalInputRequested(string id, string data)
    signal terminalResizeRequested(string id, int cols, int rows)
    signal terminalScrollRequested(string id, int offset)
    signal terminalCloseRequested(string id)
    signal runStartRequested(string command)
    signal runStopRequested()
    signal runStdinRequested(string data)
    signal focusTerminalInputRequested()
    signal clearTerminalInputRequested()
    signal clearRunInputRequested()
    signal saveRunConfigRequested(string id, string name, string command)
    signal deleteRunConfigRequested(string id)
    signal setActiveRunConfigRequested(string id)
    signal runConfigDialogOpenRequested(string name, string command)

    visible: false

    ListModel {
        id: runItemsModel
    }

    // D2.3: uma linha por terminal aberto — { termId, title }.
    ListModel {
        id: terminalsListModel
    }

    ListModel {
        id: runConfigsListModel
    }

    function clearTerminals() {
        terminalsListModel.clear();
        root.activeTerminalId = "";
        root.terminalRender = ({});
        root.terminalRenders = ({});
        root.terminalSeq = 0;
        root.terminalSession = "shell";
    }

    function clear() {
        runItemsModel.clear();
        clearTerminals();
        terminalText = "";
        configMenuVisible = false;
        runConfigDialogVisible = false;
    }

    // Um crash do core não produz `event.terminal.closed` para cada shell.
    // `CoreClient` zera terminalActive ao desconectar; descarte aqui as abas
    // que apontariam para ids do processo morto.
    onTerminalActiveChanged: {
        if (!root.terminalActive && terminalsListModel.count > 0) {
            clearTerminals();
        }
    }

    function appendRunLine(line, kind) {
        runItemsModel.append({ line: line, kind: kind });
        if (runItemsModel.count > 2000) {
            runItemsModel.remove(0);
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
    function handleTerminalOpened(id, shell) {
        root.terminalSeq += 1;
        root.terminalRenders[id] = ({});
        terminalsListModel.append({
            "termId": id,
            "title": qsTr("Terminal %1").arg(root.terminalSeq)
        });
        selectTerminal(id);
    }

    /// Troca a aba ativa. O grid da sessão volta INTACTO (o core mantém o
    /// emulador vivo de cada uma; a UI só guarda o último render).
    function selectTerminal(id) {
        root.activeTerminalId = id;
        root.terminalSession = "shell";
        root.terminalRender = root.terminalRenders[id] || ({});
        showTabRequested("terminal");
        focusTerminalInputRequested();
    }

    function newTerminal() {
        terminalOpenRequested();
    }

    function closeTerminal(id) {
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
        terminalSession = "shell";
        showTabRequested("terminal");
        // Sem nenhuma sessão viva, Alt+F12 tem que MATERIALIZAR um terminal.
        if (root.activeTerminalId === "") {
            terminalOpenRequested();
        }
        focusTerminalInputRequested();
    }

    function setTerminalSession(session) {
        terminalSession = session === "run" ? "run" : "shell";
        focusTerminalInputRequested();
    }

    function clearActiveSession() {
        if (terminalSession === "run") {
            runItemsModel.clear();
        } else {
            terminalText = "";
        }
    }

    function submitShellInput(text) {
        if (root.activeTerminalId === "") {
            terminalOpenRequested();
            return;
        }
        terminalInputRequested(root.activeTerminalId, text + "\n");
        clearTerminalInputRequested();
    }

    function startRun(command) {
        if (workspaceRoot === "" || running) {
            return;
        }
        terminalSession = "run";
        showTabRequested("terminal");
        focusTerminalInputRequested();
        runStartRequested(command);
    }

    function stopRun() {
        if (running) {
            runStopRequested();
        }
    }

    function submitRunInput(text) {
        if (text === "") {
            return;
        }
        clearRunInputRequested();
        if (running) {
            appendRunLine("> " + text, "stdin");
            runStdinRequested(text + "\n");
            return;
        }
        startRun(text);
    }

    function handleRunConfigs(configs, activeId) {
        runConfigsListModel.clear();
        activeConfigId = activeId !== undefined ? activeId : "";
        activeConfigName = "";
        for (let i = 0; i < configs.length; i++) {
            runConfigsListModel.append({
                id: configs[i].id,
                name: configs[i].name,
                command: configs[i].command
            });
            if (configs[i].id === activeConfigId) {
                activeConfigName = configs[i].name;
            }
        }
        if (activeConfigName === "") {
            activeConfigId = "";
        }
    }

    function activeConfigCommand() {
        for (let i = 0; i < runConfigsListModel.count; i++) {
            if (runConfigsListModel.get(i).id === activeConfigId) {
                return runConfigsListModel.get(i).command;
            }
        }
        return "";
    }

    function openConfigMenu(x, y) {
        configMenuX = x;
        configMenuY = y;
        configMenuVisible = true;
    }

    function closeConfigMenu() {
        configMenuVisible = false;
    }

    function chooseConfig(id) {
        configMenuVisible = false;
        setActiveRunConfigRequested(id);
    }

    function openNewConfigDialog() {
        configMenuVisible = false;
        editingConfigId = "";
        runConfigDialogVisible = true;
        runConfigDialogOpenRequested("", "");
    }

    function openEditConfigDialog() {
        if (activeConfigId === "") {
            return;
        }
        configMenuVisible = false;
        editingConfigId = activeConfigId;
        runConfigDialogVisible = true;
        runConfigDialogOpenRequested(activeConfigName, activeConfigCommand());
    }

    function confirmConfigDialog(name, command) {
        if (name === "" || command === "") {
            return;
        }
        runConfigDialogVisible = false;
        saveRunConfigRequested(editingConfigId, name, command);
    }

    function cancelConfigDialog() {
        runConfigDialogVisible = false;
    }

    function deleteActiveConfig() {
        configMenuVisible = false;
        if (activeConfigId !== "") {
            deleteRunConfigRequested(activeConfigId);
        }
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

    /// Uma sessão morreu (shell saiu ou fechamos a aba): tira da lista e, se
    /// era a ativa, cai pra vizinha. Sem sessão, a aba fica vazia e a próxima
    /// tecla reabre.
    function handleTerminalClosed(id) {
        const index = indexOfTerminal(id);
        if (index >= 0) {
            terminalsListModel.remove(index);
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

    function handleRunStarted(command) {
        appendRunLine("$ " + command, "command");
    }

    function handleRunOutput(line, stream) {
        appendRunLine(line, stream);
    }

    function handleRunFinished(success, exitCode) {
        appendRunLine(success
                      ? qsTr("== processo finalizado (codigo %1) ==").arg(exitCode)
                      : qsTr("== processo encerrou com falha (codigo %1) ==")
                            .arg(exitCode),
                      success ? "info" : "stderr");
    }

    function handleRequestFailed(method, message) {
        if (method === "run.start" || method === "run.stdin"
                || method === "run.stop") {
            appendRunLine(message, "stderr");
        }
    }
}
