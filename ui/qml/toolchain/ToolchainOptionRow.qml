import QtQuick
import KineinVectis

// Uma opcao de um papel da toolchain: marca de escolhida, nome e dica.
Rectangle {
    id: root

    property string label: ""
    property string hint: ""
    property bool chosen: false

    signal clicked()

    height: 28
    radius: Theme.radius
    color: optionArea.containsMouse ? Theme.surface2 : "transparent"

    KvIcon {
        id: mark

        anchors.left: parent.left
        anchors.leftMargin: Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        name: "check"
        size: 14
        visible: root.chosen
        active: true
    }

    Text {
        id: labelText

        anchors.left: parent.left
        anchors.leftMargin: Theme.spacingSmall + 14 + Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        width: Math.min(implicitWidth, root.width - x - Theme.spacingSmall)
        text: root.label
        color: root.chosen ? Theme.accent : Theme.textPrimary
        font.pixelSize: Theme.fontSizeBody
        elide: Text.ElideMiddle
    }

    Text {
        anchors.left: labelText.right
        anchors.leftMargin: Theme.spacingSmall
        anchors.right: parent.right
        anchors.rightMargin: Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        visible: root.hint !== ""
        text: root.hint
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeSmall
        elide: Text.ElideRight
    }

    MouseArea {
        id: optionArea

        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: root.clicked()
    }
}
