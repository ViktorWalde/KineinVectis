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
    }
}
