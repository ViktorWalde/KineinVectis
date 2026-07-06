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
    property var usagesModel
    property int usageCount: 0
    property bool createDialogVisible: false
    property string createDialogKind: "file"
    property string createDialogParentDisplayPath: ""
    property string createDialogError: ""
    property bool renameDialogVisible: false
    property string renameError: ""

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
    signal completionActivated(int index)
    signal usageOpenRequested(string path, int line, int column)
    signal createConfirmRequested(string name)
    signal createCancelRequested()
    signal renameConfirmRequested(string name)
    signal renameCancelRequested()

    radius: Theme.radiusLarge
    color: Theme.background1
    border.color: Theme.borderSoft
    border.width: 1

    function focusCreateDialog() {
        createDialog.resetAndFocus();
    }

    function openRenameDialogWithName(name) {
        renameDialog.openWithName(name);
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

    EditorTextSurface {
        id: editor

        anchors.top: tabBar.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall
        hasOpenFile: root.currentTab >= 0
        emptyMessage: root.workspaceOpen
                      ? qsTr("Clique em um arquivo no explorer para abrir.")
                      : qsTr("Abra uma pasta para comecar "
                             + "(botao \"Abrir pasta...\" acima).")
        completionVisible: root.completionVisible
        usagesVisible: root.usagesVisible
        hoverVisible: root.hoverVisible
        onTextEdited: function(text) {
            root.textEdited(text);
        }
        onCompletionMoveRequested: function(delta) {
            root.completionMoveRequested(delta);
        }
        onCompletionAcceptRequested: root.completionAcceptRequested()
        onCompletionDismissRequested: root.completionDismissRequested()
        onUsagesDismissRequested: root.usagesDismissRequested()
        onHoverDismissRequested: root.hoverDismissRequested()
        onIndentRequested: root.indentRequested()
        onUnindentRequested: root.unindentRequested()
        onNewlineRequested: root.newlineRequested()
    }

    EditorHoverPopup {
        visible: root.hoverVisible && root.hoverText !== ""
        z: 20
        anchors.top: tabBar.bottom
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

    EditorUsagesPopup {
        visible: root.usagesVisible
        z: 25
        anchors.top: tabBar.bottom
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
}
