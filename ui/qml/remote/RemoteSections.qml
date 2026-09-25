pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A faixa de SECOES do painel Remoto (fatia R1/V2, 2026-09-24).
//
// A §5.1 da especificacao pede Overview, Workspace, Run & Debug, Sistema e
// Configurar, e diz explicitamente: "nao devem ser cinco cards longos na mesma
// rolagem". A §3 diagnostica o painel antigo — "configuracao rara e operacao
// frequente disputam a mesma coluna" — e a foto de 2026-09-24 mostrou isso
// piorando quando a R0.5 acrescentou duas entradas no topo.
//
// Uma seccao por vez, entao. Componente burro: recebe o modelo e o atual, pede
// a troca por sinal.
Item {
    id: root

    // [{ id, label }]
    property var sections: []
    property string current: ""

    signal selected(string id)

    implicitHeight: linha.implicitHeight

    Row {
        id: linha

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: 2

        Repeater {
            model: root.sections

            delegate: Rectangle {
                id: aba

                required property var modelData

                readonly property bool atual: aba.modelData.id === root.current

                width: Math.max(rotulo.implicitWidth + 2 * Theme.spacingSmall, 62)
                height: 24
                radius: Theme.radius
                color: aba.atual
                       ? Theme.surfaceSelected
                       : (area.containsMouse ? Theme.surface2 : "transparent")

                Text {
                    id: rotulo

                    anchors.centerIn: parent
                    text: aba.modelData.label
                    // A seccao atual nao pode depender so' do fundo: em tema
                    // claro a diferenca some.
                    color: aba.atual ? Theme.textPrimary : Theme.textMuted
                    font.pixelSize: 10
                }

                MouseArea {
                    id: area

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.selected(aba.modelData.id)
                }
            }
        }
    }
}
