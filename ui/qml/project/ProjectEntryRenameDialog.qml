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

    KvBackdrop {
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
                font.pixelSize: Theme.fontSizeBody
                font.bold: true
            }

            Text {
                width: parent.width
                visible: root.sourceDisplayPath !== ""
                text: qsTr("Origem: ") + root.sourceDisplayPath
                color: Theme.textMuted
                font.pixelSize: Theme.fontSizeCaption
                elide: Text.ElideMiddle
            }

            Text {
                width: parent.width
                text: root.sourceDisplayPath !== ""
                      ? qsTr("Destino: ") + root.entryDisplayPath : root.entryDisplayPath
                color: Theme.textMuted
                font.pixelSize: Theme.fontSizeCaption
                elide: Text.ElideMiddle
            }

            KvTextField {
                id: entryRenameInput

                width: parent.width
                height: 30
                onAccepted: { if (!root.operationPending) root.confirmRequested(); }
                Keys.onEscapePressed: {
                    if (!root.operationPending) root.cancelRequested();
                    else if (root.pendingDismissText !== "") root.pendingDismissRequested();
                }
            }

            Text {
                width: parent.width
                visible: root.operationPending
                text: root.pendingMessage
                color: Theme.textSecondary
                font.pixelSize: Theme.fontSizeCaption
            }

            Text {
                width: parent.width
                visible: root.errorText !== ""
                text: root.errorText
                color: Theme.errorSoft
                font.pixelSize: Theme.fontSizeCaption
                wrapMode: Text.WordWrap
            }

            Row {
                anchors.right: parent.right
                spacing: Theme.spacingSmall

                KvButton {
                    compact: true
                    enabled: !root.operationPending || root.pendingDismissText !== ""
                    text: root.operationPending && root.pendingDismissText !== ""
                          ? root.pendingDismissText : qsTr("Cancelar")
                    onClicked: {
                        if (root.operationPending) root.pendingDismissRequested();
                        else root.cancelRequested();
                    }
                }

                KvButton {
                    compact: true
                    primary: true
                    enabled: !root.operationPending
                    text: root.confirmText !== "" ? root.confirmText : qsTr("Renomear")
                    onClicked: root.confirmRequested()
                }
            }
        }
    }
}
