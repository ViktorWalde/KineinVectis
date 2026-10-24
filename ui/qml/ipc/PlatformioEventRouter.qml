import QtQuick

// Roteia as respostas e o evento de `platformio.*` para o PlatformioController
// (0.191.0, 52 §14.45; o pedido da pagina desde a 0.192.0, §14.46). Inclui o `requestFailed`: um `pio` ausente ou um
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

        function onPlatformioHomeExited(code, reason) {
            root.platformioController.handleExited(code, reason);
        }

        function onPlatformioHomeReleased(releasing) {
            root.platformioController.handleReleased(releasing);
        }

        function onPlatformioHomeCommand(method, params) {
            root.platformioController.handleCommand(method, params);
        }

        function onRequestFailed(method, message, code) {
            root.platformioController.handleFailed(method, message, code);
        }
    }
}
