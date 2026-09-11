pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O painel de embarcados dentro de um dialogo, como os outros overlays.
//
// Separa o CONTEUDO (EmbeddedPanel, burro) do CHROME (fundo, moldura,
// dispensar por clique fora). Mesmo desenho do `GrafanaPanelHost`.
//
// DIMENSIONAMENTO AUTOMATICO COM PISO (decisao do autor, 2026-09-04): a altura
// segue o que ha' para mostrar — uma sonda a mais e' uma linha a mais, e a
// saida crua so' aparece quando nada foi reconhecido — nunca abaixo do piso,
// para nao piscar a cada busca, e nunca acima do que a janela oferece.
Item {
    id: root

    property var controller: null
    property var toolchainController: null
    property real maxAvailableWidth: 720
    property real maxAvailableHeight: 560

    // Piso: cabem as tres secoes com um estado vazio de sonda.
    readonly property int alturaMinima: 420
    readonly property int alturaMaxima: 640

    readonly property int quantasSondas: root.controller ? root.controller.probes.length : 0

    signal dismissRequested()

    MouseArea {
        anchors.fill: parent
        onClicked: root.dismissRequested()
    }

    Rectangle {
        anchors.centerIn: parent
        width: Math.min(560, root.maxAvailableWidth)
        height: Math.min(
            root.maxAvailableHeight,
            Math.min(root.alturaMaxima,
                     Math.max(root.alturaMinima, 400 + root.quantasSondas * 20)))
        radius: Theme.radius
        color: Theme.background1
        border.width: 1
        border.color: Theme.borderStrong

        MouseArea {
            anchors.fill: parent
        }

        EmbeddedPanel {
            anchors.fill: parent
            anchors.margins: Theme.spacingMedium
            controller: root.controller
            toolchainController: root.toolchainController
            onCloseRequested: root.dismissRequested()
        }
    }
}
