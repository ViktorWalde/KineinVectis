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

    visible: false

    Connections {
        target: root.coreClient

        function onProbesResolved(probes, toolAvailable, rawOutput, hint) {
            root.embeddedController.handleProbes(probes, toolAvailable, rawOutput, hint);
        }

        function onBuildSizeResolved(sections, regions, toolAvailable, tool, rawOutput) {
            root.embeddedController.handleBuildSize(sections, regions, toolAvailable, tool, rawOutput);
        }

        function onProjectModelResolved(model) {
            root.embeddedController.handleProject(model);
        }

        function onProjectChanged(model) {
            root.embeddedController.handleProject(model);
        }

        function onSerialPortsResolved(ports, hint) {
            root.embeddedController.handleSerialPorts(ports, hint);
        }

        // A aba do monitor e' uma sessao de terminal como outra: o painel de
        // embarcados fecha para ela aparecer, com o comando como titulo.
        function onSerialMonitorOpened(id, command, tool) {
            root.embeddedController.close();
            root.runtimeController.handleTerminalOpened(id, command, command);
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

        function onFlashProposalResolved(proposal) {
            root.embeddedController.flash.handleProposal(proposal);
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
            root.embeddedController.identity.handleFailed(method, message);
            root.embeddedController.flash.handleFailed(method, message);
            root.embeddedController.access.handleFailed(method, message);
            root.embeddedController.files.handleFailed(method, message);
        }
    }
}
