import QtQuick
// Carrega o RuntimeController.qml REAL (arquivo do projeto, sem copia).
import "../../ui/qml/runtime"

// A LINHA DO SHELL REMOTO NAO PODE SUMIR (2026-09-25, V4).
//
// O painel Remoto tem "Shell no terminal": o core devolve a linha `ssh ...` e o
// controller a manda para o terminal da IDE. Com NENHUMA sessao aberta — o
// estado normal de quem acabou de abrir a IDE — o `submitShellInput` pedia um
// terminal e devolvia SEM ENVIAR NADA. O autor via um terminal vazio, e o painel
// dizia "shell aberto no terminal".
Item {
    id: root

    property int openRequests: 0
    property var sent: []
    property string droppedLine: ""
    property int failures: 0

    RuntimeController {
        id: runtime

        workspaceRoot: "/tmp/workspace"
        terminalActive: true
        // O guarda e' de 15 s no produto; aqui encurta para o teste nao pagar
        // a espera. E' o MESMO caminho, so' que mais cedo.
        shellTimeoutMs: 60
        onTerminalOpenRequested: root.openRequests += 1
        onTerminalInputRequested: function(id, data) {
            root.sent.push(id + "|" + data);
        }
        onShellInputDropped: function(command) {
            root.droppedLine = command;
        }
    }

    // PRIMEIRO cenario: a sessao NAO nasce. A linha nao fica guardada para cair
    // depois num terminal que o autor abriu para outra coisa, e quem pediu fica
    // sabendo que ela nao foi.
    Component.onCompleted: {
        runtime.submitShellInput("ssh nunca@abre");
        if (root.openRequests !== 1) root.failures += 1;
        if (runtime.pendingShellInput !== "ssh nunca@abre") root.failures += 2;
        afterDeadline.start();
    }

    Timer {
        id: afterDeadline

        interval: 250
        onTriggered: {
            if (root.droppedLine !== "ssh nunca@abre") root.failures += 4;
            if (runtime.pendingShellInput !== "") root.failures += 8;
            if (root.sent.length !== 0) root.failures += 16;

            // SEGUNDO cenario: a sessao nasce, e a linha sai — uma vez, com o
            // \n que o shell espera.
            runtime.submitShellInput("ssh pi@10.0.0.7");
            if (root.openRequests !== 2) root.failures += 32;
            if (root.sent.length !== 0) root.failures += 64;
            runtime.handleTerminalOpened("t1", "/bin/sh");
            if (root.sent.length !== 1) root.failures += 128;
            if (root.sent[0] !== "t1|ssh pi@10.0.0.7\n") root.failures += 256;
            if (runtime.pendingShellInput !== "") root.failures += 512;

            // A SEGUNDA sessao nao recebe nada: a linha ja' foi.
            runtime.handleTerminalOpened("t2", "/bin/sh");
            if (root.sent.length !== 1) root.failures += 1024;

            // TERCEIRO: com sessao viva, vai direto para a aba ativa, sem pedir
            // terminal nenhum.
            runtime.submitShellInput("ssh outro@host");
            if (root.sent.length !== 2) root.failures += 2048;
            if (root.sent[1] !== "t2|ssh outro@host\n") root.failures += 4096;
            if (root.openRequests !== 2) root.failures += 8192;

            // E o guarda nao dispara depois de a linha ter saido.
            noLateEcho.start();
        }
    }

    Timer {
        id: noLateEcho

        interval: 250
        onTriggered: {
            if (root.droppedLine !== "ssh nunca@abre") root.failures += 16384;
            Qt.exit(root.failures === 0 ? 0 : 1);
        }
    }
}
