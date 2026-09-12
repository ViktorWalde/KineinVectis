import QtQuick

// O que o core responde/emite de `index.*` -> IndexController (os totais) e
// SearchEverywhereController (os simbolos do `#nome`, antes do LSP responder).
Item {
    id: root

    property var coreClient: null
    property var indexController: null
    property var searchEverywhereController: null

    visible: false

    Connections {
        target: root.coreClient

        function onIndexStatusResolved(stats) {
            root.indexController.handleStatus(stats);
        }

        function onIndexProgressed(files, symbols) {
            root.indexController.handleProgress(files, symbols);
        }

        function onIndexFinished(stats) {
            root.indexController.handleFinished(stats);
        }

        function onIndexContextResolved(context) {
            root.indexController.handleContext(context);
        }

        function onIndexSymbolsResolved(symbols, total, state) {
            root.searchEverywhereController.handleIndexSymbols(symbols, total, state);
        }
    }
}
