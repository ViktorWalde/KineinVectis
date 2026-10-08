import QtQuick
import KineinVectis
Item {
    id: root

    property var shellController
    property var workspaceController
    property var projectHealthController
    property var projectTree
    property var editorController
    property var indexController: null
    property var jobsController
    property var activeJobController: null
    property var runtimeController
    property var debugController
    property var gitController
    property var coverageController
    property var diagnosticsController
    property var searchController
    property var recentWorkspacesController
    property var containerController
    property var grafanaController
    property var dataSourceController
    property var settingsController: null
    property var embeddedController: null
    property var remoteController: null
    property var toolchainController: null
    property alias editorSurface: editorPaneHost.editorSurface
    property bool workspaceOpen: false
    property string workspaceRoot: ""
    property string workspaceName: ""
    property string workspaceKind: ""
    property var workspaceBuildSystems: []
    property bool testing: false
    property bool terminalActive: false
    property bool running: false
    property var logLinesModel
    property var toolsList: []

    signal listDirRequested(string path)
    signal readFileRequested(string path)
    signal openWorkspacePathRequested(string path)
    signal closeWorkspaceRequested()
    signal toolsDetectionRequested()
    // A acao da faixa de saude vai inteira para o Main (scan, configure...).
    signal healthActionRequested(string target)
    signal problemNextStepRequested(string kind, string target, string file, int line, int column)
    signal createProjectRequested(string templateId)
    signal settingsRequested()
    // Menus do trilho e das abas (0.3.7): o Main abre o menu compartilhado.
    signal shellMenuRequested(real menuX, real menuY, var items)

    onWidthChanged: if (root.shellController) root.shellController.updateViewport(width, height)
    onHeightChanged: if (root.shellController) root.shellController.updateViewport(width, height)
    Component.onCompleted: {
        root.shellController.updateViewport(width, height);
        updatePanelLimits();
    }

    // Limites dos paineis (53 §4.4): o trilho e o minimo de cada slot.
    function updatePanelLimits() {
        root.shellController.updatePanelLimits(sideBar.width, explorerPanel.minimumWidth, explorerPanel.rightMinimumWidth);
    }

    function focusSearchInput() { bottomPanel.focusSearchInput(); }

    function focusSearchReplaceInput() { bottomPanel.focusSearchReplaceInput(); }

    function clearSearchInput() { bottomPanel.clearSearchInput(); }

    function focusTerminalInput() { bottomPanel.focusTerminalInput(); }

    function clearTerminalInput() { bottomPanel.clearTerminalInput(); }

    function focusCreateDialog() { editorPaneHost.focusCreateDialog(); }

    function openRenameDialogWithName(name) { editorPaneHost.openRenameDialogWithName(name); }

    function openGoToLineDialog(prefill) { editorPaneHost.openGoToLineDialog(prefill); }

    function focusFindBar() { editorPaneHost.focusFindBar(); }

    function focusSymbols(query) { editorPaneHost.focusSymbols(query); }

    // Ctrl+F6 / Ctrl+Shift+F6 (0.3.9 F4): o teclado de area em area.
    function cycleFocus(direction) { focusCycle.cycle(root.Window.activeFocusItem, direction); }
    Connections {
        target: root.shellController
        function onFocusCycleRequested(direction) { root.cycleFocus(direction); }
    }
    FocusCycle {
        id: focusCycle

        areas: [explorerPanel, editorPaneHost, rightDock, bottomPanel]
        editorArea: editorPaneHost
    }

    // O trilho e' dado: quem monta os overlays de ambiente le' esta lista.
    readonly property alias toolWindows: railToolWindows

    ShellLayout {
        anchors.fill: parent
        railItem: sideBar
        leftItem: explorerPanel
        centerItem: centerColumn
        bottomItem: bottomPanel
        rightItem: rightSide
        rightDockItem: rightDock

        // As entradas do trilho sao DADO (V3): uma entrada no `ToolWindows`.
        ToolWindows {
            id: railToolWindows

            shellController: root.shellController
            embeddedController: root.embeddedController
            dataSourceController: root.dataSourceController
            containerController: root.containerController
            grafanaController: root.grafanaController
            remoteController: root.remoteController
            toolchainController: root.toolchainController
            runtimeController: root.runtimeController
            workspaceOpen: root.workspaceOpen
            toolsList: root.toolsList
            projectHealthController: root.projectHealthController
        }

        SideRail {
            id: sideBar

            visible: root.workspaceOpen // boas-vindas sem trilhos (autor, 2026-10-03)
            height: parent.height
            expanded: root.shellController.railExpanded
            onWidthChanged: root.updatePanelLimits()
            onExpandedToggled: root.shellController.toggleRail()
            entries: railToolWindows.leftEntries
            order: root.shellController.savedOrder("rail")
            partnerReorder: rightSide.reorder
            onEntryMoved: (id, dropIndex, visibleIds) => root.shellController.moveInBar(
                              "rail", visibleIds, id, dropIndex)
            onEntryTransferred: (id, i, ids) => root.shellController.moveRailEntryToSide(id, "right", i, ids)
            onActivated: id => railToolWindows.activate(id)
            // "⋯ Mais" e o botao direito abrem o painel de areas.
            onContextMenuRequested: function(id, menuX, menuY) {
                const pos = mapToItem(root, menuX + Theme.spacingMedium, menuY);
                railAreas.openAt(pos.x, pos.y, id);
            }
            onMoreRequested: function(menuX, menuY) {
                const pos = mapToItem(root, menuX + Theme.spacingSmall, menuY);
                railAreas.openAt(pos.x, pos.y, "");
            }
        }

        ShellLeftWindowHost {
            id: explorerPanel

            width: visible ? root.shellController.explorerWidth : 0
            height: parent.height
            // Sem projeto, nada; nunca a mesma janela dos dois lados.
            visible: root.shellController.docks.slotVisible("left", root.workspaceOpen)
            rightSlot: rightDock
            shellController: root.shellController
            projectTree: root.projectTree
            gitController: root.gitController
            dataSourceController: root.dataSourceController
            editorController: root.editorController
            containerController: root.containerController
            remoteController: root.remoteController
            grafanaController: root.grafanaController
            settingsController: root.settingsController
            onMinimumWidthChanged: root.updatePanelLimits()
            onRightMinimumWidthChanged: root.updatePanelLimits()
            workspaceName: root.workspaceName
            workspaceRoot: root.workspaceRoot
            onListDirRequested: function(path) { root.listDirRequested(path); }
            onReadFileRequested: function(path) { root.readFileRequested(path); }
            onCloseWorkspaceRequested: root.closeWorkspaceRequested()
        }

        Column {
            id: centerColumn

            width: visible
                   ? Math.max(0, parent.width - (sideBar.visible ? sideBar.width + Theme.panelGap : 0)
                              - (rightSide.visible ? rightSide.width + Theme.panelGap : 0)
                              - (rightDock.visible ? rightDock.width + Theme.panelGap : 0)
                              - (explorerPanel.visible
                                 ? explorerPanel.width + Theme.panelGap : 0))
                   : 0
            height: parent.height
            spacing: Theme.panelGap

            ProjectHealthBanner {
                id: healthBanner

                width: parent.width
                active: root.projectHealthController.active
                status: root.projectHealthController.status
                message: root.projectHealthController.message
                actionLabel: root.projectHealthController.actionLabel
                onActionRequested: root.healthActionRequested(root.projectHealthController.actionTarget)
                onDismissRequested: root.projectHealthController.dismiss()
            }

            StartScreen {
                width: parent.width
                height: visible
                        ? parent.height - (bottomPanel.visible
                          ? bottomPanel.height + Theme.panelGap : 0) : 0
                visible: !root.workspaceOpen
                recentWorkspacesController: root.recentWorkspacesController
                urlDecoder: Clipboard
                animationEnabled: root.settingsController ? root.settingsController.welcomeAnimation : true
                onAnimationToggled: on => root.settingsController.setWelcomeAnimation(on)
                onOpenWorkspaceRequested: root.shellController.requestOpenFolder()
                onWorkspacePathDropped: path => root.openWorkspacePathRequested(path)
                onNewProjectRequested: function(templateId) {
                    root.createProjectRequested(templateId);
                }
                onSettingsRequested: root.settingsRequested()
            }

            ShellEditorHost {
                dataSourceController: root.dataSourceController
                id: editorPaneHost

                width: parent.width
                visible: root.workspaceOpen
                height: visible ? parent.height
                        - (healthBanner.visible
                        ? healthBanner.height + Theme.panelGap : 0)
                        - (bottomPanel.visible
                        ? bottomPanel.height + Theme.panelGap : 0) : 0

                workspaceOpen: root.workspaceOpen
                editorController: root.editorController
                indexController: root.indexController
                shellController: root.shellController
                debugController: root.debugController
                gitController: root.gitController
                coverageController: root.coverageController
                diagnosticsController: root.diagnosticsController
                projectTree: root.projectTree
            }

            BottomPanelHost {
                id: bottomPanel

                width: parent.width
                height: root.shellController.bottomPanelHeight
                pinnedTabs: root.shellController.bottomPinned
                tabOrder: root.shellController.savedOrder("bottom")
                onTabMoved: function(key, dropIndex, visibleKeys) {
                    root.shellController.moveInBar("bottom", visibleKeys, key, dropIndex);
                }
                onTabMenuRequested: function(key, menuX, menuY) {
                    const pos = mapToItem(root, menuX, menuY);
                    root.shellMenuRequested(pos.x, pos.y, bottomPanel.tabMenuItems(key));
                }
                open: root.shellController.showBottomPanel && root.workspaceOpen
                activeTab: root.shellController.bottomTab
                problemCount: root.jobsController.problemsModel.count
                problemErrors: root.jobsController.problemsModel.count >= 0 && root.jobsController.hasErrorProblems()
                gitController: root.gitController
                testsBadge: root.jobsController.testsBadge
                testsOk: root.jobsController.testsFailed === 0
                jobsRunning: root.activeJobController ? root.activeJobController.runningCount : 0
                buildOutputModel: root.jobsController.buildOutputModel
                jobsModel: root.jobsController.jobsModel
                jobsController: root.jobsController
                testing: root.testing
                problemsModel: root.jobsController.problemsModel
                workspaceRoot: root.workspaceRoot
                terminalRender: root.runtimeController.terminalRender
                runtimeController: root.runtimeController
                terminalActive: root.terminalActive
                workspaceAvailable: root.workspaceOpen
                debugController: root.debugController
                running: root.running
                terminalsModel: root.runtimeController.terminalsModel
                activeTerminalId: root.runtimeController.activeTerminalId
                runTerminalId: root.runtimeController.runTerminalId
                searchModel: root.searchController.searchModel
                searchCaseSensitive: root.searchController.caseSensitive
                searching: root.searchController.searching
                searchTruncated: root.searchController.searchTruncated
                searchReplaceMode: root.searchController.replaceMode
                searchReplacing: root.searchController.replacing
                searchReplaceError: root.searchController.replaceError
                searchReplaceSummary: root.searchController.replaceSummary
                logLinesModel: root.logLinesModel
                toolsList: root.workspaceController.toolsList
                onHideRequested: root.shellController.showBottomPanel = false
                onTabRequested: function(tab) {
                    // Terminal sem sessao abre uma (dono: RuntimeController).
                    if (tab === "terminal" && !root.shellController.tabActive("terminal")) {
                        root.runtimeController.openTerminalPanel();
                        return;
                    }
                    root.shellController.toggleBottomTab(tab);
                }
                onRefreshToolsRequested: root.toolsDetectionRequested()
                onProblemOpenRequested: function(file, line, column) {
                    root.editorController.openDiagnostic(file, line, column);
                }
                onProblemNextStepRequested: (kind, target, file, line, column) =>
                    root.problemNextStepRequested(kind, target, file, line, column)
                onTerminalOpenRequested: root.runtimeController.openTerminalPanel()
                onTerminalKeyPressed: function(data) {
                    root.runtimeController.sendTerminalKey(data);
                }
                onTerminalResizeRequested: function(cols, rows) {
                    root.runtimeController.resizeTerminal(cols, rows);
                }
                onTerminalScrollRequested: function(offset) {
                    root.runtimeController.scrollTerminal(offset);
                }
                onTerminalClearScrollbackRequested:
                    root.runtimeController.clearTerminalScrollback()
                onTerminalWheelRequested: function(col, row, lines, modifiers) {
                    root.runtimeController.wheelTerminal(col, row, lines,
                                                         modifiers);
                }
                onTerminalSelectRequested: function(id) {
                    root.runtimeController.selectTerminal(id);
                }
                onTerminalNewRequested: root.runtimeController.newTerminal()
                onTerminalCloseTabRequested: function(id) {
                    root.runtimeController.closeTerminal(id);
                }
                onSearchRequested: function(query) {
                    root.searchController.runSearch(query);
                }
                onSearchCaseSensitivityToggleRequested: function(query) {
                    root.searchController.toggleCaseAndRun(query);
                }
                onSearchResultOpenRequested: function(path, line, column) {
                    root.editorController.openDiagnostic(path, line, column);
                }
                onSearchReplaceRequested: function(query, replacement) {
                    root.searchController.runReplace(query, replacement);
                }
            }
        }

        ShellRightDock {
            id: rightDock

            height: parent.height
            shellController: root.shellController
            workspaceOpen: root.workspaceOpen
            windowHost: explorerPanel
        }

        RightSideRail {
            id: rightSide

            visible: root.workspaceOpen
            height: parent.height
            shellController: root.shellController
            railEntries: railToolWindows
            partnerReorder: sideBar.reorder
        }
    }

    RailAreasPopup {
        id: railAreas

        anchors.fill: parent
        z: 200
        toolWindows: railToolWindows
        shellController: root.shellController
    }

    Connections {
        target: root.shellController

        function onAreasPanelRequested() {
            railAreas.openAt(sideBar.width + 2 * Theme.panelGap, Theme.spacingSmall, "");
        }
    }

    // Alcas de redimensionamento sobre os vaos (limites no ShellController).
    PanelSplitter {
        visible: explorerPanel.visible
        x: explorerPanel.x + explorerPanel.width
        width: Theme.panelGap
        height: parent.height
        onDragged: delta => root.shellController.resizeExplorer(delta)
    }

    PanelSplitter {
        visible: bottomPanel.visible
        horizontal: true
        x: centerColumn.x
        y: centerColumn.y + bottomPanel.y - Theme.panelGap
        width: centerColumn.width
        height: Theme.panelGap
        onDragged: delta => root.shellController.resizeBottomPanel(-delta)
    }
}
