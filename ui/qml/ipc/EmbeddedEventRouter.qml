import QtQuick

// O que o core responde de `probe.*`, `build.size` e `serial.*` -> EmbeddedController.
//
// Inclui o `requestFailed` pelo mesmo motivo do ToolchainEventRouter: a recusa
// e' informacao de produto — "probe-rs nao encontrado" e' a resposta.
Item {
    id: root

    property var coreClient: null
    property var embeddedController: null
    property var runtimeController: null
    property bool wasConnected: false

    visible: false

    Connections {
        target: root.coreClient

        function onStatusChanged() {
            if (root.coreClient.connected) {
                root.wasConnected = true;
            } else if (root.wasConnected) {
                root.wasConnected = false;
                root.embeddedController.handleCoreDisconnected();
            }
        }

        function onProbesResolved(probes, toolAvailable, rawOutput, hint) {
            root.embeddedController.handleProbes(probes, toolAvailable, rawOutput, hint);
        }

        function onBuildSizeResolved(sections, regions, toolAvailable, tool, rawOutput) {
            root.embeddedController.handleBuildSize(sections, regions, toolAvailable, tool, rawOutput);
        }

        function onProjectModelResolved(model) {
            root.embeddedController.handleProject(model);
        }

        function onCmakeTargetsResolved(targets, origin) {
            root.embeddedController.flash.handleTargets(targets, origin);
        }

        function onProjectChanged(model) {
            root.embeddedController.handleProject(model);
        }

        function onSerialPortsResolved(ports, hint) {
            root.embeddedController.handleSerialPorts(ports, hint);
        }

        // O monitor é uma aba de terminal; o dock continua disponível ao lado.
        function onSerialMonitorOpened(id, command, tool) {
            root.runtimeController.handleTerminalOpened(id, command, qsTr("Monitor serial · %1").arg(tool));
        }

        // Os FILHOS do EmbeddedController (identity, flash, access, files)
        // sao donos proprios: cada desfecho vai ao dono, e a recusa do core
        // (`requestFailed`) chega a todos — cada um ignora o metodo que nao
        // e' seu. Ate' 2026-09-17 (C2) isto chamava `handleIdentifyStarted`/
        // `handleIdentified` NO PAI, onde nao existem, e `flashProposalResolved`/
        // `serialAccessResolved` nao tinham consumidor: o core respondia e a
        // tela nunca recebia — o "sintoma do Docker" (40 §7.34) de novo.
        function onSerialIdentifyStarted(jobId, command) {
            root.embeddedController.identity.handleStarted(jobId, command);
        }

        function onSerialIdentified(outcome) {
            root.embeddedController.identity.handleIdentified(outcome);
        }

        function onFlashProposalResolved(proposal, requestId) {
            root.embeddedController.flash.handleProposal(proposal, requestId);
        }

        function onFlashProposalFailed(requestId, message) {
            root.embeddedController.flash.handleProposalFailed(requestId, message);
        }

        function onSerialAccessResolved(channels) {
            root.embeddedController.access.handleChannels(channels);
        }

        function onSerialFilesStarted(jobId, command) {
            root.embeddedController.files.handleStarted(jobId, command);
        }

        function onSerialFilesResolved(outcome) {
            root.embeddedController.files.handleOutcome(outcome);
        }

        function onRequestFailed(method, message) {
            root.embeddedController.handleFailed(method, message);
        }
    }
}
