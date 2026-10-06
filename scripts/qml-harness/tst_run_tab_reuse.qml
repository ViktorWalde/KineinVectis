import QtQuick
import "../../ui/qml/runtime"

Item {
    id: root
    property int requests: 0
    property int failures: 0

    RuntimeController {
        id: runtime
        workspaceRoot: "/w"
        terminalActive: true
        onRunScriptRequested: root.requests++
        onRunStartRequested: root.requests++
    }

    function check(value, message) {
        if (!value) {
            failures++;
            console.error("FALHA: " + message);
        }
    }

    Component.onCompleted: {
        runtime.handleTerminalOpened("shell", "sh");
        runtime.handleTerminalRender({id: "shell", marker: "shell intacto"});
        runtime.startScript("/w/a.py");
        runtime.startScript("/w/a.py");
        check(requests === 1, "clique duplo durante o aceite abre só uma execução");
        runtime.handleRunStarted("python a.py", "t1", "arquivo-a", "/w");
        runtime.handleTerminalRender({id: "t1", marker: "primeira"});
        runtime.handleTerminalClosed("t1", 0);
        runtime.startScript("/w/a.py");
        runtime.handleRunStarted("python a.py", "t2", "arquivo-a", "/w");
        check(runtime.terminalsModel.count === 2, "repetir mantém a quantidade de abas");
        check(runtime.terminalsModel.get(1).termId === "t2", "repetir mantém a posição da aba");
        check(Object.keys(runtime.terminalRender).length === 0, "grid anterior não vaza");
        runtime.handleTerminalRender({id: "t1", marker: "atrasada"});
        runtime.handleTerminalClosed("t1", 0);
        runtime.handleTerminalRender({id: "t2", marker: "segunda"});
        runtime.handleRunStarted("python a.py", "t2", "arquivo-a", "/w");
        check(runtime.terminalsModel.count === 2, "aceite duplicado não duplica a aba");
        check(runtime.terminalRender.marker === "segunda", "aceite duplicado preserva a saída");
        runtime.handleTerminalClosed("t2", 7);
        check(runtime.isFinishedRun("t2"), "nova tentativa guarda desfecho");
        runtime.handleRunStarted("python a.py", "t2", "arquivo-a", "/w");
        check(runtime.isFinishedRun("t2") && runtime.terminalRender.marker === "segunda",
              "aceite duplicado após o fim não apaga a saída");
        runtime.runInNewTerminal("ssh alvo");
        runtime.startScript("/w/b.py");
        runtime.handleRunStarted("python b.py", "t3", "arquivo-b", "/w");
        check(runtime.pendingShellInput === "ssh alvo", "linha de shell não entra no programa");
        check(runtime.terminalsModel.count === 3, "outro arquivo ganha sua própria aba");
        runtime.handleTerminalClosed("t2", 7);
        check(runtime.terminalsModel.count === 3, "fim duplicado não remove execução antiga concluída");
        runtime.closeTerminal("t3");
        runtime.handleTerminalClosed("t3", -1);
        check(runtime.terminalsModel.count === 2, "fechar execução viva fecha a aba");
        runtime.startScript("/w/b.py");
        runtime.handleRunStarted("python b.py", "t4", "arquivo-b", "/w");
        runtime.handleTerminalClosed("t4", 0);
        runtime.closeTerminal("t4");
        check(runtime.terminalsModel.count === 2, "fechar execução concluída libera vínculo");
        runtime.handleCoreDisconnected();
        runtime.handleRunStarted("python a.py", "t2", "arquivo-a", "/w");
        check(runtime.terminalsModel.count === 2 && !runtime.isFinishedRun("t2")
              && Object.keys(runtime.terminalRender).length === 0,
              "core reiniciado pode devolver id igual sem herdar o resultado antigo");
        runtime.handleTerminalClosed("t2", 0);
        runtime.selectTerminal("shell");
        check(runtime.terminalRender.marker === "shell intacto", "shell independente permanece");
        runtime.startScript("/w/a.py");
        runtime.handleRequestFailed("run.script", "recusado");
        const count = requests;
        runtime.startScript("/w/a.py");
        check(requests === count + 1, "erro libera próxima tentativa");
        runtime.running = true;
        runtime.startScript("/w/a.py");
        check(requests === count + 1, "execução viva não aceita outro clique");
        runtime.running = false;
        runtime.workspaceRoot = "/outro";
        check(runtime.terminalsModel.count === 0, "trocar projeto limpa automaticamente os vínculos");
        runtime.handleRunStarted("python a.py", "t5", "arquivo-a", "/w");
        check(runtime.terminalsModel.count === 0, "resposta de outro projeto não cria aba");
        runtime.startScript("/outro/a.py");
        runtime.handleCoreDisconnected();
        const disconnected = requests;
        runtime.startScript("/outro/a.py");
        check(requests === disconnected + 1, "desconexão libera o aceite pendente");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
