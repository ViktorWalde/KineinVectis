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

        function onSetKitRequested(preset, sysroot, targetTriple, chip, toolchainFile, svdFile) {
            root.coreClient.toolchainSetKit(preset, sysroot, targetTriple, chip, toolchainFile, svdFile);
        }

        function onInspectSysrootRequested(path) {
            root.coreClient.toolchainInspectSysroot(path);
        }

        function onImportKitRequested(path) {
            root.coreClient.toolchainImportKit(path);
        }

        function onInstallableRequested() {
            root.coreClient.toolchainInstallable();
        }

        function onInstallRequested(id) {
            root.coreClient.toolchainInstall(id);
        }
    }
}
