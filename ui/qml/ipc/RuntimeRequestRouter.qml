import QtQuick

// Espelho do RuntimeEventRouter: aquele traz o que o core manda (render,
// opened, closed), este leva ao core o que o RuntimeController pede.
//
// Nada aqui interpreta o terminal. Cada funcao repassa o gesto cru para o
// mesmo metodo do CoreClient que ja existia — em especial `terminalWheel`, que
// e so transporte: quem decide o que a roda significa e o core (`wheel_action`,
// protocolo 0.60.0), conforme o modo VT. A UI voltar a decidir isso foi
// exatamente o bug que fazia o Claude nao rolar e o Codex rolar.
Item {
    id: root

    property var coreClient: null
    property var runtimeController: null
    property var runConfigController: null

    visible: false

    function available(method) {
        if (coreClient && coreClient.connected !== false) return true;
        if (method === "terminal.close") runtimeController.handleRequestFailed(method,
            qsTr("O serviço da IDE está desconectado. Aguarde a reconexão e tente novamente."));
        return false;
    }

    Connections {
        target: root.runtimeController

        function onTerminalOpenRequested(cwd, forCommand) {
            const requestId = root.coreClient.terminalOpen(cwd);
            if (forCommand) root.runtimeController.bindShellRequest(requestId);
        }

        function onTerminalInputRequested(id, data) {
            root.coreClient.terminalInput(id, data);
        }

        function onTerminalResizeRequested(id, cols, rows) {
            root.coreClient.terminalResize(id, cols, rows);
        }

        function onTerminalScrollRequested(id, offset) {
            root.coreClient.terminalScroll(id, offset);
        }

        function onTerminalClearScrollbackRequested(id) {
            root.coreClient.terminalClearScrollback(id);
        }

        function onTerminalSelectAllRequested(id, selectionId) {
            root.coreClient.terminalSelectAll(id, selectionId);
        }

        function onTerminalCopySelectionRequested(id, selectionId) {
            root.coreClient.terminalCopySelection(id, selectionId);
        }

        function onTerminalWheelRequested(id, col, row, lines, modifiers) {
            root.coreClient.terminalWheel(id, col, row, lines, modifiers);
        }

        function onTerminalCloseRequested(id) {
            if (root.available("terminal.close")) root.coreClient.terminalClose(id);
        }

        // `device` (0.110.0) vai como veio: vazio e' campo ausente na ponte.
        function onRunStartRequested(command, device, flash) {
            root.runtimeController.bindRunRequest(root.available("run.start")
                ? root.coreClient.runStart(command, device, flash) : -1);
        }

        function onRunScriptRequested(path, device) {
            root.runtimeController.bindRunRequest(root.available("run.script")
                ? root.coreClient.runScript(path, device) : -1);
        }

        function onRunStopRequested() {
            root.runtimeController.bindStopRequest(root.available("run.stop") ? root.coreClient.runStop() : -1);
        }

    }

    // Os sinais de configuracao salva vivem no RunConfigController desde que
    // ele saiu do RuntimeController: escutar no lugar errado nao falha no build,
    // so deixa de funcionar em silencio.
    Connections {
        target: root.runConfigController

        function onRefreshRequested() {
            if (root.coreClient && root.coreClient.connected !== false) root.coreClient.runConfigList();
            else root.runConfigController.handleReadFailed(qsTr("O serviço da IDE está desconectado."));
        }

        function onSaveRunConfigRequested(id, name, command, flash) {
            if (!root.coreClient || root.coreClient.connected === false) {
                root.runConfigController.bindMutation(-1);
                return;
            }
            root.runConfigController.bindMutation(root.coreClient.runConfigSave(id, name, command, flash));
        }

        function onDeleteRunConfigRequested(id) {
            if (!root.coreClient || root.coreClient.connected === false) {
                root.runConfigController.bindMutation(-1);
                return;
            }
            root.runConfigController.bindMutation(root.coreClient.runConfigDelete(id));
        }

        function onSetActiveRunConfigRequested(id) {
            if (!root.coreClient || root.coreClient.connected === false) {
                root.runConfigController.bindMutation(-1);
                return;
            }
            root.runConfigController.bindMutation(root.coreClient.runConfigSetActive(id));
        }
    }
}
