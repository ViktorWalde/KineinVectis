pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A lista de CONCEITOS: a metade da tela onde o autor escolhe o que simular.
//
// Separada do `SimPanel` porque escolher e montar sao trabalhos diferentes — e
// porque o painel bateu no limite da catraca em 2026-09-05 e o corte por
// RESPONSABILIDADE era este. Esta lista nao sabe o que e' uma formula.
Item {
    id: root

    property var concepts: []
    property string selectedConcept: ""

    signal conceptSelected(string id)

    Rectangle {
        anchors.fill: parent
        radius: Theme.radiusXSmall
        color: Theme.background2
        border.width: 1
        border.color: Theme.borderSoft

        Column {
            anchors.fill: parent
            anchors.margins: Theme.spacingSmall
            spacing: Theme.spacingXSmall

            Text {
                text: qsTr("Conceito")
                color: Theme.textPrimary
                font.family: Theme.uiFont
                font.pixelSize: Theme.fontSizePanelTitle
                font.bold: true
            }

            ListView {
                id: lista

                width: parent.width
                height: parent.height - y
                clip: true
                model: root.concepts
                spacing: 1

                delegate: Rectangle {
                    id: item

                    required property var modelData

                    width: lista.width
                    height: nome.implicitHeight + curso.implicitHeight + 2 * Theme.spacingXSmall
                    radius: Theme.radiusXSmall
                    color: item.modelData.id === root.selectedConcept
                           ? Theme.surfaceSelected : "transparent"

                    Column {
                        anchors.fill: parent
                        anchors.margins: Theme.spacingXSmall
                        spacing: 0

                        Text {
                            id: nome

                            width: parent.width
                            elide: Text.ElideRight
                            text: item.modelData.name
                            color: Theme.textPrimary
                            font.family: Theme.uiFont
                            font.pixelSize: Theme.fontSizeStatus
                        }

                        Text {
                            id: curso

                            width: parent.width
                            elide: Text.ElideRight
                            // O curso ao lado do nome e' o que faz o catalogo
                            // ensinar: "queda livre" e "Fisica basica" juntos
                            // dizem onde o conceito mora na grade.
                            text: item.modelData.course
                            color: Theme.textMuted
                            font.family: Theme.uiFont
                            font.pixelSize: Theme.fontSizeStatus - 1
                        }
                    }

                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.conceptSelected(item.modelData.id)
                    }
                }
            }
        }
    }
}
