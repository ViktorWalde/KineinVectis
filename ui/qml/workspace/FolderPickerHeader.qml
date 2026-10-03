import QtQuick
import KineinVectis

Row {
    id: root

    property string title: qsTr("Abrir projeto")
    property string subtitle: ""

    signal closeRequested()

    width: parent.width
    height: root.subtitle !== "" ? 44 : 28
    spacing: Theme.spacingMedium

    Column {
        anchors.verticalCenter: parent.verticalCenter

        Text {
            text: root.title
            color: Theme.textPrimary
            font.pixelSize: Theme.fontSizeHeadline - 4
            font.bold: true
        }

        Text {
            visible: root.subtitle !== ""
            text: root.subtitle
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeSmall
        }
    }

    Item {
        width: parent.width - x - closeButton.width
        height: 1
    }

    KvIconButton {
        id: closeButton

        compact: true
        iconName: "close"
        iconSize: 16
        tooltip: qsTr("Fechar (Esc)")
        onClicked: root.closeRequested()
    }
}
