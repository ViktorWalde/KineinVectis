import QtQuick
import KineinVectis

// O pe' do seletor (0.3.8): cada modo com a sua acao principal, e o que ela
// vai fazer dito em palavras antes do clique.
//   abrir   "Criar projeto…" a esquerda; "Abrir <pasta>" e os botoes
//   escolher (o SDK do kit) "Escolher"
//   criar   "Cancelar" e "Criar projeto", habilitado so' com linguagem e nome
Item {
    id: root

    property bool pickingFolder: false
    property bool creatingProject: false
    property bool canCreate: false
    property string selectedPath: ""

    signal cancelRequested()
    signal openRequested()
    signal createRequested()
    signal switchToCreateRequested()

    width: parent.width
    height: 32

    FolderPickerButton {
        id: createLink

        anchors.left: parent.left
        anchors.verticalCenter: parent.verticalCenter
        visible: !root.creatingProject && !root.pickingFolder
        height: parent.height
        text: qsTr("+ Criar projeto…")
        onClicked: root.switchToCreateRequested()
    }

    Text {
        anchors.left: createLink.visible ? createLink.right : parent.left
        anchors.leftMargin: Theme.spacingMedium
        anchors.right: buttons.left
        anchors.rightMargin: Theme.spacingMedium
        anchors.verticalCenter: parent.verticalCenter
        visible: !root.creatingProject && root.selectedPath !== ""
        text: (root.pickingFolder ? qsTr("Escolher: ") : qsTr("Abrir: ")) + root.selectedPath
        color: Theme.textMuted
        font.family: Theme.monoFont
        font.pixelSize: Theme.fontSizeCaption
        elide: Text.ElideMiddle
        horizontalAlignment: Text.AlignRight
    }

    Row {
        id: buttons

        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        height: parent.height
        spacing: Theme.spacingSmall

        FolderPickerButton {
            text: qsTr("Cancelar")
            height: parent.height
            onClicked: root.cancelRequested()
        }

        FolderPickerButton {
            text: root.creatingProject ? qsTr("Criar projeto")
                                       : (root.pickingFolder ? qsTr("Escolher") : qsTr("Abrir"))
            height: parent.height
            primary: true
            enabled: !root.creatingProject || root.canCreate
            opacity: enabled ? 1.0 : 0.5
            onClicked: root.creatingProject ? root.createRequested() : root.openRequested()
        }
    }
}
