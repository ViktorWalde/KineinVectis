import QtQuick

Rectangle {
    id: root

    property string hoverText: ""
    property real maxAvailableWidth: 460

    signal dismissRequested()

    width: Math.min(460, maxAvailableWidth)
    height: Math.min(180, hoverTextItem.contentHeight + 2 * Theme.spacingMedium)
    radius: Theme.radius
    color: Theme.background2
    border.color: Theme.accentDim
    border.width: 1
    clip: true

    Flickable {
        anchors.fill: parent
        anchors.margins: Theme.spacingMedium
        contentWidth: width
        contentHeight: hoverTextItem.contentHeight
        clip: true

        Text {
            id: hoverTextItem

            width: parent.width
            text: root.hoverText
            color: Theme.textPrimary
            font.family: Theme.monoFont
            font.pixelSize: 11
            wrapMode: Text.WrapAnywhere
        }
    }

    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.RightButton
        onClicked: root.dismissRequested()
    }
}
