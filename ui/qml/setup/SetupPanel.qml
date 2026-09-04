pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// "Como instalar" na tela: a distro detectada, o passo a passo oficial e a
// fonte de onde ele veio.
Item {
    id: root

    property string distroName: ""
    property var tools: []
    property string expandedId: ""
    property string errorText: ""

    signal toggleRequested(string id)
    signal commandRequested(string command)
    signal closeRequested()

    Text {
        id: titulo

        anchors.top: parent.top
        anchors.left: parent.left
        text: qsTr("Instalar ferramentas")
        color: Theme.textPrimary
        font.pixelSize: 12
        font.weight: Font.DemiBold
    }

    Text {
        id: subtitulo

        anchors.top: titulo.bottom
        anchors.topMargin: 2
        anchors.left: parent.left
        anchors.right: parent.right
        wrapMode: Text.WordWrap
        text: root.distroName !== ""
              ? qsTr("Detectado: %1. Os comandos são os da documentação oficial "
                     + "de cada projeto — a IDE não os inventa nem os executa "
                     + "sozinha.").arg(root.distroName)
              : qsTr("Distribuição não reconhecida — abaixo ficam os sites oficiais.")
        color: Theme.textMuted
        font.pixelSize: 10
    }

    Flickable {
        id: rolagem

        anchors.top: subtitulo.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: rodape.top
        anchors.bottomMargin: Theme.spacingSmall
        clip: true
        contentWidth: width
        contentHeight: lista.implicitHeight
        boundsBehavior: Flickable.StopAtBounds

        Column {
            id: lista

            width: rolagem.width
            spacing: Theme.spacingMedium

            Repeater {
                model: root.tools

                delegate: SetupToolRow {
                    id: linha

                    required property var modelData

                    width: lista.width
                    tool: linha.modelData
                    expanded: linha.modelData.id === root.expandedId

                    onToggleRequested: root.toggleRequested(linha.modelData.id)
                    onCommandRequested: comando => root.commandRequested(comando)
                }
            }

            Text {
                width: parent.width
                visible: root.errorText !== ""
                wrapMode: Text.WordWrap
                text: root.errorText
                color: Theme.errorSoft
                font.pixelSize: 10
            }
        }
    }

    Row {
        id: rodape

        anchors.bottom: parent.bottom
        anchors.right: parent.right

        KvButton {
            text: qsTr("Fechar")
            compact: true
            onClicked: root.closeRequested()
        }
    }
}
