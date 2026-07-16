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

        // ---- KV Context: atalho visual, sem regra de negocio -------------
        //
        // O core nao sabe o que e "KV Context": para ele toda sessao e um
        // $SHELL no PTY. O rotulo e da UI. Se um dia isto virar parametro do
        // protocolo, a politica por programa que o 0.59.0 removeu voltou.
        runtime.terminalActive = true;
        runtime.clearTerminals();
        root.openedRequests = 0;

        // Abrir contexto pede uma sessao COMUM — nada de argumento ou perfil.
        runtime.openContext();
        if (root.openedRequests !== 1) failures += 16384;
        if (!runtime.pendingContext) failures += 32768;

        // A resposta do core vira aba rotulada e consome a marca.
        runtime.handleTerminalOpened("c1", "/bin/sh");
        if (runtime.pendingContext) failures += 65536;
        if (runtime.terminalsModel.get(0).isContext !== true) failures += 131072;
        if (String(runtime.terminalsModel.get(0).title).indexOf("KV Context") !== 0) {
            failures += 262144;
        }
        if (runtime.activeTerminalId !== "c1") failures += 524288;

        // Uma aba comum aberta depois NAO herda o rotulo.
        runtime.handleTerminalOpened("t9", "/bin/sh");
        if (runtime.terminalsModel.get(1).isContext !== false) failures += 1048576;
        if (String(runtime.terminalsModel.get(1).title).indexOf("Terminal") !== 0) {
            failures += 2097152;
        }

        // Com contexto vivo, o atalho FOCA em vez de acumular aba.
        root.openedRequests = 0;
        runtime.openContext();
        if (root.openedRequests !== 0) failures += 4194304;
        if (runtime.activeTerminalId !== "c1") failures += 8388608;
        if (runtime.terminalsModel.count !== 2) failures += 16777216;

        // Fechado o contexto, o atalho abre outro — e a numeracao nao repete.
        runtime.handleTerminalClosed("c1");
        root.openedRequests = 0;
        runtime.openContext();
        if (root.openedRequests !== 1) failures += 33554432;
        runtime.handleTerminalOpened("c2", "/bin/sh");
        const contexto = runtime.terminalsModel.get(runtime.terminalsModel.count - 1);
        if (String(contexto.title) === "KV Context 1") failures += 67108864;

        // Sem workspace o atalho e inerte (nao ha raiz para o PTY).
        runtime.clearTerminals();
        runtime.workspaceRoot = "";
        root.openedRequests = 0;
        runtime.openContext();
        if (root.openedRequests !== 0 || runtime.pendingContext) failures += 134217728;

        Qt.exit(failures);
    }
}
