import QtQuick

Item {
    id: root

    property string workspaceRoot: ""
    property string workspaceKind: ""
    property string homeDir: ""
    property int toolsCount: 0
    property bool showBottomPanel: false
    property string bottomTab: "logs"
    property bool showExplorer: true
    property bool showAssistant: false

    signal folderOpenRequested(string path)
    signal toolsDetectionRequested()

    visible: false

    function relativeToRoot(path) {
        if (workspaceRoot !== "" && path.indexOf(workspaceRoot + "/") === 0) {
            return path.substring(workspaceRoot.length + 1);
        }
        return path;
    }

    function showTab(tab) {
        bottomTab = tab;
        showBottomPanel = true;
    }

    function toggleBottomTab(tab) {
        if (showBottomPanel && bottomTab === tab) {
            showBottomPanel = false;
            return;
        }
        showTab(tab);
        if (tab === "tools" && toolsCount === 0) {
            toolsDetectionRequested();
        }
    }

    function toggleExplorer() {
        showExplorer = !showExplorer;
    }

    function toggleAssistant() {
        showAssistant = !showAssistant;
    }

    function closeAssistant() {
        showAssistant = false;
    }

    function requestOpenFolder() {
        folderOpenRequested(workspaceRoot !== "" ? workspaceRoot : homeDir);
    }

    function kindLabel(kind) {
        const labels = {
            rustCargo: "Rust/Cargo",
            cmake: "CMake",
            maven: "Maven",
            gradle: "Gradle",
            python: "Python",
            unknown: qsTr("Projeto")
        };
        return labels[kind] !== undefined ? labels[kind] : kind;
    }
}
