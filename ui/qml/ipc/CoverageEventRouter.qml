import QtQuick

// O que o core responde/emite de `coverage.*` -> CoverageController.
Item {
    id: root

    property var coreClient: null
    property var coverageController: null

    visible: false

    Connections {
        target: root.coreClient

        function onCoverageFinished(outcome) {
            root.coverageController.handleFinished(outcome);
        }

        function onCoverageLinesResolved(file, known, covered, missed) {
            root.coverageController.handleLines(file, known, covered, missed);
        }
    }
}
