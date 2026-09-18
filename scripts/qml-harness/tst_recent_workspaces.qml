import QtQuick
import "../../ui/qml/workspace"

Item {
    id: root

    property string opened: ""
    property string pinnedRoot: ""
    property bool pinnedValue: false
    property string removed: ""
    property bool cleared: false

    RecentWorkspacesController {
        id: controller

        onOpenRequested: function(rootPath) { root.opened = rootPath; }
        onPinRequested: function(rootPath, pinned) {
            root.pinnedRoot = rootPath;
            root.pinnedValue = pinned;
        }
        onRemoveRequested: function(rootPath) { root.removed = rootPath; }
        onClearRequested: root.cleared = true
    }

    Component.onCompleted: {
        let failures = 0;
        controller.handleResolved([
            {
                name: "demo",
                root: "/tmp/demo",
                lastOpenedAt: 20,
                pinned: false,
                available: true
            },
            {
                name: "missing",
                root: "/tmp/missing",
                lastOpenedAt: 10,
                pinned: true,
                available: false
            }
        ]);
        if (controller.workspaces.length !== 2 || controller.errorText !== "") failures += 1;

        controller.openWorkspace("/tmp/demo");
        if (root.opened !== "/tmp/demo") failures += 2;
        controller.openWorkspace("/tmp/missing");
        if (root.opened !== "/tmp/demo" || controller.errorText === "") failures += 4;

        controller.togglePinned("/tmp/demo");
        if (root.pinnedRoot !== "/tmp/demo" || !root.pinnedValue) failures += 8;
        controller.removeWorkspace("/tmp/missing");
        if (root.removed !== "/tmp/missing") failures += 16;
        controller.clearAll();
        if (!root.cleared) failures += 32;

        // F7: o de caminho ausente some da lista (com desfazer); o ultimo
        // aberto vem em destaque, mesmo com um fixado na frente; Enter abre.
        controller.handleResolved([
            { name: "fixado", root: "/tmp/fixado", lastOpenedAt: 5, pinned: true, available: true },
            { name: "ultimo", root: "/tmp/ultimo", lastOpenedAt: 30, pinned: false, available: true },
            { name: "sumiu", root: "/tmp/sumiu", lastOpenedAt: 20, pinned: false, available: false },
            { name: "antigo", root: "/tmp/antigo", lastOpenedAt: 10, pinned: false, available: true }
        ]);
        if (controller.visibleWorkspaces.length !== 3 || controller.hiddenMissingCount !== 1) failures += 256;
        if (controller.highlightedIndex !== 1) failures += 512;
        root.opened = "";
        controller.openHighlighted();
        if (root.opened !== "/tmp/ultimo") failures += 1024;
        controller.moveHighlight(-1);
        controller.moveHighlight(-1);
        if (controller.highlightedIndex !== 0) failures += 2048;
        controller.moveHighlight(1);
        controller.moveHighlight(1);
        controller.moveHighlight(1);
        if (controller.highlightedIndex !== 2 || controller.visibleWorkspaces[2].root !== "/tmp/antigo") failures += 4096;
        controller.restoreMissing();
        if (controller.visibleWorkspaces.length !== 4 || controller.hiddenMissingCount !== 0
            || controller.visibleWorkspaces[controller.highlightedIndex].root !== "/tmp/ultimo") failures += 8192;
        controller.handleResolved([]);
        if (controller.highlightedIndex !== -1) failures += 16384;
        controller.moveHighlight(1);
        root.opened = "";
        controller.openHighlighted();
        if (controller.highlightedIndex !== -1 || root.opened !== "") failures += 32768;

        controller.handleRequestFailed("fs.read", "ignorar");
        if (controller.errorText !== "") failures += 64;
        controller.handleRequestFailed("workspace.recent.pin", "falhou");
        if (controller.errorText !== "falhou") failures += 128;
        // O codigo de saida de um processo tem 8 BITS: Qt.exit(256) sai como 0.
        // Enquanto o bitmask ia direto para o exit, todo check com bit >= 256
        // era letra morta: passava verde mesmo quebrado, que e exatamente a
        // doenca que esta suite existe para impedir. O mask agora vai para a
        // SAIDA (onde nao trunca) e o exit so diz passou/falhou.
        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
