pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O painel de containers dentro de um dialogo, como os outros overlays de
// "Ambiente do projeto". Separa o CONTEUDO (ContainerPanel, burro) do CHROME
// (fundo, moldura, dispensar por clique fora). Mesmo desenho do
// EmbeddedPanelHost, com o mesmo dimensionamento automatico com piso.
Item {
    id: root

    property var controller: null
    property real maxAvailableWidth: 760
    property real maxAvailableHeight: 600

    readonly property int alturaMinima: 400
    readonly property int alturaMaxima: 680
    readonly property int quantos: root.controller
        ? root.controller.containers.length + root.controller.images.length : 0

    signal dismissRequested()

    MouseArea {
        anchors.fill: parent
        onClicked: root.dismissRequested()
    }

    Rectangle {
        anchors.centerIn: parent
        width: Math.min(680, root.maxAvailableWidth)
        height: Math.min(
            root.maxAvailableHeight,
            Math.min(root.alturaMaxima,
                     Math.max(root.alturaMinima, 300 + root.quantos * 24)))
        radius: Theme.radius
        color: Theme.background1
        border.width: 1
        border.color: Theme.borderStrong

        MouseArea {
            anchors.fill: parent
        }

        ContainerPanel {
            anchors.fill: parent
            anchors.margins: Theme.spacingMedium
            controller: root.controller
            onCloseRequested: root.dismissRequested()
        }
    }
}
