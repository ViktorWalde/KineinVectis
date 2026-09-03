import QtQuick

// Espelho do ToolchainEventRouter: leva ao core o que o controller pede.
Item {
    id: root

    property var coreClient: null
    property var toolchainController: null

    visible: false

    Connections {
        target: root.toolchainController

        function onGetRequested() {
            root.coreClient.toolchainGet();
        }

        function onSetRequested(role, id) {
            root.coreClient.toolchainSet(role, id);
        }
    }
}
