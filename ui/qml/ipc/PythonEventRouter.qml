import QtQuick

// O que o core responde/emite de `python.*` -> PythonController.
Item {
    id: root

    property var coreClient: null
    property var pythonController: null

    visible: false

    Connections {
        target: root.coreClient

        function onPythonStatusResolved(status) {
            root.pythonController.handleStatus(status);
        }

        function onPythonEnvironmentFinished(outcome) {
            root.pythonController.handleEnvironmentFinished(outcome);
        }
    }
}
