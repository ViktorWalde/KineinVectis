pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Uma instancia acoplada, com lado e largura conservados pelo shell.
Rectangle {
    id: root

    property var controller: null
    property bool terminalAvailable: false

    signal commandRequested(string command)
    signal closeRequested()

    readonly property real minimumWidth: 360

    function focusArea() { panel.focusArea(); }

    onVisibleChanged: {
        if (!root.controller) return;
        if (visible) {
            root.controller.prepare();
            Qt.callLater(root.focusArea);
        } else root.controller.close();
    }

    color: Theme.surface1
    radius: Theme.radius
    border.color: Theme.borderSoft
    clip: true

    SetupPanel {
        id: panel
        anchors.fill: parent
        anchors.margins: Theme.spacingMedium
        distroName: root.controller ? root.controller.distroName : ""
        tools: root.controller ? root.controller.filteredTools : []
        totalCount: root.controller ? root.controller.tools.length : 0
        terminalAvailable: root.terminalAvailable
        busy: root.controller ? root.controller.busy : false
        query: root.controller ? root.controller.query : ""
        availability: root.controller ? root.controller.availability : "all"
        expandedId: root.controller ? root.controller.expandedId : ""
        errorText: root.controller ? root.controller.errorText : ""
        onQueryEdited: text => root.controller.query = text
        onAvailabilityRequested: value => root.controller.availability = value
        onRefreshRequested: root.controller.refresh()
        onToggleRequested: id => root.controller.toggle(id)
        onCommandRequested: comando => root.commandRequested(comando)
        onCloseRequested: root.closeRequested()
    }
}
