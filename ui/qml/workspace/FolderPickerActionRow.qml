import QtQuick
import KineinVectis

// O pe' do seletor (0.3.8): cada modo com a sua acao principal, e o que ela
// vai fazer dito em palavras antes do clique.
//   abrir   "Abrir <pasta>" e os botoes (o "+ Criar projeto…" daqui saiu em
//           2026-10-03, decisao do autor: apagado e sem funcao clara no
//           "Abrir"; criar fica no cartao da tela de boas-vindas e no menu)
//   escolher (o SDK do kit) "Escolher"
//   criar   "Cancelar" e "Criar projeto", habilitado so' com linguagem e nome
//   local   "Voltar" e "Usar esta pasta" (o local do projeto novo, 0.3.9)
Item {
    id: root

    property bool pickingFolder: false
    property bool creatingProject: false
    property bool choosingLocation: false
    property bool canCreate: false
    property string selectedPath: ""

    signal cancelRequested()
    signal openRequested()
    signal createRequested()

    width: parent.width
    height: 32

    Text {
        anchors.left: parent.left
        anchors.right: buttons.left
        anchors.rightMargin: Theme.spacingMedium
        anchors.verticalCenter: parent.verticalCenter
        visible: (!root.creatingProject || root.choosingLocation) && root.selectedPath !== ""
        text: (root.choosingLocation ? qsTr("Local: ")
               : (root.pickingFolder ? qsTr("Escolher: ") : qsTr("Abrir: "))) + root.selectedPath
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

        KvButton {
            text: root.choosingLocation ? qsTr("Voltar") : qsTr("Cancelar")
            height: parent.height
            onClicked: root.cancelRequested()
        }

        KvButton {
            id: primaryButton

            readonly property bool creates: root.creatingProject && !root.choosingLocation

            text: root.choosingLocation ? qsTr("Usar esta pasta")
                  : root.creatingProject ? qsTr("Criar projeto")
                  : (root.pickingFolder ? qsTr("Escolher") : qsTr("Abrir"))
            height: parent.height
            primary: true
            enabled: !primaryButton.creates || root.canCreate
            onClicked: primaryButton.creates ? root.createRequested() : root.openRequested()
        }
    }
}
