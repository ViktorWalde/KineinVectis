import QtQuick

// O que o core responde de `container.*` -> ContainerController; a aba de
// terminal que `container.open` cria -> RuntimeController, como qualquer outra.
//
// Inclui o `requestFailed` pelo mesmo motivo dos outros roteadores: a recusa
// e' informacao de produto — "nenhum motor no PATH" e' a resposta.
Item {
    id: root

    property var coreClient: null
    property var containerController: null
    property var runtimeController: null

    visible: false

    Connections {
        target: root.coreClient

        function onContainerStatusResolved(status) {
            root.containerController.handleStatus(status);
        }

        function onContainersResolved(containers, engine, rawOutput, hint) {
            root.containerController.handleContainers(containers, engine, rawOutput, hint);
        }

        function onContainerImagesResolved(images, hint) {
            root.containerController.handleImages(images, hint);
        }

        function onContainerFinished(event) {
            root.containerController.handleFinished(event);
        }

        // A sessao ja' esta' na contabilidade do CoreClient; aqui ela vira
        // uma aba com o comando como titulo, e o painel de containers fecha
        // para o terminal aparecer.
        function onContainerTerminalOpened(id, command) {
            root.containerController.close();
            root.runtimeController.handleTerminalOpened(id, command, command);
        }

        function onRequestFailed(method, message) {
            root.containerController.handleFailed(method, message);
        }
    }
}
