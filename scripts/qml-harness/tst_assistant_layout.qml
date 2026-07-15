import QtQuick
import "../../ui/qml/shell"

Item {
    id: root
    width: 100
    height: 100

    ShellController {
        id: shell
        showAssistant: true
        showExplorer: true
        workspaceRoot: "/work"
        viewportWidth: 1280
        viewportHeight: 800
    }

    Component.onCompleted: {
        let failures = 0;

        if (!shell.effectiveShowExplorer) failures += 1;
        if (shell.assistantPresentationWidth !== shell.contextWidth) failures += 2;

        shell.assistantTerminalActive = true;
        if (!shell.effectiveShowExplorer) failures += 4;
        if (shell.assistantPresentationWidth !== 604) failures += 8;

        shell.resizeAssistant(-204);
        if (shell.assistantTerminalWidth !== 436
                || shell.assistantPresentationWidth !== 436) failures += 1;
        shell.toggleExplorer();
        if (shell.effectiveShowExplorer
                || shell.assistantPresentationWidth !== 436) failures += 2;
        shell.toggleExplorer();

        shell.toggleAssistantMaximized();
        if (!shell.assistantMaximized) failures += 16;

        shell.assistantTerminalActive = false;
        if (shell.assistantMaximized) failures += 32;

        shell.viewportWidth = 800;
        shell.assistantTerminalActive = true;
        if (shell.assistantPresentationWidth !== 300) failures += 64;

        shell.toggleAssistantMaximized();
        shell.closeAssistant();
        if (shell.showAssistant || shell.assistantMaximized) failures += 128;

        Qt.exit(failures);
    }
}
