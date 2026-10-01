import QtQuick

// Pedido temporário de leitura fora da raiz. O documento só nasce após a
// resposta do core, no modelo único de abas; troca de workspace invalida o
// pedido em trânsito.
Item {
    id: root

    property string workspaceRoot: ""
    property var documentController: null
    property var pending: ({})
    property string errorMessage: ""

    signal readFileRequested(string path)

    visible: false
    onWorkspaceRootChanged: reset()

    function reset() {
        pending = ({});
        errorMessage = "";
    }

    function open(path) {
        if (workspaceRoot === "" || !path.startsWith("/")
                || path.startsWith(workspaceRoot + "/") || path === workspaceRoot) return false;
        const next = pending;
        next[path] = workspaceRoot;
        pending = next;
        errorMessage = "";
        readFileRequested(path);
        return true;
    }

    function consume(path) {
        if (pending[path] !== workspaceRoot) return false;
        const next = pending;
        delete next[path];
        pending = next;
        return true;
    }

    function handleLoaded(path, content) {
        if (!consume(path) || documentController === null) return;
        errorMessage = "";
        documentController.handleExternalFileLoaded(path, content);
    }

    function handleFailed(path, message) {
        if (!consume(path)) return;
        errorMessage = qsTr("Não foi possível abrir %1: %2")
                       .arg(path.substring(path.lastIndexOf("/") + 1)).arg(message);
    }
}
