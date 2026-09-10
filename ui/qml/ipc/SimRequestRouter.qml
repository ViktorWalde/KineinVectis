import QtQuick

// Espelho do SimEventRouter: leva ao core o que o controller pede.
Item {
    id: root

    property var coreClient: null
    property var simController: null
    property var simRunController: null
    property var simSystemController: null

    visible: false

    Connections {
        target: root.simRunController

        function onEstimateRequested(duration, step, samples) {
            root.coreClient.simEstimate(duration, step, samples);
        }

        function onRunRequested(conceptId, formula, bindings, values, initial,
                                duration, step, method, samples) {
            root.coreClient.simRun(conceptId, formula, bindings, values, initial,
                                   duration, step, method, samples);
        }
    }

    // A forma VETORIAL. O `equations` vai como lista de mapas; a ORDEM nao
    // importa, porque o core casa pelo campo `component`.
    Connections {
        target: root.simSystemController

        function onCheckRequested(conceptId, equations) {
            root.coreClient.simCheckSystem(conceptId, equations);
        }

        function onRunRequested(conceptId, equations, values, initial,
                                duration, step, method, samples) {
            root.coreClient.simRunSystem(conceptId, equations, values, initial,
                                         duration, step, method, samples);
        }
    }

    Connections {
        target: root.simController

        function onCatalogRequested() {
            root.coreClient.simCatalog("");
        }

        function onSavedListRequested() {
            root.coreClient.simList();
        }

        function onSaveRequested(simulation) {
            root.coreClient.simSave(simulation);
        }

        function onForgetRequested(name) {
            root.coreClient.simForget(name);
        }

        function onCheckRequested(conceptId, formula, bindings) {
            root.coreClient.simCheckFormula(conceptId, formula, bindings);
        }

        function onEvaluateRequested(conceptId, formula, bindings, values) {
            root.coreClient.simEvaluate(conceptId, formula, bindings, values);
        }
    }
}
