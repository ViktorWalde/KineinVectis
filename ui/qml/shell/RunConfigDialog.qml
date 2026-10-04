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

    // A roda do mouse dentro da caixa nao atravessa para o codigo (o clique
    // passa: so' a roda e' segurada, KvBackdrop).
    KvBackdrop {
        acceptedButtons: Qt.NoButton
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
            font.pixelSize: Theme.fontSizeSmall
        }

        KvTextField {
            id: nameInput

            width: parent.width
            height: 30
            codeFont: false
            onAccepted: commandInput.forceActiveFocus()
            Keys.onEscapePressed: root.cancelRequested()
        }

        Text {
            text: qsTr("Comando (roda na raiz do projeto)")
            color: Theme.textSecondary
            font.pixelSize: Theme.fontSizeSmall
        }

        KvTextField {
            id: commandInput

            width: parent.width
            height: 30
            onAccepted: root.confirmRequested()
            Keys.onEscapePressed: root.cancelRequested()
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
                text: qsTr("Salvar")
                onClicked: root.confirmRequested()
            }
        }
    }
}
