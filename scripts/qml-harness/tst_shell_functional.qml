import QtQuick
import "../../ui/qml/project"
import "../../ui/qml/shell"

Item {
    id: root

    width: 1000
    height: 720

    property string createdFile: ""
    property string createdDirectory: ""
    property string executedScript: ""

    QtObject {
        id: automaticSettings

        property real explorerWidth: 280
        property real contextWidth: 360
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
        // O codigo de saida de um processo tem 8 BITS: Qt.exit(256) sai como 0.
        // Enquanto o bitmask ia direto para o exit, todo check com bit >= 256
        // era letra morta: passava verde mesmo quebrado, que e exatamente a
        // doenca que esta suite existe para impedir. O mask agora vai para a
        // SAIDA (onde nao trunca) e o exit so diz passou/falhou.
        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
