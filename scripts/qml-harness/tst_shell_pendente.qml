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

    property int aberturas: 0
    property var enviados: []
    property string descartada: ""
    property int falhas: 0

    RuntimeController {
        id: runtime

        workspaceRoot: "/tmp/workspace"
        terminalActive: true
        // O guarda e' de 15 s no produto; aqui encurta para o teste nao pagar
        // a espera. E' o MESMO caminho, so' que mais cedo.
        esperaDoShellMs: 60
        onTerminalOpenRequested: root.aberturas += 1
        onTerminalInputRequested: function(id, data) {
            root.enviados.push(id + "|" + data);
        }
        onShellInputDropped: function(command) {
            root.descartada = command;
        }
    }

    // PRIMEIRO cenario: a sessao NAO nasce. A linha nao fica guardada para cair
    // depois num terminal que o autor abriu para outra coisa, e quem pediu fica
    // sabendo que ela nao foi.
    Component.onCompleted: {
        runtime.submitShellInput("ssh nunca@abre");
        if (root.aberturas !== 1) root.falhas += 1;
        if (runtime.pendingShellInput !== "ssh nunca@abre") root.falhas += 2;
        depoisDoGuarda.start();
    }

    Timer {
        id: depoisDoGuarda

        interval: 250
        onTriggered: {
            if (root.descartada !== "ssh nunca@abre") root.falhas += 4;
            if (runtime.pendingShellInput !== "") root.falhas += 8;
            if (root.enviados.length !== 0) root.falhas += 16;

            // SEGUNDO cenario: a sessao nasce, e a linha sai — uma vez, com o
            // \n que o shell espera.
            runtime.submitShellInput("ssh pi@10.0.0.7");
            if (root.aberturas !== 2) root.falhas += 32;
            if (root.enviados.length !== 0) root.falhas += 64;
            runtime.handleTerminalOpened("t1", "/bin/sh");
            if (root.enviados.length !== 1) root.falhas += 128;
            if (root.enviados[0] !== "t1|ssh pi@10.0.0.7\n") root.falhas += 256;
            if (runtime.pendingShellInput !== "") root.falhas += 512;

            // A SEGUNDA sessao nao recebe nada: a linha ja' foi.
            runtime.handleTerminalOpened("t2", "/bin/sh");
            if (root.enviados.length !== 1) root.falhas += 1024;

            // TERCEIRO: com sessao viva, vai direto para a aba ativa, sem pedir
            // terminal nenhum.
            runtime.submitShellInput("ssh outro@host");
            if (root.enviados.length !== 2) root.falhas += 2048;
            if (root.enviados[1] !== "t2|ssh outro@host\n") root.falhas += 4096;
            if (root.aberturas !== 2) root.falhas += 8192;

            // E o guarda nao dispara depois de a linha ter saido.
            semEcoTardio.start();
        }
    }

    Timer {
        id: semEcoTardio

        interval: 250
        onTriggered: {
            if (root.descartada !== "ssh nunca@abre") root.falhas += 16384;
            Qt.exit(root.falhas === 0 ? 0 : 1);
        }
    }
}
