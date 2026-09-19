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

    Connections {
        target: root.runtimeController

        function onTerminalOpenRequested() {
            root.coreClient.terminalOpen();
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

        function onTerminalWheelRequested(id, col, row, lines, modifiers) {
            root.coreClient.terminalWheel(id, col, row, lines, modifiers);
        }

        function onTerminalCloseRequested(id) {
            root.coreClient.terminalClose(id);
        }

        // `device` (0.110.0) vai como veio: vazio e' campo ausente na ponte.
        function onRunStartRequested(command, device) {
            root.coreClient.runStart(command, device);
        }

        function onRunScriptRequested(path, device) {
            root.coreClient.runScript(path, device);
        }

        function onRunStopRequested() {
            root.coreClient.runStop();
        }

    }

    // Os sinais de configuracao salva vivem no RunConfigController desde que
    // ele saiu do RuntimeController: escutar no lugar errado nao falha no build,
    // so deixa de funcionar em silencio.
    Connections {
        target: root.runConfigController

        function onSaveRunConfigRequested(id, name, command) {
            root.coreClient.runConfigSave(id, name, command);
        }

        function onDeleteRunConfigRequested(id) {
            root.coreClient.runConfigDelete(id);
        }

        function onSetActiveRunConfigRequested(id) {
            root.coreClient.runConfigSetActive(id);
        }
    }
}
