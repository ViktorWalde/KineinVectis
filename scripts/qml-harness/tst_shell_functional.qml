import QtQuick
import "../../ui/qml/assistant"
import "../../ui/qml/project"
import "../../ui/qml/shell"

Item {
    id: root

    width: 1000
    height: 720

    property string createdFile: ""
    property string createdDirectory: ""
    property string openedProfile: ""
    property string closedTerminal: ""
    property string preferredProfile: ""
    property string executedScript: ""
    property int profilesRequests: 0

    QtObject {
        id: automaticSettings

        property real explorerWidth: 280
        property real contextWidth: 360
        property real assistantTerminalWidth: 640
        property real bottomPanelHeight: 260
        property real outlineWidth: 220
        property bool outlineCollapsed: false

        function hasPersistedLayout() {
            return false;
        }
    }

    ShellController {
        id: shell
    }

    ProjectTreeController {
        id: projectTree

        workspaceRoot: "/work"
        hostWidth: root.width
        hostHeight: root.height
        onCreateFileRequested: path => root.createdFile = path
        onCreateDirectoryRequested: path => root.createdDirectory = path
        onRunScriptRequested: path => root.executedScript = path
    }

    AssistantController {
        id: assistant

        onProfilesRequested: root.profilesRequests += 1
        onTerminalOpenRequested: profileId => root.openedProfile = profileId
        onTerminalCloseRequested: id => root.closedTerminal = id
        onProfilePreferenceRequested: profileId => root.preferredProfile = profileId
    }

    Component.onCompleted: {
        let failures = 0;

        shell.updateViewport(width, height);
        shell.applySettings(automaticSettings);
        if (shell.explorerWidth !== 220) failures += 1;
        if (shell.contextWidth !== 300) failures += 1;
        if (!shell.outlineCollapsed) failures += 1;

        shell.resizeExplorer(-1000);
        if (shell.explorerWidth !== 220) failures += 1;
        shell.resizeContext(1000);
        if (shell.contextWidth !== 480) failures += 1;
        shell.resizeOutline(1000);
        if (shell.outlineWidth !== 420) failures += 1;
        const collapsed = shell.outlineCollapsed;
        shell.toggleOutline();
        if (shell.outlineCollapsed === collapsed) failures += 1;
        shell.showAssistant = true;
        if (!shell.effectiveShowExplorer) failures += 1;
        shell.workspaceBuildSystems = ["cargo", "cmake"];
        if (shell.kindLabel("rustCargo") !== "Cargo + CMake") failures += 1;

        projectTree.openEntryMenu("/work/src/main.cpp", "file",
                                  "main.cpp", 990, 710);
        projectTree.openEntryCreate("directory");
        if (!projectTree.createDialogVisible
                || projectTree.createDialogParentPath !== "/work/src") {
            failures += 1;
        }
        projectTree.confirmCreateEntry("generated");
        if (createdDirectory !== "/work/src/generated") failures += 1;

        projectTree.openEntryMenu("/work/src", "directory", "src", 10, 10);
        projectTree.openEntryCreate("file");
        projectTree.confirmCreateEntry("device.rs");
        if (createdFile !== "/work/src/device.rs") failures += 1;
        projectTree.confirmCreateEntry("invalid/name");
        if (projectTree.createDialogError === "") failures += 1;

        projectTree.openEntryMenu("/work/scripts/check.sh", "file",
                                  "check.sh", 10, 10);
        if (!projectTree.entryMenuRunnable) failures += 1;
        projectTree.runEntryScript();
        if (executedScript !== "/work/scripts/check.sh"
                || projectTree.entryMenuVisible) failures += 1;
        projectTree.openEntryMenu("/work/src/main.cpp", "file",
                                  "main.cpp", 10, 10);
        if (projectTree.entryMenuRunnable) failures += 1;

        assistant.handleProfiles([
            { id: "claude", name: "Claude", command: "claude", available: false },
            { id: "codex", name: "Codex", command: "codex", available: true }
        ], "claude");
        if (assistant.selectedProfileId !== "codex") failures += 1;
        assistant.selectProfile("claude");
        assistant.startSelectedProfile();
        if (assistant.errorText === "" || openedProfile !== "") failures += 1;
        assistant.selectProfile("codex");
        assistant.startSelectedProfile();
        if (openedProfile !== "codex" || !assistant.loading) failures += 1;
        assistant.handleTerminalOpened("ai-1", "codex", "Codex", "codex");
        if (assistant.sessionId !== "ai-1" || preferredProfile !== "codex") {
            failures += 1;
        }
        assistant.handleTerminalRender({ id: "other", lines: ["ignored"] });
        if (assistant.terminalRender.id !== undefined) failures += 1;
        assistant.handleTerminalRender({ id: "ai-1", lines: ["ready"] });
        if (assistant.terminalRender.lines[0] !== "ready") failures += 1;
        assistant.switchProfile();
        if (closedTerminal !== "ai-1" || assistant.sessionId !== ""
                || profilesRequests !== 1) failures += 1;

        Qt.exit(Math.min(failures, 255));
    }
}
