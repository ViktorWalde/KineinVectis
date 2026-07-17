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
    property var runtimeController: null
    property var runConfigController: null
    property var gitController: null
    property var settingsController: null
    property bool aboutVisible: false
    property bool manualVisible: false
    property bool appMenuVisible: false
    property real appMenuX: 0
    property real appMenuY: 0
    property var appMenuItems: []

    signal appMenuActionRequested(string action)
    signal appMenuDismissed()

    anchors.fill: parent

    function resetSearchEverywhereAndFocus() {
        searchEverywhereDialog.resetAndFocus();
    }

    function openEntryRenameWithName(name) {
        entryRenameDialog.openWithName(name);
    }

    function openRunConfigDialogWith(name, command) {
        runConfigDialog.openWith(name, command);
    }

    function openAboutDialog() {
        aboutVisible = true;
    }

    function openManualDialog() {
        manualVisible = true;
    }

    function openAppMenu(x, y, items) {
        appMenuX = x;
        appMenuY = y;
        appMenuItems = items;
        appMenuVisible = items.length > 0;
    }

    function closeAppMenu() {
        if (!appMenuVisible) {
            return;
        }
        appMenuVisible = false;
        appMenuItems = [];
        appMenuDismissed();
    }

    AppMenuPopup {
        anchors.fill: parent
        visible: root.appMenuVisible
        z: 103
        menuX: root.appMenuX
        menuY: root.appMenuY
        items: root.appMenuItems
        onDismissRequested: root.closeAppMenu()
        onActionRequested: function(action) {
            root.appMenuVisible = false;
            root.appMenuItems = [];
            root.appMenuActionRequested(action);
            root.appMenuDismissed();
        }
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
        titleText: root.searchController.everywhereTitle
        recentMode: root.searchController.recentMode
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

    GitDiffDialog {
        anchors.fill: parent
        visible: root.gitController.diffDialogVisible
        z: 93
        title: root.gitController.diffDialogCommitLabel !== ""
               ? root.gitController.diffDialogCommitLabel
               : root.shellController.relativeToRoot(
                     root.gitController.diffDialogPath)
        emptyText: root.gitController.diffDialogCommitLabel !== ""
                   ? qsTr("Commit sem diff textual (merge?).")
                   : ""
        diffText: root.gitController.diffDialogText
        tracked: root.gitController.diffDialogTracked
        loading: root.gitController.diffDialogLoading
        maxAvailableWidth: root.hostWidth - 4 * Theme.spacingMedium
        maxAvailableHeight: root.hostHeight - 4 * Theme.spacingMedium
        onDismissRequested: root.gitController.closeDiffDialog()
    }

    SettingsDialog {
        anchors.fill: parent
        visible: root.settingsController.dialogVisible
        z: 97
        formatOnSave: root.settingsController.formatOnSave
        editorFontSize: root.settingsController.editorFontSize
        autoClosePairs: root.settingsController.autoClosePairs
        rigorProfile: root.settingsController.rigorProfile
        maxAvailableWidth: root.hostWidth - 4 * Theme.spacingMedium
        maxAvailableHeight: root.hostHeight - 4 * Theme.spacingMedium
        onDismissRequested: root.settingsController.closeDialog()
        onSettingChanged: function(key, value) {
            const values = {};
            values[key] = value;
            root.settingsController.setGlobal(values);
        }
    }

    AboutDialog {
        anchors.fill: parent
        visible: root.aboutVisible
        z: 98
        maxAvailableWidth: root.hostWidth - 4 * Theme.spacingMedium
        maxAvailableHeight: root.hostHeight - 4 * Theme.spacingMedium
        onDismissRequested: root.aboutVisible = false
    }

    DocumentationDialog {
        anchors.fill: parent
        visible: root.manualVisible
        z: 104
        maxAvailableWidth: root.hostWidth - 4 * Theme.spacingMedium
        maxAvailableHeight: root.hostHeight - 4 * Theme.spacingMedium
        onDismissRequested: root.manualVisible = false
    }

    GitDiscardDialog {
        anchors.fill: parent
        visible: root.gitController.discardDialogVisible
        z: 96
        entryPath: root.gitController.discardDialogPath
        maxAvailableWidth: root.hostWidth - 4 * Theme.spacingMedium
        onConfirmRequested: root.gitController.confirmDiscard()
        onCancelRequested: root.gitController.cancelDiscard()
    }

    RunConfigMenu {
        anchors.fill: parent
        visible: root.runConfigController.configMenuVisible
        z: 94
        menuX: root.runConfigController.configMenuX
        menuY: root.runConfigController.configMenuY
        configsModel: root.runConfigController.runConfigsModel
        activeConfigId: root.runConfigController.activeConfigId
        onDismissRequested: root.runConfigController.closeConfigMenu()
        onConfigChosen: function(id) {
            root.runConfigController.chooseConfig(id);
        }
        onNewRequested: root.runConfigController.openNewConfigDialog()
        onEditRequested: root.runConfigController.openEditConfigDialog()
        onDeleteRequested: root.runConfigController.deleteActiveConfig()
    }

    RunConfigDialog {
        id: runConfigDialog

        visible: root.runConfigController.runConfigDialogVisible
        z: 95
        anchors.centerIn: parent
        editing: root.runConfigController.editingConfigId !== ""
        maxAvailableWidth: root.hostWidth - 4 * Theme.spacingMedium
        onConfirmRequested: root.runConfigController.confirmConfigDialog(
                                runConfigDialog.currentName(),
                                runConfigDialog.currentCommand())
        onCancelRequested: root.runConfigController.cancelConfigDialog()
    }

    ProjectEntryContextMenu {
        anchors.fill: parent
        visible: root.projectTree.entryMenuVisible
        z: 100
        menuX: root.projectTree.entryMenuX
        menuY: root.projectTree.entryMenuY
        runnableScript: root.projectTree.entryMenuRunnable
        onDismissRequested: root.projectTree.entryMenuVisible = false
        onCreateFileRequested: root.projectTree.openEntryCreate("file")
        onCreateDirectoryRequested: root.projectTree.openEntryCreate("directory")
        onRunScriptRequested: root.projectTree.runEntryScript()
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
