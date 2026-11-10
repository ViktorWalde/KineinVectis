import QtQuick
import KineinVectis

// A barra principal (Etapa 2, F1 do roadmaps/43, 2026-09-18): TRES widgets
// — projeto, git, executar — e nada mais sempre a vista. Ate' aqui eram
// doze controles (dois sistemas de build lado a lado, icones sem rotulo,
// "Target: host local", o rotulo do sistema a direita); a foto 02 do 43
// mediu. O que saiu nao sumiu: mora no menu "⋯" do widget de executar,
// com rotulo por sistema, e na barra de status (F2).
Rectangle {
    id: root

    property bool workspaceOpen: false
    property bool coreConnected: false
    property bool building: false
    property bool testing: false
    property bool analyzing: false
    property bool running: false
    property bool runPending: false
    property bool runStopping: false
    property bool debugging: false
    property string workspaceName: ""
    property string workspaceKind: ""
    property string projectFrameworkLabel: ""
    property var workspaceBuildSystems: []
    property string activeConfigId: ""
    property string activeConfigName: ""
    property bool configMenuOpen: false
    property bool projectMenuOpen: false
    property bool actionsMenuOpen: false
    property string gitBranchLabel: ""
    property int gitAheadCount: 0
    property int gitBehindCount: 0
    property int gitChangeCount: 0
    property bool gitPanelActive: false
    // O contexto efetivo (0.3.8 F3): a toolchain que o build usa.
    property string toolchainSummary: ""
    property bool toolchainMenuOpen: false
    property string pythonSummary: ""
    property bool pythonMenuOpen: false
    // A configuracao ativa do CMake (C1-23 do C-04): o chip abre o menu dela.
    property string configurationSummary: ""
    property string configurationTooltip: ""
    property bool configurationMenuOpen: false

    signal openWorkspaceRequested()
    signal projectMenuRequested(real menuX, real menuY)
    signal actionsMenuRequested(real menuX, real menuY)
    signal configMenuRequested(real menuX, real menuY)
    signal gitPanelRequested()
    signal gitBranchMenuRequested()
    signal toolchainMenuRequested(real menuX, real menuY)
    signal pythonMenuRequested(real menuX, real menuY)
    signal configurationMenuRequested(real menuX, real menuY)
    signal contextOverflowRequested(real menuX, real menuY)
    signal widgetMoved(string key, int dropIndex, var visibleKeys)

    // A ordem que o usuario arrastou (0.3.9); quem ordena e' o grupo.
    property var order: []

    function xOf(key) {
        return leftWidgets.xOf(key);
    }

    // Contexto que existe mas nao coube: vai para o "⋯".
    readonly property bool toolchainHidden: leftWidgets.toolchainHidden
    readonly property bool pythonHidden: leftWidgets.pythonHidden
    readonly property bool configurationHidden: leftWidgets.configurationHidden
    signal runRequested()
    signal stopRunRequested()
    signal debugRequested()
    signal stopDebugRequested()

    function hasBuildSystem(buildSystem) {
        const systems = workspaceBuildSystems !== undefined
                && workspaceBuildSystems !== null ? workspaceBuildSystems : [];
        return systems.indexOf(buildSystem) >= 0;
    }

    function workspaceSystemLabel() {
        if (projectFrameworkLabel !== "") return projectFrameworkLabel;
        const cargo = hasBuildSystem("cargo");
        const cmake = hasBuildSystem("cmake");
        if (cargo && cmake) return "Cargo + CMake";
        if (cargo) return "Cargo";
        if (cmake) return "CMake";
        if (hasBuildSystem("python")) return "Python";
        if (hasBuildSystem("make")) return "Make";
        return workspaceKind === "" ? "" : workspaceKind;
    }

    // Sobre a moldura, na mesma linha do ☰ (o host a posiciona). Sem fundo:
    // o vazio entre os widgets e' a area de arrastar a janela, que fica
    // embaixo, na AppMenuBar.
    height: 38
    color: "transparent"

    HeaderLeftGroup {
        id: leftWidgets

        anchors.verticalCenter: parent.verticalCenter
        anchors.left: parent.left
        anchors.leftMargin: Theme.spacingMedium
        rightLimit: runWidget.visible ? runWidget.x : root.width
        workspaceOpen: root.workspaceOpen
        coreConnected: root.coreConnected
        workspaceName: root.workspaceName
        systemLabel: root.workspaceSystemLabel()
        projectMenuOpen: root.projectMenuOpen
        gitBranchLabel: root.gitBranchLabel
        gitAheadCount: root.gitAheadCount
        gitBehindCount: root.gitBehindCount
        gitChangeCount: root.gitChangeCount
        gitPanelActive: root.gitPanelActive
        toolchainSummary: root.toolchainSummary
        toolchainMenuOpen: root.toolchainMenuOpen
        pythonSummary: root.pythonSummary
        pythonMenuOpen: root.pythonMenuOpen
        configurationSummary: root.configurationSummary
        configurationTooltip: root.configurationTooltip
        configurationMenuOpen: root.configurationMenuOpen
        order: root.order
        onOpenWorkspaceRequested: root.openWorkspaceRequested()
        onGitPanelRequested: root.gitPanelRequested()
        onGitBranchMenuRequested: root.gitBranchMenuRequested()
        onWidgetMoved: (key, dropIndex, visibleKeys) => root.widgetMoved(key, dropIndex, visibleKeys)
        // Os pontos dos menus voltam ao espaco da barra, como antes da divisao.
        onProjectMenuRequested: function(menuX, menuY) {
            const pos = root.mapFromItem(leftWidgets, menuX, menuY);
            root.projectMenuRequested(pos.x, pos.y);
        }
        onToolchainMenuRequested: function(menuX, menuY) {
            const pos = root.mapFromItem(leftWidgets, menuX, menuY);
            root.toolchainMenuRequested(pos.x, pos.y);
        }
        onPythonMenuRequested: function(menuX, menuY) {
            const pos = root.mapFromItem(leftWidgets, menuX, menuY);
            root.pythonMenuRequested(pos.x, pos.y);
        }
        onConfigurationMenuRequested: function(menuX, menuY) {
            const pos = root.mapFromItem(leftWidgets, menuX, menuY);
            root.configurationMenuRequested(pos.x, pos.y);
        }
        onContextOverflowRequested: function(menuX, menuY) {
            const pos = root.mapFromItem(leftWidgets, menuX, menuY);
            root.contextOverflowRequested(pos.x, pos.y);
        }
    }


    // Executar fica no CENTRO-DIREITA, como na referencia: e' o gesto mais
    // frequente depois de digitar, e o olho o acha sempre no mesmo lugar.
    HeaderRunWidget {
        id: runWidget

        anchors.right: parent.right
        anchors.rightMargin: Theme.spacingMedium
        anchors.verticalCenter: parent.verticalCenter
        visible: root.workspaceOpen
        coreConnected: root.coreConnected
        building: root.building
        testing: root.testing
        analyzing: root.analyzing
        running: root.running
        runPending: root.runPending
        runStopping: root.runStopping
        debugging: root.debugging
        workspaceKind: root.workspaceKind
        activeConfigId: root.activeConfigId
        activeConfigName: root.activeConfigName
        configMenuOpen: root.configMenuOpen
        actionsMenuOpen: root.actionsMenuOpen
        onConfigMenuRequested: function(menuX, menuY) {
            const pos = root.mapFromItem(runWidget, menuX, menuY);
            root.configMenuRequested(pos.x, pos.y);
        }
        onActionsMenuRequested: function(menuX, menuY) {
            const pos = root.mapFromItem(runWidget, menuX, menuY);
            root.actionsMenuRequested(pos.x, pos.y);
        }
        onRunRequested: root.runRequested()
        onStopRunRequested: root.stopRunRequested()
        onDebugRequested: root.debugRequested()
        onStopDebugRequested: root.stopDebugRequested()
    }
}
