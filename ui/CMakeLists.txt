# Qt/QML frontend of Kinein Vectis.
# Presentation only: every action is sent to kinein-core over JSON-RPC.

find_package(Qt6 6.4 REQUIRED COMPONENTS Core Gui Qml Quick)
if(BUILD_TESTING)
    find_package(Qt6 6.4 REQUIRED COMPONENTS Test)
endif()

qt_standard_project_setup()

if(DEFINED QT_KNOWN_POLICY_QTP0004)
    qt_policy(SET QTP0004 NEW)
endif()

qt_add_executable(kinein-vectis
    src/main.cpp
)

set_source_files_properties(qml/Theme.qml PROPERTIES QT_QML_SINGLETON_TYPE TRUE)
set_source_files_properties(qml/components/TooltipController.qml PROPERTIES QT_QML_SINGLETON_TYPE TRUE)
set_source_files_properties(qml/components/KvIcon.qml PROPERTIES QT_RESOURCE_ALIAS KvIcon.qml)
set_source_files_properties(qml/components/KvTooltip.qml PROPERTIES QT_RESOURCE_ALIAS KvTooltip.qml)
set_source_files_properties(qml/components/KvTooltipHost.qml PROPERTIES QT_RESOURCE_ALIAS KvTooltipHost.qml)
set_source_files_properties(qml/components/TooltipController.qml PROPERTIES QT_RESOURCE_ALIAS TooltipController.qml)
set_source_files_properties(qml/components/KvIconButton.qml PROPERTIES QT_RESOURCE_ALIAS KvIconButton.qml)
set_source_files_properties(qml/components/KvButton.qml PROPERTIES QT_RESOURCE_ALIAS KvButton.qml)
set_source_files_properties(qml/shell/TopHeaderBar.qml PROPERTIES QT_RESOURCE_ALIAS TopHeaderBar.qml)
set_source_files_properties(qml/shell/AppMenuBar.qml PROPERTIES QT_RESOURCE_ALIAS AppMenuBar.qml)
set_source_files_properties(qml/shell/AppMenuPopup.qml PROPERTIES QT_RESOURCE_ALIAS AppMenuPopup.qml)
set_source_files_properties(qml/shell/SideRail.qml PROPERTIES QT_RESOURCE_ALIAS SideRail.qml)
set_source_files_properties(qml/shell/ShellLayout.qml PROPERTIES QT_RESOURCE_ALIAS ShellLayout.qml)
set_source_files_properties(qml/app/AppDomains.qml PROPERTIES QT_RESOURCE_ALIAS AppDomains.qml)
set_source_files_properties(qml/shell/ShellController.qml PROPERTIES QT_RESOURCE_ALIAS ShellController.qml)
set_source_files_properties(qml/shell/GlobalShortcuts.qml PROPERTIES QT_RESOURCE_ALIAS GlobalShortcuts.qml)
set_source_files_properties(qml/shell/ShellOverlays.qml PROPERTIES QT_RESOURCE_ALIAS ShellOverlays.qml)
set_source_files_properties(qml/shell/AboutDialog.qml PROPERTIES QT_RESOURCE_ALIAS AboutDialog.qml)
set_source_files_properties(qml/shell/DocumentationDialog.qml PROPERTIES QT_RESOURCE_ALIAS DocumentationDialog.qml)
set_source_files_properties(qml/shell/ShellHeaderHost.qml PROPERTIES QT_RESOURCE_ALIAS ShellHeaderHost.qml)
set_source_files_properties(qml/shell/WindowControls.qml PROPERTIES QT_RESOURCE_ALIAS WindowControls.qml)
set_source_files_properties(qml/shell/WindowResizeHandles.qml PROPERTIES QT_RESOURCE_ALIAS WindowResizeHandles.qml)
set_source_files_properties(qml/shell/ShellStatusHost.qml PROPERTIES QT_RESOURCE_ALIAS ShellStatusHost.qml)
set_source_files_properties(qml/shell/ShellWorkspaceHost.qml PROPERTIES QT_RESOURCE_ALIAS ShellWorkspaceHost.qml)
set_source_files_properties(qml/shell/ProjectHealthBanner.qml PROPERTIES QT_RESOURCE_ALIAS ProjectHealthBanner.qml)
set_source_files_properties(qml/shell/PanelSplitter.qml PROPERTIES QT_RESOURCE_ALIAS PanelSplitter.qml)
set_source_files_properties(qml/shell/VerticalScrollBar.qml PROPERTIES QT_RESOURCE_ALIAS VerticalScrollBar.qml)
set_source_files_properties(qml/shell/RunConfigDialog.qml PROPERTIES QT_RESOURCE_ALIAS RunConfigDialog.qml)
set_source_files_properties(qml/shell/RunConfigMenu.qml PROPERTIES QT_RESOURCE_ALIAS RunConfigMenu.qml)
set_source_files_properties(qml/debug/DebugController.qml PROPERTIES QT_RESOURCE_ALIAS DebugController.qml)
set_source_files_properties(qml/diagnostics/DiagnosticsController.qml PROPERTIES QT_RESOURCE_ALIAS DiagnosticsController.qml)
set_source_files_properties(qml/settings/SettingsController.qml PROPERTIES QT_RESOURCE_ALIAS SettingsController.qml)
set_source_files_properties(qml/settings/SettingsDialog.qml PROPERTIES QT_RESOURCE_ALIAS SettingsDialog.qml)
set_source_files_properties(qml/settings/SettingsToggleRow.qml PROPERTIES QT_RESOURCE_ALIAS SettingsToggleRow.qml)
set_source_files_properties(qml/ipc/SettingsEventRouter.qml PROPERTIES QT_RESOURCE_ALIAS SettingsEventRouter.qml)
set_source_files_properties(qml/git/GitController.qml PROPERTIES QT_RESOURCE_ALIAS GitController.qml)
set_source_files_properties(qml/git/GitDiffDialog.qml PROPERTIES QT_RESOURCE_ALIAS GitDiffDialog.qml)
set_source_files_properties(qml/git/GitDiscardDialog.qml PROPERTIES QT_RESOURCE_ALIAS GitDiscardDialog.qml)
set_source_files_properties(qml/panels/bottom/GitPanel.qml PROPERTIES QT_RESOURCE_ALIAS GitPanel.qml)
set_source_files_properties(qml/ipc/GitEventRouter.qml PROPERTIES QT_RESOURCE_ALIAS GitEventRouter.qml)
set_source_files_properties(qml/ipc/GitRequestRouter.qml PROPERTIES QT_RESOURCE_ALIAS GitRequestRouter.qml)
set_source_files_properties(qml/ipc/DebugEventRouter.qml PROPERTIES QT_RESOURCE_ALIAS DebugEventRouter.qml)
set_source_files_properties(qml/ipc/DebugRequestRouter.qml PROPERTIES QT_RESOURCE_ALIAS DebugRequestRouter.qml)
set_source_files_properties(qml/panels/bottom/DebugPanel.qml PROPERTIES QT_RESOURCE_ALIAS DebugPanel.qml)
set_source_files_properties(qml/shell/BottomTabBar.qml PROPERTIES QT_RESOURCE_ALIAS BottomTabBar.qml)
set_source_files_properties(qml/shell/WorkspaceStatusBar.qml PROPERTIES QT_RESOURCE_ALIAS WorkspaceStatusBar.qml)
set_source_files_properties(qml/panels/bottom/BottomPanelHost.qml PROPERTIES QT_RESOURCE_ALIAS BottomPanelHost.qml)
set_source_files_properties(qml/panels/bottom/TerminalSessionTabs.qml PROPERTIES QT_RESOURCE_ALIAS TerminalSessionTabs.qml)
set_source_files_properties(qml/panels/bottom/BuildPanel.qml PROPERTIES QT_RESOURCE_ALIAS BuildPanel.qml)
set_source_files_properties(qml/panels/bottom/IdeLogPanel.qml PROPERTIES QT_RESOURCE_ALIAS IdeLogPanel.qml)
set_source_files_properties(qml/panels/bottom/JobsPanel.qml PROPERTIES QT_RESOURCE_ALIAS JobsPanel.qml)
set_source_files_properties(qml/panels/bottom/ProblemsPanel.qml PROPERTIES QT_RESOURCE_ALIAS ProblemsPanel.qml)
set_source_files_properties(qml/panels/bottom/RunPanel.qml PROPERTIES QT_RESOURCE_ALIAS RunPanel.qml)
set_source_files_properties(qml/panels/bottom/SearchPanel.qml PROPERTIES QT_RESOURCE_ALIAS SearchPanel.qml)
set_source_files_properties(qml/panels/bottom/TerminalPanel.qml PROPERTIES QT_RESOURCE_ALIAS TerminalPanel.qml)
set_source_files_properties(qml/panels/bottom/TerminalViewport.qml PROPERTIES QT_RESOURCE_ALIAS TerminalViewport.qml)
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
set_source_files_properties(qml/editor/EditorCompletionController.qml PROPERTIES QT_RESOURCE_ALIAS EditorCompletionController.qml)
set_source_files_properties(qml/editor/EditorCodeActionController.qml PROPERTIES QT_RESOURCE_ALIAS EditorCodeActionController.qml)
set_source_files_properties(qml/editor/EditorPane.qml PROPERTIES QT_RESOURCE_ALIAS EditorPane.qml)
set_source_files_properties(qml/editor/EditorTabsBar.qml PROPERTIES QT_RESOURCE_ALIAS EditorTabsBar.qml)
set_source_files_properties(qml/editor/EditorTextSurface.qml PROPERTIES QT_RESOURCE_ALIAS EditorTextSurface.qml)
set_source_files_properties(qml/editor/EditorAutoClosePairs.qml PROPERTIES QT_RESOURCE_ALIAS EditorAutoClosePairs.qml)
set_source_files_properties(qml/editor/EditorKeyRoutes.qml PROPERTIES QT_RESOURCE_ALIAS EditorKeyRoutes.qml)
set_source_files_properties(qml/editor/EditorGutter.qml PROPERTIES QT_RESOURCE_ALIAS EditorGutter.qml)
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

