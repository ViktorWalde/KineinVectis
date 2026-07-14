import QtQuick

Rectangle {
    id: root

    property string text: ""

    implicitWidth: tooltipText.implicitWidth + 2 * Theme.spacingSmall
    implicitHeight: 24
    radius: Theme.radius
    color: Theme.background2
    border.color: Theme.borderStrong
    border.width: 1
    visible: text !== ""
    z: 1000

    Text {
        id: tooltipText

        anchors.centerIn: parent
        text: root.text
        color: Theme.textPrimary
        font.pixelSize: 11
    }
}
