import QtQuick
import KineinVectis

Row {
    id: root

    signal cancelRequested()
    signal openRequested()

    width: parent.width
    height: 32
    spacing: Theme.spacingSmall

    Item {
        width: parent.width - cancelButton.width - openButton.width
               - Theme.spacingSmall
        height: 1
    }

    FolderPickerButton {
        id: cancelButton

        text: qsTr("Cancelar")
        height: parent.height
        onClicked: root.cancelRequested()
    }

    FolderPickerButton {
        id: openButton

        text: qsTr("Abrir")
        height: parent.height
        primary: true
        onClicked: root.openRequested()
    }
}
