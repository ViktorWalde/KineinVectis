import QtQuick
import KineinVectis

Item {
    id: root

    property real hostWidth: 0
    property real hostHeight: 0
    property var searchEverywhereController: null
    property var projectTree: null
    property var editorController: null
    property var shellController: null
    property var runtimeController: null
    property var runConfigController: null
    property var gitController: null
    property var settingsController: null
    property var configActionController: null
    property var libraryController: null
    property var dataSourceController: null
    property var remoteController: null
    property var grafanaController: null
    property var embeddedController: null
    property var setupController: null
    property var containerController: null
    property var toolchainController: null
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
        projectOverlays.openEntryRenameWithName(name);
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

        visible: root.searchEverywhereController.everywhereVisible
        z: 90
        anchors.centerIn: parent
        resultsModel: root.searchEverywhereController.everywhereModel
        resultCount: root.searchEverywhereController.everywhereModel.count
        currentIndex: root.searchEverywhereController.everywhereIndex
        loading: root.searchEverywhereController.everywhereLoading
        truncated: root.searchEverywhereController.everywhereTruncated
        errorText: root.searchEverywhereController.everywhereError
        titleText: root.searchEverywhereController.everywhereTitle
        recentMode: root.searchEverywhereController.recentMode
        maxAvailableWidth: root.hostWidth - 80
        maxAvailableHeight: root.hostHeight - 120
        onQueryChanged: root.searchEverywhereController.scheduleSearchEverywhere(
                            searchEverywhereDialog.currentQuery())
        onAcceptRequested: root.searchEverywhereController.acceptSearchEverywhere()
        onDismissRequested: {
            root.searchEverywhereController.everywhereVisible = false;
            root.editorController.focusEditor();
        }
        onMoveDownRequested: root.searchEverywhereController.moveEverywhereDown()
        onMoveUpRequested: root.searchEverywhereController.moveEverywhereUp()
        onResultHovered: function(index) {
            root.searchEverywhereController.everywhereIndex = index;
        }
        onResultActivated: function(index) {
            root.searchEverywhereController.everywhereIndex = index;
            root.searchEverywhereController.acceptSearchEverywhere();
        }
    }

    GitDiffDialog {
        anchors.fill: parent
        visible: root.gitController.diffDialogVisible
        z: 93
        title: root.shellController.relativeToRoot(root.gitController.diffDialogPath)
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
        autoSave: root.settingsController.autoSave
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

    ToolchainMenu {
        anchors.fill: parent
        visible: root.toolchainController.menuVisible
        z: 95
        controller: root.toolchainController
        menuX: root.toolchainController.menuX
        menuY: root.toolchainController.menuY
        onDismissRequested: root.toolchainController.closeMenu()
    }

    ConfigActionsDialog {
        anchors.fill: parent
        visible: root.configActionController.dialogVisible
        z: 99
        controller: root.configActionController
        libraryController: root.libraryController
        maxAvailableWidth: root.hostWidth - 4 * Theme.spacingMedium
        maxAvailableHeight: root.hostHeight - 4 * Theme.spacingMedium
        onDismissRequested: root.configActionController.closeDialog()
        // Passo de biblioteca vira Configuration Action, com a previa e o
        // consentimento dela — o mesmo caminho de antes, agora sem trocar de
        // painel no meio.
        onApplyStepRequested: (actionId, params) =>
            root.configActionController.openWith(actionId, params)
    }

    // Os seis paineis de AMBIENTE DO PROJETO moram em arquivo proprio.
    //
    // O corte e' por RESPONSABILIDADE, e nao por tamanho: os seis tem a mesma
    // forma — moldura de dialogo sobre um controller com `panelVisible`, e o
    // mesmo ciclo abrir/fechar — e sao o mesmo agrupamento que o menu ja'
    // chama de "Ambiente do projeto". O resto deste arquivo sao overlays de
    // natureza diferente: menus, dialogos modais e o popup do menu principal.
    ShellEnvironmentOverlays {
        anchors.fill: parent
        hostWidth: root.hostWidth
        hostHeight: root.hostHeight
        libraryController: root.libraryController
        dataSourceController: root.dataSourceController
        remoteController: root.remoteController
        grafanaController: root.grafanaController
        embeddedController: root.embeddedController
        toolchainController: root.toolchainController
        setupController: root.setupController
        containerController: root.containerController
        configActionController: root.configActionController
        runtimeController: root.runtimeController
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

    ShellProjectOverlays {
        id: projectOverlays

        anchors.fill: parent
        hostWidth: root.hostWidth
        projectTree: root.projectTree
        shellController: root.shellController
        editorController: root.editorController
    }
}
