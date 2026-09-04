import QtQuick

// Espelho do SetupEventRouter: leva ao core o que o controller pede.
Item {
    id: root

    property var coreClient: null
    property var setupController: null

    visible: false

    Connections {
        target: root.setupController

        function onListRequested() {
            root.coreClient.setupList();
        }
    }
}
