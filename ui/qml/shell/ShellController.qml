import QtQuick
import KineinVectis

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
    // TAMANHO PREFERIDO x TAMANHO EXIBIDO (roadmap 53 §4.4, decisao do autor
    // de 2026-10-01). O preferido e' o que a pessoa escolheu e o que se grava;
    // o exibido e' o preferido dentro dos limites de AGORA: o minimo que o
    // conteudo declara (o rodape do Git, por exemplo) e o maximo que deixa o
    // editor com `editorMinimumWidth`. A janela menor so' limita o exibido —
    // maximizar devolve o tamanho escolhido. Ate' aqui os limites eram
    // constantes (220–420), e o rodape do Git estourava a 1024 px (F0).
    property real explorerPreferredWidth: 280
    property real bottomPreferredHeight: 260
    property real outlineWidth: 220
    // O minimo do painel da esquerda, DECLARADO por quem esta' nele
    // (ShellLeftWindowHost); o host liga. 220 e' o piso do explorer.
    property real leftMinimumWidth: 220
    // O que o trilho ocupa (o host informa, como o viewport).
    property real railWidth: 0
    // O que o editor nunca perde. 480 e' o mesmo numero que o EditorPane ja'
    // usava para so' abrir a Estrutura com espaco (`outlineWidth + 480`).
    readonly property real editorMinimumWidth: 480
    readonly property real editorMinimumHeight: 160
    // `viewportWidth`/`Height` sao o espaco do host (updateViewport): o trilho,
    // os vaos e o editor minimo saem dele; o resto e' o teto do painel.
    readonly property real leftMaximumWidth: viewportWidth - railWidth - 2 * Theme.panelGap
                                             - editorMinimumWidth
    readonly property real bottomMaximumHeight: viewportHeight - Theme.panelGap
                                                - editorMinimumHeight
    readonly property real explorerWidth: panelSize(explorerPreferredWidth, leftMinimumWidth,
                                                    leftMaximumWidth)
    readonly property real bottomPanelHeight: panelSize(bottomPreferredHeight, 160,
                                                        bottomMaximumHeight)
    property bool outlineCollapsed: false
    // O trilho lateral com rotulos (F1 modo expandido); persistido no layout.
    property bool railExpanded: false
    // O slot a esquerda do trilho e' UM (E3-3, roadmaps/44 §4.1): o
    // explorer OU a janela do Git, como a referencia alterna Project/Commit.
    property string leftWindow: "explorer"
    // As janelas que o slot da esquerda conhece (o codec valida contra esta).
    readonly property var leftWindows: ["explorer", "git"]
    // O trilho por areas (0.3.7 F1, 53 §4.2): o que o USUARIO decidiu. O que
    // o core sabe (fatos) nao mora aqui. Vai no `layout`, por workspace.
    property var railState: ({ pinned: [], unpinned: [], hidden: [] })
    property var bottomPinned: []
    readonly property bool effectiveShowExplorer: showExplorer && leftWindow === "explorer"
    readonly property bool gitWindowVisible: showExplorer && leftWindow === "git"

    // `intent`: "open" (abrir uma pasta) ou "createProject" (o seletor ja'
    // no modo de criar, com a escolha de linguagem). Uma porta, duas intencoes.
    signal folderOpenRequested(string path, string intent)
    signal toolsDetectionRequested()
    // "Exibir > Areas da IDE..." e a paleta: o host abre o painel de areas.
    signal areasPanelRequested()
    // `scope`: "workspace" (o layout do projeto aberto) ou "global" (o
    // padrao de quem ainda nao tem layout, e as preferencias do usuario).
    signal layoutSaveRequested(string scope, var values)

    // O ultimo layout gravado, em texto: o eco do `settings.set` o devolve, e
    // reaplica-lo no meio de um arrasto puxaria o painel de volta.
    property string savedLayoutText: ""
    property bool applyingLayout: false

    // O que a pessoa abre e fecha tambem e' layout (R5: "volta como estava").
    onLeftWindowChanged: layoutStateChanged()
    onShowExplorerChanged: layoutStateChanged()
    onShowBottomPanelChanged: layoutStateChanged()
    onBottomTabChanged: layoutStateChanged()

    // Trocar de projeto: nada do anterior pode ser gravado no novo antes de
    // o layout dele chegar (o `settings.get` da troca chama applySettings).
    onWorkspaceRootChanged: {
        layoutSaveTimer.stop();
        layoutLoaded = false;
    }

    function layoutStateChanged() {
        if (layoutLoaded && !applyingLayout) {
            persistLayoutSoon();
        }
    }

    visible: false

    Timer {
        id: layoutSaveTimer

        interval: 250
        repeat: false
        onTriggered: {
            const layout = root.layoutSnapshot();
            root.savedLayoutText = JSON.stringify(layout);
            root.layoutSaveRequested(root.workspaceRoot !== "" ? "workspace" : "global",
                                     { layout: layout });
        }
    }

    function clamp(value, minimum, maximum) {
        return Math.max(minimum, Math.min(maximum, value));
    }

    // O tamanho exibido: o preferido entre o minimo e o maximo de agora. Se
    // os dois brigam (janela estreita demais), o MINIMO vence: o conteudo nao
    // quebra, e quem cede e' o editor (a janela nunca fica abaixo de 800).
    function panelSize(preferred, minimum, maximum) {
        return Math.max(minimum, Math.min(preferred, maximum));
    }

    // O retrato do layout e as listas do usuario: funcoes puras do codec.
    readonly property ShellLayoutCodec codec: ShellLayoutCodec {}

    function layoutSnapshot() {
        return codec.snapshot(root);
    }

    // Aplica um layout schema 1 (o core so' entrega os que entende).
    function applyLayout(layout) {
        const values = codec.decode(layout, root);
        for (const key in values) {
            root[key] = values[key];
        }
    }

    function pinArea(id) { setRail(codec.withMembership(railState, id, true, false, false)); }
    function unpinArea(id) { setRail(codec.withMembership(railState, id, false, true, false)); }
    function hideArea(id) { setRail(codec.withMembership(railState, id, false, false, true)); }
    function restoreRail() { setRail(codec.emptyRail()); }
    // Tira a area de todas as listas: volta ao padrao de fabrica dela.
    function resetArea(id) { setRail(codec.withMembership(railState, id, false, false, false)); }

    function setRail(state) {
        railState = state;
        persistLayoutSoon();
    }

    // As abas de baixo que o usuario fixou (0.3.7 F2): aparecem sempre.
    function setBottomPinned(tab, pinned) {
        bottomPinned = codec.withItem(bottomPinned, tab, pinned);
        persistLayoutSoon();
    }

    function applySettings(settingsController) {
        // O modo do trilho e' preferencia por si: vale mesmo sem o resto do layout salvo.
        railExpanded = settingsController.railExpanded === true;
        const layout = settingsController.layout;
        if (layout !== null && layout !== undefined) {
            // O eco do que acabamos de gravar nao e' um layout novo.
            if (JSON.stringify(layout) !== savedLayoutText) {
                applyingLayout = true;
                applyLayout(layout);
                applyingLayout = false;
                savedLayoutText = JSON.stringify(layout);
            }
            persistedLayout = true;
        } else if (settingsController.hasPersistedLayout()) {
            // Os campos de antes do `layout` (um ciclo de migracao, 53 §4.4).
            persistedLayout = true;
            explorerPreferredWidth = clamp(settingsController.explorerWidth, 220, 420);
            bottomPreferredHeight = clamp(settingsController.bottomPanelHeight, 160, 480);
            outlineWidth = clamp(settingsController.outlineWidth, 160, 420);
            outlineCollapsed = settingsController.outlineCollapsed;
        } else {
            persistedLayout = false;
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

    // O host informa o que muda os limites: a largura do trilho (compacto ou
    // expandido) e o minimo que o conteudo da esquerda declara.
    function updatePanelLimits(currentRailWidth, leftMinimum) {
        railWidth = currentRailWidth;
        leftMinimumWidth = Math.max(220, leftMinimum);
    }

    function applyAutomaticLayout() {
        explorerPreferredWidth = clamp(viewportWidth * 0.22, 220, 300);
        bottomPreferredHeight = clamp(viewportHeight * 0.32, 180, 300);
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
        if (tab === "areas") {
            areasPanelRequested();
            return;
        }
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

    // O trilho e' preferencia do USUARIO, nao do projeto: vai ao global.
    function toggleRail() {
        railExpanded = !railExpanded;
        layoutSaveRequested("global", { railExpanded: railExpanded });
    }

    // Arrastar parte do que se ve e para nos limites de agora: o preferido
    // passa a ser o que ficou na tela.
    function resizeExplorer(delta) {
        explorerPreferredWidth = panelSize(explorerWidth + delta, leftMinimumWidth, leftMaximumWidth);
        persistLayoutSoon();
    }

    function resizeBottomPanel(delta) {
        bottomPreferredHeight = panelSize(bottomPanelHeight + delta, 160, bottomMaximumHeight);
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
        requestFolder("open");
    }

    // Quem entende a intencao e' o seletor (FolderPickerDialog.openWith); o
    // shell so' leva o pedido com o ponto de partida de sempre.
    function requestFolder(intent) {
        folderOpenRequested(workspaceRoot !== "" ? workspaceRoot : homeDir, intent);
    }

    function kindLabel(kind, buildSystems) {
        const systems = buildSystems !== undefined && buildSystems !== null
                ? buildSystems : workspaceBuildSystems;
        if (systems.indexOf("cargo") >= 0 && systems.indexOf("cmake") >= 0) {
            return "Cargo + CMake";
        }
        if (kind === "unknown") {
            return qsTr("Projeto");
        }
        const label = ProjectKindNames.label(kind);
        return label !== "" ? label : kind;
    }
}
