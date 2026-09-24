import QtQuick
import KineinVectis

// Politica de acoes antes embutida no TerminalPanel: menu/teclado convergem
// aqui. Selecao, input e scroll continuam pertencendo aos donos existentes.
Item {
    id: root
    readonly property TerminalInputController input: inputController
    property string sessionId: ""
    property bool applicationCursor: false
    property bool bracketedPaste: false
    property TerminalSelectionController selectionController: null
    property TerminalScrollController scrollController: null
    property bool terminalActive: false
    property bool canSelectAll: false
    property bool workspaceAvailable: false
    property bool sessionAvailable: false

    signal keyPressed(string data)
    signal clearScrollbackRequested()
    signal newRequested()
    signal closeRequested()
    signal focusRequested()
    signal openRequested()
    signal contextMenuRequested()

    visible: false

    TerminalInputController {
        id: inputController
        terminalActive: root.terminalActive
        applicationCursor: root.applicationCursor
        sessionId: root.sessionId
        bracketedPaste: root.bracketedPaste
        hasSelection: root.selectionController ? root.selectionController.hasSelection : false
        onOpenRequested: root.openRequested()
        onCopyRequested: root.perform("terminal.copy")
        onPasteRequested: root.perform("terminal.paste")
        onClearSelectionRequested: root.selectionController.clear()
        onContextMenuRequested: root.contextMenuRequested()
        onDataRequested: function(data) {
            root.scrollController.snapToBottom();
            root.keyPressed(data);
        }
    }

    function perform(action) {
        if (action === "terminal.copy") selectionController.copySelection();
        else if (action === "terminal.paste") inputController.requestPaste(Clipboard.text());
        else if (action === "terminal.selectAll" && canSelectAll) selectionController.selectAll();
        else if (action === "terminal.selectVisible") selectionController.selectVisible();
        else if (action === "terminal.clearScreen" && terminalActive) {
            selectionController.clear();
            scrollController.snapToBottom(); keyPressed("\x0c");
        } else if (action === "terminal.clearScrollback" && terminalActive) {
            selectionController.clear(); scrollController.snapToBottom(); clearScrollbackRequested();
        } else if (action === "terminal.new" && workspaceAvailable) newRequested();
        else if (action === "terminal.close" && sessionAvailable) closeRequested();
        focusRequested();
    }
}
