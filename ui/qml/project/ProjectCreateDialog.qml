import QtQuick

Rectangle {
    id: root

    property string dialogKind: "file"
    property string parentDisplayPath: ""
    property string errorText: ""
    property real maxAvailableWidth: 360

    signal confirmRequested()
    signal cancelRequested()

    width: Math.min(360, maxAvailableWidth)
    height: createColumn.height + 2 * Theme.spacingMedium
    radius: Theme.radius
    color: Theme.background2
    border.color: Theme.accent
    border.width: 1

    function resetAndFocus() {
        createNameInput.text = "";
        createNameInput.forceActiveFocus();
    }

    function currentName() {
        return createNameInput.text.trim();
    }

    Column {
        id: createColumn

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingMedium
        spacing: Theme.spacingSmall

        Text {
            text: root.dialogKind === "directory"
                  ? qsTr("Nova pasta") : qsTr("Novo arquivo")
            color: Theme.textPrimary
            font.pixelSize: 12
            font.bold: true
        }

        Text {
            width: parent.width
            text: root.parentDisplayPath
            color: Theme.textMuted
            font.pixelSize: 10
            elide: Text.ElideMiddle
        }

        Rectangle {
            width: parent.width
            height: 30
            radius: Theme.radius
            color: Theme.background0
            border.color: createNameInput.activeFocus
                          ? Theme.accent : Theme.borderSoft
            border.width: 1

            TextInput {
                id: createNameInput

                anchors.fill: parent
                anchors.margins: Theme.spacingSmall
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

        Text {
            width: parent.width
            visible: root.errorText !== ""
            text: root.errorText
            color: Theme.errorSoft
            font.pixelSize: 10
            wrapMode: Text.WordWrap
        }

        Row {
            anchors.right: parent.right
            spacing: Theme.spacingSmall

            Rectangle {
                width: createCancelText.width + 2 * Theme.spacingMedium
                height: 24
                radius: Theme.radius
                color: createCancelArea.containsMouse
                       ? Theme.surface2 : Theme.surface1
                border.color: Theme.borderSoft
                border.width: 1

                Text {
                    id: createCancelText

                    anchors.centerIn: parent
                    text: qsTr("Cancelar")
                    color: Theme.textSecondary
                    font.pixelSize: 11
                }

                MouseArea {
                    id: createCancelArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.cancelRequested()
                }
            }

            Rectangle {
                width: createConfirmText.width + 2 * Theme.spacingMedium
                height: 24
                radius: Theme.radius
                color: createConfirmArea.pressed ? Theme.accentDim : Theme.accent

                Text {
                    id: createConfirmText

                    anchors.centerIn: parent
                    text: qsTr("Criar")
                    color: Theme.background0
                    font.pixelSize: 11
                    font.bold: true
                }

                MouseArea {
                    id: createConfirmArea

                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.confirmRequested()
                }
            }
        }
    }
}
