import QtQuick
import KineinVectis

Rectangle {
    id: root

    property bool editing: false
    property bool busy: false
    property string errorText: ""
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

    function moveFieldFocus(event, nextItem, previousItem) {
        const reverse = event.key === Qt.Key_Backtab
                || (event.key === Qt.Key_Tab && (event.modifiers & Qt.ShiftModifier));
        if (event.key !== Qt.Key_Tab && !reverse) return;
        (reverse ? previousItem : nextItem).forceActiveFocus();
        event.accepted = true;
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
            objectName: "runConfigName"
            enabled: !root.busy

            width: parent.width
            height: 30
            codeFont: false
            onAccepted: commandInput.forceActiveFocus()
            onKeyPressed: (event) => root.moveFieldFocus(event, commandInput, saveButton.enabled ? saveButton : cancelButton)
            Keys.onEscapePressed: root.cancelRequested()
        }

        Text {
            text: qsTr("Comando (roda na raiz do projeto)")
            color: Theme.textSecondary
            font.pixelSize: Theme.fontSizeSmall
        }

        KvTextField {
            id: commandInput
            objectName: "runConfigCommand"
            enabled: !root.busy

            width: parent.width
            height: 30
            onAccepted: root.confirmRequested()
            onKeyPressed: (event) => root.moveFieldFocus(event, cancelButton, nameInput)
            Keys.onEscapePressed: root.cancelRequested()
        }

        KvVerdict {
            objectName: "runConfigError"
            width: parent.width
            message: root.errorText
            heading: qsTr("Não foi possível salvar a configuração")
            codeFont: false
            animateChanges: true
        }

        Row {
            anchors.right: parent.right
            spacing: Theme.spacingSmall

            KvButton {
                id: cancelButton
                objectName: "runConfigCancel"
                KeyNavigation.tab: saveButton.enabled ? saveButton : nameInput
                KeyNavigation.backtab: commandInput
                Keys.onEscapePressed: root.cancelRequested()
                compact: true
                text: qsTr("Cancelar")
                onClicked: root.cancelRequested()
            }

            KvButton {
                id: saveButton
                objectName: "runConfigSave"
                KeyNavigation.tab: nameInput
                KeyNavigation.backtab: cancelButton
                Keys.onEscapePressed: root.cancelRequested()
                compact: true
                primary: true
                text: root.busy ? qsTr("Salvando…") : qsTr("Salvar")
                enabled: !root.busy && nameInput.text.trim() !== "" && commandInput.text.trim() !== ""
                onClicked: root.confirmRequested()
            }
        }
    }
}
