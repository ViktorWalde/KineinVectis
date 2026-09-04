pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O painel de instalacao dentro de um dialogo, como os outros overlays.
Item {
    id: root

    property var controller: null
    property real maxAvailableWidth: 720
    property real maxAvailableHeight: 520

    signal dismissRequested()
    signal commandRequested(string command)

    MouseArea {
        anchors.fill: parent
        onClicked: root.dismissRequested()
    }

    Rectangle {
        anchors.centerIn: parent
        width: Math.min(620, root.maxAvailableWidth)
        height: Math.min(500, root.maxAvailableHeight)
        radius: Theme.radius
        color: Theme.background1
        border.width: 1
        border.color: Theme.borderStrong

        MouseArea {
            anchors.fill: parent
        }

        SetupPanel {
            anchors.fill: parent
            anchors.margins: Theme.spacingMedium

            distroName: root.controller ? root.controller.distroName : ""
            tools: root.controller ? root.controller.tools : []
            expandedId: root.controller ? root.controller.expandedId : ""
            errorText: root.controller ? root.controller.errorText : ""

            onToggleRequested: id => root.controller.toggle(id)
            onCommandRequested: comando => root.commandRequested(comando)
            onCloseRequested: root.dismissRequested()
        }
    }
}
