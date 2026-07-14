import QtQuick

Item {
    id: root

    property var coreClient: null
    property var searchController: null

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
            root.searchController.handleFileSearchResults(matches, truncated);
        }

        function onCommandsListed(commands) {
            root.searchController.handleCommandsListed(commands);
        }

        function onLspSymbolsResolved(symbols) {
            root.searchController.handleSymbolsResolved(symbols);
        }

        function onRequestFailed(method, message) {
            if (method === "fs.search" || method === "fs.replace"
                    || method === "fs.findFiles"
                    || method === "lsp.documentSymbols"
                    || method === "lsp.workspaceSymbols") {
                root.searchController.handleRequestFailed(method, message);
            }
        }
    }
}
