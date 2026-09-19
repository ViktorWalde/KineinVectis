# Qt/QML frontend of Kinein Vectis.
# Presentation only: every action is sent to kinein-core over JSON-RPC.

find_package(Qt6 6.4 REQUIRED COMPONENTS Core Gui Qml Quick)

qt_standard_project_setup()

if(DEFINED QT_KNOWN_POLICY_QTP0004)
    qt_policy(SET QTP0004 NEW)
endif()

qt_add_executable(kinein-vectis
    src/main.cpp
)

set_source_files_properties(qml/Theme.qml PROPERTIES QT_QML_SINGLETON_TYPE TRUE)
set_source_files_properties(qml/StatusColors.qml PROPERTIES QT_QML_SINGLETON_TYPE TRUE)
set_source_files_properties(qml/components/TooltipController.qml PROPERTIES QT_QML_SINGLETON_TYPE TRUE)
set_source_files_properties(qml/components/KvIcon.qml PROPERTIES QT_RESOURCE_ALIAS KvIcon.qml)
set_source_files_properties(qml/components/KvIconGlyphs.js PROPERTIES QT_RESOURCE_ALIAS KvIconGlyphs.js)
set_source_files_properties(qml/components/KvToggleChip.qml PROPERTIES QT_RESOURCE_ALIAS KvToggleChip.qml)
set_source_files_properties(qml/components/KvPanelFrame.qml PROPERTIES QT_RESOURCE_ALIAS KvPanelFrame.qml)
set_source_files_properties(qml/components/KvPanelHeader.qml PROPERTIES QT_RESOURCE_ALIAS KvPanelHeader.qml)
set_source_files_properties(qml/components/KvVerdict.qml PROPERTIES QT_RESOURCE_ALIAS KvVerdict.qml)
set_source_files_properties(qml/components/KvDataGrid.qml PROPERTIES QT_RESOURCE_ALIAS KvDataGrid.qml)
set_source_files_properties(qml/components/GridRules.qml PROPERTIES QT_RESOURCE_ALIAS GridRules.qml)
set_source_files_properties(qml/components/KvBarButton.qml PROPERTIES QT_RESOURCE_ALIAS KvBarButton.qml)
set_source_files_properties(qml/components/KvTooltip.qml PROPERTIES QT_RESOURCE_ALIAS KvTooltip.qml)
set_source_files_properties(qml/components/KvTooltipHost.qml PROPERTIES QT_RESOURCE_ALIAS KvTooltipHost.qml)
set_source_files_properties(qml/components/TooltipController.qml PROPERTIES QT_RESOURCE_ALIAS TooltipController.qml)
set_source_files_properties(qml/components/KvIconButton.qml PROPERTIES QT_RESOURCE_ALIAS KvIconButton.qml)
set_source_files_properties(qml/components/KvButton.qml PROPERTIES QT_RESOURCE_ALIAS KvButton.qml)
set_source_files_properties(qml/shell/TopHeaderBar.qml PROPERTIES QT_RESOURCE_ALIAS TopHeaderBar.qml)
set_source_files_properties(qml/workspace/LspStatusController.qml PROPERTIES QT_RESOURCE_ALIAS LspStatusController.qml)
set_source_files_properties(qml/jobs/ActiveJobController.qml PROPERTIES QT_RESOURCE_ALIAS ActiveJobController.qml)
set_source_files_properties(qml/jobs/ProblemRules.qml PROPERTIES QT_RESOURCE_ALIAS ProblemRules.qml)
set_source_files_properties(qml/shell/StatusBarJobWidget.qml PROPERTIES QT_RESOURCE_ALIAS StatusBarJobWidget.qml)
set_source_files_properties(qml/shell/HeaderRunWidget.qml PROPERTIES QT_RESOURCE_ALIAS HeaderRunWidget.qml)
set_source_files_properties(qml/shell/HeaderGitWidget.qml PROPERTIES QT_RESOURCE_ALIAS HeaderGitWidget.qml)
set_source_files_properties(qml/shell/HeaderProjectWidget.qml PROPERTIES QT_RESOURCE_ALIAS HeaderProjectWidget.qml)
set_source_files_properties(qml/shell/AppMenuBar.qml PROPERTIES QT_RESOURCE_ALIAS AppMenuBar.qml)
set_source_files_properties(qml/shell/AppMenuPopup.qml PROPERTIES QT_RESOURCE_ALIAS AppMenuPopup.qml)
set_source_files_properties(qml/shell/SideRail.qml PROPERTIES QT_RESOURCE_ALIAS SideRail.qml)
set_source_files_properties(qml/shell/ShellLayout.qml PROPERTIES QT_RESOURCE_ALIAS ShellLayout.qml)
set_source_files_properties(qml/app/AppDomains.qml PROPERTIES QT_RESOURCE_ALIAS AppDomains.qml)
set_source_files_properties(qml/app/StartupCommands.qml PROPERTIES QT_RESOURCE_ALIAS StartupCommands.qml)
set_source_files_properties(qml/app/AppRouters.qml PROPERTIES QT_RESOURCE_ALIAS AppRouters.qml)
set_source_files_properties(qml/shell/ShellController.qml PROPERTIES QT_RESOURCE_ALIAS ShellController.qml)
set_source_files_properties(qml/shell/GlobalShortcuts.qml PROPERTIES QT_RESOURCE_ALIAS GlobalShortcuts.qml)
set_source_files_properties(qml/shell/EnvironmentShortcuts.qml PROPERTIES QT_RESOURCE_ALIAS EnvironmentShortcuts.qml)
set_source_files_properties(qml/shell/ShellOverlays.qml PROPERTIES QT_RESOURCE_ALIAS ShellOverlays.qml)
set_source_files_properties(qml/shell/AboutDialog.qml PROPERTIES QT_RESOURCE_ALIAS AboutDialog.qml)
set_source_files_properties(qml/shell/DocumentationDialog.qml PROPERTIES QT_RESOURCE_ALIAS DocumentationDialog.qml)
set_source_files_properties(qml/shell/ShellHeaderHost.qml PROPERTIES QT_RESOURCE_ALIAS ShellHeaderHost.qml)
set_source_files_properties(qml/shell/WindowControls.qml PROPERTIES QT_RESOURCE_ALIAS WindowControls.qml)
set_source_files_properties(qml/shell/WindowResizeHandles.qml PROPERTIES QT_RESOURCE_ALIAS WindowResizeHandles.qml)
set_source_files_properties(qml/shell/ShellStatusHost.qml PROPERTIES QT_RESOURCE_ALIAS ShellStatusHost.qml)
set_source_files_properties(qml/shell/ShellWorkspaceHost.qml PROPERTIES QT_RESOURCE_ALIAS ShellWorkspaceHost.qml)
set_source_files_properties(qml/shell/ShellEditorHost.qml PROPERTIES QT_RESOURCE_ALIAS ShellEditorHost.qml)
set_source_files_properties(qml/shell/ShellEditorOverlayHost.qml PROPERTIES QT_RESOURCE_ALIAS ShellEditorOverlayHost.qml)
set_source_files_properties(qml/shell/ShellProjectOverlays.qml PROPERTIES QT_RESOURCE_ALIAS ShellProjectOverlays.qml)
set_source_files_properties(qml/shell/AppMenuItems.qml PROPERTIES QT_RESOURCE_ALIAS AppMenuItems.qml)
set_source_files_properties(qml/shell/ProjectHealthBanner.qml PROPERTIES QT_RESOURCE_ALIAS ProjectHealthBanner.qml)
set_source_files_properties(qml/shell/PanelSplitter.qml PROPERTIES QT_RESOURCE_ALIAS PanelSplitter.qml)
set_source_files_properties(qml/shell/VerticalScrollBar.qml PROPERTIES QT_RESOURCE_ALIAS VerticalScrollBar.qml)
set_source_files_properties(qml/shell/RunConfigDialog.qml PROPERTIES QT_RESOURCE_ALIAS RunConfigDialog.qml)
set_source_files_properties(qml/shell/RunConfigMenu.qml PROPERTIES QT_RESOURCE_ALIAS RunConfigMenu.qml)
set_source_files_properties(qml/debug/DebugController.qml PROPERTIES QT_RESOURCE_ALIAS DebugController.qml)
set_source_files_properties(qml/library/LibraryController.qml PROPERTIES QT_RESOURCE_ALIAS LibraryController.qml)
set_source_files_properties(qml/library/LibraryPanel.qml PROPERTIES QT_RESOURCE_ALIAS LibraryPanel.qml)
set_source_files_properties(qml/library/LibraryTargetPicker.qml PROPERTIES QT_RESOURCE_ALIAS LibraryTargetPicker.qml)
set_source_files_properties(qml/datasource/DataSourceController.qml PROPERTIES QT_RESOURCE_ALIAS DataSourceController.qml)
set_source_files_properties(qml/grafana/GrafanaController.qml PROPERTIES QT_RESOURCE_ALIAS GrafanaController.qml)
set_source_files_properties(qml/grafana/GrafanaPanel.qml PROPERTIES QT_RESOURCE_ALIAS GrafanaPanel.qml)
set_source_files_properties(qml/grafana/GrafanaPanelHost.qml PROPERTIES QT_RESOURCE_ALIAS GrafanaPanelHost.qml)
set_source_files_properties(qml/grafana/GrafanaFindings.qml PROPERTIES QT_RESOURCE_ALIAS GrafanaFindings.qml)
set_source_files_properties(qml/grafana/GrafanaVerdict.qml PROPERTIES QT_RESOURCE_ALIAS GrafanaVerdict.qml)
set_source_files_properties(qml/grafana/GrafanaTokenPrompt.qml PROPERTIES QT_RESOURCE_ALIAS GrafanaTokenPrompt.qml)
set_source_files_properties(qml/ipc/GrafanaEventRouter.qml PROPERTIES QT_RESOURCE_ALIAS GrafanaEventRouter.qml)
set_source_files_properties(qml/ipc/GrafanaRequestRouter.qml PROPERTIES QT_RESOURCE_ALIAS GrafanaRequestRouter.qml)
set_source_files_properties(qml/embedded/EmbeddedController.qml PROPERTIES QT_RESOURCE_ALIAS EmbeddedController.qml)
set_source_files_properties(qml/embedded/EmbeddedPanel.qml PROPERTIES QT_RESOURCE_ALIAS EmbeddedPanel.qml)
set_source_files_properties(qml/embedded/EmbeddedPanelHost.qml PROPERTIES QT_RESOURCE_ALIAS EmbeddedPanelHost.qml)
set_source_files_properties(qml/embedded/EmbeddedKitField.qml PROPERTIES QT_RESOURCE_ALIAS EmbeddedKitField.qml)
set_source_files_properties(qml/embedded/EmbeddedKitView.qml PROPERTIES QT_RESOURCE_ALIAS EmbeddedKitView.qml)
set_source_files_properties(qml/embedded/EmbeddedSizeView.qml PROPERTIES QT_RESOURCE_ALIAS EmbeddedSizeView.qml)
set_source_files_properties(qml/embedded/EmbeddedSerialView.qml PROPERTIES QT_RESOURCE_ALIAS EmbeddedSerialView.qml)
set_source_files_properties(qml/embedded/EmbeddedIdentityController.qml PROPERTIES QT_RESOURCE_ALIAS EmbeddedIdentityController.qml)
set_source_files_properties(qml/embedded/EmbeddedIdentityView.qml PROPERTIES QT_RESOURCE_ALIAS EmbeddedIdentityView.qml)
set_source_files_properties(qml/embedded/EmbeddedFlashController.qml PROPERTIES QT_RESOURCE_ALIAS EmbeddedFlashController.qml)
set_source_files_properties(qml/embedded/EmbeddedFlashView.qml PROPERTIES QT_RESOURCE_ALIAS EmbeddedFlashView.qml)
set_source_files_properties(qml/embedded/EmbeddedAccessController.qml PROPERTIES QT_RESOURCE_ALIAS EmbeddedAccessController.qml)
set_source_files_properties(qml/embedded/EmbeddedAccessView.qml PROPERTIES QT_RESOURCE_ALIAS EmbeddedAccessView.qml)
set_source_files_properties(qml/embedded/EmbeddedFilesController.qml PROPERTIES QT_RESOURCE_ALIAS EmbeddedFilesController.qml)
set_source_files_properties(qml/embedded/EmbeddedFilesView.qml PROPERTIES QT_RESOURCE_ALIAS EmbeddedFilesView.qml)
set_source_files_properties(qml/embedded/EmbeddedProjectView.qml PROPERTIES QT_RESOURCE_ALIAS EmbeddedProjectView.qml)
set_source_files_properties(qml/embedded/EmbeddedAdapterView.qml PROPERTIES QT_RESOURCE_ALIAS EmbeddedAdapterView.qml)
set_source_files_properties(qml/embedded/EmbeddedInstallView.qml PROPERTIES QT_RESOURCE_ALIAS EmbeddedInstallView.qml)
set_source_files_properties(qml/embedded/EmbeddedKitImportView.qml PROPERTIES QT_RESOURCE_ALIAS EmbeddedKitImportView.qml)
set_source_files_properties(qml/container/ContainerController.qml PROPERTIES QT_RESOURCE_ALIAS ContainerController.qml)
set_source_files_properties(qml/index/IndexController.qml PROPERTIES QT_RESOURCE_ALIAS IndexController.qml)
set_source_files_properties(qml/ipc/IndexEventRouter.qml PROPERTIES QT_RESOURCE_ALIAS IndexEventRouter.qml)
set_source_files_properties(qml/ipc/IndexRequestRouter.qml PROPERTIES QT_RESOURCE_ALIAS IndexRequestRouter.qml)
set_source_files_properties(qml/python/PythonController.qml PROPERTIES QT_RESOURCE_ALIAS PythonController.qml)
set_source_files_properties(qml/ipc/PythonEventRouter.qml PROPERTIES QT_RESOURCE_ALIAS PythonEventRouter.qml)
set_source_files_properties(qml/ipc/PythonRequestRouter.qml PROPERTIES QT_RESOURCE_ALIAS PythonRequestRouter.qml)
set_source_files_properties(qml/container/ContainerPanel.qml PROPERTIES QT_RESOURCE_ALIAS ContainerPanel.qml)
set_source_files_properties(qml/container/ContainerListView.qml PROPERTIES QT_RESOURCE_ALIAS ContainerListView.qml)
set_source_files_properties(qml/container/ContainerPanelHost.qml PROPERTIES QT_RESOURCE_ALIAS ContainerPanelHost.qml)
set_source_files_properties(qml/ipc/ContainerEventRouter.qml PROPERTIES QT_RESOURCE_ALIAS ContainerEventRouter.qml)
set_source_files_properties(qml/ipc/ContainerRequestRouter.qml PROPERTIES QT_RESOURCE_ALIAS ContainerRequestRouter.qml)
set_source_files_properties(qml/app/AppEnvironmentDomains.qml PROPERTIES QT_RESOURCE_ALIAS AppEnvironmentDomains.qml)
set_source_files_properties(qml/ipc/EmbeddedEventRouter.qml PROPERTIES QT_RESOURCE_ALIAS EmbeddedEventRouter.qml)
set_source_files_properties(qml/ipc/EmbeddedRequestRouter.qml PROPERTIES QT_RESOURCE_ALIAS EmbeddedRequestRouter.qml)
set_source_files_properties(qml/datasource/DataSourcePanel.qml PROPERTIES QT_RESOURCE_ALIAS DataSourcePanel.qml)
set_source_files_properties(qml/datasource/DataSourceQuery.qml PROPERTIES QT_RESOURCE_ALIAS DataSourceQuery.qml)
set_source_files_properties(qml/datasource/DataSourcePanelHost.qml PROPERTIES QT_RESOURCE_ALIAS DataSourcePanelHost.qml)
set_source_files_properties(qml/datasource/DataSourceForm.qml PROPERTIES QT_RESOURCE_ALIAS DataSourceForm.qml)
set_source_files_properties(qml/datasource/DataSourceList.qml PROPERTIES QT_RESOURCE_ALIAS DataSourceList.qml)
set_source_files_properties(qml/datasource/DataSourceDiscoveryController.qml PROPERTIES QT_RESOURCE_ALIAS DataSourceDiscoveryController.qml)
set_source_files_properties(qml/datasource/DataSourceCreateBox.qml PROPERTIES QT_RESOURCE_ALIAS DataSourceCreateBox.qml)
set_source_files_properties(qml/datasource/DataSourceVerdict.qml PROPERTIES QT_RESOURCE_ALIAS DataSourceVerdict.qml)
set_source_files_properties(qml/datasource/DataSourceStructure.qml PROPERTIES QT_RESOURCE_ALIAS DataSourceStructure.qml)
set_source_files_properties(qml/datasource/DataSourceCollections.qml PROPERTIES QT_RESOURCE_ALIAS DataSourceCollections.qml)
set_source_files_properties(qml/setup/SetupController.qml PROPERTIES QT_RESOURCE_ALIAS SetupController.qml)
set_source_files_properties(qml/setup/SetupPanel.qml PROPERTIES QT_RESOURCE_ALIAS SetupPanel.qml)
set_source_files_properties(qml/setup/SetupPanelHost.qml PROPERTIES QT_RESOURCE_ALIAS SetupPanelHost.qml)
set_source_files_properties(qml/setup/SetupToolRow.qml PROPERTIES QT_RESOURCE_ALIAS SetupToolRow.qml)
set_source_files_properties(qml/datasource/DataSourceField.qml PROPERTIES QT_RESOURCE_ALIAS DataSourceField.qml)
set_source_files_properties(qml/library/LibraryPanelHost.qml PROPERTIES QT_RESOURCE_ALIAS LibraryPanelHost.qml)
set_source_files_properties(qml/library/LibraryPlanView.qml PROPERTIES QT_RESOURCE_ALIAS LibraryPlanView.qml)
set_source_files_properties(qml/ipc/LibraryRequestRouter.qml PROPERTIES QT_RESOURCE_ALIAS LibraryRequestRouter.qml)
set_source_files_properties(qml/ipc/LibraryEventRouter.qml PROPERTIES QT_RESOURCE_ALIAS LibraryEventRouter.qml)
set_source_files_properties(qml/shell/ShellEnvironmentOverlays.qml PROPERTIES QT_RESOURCE_ALIAS ShellEnvironmentOverlays.qml)
set_source_files_properties(qml/ipc/DataSourceEventRouter.qml PROPERTIES QT_RESOURCE_ALIAS DataSourceEventRouter.qml)
set_source_files_properties(qml/ipc/DataSourceRequestRouter.qml PROPERTIES QT_RESOURCE_ALIAS DataSourceRequestRouter.qml)
set_source_files_properties(qml/ipc/SetupEventRouter.qml PROPERTIES QT_RESOURCE_ALIAS SetupEventRouter.qml)
set_source_files_properties(qml/ipc/SetupRequestRouter.qml PROPERTIES QT_RESOURCE_ALIAS SetupRequestRouter.qml)
set_source_files_properties(qml/debug/DebugWatches.qml PROPERTIES QT_RESOURCE_ALIAS DebugWatches.qml)
set_source_files_properties(qml/debug/DebugInspector.qml PROPERTIES QT_RESOURCE_ALIAS DebugInspector.qml)
set_source_files_properties(qml/debug/DebugInspectController.qml PROPERTIES QT_RESOURCE_ALIAS DebugInspectController.qml)
set_source_files_properties(qml/debug/DebugInspectView.qml PROPERTIES QT_RESOURCE_ALIAS DebugInspectView.qml)
set_source_files_properties(qml/diagnostics/DiagnosticsController.qml PROPERTIES QT_RESOURCE_ALIAS DiagnosticsController.qml)
set_source_files_properties(qml/settings/SettingsController.qml PROPERTIES QT_RESOURCE_ALIAS SettingsController.qml)
set_source_files_properties(qml/settings/SettingsDialog.qml PROPERTIES QT_RESOURCE_ALIAS SettingsDialog.qml)
set_source_files_properties(qml/settings/SettingsToggleRow.qml PROPERTIES QT_RESOURCE_ALIAS SettingsToggleRow.qml)
set_source_files_properties(qml/ipc/SettingsEventRouter.qml PROPERTIES QT_RESOURCE_ALIAS SettingsEventRouter.qml)
set_source_files_properties(qml/configaction/ConfigActionController.qml PROPERTIES QT_RESOURCE_ALIAS ConfigActionController.qml)
set_source_files_properties(qml/configaction/ConfigActionsDialog.qml PROPERTIES QT_RESOURCE_ALIAS ConfigActionsDialog.qml)
set_source_files_properties(qml/configaction/ConfigActionList.qml PROPERTIES QT_RESOURCE_ALIAS ConfigActionList.qml)
set_source_files_properties(qml/configaction/ConfigActionPreview.qml PROPERTIES QT_RESOURCE_ALIAS ConfigActionPreview.qml)
set_source_files_properties(qml/configaction/ConfigActionDiffView.qml PROPERTIES QT_RESOURCE_ALIAS ConfigActionDiffView.qml)
set_source_files_properties(qml/configaction/ConfigActionField.qml PROPERTIES QT_RESOURCE_ALIAS ConfigActionField.qml)
set_source_files_properties(qml/ipc/ConfigActionEventRouter.qml PROPERTIES QT_RESOURCE_ALIAS ConfigActionEventRouter.qml)
set_source_files_properties(qml/ipc/ConfigActionRequestRouter.qml PROPERTIES QT_RESOURCE_ALIAS ConfigActionRequestRouter.qml)
set_source_files_properties(qml/toolchain/ToolchainController.qml PROPERTIES QT_RESOURCE_ALIAS ToolchainController.qml)
set_source_files_properties(qml/toolchain/ToolchainMenu.qml PROPERTIES QT_RESOURCE_ALIAS ToolchainMenu.qml)
set_source_files_properties(qml/ipc/ToolchainEventRouter.qml PROPERTIES QT_RESOURCE_ALIAS ToolchainEventRouter.qml)
set_source_files_properties(qml/ipc/ToolchainRequestRouter.qml PROPERTIES QT_RESOURCE_ALIAS ToolchainRequestRouter.qml)
set_source_files_properties(qml/git/GitController.qml PROPERTIES QT_RESOURCE_ALIAS GitController.qml)
set_source_files_properties(qml/git/GitHistoryController.qml PROPERTIES QT_RESOURCE_ALIAS GitHistoryController.qml)
set_source_files_properties(qml/git/GitChangesList.qml PROPERTIES QT_RESOURCE_ALIAS GitChangesList.qml)
set_source_files_properties(qml/git/GitHistoryList.qml PROPERTIES QT_RESOURCE_ALIAS GitHistoryList.qml)
set_source_files_properties(qml/git/GitRules.qml PROPERTIES QT_RESOURCE_ALIAS GitRules.qml)
set_source_files_properties(qml/git/GitInspectorController.qml PROPERTIES QT_RESOURCE_ALIAS GitInspectorController.qml)
set_source_files_properties(qml/git/GitPatchView.qml PROPERTIES QT_RESOURCE_ALIAS GitPatchView.qml)
set_source_files_properties(qml/git/GitInspectorPane.qml PROPERTIES QT_RESOURCE_ALIAS GitInspectorPane.qml)
set_source_files_properties(qml/git/GitBranchMenu.qml PROPERTIES QT_RESOURCE_ALIAS GitBranchMenu.qml)
set_source_files_properties(qml/git/GitCommitBox.qml PROPERTIES QT_RESOURCE_ALIAS GitCommitBox.qml)
set_source_files_properties(qml/git/GitDiffDialog.qml PROPERTIES QT_RESOURCE_ALIAS GitDiffDialog.qml)
set_source_files_properties(qml/git/GitDiscardDialog.qml PROPERTIES QT_RESOURCE_ALIAS GitDiscardDialog.qml)
set_source_files_properties(qml/panels/bottom/GitPanel.qml PROPERTIES QT_RESOURCE_ALIAS GitPanel.qml)
set_source_files_properties(qml/ipc/GitEventRouter.qml PROPERTIES QT_RESOURCE_ALIAS GitEventRouter.qml)
set_source_files_properties(qml/ipc/CoverageEventRouter.qml PROPERTIES QT_RESOURCE_ALIAS CoverageEventRouter.qml)
set_source_files_properties(qml/ipc/CoverageRequestRouter.qml PROPERTIES QT_RESOURCE_ALIAS CoverageRequestRouter.qml)
set_source_files_properties(qml/coverage/CoverageController.qml PROPERTIES QT_RESOURCE_ALIAS CoverageController.qml)
set_source_files_properties(qml/ipc/RemoteEventRouter.qml PROPERTIES QT_RESOURCE_ALIAS RemoteEventRouter.qml)
set_source_files_properties(qml/ipc/RemoteRequestRouter.qml PROPERTIES QT_RESOURCE_ALIAS RemoteRequestRouter.qml)
set_source_files_properties(qml/remote/RemoteController.qml PROPERTIES QT_RESOURCE_ALIAS RemoteController.qml)
set_source_files_properties(qml/remote/RemoteList.qml PROPERTIES QT_RESOURCE_ALIAS RemoteList.qml)
set_source_files_properties(qml/remote/RemoteForm.qml PROPERTIES QT_RESOURCE_ALIAS RemoteForm.qml)
set_source_files_properties(qml/remote/RemoteVerdict.qml PROPERTIES QT_RESOURCE_ALIAS RemoteVerdict.qml)
set_source_files_properties(qml/remote/RemotePanel.qml PROPERTIES QT_RESOURCE_ALIAS RemotePanel.qml)
set_source_files_properties(qml/remote/RemotePanelHost.qml PROPERTIES QT_RESOURCE_ALIAS RemotePanelHost.qml)
set_source_files_properties(qml/remote/RemoteMirrorView.qml PROPERTIES QT_RESOURCE_ALIAS RemoteMirrorView.qml)
set_source_files_properties(qml/ipc/GitRequestRouter.qml PROPERTIES QT_RESOURCE_ALIAS GitRequestRouter.qml)
set_source_files_properties(qml/ipc/DebugEventRouter.qml PROPERTIES QT_RESOURCE_ALIAS DebugEventRouter.qml)
set_source_files_properties(qml/ipc/DebugRequestRouter.qml PROPERTIES QT_RESOURCE_ALIAS DebugRequestRouter.qml)
set_source_files_properties(qml/debug/DebugAttachForm.qml PROPERTIES QT_RESOURCE_ALIAS DebugAttachForm.qml)
set_source_files_properties(qml/panels/bottom/DebugPanel.qml PROPERTIES QT_RESOURCE_ALIAS DebugPanel.qml)
set_source_files_properties(qml/shell/BottomTabBar.qml PROPERTIES QT_RESOURCE_ALIAS BottomTabBar.qml)
set_source_files_properties(qml/shell/StatusBarProjectSummaries.qml PROPERTIES QT_RESOURCE_ALIAS StatusBarProjectSummaries.qml)
set_source_files_properties(qml/shell/WorkspaceStatusBar.qml PROPERTIES QT_RESOURCE_ALIAS WorkspaceStatusBar.qml)
set_source_files_properties(qml/panels/bottom/BottomPanelHost.qml PROPERTIES QT_RESOURCE_ALIAS BottomPanelHost.qml)
set_source_files_properties(qml/panels/bottom/TerminalSessionTabs.qml PROPERTIES QT_RESOURCE_ALIAS TerminalSessionTabs.qml)
set_source_files_properties(qml/panels/bottom/BuildPanel.qml PROPERTIES QT_RESOURCE_ALIAS BuildPanel.qml)
set_source_files_properties(qml/panels/bottom/IdeLogPanel.qml PROPERTIES QT_RESOURCE_ALIAS IdeLogPanel.qml)
set_source_files_properties(qml/panels/bottom/JobsPanel.qml PROPERTIES QT_RESOURCE_ALIAS JobsPanel.qml)
set_source_files_properties(qml/panels/bottom/ProblemsPanel.qml PROPERTIES QT_RESOURCE_ALIAS ProblemsPanel.qml)
set_source_files_properties(qml/panels/bottom/SearchPanel.qml PROPERTIES QT_RESOURCE_ALIAS SearchPanel.qml)
set_source_files_properties(qml/panels/bottom/TerminalPanel.qml PROPERTIES QT_RESOURCE_ALIAS TerminalPanel.qml)
set_source_files_properties(qml/panels/bottom/TerminalViewport.qml PROPERTIES QT_RESOURCE_ALIAS TerminalViewport.qml)
set_source_files_properties(qml/panels/bottom/TerminalSpanRules.qml PROPERTIES QT_RESOURCE_ALIAS TerminalSpanRules.qml)
set_source_files_properties(qml/panels/bottom/SearchReplaceBar.qml PROPERTIES QT_RESOURCE_ALIAS SearchReplaceBar.qml)
set_source_files_properties(qml/panels/bottom/TerminalGeometryOverlay.qml PROPERTIES QT_RESOURCE_ALIAS TerminalGeometryOverlay.qml)
set_source_files_properties(qml/panels/bottom/TerminalMetrics.qml PROPERTIES QT_RESOURCE_ALIAS TerminalMetrics.qml)
set_source_files_properties(qml/panels/bottom/TerminalScrollController.qml PROPERTIES QT_RESOURCE_ALIAS TerminalScrollController.qml)
set_source_files_properties(qml/panels/bottom/TerminalSelectionController.qml PROPERTIES QT_RESOURCE_ALIAS TerminalSelectionController.qml)
set_source_files_properties(qml/panels/bottom/TerminalInputController.qml PROPERTIES QT_RESOURCE_ALIAS TerminalInputController.qml)
set_source_files_properties(qml/panels/bottom/TestsPanel.qml PROPERTIES QT_RESOURCE_ALIAS TestsPanel.qml)
set_source_files_properties(qml/panels/bottom/ToolsPanel.qml PROPERTIES QT_RESOURCE_ALIAS ToolsPanel.qml)
set_source_files_properties(qml/workspace/FolderPickerDialog.qml PROPERTIES QT_RESOURCE_ALIAS FolderPickerDialog.qml)
set_source_files_properties(qml/workspace/FolderPickerController.qml PROPERTIES QT_RESOURCE_ALIAS FolderPickerController.qml)
set_source_files_properties(qml/workspace/FolderPickerButton.qml PROPERTIES QT_RESOURCE_ALIAS FolderPickerButton.qml)
set_source_files_properties(qml/workspace/FolderPickerHeader.qml PROPERTIES QT_RESOURCE_ALIAS FolderPickerHeader.qml)
set_source_files_properties(qml/workspace/FolderPickerPathBar.qml PROPERTIES QT_RESOURCE_ALIAS FolderPickerPathBar.qml)
set_source_files_properties(qml/workspace/FolderPickerCreatePanel.qml PROPERTIES QT_RESOURCE_ALIAS FolderPickerCreatePanel.qml)
set_source_files_properties(qml/workspace/FolderPickerDirectoryList.qml PROPERTIES QT_RESOURCE_ALIAS FolderPickerDirectoryList.qml)
set_source_files_properties(qml/workspace/FolderPickerActionRow.qml PROPERTIES QT_RESOURCE_ALIAS FolderPickerActionRow.qml)
set_source_files_properties(qml/workspace/FolderPickerCard.qml PROPERTIES QT_RESOURCE_ALIAS FolderPickerCard.qml)
set_source_files_properties(qml/workspace/WorkspaceController.qml PROPERTIES QT_RESOURCE_ALIAS WorkspaceController.qml)
set_source_files_properties(qml/workspace/RecentWorkspacesController.qml PROPERTIES QT_RESOURCE_ALIAS RecentWorkspacesController.qml)
set_source_files_properties(qml/workspace/RecentWorkspacesCard.qml PROPERTIES QT_RESOURCE_ALIAS RecentWorkspacesCard.qml)
set_source_files_properties(qml/workspace/StartScreen.qml PROPERTIES QT_RESOURCE_ALIAS StartScreen.qml)
set_source_files_properties(qml/workspace/ProjectHealthController.qml PROPERTIES QT_RESOURCE_ALIAS ProjectHealthController.qml)
set_source_files_properties(qml/workspace/WorkspaceUiResetter.qml PROPERTIES QT_RESOURCE_ALIAS WorkspaceUiResetter.qml)
set_source_files_properties(qml/jobs/JobsController.qml PROPERTIES QT_RESOURCE_ALIAS JobsController.qml)
set_source_files_properties(qml/runtime/RuntimeController.qml PROPERTIES QT_RESOURCE_ALIAS RuntimeController.qml)
set_source_files_properties(qml/runtime/RunConfigController.qml PROPERTIES QT_RESOURCE_ALIAS RunConfigController.qml)
set_source_files_properties(qml/search/SearchController.qml PROPERTIES QT_RESOURCE_ALIAS SearchController.qml)
set_source_files_properties(qml/search/SearchEverywhereController.qml PROPERTIES QT_RESOURCE_ALIAS SearchEverywhereController.qml)
set_source_files_properties(qml/ipc/WorkspaceEventRouter.qml PROPERTIES QT_RESOURCE_ALIAS WorkspaceEventRouter.qml)
set_source_files_properties(qml/ipc/EditorEventRouter.qml PROPERTIES QT_RESOURCE_ALIAS EditorEventRouter.qml)
set_source_files_properties(qml/ipc/EditorRequestRouter.qml PROPERTIES QT_RESOURCE_ALIAS EditorRequestRouter.qml)
set_source_files_properties(qml/ipc/JobsEventRouter.qml PROPERTIES QT_RESOURCE_ALIAS JobsEventRouter.qml)
set_source_files_properties(qml/ipc/SearchEventRouter.qml PROPERTIES QT_RESOURCE_ALIAS SearchEventRouter.qml)
set_source_files_properties(qml/ipc/ProjectTreeRequestRouter.qml PROPERTIES QT_RESOURCE_ALIAS ProjectTreeRequestRouter.qml)
set_source_files_properties(qml/ipc/SearchRequestRouter.qml PROPERTIES QT_RESOURCE_ALIAS SearchRequestRouter.qml)
set_source_files_properties(qml/ipc/RuntimeEventRouter.qml PROPERTIES QT_RESOURCE_ALIAS RuntimeEventRouter.qml)
set_source_files_properties(qml/ipc/RuntimeRequestRouter.qml PROPERTIES QT_RESOURCE_ALIAS RuntimeRequestRouter.qml)
set_source_files_properties(qml/project/ProjectCreateDialog.qml PROPERTIES QT_RESOURCE_ALIAS ProjectCreateDialog.qml)
set_source_files_properties(qml/project/ProjectTreeController.qml PROPERTIES QT_RESOURCE_ALIAS ProjectTreeController.qml)
set_source_files_properties(qml/project/ProjectTreeRevealController.qml PROPERTIES QT_RESOURCE_ALIAS ProjectTreeRevealController.qml)
set_source_files_properties(qml/project/ProjectTreeRules.qml PROPERTIES QT_RESOURCE_ALIAS ProjectTreeRules.qml)
set_source_files_properties(qml/project/ProjectExplorer.qml PROPERTIES QT_RESOURCE_ALIAS ProjectExplorer.qml)
set_source_files_properties(qml/project/ProjectEntryContextMenu.qml PROPERTIES QT_RESOURCE_ALIAS ProjectEntryContextMenu.qml)
set_source_files_properties(qml/project/ProjectEntryRenameDialog.qml PROPERTIES QT_RESOURCE_ALIAS ProjectEntryRenameDialog.qml)
set_source_files_properties(qml/project/ProjectEntryDeleteDialog.qml PROPERTIES QT_RESOURCE_ALIAS ProjectEntryDeleteDialog.qml)
set_source_files_properties(qml/command/SearchEverywhereDialog.qml PROPERTIES QT_RESOURCE_ALIAS SearchEverywhereDialog.qml)
set_source_files_properties(qml/command/CommandDispatcher.qml PROPERTIES QT_RESOURCE_ALIAS CommandDispatcher.qml)
set_source_files_properties(qml/editor/SymbolRenameDialog.qml PROPERTIES QT_RESOURCE_ALIAS SymbolRenameDialog.qml)
set_source_files_properties(qml/editor/EditorWorkspaceEditPreviewDialog.qml PROPERTIES QT_RESOURCE_ALIAS EditorWorkspaceEditPreviewDialog.qml)
set_source_files_properties(qml/editor/EditorController.qml PROPERTIES QT_RESOURCE_ALIAS EditorController.qml)
set_source_files_properties(qml/editor/EditorSurfaceBridge.qml PROPERTIES QT_RESOURCE_ALIAS EditorSurfaceBridge.qml)
set_source_files_properties(qml/editor/EditorDocumentController.qml PROPERTIES QT_RESOURCE_ALIAS EditorDocumentController.qml)
set_source_files_properties(qml/editor/EditorTextController.qml PROPERTIES QT_RESOURCE_ALIAS EditorTextController.qml)
set_source_files_properties(qml/editor/EditorLanguageController.qml PROPERTIES QT_RESOURCE_ALIAS EditorLanguageController.qml)
set_source_files_properties(qml/editor/EditorHighlightController.qml PROPERTIES QT_RESOURCE_ALIAS EditorHighlightController.qml)
set_source_files_properties(qml/editor/EditorFormatController.qml PROPERTIES QT_RESOURCE_ALIAS EditorFormatController.qml)
set_source_files_properties(qml/editor/EditorPersistenceController.qml PROPERTIES QT_RESOURCE_ALIAS EditorPersistenceController.qml)
set_source_files_properties(qml/editor/EditorCompletionController.qml PROPERTIES QT_RESOURCE_ALIAS EditorCompletionController.qml)
set_source_files_properties(qml/editor/EditorPane.qml PROPERTIES QT_RESOURCE_ALIAS EditorPane.qml)
set_source_files_properties(qml/editor/EditorTabsBar.qml PROPERTIES QT_RESOURCE_ALIAS EditorTabsBar.qml)
set_source_files_properties(qml/editor/EditorTextSurface.qml PROPERTIES QT_RESOURCE_ALIAS EditorTextSurface.qml)
set_source_files_properties(qml/editor/EditorGutter.qml PROPERTIES QT_RESOURCE_ALIAS EditorGutter.qml)
set_source_files_properties(qml/editor/EditorGutterLineMarks.qml PROPERTIES QT_RESOURCE_ALIAS EditorGutterLineMarks.qml)
set_source_files_properties(qml/editor/TextRules.qml PROPERTIES QT_RESOURCE_ALIAS TextRules.qml)
set_source_files_properties(qml/editor/EditorBreadcrumbs.qml PROPERTIES QT_RESOURCE_ALIAS EditorBreadcrumbs.qml)
set_source_files_properties(qml/editor/EditorOutlineHandle.qml PROPERTIES QT_RESOURCE_ALIAS EditorOutlineHandle.qml)
set_source_files_properties(qml/editor/EditorTextGeometry.qml PROPERTIES QT_RESOURCE_ALIAS EditorTextGeometry.qml)
set_source_files_properties(qml/editor/EditorSelectionLadder.qml PROPERTIES QT_RESOURCE_ALIAS EditorSelectionLadder.qml)
set_source_files_properties(qml/editor/PathRules.qml PROPERTIES QT_RESOURCE_ALIAS PathRules.qml)
set_source_files_properties(qml/editor/EditorExternalChangeController.qml PROPERTIES QT_RESOURCE_ALIAS EditorExternalChangeController.qml)
set_source_files_properties(qml/editor/EditorJumpController.qml PROPERTIES QT_RESOURCE_ALIAS EditorJumpController.qml)
set_source_files_properties(qml/editor/EditorRecentFiles.qml PROPERTIES QT_RESOURCE_ALIAS EditorRecentFiles.qml)
set_source_files_properties(qml/editor/EditorTypingController.qml PROPERTIES QT_RESOURCE_ALIAS EditorTypingController.qml)
set_source_files_properties(qml/editor/EditorLineHighlights.qml PROPERTIES QT_RESOURCE_ALIAS EditorLineHighlights.qml)
set_source_files_properties(qml/editor/EditorDiagnosticTooltip.qml PROPERTIES QT_RESOURCE_ALIAS EditorDiagnosticTooltip.qml)
set_source_files_properties(qml/editor/EditorHoverPopup.qml PROPERTIES QT_RESOURCE_ALIAS EditorHoverPopup.qml)
set_source_files_properties(qml/editor/EditorCompletionPopup.qml PROPERTIES QT_RESOURCE_ALIAS EditorCompletionPopup.qml)
set_source_files_properties(qml/editor/EditorActionsPopup.qml PROPERTIES QT_RESOURCE_ALIAS EditorActionsPopup.qml)
set_source_files_properties(qml/editor/EditorGoToLineDialog.qml PROPERTIES QT_RESOURCE_ALIAS EditorGoToLineDialog.qml)
set_source_files_properties(qml/editor/EditorUsagesPopup.qml PROPERTIES QT_RESOURCE_ALIAS EditorUsagesPopup.qml)
set_source_files_properties(qml/editor/EditorFindController.qml PROPERTIES QT_RESOURCE_ALIAS EditorFindController.qml)
set_source_files_properties(qml/editor/EditorFindBar.qml PROPERTIES QT_RESOURCE_ALIAS EditorFindBar.qml)
set_source_files_properties(qml/editor/EditorExternalChangeBanner.qml PROPERTIES QT_RESOURCE_ALIAS EditorExternalChangeBanner.qml)
set_source_files_properties(qml/editor/EditorOutlineController.qml PROPERTIES QT_RESOURCE_ALIAS EditorOutlineController.qml)
set_source_files_properties(qml/editor/EditorOutlinePanel.qml PROPERTIES QT_RESOURCE_ALIAS EditorOutlinePanel.qml)

