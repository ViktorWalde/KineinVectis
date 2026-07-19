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
    property string hoverText: ""
    property var completionModel
    property int completionCount: 0
    property int completionIndex: 0
    property bool actionsVisible: false
    property var actionsModel
    property int actionCount: 0
    property int actionsIndex: 0
    property var usagesModel
    property int usageCount: 0
    property bool createDialogVisible: false
    property string createDialogKind: "file"
    property string createDialogParentDisplayPath: ""
    property string createDialogError: ""
    property bool renameDialogVisible: false
    property string renameError: ""
    property bool workspaceEditPreviewVisible: false
    property string workspaceEditTitle: ""
    property var workspaceEditFiles: []
    property int workspaceEditCount: 0
    property string workspaceEditError: ""
    property bool goToLineDialogVisible: false
    // D1b (docs/roadmaps/24): barra de Find/Replace do arquivo (Ctrl+F / Ctrl+H).
    property bool findBarVisible: false
    property bool findReplaceMode: false
    property string findQuery: ""
    property string findReplacement: ""
    property bool findCaseSensitive: false
    property bool findWholeWord: false
    property bool findUseRegex: false
    property bool findInvalidRegex: false
    property int findMatchCount: 0
    property int findCurrentDisplay: 0
    property var breakpointLines: []
    property int executionLine: 0
    // C4: caminho relativo do arquivo atual, "src/lsp/manager.rs" → segmentos.
    property string breadcrumbPath: ""
    property var diffLineKinds: ({})
    property int diffRevision: 0
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
    readonly property bool outlineExpanded: outlineItems.length > 0
                                             && !outlineCollapsed
                                             && width >= outlineWidth + 480

    signal gutterLineClicked(int line)
    signal tabSelected(int index)
    signal tabCloseRequested(int index)
    signal saveRequested()
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
    signal completionActivated(int index)
    signal actionsMoveRequested(int delta)
    signal actionsAcceptRequested()
    signal actionsDismissRequested()
    signal actionActivated(int index)
    signal usageOpenRequested(string path, int line, int column)
    signal createConfirmRequested(string name)
    signal createCancelRequested()
    signal renameConfirmRequested(string name)
    signal renameCancelRequested()
    signal workspaceEditApplyRequested()
    signal workspaceEditCancelRequested()
    signal goToLineConfirmRequested(string value)
    signal goToLineCancelRequested()
    signal findQueryEdited(string text)
    signal findReplacementEdited(string text)
    signal findNextRequested()
    signal findPreviousRequested()
    signal findReplaceRequested()
    signal findReplaceAllRequested()
    signal findCaseToggleRequested()
    signal findWholeWordToggleRequested()
    signal findRegexToggleRequested()
    signal findCloseRequested()
    signal externalReloadRequested()
    signal externalKeepLocalRequested()
    signal watchErrorDismissRequested()
    signal outlineOpenRequested(int line, int column)
    signal outlineResizeRequested(real delta)
    signal outlineResetRequested()
    signal outlineToggleRequested()

    // §4.2: regiao plana; separacao por divisor de 1px, arredondamento interno.
    color: Theme.background1

    function focusCreateDialog() {
        createDialog.resetAndFocus();
    }

    function openRenameDialogWithName(name) {
        renameDialog.openWithName(name);
    }

    function openGoToLineDialog(prefill) {
        goToLineDialog.openWithValue(prefill);
    }

    function focusFindBar() {
        findBar.focusQuery();
    }

    EditorTabsBar {
        id: tabBar

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall
        filesModel: root.filesModel
        fileCount: root.fileCount
        currentIndex: root.currentTab
        onTabSelected: function(index) {
            root.tabSelected(index);
        }
        onTabCloseRequested: function(index) {
            root.tabCloseRequested(index);
        }
        onSaveRequested: root.saveRequested()
    }

    Row {
        id: breadcrumbsBar

        anchors.top: tabBar.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.leftMargin: Theme.spacingMedium
        anchors.rightMargin: Theme.spacingSmall
        height: visible ? 18 : 0
        visible: root.currentTab >= 0 && root.breadcrumbPath !== ""
        spacing: Theme.spacingXSmall

        Repeater {
            model: root.breadcrumbPath.split("/")

            delegate: Row {
                id: breadcrumbSegment

                required property int index
                required property string modelData

                spacing: Theme.spacingXSmall

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: breadcrumbSegment.modelData
                    color: breadcrumbSegment.index
                           === root.breadcrumbPath.split("/").length - 1
                           ? Theme.textSecondary : Theme.textMuted
                    font.pixelSize: 11
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: breadcrumbSegment.index
                             < root.breadcrumbPath.split("/").length - 1
                    text: "›"
                    color: Theme.textMuted
                    font.pixelSize: 11
                }
            }
        }
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

    EditorTextSurface {
        id: editor

        anchors.top: externalBanner.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: outlineSplitter.visible ? outlineSplitter.left : parent.right
        anchors.rightMargin: Theme.spacingSmall
        anchors.margins: Theme.spacingSmall
        hasOpenFile: root.currentTab >= 0
        breakpointLines: root.breakpointLines
        executionLine: root.executionLine
        diffLineKinds: root.diffLineKinds
        diffRevision: root.diffRevision
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

    EditorOutlinePanel {
        id: outlinePanel

        anchors.top: externalBanner.bottom
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.rightMargin: Theme.spacingSmall
        anchors.bottomMargin: Theme.spacingSmall
        width: visible ? root.outlineWidth : 0
        visible: root.outlineExpanded
        items: root.outlineItems
        onCollapseRequested: root.outlineToggleRequested()
        onOpenRequested: function(line, column) {
            root.outlineOpenRequested(line, column);
        }
    }

    PanelSplitter {
        id: outlineSplitter

        visible: outlinePanel.visible
        x: outlinePanel.x - Theme.splitterGrip
        width: Theme.splitterGrip
        anchors.top: outlinePanel.top
        anchors.bottom: outlinePanel.bottom
        onDragged: function(delta) {
            root.outlineResizeRequested(-delta);
        }
        onResetRequested: root.outlineResetRequested()
    }

    Rectangle {
        id: collapsedOutlineHandle

        visible: root.outlineItems.length > 0 && !outlinePanel.visible
        anchors.right: parent.right
        anchors.rightMargin: Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        width: 28
        height: 96
        radius: Theme.radius
        color: outlineHandleMouse.containsMouse
               ? Theme.surfaceSelected : Theme.surface1
        border.color: outlineHandleMouse.containsMouse
                      ? Theme.accent : Theme.borderSoft
        border.width: 1
        z: 18

        KvIcon {
            anchors.top: parent.top
            anchors.topMargin: Theme.spacingSmall
            anchors.horizontalCenter: parent.horizontalCenter
            name: "project"
            size: 16
            active: outlineHandleMouse.containsMouse
        }

        Text {
            anchors.centerIn: parent
            anchors.verticalCenterOffset: 12
            text: qsTr("Estrutura")
            rotation: -90
            color: Theme.textSecondary
            font.pixelSize: 10
            font.bold: true
        }

        MouseArea {
            id: outlineHandleMouse

            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: root.outlineToggleRequested()
            onContainsMouseChanged: {
                if (containsMouse) {
                    TooltipController.showFor(collapsedOutlineHandle,
                                              qsTr("Expandir Estrutura"),
                                              "bottom");
                } else {
                    TooltipController.hideFor(collapsedOutlineHandle);
                }
            }
        }
    }

    // D1b: flutua sobre o editor no canto superior direito (VS Code), acima
    // do texto mas abaixo dos popups de completion/actions.
    EditorFindBar {
        id: findBar

        visible: root.findBarVisible && root.currentTab >= 0
        z: 22
        anchors.top: externalBanner.bottom
        anchors.right: parent.right
        anchors.topMargin: Theme.spacingSmall
        anchors.rightMargin: 2 * Theme.spacingSmall
        maxAvailableWidth: root.width - 4 * Theme.spacingSmall
        replaceMode: root.findReplaceMode
        query: root.findQuery
        replacement: root.findReplacement
        caseSensitive: root.findCaseSensitive
        wholeWord: root.findWholeWord
        useRegex: root.findUseRegex
        invalidRegex: root.findInvalidRegex
        matchCount: root.findMatchCount
        currentMatch: root.findCurrentDisplay
        onQueryEdited: function(text) {
            root.findQueryEdited(text);
        }
        onReplacementEdited: function(text) {
            root.findReplacementEdited(text);
        }
        onFindNextRequested: root.findNextRequested()
        onFindPreviousRequested: root.findPreviousRequested()
        onReplaceRequested: root.findReplaceRequested()
        onReplaceAllRequested: root.findReplaceAllRequested()
        onCaseToggleRequested: root.findCaseToggleRequested()
        onWholeWordToggleRequested: root.findWholeWordToggleRequested()
        onRegexToggleRequested: root.findRegexToggleRequested()
        onCloseRequested: root.findCloseRequested()
    }

    EditorHoverPopup {
        visible: root.hoverVisible && root.hoverText !== ""
        z: 20
        anchors.top: externalBanner.bottom
        anchors.right: parent.right
        anchors.topMargin: 2 * Theme.spacingSmall
        anchors.rightMargin: 2 * Theme.spacingSmall
        hoverText: root.hoverText
        maxAvailableWidth: root.width - 4 * Theme.spacingSmall
        onDismissRequested: root.hoverDismissRequested()
    }

    EditorCompletionPopup {
        visible: root.completionVisible && root.currentTab >= 0
        z: 30
        itemsModel: root.completionModel
        completionCount: root.completionCount
        currentIndex: root.completionIndex
        maxAvailableWidth: root.width - 4 * Theme.spacingSmall
        x: {
            const rect = editor.cursorRectangle;
            const point = editor.cursorPointIn(root);
            return Math.max(Theme.spacingSmall,
                            Math.min(point.x, root.width - width - Theme.spacingSmall));
        }
        y: {
            const rect = editor.cursorRectangle;
            const point = editor.cursorPointIn(root);
            const below = point.y + rect.height + 4;
            if (below + height > root.height - Theme.spacingSmall) {
                return Math.max(Theme.spacingSmall, point.y - height - 4);
            }
            return below;
        }
        onCompletionActivated: function(index) {
            root.completionActivated(index);
        }
    }

    EditorActionsPopup {
        visible: root.actionsVisible && root.currentTab >= 0
        z: 30
        itemsModel: root.actionsModel
        actionCount: root.actionCount
        currentIndex: root.actionsIndex
        maxAvailableWidth: root.width - 4 * Theme.spacingSmall
        x: {
            const point = editor.cursorPointIn(root);
            return Math.max(Theme.spacingSmall,
                            Math.min(point.x, root.width - width - Theme.spacingSmall));
        }
        y: {
            const rect = editor.cursorRectangle;
            const point = editor.cursorPointIn(root);
            const below = point.y + rect.height + 4;
            if (below + height > root.height - Theme.spacingSmall) {
                return Math.max(Theme.spacingSmall, point.y - height - 4);
            }
            return below;
        }
        onActionActivated: function(index) {
            root.actionActivated(index);
        }
        onDismissRequested: root.actionsDismissRequested()
    }

    EditorUsagesPopup {
        visible: root.usagesVisible
        z: 25
        anchors.top: externalBanner.bottom
        anchors.right: parent.right
        anchors.topMargin: 2 * Theme.spacingSmall
        anchors.rightMargin: 2 * Theme.spacingSmall
        itemsModel: root.usagesModel
        usageCount: root.usageCount
        maxAvailableWidth: root.width - 4 * Theme.spacingSmall
        onCloseRequested: root.usagesDismissRequested()
        onUsageOpenRequested: function(path, line, column) {
            root.usageOpenRequested(path, line, column);
        }
    }

    ProjectCreateDialog {
        id: createDialog

        visible: root.createDialogVisible
        z: 40
        anchors.centerIn: parent
        dialogKind: root.createDialogKind
        parentDisplayPath: root.createDialogParentDisplayPath
        errorText: root.createDialogError
        maxAvailableWidth: root.width - 4 * Theme.spacingSmall
        onConfirmRequested: root.createConfirmRequested(createDialog.currentName())
        onCancelRequested: root.createCancelRequested()
    }

    SymbolRenameDialog {
        id: renameDialog

        visible: root.renameDialogVisible
        z: 40
        anchors.centerIn: parent
        errorText: root.renameError
        maxAvailableWidth: root.width - 4 * Theme.spacingSmall
        onConfirmRequested: root.renameConfirmRequested(renameDialog.currentName())
        onCancelRequested: root.renameCancelRequested()
    }

    EditorWorkspaceEditPreviewDialog {
        visible: root.workspaceEditPreviewVisible
        z: 45
        anchors.centerIn: parent
        operationTitle: root.workspaceEditTitle
        files: root.workspaceEditFiles
        editCount: root.workspaceEditCount
        errorText: root.workspaceEditError
        maxAvailableWidth: root.width - 4 * Theme.spacingSmall
        maxAvailableHeight: root.height - 4 * Theme.spacingSmall
        onApplyRequested: root.workspaceEditApplyRequested()
        onCancelRequested: root.workspaceEditCancelRequested()
    }

    EditorGoToLineDialog {
        id: goToLineDialog

        visible: root.goToLineDialogVisible
        z: 40
        anchors.centerIn: parent
        maxAvailableWidth: root.width - 4 * Theme.spacingSmall
        onConfirmRequested: root.goToLineConfirmRequested(goToLineDialog.currentValue())
        onCancelRequested: root.goToLineCancelRequested()
    }
}
