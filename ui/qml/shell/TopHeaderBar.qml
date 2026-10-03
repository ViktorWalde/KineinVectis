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
    // O contexto efetivo (0.3.8 F3): a toolchain que o build usa.
    property string toolchainSummary: ""
    property bool toolchainMenuOpen: false
    property string pythonSummary: ""
    property bool pythonMenuOpen: false

    signal openWorkspaceRequested()
    signal projectMenuRequested(real menuX, real menuY)
    signal actionsMenuRequested(real menuX, real menuY)
    signal configMenuRequested(real menuX, real menuY)
    signal gitPanelRequested()
    signal gitBranchMenuRequested()
    signal toolchainMenuRequested(real menuX, real menuY)
    signal pythonMenuRequested(real menuX, real menuY)
    signal contextOverflowRequested(real menuX, real menuY)
    signal widgetMoved(string key, int dropIndex, var visibleKeys)

    // A ordem que o usuario arrastou (0.3.9). Os widgets sao fixos (nao ha'
    // Repeater): o grupo da esquerda e' um Item e cada widget calcula o x
    // pela ordem — a soma dos visiveis antes dele. (Reanexar por um pai
    // invisivel deixava o chip que aparecia depois fora do grafo de cena, e
    // o stackBefore/stackAfter do C++ nao chega ao QML.)
    property var order: []
    readonly property ShellLayoutCodec codec: ShellLayoutCodec {}
    readonly property var widgetOrder: root.codec.ordered(["project", "git", "toolchain", "python"],
                                                          root.order)

    function xOf(key) {
        const widgets = { project: projectWidget, git: gitWidget,
                          toolchain: contextWidget, python: pythonWidget };
        let x = 0;
        for (const other of root.widgetOrder) {
            if (other === key) return x;
            if (widgets[other].visible) x += widgets[other].width + leftWidgets.spacing;
        }
        // O "⋯" (chave fora da ordem) e' sempre o ultimo.
        return x;
    }

    ReorderController {
        id: headerReorder

        container: leftWidgets
        onMoved: function(key, dropIndex, visibleKeys) {
            root.widgetMoved(key, dropIndex, visibleKeys);
        }
    }

    // Contexto que existe mas nao coube: vai para o "⋯".
    readonly property bool toolchainHidden: root.workspaceOpen && root.toolchainSummary !== ""
                                            && !contextWidget.visible
    readonly property bool pythonHidden: root.workspaceOpen && root.pythonSummary !== ""
                                         && !pythonWidget.visible
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

    // Sobre a moldura, na mesma linha do ☰ (o host a posiciona). Sem fundo:
    // o vazio entre os widgets e' a area de arrastar a janela, que fica
    // embaixo, na AppMenuBar.
    height: 38
    color: "transparent"

    Item {
        id: leftWidgets

        property real spacing: Theme.spacingMedium

        anchors.verticalCenter: parent.verticalCenter
        anchors.left: parent.left
        anchors.leftMargin: Theme.spacingMedium
        height: 32
        width: overflowButton.x + (overflowButton.visible ? overflowButton.width : 0)

        HeaderProjectWidget {
            id: projectWidget

            reorder: headerReorder
            reorderKey: "project"
            x: root.xOf("project")
            Behavior on x {
                NumberAnimation { duration: Theme.motionFast; easing.type: Theme.easingStandard }
            }
            opacity: headerReorder.opacityFor("project")
            // Na tela de boas-vindas o "Abrir projeto" do topo repetia o cartao
            // (decisao do autor, 2026-10-03): o widget so' existe com projeto.
            visible: root.workspaceOpen

            anchors.verticalCenter: parent.verticalCenter
            workspaceOpen: root.workspaceOpen
            workspaceName: root.workspaceName
            systemLabel: root.workspaceSystemLabel()
            coreConnected: root.coreConnected
            menuOpen: root.projectMenuOpen
            onOpenWorkspaceRequested: root.openWorkspaceRequested()
            onMenuRequested: function(menuX, menuY) {
                const pos = root.mapFromItem(projectWidget, menuX, menuY);
                root.projectMenuRequested(pos.x, pos.y);
            }
        }

        HeaderGitWidget {
            id: gitWidget

            reorder: headerReorder
            reorderKey: "git"
            x: root.xOf("git")
            Behavior on x {
                NumberAnimation { duration: Theme.motionFast; easing.type: Theme.easingStandard }
            }
            opacity: headerReorder.opacityFor("git")

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

        HeaderContextWidget {
            id: contextWidget

            reorder: headerReorder
            reorderKey: "toolchain"
            x: root.xOf("toolchain")
            Behavior on x {
                NumberAnimation { duration: Theme.motionFast; easing.type: Theme.easingStandard }
            }
            opacity: headerReorder.opacityFor("toolchain")

            anchors.verticalCenter: parent.verticalCenter
            summary: root.workspaceOpen ? root.toolchainSummary : ""
            tooltipText: qsTr("Toolchain efetiva do build — clique para trocar")
            menuOpen: root.toolchainMenuOpen
            // Prioridade por largura: a toolchain pega primeiro o que sobra
            // ate' o executar; o Python, o resto. Sem lugar, o chip sai.
            // Reserva o lugar do "⋯" (32 px e o espaco) para ele nunca empurrar.
            availableWidth: (runWidget.visible ? runWidget.x : root.width) - Theme.spacingLarge
                            - leftWidgets.x - projectWidget.width - leftWidgets.spacing
                            - (gitWidget.visible ? gitWidget.width + leftWidgets.spacing : 0)
                            - 32 - leftWidgets.spacing
            onMenuRequested: function(menuX, menuY) {
                const pos = root.mapFromItem(contextWidget, menuX, menuY);
                root.toolchainMenuRequested(pos.x, pos.y);
            }
        }

        HeaderContextWidget {
            id: pythonWidget

            reorder: headerReorder
            reorderKey: "python"
            x: root.xOf("python")
            Behavior on x {
                NumberAnimation { duration: Theme.motionFast; easing.type: Theme.easingStandard }
            }
            opacity: headerReorder.opacityFor("python")

            anchors.verticalCenter: parent.verticalCenter
            summary: root.workspaceOpen ? root.pythonSummary : ""
            iconName: "tree-file-python"
            tooltipText: qsTr("Python do projeto — interpretador e ambiente")
            menuOpen: root.pythonMenuOpen
            availableWidth: contextWidget.availableWidth
                            - (contextWidget.visible ? contextWidget.width + leftWidgets.spacing : 0)
            onMenuRequested: function(menuX, menuY) {
                const pos = root.mapFromItem(pythonWidget, menuX, menuY);
                root.pythonMenuRequested(pos.x, pos.y);
            }
        }

        KvIconButton {
            id: overflowButton

            x: root.xOf("")

            anchors.verticalCenter: parent.verticalCenter
            visible: root.toolchainHidden || root.pythonHidden
            iconName: "more"
            iconSize: 16
            focus: false
            focusOnClick: false
            tooltip: qsTr("Contexto que não coube")
            onClicked: {
                const pos = root.mapFromItem(overflowButton, 0, overflowButton.height);
                root.contextOverflowRequested(pos.x, pos.y);
            }

            // O aviso nao some com o chip: o Python escondido com ⚠ acende o ⋯.
            Rectangle {
                anchors.top: parent.top
                anchors.right: parent.right
                anchors.margins: 5
                visible: root.pythonHidden && root.pythonSummary.indexOf("⚠") >= 0
                width: 7
                height: width
                radius: width / 2
                color: Theme.warningSoft
            }
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
