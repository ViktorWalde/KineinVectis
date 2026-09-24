import QtQuick

// Tradução de teclado Qt para sequências de um terminal xterm-256color.
Item {
    id: root

    property bool terminalActive: false
    property bool applicationCursor: false
    property bool hasSelection: false
    property string sessionId: ""
    property bool bracketedPaste: false
    property string pendingPaste: ""
    readonly property bool pastePending: pendingPaste !== ""

    signal openRequested()
    signal copyRequested()
    signal pasteRequested()
    signal clearSelectionRequested()
    signal contextMenuRequested()
    signal dataRequested(string data)

    visible: false
    onSessionIdChanged: cancelPaste()
    onTerminalActiveChanged: if (!terminalActive) cancelPaste()

    function cancelPaste() { pendingPaste = ""; }

    function requestPaste(text) {
        // Clique do meio/novos pedidos nao podem substituir nem contornar
        // uma confirmacao que ja esta aberta.
        if (pastePending || !terminalActive || text === "") return;
        // O shell assume multilinha apenas quando anuncia bracketed paste.
        // CR isolado tambem pode executar; ESC nao pode fechar o envelope.
        if ((!bracketedPaste && /[\r\n]/.test(text))
                || /[\x00-\x08\x0b\x0c\x0e-\x1f\x7f]/.test(text)) {
            pendingPaste = text;
        } else sendPaste(text);
    }

    function confirmPaste(singleLine) {
        if (!pastePending || !terminalActive) return;
        const text = singleLine ? pendingPaste.replace(/[\r\n]+$/g, "").replace(/\r\n|[\r\n]/g, " ")
                                : pendingPaste;
        cancelPaste();
        sendPaste(text);
    }

    function sendPaste(text) {
        // ESC do clipboard nao pode encerrar o envelope de bracketed paste.
        // Outros controles exigem confirmacao; LF/CRLF viram Enter do teclado.
        const normalized = text.replace(/\x1b/g, "\u241b").replace(/\r?\n/g, "\r");
        if (normalized !== "") dataRequested(bracketedPaste
                ? "\x1b[200~" + normalized + "\x1b[201~" : normalized);
    }

    function isContextMenuKey(event) {
        return event.modifiers === Qt.NoModifier && event.key === Qt.Key_Menu
                || event.modifiers === Qt.ShiftModifier && event.key === Qt.Key_F10;
    }

    function overrideShortcut(event) {
        // Shift+F10 global executa o projeto. Com foco aqui, o menu local
        // precisa receber o KeyPress antes de o Shortcut global disparar.
        if (pastePending || isContextMenuKey(event)) event.accepted = true;
    }

    function handleKey(event) {
        if (pastePending) { event.accepted = true; return; }
        const ctrl = (event.modifiers & Qt.ControlModifier) !== 0;
        const shift = (event.modifiers & Qt.ShiftModifier) !== 0;
        const alt = (event.modifiers & Qt.AltModifier) !== 0;
        // Em layouts internacionais o AltGr pode chegar junto de Ctrl+Alt.
        // Ele produz texto e nao e a tecla Meta do terminal: prefixar ESC
        // aqui quebraria caracteres como @, | e chaves.
        const altGr = (event.modifiers & Qt.GroupSwitchModifier) !== 0;
        // Ctrl+C pertence sempre ao PTY, mesmo com selecao. Copiar exige
        // Ctrl+Shift+C; a selecao nunca muda a intencao de interromper.
        if (isContextMenuKey(event)) {
            contextMenuRequested();
            event.accepted = true;
            return;
        }
        if (ctrl && shift && !alt && !altGr && event.key === Qt.Key_C) {
            copyRequested();
            event.accepted = true;
            return;
        }
        if (event.key === Qt.Key_Escape && hasSelection) {
            clearSelectionRequested();
            event.accepted = true;
            return;
        }
        if (!terminalActive) {
            openRequested();
            event.accepted = true;
            return;
        }
        if (ctrl && alt && !shift && !altGr && event.key === Qt.Key_V) {
            dataRequested("\x16");
            event.accepted = true;
            return;
        }
        if (ctrl && !alt && !altGr && event.key === Qt.Key_V) {
            pasteRequested();
            event.accepted = true;
            return;
        }
        if (!ctrl && !alt && shift && event.key === Qt.Key_Insert) {
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
            if (ctrl && !altGr && event.key === Qt.Key_Space) {
                data = "\x00";
            } else if (ctrl && !altGr
                       && event.key >= Qt.Key_A && event.key <= Qt.Key_Z) {
                data = String.fromCharCode(event.key - Qt.Key_A + 1);
            } else if (event.text !== "") {
                data = event.text;
            }
        }
        if (alt && !altGr && data !== "" && event.key !== Qt.Key_Alt) {
            data = "\x1b" + data;
        }
        if (data !== "") {
            dataRequested(data);
            event.accepted = true;
        }
    }
}
