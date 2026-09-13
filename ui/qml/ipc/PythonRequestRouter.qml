import QtQuick

// Espelho do PythonEventRouter: leva ao core o que o controller pede.
Item {
    id: root

    property var coreClient: null
    property var pythonController: null

    visible: false

    Connections {
        target: root.pythonController

        function onStatusRequested() {
            root.coreClient.pythonStatus();
        }

        function onCreateEnvironmentRequested(tool) {
            root.coreClient.pythonCreateEnvironment(tool);
        }
    }
}
