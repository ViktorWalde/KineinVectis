pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Uma ferramenta na lista: se ja' esta instalada, o que ela faz, e o passo a
// passo quando aberto.
Item {
    id: root

    property var tool: null
    property bool expanded: false

    signal toggleRequested()
    signal commandRequested(string command)

    readonly property var guide: root.tool && root.tool.guide ? root.tool.guide : null

    implicitHeight: coluna.implicitHeight

    Column {
        id: coluna

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: 2

        Row {
            width: parent.width
            spacing: Theme.spacingSmall

            Rectangle {
                anchors.verticalCenter: parent.verticalCenter
                width: 8
                height: 8
                radius: 4
                color: root.tool && root.tool.installed
                       ? Theme.successSoft : Theme.textDisabled
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: root.tool ? root.tool.name : ""
                color: Theme.textPrimary
                font.pixelSize: 11
                font.weight: Font.DemiBold
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: root.tool && root.tool.installed
                      ? qsTr("já instalado") : qsTr("não encontrado")
                color: root.tool && root.tool.installed
                       ? Theme.successSoft : Theme.textMuted
                font.pixelSize: 10
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("site oficial ↗")
                color: siteArea.containsMouse ? Theme.accent : Theme.textSecondary
                font.pixelSize: 10
                font.underline: siteArea.containsMouse

                MouseArea {
                    id: siteArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: Qt.openUrlExternally(root.tool.website)
                }
            }
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            text: root.tool ? root.tool.summary : ""
            color: Theme.textMuted
            font.pixelSize: 10
        }

        KvButton {
            compact: true
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
            font.pixelSize: 10
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
                    font.pixelSize: 10
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
                        text: passo.modelData.command
                        color: Theme.textPrimary
                        font.family: Theme.monoFont
                        font.pixelSize: 10
                    }
                }

                Row {
                    spacing: Theme.spacingXSmall

                    KvButton {
                        compact: true
                        text: qsTr("Copiar")
                        onClicked: Clipboard.setText(passo.modelData.command)
                    }

                    KvButton {
                        compact: true
                        text: qsTr("Enviar ao terminal")
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
            font.pixelSize: 9

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