set(KINEIN_MANUAL_FILE "${CMAKE_SOURCE_DIR}/DocsPublic/manual.md")
set_source_files_properties("${KINEIN_MANUAL_FILE}" PROPERTIES QT_RESOURCE_ALIAS MANUAL.md)

qt_add_qml_module(kinein-vectis
    URI KineinVectis
    VERSION 1.0
    RESOURCE_PREFIX /
    DEPENDENCIES QtQuick QtQuick.Window
    QML_FILES
        qml/Main.qml
        qml/Theme.qml
        qml/StatusColors.qml
        qml/components/KvIcon.qml
        qml/components/KvIconGlyphs.js
        qml/components/KvToggleChip.qml
        qml/components/KvPanelFrame.qml
        qml/components/KvPanelHeader.qml
        qml/components/KvVerdict.qml
        qml/components/KvDataGrid.qml
        qml/components/GridRules.qml
        qml/components/KvBarButton.qml
        qml/components/KvTooltip.qml
        qml/components/KvTooltipHost.qml
        qml/components/TooltipController.qml
        qml/components/KvIconButton.qml
        qml/components/KvButton.qml
        qml/shell/TopHeaderBar.qml
        qml/workspace/LspStatusController.qml
        qml/jobs/ActiveJobController.qml
        qml/jobs/ProblemRules.qml
        qml/shell/StatusBarJobWidget.qml
        qml/shell/HeaderRunWidget.qml
        qml/shell/HeaderGitWidget.qml
        qml/shell/HeaderProjectWidget.qml
        qml/shell/AppMenuBar.qml
        qml/shell/AppMenuPopup.qml
        qml/shell/SideRail.qml
        qml/shell/ShellLayout.qml
        qml/app/AppDomains.qml
        qml/app/StartupCommands.qml
        qml/app/AppRouters.qml
        qml/shell/ShellController.qml
        qml/shell/GlobalShortcuts.qml
        qml/shell/EnvironmentShortcuts.qml
        qml/shell/ShellEnvironmentOverlays.qml
        qml/shell/ShellOverlays.qml
        qml/shell/AboutDialog.qml
        qml/shell/DocumentationDialog.qml
        qml/shell/ShellHeaderHost.qml
        qml/shell/WindowControls.qml
        qml/shell/WindowResizeHandles.qml
        qml/shell/ShellStatusHost.qml
        qml/shell/ShellWorkspaceHost.qml
        qml/shell/ShellEditorHost.qml
        qml/shell/ShellEditorOverlayHost.qml
        qml/shell/ShellProjectOverlays.qml
        qml/shell/AppMenuItems.qml
        qml/shell/ProjectHealthBanner.qml
        qml/shell/PanelSplitter.qml
        qml/shell/VerticalScrollBar.qml
        qml/shell/RunConfigDialog.qml
        qml/shell/RunConfigMenu.qml
        qml/debug/DebugController.qml
        qml/library/LibraryController.qml
        qml/library/LibraryPanel.qml
        qml/library/LibraryTargetPicker.qml
        qml/datasource/DataSourceController.qml
        qml/grafana/GrafanaController.qml
        qml/grafana/GrafanaPanel.qml
        qml/grafana/GrafanaPanelHost.qml
        qml/grafana/GrafanaFindings.qml
        qml/grafana/GrafanaVerdict.qml
        qml/grafana/GrafanaTokenPrompt.qml
        qml/ipc/GrafanaEventRouter.qml
        qml/ipc/GrafanaRequestRouter.qml
        qml/embedded/EmbeddedController.qml
        qml/embedded/EmbeddedPanel.qml
        qml/embedded/EmbeddedPanelHost.qml
        qml/embedded/EmbeddedKitField.qml
        qml/embedded/EmbeddedKitView.qml
        qml/embedded/EmbeddedSizeView.qml
        qml/embedded/EmbeddedSerialView.qml
        qml/embedded/EmbeddedIdentityController.qml
        qml/embedded/EmbeddedIdentityView.qml
        qml/embedded/EmbeddedFlashController.qml
        qml/embedded/EmbeddedFlashView.qml
        qml/embedded/EmbeddedAccessController.qml
        qml/embedded/EmbeddedAccessView.qml
        qml/embedded/EmbeddedFilesController.qml
        qml/embedded/EmbeddedFilesView.qml
        qml/embedded/EmbeddedProjectView.qml
        qml/embedded/EmbeddedAdapterView.qml
        qml/embedded/EmbeddedInstallView.qml
        qml/embedded/EmbeddedKitImportView.qml
        qml/container/ContainerController.qml
        qml/index/IndexController.qml
        qml/ipc/IndexEventRouter.qml
        qml/ipc/IndexRequestRouter.qml
        qml/python/PythonController.qml
        qml/ipc/PythonEventRouter.qml
        qml/ipc/PythonRequestRouter.qml
        qml/container/ContainerPanel.qml
        qml/container/ContainerListView.qml
        qml/container/ContainerPanelHost.qml
        qml/ipc/ContainerEventRouter.qml
        qml/ipc/ContainerRequestRouter.qml
        qml/app/AppEnvironmentDomains.qml
        qml/ipc/EmbeddedEventRouter.qml
        qml/ipc/EmbeddedRequestRouter.qml
        qml/datasource/DataSourcePanel.qml
        qml/datasource/DataSourceQuery.qml
        qml/datasource/DataSourcePanelHost.qml
        qml/datasource/DataSourceForm.qml
        qml/datasource/DataSourceList.qml
        qml/datasource/DataSourceDiscoveryController.qml
        qml/datasource/DataSourceCreateBox.qml
        qml/datasource/DataSourceVerdict.qml
        qml/datasource/DataSourceStructure.qml
        qml/datasource/DataSourceCollections.qml
        qml/setup/SetupController.qml
        qml/setup/SetupPanel.qml
        qml/setup/SetupPanelHost.qml
        qml/setup/SetupToolRow.qml
        qml/datasource/DataSourceField.qml
        qml/library/LibraryPanelHost.qml
        qml/library/LibraryPlanView.qml
        qml/ipc/LibraryRequestRouter.qml
        qml/ipc/LibraryEventRouter.qml
        qml/ipc/DataSourceEventRouter.qml
        qml/ipc/DataSourceRequestRouter.qml
        qml/ipc/SetupEventRouter.qml
        qml/ipc/SetupRequestRouter.qml
        qml/debug/DebugWatches.qml
        qml/debug/DebugInspector.qml
        qml/debug/DebugInspectController.qml
        qml/debug/DebugInspectView.qml
        qml/diagnostics/DiagnosticsController.qml
        qml/settings/SettingsController.qml
        qml/settings/SettingsDialog.qml
        qml/settings/SettingsToggleRow.qml
        qml/ipc/SettingsEventRouter.qml
        qml/configaction/ConfigActionController.qml
        qml/configaction/ConfigActionsDialog.qml
        qml/configaction/ConfigActionList.qml
        qml/configaction/ConfigActionPreview.qml
        qml/configaction/ConfigActionDiffView.qml
        qml/configaction/ConfigActionField.qml
        qml/ipc/ConfigActionEventRouter.qml
        qml/ipc/ConfigActionRequestRouter.qml
        qml/toolchain/ToolchainController.qml
        qml/toolchain/ToolchainMenu.qml
        qml/ipc/ToolchainEventRouter.qml
        qml/ipc/ToolchainRequestRouter.qml
        qml/git/GitController.qml
        qml/git/GitHistoryController.qml
        qml/git/GitChangesList.qml
        qml/git/GitHistoryList.qml
        qml/git/GitRules.qml
        qml/git/GitInspectorController.qml
        qml/git/GitPatchView.qml
        qml/git/GitInspectorPane.qml
        qml/git/GitBranchMenu.qml
        qml/git/GitCommitBox.qml
        qml/git/GitDiffDialog.qml
        qml/git/GitDiscardDialog.qml
        qml/panels/bottom/GitPanel.qml
        qml/ipc/GitEventRouter.qml
        qml/ipc/CoverageEventRouter.qml
        qml/ipc/CoverageRequestRouter.qml
        qml/coverage/CoverageController.qml
        qml/ipc/RemoteEventRouter.qml
        qml/ipc/RemoteRequestRouter.qml
        qml/remote/RemoteController.qml
        qml/remote/RemoteList.qml
        qml/remote/RemoteForm.qml
        qml/remote/RemoteVerdict.qml
        qml/remote/RemotePanel.qml
        qml/remote/RemotePanelHost.qml
        qml/remote/RemoteMirrorView.qml
        qml/ipc/GitRequestRouter.qml
        qml/ipc/DebugEventRouter.qml
        qml/ipc/DebugRequestRouter.qml
        qml/debug/DebugAttachForm.qml
        qml/panels/bottom/DebugPanel.qml
        qml/shell/BottomTabBar.qml
        qml/shell/WorkspaceStatusBar.qml
        qml/shell/StatusBarProjectSummaries.qml
        qml/panels/bottom/BottomPanelHost.qml
        qml/panels/bottom/TerminalSessionTabs.qml
        qml/panels/bottom/BuildPanel.qml
        qml/panels/bottom/IdeLogPanel.qml
        qml/panels/bottom/JobsPanel.qml
        qml/panels/bottom/ProblemsPanel.qml
        qml/panels/bottom/SearchPanel.qml
        qml/panels/bottom/TerminalPanel.qml
        qml/panels/bottom/TerminalViewport.qml
        qml/panels/bottom/TerminalSpanRules.qml
        qml/panels/bottom/SearchReplaceBar.qml
        qml/panels/bottom/TerminalGeometryOverlay.qml
        qml/panels/bottom/TerminalMetrics.qml
        qml/panels/bottom/TerminalScrollController.qml
        qml/panels/bottom/TerminalSelectionController.qml
        qml/panels/bottom/TerminalInputController.qml
        qml/panels/bottom/TestsPanel.qml
        qml/panels/bottom/ToolsPanel.qml
        qml/workspace/FolderPickerDialog.qml
        qml/workspace/FolderPickerController.qml
        qml/workspace/FolderPickerButton.qml
        qml/workspace/FolderPickerHeader.qml
        qml/workspace/FolderPickerPathBar.qml
        qml/workspace/FolderPickerCreatePanel.qml
        qml/workspace/FolderPickerDirectoryList.qml
        qml/workspace/FolderPickerActionRow.qml
        qml/workspace/FolderPickerCard.qml
        qml/workspace/WorkspaceController.qml
        qml/workspace/RecentWorkspacesController.qml
        qml/workspace/RecentWorkspacesCard.qml
        qml/workspace/StartScreen.qml
        qml/workspace/ProjectHealthController.qml
        qml/workspace/WorkspaceUiResetter.qml
        qml/jobs/JobsController.qml
        qml/runtime/RuntimeController.qml
        qml/runtime/RunConfigController.qml
        qml/search/SearchController.qml
        qml/search/SearchEverywhereController.qml
        qml/ipc/WorkspaceEventRouter.qml
        qml/ipc/EditorEventRouter.qml
        qml/ipc/EditorRequestRouter.qml
        qml/ipc/JobsEventRouter.qml
        qml/ipc/SearchEventRouter.qml
        qml/ipc/ProjectTreeRequestRouter.qml
        qml/ipc/SearchRequestRouter.qml
        qml/ipc/RuntimeEventRouter.qml
        qml/ipc/RuntimeRequestRouter.qml
        qml/project/ProjectCreateDialog.qml
        qml/project/ProjectTreeController.qml
        qml/project/ProjectTreeRevealController.qml
        qml/project/ProjectTreeRules.qml
        qml/project/ProjectExplorer.qml
        qml/project/ProjectEntryContextMenu.qml
        qml/project/ProjectEntryRenameDialog.qml
        qml/project/ProjectEntryDeleteDialog.qml
        qml/command/SearchEverywhereDialog.qml
        qml/command/CommandDispatcher.qml
        qml/editor/SymbolRenameDialog.qml
        qml/editor/EditorWorkspaceEditPreviewDialog.qml
        qml/editor/EditorController.qml
        qml/editor/EditorSurfaceBridge.qml
        qml/editor/EditorDocumentController.qml
        qml/editor/EditorTextController.qml
        qml/editor/EditorLanguageController.qml
        qml/editor/EditorHighlightController.qml
        qml/editor/EditorFormatController.qml
        qml/editor/EditorPersistenceController.qml
        qml/editor/EditorCompletionController.qml
        qml/editor/EditorPane.qml
        qml/editor/EditorTabsBar.qml
        qml/editor/EditorTextSurface.qml
        qml/editor/EditorGutter.qml
        qml/editor/EditorGutterLineMarks.qml
        qml/editor/TextRules.qml
        qml/editor/EditorBreadcrumbs.qml
        qml/editor/EditorOutlineHandle.qml
        qml/editor/EditorTextGeometry.qml
        qml/editor/EditorSelectionLadder.qml
        qml/editor/PathRules.qml
        qml/editor/EditorExternalChangeController.qml
        qml/editor/EditorJumpController.qml
        qml/editor/EditorRecentFiles.qml
        qml/editor/EditorTypingController.qml
        qml/editor/EditorLineHighlights.qml
        qml/editor/EditorDiagnosticTooltip.qml
        qml/editor/EditorHoverPopup.qml
        qml/editor/EditorCompletionPopup.qml
        qml/editor/EditorActionsPopup.qml
        qml/editor/EditorGoToLineDialog.qml
        qml/editor/EditorUsagesPopup.qml
        qml/editor/EditorFindController.qml
        qml/editor/EditorFindBar.qml
        qml/editor/EditorExternalChangeBanner.qml
        qml/editor/EditorOutlineController.qml
        qml/editor/EditorOutlinePanel.qml
    SOURCES
        src/clipboard.cpp
        src/clipboard.h
        src/documentation.cpp
        src/documentation.h
        src/core_client.cpp
        src/core_client.h
        src/debug_flags.cpp
        src/debug_flags.h
        src/core_client_dispatch.cpp
        src/core_client_notifications.cpp
        src/core_client_dispatch_cmake.cpp
        src/core_client_dispatch_lsp.cpp
        src/core_client_dispatch_debug.cpp
        src/core_client_requests_debug.cpp
        src/core_client_requests_git.cpp
        src/core_client_requests_run.cpp
        src/core_client_configaction.cpp
        src/core_client_toolchain.cpp
        src/core_client_library.cpp
        src/core_client_datasource.cpp
    src/core_client_grafana.cpp
    src/core_client_probe.cpp
    src/core_client_container.cpp
    src/core_client_index.cpp
    src/core_client_python.cpp
    src/core_client_coverage.cpp
    src/core_client_remote.cpp
        src/core_client_log.cpp
        src/core_client_process.cpp
        src/core_client_requests.cpp
        src/core_client_state.cpp
        src/editor_highlighter.cpp
        src/editor_highlighter.h
        src/editor_highlighter_palette.h
        src/editor_highlighter_rules.cpp
        src/editor_highlighter_tokens.cpp
        src/editor_highlighter_overlays.cpp
        src/editor_highlighter_folding.cpp
        src/typing_perf_harness.cpp
        src/typing_perf_harness.h
        src/window_chrome_controller.cpp
        src/window_chrome_controller.h
    RESOURCES
        assets/icons/tree/folder-closed.svg
        assets/icons/tree/folder-open.svg
        assets/icons/tree/file-c.svg
        assets/icons/tree/file-cpp.svg
        assets/icons/tree/file-rust.svg
        assets/icons/tree/file-python.svg
        "${KINEIN_MANUAL_FILE}"
)

