import QtQuick

// Roteia as respostas e o evento de `platformio.*` para o PlatformioController
// (0.191.0, 52 §14.45). Inclui o `requestFailed`: um `pio` ausente ou um
// servidor que nao subiu e' o que a janela diz.
Item {
    id: root

    property var coreClient: null
    property var platformioController: null

    visible: false

    Connections {
        target: root.coreClient

        function onPlatformioHomeStarted(url, port) {
            root.platformioController.handleStarted(url, port);
        }

        function onPlatformioHomeStopped(stopped) {
            root.platformioController.handleStopped(stopped);
        }

        function onPlatformioHomeExited(code) {
            root.platformioController.handleExited(code);
        }

        function onRequestFailed(method, message, code) {
            root.platformioController.handleFailed(method, message, code);
        }
    }
}
