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

    // O setup e' filho do controller (roadmap 48 §8.3): pede por conta propria.
    Connections {
        target: root.remoteController ? root.remoteController.setup : null

        function onDiscoverRequested() {
            root.coreClient.remoteDiscover();
        }

        function onResolveRequested(host) {
            root.coreClient.remoteResolve(host);
        }

        function onParseRequested(command) {
            root.coreClient.remoteParseCommand(command);
        }
    }

    // O espelho e o sync (roadmap 48 §8.3), tambem filho do controller.
    Connections {
        target: root.remoteController ? root.remoteController.workspace : null

        function onOpenRequested(name, path) {
            root.coreClient.remoteOpen(name, path);
        }

        function onSyncRequested(direction, paths) {
            root.coreClient.remoteSync(direction, paths);
        }

        // O espelho abre pelo caminho de sempre: e' um workspace local.
        function onWorkspaceOpenRequested(path) {
            root.coreClient.openWorkspace(path);
        }
    }

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

    // O painel Remoto PROMETEU "shell aberto no terminal". Se a sessao nao
    // nasceu e a linha foi descartada, quem prometeu desmente — no mesmo lugar
    // onde a promessa aparece.
    Connections {
        target: root.runtimeController

        function onShellInputDropped(command) {
            if (root.remoteController === null) {
                return;
            }
            root.remoteController.reportShellDropped(command);
        }
    }
}
