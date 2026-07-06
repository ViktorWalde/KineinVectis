import QtQuick

Item {
    id: root

    property string workspaceRoot: ""
    property string activeWorkspaceRoot: ""
    property var toolsList: []

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
