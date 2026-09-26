import QtQuick

// Resultados de busca, substituicao e Search Everywhere -> os DOIS donos.
//
// Dois blocos porque sao dois donos desde 2026-09-02: o painel de baixo (que
// tem `fs.replace` atras dele) e a caixa modal. Entregar tudo a um so
// significaria devolver o acoplamento que o corte desfez.
Item {
    id: root

    property var coreClient: null
    property var searchController: null
    property var searchEverywhereController: null

    visible: false

    Connections {
        target: root.coreClient

        function onSearchResults(matches, truncated) {
            root.searchController.handleSearchResults(matches, truncated);
        }

        function onFilesReplaced(files, replacements) {
            root.searchController.handleReplaceResult(files, replacements);
        }

        function onFileSearchResults(matches, truncated) {
            root.searchEverywhereController.handleFileSearchResults(matches, truncated);
        }

        function onCommandsListed(commands) {
            root.searchEverywhereController.handleCommandsListed(commands);
        }

        function onLspDocumentSymbolsResolved(path, symbols) {
            root.searchEverywhereController.handleDocumentSymbols(path, symbols);
        }

        function onLspWorkspaceSymbolsResolved(query, symbols) {
            root.searchEverywhereController.handleWorkspaceSymbols(query, symbols);
        }

        // A recusa vai ao dono do metodo: `fs.search`/`fs.replace` sao do
        // painel; `fs.findFiles` e os symbols sao da caixa.
        function onRequestFailed(method, message) {
            if (method === "fs.search" || method === "fs.replace") {
                root.searchController.handleRequestFailed(method, message);
            }
            if (method === "fs.findFiles" || method === "lsp.documentSymbols"
                    || method === "lsp.workspaceSymbols") {
                root.searchEverywhereController.handleRequestFailed(method, message);
            }
        }
    }
}
