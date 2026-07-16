import QtQuick

Item {
    id: root

    property string workspaceRoot: ""
    property bool running: false
    property bool terminalActive: false
    property bool terminalPanelVisible: false
    // D2 (docs/roadmaps/24): grid do terminal (cores/cursor/spans) vindo do core. Com a
    // D2.3 há VÁRIAS sessões: este é o grid da ABA ATIVA. Os demais ficam em
    // `terminalRenders` e voltam intactos ao trocar de aba.
    property var terminalRender: ({})
    // D2.3 (docs/roadmaps/24): multi-terminal. O core dá um `id` por sessão; a UI guarda
    // a lista (pras abas), o render de cada uma e quem está ativa.
    property alias terminalsModel: terminalsListModel
    property string activeTerminalId: ""
    property var terminalRenders: ({})
    // Numeração das abas: NUNCA decrementa. Se decrementasse, fechar a 2 de 3
    // faria a próxima nascer "Terminal 3" de novo — dois com o mesmo nome.
    property int terminalSeq: 0
    // Numeracao propria do KV Context e a marca de que o proximo
    // `terminal.open` que voltar do core e uma sessao de contexto. Estado
    // 100% de UI: o core nao conhece a distincao.
    property int contextSeq: 0
    property bool pendingContext: false
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
    signal terminalWheelRequested(string id, int col, int row, int lines,
                                  int modifiers)
    signal terminalCloseRequested(string id)
    signal runStartRequested(string command)
    signal runScriptRequested(string path)
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

    // `terminal.open` pode FALHAR (ex.: teto de 12 sessões). O erro vai para o
    // handler genérico do CoreClient e `handleTerminalOpened` nunca vem — sem
    // isto a marca ficaria presa e a PRÓXIMA aba comum nasceria rotulada
    // "KV Context". A marca é cosmética, mas errada é errada.
    Timer {
        id: contextIntentTimeout

        interval: 4000
        repeat: false
        onTriggered: root.pendingContext = false
    }

    function clearTerminals() {
        terminalsListModel.clear();
        root.activeTerminalId = "";
        root.terminalRender = ({});
        root.terminalRenders = ({});
        root.terminalSeq = 0;
        root.contextSeq = 0;
        root.pendingContext = false;
        contextIntentTimeout.stop();
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
    ///
    /// O core NÃO sabe se a sessão é "KV Context" — para ele toda sessão é um
    /// `$SHELL` no PTY, igual. O rótulo é decisão da UI e mora só aqui; por isso
    /// `pendingContext` é consumido no retorno, e não vira parâmetro do
    /// protocolo. Ver docs/roadmaps/26 e PONTO_ATUAL §0.2b.
    function handleTerminalOpened(id, shell) {
        root.terminalRenders[id] = ({});
        contextIntentTimeout.stop();
        const isContext = root.pendingContext;
        root.pendingContext = false;
        if (isContext) {
            root.contextSeq += 1;
            terminalsListModel.append({
                "termId": id,
                "title": qsTr("KV Context %1").arg(root.contextSeq),
                "isContext": true
            });
        } else {
            root.terminalSeq += 1;
            terminalsListModel.append({
                "termId": id,
                "title": qsTr("Terminal %1").arg(root.terminalSeq),
                "isContext": false
            });
        }
        selectTerminal(id);
    }

    /// KV Context: atalho visual para uma sessão de terminal DEDICADA, para a
    /// sessão do agente não se perder entre os terminais de build.
    ///
    /// É só isso — nenhuma regra de negócio, em camada nenhuma. Não injeta
    /// argumento, não escolhe programa, não filtra byte: abre o mesmo
    /// `terminal.open` que o Alt+F12 abre e apenas rotula a aba. Quem roda
    /// `claude`/`codex` é você, digitando, como em qualquer terminal. Foi
    /// exatamente a política por programa no core que o 0.59.0 removeu.
    ///
    /// Se já existe uma sessão de contexto viva, foca ela em vez de acumular
    /// abas — o ponto é melhorar o fluxo, não multiplicar terminal.
    function openContext() {
        if (workspaceRoot === "") {
            return;
        }
        terminalSession = "shell";
        showTabRequested("terminal");
        const existente = firstContextTerminal();
        if (existente !== "") {
            selectTerminal(existente);
            return;
        }
        root.pendingContext = true;
        contextIntentTimeout.restart();
        terminalOpenRequested();
    }

    /// Id da primeira sessão de contexto viva, ou "" se não houver.
    function firstContextTerminal() {
        for (let i = 0; i < terminalsListModel.count; i++) {
            const item = terminalsListModel.get(i);
            if (item.isContext === true) {
                return item.termId;
            }
        }
        return "";
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

    function startScript(path) {
        if (workspaceRoot === "" || running || path === "") {
            return;
        }
        terminalSession = "run";
        showTabRequested("terminal");
        focusTerminalInputRequested();
        runScriptRequested(path);
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

    // Repassa o gesto cru; nao interpreta. Ver TerminalScrollController.
    function wheelTerminal(col, row, lines, modifiers) {
        if (root.activeTerminalId !== "") {
            terminalWheelRequested(root.activeTerminalId, col, row, lines,
                                   modifiers);
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
