import QtQuick
import KineinVectis

Rectangle {
    id: root

    property string text: ""
    property bool primary: false

    signal clicked()

    width: label.width + 2 * Theme.spacingMedium
    height: 30
    radius: Theme.radius
    opacity: enabled ? 1.0 : 0.45
    color: root.primary
           ? (buttonArea.pressed ? Theme.accentDim : Theme.accent)
           : (buttonArea.containsMouse ? Theme.surface2 : Theme.background2)
    border.color: root.primary ? "transparent" : Theme.borderSoft
    border.width: root.primary ? 0 : 1

    Text {
        id: label

        anchors.centerIn: parent
        text: root.text
        color: root.primary ? Theme.background0 : Theme.textSecondary
        font.pixelSize: 12
        font.bold: root.primary
    }

    MouseArea {
        id: buttonArea

        anchors.fill: parent
        enabled: root.enabled
        hoverEnabled: true
        cursorShape: root.enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
        onClicked: root.clicked()
    }
}
