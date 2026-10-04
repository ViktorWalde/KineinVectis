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

    // A roda do mouse dentro da caixa nao atravessa para o codigo (o clique
    // passa: so' a roda e' segurada, KvBackdrop).
    KvBackdrop {
        acceptedButtons: Qt.NoButton
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
            font.pixelSize: Theme.fontSizeBody
            font.bold: true
        }

        KvTextField {
            id: renameInput

            width: parent.width
            height: 30
            onAccepted: root.confirmRequested()
            Keys.onEscapePressed: root.cancelRequested()
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
                text: qsTr("Cancelar")
                onClicked: root.cancelRequested()
            }

            KvButton {
                compact: true
                primary: true
                text: qsTr("Renomear")
                onClicked: root.confirmRequested()
            }
        }
    }
}
