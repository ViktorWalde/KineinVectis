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
    // Rotulo e tipo pedidos para a PROXIMA sessao que o core abrir. Vazios: aba
    // comum, numerada "Terminal N". O `kind` e OPACO aqui — quem pede e quem
    // sabe o que ele significa; este controller so o carimba na aba e o devolve
    // em `terminalOpened`. Estado 100% de UI: o core nao conhece a distincao.
    property string pendingLabel: ""
    property string pendingKind: ""
    // ListModel nao notifica mudanca de conteudo para funcoes; este contador
    // e o gatilho de reavaliacao dos bindings que dependem da lista.
    property int terminalsRevision: 0
    property string terminalText: ""
    property alias runModel: runItemsModel
    // Sessao visivel dentro da aba Terminal (fatia M2.1): o shell PTY e o
    // processo controlado (run.*) dividem a mesma aba, mas nunca o backend.
    property string terminalSession: "shell"

    signal showTabRequested(string tab)
    signal terminalOpenRequested()
    signal terminalOpened(string id, string kind)
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

    visible: false

    ListModel {
        id: runItemsModel
    }

    // D2.3: uma linha por terminal aberto — { termId, title }.
    ListModel {
        id: terminalsListModel
    }

    // `terminal.open` pode FALHAR (ex.: teto de 12 sessões). O erro vai para o
    // handler genérico do CoreClient e `handleTerminalOpened` nunca vem — sem
    // isto a marca ficaria presa e a PRÓXIMA aba comum nasceria com o rótulo
    // pedido por outra. A marca é cosmética, mas errada é errada.
    Timer {
        id: pendingLabelTimeout

        interval: 4000
        repeat: false
        onTriggered: {
            root.pendingLabel = "";
            root.pendingKind = "";
        }
    }

    function clearTerminals() {
        terminalsListModel.clear();
        root.terminalsRevision += 1;
        root.activeTerminalId = "";
        root.terminalRender = ({});
        root.terminalRenders = ({});
        root.terminalSeq = 0;
        root.pendingLabel = "";
        root.pendingKind = "";
        pendingLabelTimeout.stop();
        root.terminalSession = "shell";
    }

    function clear() {
        runItemsModel.clear();
        clearTerminals();
        terminalText = "";
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
    /// Para o core toda sessão é um `$SHELL` no PTY, igual. Rótulo e tipo são
    /// decisão da UI — por isso são consumidos aqui, no retorno, em vez de
    /// virarem parâmetro do protocolo. Ver docs/roadmaps/26 e PONTO_ATUAL §0.2b.
    function handleTerminalOpened(id, shell) {
        root.terminalRenders[id] = ({});
        pendingLabelTimeout.stop();
        const label = root.pendingLabel;
        const kind = root.pendingKind;
        root.pendingLabel = "";
        root.pendingKind = "";
        // Sessão rotulada não consome número de "Terminal N": quem a pediu tem
        // a própria numeração, e pular um número aqui confundiria as abas.
        if (label === "") {
            root.terminalSeq += 1;
        }
        terminalsListModel.append({
            "termId": id,
            "title": label !== "" ? label : qsTr("Terminal %1").arg(root.terminalSeq),
            "kind": kind
        });
        root.terminalsRevision += 1;
        selectTerminal(id);
        terminalOpened(id, kind);
    }

    /// Abre uma sessão já pedindo rótulo e tipo para a aba. O `kind` é OPACO
    /// aqui: quem pede é quem sabe o que ele significa — este controller nunca
    /// o interpreta. Sem rótulo próprio, o caminho é `newTerminal()`.
    function openLabeledTerminal(label, kind) {
        if (workspaceRoot === "") {
            return;
        }
        terminalSession = "shell";
        showTabRequested("terminal");
        root.pendingLabel = label;
        root.pendingKind = kind;
        pendingLabelTimeout.restart();
        terminalOpenRequested();
    }

    /// Id da primeira sessão viva de um tipo, ou "" se não houver.
    function firstTerminalOfKind(kind) {
        for (let i = 0; i < terminalsListModel.count; i++) {
            const item = terminalsListModel.get(i);
            if (item.kind === kind) {
                return item.termId;
            }
        }
        return "";
    }

    /// Tipo da aba ativa, "" para sessão comum. Existe como propriedade (e não
    /// função) porque alimenta binding; `terminalsRevision` está aí só para o
    /// binding reavaliar — `ListModel` não notifica mudança de conteúdo.
    readonly property string activeTerminalKind: {
        void root.terminalsRevision;
        if (root.activeTerminalId === "") {
            return "";
        }
        const index = indexOfTerminal(root.activeTerminalId);
        return index >= 0 ? terminalsListModel.get(index).kind : "";
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

    /// Digita numa sessão ESPECÍFICA, exatamente como se o usuário tivesse
    /// digitado. Não reabre nada: quem chama já sabe que a sessão existe.
    function sendTerminalInput(id, data) {
        if (id === "") {
            return;
        }
        terminalInputRequested(id, data);
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
