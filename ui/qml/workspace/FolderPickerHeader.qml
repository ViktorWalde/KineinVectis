import QtQuick
import KineinVectis

Row {
    id: root

    signal closeRequested()

    width: parent.width
    height: 28
    spacing: Theme.spacingMedium

    Text {
        anchors.verticalCenter: parent.verticalCenter
        text: qsTr("Abrir ou criar projeto")
        color: Theme.textPrimary
        font.pixelSize: 16
        font.bold: true
    }

    Item {
        width: parent.width - x - closeButton.width
        height: 1
    }

    FolderPickerButton {
        id: closeButton

        width: 26
        height: 26
        text: "x"
        onClicked: root.closeRequested()
    }
}
