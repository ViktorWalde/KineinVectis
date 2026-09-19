import QtQuick

Item {
    id: root

    property string workspaceRoot: ""
    property string workspaceKind: ""
    property var workspaceBuildSystems: []
    property string homeDir: ""
    property int toolsCount: 0
    property bool showBottomPanel: false
    property string bottomTab: "logs"
    property bool showExplorer: true
    property real viewportWidth: 1280
    property real viewportHeight: 720
    property bool layoutLoaded: false
    property bool persistedLayout: false
    // Dimensoes padrao e limites de DocsPublic/especificacoes/sistema-de-layout.md
    // §6.4; persistencia de layout entra com Settings (M4).
    property real explorerWidth: 280
    property real contextWidth: 360
    property real bottomPanelHeight: 260
    property real outlineWidth: 220
    property bool outlineCollapsed: false
    // O trilho lateral com rotulos (F1 modo expandido); persistido no layout.
    property bool railExpanded: false
    // O slot a esquerda do trilho e' UM (E3-3, roadmaps/44 §4.1): o
    // explorer OU a janela do Git, como a referencia alterna Project/Commit.
    property string leftWindow: "explorer"
    readonly property bool effectiveShowExplorer: showExplorer && leftWindow === "explorer"
    readonly property bool gitWindowVisible: showExplorer && leftWindow === "git"

    signal folderOpenRequested(string path)
    signal toolsDetectionRequested()
    signal layoutSaveRequested(var values)

    visible: false

    Timer {
        id: layoutSaveTimer

        interval: 250
        repeat: false
        onTriggered: root.layoutSaveRequested({
            explorerWidth: Math.round(root.explorerWidth),
            contextWidth: Math.round(root.contextWidth),
            bottomPanelHeight: Math.round(root.bottomPanelHeight),
            outlineWidth: Math.round(root.outlineWidth),
            outlineCollapsed: root.outlineCollapsed,
            railExpanded: root.railExpanded
        })
    }

    function clamp(value, minimum, maximum) {
        return Math.max(minimum, Math.min(maximum, value));
    }

    function applySettings(settingsController) {
        // O modo do trilho e' preferencia por si: vale mesmo sem o resto do layout salvo.
        railExpanded = settingsController.railExpanded === true;
        persistedLayout = settingsController.hasPersistedLayout();
        if (persistedLayout) {
            explorerWidth = clamp(settingsController.explorerWidth, 220, 420);
            contextWidth = clamp(settingsController.contextWidth, 300, 480);
            bottomPanelHeight = clamp(settingsController.bottomPanelHeight,
                                      160, 480);
            outlineWidth = clamp(settingsController.outlineWidth, 160, 420);
            outlineCollapsed = settingsController.outlineCollapsed;
        } else {
            applyAutomaticLayout();
        }
        layoutLoaded = true;
    }

    function updateViewport(width, height) {
        viewportWidth = width;
        viewportHeight = height;
        if (layoutLoaded && !persistedLayout) {
            applyAutomaticLayout();
        }
    }

    function applyAutomaticLayout() {
        explorerWidth = clamp(viewportWidth * 0.22, 220, 300);
        contextWidth = clamp(viewportWidth * 0.28, 300, 380);
        bottomPanelHeight = clamp(viewportHeight * 0.32, 180, 300);
        outlineWidth = clamp(viewportWidth * 0.18, 180, 260);
        // A aba Simbolos nasce RECOLHIDA (a alca fica; Alt+7 ou clique
        // abre) e nao muda sozinha por largura — o autor perdeu a aba
        // assim (E3-2, roadmaps/44).
        outlineCollapsed = true;
    }

    function persistLayoutSoon() {
        persistedLayout = true;
        layoutSaveTimer.restart();
    }

    function relativeToRoot(path) {
        if (workspaceRoot !== "" && path.indexOf(workspaceRoot + "/") === 0) {
            return path.substring(workspaceRoot.length + 1);
        }
        return path;
    }

    // "git" NAO e' mais aba de baixo: e' a janela a esquerda (E3-3). Quem
    // pedia a aba (paleta, menu Exibir, cabecalho, trilho) continua
    // chamando showTab/toggleBottomTab/tabActive — este e' o ponto de corte.
    function showTab(tab) {
        if (tab === "git") {
            showGitWindow();
            return;
        }
        bottomTab = tab;
        showBottomPanel = true;
    }

    // A aba de baixo `tab` esta' VISIVEL agora: um dono para a derivacao
    // que a barra principal (F1) e o trilho perguntam.
    function tabActive(tab) {
        if (tab === "git") {
            return gitWindowVisible;
        }
        return showBottomPanel && bottomTab === tab;
    }

    function toggleBottomTab(tab) {
        if (tab === "git") {
            toggleGitWindow();
            return;
        }
        if (tabActive(tab)) {
            showBottomPanel = false;
            return;
        }
        showTab(tab);
        if (tab === "tools" && toolsCount === 0) {
            toolsDetectionRequested();
        }
    }

    function toggleRail() {
        railExpanded = !railExpanded;
        persistLayoutSoon();
    }

    function resizeExplorer(delta) {
        explorerWidth = Math.max(220, Math.min(420, explorerWidth + delta));
        persistLayoutSoon();
    }

    function resizeContext(delta) {
        contextWidth = Math.max(300, Math.min(480, contextWidth + delta));
        persistLayoutSoon();
    }

    function resizeBottomPanel(delta) {
        bottomPanelHeight = Math.max(160, Math.min(480, bottomPanelHeight + delta));
        persistLayoutSoon();
    }

    function resizeOutline(delta) {
        outlineWidth = Math.max(160, Math.min(420, outlineWidth + delta));
        persistLayoutSoon();
    }

    function resetOutlineWidth() {
        outlineWidth = 220;
        persistLayoutSoon();
    }

    function toggleOutline() {
        outlineCollapsed = !outlineCollapsed;
        persistLayoutSoon();
    }

    // Alt+7 / paleta `index.symbols`: a aba Simbolos abre e o campo ganha
    // o foco (E3-2). O foco e' do host; daqui sai o pedido.
    signal symbolsFocusRequested(string query)

    function openSymbols(query) {
        outlineCollapsed = false;
        persistLayoutSoon();
        symbolsFocusRequested(query === undefined ? "" : query);
    }

    // O icone do que esta' aberto fecha o slot; o do outro traz o outro.
    function toggleExplorer() {
        if (leftWindow !== "explorer") {
            leftWindow = "explorer";
            showExplorer = true;
            return;
        }
        showExplorer = !showExplorer;
    }

    function toggleGitWindow() {
        if (leftWindow !== "git") {
            showGitWindow();
            return;
        }
        showExplorer = !showExplorer;
    }

    function showGitWindow() {
        leftWindow = "git";
        showExplorer = true;
    }

    function requestOpenFolder() {
        folderOpenRequested(workspaceRoot !== "" ? workspaceRoot : homeDir);
    }

    function kindLabel(kind, buildSystems) {
        const systems = buildSystems !== undefined && buildSystems !== null
                ? buildSystems : workspaceBuildSystems;
        if (systems.indexOf("cargo") >= 0 && systems.indexOf("cmake") >= 0) {
            return "Cargo + CMake";
        }
        const labels = {
            rustCargo: "Rust/Cargo",
            cmake: "CMake",
            maven: "Maven",
            gradle: "Gradle",
            python: "Python",
            make: "Make",
            platformIo: "PlatformIO",
            unknown: qsTr("Projeto")
        };
        return labels[kind] !== undefined ? labels[kind] : kind;
    }
}
