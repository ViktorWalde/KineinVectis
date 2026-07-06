import QtQuick
import KineinVectis

Column {
    id: root

    property var controller

    width: parent.width
    height: pathBox.height + navigationRow.height + Theme.spacingSmall
    spacing: Theme.spacingSmall

    function focusPath() {
        pathField.forceActiveFocus();
    }

    Rectangle {
        id: pathBox

        width: parent.width
        height: 34
        radius: Theme.radius
        color: Theme.background0
        border.color: pathField.activeFocus ? Theme.accent : Theme.borderSoft
        border.width: 1

        TextInput {
            id: pathField

            anchors.fill: parent
            anchors.margins: Theme.spacingSmall
            verticalAlignment: TextInput.AlignVCenter
            text: root.controller.pathDraft
            color: Theme.textPrimary
            selectedTextColor: Theme.textPrimary
            selectionColor: Theme.accentDim
            font.family: Theme.monoFont
            font.pixelSize: 12
            clip: true
            selectByMouse: true
            onTextEdited: root.controller.pathDraft = text
            onAccepted: root.controller.browsePath(text)
        }
    }

    Row {
        id: navigationRow

        width: parent.width
        height: 30
        spacing: Theme.spacingSmall

        FolderPickerButton {
            text: qsTr("Inicio")
            height: parent.height
            onClicked: root.controller.browsePath(root.controller.homePath)
        }

        FolderPickerButton {
            text: qsTr("Subir")
            height: parent.height
            enabled: root.controller.parentPath !== ""
            onClicked: root.controller.browsePath(root.controller.parentPath)
        }

        FolderPickerButton {
            text: qsTr("Ir")
            height: parent.height
            primary: true
            onClicked: root.controller.browsePath(pathField.text)
        }

        FolderPickerButton {
            text: qsTr("+ pasta")
            height: parent.height
            onClicked: root.controller.beginCreateFolder()
        }

        FolderPickerButton {
            text: qsTr("+ projeto")
            height: parent.height
            onClicked: root.controller.beginCreateProject()
        }
    }
}
