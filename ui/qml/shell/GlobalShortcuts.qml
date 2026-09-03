import QtQuick

// Atalhos globais. Regra do repositorio (pedido do usuario, 2026-07-09):
// toda acao com tecla F ganha uma sequencia alternativa SEM F-key
// (notebooks exigem Fn) — os dois atalhos valem sempre; alem disso toda
// acao de atalho precisa ter acionamento manual por botao/comando.
Item {
    id: root

    property var editorController: null
    property var jobsController: null
    property var runtimeController: null
    property var debugController: null
    property var searchController: null
    property var searchEverywhereController: null
    property var settingsController: null
    property var configActionController: null

    visible: false

    Shortcut {
        sequences: [StandardKey.Save]
        onActivated: root.editorController.saveCurrentFile()
    }

    Shortcut {
        sequence: "Ctrl+Shift+S"
        onActivated: root.editorController.saveAllFiles()
    }

    Shortcut {
        sequences: ["Ctrl+F9", "Ctrl+Alt+B"]
        onActivated: root.jobsController.startBuild()
    }

    Shortcut {
        sequences: ["Ctrl+Shift+F9", "Ctrl+Alt+T"]
        onActivated: root.jobsController.startTests()
    }

    Shortcut {
        sequences: ["Shift+F9", "Ctrl+Alt+D"]
        onActivated: root.debugController.startDebug()
    }

    Shortcut {
        sequences: ["F9", "Ctrl+Alt+C"]
        enabled: root.debugController.sessionActive
        onActivated: root.debugController.continueDebug()
    }

    Shortcut {
        sequences: ["F8", "Ctrl+Alt+N"]
        enabled: root.debugController.sessionActive
        onActivated: root.debugController.stepOver()
    }

    Shortcut {
        sequences: ["F7", "Ctrl+Alt+I"]
        enabled: root.debugController.sessionActive
        onActivated: root.debugController.stepInto()
    }

    Shortcut {
        sequences: ["Shift+F8", "Ctrl+Alt+U"]
        enabled: root.debugController.sessionActive
        onActivated: root.debugController.stepOutOf()
    }

    Shortcut {
        sequence: "Ctrl+Shift+L"
        onActivated: root.jobsController.startQuality()
    }

    Shortcut {
        sequence: "Ctrl+Alt+L"
        onActivated: root.editorController.formatCurrentFile()
    }

    Shortcut {
        sequence: "Alt+Return"
        onActivated: root.editorController.requestCodeActions()
    }

    Shortcut {
        sequence: "Ctrl+D"
        onActivated: root.editorController.duplicateLine()
    }

    Shortcut {
        sequence: "Alt+Shift+Up"
        onActivated: root.editorController.moveLineUp()
    }

    Shortcut {
        sequence: "Alt+Shift+Down"
        onActivated: root.editorController.moveLineDown()
    }

    Shortcut {
        sequence: "Ctrl+/"
        onActivated: root.editorController.toggleComment()
    }

    Shortcut {
        sequence: "Ctrl+Y"
        onActivated: root.editorController.deleteLine()
    }

    Shortcut {
        sequence: "Ctrl+G"
        onActivated: root.editorController.openGoToLine()
    }

    // D1b (docs/roadmaps/24): Find/Replace NO ARQUIVO. Nao confundir com o
    // Ctrl+Shift+F (busca no projeto, ripgrep no core).
    Shortcut {
        sequence: "Ctrl+F"
        onActivated: root.editorController.openFind()
    }

    Shortcut {
        sequence: "Ctrl+H"
        onActivated: root.editorController.openFindReplace()
    }

    Shortcut {
        sequence: "Ctrl+E"
        onActivated: root.searchEverywhereController.openRecentFiles()
    }

    Shortcut {
        sequence: "Ctrl+Shift+H"
        onActivated: root.searchController.openReplacePanel()
    }

    // F3/Shift+F3 navegam mesmo com a barra fechada (reusam o ultimo termo).
    // Alternativa SEM F-key pela regra do repositorio; acionamento manual
    // pelos botões anterior/próximo da barra.
    Shortcut {
        sequences: ["F3", "Ctrl+Alt+G"]
        onActivated: root.editorController.findNext()
    }

    Shortcut {
        sequences: ["Shift+F3", "Ctrl+Alt+Shift+G"]
        onActivated: root.editorController.findPrevious()
    }

    Shortcut {
        sequence: "Ctrl+W"
        onActivated: root.editorController.expandSelection()
    }

    Shortcut {
        sequence: "Ctrl+Shift+W"
        onActivated: root.editorController.shrinkSelection()
    }

    Shortcut {
        sequence: "Ctrl+B"
        onActivated: root.editorController.requestDefinition()
    }

    Shortcut {
        sequence: "Alt+O"
        onActivated: root.editorController.requestSwitchSourceHeader()
    }

    Shortcut {
        sequences: ["F2", "Ctrl+Alt+E"]
        onActivated: root.editorController.goToNextDiagnostic()
    }

    Shortcut {
        sequences: ["Shift+F2", "Ctrl+Alt+Shift+E"]
        onActivated: root.editorController.goToPrevDiagnostic()
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
        sequences: ["Shift+F6", "Ctrl+Shift+R"]
        onActivated: root.editorController.openRenameDialog()
    }

    Shortcut {
        sequences: ["Alt+F7", "Ctrl+Shift+U"]
        onActivated: root.editorController.requestUsages()
    }

    Shortcut {
        sequence: "Ctrl+Shift+F"
        onActivated: root.searchController.openSearchPanel()
    }

    Shortcut {
        sequence: "Ctrl+Alt+S"
        onActivated: root.settingsController.openDialog()
    }

    Shortcut {
        sequence: "Ctrl+Alt+P"
        onActivated: root.configActionController.openDialog()
    }

    Shortcut {
        sequence: "Ctrl+Shift+N"
        onActivated: root.searchEverywhereController.openSearchEverywhere()
    }

    Shortcut {
        sequence: "Ctrl+Shift+A"
        onActivated: root.searchEverywhereController.openSearchEverywhere()
    }

    Shortcut {
        sequences: ["Shift+F10", "Ctrl+Alt+R"]
        onActivated: root.runtimeController.startRun("")
    }

    Shortcut {
        sequences: ["Ctrl+F2", "Ctrl+Alt+X"]
        onActivated: root.runtimeController.stopRun()
    }

    Shortcut {
        sequences: ["Alt+F12", "Ctrl+`"]
        onActivated: root.runtimeController.openTerminalPanel()
    }
}
