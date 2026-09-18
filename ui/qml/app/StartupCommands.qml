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

    Connections {
        target: root.coreClient

        function onWorkspaceChanged() {
            if (root.coreClient.workspaceRoot !== "" && root.coreClient.startupCommands.length > 0) {
                atraso.restart();
            }
        }
    }
}
