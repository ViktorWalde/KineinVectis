import QtQuick

// Leva os pedidos do PlatformioController ao CoreClient (0.191.0, 52 §14.45).
Item {
    id: root

    property var coreClient: null
    property var platformioController: null

    visible: false

    Connections {
        target: root.platformioController

        function onHomeStartRequested() {
            root.coreClient.platformioHomeStart();
        }

        function onHomeStopRequested() {
            root.coreClient.platformioHomeStop();
        }

        function onHomeReleaseRequested(afterSeconds) {
            root.coreClient.platformioHomeRelease(afterSeconds);
        }

        // O projeto que o PIO Home criou abre como o "Abrir pasta": sem
        // confianca, como qualquer pasta (§14.46).
        function onOpenProjectRequested(path) {
            root.coreClient.openWorkspace(path);
        }
    }
}
