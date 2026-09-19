import QtQuick

// Espelho do IndexEventRouter: leva ao core o que os controllers pedem.
Item {
    id: root

    property var coreClient: null
    property var indexController: null
    property var searchEverywhereController: null

    visible: false

    Connections {
        target: root.indexController

        function onStatusRequested() {
            root.coreClient.indexStatus();
        }

        function onContextRequested(path) {
            root.coreClient.indexContext(path);
        }
    }

    Connections {
        target: root.searchEverywhereController

        function onIndexSymbolsRequested(query) {
            root.coreClient.indexSymbols(query, 200);
        }
    }

    // A aba Simbolos (E3-2) pede o mesmo metodo; cada dono guarda o seu.
    Connections {
        target: root.indexController ? root.indexController.symbols : null

        function onIndexSymbolsRequested(query) {
            root.coreClient.indexSymbols(query, 200);
        }
    }
}
