import QtQuick

// Resultado de settings.get/set → SettingsController (fatia M4.1).
Item {
    id: root

    property var coreClient: null
    property var settingsController: null

    visible: false

    Connections {
        target: root.coreClient

        function onSettingsResolved(effective, global, workspace, problems) {
            root.settingsController.handleResolved(effective, global, workspace, problems);
        }

        // INF-07 (0.220.0): o `settings.set` recusado (o arquivo que a IDE
        // nao entende) ou que nao gravou; antes, a falha sumia calada.
        function onRequestFailed(method, message) {
            if (method === "settings.set") root.settingsController.handleSetFailed(message);
        }
    }
}
