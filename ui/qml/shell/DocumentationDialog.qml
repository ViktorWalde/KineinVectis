pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Preview Markdown interno, equivalente ao modo de documentação renderizada
// das IDEs maduras. O conteúdo vem do MANUAL.md empacotado na própria build;
// não há rede, navegador externo nem uma segunda cópia manual do texto.
Item {
    id: root

    property real maxAvailableWidth: 1100
    property real maxAvailableHeight: 760

    signal dismissRequested()

    onVisibleChanged: {
        if (visible) {
            forceActiveFocus();
        }
    }

    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onClicked: root.dismissRequested()
    }

    Rectangle {
        id: dialog

        anchors.centerIn: parent
        width: Math.min(1040, root.maxAvailableWidth)
        height: Math.min(700, root.maxAvailableHeight)
        radius: Theme.radiusDialog
        color: Theme.background2
        border.color: Theme.borderStrong
        border.width: 1

        MouseArea {
            anchors.fill: parent
        }

        Rectangle {
            id: header

            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            height: 48
            radius: Theme.radiusDialog
            color: Theme.surface1

            Rectangle {
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.bottom: parent.bottom
                height: Theme.radiusDialog
                color: parent.color
            }

            Row {
                anchors.left: parent.left
                anchors.leftMargin: Theme.spacingLarge
                anchors.verticalCenter: parent.verticalCenter
                spacing: Theme.spacingSmall

                KvIcon {
                    anchors.verticalCenter: parent.verticalCenter
                    name: "help"
                    size: 20
                    active: true
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: qsTr("Manual do Kinein Vectis")
                    color: Theme.textPrimary
                    font.pixelSize: 14
                    font.bold: true
                }

                Rectangle {
                    anchors.verticalCenter: parent.verticalCenter
                    width: previewLabel.implicitWidth + 2 * Theme.spacingSmall
                    height: 20
                    radius: 10
                    color: Theme.surfaceSelected

                    Text {
                        id: previewLabel

                        anchors.centerIn: parent
                        text: qsTr("Visualização")
                        color: Theme.textSecondary
                        font.pixelSize: 9
                        font.bold: true
                    }
                }
            }

            Row {
                anchors.right: parent.right
                anchors.rightMargin: Theme.spacingSmall
                anchors.verticalCenter: parent.verticalCenter
                spacing: Theme.spacingSmall

                KvButton {
                    compact: true
                    text: qsTr("Topo")
                    iconName: "back"
                    onClicked: manualFlick.contentY = 0
                }

                KvIconButton {
                    iconName: "close"
                    tooltip: qsTr("Fechar manual")
                    onClicked: root.dismissRequested()
                }
            }
        }

        Flickable {
            id: manualFlick

            anchors.top: header.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.margins: Theme.spacingLarge
            anchors.rightMargin: Theme.spacingRegion
            clip: true
            contentWidth: width
            contentHeight: manualText.paintedHeight + 2 * Theme.spacingLarge
            boundsBehavior: Flickable.StopAtBounds

            TextEdit {
                id: manualText

                x: Theme.spacingLarge
                y: Theme.spacingSmall
                width: manualFlick.width - 2 * Theme.spacingLarge
                readOnly: true
                selectByMouse: true
                text: Documentation.manualMarkdown
                textFormat: TextEdit.MarkdownText
                wrapMode: TextEdit.Wrap
                color: Theme.textPrimary
                selectionColor: Theme.accentDim
                selectedTextColor: Theme.textPrimary
                font.family: Theme.uiFont
                font.pixelSize: 13
            }

            VerticalScrollBar {
                anchors.top: parent.top
                anchors.right: parent.right
                height: manualFlick.height
                contentSize: manualFlick.contentHeight
                viewportSize: manualFlick.height
                position: manualFlick.contentY
                onMoveRequested: function(position) {
                    manualFlick.contentY = position;
                }
            }
        }
    }

    Keys.onEscapePressed: root.dismissRequested()
}
