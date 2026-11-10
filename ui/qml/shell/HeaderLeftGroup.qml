import QtQuick
import KineinVectis

// O GRUPO DA ESQUERDA do cabecalho: projeto, git e os chips de contexto, na
// ordem que o usuario arrastou (0.3.9), e o "⋯" do contexto que nao coube.
// Saiu do TopHeaderBar em 2026-11-09, quando entrou o chip da configuracao
// do CMake (C1-23 do C-04): a barra poe o grupo e o executar; o grupo
// ordena e mede os seus. Os pontos dos menus saem no espaco do grupo.
Item {
    id: root

    property bool workspaceOpen: false
    property bool coreConnected: false
    property string workspaceName: ""
    property string systemLabel: ""
    property bool projectMenuOpen: false
    property string gitBranchLabel: ""
    property int gitAheadCount: 0
    property int gitBehindCount: 0
    property int gitChangeCount: 0
    property bool gitPanelActive: false
    property string toolchainSummary: ""
    property bool toolchainMenuOpen: false
    property string pythonSummary: ""
    property bool pythonMenuOpen: false
    // A configuracao ativa do CMake (C1-23): o chip abre o menu dela.
    property string configurationSummary: ""
    property string configurationTooltip: ""
    property bool configurationMenuOpen: false
    // Ate' onde o grupo pode ir: o executar, ou o fim da barra.
    property real rightLimit: 0
    property real spacing: Theme.spacingMedium

    signal openWorkspaceRequested()
    signal projectMenuRequested(real menuX, real menuY)
    signal gitPanelRequested()
    signal gitBranchMenuRequested()
    signal toolchainMenuRequested(real menuX, real menuY)
    signal pythonMenuRequested(real menuX, real menuY)
    signal configurationMenuRequested(real menuX, real menuY)
    signal contextOverflowRequested(real menuX, real menuY)
    signal widgetMoved(string key, int dropIndex, var visibleKeys)

    // A ordem que o usuario arrastou (0.3.9). Os widgets sao fixos (nao ha'
    // Repeater): o grupo da esquerda e' um Item e cada widget calcula o x
    // pela ordem — a soma dos visiveis antes dele. (Reanexar por um pai
    // invisivel deixava o chip que aparecia depois fora do grafo de cena, e
    // o stackBefore/stackAfter do C++ nao chega ao QML.)
    property var order: []
    readonly property ShellLayoutCodec codec: ShellLayoutCodec {}
    readonly property var widgetOrder: root.codec.ordered(["project", "git", "configuration",
                                                           "toolchain", "python"], root.order)

    function xOf(key) {
        const widgets = { project: projectWidget, git: gitWidget,
                          configuration: configurationWidget, toolchain: contextWidget,
                          python: pythonWidget };
        let x = 0;
        for (const other of root.widgetOrder) {
            if (other === key) return x;
            if (widgets[other].visible) x += widgets[other].width + root.spacing;
        }
        // O "⋯" (chave fora da ordem) e' sempre o ultimo.
        return x;
    }

    ReorderController {
        id: headerReorder

        container: root
        onMoved: function(key, dropIndex, visibleKeys) {
            root.widgetMoved(key, dropIndex, visibleKeys);
        }
    }

    // Contexto que existe mas nao coube: vai para o "⋯".
    readonly property bool toolchainHidden: root.workspaceOpen && root.toolchainSummary !== ""
                                            && !contextWidget.visible
    readonly property bool pythonHidden: root.workspaceOpen && root.pythonSummary !== ""
                                         && !pythonWidget.visible
    readonly property bool configurationHidden: root.workspaceOpen
                                                && root.configurationSummary !== ""
                                                && !configurationWidget.visible

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
        systemLabel: root.systemLabel
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

    // A configuracao ativa do CMake (C1-23): pega primeiro o que sobra ate'
    // o executar, porque e' curta e decide a pasta do build; a toolchain, o
    // que ela deixa. Reserva o lugar do "⋯" (32 px e o espaco).
    HeaderContextWidget {
        id: configurationWidget

        reorder: headerReorder
        reorderKey: "configuration"
        x: root.xOf("configuration")
        Behavior on x {
            NumberAnimation { duration: Theme.motionFast; easing.type: Theme.easingStandard }
        }
        opacity: headerReorder.opacityFor("configuration")

        anchors.verticalCenter: parent.verticalCenter
        summary: root.workspaceOpen ? root.configurationSummary : ""
        iconName: "layers"
        tooltipText: root.configurationTooltip
        menuOpen: root.configurationMenuOpen
        availableWidth: root.rightLimit - Theme.spacingLarge - root.x - projectWidget.width
                        - root.spacing - (gitWidget.visible ? gitWidget.width + root.spacing : 0)
                        - 32 - root.spacing
        onMenuRequested: function(menuX, menuY) {
            const pos = root.mapFromItem(configurationWidget, menuX, menuY);
            root.configurationMenuRequested(pos.x, pos.y);
        }
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
        availableWidth: configurationWidget.availableWidth
                        - (configurationWidget.visible ? configurationWidget.width + root.spacing : 0)
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
                        - (contextWidget.visible ? contextWidget.width + root.spacing : 0)
        onMenuRequested: function(menuX, menuY) {
            const pos = root.mapFromItem(pythonWidget, menuX, menuY);
            root.pythonMenuRequested(pos.x, pos.y);
        }
    }

    KvIconButton {
        id: overflowButton

        x: root.xOf("")

        anchors.verticalCenter: parent.verticalCenter
        visible: root.toolchainHidden || root.pythonHidden || root.configurationHidden
        iconName: "more"
        iconSize: 16
        focus: false
        focusOnClick: false
        tooltip: qsTr("Contexto que não coube")
        onClicked: {
            const pos = root.mapFromItem(overflowButton, 0, overflowButton.height);
            root.contextOverflowRequested(pos.x, pos.y);
        }

        // O aviso nao some com o chip: o Python ou a configuracao escondidos
        // com ⚠ acendem o ⋯.
        Rectangle {
            anchors.top: parent.top
            anchors.right: parent.right
            anchors.margins: 5
            visible: (root.pythonHidden && root.pythonSummary.indexOf("⚠") >= 0)
                     || (root.configurationHidden && root.configurationSummary.indexOf("⚠") >= 0)
            width: 7
            height: width
            radius: width / 2
            color: Theme.warningSoft
        }
    }
}
