import QtQuick

// Roteia as respostas de `sim.*` do CoreClient para o controller.
//
// Inclui o `requestFailed` porque neste dominio a recusa e' informacao de
// produto: "falta o valor de v" e "o resultado nao e' um numero utilizavel"
// sao coisas que o autor precisa ler, nao erros internos.
Item {
    id: root

    property var coreClient: null
    property var simController: null
    property var simRunController: null
    property var simSystemController: null

    visible: false

    Connections {
        target: root.coreClient

        function onSimCatalogResolved(concepts) {
            root.simController.handleCatalog(concepts);
        }

        function onSimFormulaChecked(result) {
            root.simController.handleChecked(result);
        }

        function onSimEvaluated(result) {
            root.simController.handleEvaluated(result);
        }

        function onSimSavedListResolved(simulations) {
            root.simController.handleSavedList(simulations);
        }

        function onSimEstimated(result) {
            root.simRunController.handleEstimated(result);
        }

        function onSimRan(result) {
            root.simRunController.handleRan(result);
        }

        function onSimSystemChecked(result) {
            root.simSystemController.applyCheck(result);
        }

        function onSimSystemRan(result) {
            root.simSystemController.applyRun(result);
        }

        // A recusa vai aos TRES: cada um sabe quais metodos sao dele e ignora
        // o resto. Foi o que evitou que o erro de `git.status` acendesse o
        // aviso do menu de toolchain.
        function onRequestFailed(method, message) {
            root.simController.handleFailed(method, message);
            root.simRunController.handleFailed(method, message);
            root.simSystemController.handleFailed(method, message);
        }
    }
}
