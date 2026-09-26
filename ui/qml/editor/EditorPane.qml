pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

Rectangle {
    id: root

    property alias editorSurface: editor
    property var filesModel
    property int fileCount: 0
    property int currentTab: -1
    property bool workspaceOpen: false
    property bool completionVisible: false
    property bool usagesVisible: false
    property bool hoverVisible: false
    property bool actionsVisible: false
    property var breakpointLines: []
    property int executionLine: 0
    // C4: caminho relativo do arquivo atual, "src/lsp/manager.rs" → segmentos.
    property string breadcrumbPath: ""
    property var diffLineKinds: ({})
    property int diffRevision: 0
    property var coverageLineKinds: ({})
    property int coverageRevision: 0
    property bool blameActive: false
    property var blameLineAnnotations: ({})
    property int blameRevision: 0
    // T6: diagnósticos do arquivo ativo (sublinhado + gutter).
    property var diagnosticSpans: []
    property var diagnosticByLine: ({})
    property int diagnosticRevision: 0
    // M4.1: auto-close de pares (setting).
    property bool autoCloseEnabled: true
    property bool externalConflict: false
    property bool externalDeleted: false
    property string externalMessage: ""
    property string watchError: ""
    property var outlineItems: []
    property real outlineWidth: 220
    property bool outlineCollapsed: false
    // A aba Simbolos (E3-2): a busca no projeto vale sem arquivo aberto,
    // entao a aba existe sempre que ha' projeto; a estrutura, quando ha'.
    property var symbols: null
    readonly property bool outlineAvailable: outlineItems.length > 0 || workspaceOpen
    readonly property bool outlineExpanded: outlineAvailable
                                             && !outlineCollapsed
                                             && width >= outlineWidth + 480

    function focusSymbols(query) {
        outlineSide.focusSearch(query);
    }

    // A PREVIA DE MARKDOWN (V5/M1). O modo e' do documento; esta tela so'
    // desenha o que ele diz. `markdownAvailable` liga a barra de modo — um
    // `.py` nao ganha um botao "Preview" que nao faz nada.
    property bool markdownAvailable: false
    // "edit" | "preview" | "side" — quem decide e' o MarkdownPreviewController.
    property string previewMode: "edit"
    property real previewWidth: 420
    property string currentFilePath: ""
    property int currentDocId: 0
    property string workspaceRoot: ""
    readonly property bool sideBySide: markdownAvailable && previewMode === "side"
    readonly property bool previewOnly: markdownAvailable && previewMode === "preview"
    readonly property bool previewing: sideBySide || previewOnly

    signal previewModeSelected(string mode)
    signal previewResizeRequested(real delta)
    signal previewLocalFileRequested(string path)
    signal previewWebUrlRequested(string url)

    signal gutterLineClicked(int line)
    signal codeActionsRequested(int line)
    signal tabSelected(int docId)
    signal tabCloseRequested(int docId)
    signal textEdited(string text)
    signal completionMoveRequested(int delta)
    signal completionAcceptRequested()
    signal completionDismissRequested()
    signal usagesDismissRequested()
    signal hoverDismissRequested()
    signal indentRequested()
    signal unindentRequested()
    signal newlineRequested()
    signal closerBraceRequested()
    signal smartHomeRequested(bool extendSelection)
    signal actionsMoveRequested(int delta)
    signal actionsAcceptRequested()
    signal actionsDismissRequested()
    signal externalReloadRequested()
    signal externalKeepLocalRequested()
    signal watchErrorDismissRequested()
    signal outlineOpenRequested(int line, int column)
    signal outlineResizeRequested(real delta)
    signal outlineResetRequested()
    signal outlineToggleRequested()

    // Onde acaba o cabecalho (abas + trilha + faixa de conflito). O host de
    // overlay ancora nisso o que flutua no topo; ver ShellEditorOverlayHost.
    readonly property real overlayTop: externalBanner.y + externalBanner.height

    radius: Theme.radiusLarge
    color: Theme.background1
    border.color: Theme.borderSoft
    border.width: 1

    EditorTabsBar {
        id: tabBar

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall
        filesModel: root.filesModel
        fileCount: root.fileCount
        currentIndex: root.currentTab
        onTabSelected: function(docId) {
            root.tabSelected(docId);
        }
        onTabCloseRequested: function(docId) {
            root.tabCloseRequested(docId);
        }
    }

    EditorBreadcrumbs {
        id: breadcrumbsBar

        anchors.top: tabBar.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.leftMargin: Theme.spacingMedium
        anchors.rightMargin: Theme.spacingSmall
        visible: root.currentTab >= 0 && root.breadcrumbPath !== ""
        path: root.breadcrumbPath
    }

    EditorExternalChangeBanner {
        id: externalBanner

        anchors.top: breadcrumbsBar.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.leftMargin: Theme.spacingSmall
        anchors.rightMargin: Theme.spacingSmall
        anchors.topMargin: visible ? Theme.spacingXSmall : 0
        active: root.externalConflict || root.watchError !== ""
        deleted: root.externalDeleted
        watcherFailure: !root.externalConflict && root.watchError !== ""
        message: root.externalConflict ? root.externalMessage
                 : qsTr("O monitor de arquivos falhou: %1. Saves continuam protegidos contra conflito.")
                   .arg(root.watchError)
        onReloadRequested: root.externalReloadRequested()
        onKeepLocalRequested: root.externalKeepLocalRequested()
        onDismissRequested: root.watchErrorDismissRequested()
    }

    MarkdownModeBar {
        id: modeBar

        anchors.top: externalBanner.bottom
        anchors.right: parent.right
        anchors.rightMargin: Theme.spacingSmall
        anchors.topMargin: visible ? Theme.spacingXSmall : 0
        visible: root.markdownAvailable
        mode: root.previewMode
        onModeSelected: function(mode) {
            root.previewModeSelected(mode);
        }
    }

    MarkdownPreviewPane {
        id: preview

        anchors.top: modeBar.visible ? modeBar.bottom : externalBanner.bottom
        anchors.bottom: parent.bottom
        // SEM ancora a' esquerda, e com LARGURA explicita. A primeira versao
        // tentava `anchors.left: sideBySide ? undefined : parent.left`, e a foto
        // mostrou a previa cobrindo o editor: desancorar por ternario nao
        // funciona: a ancora fica onde estava.
        anchors.right: outlineSide.left
        anchors.rightMargin: Theme.spacingSmall
        anchors.topMargin: Theme.spacingSmall
        anchors.bottomMargin: Theme.spacingSmall
        width: root.sideBySide
               ? root.previewWidth
               : Math.max(0, outlineSide.x - 2 * Theme.spacingSmall)
        visible: root.previewing
        // O BUFFER, e nao o disco: a previa acompanha o que ainda nao foi
        // salvo (§3.3). Sem previa aberta o conteudo fica vazio, porque
        // renderizar o que ninguem esta' vendo e' trabalho jogado fora (§7).
        content: root.previewing ? editor.text : ""
        documentPath: root.currentFilePath
        workspaceRoot: root.workspaceRoot
        docId: root.currentDocId
        onLocalFileRequested: function(path) {
            root.previewLocalFileRequested(path);
        }
        onWebUrlRequested: function(url) {
            root.previewWebUrlRequested(url);
        }
    }

    PanelSplitter {
        id: previewSplitter

        visible: root.sideBySide
        x: preview.x - Theme.panelGap
        width: Theme.panelGap
        anchors.top: preview.top
        anchors.bottom: preview.bottom
        onDragged: function(delta) {
            root.previewResizeRequested(-delta);
        }
        onResetRequested: root.previewResizeRequested(0)
    }

    EditorTextSurface {
        id: editor

        visible: !root.previewOnly
        anchors.top: modeBar.visible ? modeBar.bottom : externalBanner.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: root.sideBySide ? previewSplitter.left : outlineSide.left
        anchors.rightMargin: Theme.spacingSmall
        anchors.margins: Theme.spacingSmall
        hasOpenFile: root.currentTab >= 0
        breakpointLines: root.breakpointLines
        executionLine: root.executionLine
        diffLineKinds: root.diffLineKinds
        diffRevision: root.diffRevision
        coverageLineKinds: root.coverageLineKinds
        coverageRevision: root.coverageRevision
        blameActive: root.blameActive
        blameLineAnnotations: root.blameLineAnnotations
        blameRevision: root.blameRevision
        diagnosticSpans: root.diagnosticSpans
        diagnosticByLine: root.diagnosticByLine
        diagnosticRevision: root.diagnosticRevision
        autoCloseEnabled: root.autoCloseEnabled
        onGutterLineClicked: function(line) {
            root.gutterLineClicked(line);
        }
        onCodeActionsRequested: function(line) {
            root.codeActionsRequested(line);
        }
        emptyMessage: root.workspaceOpen
                      ? qsTr("Clique em um arquivo no explorer para abrir.")
                      : qsTr("Abra uma pasta para comecar "
                             + "(botao \"Abrir pasta...\" acima).")
        completionVisible: root.completionVisible
        usagesVisible: root.usagesVisible
        hoverVisible: root.hoverVisible
        actionsVisible: root.actionsVisible
        onTextEdited: function(text) {
            root.textEdited(text);
        }
        onCompletionMoveRequested: function(delta) {
            root.completionMoveRequested(delta);
        }
        onCompletionAcceptRequested: root.completionAcceptRequested()
        onCompletionDismissRequested: root.completionDismissRequested()
        onActionsMoveRequested: function(delta) {
            root.actionsMoveRequested(delta);
        }
        onActionsAcceptRequested: root.actionsAcceptRequested()
        onActionsDismissRequested: root.actionsDismissRequested()
        onUsagesDismissRequested: root.usagesDismissRequested()
        onHoverDismissRequested: root.hoverDismissRequested()
        onIndentRequested: root.indentRequested()
        onUnindentRequested: root.unindentRequested()
        onNewlineRequested: root.newlineRequested()
        onCloserBraceRequested: root.closerBraceRequested()
        onSmartHomeRequested: function(extendSelection) {
            root.smartHomeRequested(extendSelection);
        }
    }

    EditorOutlineSide {
        id: outlineSide

        anchors.top: externalBanner.bottom
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        panelWidth: root.outlineWidth
        expanded: root.outlineExpanded
        available: root.outlineAvailable
        items: root.outlineItems
        symbols: root.symbols
        onCollapseRequested: root.outlineToggleRequested()
        onOpenRequested: function(line, column) {
            root.outlineOpenRequested(line, column);
        }
        onResizeRequested: function(delta) {
            root.outlineResizeRequested(delta);
        }
        onResetRequested: root.outlineResetRequested()
    }

}
