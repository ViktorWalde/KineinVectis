pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O MODELO do projeto embarcado na tela (pilar 0 do roadmaps/42): o que o
// projeto E' — framework com o arquivo que o provou, o alvo deduzido com os
// motores sugeridos, o que falta na maquina com o passo oficial, e o que o
// modelo nao conseguiu decidir. Dono proprio, como o EmbeddedSizeView e o
// EmbeddedSerialView: o painel ja' esta' no limite e "mostrar o modelo" e'
// outra responsabilidade. Burro: le do controller; nao deduz nada aqui.
Item {
    id: root

    property var controller: null

    implicitHeight: coluna.implicitHeight

    Column {
        id: coluna

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingSmall

        Text {
            text: qsTr("Projeto")
            color: Theme.textSecondary
            font.pixelSize: 11
            font.bold: true
        }

        Repeater {
            model: root.controller ? root.controller.projectFrameworks : []

            Text {
                id: linhaFramework

                required property var modelData

                width: coluna.width
                text: "● " + root.controller.frameworkSummary(linhaFramework.modelData)
                color: Theme.textPrimary
                font.family: Theme.monoFont
                font.pixelSize: 11
                elide: Text.ElideMiddle
            }
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            visible: root.controller && !root.controller.projectEmbedded
            text: root.controller && root.controller.projectBusy ? qsTr("lendo o projeto…")
                  : qsTr("nenhum framework de embarcado reconhecido (ESP-IDF, Zephyr, pico-sdk, PlatformIO, STM32Cube, Rust bare metal, MicroPython, Yocto, Buildroot)")
            color: Theme.textSecondary
            font.pixelSize: 11
        }

        // O alvo: chip, familia, motores — e a evidencia de cada deducao
        // logo abaixo, porque "de onde veio" e' o que separa modelo de palpite.
        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            visible: root.controller && root.controller.projectEmbedded
            text: root.controller ? qsTr("alvo: %1").arg(root.controller.targetSummary(root.controller.projectTarget)) : ""
            color: Theme.textPrimary
            font.family: Theme.monoFont
            font.pixelSize: 11
        }

        Repeater {
            model: root.controller && root.controller.projectTarget.evidence !== undefined
                   ? root.controller.projectTarget.evidence : []

            Text {
                id: linhaEvidencia

                required property var modelData

                width: coluna.width
                wrapMode: Text.WordWrap
                text: "  ↳ " + linhaEvidencia.modelData
                color: Theme.textMuted
                font.pixelSize: 10
            }
        }

        // O que falta na maquina, com o passo oficial. A IDE imprime; nunca
        // roda.
        Repeater {
            model: root.controller ? root.controller.projectSdks : []

            Text {
                id: linhaSdk

                required property var modelData

                width: coluna.width
                wrapMode: Text.WordWrap
                text: (linhaSdk.modelData.found ? "✓ " : "✗ ") + linhaSdk.modelData.label
                      + (linhaSdk.modelData.found && linhaSdk.modelData.path !== undefined
                         ? " · " + linhaSdk.modelData.path
                         : (linhaSdk.modelData.hint !== undefined ? " — " + linhaSdk.modelData.hint : ""))
                color: linhaSdk.modelData.found ? Theme.textSecondary : Theme.warningSoft
                font.pixelSize: 10
            }
        }

        Repeater {
            model: root.controller ? root.controller.projectHints : []

            Text {
                id: linhaDica

                required property var modelData

                width: coluna.width
                wrapMode: Text.WordWrap
                visible: linhaDica.modelData.indexOf("falta ") !== 0
                text: linhaDica.modelData
                color: Theme.textMuted
                font.pixelSize: 10
            }
        }
    }
}
