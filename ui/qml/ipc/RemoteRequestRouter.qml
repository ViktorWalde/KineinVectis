import QtQuick

// Espelho do RemoteEventRouter: leva ao core o que o controller pede. O que
// o `remote.command` produz vai para o dono certo: run configs, kit
// (`toolchain.setKit` so' com remoteTarget/debugServer) ou o terminal da IDE.
Item {
    id: root

    property var coreClient: null
    property var remoteController: null
    property var runtimeController: null

    visible: false

    Connections {
        target: root.remoteController

        function onListRequested() {
            root.coreClient.remoteList();
        }

        function onSaveRequested(target) {
            root.coreClient.remoteSave(target);
        }

        function onRemoveRequested(name) {
            root.coreClient.remoteRemove(name);
        }

        function onProbeRequested(name) {
            root.coreClient.remoteProbe(name);
        }

        function onDeployRequested(name, source, dest) {
            root.coreClient.remoteDeploy(name, source, dest);
        }

        function onCommandRequested(name, kind, program, port) {
            root.coreClient.remoteCommand(name, kind, program, port);
        }

        function onRunConfigRequested(name, command) {
            root.coreClient.runConfigSave("", name, command);
        }

        function onKitRemoteRequested(remoteTarget, debugServer) {
            root.coreClient.toolchainSetKitRemote(remoteTarget, debugServer);
        }

        function onShellRequested(command) {
            root.runtimeController.submitShellInput(command);
        }
    }
}