set(KINEIN_MANUAL_FILE "${CMAKE_SOURCE_DIR}/MANUAL.md")
set_source_files_properties("${KINEIN_MANUAL_FILE}" PROPERTIES QT_RESOURCE_ALIAS MANUAL.md)

qt_add_qml_module(kinein-vectis
    URI KineinVectis
    VERSION 1.0
    RESOURCE_PREFIX /
    QML_FILES
        qml/Main.qml
        qml/Theme.qml
        qml/components/KvIcon.qml
        qml/components/KvTooltip.qml
        qml/components/KvTooltipHost.qml
        qml/components/TooltipController.qml
        qml/components/KvIconButton.qml
        qml/components/KvButton.qml
        qml/shell/TopHeaderBar.qml
        qml/shell/AppMenuBar.qml
        qml/shell/AppMenuPopup.qml
        qml/shell/SideRail.qml
        qml/shell/ShellLayout.qml
        qml/app/AppDomains.qml
        qml/shell/ShellController.qml
        qml/shell/GlobalShortcuts.qml
        qml/shell/ShellOverlays.qml
        qml/shell/AboutDialog.qml
        qml/shell/DocumentationDialog.qml
        qml/shell/ShellHeaderHost.qml
        qml/shell/WindowControls.qml
        qml/shell/WindowResizeHandles.qml
        qml/shell/ShellStatusHost.qml
        qml/shell/ShellWorkspaceHost.qml
        qml/shell/ProjectHealthBanner.qml
        qml/shell/PanelSplitter.qml
        qml/shell/VerticalScrollBar.qml
        qml/shell/RunConfigDialog.qml
        qml/shell/RunConfigMenu.qml
        qml/debug/DebugController.qml
        qml/diagnostics/DiagnosticsController.qml
        qml/settings/SettingsController.qml
        qml/settings/SettingsDialog.qml
        qml/settings/SettingsToggleRow.qml
        qml/ipc/SettingsEventRouter.qml
        qml/git/GitController.qml
        qml/git/GitDiffDialog.qml
        qml/git/GitDiscardDialog.qml
        qml/panels/bottom/GitPanel.qml
        qml/ipc/GitEventRouter.qml
        qml/ipc/GitRequestRouter.qml
        qml/ipc/DebugEventRouter.qml
        qml/ipc/DebugRequestRouter.qml
        qml/panels/bottom/DebugPanel.qml
        qml/shell/BottomTabBar.qml
        qml/shell/WorkspaceStatusBar.qml
        qml/panels/bottom/BottomPanelHost.qml
        qml/panels/bottom/TerminalSessionTabs.qml
        qml/panels/bottom/BuildPanel.qml
        qml/panels/bottom/IdeLogPanel.qml
        qml/panels/bottom/JobsPanel.qml
        qml/panels/bottom/ProblemsPanel.qml
        qml/panels/bottom/RunPanel.qml
        qml/panels/bottom/SearchPanel.qml
        qml/panels/bottom/TerminalPanel.qml
        qml/panels/bottom/TerminalViewport.qml
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
        qml/editor/EditorCompletionController.qml
        qml/editor/EditorCodeActionController.qml
        qml/editor/EditorPane.qml
        qml/editor/EditorTabsBar.qml
        qml/editor/EditorTextSurface.qml
        qml/editor/EditorAutoClosePairs.qml
        qml/editor/EditorKeyRoutes.qml
        qml/editor/EditorGutter.qml
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
        src/core_client_dispatch_language.cpp
        src/core_client_dispatch_debug.cpp
        src/core_client_dispatch_build.cpp
        src/core_client_log.cpp
        src/core_client_process.cpp
        src/core_client_requests.cpp
        src/core_client_state.cpp
        src/editor_highlighter.cpp
        src/editor_highlighter.h
        src/editor_highlighter_rules.cpp
        src/editor_highlighter_palette.h
        src/auto_close_regions.cpp
        src/auto_close_regions.h
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

# ---------------------------------------------------------------------------
# Testes de C++ (2026-07-17). Ate aqui a ui/src nao tinha NENHUM: os gates de
# C++ sao estaticos (clang-format, clang-tidy) e os harnesses QML evitam o
# modulo C++ por construcao. Este alvo existe para que o proximo teste tenha
# onde morar — nao para cobrir tudo de uma vez.
#
# Cada teste compila as unidades que exercita, em vez de linkar o app: o
# `kinein-vectis` e' um executavel com QML, engine e main(); transformar tudo em
# biblioteca so' para testar seria reestruturar o build antes de existir o
# primeiro teste. Quando o numero de alvos justificar, vira lib — nao antes.
if(BUILD_TESTING)
    qt_add_executable(tst_editor_highlighter
        tests/tst_editor_highlighter.cpp
        src/editor_highlighter.cpp
        src/editor_highlighter.h
        src/editor_highlighter_rules.cpp
        src/editor_highlighter_palette.h
    )
    target_include_directories(tst_editor_highlighter PRIVATE src)
    # Qt6::Quick entra so' porque `editor_highlighter.h` expoe um
    # `QQuickTextDocument*` — o realce em si nao depende do Quick. E' o primeiro
    # acoplamento que o split do arquivo deve desfazer (ver E5/E6 no PONTO_ATUAL).
    target_link_libraries(tst_editor_highlighter
        PRIVATE Qt6::Core Qt6::Gui Qt6::Qml Qt6::Quick Qt6::Test)
    # Teste roda sob as MESMAS regras do produto: warnings-as-errors e
    # sanitizers no debug. Teste com regra mais frouxa que o codigo testado e'
    # como gate que nunca reprova — passa por nao olhar.
    kinein_enable_strict_compiler_options(tst_editor_highlighter)
    # Mesma excecao do alvo principal, e pelo mesmo motivo: o MOC do Qt 6 usa
    # CTAD que o Clang 22 diagnostica sob os warnings rigorosos.
    set_source_files_properties(
        "${CMAKE_CURRENT_BINARY_DIR}/tst_editor_highlighter_autogen/mocs_compilation.cpp"
        PROPERTIES COMPILE_OPTIONS -w
    )
    # O QTEST_MAIN exige `#include "<nome>.moc"` no PROPRIO .cpp, porque a classe
    # Q_OBJECT e' local ao arquivo. O codigo gerado entra na unidade do teste e
    # nao da' para isola-lo em mocs_compilation.cpp como o alvo principal faz.
    # Suprime-se APENAS o diagnostico que o MOC dispara — todo o resto do
    # -Werror continua valendo sobre o teste, que e' o ponto.
    set_source_files_properties(tests/tst_editor_highlighter.cpp
        PROPERTIES COMPILE_OPTIONS -Wno-ctad-maybe-unsupported
    )
    add_test(NAME tst_editor_highlighter COMMAND tst_editor_highlighter)
    # offscreen: o gate roda sem servidor grafico (mesma razao do smoke).
    set_tests_properties(tst_editor_highlighter PROPERTIES
        ENVIRONMENT "QT_QPA_PLATFORM=offscreen")

    # E6: rastreio dos fechadores auto-inseridos (QTextCursor). Mesmo padrao do
    # alvo acima — inclusive as duas excecoes de codigo GERADO pelo MOC.
    qt_add_executable(tst_auto_close_regions
        tests/tst_auto_close_regions.cpp
        src/auto_close_regions.cpp
        src/auto_close_regions.h
    )
    target_include_directories(tst_auto_close_regions PRIVATE src)
    target_link_libraries(tst_auto_close_regions
        PRIVATE Qt6::Core Qt6::Gui Qt6::Qml Qt6::Quick Qt6::Test)
    kinein_enable_strict_compiler_options(tst_auto_close_regions)
    set_source_files_properties(
        "${CMAKE_CURRENT_BINARY_DIR}/tst_auto_close_regions_autogen/mocs_compilation.cpp"
        PROPERTIES COMPILE_OPTIONS -w
    )
    set_source_files_properties(tests/tst_auto_close_regions.cpp
        PROPERTIES COMPILE_OPTIONS -Wno-ctad-maybe-unsupported
    )
    add_test(NAME tst_auto_close_regions COMMAND tst_auto_close_regions)
    set_tests_properties(tst_auto_close_regions PROPERTIES
        ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
endif()

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
install(
    FILES "${KINEIN_MANUAL_FILE}"
    DESTINATION "${CMAKE_INSTALL_DATADIR}/doc/kinein-vectis"
)
install(
    FILES
        "${CMAKE_SOURCE_DIR}/LICENSE-MIT.txt"
        "${CMAKE_SOURCE_DIR}/LICENSE-APACHE-2.0.txt"
        "${CMAKE_SOURCE_DIR}/packaging/appimage/KINEIN_LICENSE_NOTICE.txt"
    DESTINATION "${CMAKE_INSTALL_DATADIR}/doc/kinein-vectis"
)
