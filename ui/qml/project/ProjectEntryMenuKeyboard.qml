import QtQuick

// Navegacao do menu existente. Indices fixos mantem mouse e teclado na mesma
// lista de acoes; executar e depurar podem estar ausentes.
QtObject {
    id: root

    property bool runnableScript: false
    property bool debuggableScript: false
    property bool singleSelection: true
    property bool fileSelectionAvailable: true
    property bool pasteAvailable: false
    property int currentAction: 0

    signal actionRequested(int index)
    signal dismissRequested()

    function available(index) {
        if (index === 2) return runnableScript;
        if (index === 3) return debuggableScript;
        if (index === 8) return pasteAvailable;
        if (index === 6 || index === 7 || index === 9 || index === 10)
            return fileSelectionAvailable;
        if (index >= 4) return singleSelection;
        return index >= 0 && index <= 12;
    }

    function move(direction) {
        for (let step = 1; step <= 13; ++step) {
            const index = (currentAction + direction * step + 52) % 13;
            if (available(index)) {
                currentAction = index;
                return;
            }
        }
    }

    function handleKey(event) {
        switch (event.key) {
        case Qt.Key_Down:
            move(1);
            return true;
        case Qt.Key_Up:
            move(-1);
            return true;
        case Qt.Key_Return:
        case Qt.Key_Enter:
        case Qt.Key_Space:
            if (available(currentAction)) actionRequested(currentAction);
            return true;
        case Qt.Key_Escape:
            dismissRequested();
            return true;
        default:
            return false;
        }
    }
}
