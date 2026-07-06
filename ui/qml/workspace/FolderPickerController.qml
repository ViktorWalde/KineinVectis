import QtQuick

Item {
    id: root

    property string currentPath: ""
    property string parentPath: ""
    property string selectedPath: ""
    property string homePath: "/"
    property string errorText: ""
    property bool loading: false
    property string createMode: ""
    property string createTemplate: "cppCmake"
    property string pendingSelectionPath: ""
    property string pathDraft: ""
    property string createName: ""
    property alias entriesModel: entryModel

    signal browseRequested(string path)
    signal openRequested(string path)
    signal createFolderRequested(string parent, string name)
    signal createProjectRequested(string parent, string name, string templateId)
    signal pathFocusRequested()
    signal createFocusRequested()

    visible: false

    ListModel {
        id: entryModel
    }

    function open(startPath) {
        const path = startPath !== "" ? startPath : "/";
        errorText = "";
        selectedPath = path;
        createMode = "";
        pendingSelectionPath = "";
        pathDraft = path;
        browsePath(path);
        pathFocusRequested();
    }

    function close() {
        loading = false;
        errorText = "";
        createMode = "";
        createName = "";
    }

    function browsePath(path) {
        const cleanPath = path.trim();
        if (cleanPath === "") {
            return;
        }
        loading = true;
        errorText = "";
        browseRequested(cleanPath);
    }

    function setListing(path, parent, entries) {
        loading = false;
        currentPath = path;
        parentPath = parent;
        selectedPath = pendingSelectionPath !== "" ? pendingSelectionPath : path;
        pendingSelectionPath = "";
        errorText = "";
        pathDraft = path;

        entryModel.clear();
        for (let i = 0; i < entries.length; i++) {
            entryModel.append({
                name: entries[i].name,
                path: entries[i].path
            });
        }
    }

    function showError(message) {
        loading = false;
        errorText = message;
    }

    function openSelected() {
        const path = selectedPath !== "" ? selectedPath : currentPath;
        if (path !== "") {
            openRequested(path);
        }
    }

    function beginCreateFolder() {
        createMode = "folder";
        errorText = "";
        createName = "";
        createFocusRequested();
    }

    function beginCreateProject() {
        createMode = "project";
        errorText = "";
        createName = "";
        createFocusRequested();
    }

    function cancelCreate() {
        createMode = "";
        errorText = "";
        createName = "";
    }

    function submitCreate() {
        const name = createName.trim();
        if (name === "") {
            errorText = qsTr("Informe um nome.");
            return;
        }
        if (createMode === "folder") {
            createFolderRequested(currentPath, name);
        } else if (createMode === "project") {
            createProjectRequested(currentPath, name, createTemplate);
        }
    }

    function selectAfterRefresh(path) {
        pendingSelectionPath = path;
    }
}
