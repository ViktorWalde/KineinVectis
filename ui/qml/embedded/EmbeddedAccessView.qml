pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Permissao por canal (E2): o veredito medido de cada canal, o problema e
// os passos oficiais — cada passo com um botao que o ESCREVE no terminal da
// IDE. Dono proprio pela mesma regra das outras views do painel.
//
// Burro: le de `controller.access`. O botao "Permissões" que pede o
// diagnostico esta' no cabecalho das portas (EmbeddedSerialView).
Item {
    id: root

    property var controller: null
    readonly property var access: controller ? controller.access : null

    implicitHeight: coluna.implicitHeight
    visible: access !== null && (access.diagnosed || access.busy || access.errorText !== "")

    Column {
        id: coluna

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingSmall

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            text: root.access
                  ? (root.access.busy ? qsTr("Permissões — medindo…")
                     : (root.access.problems === 0
                        ? qsTr("Permissões — nada a fazer nos canais medidos")
                        : qsTr("Permissões — %1 canal(is) precisam de um passo").arg(root.access.problems)))
                  : ""
            color: Theme.textSecondary
            font.pixelSize: 11
            font.bold: true
        }

        Repeater {
            model: root.access ? root.access.channels : []

            Column {
                id: canal

                required property var modelData

                width: coluna.width
                spacing: Theme.spacingXSmall

                Text {
                    width: parent.width
                    wrapMode: Text.WordWrap
                    text: (canal.modelData.ok ? "✓ " : "✗ ") + root.access.channelTitle(canal.modelData)
                          + " — " + canal.modelData.detail
                    color: canal.modelData.ok ? Theme.textPrimary : Theme.errorSoft
                    font.pixelSize: 11
                }

                Text {
                    width: parent.width
                    wrapMode: Text.WordWrap
                    visible: canal.modelData.distroDidIt !== undefined && canal.modelData.distroDidIt !== null
                    text: canal.modelData.distroDidIt !== undefined && canal.modelData.distroDidIt !== null
                          ? canal.modelData.distroDidIt : ""
                    color: Theme.textMuted
                    font.pixelSize: 10
                }

                Text {
                    width: parent.width
                    wrapMode: Text.WordWrap
                    visible: canal.modelData.problem !== undefined && canal.modelData.problem !== null
                    text: canal.modelData.problem !== undefined && canal.modelData.problem !== null
                          ? canal.modelData.problem : ""
                    color: Theme.warningSoft
                    font.pixelSize: 10
                }

                // Um passo por linha: a explicacao, o comando em mono e o botao
                // que o escreve no terminal da IDE.
                Repeater {
                    model: root.access.stepsOf(canal.modelData)

                    Column {
                        id: passo

                        required property var modelData

                        width: canal.width
                        spacing: 2

                        Text {
                            width: parent.width
                            wrapMode: Text.WordWrap
                            text: passo.modelData.explanation
                            color: Theme.textSecondary
                            font.pixelSize: 10
                        }

                        Row {
                            width: parent.width
                            spacing: Theme.spacingSmall

                            Text {
                                width: parent.width - escrever.width - Theme.spacingSmall
                                anchors.verticalCenter: parent.verticalCenter
                                wrapMode: Text.WrapAnywhere
                                text: passo.modelData.command
                                color: Theme.textPrimary
                                font.family: Theme.monoFont
                                font.pixelSize: 10
                            }

                            KvButton {
                                id: escrever

                                anchors.verticalCenter: parent.verticalCenter
                                text: qsTr("Escrever no terminal")
                                compact: true
                                onClicked: root.access.runStep(passo.modelData.command)
                            }
                        }
                    }
                }

                Text {
                    width: parent.width
                    wrapMode: Text.WordWrap
                    visible: canal.modelData.fix !== undefined && canal.modelData.fix !== null
                    text: canal.modelData.fix !== undefined && canal.modelData.fix !== null
                          ? qsTr("fonte: %1 (conferida em %2)").arg(canal.modelData.fix.sourceUrl).arg(canal.modelData.fix.checkedOn)
                          : ""
                    color: Theme.textMuted
                    font.pixelSize: 9
                }
            }
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            visible: root.access && root.access.errorText !== ""
            text: root.access ? root.access.errorText : ""
            color: Theme.errorSoft
            font.pixelSize: 10
        }
    }
}
