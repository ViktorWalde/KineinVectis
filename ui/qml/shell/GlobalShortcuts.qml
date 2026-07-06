import QtQuick

Item {
    id: root

    property var editorController: null
    property var jobsController: null
    property var runtimeController: null
    property var searchController: null

    visible: false

    Shortcut {
        sequences: [StandardKey.Save]
        onActivated: root.editorController.saveCurrentFile()
    }

    Shortcut {
        sequence: "Ctrl+F9"
        onActivated: root.jobsController.startBuild()
    }

    Shortcut {
        sequence: "Ctrl+Shift+F9"
        onActivated: root.jobsController.startTests()
    }

    Shortcut {
        sequence: "Ctrl+Shift+L"
        onActivated: root.jobsController.startQuality()
    }

    Shortcut {
        sequence: "Ctrl+B"
        onActivated: root.editorController.requestDefinition()
    }

    Shortcut {
        sequence: "Ctrl+Q"
        onActivated: root.editorController.requestHover()
    }

    Shortcut {
        sequence: "Ctrl+Space"
        onActivated: root.editorController.requestCompletion()
    }

    Shortcut {
        sequence: "Shift+F6"
        onActivated: root.editorController.openRenameDialog()
    }

    Shortcut {
        sequence: "Alt+F7"
        onActivated: root.editorController.requestUsages()
    }

    Shortcut {
        sequence: "Ctrl+Shift+F"
        onActivated: root.searchController.openSearchPanel()
    }

    Shortcut {
        sequence: "Ctrl+Shift+N"
        onActivated: root.searchController.openSearchEverywhere()
    }

    Shortcut {
        sequence: "Ctrl+Shift+A"
        onActivated: root.searchController.openSearchEverywhere()
    }

    Shortcut {
        sequence: "Shift+F10"
        onActivated: root.runtimeController.startRun("")
    }

    Shortcut {
        sequence: "Ctrl+F2"
        onActivated: root.runtimeController.stopRun()
    }

    Shortcut {
        sequence: "Alt+F12"
        onActivated: root.runtimeController.openTerminalPanel()
    }
}
