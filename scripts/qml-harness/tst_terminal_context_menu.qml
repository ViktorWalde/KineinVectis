import QtQuick
import "../../ui/qml/panels/bottom"

Item {
    id: root
    width: 640
    height: 480

    TerminalContextMenu {
        id: menu
        anchors.fill: parent
        canCopy: true
        canPaste: false
        canSelectVisible: true
        canSelectAll: true
        canUseSession: true
        canClearScrollback: false
        canCreateSession: true
        canCloseSession: true
    }

    Component.onCompleted: {
        let failures = 0;
        const entries = menu.entries();
        if (entries.length !== 8) failures += 1;
        if (entries[0].action !== "terminal.copy" || !entries[0].enabled
                || entries[0].shortcut !== "Ctrl+Shift+C") failures += 2;
        if (entries[1].action !== "terminal.paste" || entries[1].enabled
                || entries[1].shortcut !== "Ctrl+V") failures += 4;
        if (entries[2].action !== "terminal.selectAll" || !entries[2].enabled
                || entries[2].label !== "Selecionar Tudo") failures += 8;
        if (entries[3].action !== "terminal.selectVisible") failures += 4096;
        if (entries[4].action !== "terminal.clearScreen") failures += 16;
        if (entries[5].action !== "terminal.clearScrollback"
                || entries[5].enabled) failures += 32;
        if (entries[6].action !== "terminal.new" || !entries[6].enabled)
            failures += 64;
        if (entries[6].shortcut !== "") failures += 256;
        if (entries[7].action !== "terminal.close" || !entries[7].enabled)
            failures += 128;
        menu.showForItem(root, 10, 10);
        if (menu.popupX !== 10 || menu.popupY !== 10) failures += 2048;
        menu.sessionId = "another-session";
        if (menu.open) failures += 512;
        menu.showAt(10, 10);
        menu.available = false;
        if (menu.open) failures += 1024;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
