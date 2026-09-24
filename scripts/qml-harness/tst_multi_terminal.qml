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
        if (runtime.terminalsModel.get(0).title !== "terminal"
                || runtime.terminalsModel.get(1).title !== "terminal1") failures += 16384;

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

        runtime.handleTerminalOpened("t3", "/bin/sh");
        if (runtime.terminalsModel.get(1).title !== "terminal"
                || Object.keys(runtime.terminalRender).length !== 0) failures += 32768;
        runtime.handleTerminalOpened("t4", "/bin/sh");
        if (runtime.terminalsModel.get(2).title !== "terminal2") failures += 65536;
        runtime.handleTerminalClosed("t2");
        runtime.handleTerminalOpened("t5", "/bin/sh");
        if (runtime.terminalsModel.get(2).title !== "terminal1") failures += 131072;
        runtime.handleTerminalOpened("t6", "/bin/sh", "logs do container");
        if (runtime.terminalsModel.get(3).title !== "logs do container") failures += 262144;
        runtime.handleTerminalRender({ id: "t1", marker: "antigo mesmo rotulo" });
        runtime.selectTerminal("t3");
        if (Object.keys(runtime.terminalRender).length !== 0) failures += 524288;
        runtime.handleTerminalClosed("t5");
        runtime.handleTerminalOpened("t7", "/bin/sh", "terminal1");
        runtime.handleTerminalOpened("t8", "/bin/sh");
        if (runtime.terminalsModel.get(4).title !== "terminal3") failures += 1048576;

        // Crash/desconexao nao emite closed individual: terminalActive limpa tudo.
        runtime.terminalActive = false;
        if (runtime.terminalsModel.count !== 0) failures += 1024;
        if (runtime.activeTerminalId !== "") failures += 2048;
        if (Object.keys(runtime.terminalRender).length !== 0) failures += 4096;

        // Sem sessao, abrir o painel pede um shell novo.
        runtime.openTerminalPanel();
        if (root.openedRequests !== 1) failures += 8192;

        // O codigo de saida de um processo tem 8 BITS: Qt.exit(256) sai como 0.
        // Enquanto o bitmask ia direto para o exit, todo check com bit >= 256
        // era letra morta: passava verde mesmo quebrado, que e exatamente a
        // doenca que esta suite existe para impedir. O mask agora vai para a
        // SAIDA (onde nao trunca) e o exit so diz passou/falhou.
        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
