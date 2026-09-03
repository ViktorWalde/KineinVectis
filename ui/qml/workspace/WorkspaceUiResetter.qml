import QtQuick

// Dono unico do "esqueca tudo do workspace anterior".
//
// Ele existe porque estado que muda junto precisa de UM dono: espalhar o reset
// pelos caminhos que trocam de workspace foi o que, no core, deixou o autosave
// do projeto NOVO indo para o banco do ANTERIOR (2026-08-29).
//
// A lista abaixo e a definicao do que e' "por workspace". O que NAO entra, e a
// ausencia e deliberada: o `ShellController`. O que ele guarda — painel aberto,
// aba de baixo, larguras, explorer visivel — e preferencia do USUARIO,
// persistida em settings; zerar isso a cada troca de projeto seria perder a
// configuracao de quem esta trabalhando. Ele foi recebido aqui como propriedade
// e nunca usado ate 2026-09-02, quando a fiacao morta saiu (roadmap 30, etapa
// 10). Se algum dia o shell ganhar estado POR WORKSPACE, ele volta — e com
// chamada no `clear()`, nao so com a propriedade.
Item {
    id: root

    property var projectTree: null
    property var editorController: null
    property var jobsController: null
    property var searchController: null
    property var searchEverywhereController: null
    property var runtimeController: null
    property var runConfigController: null
    property var debugController: null
    property var gitController: null
    property var diagnosticsController: null
    property var bottomPanelHost: null

    visible: false

    function clear() {
        projectTree.clear();
        editorController.clear();
        jobsController.clear();
        searchController.clear();
        searchEverywhereController.clear();
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
