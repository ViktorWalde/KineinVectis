import QtQuick
import "../../ui/qml/panels/bottom"

Item {
    id: root
    width: 100
    height: 100

    property string receivedData: ""
    property int copies: 0
    property int pastes: 0
    property int opens: 0
    property int selectionClears: 0
    property int menus: 0

    TerminalInputController {
        id: input
        terminalActive: true
        onDataRequested: function(value) { root.receivedData = value; }
        onCopyRequested: root.copies += 1
        onPasteRequested: root.pastes += 1
        onClearSelectionRequested: root.selectionClears += 1
        onOpenRequested: root.opens += 1
        onContextMenuRequested: root.menus += 1
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
        input.handleKey(key(Qt.Key_Q, "@", Qt.ControlModifier
                            | Qt.AltModifier | Qt.GroupSwitchModifier));
        if (root.receivedData !== "@") failures += 32768;

        input.handleKey(key(Qt.Key_C, "c",
                            Qt.ControlModifier | Qt.ShiftModifier));
        input.handleKey(key(Qt.Key_V, "v",
                            Qt.ControlModifier | Qt.ShiftModifier));
        if (root.copies !== 1 || root.pastes !== 1) failures += 64;

        input.handleKey(key(Qt.Key_V, "v", Qt.ControlModifier));
        input.handleKey(key(Qt.Key_Insert, "", Qt.ShiftModifier));
        if (root.pastes !== 3) failures += 2048;

        input.hasSelection = true;
        root.receivedData = "nao-alterar";
        input.handleKey(key(Qt.Key_C, "c", Qt.ControlModifier));
        if (root.copies !== 1 || root.receivedData !== "\x03") failures += 4096;
        root.receivedData = "nao-alterar";
        input.handleKey(key(Qt.Key_C, "c", Qt.ControlModifier));
        if (root.copies !== 1 || root.receivedData !== "\x03") failures += 2097152;
        root.receivedData = "nao-alterar";
        input.handleKey(key(Qt.Key_C, "c", Qt.ControlModifier | Qt.ShiftModifier));
        if (root.copies !== 2 || root.receivedData !== "nao-alterar") failures += 4194304;
        input.handleKey(key(Qt.Key_Escape, "", Qt.NoModifier));
        if (root.selectionClears !== 1) failures += 8192;
        input.hasSelection = false;

        input.handleKey(key(Qt.Key_Return, "", Qt.NoModifier));
        if (root.receivedData !== "\r") failures += 128;

        input.handleKey(key(Qt.Key_Backspace, "", Qt.NoModifier));
        if (root.receivedData !== "\x7f") failures += 256;

        input.handleKey(key(Qt.Key_C, "c", Qt.ControlModifier));
        if (root.receivedData !== "\x03") failures += 512;

        input.handleKey(key(Qt.Key_V, "v",
                            Qt.ControlModifier | Qt.AltModifier));
        if (root.receivedData !== "\x16") failures += 16384;

        input.handleKey(key(Qt.Key_F10, "", Qt.ShiftModifier));
        input.handleKey(key(Qt.Key_Menu, "", Qt.NoModifier));
        if (root.menus !== 2 || root.receivedData !== "\x16") failures += 65536;
        const menuOverride = key(Qt.Key_F10, "", Qt.ShiftModifier);
        input.overrideShortcut(menuOverride);
        if (!menuOverride.accepted) failures += 262144;
        const paletteOverride = key(Qt.Key_A, "", Qt.ControlModifier | Qt.ShiftModifier);
        input.overrideShortcut(paletteOverride);
        if (paletteOverride.accepted) failures += 524288;
        input.handleKey(key(Qt.Key_A, "a", Qt.ControlModifier));
        if (root.receivedData !== "\x01") failures += 1048576;
        input.handleKey(key(Qt.Key_C, "c", Qt.ControlModifier | Qt.AltModifier | Qt.ShiftModifier));
        if (root.copies !== 2 || root.receivedData !== "\x1b\x03") failures += 131072;

        // Como em um terminal integrado, o componente traduz apenas teclado
        // para bytes do PTY; prompt, cursor e entrada pertencem ao grid VT.
        input.terminalActive = false;
        const unopened = key(Qt.Key_A, "a", Qt.NoModifier);
        input.handleKey(unopened);
        if (root.opens !== 1 || !unopened.accepted) failures += 1024;
        // O codigo de saida de um processo tem 8 BITS: Qt.exit(256) sai como 0.
        // Enquanto o bitmask ia direto para o exit, todo check com bit >= 256
        // era letra morta: passava verde mesmo quebrado, que e exatamente a
        // doenca que esta suite existe para impedir. O mask agora vai para a
        // SAIDA (onde nao trunca) e o exit so diz passou/falhou.
        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
