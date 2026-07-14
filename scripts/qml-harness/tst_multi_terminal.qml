import QtQuick
// Carrega o RuntimeController.qml REAL (arquivo do projeto, sem copia).
import "../../ui/qml/runtime"

Item {
    id: root
    width: 100
    height: 100

    property int openedRequests: 0
    property string inputId: ""
    property string inputData: ""
    property string closeId: ""

    RuntimeController {
        id: runtime

        workspaceRoot: "/tmp/workspace"
        terminalActive: true
        onTerminalOpenRequested: root.openedRequests += 1
        onTerminalInputRequested: function(id, data) {
            root.inputId = id;
            root.inputData = data;
        }
        onTerminalCloseRequested: function(id) {
            root.closeId = id;
        }
    }

    Component.onCompleted: {
        let failures = 0;

        // Cada resposta de open materializa uma aba e a mais nova fica ativa.
        runtime.handleTerminalOpened("t1", "/bin/sh");
        runtime.handleTerminalRender({ id: "t1", marker: "primeiro" });
        runtime.handleTerminalOpened("t2", "/bin/sh");
        runtime.handleTerminalRender({ id: "t2", marker: "segundo" });
        if (runtime.terminalsModel.count !== 2) failures += 1;
        if (runtime.activeTerminalId !== "t2") failures += 2;
        if (runtime.terminalRender.marker !== "segundo") failures += 4;

        // Trocar de aba restaura o ultimo grid daquela sessao e o input leva id.
        runtime.selectTerminal("t1");
        if (runtime.terminalRender.marker !== "primeiro") failures += 8;
        runtime.sendTerminalKey("x");
        if (root.inputId !== "t1" || root.inputData !== "x") failures += 16;

        // Fechar a ativa seleciona uma vizinha sem destruir o estado dela.
        runtime.closeTerminal("t1");
        if (root.closeId !== "t1") failures += 32;
        runtime.handleTerminalClosed("t1");
        if (runtime.terminalsModel.count !== 1) failures += 64;
        if (runtime.activeTerminalId !== "t2") failures += 128;
        if (runtime.terminalRender.marker !== "segundo") failures += 256;

        // Render atrasado de uma sessao fechada nao pode ressuscitar estado.
        runtime.handleTerminalRender({ id: "t1", marker: "atrasado" });
        runtime.selectTerminal("t2");
        if (runtime.terminalRender.marker !== "segundo") failures += 512;

        // Crash/desconexao nao emite closed individual: terminalActive limpa tudo.
        runtime.terminalActive = false;
        if (runtime.terminalsModel.count !== 0) failures += 1024;
        if (runtime.activeTerminalId !== "") failures += 2048;
        if (Object.keys(runtime.terminalRender).length !== 0) failures += 4096;

        // Sem sessao, abrir o painel pede um shell novo.
        runtime.openTerminalPanel();
        if (root.openedRequests !== 1) failures += 8192;

        Qt.exit(failures);
    }
}
