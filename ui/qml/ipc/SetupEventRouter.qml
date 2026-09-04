import QtQuick

// Roteia a resposta de `setup.list` do CoreClient para o SetupController.
//
// Inclui o `requestFailed` porque neste dominio a recusa e' informacao de
// produto: "nao consegui ler a distribuicao" e' algo que o autor precisa ler.
Item {
    id: root

    property var coreClient: null
    property var setupController: null

    visible: false

    Connections {
        target: root.coreClient

        function onSetupListResolved(distroName, family, tools) {
            root.setupController.handleList(distroName, family, tools);
        }

        function onRequestFailed(method, message) {
            root.setupController.handleFailed(method, message);
        }
    }
}
