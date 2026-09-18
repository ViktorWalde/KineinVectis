pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A INSPECAO do programa parado: pilha de chamadas e variaveis do frame.
//
// Saiu do DebugPanel.qml em 2026-09-03 (etapa 15 do roadmaps/34), que estava
// em 370/300 e barraria o acrescimo dos watches. O DebugPanel ficou com o que
// vale com o programa RODANDO — controles e saida —, e o que so' existe com a
// thread parada mora aqui e no irmao DebugWatches.qml.
//
// Os dois nao viraram um arquivo so' porque o resultado passava de 300: a §4
// regra 8 manda dividir por area e deixar a CONTAGEM de arquivos crescer, e a
// area comum e' a pasta debug/, nao o arquivo.
Item {
    id: root

    property bool paused: false
    property var framesModel
    property var variablesModel
    property int currentFrameIndex: -1

    signal frameActivated(int index)
    signal variableToggled(int index)

    Rectangle {
        id: framesBox

        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        width: 230

        radius: Theme.radius
        color: Theme.background1
        border.color: Theme.borderSoft
        border.width: 1

        Text {
            id: framesTitle

            anchors.top: parent.top
            anchors.left: parent.left
            anchors.margins: Theme.spacingSmall
            text: qsTr("Frames")
            color: Theme.textMuted
            font.pixelSize: 10
            font.bold: true
        }

        ListView {
            anchors.top: framesTitle.bottom
            anchors.topMargin: Theme.spacingXSmall
                anchors.left: parent.left
            anchors.right: parent.right
            anchors.margins: Theme.spacingSmall
            clip: true
            model: root.framesModel

            delegate: Rectangle {
                id: frameRow

                required property int index
                required property string name
                required property string file
                required property int line

                width: ListView.view.width
                height: 22
                radius: Theme.radiusXSmall
                color: root.currentFrameIndex === frameRow.index
                       ? Theme.surfaceSelected
                       : (frameRowArea.containsMouse ? Theme.surface2
                                                     : "transparent")

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.right: parent.right
                                anchors.rightMargin: Theme.spacingSmall
                    text: frameRow.line > 0
                          ? qsTr("%1  ·  :%2").arg(frameRow.name).arg(frameRow.line)
                          : frameRow.name
                    color: root.currentFrameIndex === frameRow.index
                           ? Theme.textPrimary : Theme.textSecondary
                    font.pixelSize: 11
                    elide: Text.ElideRight
                }

                MouseArea {
                    id: frameRowArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.frameActivated(frameRow.index)
                }
            }
        }
    }

    Rectangle {
        id: variablesBox

        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.left: framesBox.right
        anchors.leftMargin: Theme.spacingSmall
        anchors.right: parent.right

        radius: Theme.radius
        color: Theme.background1
        border.color: Theme.borderSoft
        border.width: 1

        Text {
            id: variablesTitle

            anchors.top: parent.top
            anchors.left: parent.left
            anchors.margins: Theme.spacingSmall
            text: qsTr("Variáveis")
            color: Theme.textMuted
            font.pixelSize: 10
            font.bold: true
        }

        ListView {
            anchors.top: variablesTitle.bottom
            anchors.topMargin: Theme.spacingXSmall
                anchors.left: parent.left
                anchors.margins: Theme.spacingSmall
            clip: true
            model: root.variablesModel

            delegate: Rectangle {
                id: variableRowDelegate

                required property int index
                required property string name
                required property string value
                required property string typeName
                required property real reference
                required property int depth
                required property bool expanded

                width: ListView.view.width
                height: 20
                radius: Theme.radiusXSmall
                color: variableRowArea.containsMouse
                       && variableRowDelegate.reference > 0
                       ? Theme.surface2 : "transparent"

                Text {
                    id: variableArrow

                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacingSmall
                                        + variableRowDelegate.depth * 14
                    width: 12
                    text: variableRowDelegate.reference > 0
                          ? (variableRowDelegate.expanded ? "▾" : "▸") : ""
                    color: Theme.textMuted
                    font.pixelSize: 10
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: variableArrow.right
                    anchors.right: parent.right
                    anchors.rightMargin: Theme.spacingSmall
                    text: variableRowDelegate.typeName !== ""
                          ? qsTr("%1 = %2  (%3)")
                                .arg(variableRowDelegate.name)
                                .arg(variableRowDelegate.value)
                                .arg(variableRowDelegate.typeName)
                          : qsTr("%1 = %2")
                                .arg(variableRowDelegate.name)
                                .arg(variableRowDelegate.value)
                    color: Theme.textSecondary
                    font.family: Theme.monoFont
                    font.pixelSize: 11
                    elide: Text.ElideRight
                }

                MouseArea {
                    id: variableRowArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: variableRowDelegate.reference > 0
                                 ? Qt.PointingHandCursor : Qt.ArrowCursor
                    onClicked: root.variableToggled(variableRowDelegate.index)
                }
            }
        }
    }
}
