import QtQuick
import KineinVectis

Rectangle {
    id: root

    property bool editing: false
    property real maxAvailableWidth: 420

    signal confirmRequested()
    signal cancelRequested()

    width: Math.min(420, maxAvailableWidth)
    height: configColumn.height + 2 * Theme.spacingMedium
    radius: Theme.radius
    color: Theme.background2
    border.color: Theme.accent
    border.width: 1

    function openWith(name, command) {
        nameInput.text = name;
        commandInput.text = command;
        nameInput.forceActiveFocus();
        nameInput.selectAll();
    }

    function currentName() {
        return nameInput.text.trim();
    }

    function currentCommand() {
        return commandInput.text.trim();
    }

    Column {
        id: configColumn

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingMedium
        spacing: Theme.spacingSmall

        Text {
            text: root.editing ? qsTr("Editar configuração de execução")
                               : qsTr("Nova configuração de execução")
            color: Theme.textPrimary
            font.pixelSize: Theme.fontSizePanelTitle
            font.bold: true
        }

        Text {
            text: qsTr("Nome")
            color: Theme.textSecondary
            font.pixelSize: 11
        }

        Rectangle {
            width: parent.width
            height: 30
            radius: Theme.radius
            color: Theme.background0
            border.color: nameInput.activeFocus ? Theme.accent : Theme.borderSoft
            border.width: 1

            TextInput {
                id: nameInput

                anchors.fill: parent
                anchors.leftMargin: Theme.spacingSmall
                anchors.rightMargin: Theme.spacingSmall
                verticalAlignment: TextInput.AlignVCenter
                color: Theme.textPrimary
                selectionColor: Theme.accentDim
                selectedTextColor: Theme.textPrimary
                font.pixelSize: 12
                clip: true
                selectByMouse: true
                onAccepted: commandInput.forceActiveFocus()
                Keys.onEscapePressed: root.cancelRequested()
            }
        }

        Text {
            text: qsTr("Comando (roda na raiz do projeto)")
            color: Theme.textSecondary
            font.pixelSize: 11
        }

        Rectangle {
            width: parent.width
            height: 30
            radius: Theme.radius
            color: Theme.background0
            border.color: commandInput.activeFocus ? Theme.accent : Theme.borderSoft
            border.width: 1

            TextInput {
                id: commandInput

                anchors.fill: parent
                anchors.leftMargin: Theme.spacingSmall
                anchors.rightMargin: Theme.spacingSmall
                verticalAlignment: TextInput.AlignVCenter
                color: Theme.textPrimary
                selectionColor: Theme.accentDim
                selectedTextColor: Theme.textPrimary
                font.family: Theme.monoFont
                font.pixelSize: 12
                clip: true
                selectByMouse: true
                onAccepted: root.confirmRequested()
                Keys.onEscapePressed: root.cancelRequested()
            }
        }

        Row {
            anchors.right: parent.right
            spacing: Theme.spacingSmall

            Rectangle {
                width: cancelText.width + 2 * Theme.spacingMedium
                height: 26
                radius: Theme.radius
                color: cancelArea.containsMouse ? Theme.surface2 : Theme.surface1
                border.color: Theme.borderSoft
                border.width: 1

                Text {
                    id: cancelText

                    anchors.centerIn: parent
                    text: qsTr("Cancelar")
                    color: Theme.textSecondary
                    font.pixelSize: 11
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
                width: confirmText.width + 2 * Theme.spacingMedium
                height: 26
                radius: Theme.radius
                color: confirmArea.pressed ? Theme.accentDim : Theme.accent

                Text {
                    id: confirmText

                    anchors.centerIn: parent
                    text: qsTr("Salvar")
                    color: Theme.background0
                    font.pixelSize: 11
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