kinein_enable_strict_compiler_options(kinein-vectis)

# Qt 6 gera registro, cache AOT e MOC com construcoes que GCC 16/Clang 22
# diagnosticam sob os warnings rigorosos da Kinein. As excecoes ficam presas
# aos fontes GERADOS; os arquivos ui/src continuam integralmente sob -Werror.
get_target_property(KINEIN_UI_SOURCES kinein-vectis SOURCES)
foreach(KINEIN_UI_SOURCE IN LISTS KINEIN_UI_SOURCES)
    if(KINEIN_UI_SOURCE MATCHES "qmltyperegistrations\\.cpp$|/\\.rcc/qmlcache/|_autogen/mocs_compilation\\.cpp$")
        set_property(
            SOURCE "${KINEIN_UI_SOURCE}"
            APPEND PROPERTY COMPILE_OPTIONS
                -w
        )
    endif()
endforeach()
set_source_files_properties(
    "${CMAKE_CURRENT_BINARY_DIR}/kinein-vectis_autogen/mocs_compilation.cpp"
    PROPERTIES COMPILE_OPTIONS -w
)

# The generated qmltyperegistrations.cpp includes headers by base name.
target_include_directories(kinein-vectis PRIVATE src)

target_link_libraries(kinein-vectis PRIVATE Qt6::Core Qt6::Gui Qt6::Qml Qt6::Quick)

