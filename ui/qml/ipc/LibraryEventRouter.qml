import QtQuick

// Roteia as respostas de `library.*` do CoreClient para o LibraryController.
//
// Inclui o `requestFailed` porque neste dominio a recusa e informacao de
// produto: "biblioteca desconhecida" e "alvo vazio" sao coisas que o usuario
// precisa ler, nao erros internos.
Item {
    id: root

    property var coreClient: null
    property var libraryController: null

    visible: false

    Connections {
        target: root.coreClient

        function onLibraryListResolved(libraries) {
            root.libraryController.handleList(libraries);
        }

        function onLibraryPlanResolved(plan) {
            root.libraryController.handlePlan(plan);
        }

        function onRequestFailed(method, message) {
            root.libraryController.handleFailed(method, message);
        }
    }
}
