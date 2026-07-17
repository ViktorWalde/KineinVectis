import QtQuick

// Leva ao core o que a arvore de projeto pede: listar diretorio, criar, ler,
// renomear e apagar. Nao ha EventRouter par — as respostas de `fs.*` chegam pelo
// WorkspaceEventRouter, que ja e o dono desse dominio no sentido de entrada.
//
// So pedido ao core entra aqui. O que a arvore pede a OUTROS dominios (rodar
// script, renomear aba, focar editor) e composicao e fica no Main.qml.
Item {
    id: root

    property var coreClient: null
    property var projectTree: null

    visible: false

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

        function onDeletePathRequested(path) {
            root.coreClient.deletePath(path);
        }
    }
}
