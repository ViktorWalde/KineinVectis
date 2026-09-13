import QtQuick

// Espelho do ToolchainEventRouter: leva ao core o que o controller pede.
Item {
    id: root

    property var coreClient: null
    property var toolchainController: null

    visible: false

    Connections {
        target: root.toolchainController

        function onGetRequested(preset) {
            root.coreClient.toolchainGet(preset);
        }

        function onSetRequested(role, id, preset) {
            root.coreClient.toolchainSet(role, id, preset);
        }

        function onSetKitRequested(preset, sysroot, targetTriple, chip) {
            root.coreClient.toolchainSetKit(preset, sysroot, targetTriple, chip);
        }

        function onInstallableRequested() {
            root.coreClient.toolchainInstallable();
        }

        function onInstallRequested(id) {
            root.coreClient.toolchainInstall(id);
        }
    }
}
