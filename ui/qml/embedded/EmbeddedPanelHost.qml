pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Mesma instância nos slots da HUD; o shell conserva lado e largura.
Rectangle {
    id: root

    property var controller: null
    property var toolchainController: null

    readonly property real minimumWidth: 400

    signal closeRequested()

    function prepare() {
        if (!root.visible || !root.controller || root.controller.workspaceRoot === "") return;
        root.controller.panelVisible = true;
        if (!root.controller.projectBusy) root.controller.refresh();
    }

    onVisibleChanged: {
        if (!root.controller) return;
        if (visible) root.prepare();
        else root.controller.close();
    }

    Connections {
        target: root.controller
        function onWorkspaceRootChanged() { Qt.callLater(root.prepare); }
    }

    color: Theme.surface1
    radius: Theme.radius
    border.color: Theme.borderSoft
    clip: true

    EmbeddedPanel {
        anchors.fill: parent
        anchors.margins: Theme.spacingMedium
        controller: root.controller
        toolchainController: root.toolchainController
        onCloseRequested: root.closeRequested()
    }
}
