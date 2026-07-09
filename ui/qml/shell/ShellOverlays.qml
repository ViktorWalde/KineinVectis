import QtQuick
import KineinVectis

Item {
    id: root

    property real hostWidth: 0
    property real hostHeight: 0
    property var searchController: null
    property var projectTree: null
    property var editorController: null
    property var shellController: null

    anchors.fill: parent

    function resetSearchEverywhereAndFocus() {
        searchEverywhereDialog.resetAndFocus();
    }

    function openEntryRenameWithName(name) {
        entryRenameDialog.openWithName(name);
    }

    SearchEverywhereDialog {
        id: searchEverywhereDialog

        visible: root.searchController.everywhereVisible
        z: 90
        anchors.centerIn: parent
        resultsModel: root.searchController.everywhereModel
        resultCount: root.searchController.everywhereModel.count
        currentIndex: root.searchController.everywhereIndex
        loading: root.searchController.everywhereLoading
        truncated: root.searchController.everywhereTruncated
        errorText: root.searchController.everywhereError
        maxAvailableWidth: root.hostWidth - 80
        maxAvailableHeight: root.hostHeight - 120
        onQueryChanged: root.searchController.scheduleSearchEverywhere(
                            searchEverywhereDialog.currentQuery())
        onAcceptRequested: root.searchController.acceptSearchEverywhere()
        onDismissRequested: {
            root.searchController.everywhereVisible = false;
            root.editorController.focusEditor();
        }
        onMoveDownRequested: root.searchController.moveEverywhereDown()
        onMoveUpRequested: root.searchController.moveEverywhereUp()
        onResultHovered: function(index) {
            root.searchController.everywhereIndex = index;
        }
        onResultActivated: function(index) {
            root.searchController.everywhereIndex = index;
            root.searchController.acceptSearchEverywhere();
        }
    }

    ProjectEntryContextMenu {
        anchors.fill: parent
        visible: root.projectTree.entryMenuVisible
        z: 100
        menuX: root.projectTree.entryMenuX
        menuY: root.projectTree.entryMenuY
        onDismissRequested: root.projectTree.entryMenuVisible = false
        onRenameRequested: root.projectTree.openEntryRename()
        onDeleteRequested: root.projectTree.openEntryDelete()
    }

    ProjectEntryRenameDialog {
        id: entryRenameDialog

        anchors.fill: parent
        visible: root.projectTree.entryRenameVisible
        z: 101
        entryKind: root.projectTree.entryRenameKind
        entryDisplayPath: root.shellController.relativeToRoot(
                              root.projectTree.entryRenamePath)
        errorText: root.projectTree.entryRenameError
        maxAvailableWidth: root.hostWidth - 4 * Theme.spacingMedium
        onConfirmRequested: root.projectTree.confirmEntryRename(
                                entryRenameDialog.currentName())
        onCancelRequested: {
            root.projectTree.entryRenameVisible = false;
            root.editorController.focusEditor();
        }
    }

    ProjectEntryDeleteDialog {
        anchors.fill: parent
        visible: root.projectTree.entryDeleteVisible
        z: 102
        entryKind: root.projectTree.entryDeleteKind
        entryName: root.projectTree.entryDeleteName
        errorText: root.projectTree.entryDeleteError
        maxAvailableWidth: root.hostWidth - 4 * Theme.spacingMedium
        onConfirmRequested: root.projectTree.confirmEntryDelete()
        onCancelRequested: {
            root.projectTree.entryDeleteVisible = false;
            root.editorController.focusEditor();
        }
    }
}
