pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: root

    property real menuX: 0
    property real menuY: 0
    property var items: []

    signal dismissRequested()
    signal actionRequested(string action)

    focus: visible

    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onClicked: root.dismissRequested()
    }

    Rectangle {
        x: Math.max(Theme.spacingSmall,
                    Math.min(root.menuX,
                             root.width - width - Theme.spacingSmall))
        y: Math.max(Theme.spacingSmall,
                    Math.min(root.menuY,
                             root.height - height - Theme.spacingSmall))
        width: 220
        height: menuColumn.implicitHeight + 2 * Theme.spacingSmall
        radius: Theme.radiusLarge
        color: Theme.background2
        border.color: Theme.borderStrong
        border.width: 1

        MouseArea {
            anchors.fill: parent
        }

        Column {
            id: menuColumn

            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.margins: Theme.spacingSmall

            Repeater {
                model: root.items

                delegate: Rectangle {
                    id: menuItem

                    required property var modelData

                    width: menuColumn.width
                    height: 28
                    radius: Theme.radius
                    color: itemArea.containsMouse && modelData.enabled
                           ? Theme.surfaceSelected : "transparent"
                    opacity: modelData.enabled ? 1.0 : 0.72

                    Text {
                        anchors.left: parent.left
                        anchors.leftMargin: Theme.spacingSmall
                        anchors.verticalCenter: parent.verticalCenter
                        text: menuItem.modelData.label
                        color: menuItem.modelData.enabled
                               ? Theme.textPrimary : Theme.textDisabled
                        font.pixelSize: 12
                    }

                    MouseArea {
                        id: itemArea

                        anchors.fill: parent
                        enabled: menuItem.modelData.enabled
                        hoverEnabled: true
                        cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
                        onClicked: root.actionRequested(menuItem.modelData.action)
                    }
                }
            }
        }
    }

    Keys.onEscapePressed: root.dismissRequested()
}
