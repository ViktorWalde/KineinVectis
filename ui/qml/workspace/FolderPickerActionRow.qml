import QtQuick
import KineinVectis

Row {
    id: root

    // "Escolher" quando a pasta vai para outro fim que abrir um projeto.
    property bool pickingFolder: false

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

        text: root.pickingFolder ? qsTr("Escolher") : qsTr("Abrir")
        height: parent.height
        primary: true
        onClicked: root.openRequested()
    }
}
