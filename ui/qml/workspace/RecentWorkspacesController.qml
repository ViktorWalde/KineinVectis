import QtQuick

Item {
    id: root

    property var workspaces: []
    property string errorText: ""

    signal listRequested()
    signal openRequested(string rootPath)
    signal pinRequested(string rootPath, bool pinned)
    signal removeRequested(string rootPath)
    signal clearRequested()

    visible: false

    function handleResolved(items) {
        workspaces = items !== undefined && items !== null ? items : [];
        errorText = "";
    }

    function handleRequestFailed(method, message) {
        if (method.indexOf("workspace.recent.") === 0) {
            errorText = message;
        }
    }

    function entryFor(rootPath) {
        for (let index = 0; index < workspaces.length; index++) {
            if (workspaces[index].root === rootPath) {
                return workspaces[index];
            }
        }
        return null;
    }

    function openWorkspace(rootPath) {
        const entry = entryFor(rootPath);
        if (entry === null || !entry.available) {
            errorText = qsTr("Esse workspace não está mais disponível.");
            return;
        }
        errorText = "";
        openRequested(rootPath);
    }

    function togglePinned(rootPath) {
        const entry = entryFor(rootPath);
        if (entry === null) {
            return;
        }
        pinRequested(rootPath, !entry.pinned);
    }

    function removeWorkspace(rootPath) {
        if (entryFor(rootPath) !== null) {
            removeRequested(rootPath);
        }
    }

    function clearAll() {
        if (workspaces.length > 0) {
            clearRequested();
        }
    }
}
