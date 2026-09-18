import QtQuick

// O que o core responde/emite de `remote.*` -> RemoteController.
Item {
    id: root

    property var coreClient: null
    property var remoteController: null

    visible: false

    Connections {
        target: root.coreClient

        function onRemoteTargetsResolved(targets) {
            root.remoteController.handleTargets(targets);
        }

        function onRemoteJobAccepted(method, jobId, command) {
            root.remoteController.handleJobAccepted(method, jobId, command);
        }

        function onRemoteCommandResolved(result) {
            root.remoteController.handleCommand(result);
        }

        function onRemoteProbed(outcome) {
            root.remoteController.handleProbed(outcome);
        }

        function onRemoteDeployed(outcome) {
            root.remoteController.handleDeployed(outcome);
        }

        function onRemoteOpenAccepted(jobId, command, mirror) {
            root.remoteController.handleOpenAccepted(jobId, command, mirror);
        }

        function onRemoteSynced(outcome) {
            root.remoteController.handleSynced(outcome);
        }

        function onRemoteMirrorChanged(mirror) {
            root.remoteController.handleMirror(mirror);
        }

        function onRequestFailed(method, message, code) {
            root.remoteController.handleFailed(method, message);
        }
    }
}
