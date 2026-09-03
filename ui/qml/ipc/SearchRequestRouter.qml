import QtQuick

// Espelho do SearchEventRouter: aquele traz os resultados, este leva ao core o
// que os dois donos da busca pedem.
//
// POR QUE CONHECE O EDITOR, como o GitRequestRouter. Duas arestas reais:
//
//   - `replace` na busca reescreve arquivo em disco e nao pode rodar com
//     arquivo modificado no editor — a edicao nao salva seria perdida. Mesma
//     guarda do git, mesmo motivo.
//   - symbols de documento/workspace precisam do arquivo ATIVO e do seu texto,
//     que sao estado do editor. O core recebe o conteudo junto porque a fonte
//     de verdade do buffer e a UI, nao o disco.
//
// Fora isso, so pedido cru ao core.
Item {
    id: root

    property var coreClient: null
    property var searchController: null
    property var searchEverywhereController: null
    property var editorController: null

    visible: false

    Connections {
        target: root.searchController

        function onSearchInFilesRequested(query, caseSensitive) {
            root.coreClient.searchInFiles(query, caseSensitive);
        }

        function onReplaceInFilesRequested(query, replacement, caseSensitive) {
            if (root.editorController.hasModifiedFiles()) {
                root.searchController.rejectReplaceForDirtyEditors();
                return;
            }
            root.coreClient.replaceInFiles(query, replacement, caseSensitive);
        }
    }

    // Segundo dono: a caixa modal. Bloco proprio pelo mesmo motivo do
    // EditorRequestRouter — escutar no objeto errado nao quebra build, o
    // handler so nunca dispara (ARCHITECTURE.md §8).
    Connections {
        target: root.searchEverywhereController

        function onFindFilesRequested(query) {
            root.coreClient.findFiles(query);
        }

        function onDocumentSymbolsRequested() {
            root.coreClient.requestDocumentSymbols(
                        root.editorController.currentFilePath(),
                        root.editorController.editorText());
        }

        function onWorkspaceSymbolsRequested(query) {
            root.coreClient.requestWorkspaceSymbols(
                        root.editorController.currentFilePath(),
                        root.editorController.editorText(), query);
        }

        function onListCommandsRequested() {
            root.coreClient.listCommands();
        }

        function onReadFileRequested(path) {
            root.coreClient.readFile(path);
        }
    }
}
