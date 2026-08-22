import QtQuick

Item {
    id: root

    property var projectTree: null
    property var markdownMode: null
    property var editorController: null
    property var jobsController: null
    property var compileContextController: null
    property var buildTargetsController: null
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
        // Contexto de compilacao do projeto anterior nao pode sobreviver a
        // troca: caminho relativo colide e o painel mostraria o arquivo errado.
        compileContextController.clear();
        buildTargetsController.clear();
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
