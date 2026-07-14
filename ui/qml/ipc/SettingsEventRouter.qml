import QtQuick

// Resultado de settings.get/set → SettingsController (fatia M4.1).
Item {
    id: root

    property var coreClient: null
    property var settingsController: null

    visible: false

    Connections {
        target: root.coreClient

        function onSettingsResolved(effective, global, workspace) {
            root.settingsController.handleResolved(effective, global, workspace);
        }
    }
}
