import QtQuick
// O RuntimeController REAL (arquivo do projeto, sem copia).
import "../../ui/qml/runtime"

// A ABA DA EXECUCAO NAO PODE SUMIR (2026-10-04, achado contra um sshd real).
//
// Uma execucao rapida (o `app` no alvo, 200 ms) era a ULTIMA sessao viva. O
// CoreClient derrubava o "terminal ativo" ANTES de avisar o fechamento, e o
// controller, ao ver o terminal inativo, descartava toda aba que nao fosse
// execucao terminada — a da execucao ainda nao constava como terminada. O ▶
// rodava e nada aparecia. A ordem certa (a do CoreClient desde entao): o
// fechamento primeiro, o "terminal ativo" depois. Este teste trava o que essa
// ordem garante.
Item {
    id: root

    RuntimeController {
        id: runtime

        workspaceRoot: "/w"
        terminalActive: true
        activeRunName: "Rodar em pi"
    }

    Component.onCompleted: {
        let failures = 0;
        runtime.startRun("");
        runtime.handleRunStarted("ssh -tt pi 'app'", "t1");
        if (runtime.terminalsModel.count !== 1) failures += 1;
        if (runtime.terminalsModel.get(0).title !== "▶ Rodar em pi") failures += 2;
        // A ordem do CoreClient: fechou, e so' depois o terminal fica inativo.
        runtime.handleTerminalClosed("t1", 0);
        runtime.terminalActive = false;
        if (runtime.terminalsModel.count !== 1) failures += 4;
        if (runtime.terminalsModel.get(0).title !== "▶ Rodar em pi ✓") failures += 8;
        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