# Keep the install tree AppDir/FHS-compatible. The Rust core is built and
# installed by the AppImage recipe because it is not a CMake target.
install(TARGETS kinein-vectis RUNTIME DESTINATION "${CMAKE_INSTALL_BINDIR}")
install(
    FILES assets/app-icon.png
    DESTINATION "${CMAKE_INSTALL_DATADIR}/icons/hicolor/512x512/apps"
    RENAME kinein-vectis.png
)
install(
    FILES "${CMAKE_SOURCE_DIR}/packaging/appimage/io.github.viktorwalde.KineinVectis.desktop"
    DESTINATION "${CMAKE_INSTALL_DATADIR}/applications"
)
install(
    FILES "${CMAKE_SOURCE_DIR}/packaging/appimage/io.github.viktorwalde.KineinVectis.appdata.xml"
    DESTINATION "${CMAKE_INSTALL_DATADIR}/metainfo"
)
# RENAME: a fonte e' `DocsPublic/manual.md` e o pacote promete `MANUAL.md`
# (o smoke `testar-appimage.sh` exige esse nome). Sem o RENAME o AppImage
# entregava `manual.md` e o gate reprovava o artefato — pente-fino 2026-09-18.
install(
    FILES "${KINEIN_MANUAL_FILE}"
    DESTINATION "${CMAKE_INSTALL_DATADIR}/doc/kinein-vectis"
    RENAME MANUAL.md
)
install(
    FILES
        "${CMAKE_SOURCE_DIR}/LICENSE-MIT.txt"
        "${CMAKE_SOURCE_DIR}/LICENSE-APACHE-2.0.txt"
        "${CMAKE_SOURCE_DIR}/packaging/appimage/KINEIN_LICENSE_NOTICE.txt"
    DESTINATION "${CMAKE_INSTALL_DATADIR}/doc/kinein-vectis"
)
