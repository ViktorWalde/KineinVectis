import QtQuick

// Espelho do CoverageEventRouter: leva ao core o que o controller pede.
// O job (coverage.run) sai do JobsController, como build/testes/analise.
Item {
    id: root

    property var coreClient: null
    property var coverageController: null

    visible: false

    Connections {
        target: root.coverageController

        function onLinesRequested(file) {
            root.coreClient.coverageLines(file);
        }
    }
}
