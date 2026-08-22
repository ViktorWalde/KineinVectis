import QtQuick
import KineinVectis

// Uma linha do menu de alvos. Arquivo próprio porque o delegate do Repeater e
// a entrada fixa "Todos os alvos" são a MESMA linha — duplicá-la deixaria as
// duas divergirem no primeiro ajuste visual.
Rectangle {
    id: row

    property string label: ""
    property string kind: ""
    property bool selected: false

    signal chosen()

    height: 26
    color: mouse.containsMouse ? Theme.surface2 : "transparent"
    radius: Theme.radiusXSmall

    Text {
        anchors.left: parent.left
        anchors.leftMargin: Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        width: parent.width - kindLabel.width - 3 * Theme.spacingSmall
        text: row.label
        color: row.selected ? Theme.accentActive : Theme.textPrimary
        font.pixelSize: 12
        font.bold: row.selected
        elide: Text.ElideMiddle
    }

    Text {
        id: kindLabel

        anchors.right: parent.right
        anchors.rightMargin: Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        visible: row.kind !== ""
        text: row.kind
        color: Theme.textMuted
        font.pixelSize: 10
    }

    MouseArea {
        id: mouse

        anchors.fill: parent
        hoverEnabled: true
        onClicked: row.chosen()
    }
}
