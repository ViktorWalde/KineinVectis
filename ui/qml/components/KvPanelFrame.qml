import QtQuick
import KineinVectis

// A MOLDURA comum dos paineis de ambiente (Etapa 2 F8): centralizada,
// dispensa por clique fora (nao por clique dentro), e o conteudo ROLA
// quando nao cabe — o Embarcados vazava da moldura a 800 px (foto 12c).
//
// Dois modos: `contentHeight` 0 (padrao) e' o painel que preenche a
// moldura e rola por dentro (banco, remoto, containers); `contentHeight`
// > 0 e' a altura que o painel pede, e a moldura rola o que passar.
Item {
    id: root

    default property alias content: conteudo.data
    property real panelWidth: 720
    property real panelHeight: 560
    property real maxAvailableWidth: 720
    property real maxAvailableHeight: 520
    property real contentHeight: 0

    signal dismissRequested()

    readonly property real frameWidth: Math.min(panelWidth, maxAvailableWidth)
    readonly property real frameHeight: Math.min(panelHeight, maxAvailableHeight)

    MouseArea {
        anchors.fill: parent
        onClicked: root.dismissRequested()
    }

    Rectangle {
        anchors.centerIn: parent
        width: root.frameWidth
        height: root.frameHeight
        radius: Theme.radius
        color: Theme.background1
        border.width: 1
        border.color: Theme.borderStrong

        MouseArea {
            anchors.fill: parent
        }

        Flickable {
            id: rolagem

            anchors.fill: parent
            anchors.margins: Theme.spacingMedium
            clip: true
            contentWidth: width
            contentHeight: conteudo.height
            interactive: root.contentHeight > height
            boundsBehavior: Flickable.StopAtBounds
            flickableDirection: Flickable.VerticalFlick

            Item {
                id: conteudo

                width: rolagem.width
                height: Math.max(rolagem.height, root.contentHeight)
            }
        }
    }
}
