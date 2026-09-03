import QtQuick

// Espelho do LibraryEventRouter: leva ao core o que o controller pede.
Item {
    id: root

    property var coreClient: null
    property var libraryController: null

    visible: false

    Connections {
        target: root.libraryController

        function onListRequested() {
            root.coreClient.libraryList();
        }

        function onPlanRequested(id, target) {
            root.coreClient.libraryPlan(id, target);
        }
    }
}
