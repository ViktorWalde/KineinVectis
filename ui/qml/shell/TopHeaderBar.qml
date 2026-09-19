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
    property bool debugging: false
    property string workspaceName: ""
    property string workspaceKind: ""
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

    signal openWorkspaceRequested()
    signal projectMenuRequested(real menuX, real menuY)
    signal actionsMenuRequested(real menuX, real menuY)
    signal configMenuRequested(real menuX, real menuY)
    signal gitPanelRequested()
    signal gitBranchMenuRequested()
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
        const cargo = hasBuildSystem("cargo");
        const cmake = hasBuildSystem("cmake");
        if (cargo && cmake) return "Cargo + CMake";
        if (cargo) return "Cargo";
        if (cmake) return "CMake";
        if (hasBuildSystem("python")) return "Python";
        if (hasBuildSystem("make")) return "Make";
        return workspaceKind === "" ? "" : workspaceKind;
    }

    height: 44
    color: Theme.background1
    border.color: Theme.borderSoft
    border.width: 1

    Row {
        anchors.verticalCenter: parent.verticalCenter
        anchors.left: parent.left
        anchors.leftMargin: Theme.spacingMedium
        spacing: Theme.spacingMedium

        HeaderProjectWidget {
            id: projeto

            anchors.verticalCenter: parent.verticalCenter
            workspaceOpen: root.workspaceOpen
            workspaceName: root.workspaceName
            systemLabel: root.workspaceSystemLabel()
            coreConnected: root.coreConnected
            menuOpen: root.projectMenuOpen
            onOpenWorkspaceRequested: root.openWorkspaceRequested()
            onMenuRequested: function(menuX, menuY) {
                const pos = root.mapFromItem(projeto, menuX, menuY);
                root.projectMenuRequested(pos.x, pos.y);
            }
        }

        HeaderGitWidget {
            anchors.verticalCenter: parent.verticalCenter
            visible: root.workspaceOpen && root.gitBranchLabel !== ""
            branchLabel: root.gitBranchLabel
            aheadCount: root.gitAheadCount
            behindCount: root.gitBehindCount
            changeCount: root.gitChangeCount
            panelActive: root.gitPanelActive
            onPanelRequested: root.gitPanelRequested()
            onBranchMenuRequested: root.gitBranchMenuRequested()
        }
    }

    // Executar fica no CENTRO-DIREITA, como na referencia: e' o gesto mais
    // frequente depois de digitar, e o olho o acha sempre no mesmo lugar.
    HeaderRunWidget {
        id: executar

        anchors.right: parent.right
        anchors.rightMargin: Theme.spacingMedium
        anchors.verticalCenter: parent.verticalCenter
        visible: root.workspaceOpen
        coreConnected: root.coreConnected
        building: root.building
        testing: root.testing
        analyzing: root.analyzing
        running: root.running
        debugging: root.debugging
        workspaceKind: root.workspaceKind
        activeConfigId: root.activeConfigId
        activeConfigName: root.activeConfigName
        configMenuOpen: root.configMenuOpen
        actionsMenuOpen: root.actionsMenuOpen
        onConfigMenuRequested: function(menuX, menuY) {
            const pos = root.mapFromItem(executar, menuX, menuY);
            root.configMenuRequested(pos.x, pos.y);
        }
        onActionsMenuRequested: function(menuX, menuY) {
            const pos = root.mapFromItem(executar, menuX, menuY);
            root.actionsMenuRequested(pos.x, pos.y);
        }
        onRunRequested: root.runRequested()
        onStopRunRequested: root.stopRunRequested()
        onDebugRequested: root.debugRequested()
        onStopDebugRequested: root.stopDebugRequested()
    }
}
