import QtQuick

// Atalhos globais. Regra do repositorio (pedido do usuario, 2026-07-09):
// toda acao com tecla F ganha uma sequencia alternativa SEM F-key
// (notebooks exigem Fn) — os dois atalhos valem sempre; alem disso toda
// acao de atalho precisa ter acionamento manual por botao/comando.
Item {
    id: root

    property var editorController: null
    property var shellController: null
    property var jobsController: null
    property var runtimeController: null
    property var debugController: null
    property var searchController: null
    property var searchEverywhereController: null

    visible: false

    Shortcut {
        // comando: workspace.open
        // Ate' 2026-09-04 a paleta anunciava Ctrl+O e NENHUM Shortcut o
        // ligava: apertar nao fazia nada. Abrir pasta so' existia no menu e
        // num botao da tela vazia.
        sequences: [StandardKey.Open]
        onActivated: root.shellController.requestOpenFolder()
    }

    Shortcut {
        // comando: fs.write
        sequences: [StandardKey.Save]
        onActivated: root.editorController.saveCurrentFile()
    }

    Shortcut {
        sequence: "Ctrl+Shift+S"
        onActivated: root.editorController.saveAllFiles()
    }

    Shortcut {
        // comando: build.run
        sequences: ["Ctrl+F9", "Ctrl+Alt+B"]
        onActivated: root.jobsController.startBuild()
    }

    Shortcut {
        // comando: test.run
        sequences: ["Ctrl+Shift+F9", "Ctrl+Alt+T"]
        onActivated: root.jobsController.startTests()
    }

    Shortcut {
        // comando: debug.start
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
        // comando: quality.run
        sequence: "Ctrl+Shift+L"
        onActivated: root.jobsController.startQuality()
    }

    Shortcut {
        // comando: format.text
        sequence: "Ctrl+Alt+L"
        onActivated: root.editorController.formatCurrentFile()
    }

    Shortcut {
        // comando: lsp.codeActions
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

    // D1b (DocsPublic/roadmaps/24): Find/Replace NO ARQUIVO. Nao confundir com o
    // Ctrl+Shift+F (busca no projeto, ripgrep no core).
    Shortcut {
        // comando: editor.find
        sequence: "Ctrl+F"
        onActivated: root.editorController.openFind()
    }

    Shortcut {
        // comando: editor.replace
        sequence: "Ctrl+H"
        onActivated: root.editorController.openFindReplace()
    }

    Shortcut {
        sequence: "Ctrl+E"
        onActivated: root.searchEverywhereController.openRecentFiles()
    }

    Shortcut {
        // comando: fs.replace
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
        // comando: lsp.definition
        sequence: "Ctrl+B"
        onActivated: root.editorController.requestDefinition()
    }

    Shortcut {
        // comando: lsp.switchSourceHeader
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
        // comando: lsp.hover
        sequence: "Ctrl+Q"
        onActivated: root.editorController.requestHover()
    }

    Shortcut {
        // comando: lsp.completion
        sequence: "Ctrl+Space"
        onActivated: root.editorController.requestCompletion()
    }

    Shortcut {
        // comando: lsp.rename
        sequences: ["Shift+F6", "Ctrl+Shift+R"]
        onActivated: root.editorController.openRenameDialog()
    }

    Shortcut {
        // comando: lsp.references
        sequences: ["Alt+F7", "Ctrl+Shift+U"]
        onActivated: root.editorController.requestUsages()
    }

    Shortcut {
        // comando: fs.search
        sequence: "Ctrl+Shift+F"
        onActivated: root.searchController.openSearchPanel()
    }

    Shortcut {
        // comando: fs.findFiles
        sequence: "Ctrl+Shift+N"
        onActivated: root.searchEverywhereController.openSearchEverywhere()
    }

    Shortcut {
        // comando: index.symbols
        sequence: "Alt+7"
        onActivated: root.shellController.openSymbols("")
    }

    Shortcut {
        // comando: command.list
        sequence: "Ctrl+Shift+A"
        onActivated: root.searchEverywhereController.openSearchEverywhere()
    }

    Shortcut {
        // comando: run.start
        sequences: ["Shift+F10", "Ctrl+Alt+R"]
        onActivated: root.runtimeController.startRun("")
    }

    Shortcut {
        // comando: run.stop
        sequences: ["Ctrl+F2", "Ctrl+Alt+X"]
        onActivated: root.runtimeController.stopRun()
    }

    Shortcut {
        // comando: terminal.open
        sequences: ["Alt+F12", "Ctrl+`"]
        onActivated: root.runtimeController.openTerminalPanel()
    }
}
