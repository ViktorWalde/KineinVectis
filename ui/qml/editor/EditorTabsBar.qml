pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// As abas do editor (Etapa 2, F3 do roadmaps/43, 2026-09-18): a aba ATIVA e'
// obvia — fundo do editor, borda de acento em cima, texto primario — e as
// inativas ficam no fundo do painel (o "islands" da referencia: a aba ativa
// e o editor sao a mesma superficie). O arquivo modificado mostra "●" no
// lugar do ✕ ate' o mouse chegar; o botao "Salvar" amarelo permanente saiu:
// salvar e' Ctrl+S ou o autosave (decisao do autor, 2026-09-18).
Item {
    id: root

    property var filesModel
    property int fileCount: 0
    property int currentIndex: -1

    signal tabSelected(int index)
    signal tabCloseRequested(int index)

    height: fileCount > 0 ? 36 : 0
    visible: fileCount > 0

    Row {
        anchors.left: parent.left
        anchors.bottom: parent.bottom
        spacing: 2

        Repeater {
            model: root.filesModel

            delegate: Rectangle {
                id: tabDelegate

                required property int index
                required property string name
                required property bool modified

                readonly property bool active: index === root.currentIndex

                width: tabLabel.width + closeButton.width + 3 * Theme.spacingSmall
                height: 32
                radius: Theme.radius
                color: active ? Theme.background0 : (tabArea.containsMouse ? Theme.surface2 : Theme.surface1)

                // A borda de acento em cima: a marca da aba ativa.
                Rectangle {
                    anchors.top: parent.top
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.leftMargin: Theme.radius
                    anchors.rightMargin: Theme.radius
                    height: 2
                    radius: 1
                    visible: tabDelegate.active
                    color: Theme.accent
                }

                Text {
                    id: tabLabel

                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacingSmall
                    text: tabDelegate.name
                    color: tabDelegate.active ? Theme.textPrimary : Theme.textSecondary
                    font.pixelSize: 12
                    font.weight: tabDelegate.active ? Font.DemiBold : Font.Normal
                }

                // Modificado: "●" onde o ✕ fica; o ✕ volta ao pairar.
                Text {
                    anchors.centerIn: closeButton
                    visible: tabDelegate.modified && !closeArea.containsMouse
                    text: "●"
                    color: Theme.accent
                    font.pixelSize: 11
                }

                KvIconButton {
                    id: closeButton

                    anchors.verticalCenter: parent.verticalCenter
                    anchors.right: parent.right
                    anchors.rightMargin: 2
                    width: 24
                    height: 24
                    opacity: tabDelegate.modified && !closeArea.containsMouse ? 0 : 1
                    iconName: "close"
                    iconSize: 13
                    danger: true
                    tooltip: tabDelegate.modified ? qsTr("Fechar aba (não salvo)") : qsTr("Fechar aba")
                    onClicked: root.tabCloseRequested(tabDelegate.index)

                    MouseArea {
                        id: closeArea

                        anchors.fill: parent
                        hoverEnabled: true
                        acceptedButtons: Qt.NoButton
                    }
                }

                MouseArea {
                    id: tabArea

                    anchors.fill: parent
                    anchors.rightMargin: closeButton.width + Theme.spacingSmall
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.tabSelected(tabDelegate.index)
                }
            }
        }
    }
}
