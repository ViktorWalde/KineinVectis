pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O painel de observabilidade dentro de um dialogo, como os outros overlays.
//
// Separa o CONTEUDO (GrafanaPanel, burro) do CHROME (fundo, moldura, dispensar
// por clique fora). Mesmo desenho do `DataSourcePanelHost`.
//
// DIMENSIONAMENTO AUTOMATICO COM PISO (decisao do autor, 2026-09-04): a altura
// segue o que ha' para mostrar — um workspace sem Grafana precisa de um
// formulario, um com quarenta dashboards precisa de espaco — mas nunca abaixo
// do piso, para o dialogo nao piscar de tamanho a cada sonda, e nunca acima do
// que a janela oferece.
Item {
    id: root

    property var controller: null
    property real maxAvailableWidth: 720
    property real maxAvailableHeight: 560

    // Piso: cabe o formulario inteiro sem rolagem.
    readonly property int alturaMinima: 320
    // Teto proprio: alem disto a lista vira despejo e o dialogo vira a tela.
    readonly property int alturaMaxima: 620

    readonly property int quantosAchados: root.controller
        ? root.controller.matches.length + root.controller.dataSources.length
          + root.controller.dashboards.length
        : 0

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
                     Math.max(root.alturaMinima, 300 + root.quantosAchados * 30)))
        radius: Theme.radius
        color: Theme.background1
        border.width: 1
        border.color: Theme.borderStrong

        MouseArea {
            anchors.fill: parent
        }

        GrafanaPanel {
            anchors.fill: parent
            anchors.margins: Theme.spacingMedium
            controller: root.controller
        }
    }
}
