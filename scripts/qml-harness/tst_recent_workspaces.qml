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

        controller.handleResolved([]);
        controller.handleRequestFailed("fs.read", "ignorar");
        if (controller.errorText !== "") failures += 64;
        controller.handleRequestFailed("workspace.recent.pin", "falhou");
        if (controller.errorText !== "falhou") failures += 128;

        Qt.exit(failures);
    }
}
