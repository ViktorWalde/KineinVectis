pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// WATCHES: expressoes que o usuario avalia no frame parado.
//
// Nasceu em 2026-09-03 com a etapa 15 do roadmaps/34. Um watch e' literalmente
// uma variavel avaliada sob demanda: o `reference` que o `debug.evaluate`
// devolve alimenta a MESMA arvore de expansao que o `debug.variables` usa, e
// por isso este componente e irmao do DebugInspector, nao um estranho.
//
// Componente burro: recebe o modelo por property, pede por signal. Quem
// reavalia a cada parada e' o DebugController.
Item {
    id: root

    property bool paused: false
    property var watchesModel

    signal watchAdded(string expression)
    signal watchRemoved(int index)

    function focusInput() {
        watchInput.forceActiveFocus();
    }

    Rectangle {
        id: watchesBox

        anchors.fill: parent
        visible: root.paused
        radius: Theme.radius
        color: Theme.surface1
        border.width: 1
        border.color: Theme.borderSoft

        Text {
            id: watchesTitle

            anchors.top: parent.top
            anchors.left: parent.left
            anchors.margins: Theme.spacingSmall
            text: qsTr("Watches")
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
        }

        KvTextField {
            id: watchInput

            anchors.top: watchesTitle.bottom
            anchors.topMargin: 2
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.leftMargin: Theme.spacingSmall
            anchors.rightMargin: Theme.spacingSmall
            height: 24
            pixelSize: Theme.fontSizeSmall
            placeholder: qsTr("expressão + Enter")
            onAccepted: {
                const expressao = watchInput.text.trim();
                if (expressao === "") {
                    return;
                }
                root.watchAdded(expressao);
                watchInput.text = "";
            }
        }

        ListView {
            id: watchesView

            anchors.top: watchInput.bottom
            anchors.topMargin: 2
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.leftMargin: Theme.spacingSmall
            anchors.rightMargin: Theme.spacingSmall
            anchors.bottomMargin: Theme.spacingSmall
            clip: true
            model: root.watchesModel

            delegate: Item {
                id: watchRow

                required property int index
                required property string expression
                required property string value
                required property bool failed

                width: watchesView.width
                height: 16

                Text {
                    anchors.left: parent.left
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width * 0.4
                    elide: Text.ElideRight
                    text: watchRow.expression
                    color: Theme.textMuted
                    font.pixelSize: Theme.fontSizeSmall
                }

                Text {
                    anchors.left: parent.left
                    anchors.leftMargin: parent.width * 0.42
                    anchors.right: removeArea.left
                    anchors.verticalCenter: parent.verticalCenter
                    elide: Text.ElideRight
                    text: watchRow.value
                    color: watchRow.failed ? Theme.errorSoft : Theme.textPrimary
                    font.pixelSize: Theme.fontSizeSmall
                }

                MouseArea {
                    id: removeArea

                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    width: 14
                    height: 14
                    onClicked: root.watchRemoved(watchRow.index)

                    Text {
                        anchors.centerIn: parent
                        text: "×"
                        color: Theme.textDisabled
                        font.pixelSize: Theme.fontSizeBody
                    }
                }
            }
        }
    }
}
