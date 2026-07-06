import QtQuick

Item {
    id: root

    property var projectTree: null
    property var editorController: null
    property var jobsController: null
    property var searchController: null
    property var runtimeController: null
    property var bottomPanelHost: null

    visible: false

    function clear() {
        projectTree.clear();
        editorController.clear();
        jobsController.clear();
        searchController.clear();
        runtimeController.clear();
        bottomPanelHost.clearSearchInput();
        bottomPanelHost.clearRunInput();
        bottomPanelHost.clearTerminalInput();
    }
}
