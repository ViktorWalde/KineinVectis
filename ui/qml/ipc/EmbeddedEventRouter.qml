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

        function onSerialIdentifyStarted(jobId, command) {
            root.embeddedController.handleIdentifyStarted(jobId, command);
        }

        function onSerialIdentified(outcome) {
            root.embeddedController.handleIdentified(outcome);
        }

        function onRequestFailed(method, message) {
            root.embeddedController.handleFailed(method, message);
        }
    }
}
