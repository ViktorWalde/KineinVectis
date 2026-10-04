import QtQuick
import KineinVectis

// "Nova pasta" dentro do navegador (0.3.9): uma linha sobre a lista, com o
// nome ja' em foco. Enter cria, Esc ou o x desistem. Nao mexe no projeto que
// se esta' criando — e' so' a pasta do local.
Rectangle {
    id: root

    property var controller

    visible: root.controller.creatingFolder
    height: visible ? 38 : 0
    radius: Theme.radius
    color: Theme.background2
    border.color: Theme.borderSoft
    border.width: 1

    onVisibleChanged: if (visible) nameField.forceActiveFocus()

    KvFileIcon {
        id: icon

        anchors.left: parent.left
        anchors.leftMargin: Theme.spacingMedium
        anchors.verticalCenter: parent.verticalCenter
        directory: true
        size: 18
    }

    KvTextField {
        id: nameField

        anchors.left: icon.right
        anchors.leftMargin: Theme.spacingMedium
        anchors.right: createButton.left
        anchors.rightMargin: Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        height: 26
        codeFont: false
        text: root.controller.folderName
        placeholder: qsTr("nome da nova pasta")
        onEdited: (text) => { root.controller.folderName = text; }
        onAccepted: root.controller.submitCreateFolder()
        Keys.onEscapePressed: (event) => {
            root.controller.cancelCreateFolder();
            event.accepted = true;
        }
    }

    FolderPickerButton {
        id: createButton

        anchors.right: cancelButton.left
        anchors.rightMargin: Theme.spacingXSmall
        anchors.verticalCenter: parent.verticalCenter
        height: 26
        text: qsTr("Criar pasta")
        primary: true
        onClicked: root.controller.submitCreateFolder()
    }

    KvIconButton {
        id: cancelButton

        anchors.right: parent.right
        anchors.rightMargin: Theme.spacingXSmall
        anchors.verticalCenter: parent.verticalCenter
        compact: true
        iconName: "close"
        tooltip: qsTr("Cancelar")
        focus: false
        onClicked: root.controller.cancelCreateFolder()
    }
}
