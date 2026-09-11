pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O tamanho do binario embarcado: botao "Medir" e uma barra por regiao do
// linker script. Saiu do EmbeddedPanel em 2026-09-11 quando a catraca o
// reprovou (366/300): medir/exibir tamanho e' outra responsabilidade que a do
// painel (sonda, kit, depurador). Burro, como o resto: le do controller.
Item {
    id: root

    property var controller: null

    implicitHeight: coluna.implicitHeight

    Column {
        id: coluna

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingSmall

        Row {
            width: parent.width
            spacing: Theme.spacingSmall

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("Tamanho do binário")
                color: Theme.textSecondary
                font.pixelSize: 11
                font.bold: true
            }

            KvButton {
                text: root.controller && root.controller.sizeBusy
                      ? qsTr("medindo…") : qsTr("Medir tamanho")
                compact: true
                enabled: root.controller !== null && !root.controller.sizeBusy
                onClicked: root.controller.measureSize()
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                visible: root.controller && root.controller.sizeMeasured
                         && root.controller.sizeTool !== ""
                text: root.controller ? root.controller.sizeTool : ""
                color: Theme.textMuted
                font.family: Theme.monoFont
                font.pixelSize: 10
            }
        }

        // Uma barra por regiao do linker script: usado / capacidade.
        Repeater {
            model: root.controller ? root.controller.sizeRegions : []

            Column {
                id: linhaRegiao

                required property var modelData

                readonly property real fracao: root.controller.fracaoUsada(linhaRegiao.modelData)
                // O limiar que acende o aviso: 90% e' o ponto em que o proximo
                // commit pode nao caber, e vale ver antes de gravar.
                readonly property bool apertado: fracao >= 0.9

                width: parent.width
                spacing: 2

                Text {
                    width: parent.width
                    text: linhaRegiao.modelData.name + ": "
                          + linhaRegiao.modelData.used + " / " + linhaRegiao.modelData.size
                          + " B  (" + Math.round(linhaRegiao.fracao * 100) + "%)"
                          + (linhaRegiao.apertado ? qsTr("  — quase cheio") : "")
                    color: linhaRegiao.apertado ? Theme.warningSoft : Theme.textPrimary
                    font.family: Theme.monoFont
                    font.pixelSize: 11
                }

                Rectangle {
                    width: parent.width
                    height: 6
                    radius: 3
                    color: Theme.background0
                    border.width: 1
                    border.color: Theme.borderSoft

                    Rectangle {
                        width: Math.max(0, Math.min(1, linhaRegiao.fracao)) * (parent.width - 2)
                        height: parent.height - 2
                        x: 1
                        y: 1
                        radius: 2
                        color: linhaRegiao.apertado ? Theme.warningSoft : Theme.successSoft
                    }
                }
            }
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            // Medido, sem regiao: ou nao havia linker script com MEMORY, ou o
            // core nao o entendeu. Mostra os totais de secao em vez da barra.
            visible: root.controller && root.controller.sizeMeasured
                     && root.controller.sizeRegions.length === 0
            text: root.controller && !root.controller.sizeToolAvailable
                  ? qsTr("a ferramenta de tamanho (size) não foi encontrada para este kit")
                  : qsTr("sem linker script com bloco MEMORY: medido por seção, sem a fração de uso")
            color: Theme.textMuted
            font.pixelSize: 10
        }

        Repeater {
            model: (root.controller && root.controller.sizeMeasured
                    && root.controller.sizeRegions.length === 0)
                   ? root.controller.sizeSections : []

            Text {
                id: linhaSecao

                required property var modelData

                width: parent.width
                text: "  " + linhaSecao.modelData.name + "  " + linhaSecao.modelData.size + " B"
                color: Theme.textSecondary
                font.family: Theme.monoFont
                font.pixelSize: 10
            }
        }
    }
}
