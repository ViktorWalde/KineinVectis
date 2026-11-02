pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Uma ferramenta na lista: se ja' esta instalada, o que ela faz, e o passo a
// passo quando aberto.
Rectangle {
    id: root

    property var tool: null
    property bool expanded: false
    property bool selected: false
    property bool terminalAvailable: false

    color: root.selected ? Theme.surface2 : "transparent"
    radius: Theme.radius
    border.width: root.selected ? 1 : 0
    border.color: Theme.accent

    signal toggleRequested()
    signal commandRequested(string command)

    readonly property var guide: root.tool && root.tool.guide ? root.tool.guide : null

    implicitHeight: coluna.implicitHeight

    Column {
        id: coluna

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: 2

        Text {
            width: parent.width
            text: root.tool ? root.tool.name : ""
            textFormat: Text.PlainText
            wrapMode: Text.WordWrap
            color: Theme.textPrimary
            font.pixelSize: Theme.fontSizeSmall
            font.weight: Font.DemiBold
        }

        Flow {
            width: parent.width
            spacing: Theme.spacingSmall

            Text {
                text: root.tool && root.tool.installed
                      ? qsTr("Instalada") : qsTr("Não encontrada")
                color: root.tool && root.tool.installed ? Theme.successSoft : Theme.textMuted
                font.pixelSize: Theme.fontSizeCaption
            }

            KvButton {
                compact: true
                activeFocusOnTab: true
                text: qsTr("Site oficial ↗")
                onClicked: if (root.tool) Qt.openUrlExternally(root.tool.website)
            }
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            textFormat: Text.PlainText
            text: root.tool ? root.tool.summary : ""
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
        }

        KvButton {
            compact: true
            activeFocusOnTab: true
            visible: root.guide !== null
            text: root.expanded ? qsTr("Ocultar passo a passo")
                                : qsTr("Ver passo a passo")
            onClicked: root.toggleRequested()
        }

        Text {
            width: parent.width
            visible: root.guide === null
            wrapMode: Text.WordWrap
            text: qsTr("A documentação oficial não cobre esta distribuição. "
                       + "Abra o site acima — a IDE não inventa comando.")
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
        }

        Repeater {
            model: root.expanded && root.guide ? root.guide.steps : []

            delegate: Column {
                id: passo

                required property int index
                required property var modelData

                width: coluna.width
                spacing: 1

                Text {
                    width: parent.width
                    wrapMode: Text.WordWrap
                    text: (passo.index + 1) + ". " + passo.modelData.explanation
                    color: Theme.textSecondary
                    font.pixelSize: Theme.fontSizeCaption
                }

                Rectangle {
                    width: parent.width
                    height: comando.implicitHeight + 2 * Theme.spacingXSmall
                    radius: Theme.radiusXSmall
                    color: Theme.backgroundEditor
                    border.width: 1
                    border.color: Theme.borderSoft

                    Text {
                        id: comando

                        x: Theme.spacingXSmall
                        y: Theme.spacingXSmall
                        width: parent.width - 2 * Theme.spacingXSmall
                        wrapMode: Text.WrapAnywhere
                        textFormat: Text.PlainText
                        text: passo.modelData.command
                        color: Theme.textPrimary
                        font.family: Theme.monoFont
                        font.pixelSize: Theme.fontSizeCaption
                    }
                }

                Flow {
                    width: parent.width
                    spacing: Theme.spacingXSmall

                    KvButton {
                        compact: true
                        activeFocusOnTab: true
                        text: qsTr("Copiar")
                        onClicked: Clipboard.setText(passo.modelData.command)
                    }

                    KvButton {
                        compact: true
                        activeFocusOnTab: true
                        text: qsTr("Enviar ao terminal")
                        enabled: root.terminalAvailable
                        tooltip: root.terminalAvailable ? qsTr("Executa o comando exibido em uma nova sessão de terminal.")
                                                        : qsTr("Abra um projeto para usar o terminal da IDE.")
                        onClicked: root.commandRequested(passo.modelData.command)
                    }
                }
            }
        }

        // A FONTE, sempre visivel com o passo a passo. Desconfiar de um comando
        // com `sudo` que apareceu numa tela e' a atitude certa.
        Text {
            width: parent.width
            visible: root.expanded && root.guide !== null
            wrapMode: Text.WordWrap
            text: root.guide
                  ? qsTr("Fonte: %1 — conferida em %2")
                    .arg(root.guide.sourceUrl).arg(root.guide.checkedAt)
                  : ""
            color: fonteArea.containsMouse ? Theme.accent : Theme.textMuted
            font.pixelSize: Theme.fontSizeMicro

            MouseArea {
                id: fonteArea

                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: Qt.openUrlExternally(root.guide.sourceUrl)
            }
        }
    }
}
