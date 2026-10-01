import QtQuick

// Comandos da paleta executados DEPOIS de o workspace abrir (Etapa 2 F5/F6,
// 2026-09-18): `KINEIN_STARTUP_COMMANDS=build.run,view.problems` — o que o
// gate e a foto headless precisam para ver a IDE num estado (um build que
// falha, um painel aberto) sem um clique. Sem a env, nada acontece.
//
// O PASSEIO (0.3.6, roadmap 53 §5.2): com `@passo=<ms>` como primeiro item, os
// comandos rodam UM POR VEZ, com esse intervalo, e cada passo deixa
// `KINEIN_PASSEIO passo=<id>` no stderr. Um aviso do motor QML que aparece
// depois dessa linha e antes da proxima pertence a esse comando — e' assim que
// o gate acha o DONO de cada aviso, em vez de so' saber que ele existe.
Item {
    id: root

    property var coreClient: null
    property var commandDispatcher: null
    property int delayMs: 1500
    property var pending: []

    visible: false

    function stepInterval(commands) {
        const first = commands.length > 0 ? commands[0] : "";
        return first.indexOf("@passo=") === 0 ? parseInt(first.substring(7), 10) : 0;
    }

    Timer {
        id: stepTimer

        repeat: true
        onTriggered: {
            if (root.pending.length === 0) {
                stop();
                console.info("KINEIN_PASSEIO fim");
                return;
            }
            const next = root.pending[0];
            root.pending = root.pending.slice(1);
            console.info("KINEIN_PASSEIO passo=" + next);
            root.commandDispatcher.execute(next);
        }
    }

    Timer {
        id: atraso

        interval: root.delayMs
        repeat: false
        onTriggered: {
            const commands = root.coreClient ? root.coreClient.startupCommands : [];
            const interval = root.stepInterval(commands);
            if (interval > 0) {
                root.pending = commands.slice(1);
                stepTimer.interval = interval;
                stepTimer.start();
                return;
            }
            for (let i = 0; i < commands.length; i++) {
                root.commandDispatcher.execute(commands[i]);
            }
        }

    }

    // Um id que ninguem trata NAO pode terminar em silencio (V3). Aqui e' onde
    // isso mais dói: um erro de digitacao no `KINEIN_STARTUP_COMMANDS` fazia a
    // foto headless e o gate medirem uma IDE em estado diferente do pedido, sem
    // nada dizer. Vai para stderr, que e' onde quem roda headless olha.
    Connections {
        target: root.commandDispatcher

        function onUnknownCommand(id) {
            console.warn("KINEIN_STARTUP_COMMANDS: ninguem trata o comando `"
                         + id + "` — confira o id na paleta.");
        }
    }

    Connections {
        target: root.coreClient

        function onWorkspaceChanged() {
            if (root.coreClient.workspaceRoot !== "" && root.coreClient.startupCommands.length > 0) {
                atraso.restart();
            }
        }
    }
}
