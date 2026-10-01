import QtQuick

Item {
    id: root

    property string entryKind: "file"
    property string entryDisplayPath: ""
    property string sourceDisplayPath: ""
    property string errorText: ""
    property string titleText: ""
    property string confirmText: ""
    property bool operationPending: false
    property string pendingMessage: ""
    property string pendingDismissText: ""
    property real maxAvailableWidth: 360

    signal confirmRequested()
    signal cancelRequested()
    signal pendingDismissRequested()

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
        width: Math.min(360, root.maxAvailableWidth)
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
                text: root.titleText !== "" ? root.titleText
                      : root.entryKind === "directory"
                        ? qsTr("Renomear pasta") : qsTr("Renomear arquivo")
                color: Theme.textPrimary
                font.pixelSize: 12
                font.bold: true
            }

            Text {
                width: parent.width
                visible: root.sourceDisplayPath !== ""
                text: qsTr("Origem: ") + root.sourceDisplayPath
                color: Theme.textMuted
                font.pixelSize: 10
                elide: Text.ElideMiddle
            }

            Text {
                width: parent.width
                text: root.sourceDisplayPath !== ""
                      ? qsTr("Destino: ") + root.entryDisplayPath : root.entryDisplayPath
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
                    onAccepted: { if (!root.operationPending) root.confirmRequested(); }
                    Keys.onEscapePressed: {
                        if (!root.operationPending) root.cancelRequested();
                        else if (root.pendingDismissText !== "") root.pendingDismissRequested();
                    }
                }
            }

            Text {
                width: parent.width
                visible: root.operationPending
                text: root.pendingMessage
                color: Theme.textSecondary
                font.pixelSize: 10
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
                        text: root.operationPending && root.pendingDismissText !== ""
                              ? root.pendingDismissText : qsTr("Cancelar")
                        color: root.operationPending && root.pendingDismissText === ""
                               ? Theme.textMuted : Theme.textSecondary
                        font.pixelSize: 11
                    }

                    MouseArea {
                        id: entryRenameCancelArea

                        anchors.fill: parent
                        enabled: !root.operationPending || root.pendingDismissText !== ""
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            if (root.operationPending) root.pendingDismissRequested();
                            else root.cancelRequested();
                        }
                    }
                }

                Rectangle {
                    width: entryRenameConfirmText.width + 2 * Theme.spacingMedium
                    height: 24
                    radius: Theme.radius
                    color: root.operationPending ? Theme.surface1
                           : entryRenameConfirmArea.pressed ? Theme.accentDim : Theme.accent

                    Text {
                        id: entryRenameConfirmText

                        anchors.centerIn: parent
                        text: root.confirmText !== "" ? root.confirmText : qsTr("Renomear")
                        color: root.operationPending ? Theme.textMuted : Theme.background0
                        font.pixelSize: 11
                    }

                    MouseArea {
                        id: entryRenameConfirmArea

                        anchors.fill: parent
                        enabled: !root.operationPending
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.confirmRequested()
                    }
                }
            }
        }
    }
}
