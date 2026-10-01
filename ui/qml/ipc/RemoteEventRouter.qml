import QtQuick

// O que o core responde/emite de `remote.*` -> RemoteController.
Item {
    id: root

    property var coreClient: null
    property var remoteController: null

    visible: false

    Connections {
        target: root.coreClient

        function onRemoteAliasesDiscovered(aliases, sources) {
            root.remoteController.setup.handleAliases(aliases, sources);
        }

        function onRemoteHostResolved(summary) {
            root.remoteController.setup.handleResolved(summary);
        }

        function onRemoteCommandParsed(proposal) {
            root.remoteController.setup.handleParsed(proposal);
        }

        function onRemoteTargetsResolved(targets) {
            root.remoteController.handleTargets(targets);
        }

        function onRemoteJobAccepted(method, jobId, command) {
            if (method === "remote.directories") {
                return;
            }
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
            root.remoteController.workspace.handleOpenAccepted(jobId, command, mirror);
        }

        function onRemoteDirectoriesResolved(outcome) {
            root.remoteController.workspace.handleDirectories(outcome);
        }

        function onRemoteDirectoriesFailed(name, path, message) {
            root.remoteController.workspace.handleBrowseFailed(name, path, message);
        }

        function onRemoteSynced(outcome) {
            root.remoteController.workspace.handleSynced(outcome);
        }

        function onRemoteMirrorChanged(mirror) {
            root.remoteController.workspace.handleMirror(mirror);
        }

        function onRequestFailed(method, message, code) {
            if (method === "remote.directories") {
                return;
            }
            root.remoteController.handleFailed(method, message);
        }
    }
}
