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

        visible: searchController.everywhereVisible
        z: 90
        anchors.centerIn: parent
        resultsModel: searchController.everywhereModel
        resultCount: searchController.everywhereModel.count
        currentIndex: searchController.everywhereIndex
        loading: searchController.everywhereLoading
        truncated: searchController.everywhereTruncated
        errorText: searchController.everywhereError
        maxAvailableWidth: root.hostWidth - 80
        maxAvailableHeight: root.hostHeight - 120
        onQueryChanged: searchController.scheduleSearchEverywhere(
                            searchEverywhereDialog.currentQuery())
        onAcceptRequested: searchController.acceptSearchEverywhere()
        onDismissRequested: {
            searchController.everywhereVisible = false;
            editorController.focusEditor();
        }
        onMoveDownRequested: searchController.moveEverywhereDown()
        onMoveUpRequested: searchController.moveEverywhereUp()
        onResultHovered: function(index) {
            searchController.everywhereIndex = index;
        }
        onResultActivated: function(index) {
            searchController.everywhereIndex = index;
            searchController.acceptSearchEverywhere();
        }
    }

    ProjectEntryContextMenu {
        anchors.fill: parent
        visible: projectTree.entryMenuVisible
        z: 100
        menuX: projectTree.entryMenuX
        menuY: projectTree.entryMenuY
        onDismissRequested: projectTree.entryMenuVisible = false
        onRenameRequested: projectTree.openEntryRename()
        onDeleteRequested: projectTree.openEntryDelete()
    }

    ProjectEntryRenameDialog {
        id: entryRenameDialog

        anchors.fill: parent
        visible: projectTree.entryRenameVisible
        z: 101
        entryKind: projectTree.entryRenameKind
        entryDisplayPath: shellController.relativeToRoot(projectTree.entryRenamePath)
        errorText: projectTree.entryRenameError
        maxAvailableWidth: root.hostWidth - 4 * Theme.spacingMedium
        onConfirmRequested: projectTree.confirmEntryRename(
                                entryRenameDialog.currentName())
        onCancelRequested: {
            projectTree.entryRenameVisible = false;
            editorController.focusEditor();
        }
    }

    ProjectEntryDeleteDialog {
        anchors.fill: parent
        visible: projectTree.entryDeleteVisible
        z: 102
        entryKind: projectTree.entryDeleteKind
        entryName: projectTree.entryDeleteName
        errorText: projectTree.entryDeleteError
        maxAvailableWidth: root.hostWidth - 4 * Theme.spacingMedium
        onConfirmRequested: projectTree.confirmEntryDelete()
        onCancelRequested: {
            projectTree.entryDeleteVisible = false;
            editorController.focusEditor();
        }
    }
}
