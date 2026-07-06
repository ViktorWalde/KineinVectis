import QtQuick

Item {
    id: root

    property string entryKind: "file"
    property string entryDisplayPath: ""
    property string errorText: ""
    property real maxAvailableWidth: 360

    signal confirmRequested()
    signal cancelRequested()

    function openWithName(name) {
        entryRenameInput.text = name;
        entryRenameInput.forceActiveFocus();
        entryRenameInput.selectAll();
    }

    function currentName() {
        return entryRenameInput.text.trim();
    }

    MouseArea {
        anchors.fill: parent
    }

    Rectangle {
        anchors.centerIn: parent
        width: Math.min(360, maxAvailableWidth)
        height: entryRenameColumn.height + 2 * Theme.spacingMedium
        radius: Theme.radius
        color: Theme.background2
        border.color: Theme.accent
        border.width: 1

        Column {
            id: entryRenameColumn

            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.margins: Theme.spacingMedium
            spacing: Theme.spacingSmall

            Text {
                text: root.entryKind === "directory"
                      ? qsTr("Renomear pasta") : qsTr("Renomear arquivo")
                color: Theme.textPrimary
                font.pixelSize: 12
                font.bold: true
            }

            Text {
                width: parent.width
                text: root.entryDisplayPath
                color: Theme.textMuted
                font.pixelSize: 10
                elide: Text.ElideMiddle
            }

            Rectangle {
                width: parent.width
                height: 30
                radius: Theme.radius
                color: Theme.background0
                border.color: entryRenameInput.activeFocus
                              ? Theme.accent : Theme.borderSoft
                border.width: 1

                TextInput {
                    id: entryRenameInput

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
                    width: entryRenameCancelText.width + 2 * Theme.spacingMedium
                    height: 24
                    radius: Theme.radius
                    color: entryRenameCancelArea.containsMouse
                           ? Theme.surface2 : Theme.surface1
                    border.color: Theme.borderSoft
                    border.width: 1

                    Text {
                        id: entryRenameCancelText

                        anchors.centerIn: parent
                        text: qsTr("Cancelar")
                        color: Theme.textSecondary
                        font.pixelSize: 11
                    }

                    MouseArea {
                        id: entryRenameCancelArea

                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.cancelRequested()
                    }
                }

                Rectangle {
                    width: entryRenameConfirmText.width + 2 * Theme.spacingMedium
                    height: 24
                    radius: Theme.radius
                    color: entryRenameConfirmArea.pressed
                           ? Theme.accentDim : Theme.accent

                    Text {
                        id: entryRenameConfirmText

                        anchors.centerIn: parent
                        text: qsTr("Renomear")
                        color: Theme.background0
                        font.pixelSize: 11
                    }

                    MouseArea {
                        id: entryRenameConfirmArea

                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.confirmRequested()
                    }
                }
            }
        }
    }
}
