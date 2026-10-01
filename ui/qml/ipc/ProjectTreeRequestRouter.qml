import QtQuick

// Leva ao core o que a arvore de projeto pede: listar diretorio, criar, ler,
// renomear, copiar e apagar. Nao ha EventRouter par — as respostas de `fs.*` chegam pelo
// WorkspaceEventRouter, que ja e o dono desse dominio no sentido de entrada.
//
// So pedido ao core entra aqui. O que a arvore pede a OUTROS dominios (rodar
// script, renomear aba, focar editor) e composicao e fica no Main.qml.
Item {
    id: root

    property var coreClient: null
    property var projectTree: null
    property var documentController: null

    visible: false

    function canRemove(path) {
        if (root.projectTree.entryDeletePending || path !== root.projectTree.entryDeletePath)
            return false;
        if (root.documentController !== null
                && !root.documentController.hasUnsavedUnderPath(path)) return true;
        root.projectTree.entryDeleteError = root.documentController === null
                ? qsTr("O estado do editor não está disponível. Tente novamente.")
                : qsTr("Há alterações não salvas neste arquivo ou pasta. Salve ou feche as abas afetadas antes de excluir.");
        root.projectTree.entryDeleteVisible = true;
        return false;
    }

    function requestRemove(method, path) {
        if (!root.canRemove(path)) return;
        root.projectTree.entryDeleteMethod = method;
        if (method === "fs.trash") root.coreClient.trashPath(path);
        else root.coreClient.deletePath(path);
    }

    Connections {
        target: root.projectTree

        function onListDirRequested(path) {
            root.coreClient.listDir(path);
        }

        function onCreateFileRequested(path) {
            root.coreClient.createFile(path, "");
        }

        function onCreateDirectoryRequested(path) {
            root.coreClient.createDirectory(path);
        }

        function onReadFileRequested(path) {
            root.coreClient.readFile(path);
        }

        function onRenamePathRequested(from, to) {
            root.coreClient.renamePath(from, to);
        }

        function onCopyPathRequested(from, to) {
            root.coreClient.copyPath(from, to);
        }

        function onTransferBatchRequested(operation, items) {
            root.coreClient.transferPaths(operation, items);
        }

        function onDeletePathRequested(path) {
            root.requestRemove("fs.delete", path);
        }

        function onTrashPathRequested(path) {
            root.requestRemove("fs.trash", path);
        }
    }
}
