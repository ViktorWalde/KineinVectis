import QtQuick

Item {
    id: root

    property var projectTree: null
    property var markdownMode: null
    property var editorController: null
    property var jobsController: null
    property var searchController: null
    property var runtimeController: null
    property var runConfigController: null
    property var debugController: null
    property var gitController: null
    property var diagnosticsController: null
    property var shellController: null
    property var bottomPanelHost: null

    visible: false

    function clear() {
        projectTree.clear();
        // Fechar o workspace descarta o modo "imagem" por arquivo: caminho
        // relativo de OUTRO projeto nao pode herdar o estado deste.
        markdownMode.clear();
        editorController.clear();
        jobsController.clear();
        searchController.clear();
        runtimeController.clear();
        runConfigController.clear();
        debugController.clear();
        gitController.clear();
        diagnosticsController.clear();
        bottomPanelHost.clearSearchInput();
        bottomPanelHost.clearRunInput();
        bottomPanelHost.clearTerminalInput();
    }
}
