import QtQuick
import "../../ui/qml/panels/bottom"

Item {
    id: root
    width: 100
    height: 100

    property string receivedData: ""
    property int copies: 0
    property int pastes: 0

    TerminalInputController {
        id: input
        terminalActive: true
        onDataRequested: function(value) { root.receivedData = value; }
        onCopyRequested: root.copies += 1
        onPasteRequested: root.pastes += 1
    }

    function key(code, text, modifiers) {
        return {
            "key": code,
            "text": text,
            "modifiers": modifiers,
            "accepted": false
        };
    }

    Component.onCompleted: {
        let failures = 0;

        input.handleKey(key(Qt.Key_Up, "", Qt.NoModifier));
        if (root.receivedData !== "\x1b[A") failures += 1;

        input.applicationCursor = true;
        input.handleKey(key(Qt.Key_Up, "", Qt.NoModifier));
        if (root.receivedData !== "\x1bOA") failures += 2;

        input.handleKey(key(Qt.Key_Tab, "\t", Qt.ShiftModifier));
        if (root.receivedData !== "\x1b[Z") failures += 4;

        input.handleKey(key(Qt.Key_Backtab, "", Qt.ShiftModifier));
        if (root.receivedData !== "\x1b[Z") failures += 8;

        input.handleKey(key(Qt.Key_X, "x", Qt.AltModifier));
        if (root.receivedData !== "\x1bx") failures += 16;

        input.handleKey(key(Qt.Key_At, "@", Qt.ControlModifier
                            | Qt.AltModifier | Qt.GroupSwitchModifier));
        if (root.receivedData !== "@") failures += 32;

        input.handleKey(key(Qt.Key_C, "c",
                            Qt.ControlModifier | Qt.ShiftModifier));
        input.handleKey(key(Qt.Key_V, "v",
                            Qt.ControlModifier | Qt.ShiftModifier));
        if (root.copies !== 1 || root.pastes !== 1) failures += 64;

        // O KV Context so abre a faixa quando o usuario realmente escreve:
        // cursor parado/ocioso nao pode deixar uma caixa vazia no transcript.
        input.inputRowDecoration = true;
        input.cursorVisible = true;
        input.cursorRow = 7;
        input.gridRows = 20;
        if (input.inputRowDecorationVisible) failures += 128;

        input.handleKey(key(Qt.Key_A, "a", Qt.NoModifier));
        if (!input.inputRowDecorationVisible
                || input.inputRowDecorationStartRow !== 7
                || input.inputRowDecorationEndRow !== 7) failures += 256;

        // Uma entrada longa que quebrou linha deve ter UMA faixa envolvendo
        // todo o intervalo, nao so a ultima linha do cursor.
        input.cursorRow = 8;
        if (input.inputRowDecorationStartRow !== 7
                || input.inputRowDecorationEndRow !== 8) failures += 512;

        // Enter encerra a entrada antes mesmo do proximo render; assim nao
        // sobra a caixa vazia vista na captura enquanto a CLI processa.
        input.handleKey(key(Qt.Key_Return, "", Qt.NoModifier));
        if (input.inputRowDecorationVisible
                || input.inputRowDecorationStartRow !== -1
                || input.inputRowDecorationEndRow !== -1) failures += 1024;

        // Histórico e cursor fora da grade nunca recebem uma moldura falsa.
        input.handleKey(key(Qt.Key_B, "b", Qt.NoModifier));
        input.scrollOffset = 1;
        if (input.inputRowDecorationVisible) failures += 2048;
        input.scrollOffset = 0;
        input.cursorRow = 20;
        if (input.inputRowDecorationVisible) failures += 4096;

        Qt.exit(failures);
    }
}
