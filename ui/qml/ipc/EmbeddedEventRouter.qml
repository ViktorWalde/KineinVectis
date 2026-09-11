import QtQuick

// O que o core responde de `probe.*` -> EmbeddedController.
//
// Inclui o `requestFailed` pelo mesmo motivo do ToolchainEventRouter: a recusa
// e' informacao de produto — "probe-rs nao encontrado" e' a resposta.
Item {
    id: root

    property var coreClient: null
    property var embeddedController: null

    visible: false

    Connections {
        target: root.coreClient

        function onProbesResolved(probes, toolAvailable, rawOutput, hint) {
            root.embeddedController.handleProbes(probes, toolAvailable, rawOutput, hint);
        }

        function onRequestFailed(method, message) {
            root.embeddedController.handleFailed(method, message);
        }
    }
}
