import QtQuick

Rectangle {
    id: root

    property string errorText: ""
    property real maxAvailableWidth: 360

    signal confirmRequested()
    signal cancelRequested()

    width: Math.min(360, maxAvailableWidth)
    height: renameColumn.height + 2 * Theme.spacingMedium
    radius: Theme.radius
    color: Theme.background2
    border.color: Theme.accent
    border.width: 1

    function openWithName(name) {
        renameInput.text = name;
        renameInput.forceActiveFocus();
        renameInput.selectAll();
    }

    function currentName() {
        return renameInput.text.trim();
    }

    Column {
        id: renameColumn

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingMedium
        spacing: Theme.spacingSmall

        Text {
            text: qsTr("Renomear simbolo")
            color: Theme.textPrimary
            font.pixelSize: 12
            font.bold: true
        }

        Rectangle {
            width: parent.width
            height: 30
            radius: Theme.radius
            color: Theme.background0
            border.color: renameInput.activeFocus ? Theme.accent : Theme.borderSoft
            border.width: 1

            TextInput {
                id: renameInput

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
                width: renameCancelText.width + 2 * Theme.spacingMedium
                height: 24
                radius: Theme.radius
                color: renameCancelArea.containsMouse
                       ? Theme.surface2 : Theme.surface1
                border.color: Theme.borderSoft
                border.width: 1

                Text {
                    id: renameCancelText

                    anchors.centerIn: parent
                    text: qsTr("Cancelar")
                    color: Theme.textSecondary
                    font.pixelSize: 11
                }

                MouseArea {
                    id: renameCancelArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.cancelRequested()
                }
            }

            Rectangle {
                width: renameConfirmText.width + 2 * Theme.spacingMedium
                height: 24
                radius: Theme.radius
                color: renameConfirmArea.pressed ? Theme.accentDim : Theme.accent

                Text {
                    id: renameConfirmText

                    anchors.centerIn: parent
                    text: qsTr("Renomear")
                    color: Theme.background0
                    font.pixelSize: 11
                    font.bold: true
                }

                MouseArea {
                    id: renameConfirmArea

                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.confirmRequested()
                }
            }
        }
    }
}
