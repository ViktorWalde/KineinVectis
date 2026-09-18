import QtQuick

Item {
    id: root

    property string workspaceRoot: ""
    property string activeWorkspaceRoot: ""
    property var toolsList: []

    // Uma ferramenta detectada durante a varredura substitui a de mesmo id
    // (ou entra): a lista cresce enquanto o scan roda.
    function handleToolDetected(tool) {
        if (!tool || !tool.id) {
            return;
        }
        const lista = (toolsList || []).filter(function(t) { return t.id !== tool.id; });
        lista.push(tool);
        toolsList = lista;
    }

    signal clearWorkspaceUiRequested()

    visible: false

    function handleWorkspaceChanged() {
        const newRoot = workspaceRoot;
        if (newRoot === "") {
            clearWorkspaceUiRequested();
            activeWorkspaceRoot = "";
            return;
        }
        if (activeWorkspaceRoot !== "" && activeWorkspaceRoot !== newRoot) {
            clearWorkspaceUiRequested();
        }
        activeWorkspaceRoot = newRoot;
    }
}
