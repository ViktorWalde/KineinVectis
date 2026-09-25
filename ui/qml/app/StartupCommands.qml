import QtQuick

// Comandos da paleta executados DEPOIS de o workspace abrir (Etapa 2 F5/F6,
// 2026-09-18): `KINEIN_STARTUP_COMMANDS=build.run,view.problems` — o que o
// gate e a foto headless precisam para ver a IDE num estado (um build que
// falha, um painel aberto) sem um clique. Sem a env, nada acontece.
Item {
    id: root

    property var coreClient: null
    property var commandDispatcher: null
    property int delayMs: 1500

    visible: false

    Timer {
        id: atraso

        interval: root.delayMs
        repeat: false
        onTriggered: {
            const commands = root.coreClient ? root.coreClient.startupCommands : [];
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
