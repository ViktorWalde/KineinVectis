import QtQuick
import KineinVectis

// Uma linha da coluna de locais: icone, nome e o caminho na dica.
Rectangle {
    id: root

    property string iconName: "folder"
    property string label: ""
    property string hint: ""
    property bool current: false

    signal clicked()

    height: 30
    radius: Theme.radius
    color: root.current ? Theme.surfaceSelected
                        : (rowMouse.containsMouse ? Theme.surface2 : "transparent")

    Behavior on color {
        ColorAnimation { duration: Theme.motionFast }
    }

    KvIcon {
        id: icon

        anchors.left: parent.left
        anchors.leftMargin: Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        name: root.iconName
        size: 16
        active: root.current
    }

    Text {
        anchors.left: icon.right
        anchors.leftMargin: Theme.spacingSmall
        anchors.right: parent.right
        anchors.rightMargin: Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        text: root.label
        color: root.current ? Theme.textPrimary : Theme.textSecondary
        font.pixelSize: Theme.fontSizeBody
        elide: Text.ElideRight
    }

    MouseArea {
        id: rowMouse

        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: root.clicked()
        onContainsMouseChanged: {
            if (containsMouse) {
                TooltipController.showFor(root, root.hint, "right");
            } else {
                TooltipController.hideFor(root);
            }
        }
    }
}
