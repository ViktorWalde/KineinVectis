import QtQuick
import KineinVectis

// Confirmação da ação DESTRUTIVA de descartar mudanças (fatia M3.3).
// Guardrail docs/arquitetura/19: o core executa sem perguntar; quem confirma é a UI.
Item {
    id: root

    property string entryPath: ""
    property real maxAvailableWidth: 480

    signal confirmRequested()
    signal cancelRequested()

    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onClicked: root.cancelRequested()
    }

    Rectangle {
        anchors.centerIn: parent
        width: Math.min(460, root.maxAvailableWidth)
        height: discardColumn.height + 2 * Theme.spacingLarge
        radius: Theme.radiusLarge
        color: Theme.background2
        border.color: Theme.borderStrong
        border.width: 1

        MouseArea {
            anchors.fill: parent
        }

        Column {
            id: discardColumn

            anchors.centerIn: parent
            width: parent.width - 2 * Theme.spacingLarge
            spacing: Theme.spacingMedium

            Text {
                width: parent.width
                text: qsTr("Descartar mudanças?")
                color: Theme.textPrimary
                font.pixelSize: 14
                font.bold: true
            }

            Text {
                width: parent.width
                text: qsTr("As mudanças de \"%1\" serão descartadas "
                           + "(arquivo novo será apagado). Isso não tem "
                           + "desfazer.").arg(root.entryPath)
                color: Theme.textSecondary
                font.pixelSize: 12
                wrapMode: Text.WordWrap
            }

            Row {
                anchors.right: parent.right
                spacing: Theme.spacingSmall

                Rectangle {
                    width: cancelLabel.width + 2 * Theme.spacingMedium
                    height: 28
                    radius: Theme.radius
                    color: cancelArea.containsMouse ? Theme.surface2 : Theme.surface1
                    border.color: Theme.borderSoft
                    border.width: 1

                    Text {
                        id: cancelLabel

                        anchors.centerIn: parent
                        text: qsTr("Cancelar")
                        color: Theme.textPrimary
                        font.pixelSize: 12
                    }

                    MouseArea {
                        id: cancelArea

                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.cancelRequested()
                    }
                }

                Rectangle {
                    width: confirmLabel.width + 2 * Theme.spacingMedium
                    height: 28
                    radius: Theme.radius
                    color: confirmArea.pressed ? Theme.surface1 : Theme.errorSoft

                    Text {
                        id: confirmLabel

                        anchors.centerIn: parent
                        text: qsTr("Descartar")
                        color: Theme.background0
                        font.pixelSize: 12
                        font.bold: true
                    }

                    MouseArea {
                        id: confirmArea

                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.confirmRequested()
                    }
                }
            }
        }
    }

    Keys.onEscapePressed: root.cancelRequested()

    onVisibleChanged: {
        if (visible) {
            forceActiveFocus();
        }
    }
}
