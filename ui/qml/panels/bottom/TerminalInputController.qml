import QtQuick

// Tradução de teclado Qt para sequências de um terminal xterm-256color.
Item {
    id: root

    property bool terminalActive: false
    property bool applicationCursor: false
    property bool inputRowDecoration: false
    property bool cursorVisible: false
    property int cursorRow: 0
    property int gridRows: 0
    property int scrollOffset: 0
    property bool inputRowDecorationActive: false
    property int inputRowDecorationStartRow: -1
    property int inputRowDecorationEndRow: -1

    // O modo inline de algumas TUIs (Codex --no-alt-screen) preserva o
    // scrollback, mas deixa o compositor como uma linha solta no grid. O host
    // pode pedir um guia visual sem criar outro campo de input nem interpretar
    // o conteúdo da aplicação. O intervalo acompanha as quebras da entrada e
    // some quando ela é enviada ou deixa de pertencer ao fundo ao vivo.
    readonly property bool inputRowDecorationVisible: inputRowDecoration
            && inputRowDecorationActive && terminalActive && cursorVisible
            && scrollOffset === 0 && inputRowDecorationStartRow >= 0
            && inputRowDecorationEndRow >= inputRowDecorationStartRow
            && inputRowDecorationEndRow < gridRows

    signal openRequested()
    signal copyRequested()
    signal pasteRequested()
    signal dataRequested(string data)

    visible: false

    onTerminalActiveChanged: {
        if (!terminalActive) resetInputRowDecoration();
    }
    onInputRowDecorationChanged: {
        if (!inputRowDecoration) resetInputRowDecoration();
    }
    onCursorRowChanged: {
        if (!inputRowDecorationActive) return;
        if (cursorRow < 0 || cursorRow >= gridRows) {
            resetInputRowDecoration();
            return;
        }
        inputRowDecorationStartRow = Math.min(
            inputRowDecorationStartRow, cursorRow);
        inputRowDecorationEndRow = Math.max(
            inputRowDecorationEndRow, cursorRow);
    }

    function beginInputRowDecoration() {
        if (!inputRowDecoration || !terminalActive
                || cursorRow < 0 || cursorRow >= gridRows) {
            return;
        }
        if (!inputRowDecorationActive) {
            inputRowDecorationStartRow = cursorRow;
            inputRowDecorationEndRow = cursorRow;
            inputRowDecorationActive = true;
        }
    }

    function resetInputRowDecoration() {
        inputRowDecorationActive = false;
        inputRowDecorationStartRow = -1;
        inputRowDecorationEndRow = -1;
    }

    function notePastedInput() {
        beginInputRowDecoration();
    }

    function handleKey(event) {
        if (!terminalActive) {
            openRequested();
            event.accepted = true;
            return;
        }
        const ctrl = (event.modifiers & Qt.ControlModifier) !== 0;
        const shift = (event.modifiers & Qt.ShiftModifier) !== 0;
        const alt = (event.modifiers & Qt.AltModifier) !== 0;
        // Em layouts internacionais o AltGr pode chegar junto de Ctrl+Alt.
        // Ele produz texto e nao e a tecla Meta do terminal: prefixar ESC
        // aqui quebraria caracteres como @, | e chaves.
        const altGr = (event.modifiers & Qt.GroupSwitchModifier) !== 0;
        if (ctrl && shift && event.key === Qt.Key_C) {
            copyRequested();
            event.accepted = true;
            return;
        }
        if (ctrl && shift && event.key === Qt.Key_V) {
            pasteRequested();
            event.accepted = true;
            return;
        }
        let data = "";
        switch (event.key) {
        case Qt.Key_Return:
        case Qt.Key_Enter: data = "\r"; break;
        case Qt.Key_Backspace: data = "\x7f"; break;
        case Qt.Key_Tab: data = shift ? "\x1b[Z" : "\t"; break;
        case Qt.Key_Backtab: data = "\x1b[Z"; break;
        case Qt.Key_Escape: data = "\x1b"; break;
        case Qt.Key_Up: data = applicationCursor ? "\x1bOA" : "\x1b[A"; break;
        case Qt.Key_Down: data = applicationCursor ? "\x1bOB" : "\x1b[B"; break;
        case Qt.Key_Right: data = applicationCursor ? "\x1bOC" : "\x1b[C"; break;
        case Qt.Key_Left: data = applicationCursor ? "\x1bOD" : "\x1b[D"; break;
        case Qt.Key_Home: data = applicationCursor ? "\x1bOH" : "\x1b[H"; break;
        case Qt.Key_End: data = applicationCursor ? "\x1bOF" : "\x1b[F"; break;
        case Qt.Key_Insert: data = "\x1b[2~"; break;
        case Qt.Key_PageUp: data = "\x1b[5~"; break;
        case Qt.Key_PageDown: data = "\x1b[6~"; break;
        case Qt.Key_Delete: data = "\x1b[3~"; break;
        case Qt.Key_F1: data = "\x1bOP"; break;
        case Qt.Key_F2: data = "\x1bOQ"; break;
        case Qt.Key_F3: data = "\x1bOR"; break;
        case Qt.Key_F4: data = "\x1bOS"; break;
        case Qt.Key_F5: data = "\x1b[15~"; break;
        case Qt.Key_F6: data = "\x1b[17~"; break;
        case Qt.Key_F7: data = "\x1b[18~"; break;
        case Qt.Key_F8: data = "\x1b[19~"; break;
        case Qt.Key_F9: data = "\x1b[20~"; break;
        case Qt.Key_F10: data = "\x1b[21~"; break;
        case Qt.Key_F11: data = "\x1b[23~"; break;
        case Qt.Key_F12: data = "\x1b[24~"; break;
        default:
            if (ctrl && event.key === Qt.Key_Space) {
                data = "\x00";
            } else if (ctrl && event.key >= Qt.Key_A && event.key <= Qt.Key_Z) {
                data = String.fromCharCode(event.key - Qt.Key_A + 1);
            } else if (event.text !== "") {
                data = event.text;
            }
        }
        if (alt && !altGr && data !== "" && event.key !== Qt.Key_Alt) {
            data = "\x1b" + data;
        }
        if (data !== "") {
            if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter
                    || event.key === Qt.Key_Escape
                    || (ctrl && event.key === Qt.Key_C)) {
                resetInputRowDecoration();
            } else if (event.text !== ""
                       || ((event.key === Qt.Key_Backspace
                            || event.key === Qt.Key_Delete)
                           && inputRowDecorationActive)) {
                beginInputRowDecoration();
            }
            dataRequested(data);
            event.accepted = true;
        }
    }
}
