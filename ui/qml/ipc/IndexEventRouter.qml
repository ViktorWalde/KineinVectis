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
            root.indexController.symbols.handleIndexSymbols(symbols, total, state);
        }

        // A mesma resposta serve aos dois donos, e cada um decide se e' dele:
        // a caixa compara com o pedido que fez, a aba compara o caminho com o
        // arquivo ativo. Pedir duas vezes o mesmo seria desperdicio.
        function onLspDocumentSymbolsResolved(path, symbols) {
            root.indexController.symbols.handleDocumentSymbols(path, symbols);
        }
    }
}
