import QtQuick
import KineinVectis

// A PRIMEIRA LINHA comum dos paineis de ambiente (Etapa 2 F8): titulo,
// subtitulo de UMA linha, a acao primaria a direita e o x. Os quatro
// paineis (banco, remoto, embarcados, containers) comecam igual.
Item {
    id: root

    property string title: ""
    property string subtitle: ""
    property string primaryLabel: ""
    property string primaryIcon: ""
    property bool primaryEnabled: true
    property bool primaryBusy: false

    signal primaryRequested()
    signal closeRequested()

    implicitHeight: 40

    Column {
        anchors.left: parent.left
        anchors.right: acoes.left
        anchors.rightMargin: Theme.spacingMedium
        anchors.verticalCenter: parent.verticalCenter
        spacing: 2

        Text {
            width: parent.width
            text: root.title
            color: Theme.textPrimary
            font.pixelSize: Theme.fontSizePanelTitle
            font.bold: true
            elide: Text.ElideRight
        }

        Text {
            width: parent.width
            visible: root.subtitle !== ""
            text: root.subtitle
            color: Theme.textMuted
            font.pixelSize: 10
            elide: Text.ElideRight
        }
    }

    Row {
        id: acoes

        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        spacing: Theme.spacingSmall

        KvButton {
            visible: root.primaryLabel !== ""
            primary: true
            compact: true
            enabled: root.primaryEnabled && !root.primaryBusy
            text: root.primaryLabel
            iconName: root.primaryIcon
            onClicked: root.primaryRequested()
        }

        KvIconButton {
            compact: true
            iconName: "close"
            tooltip: qsTr("Fechar")
            onClicked: root.closeRequested()
        }
    }
}
