pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Confirmacao de colagem, composta com a moldura e botoes comuns da IDE.
// O texto pendente e o envio continuam no TerminalInputController.
KvPanelFrame {
    id: root
    property string text: ""
    property int choice: 0
    signal cancelRequested()
    signal pasteRequested(bool singleLine)

    panelWidth: 560
    panelHeight: 340
    contentHeight: 260
    maxAvailableWidth: Math.max(0, width - 32)
    maxAvailableHeight: Math.max(0, height - 32)
    Accessible.role: Accessible.Dialog
    Accessible.name: heading.text
    onDismissRequested: cancelRequested()
    onVisibleChanged: if (visible) { choice = 0; Qt.callLater(takeFocus); }

    function takeFocus() { if (visible) forceActiveFocus(); }
    function activateChoice() {
        if (choice === 0) cancelRequested();
        else pasteRequested(choice === 1);
    }

    Keys.onPressed: function(event) {
        event.accepted = true;
        if (event.key === Qt.Key_Escape) root.cancelRequested();
        else if (event.key === Qt.Key_Tab || event.key === Qt.Key_Right)
            root.choice = (root.choice + 1) % 3;
        else if (event.key === Qt.Key_Backtab || event.key === Qt.Key_Left)
            root.choice = (root.choice + 2) % 3;
        else if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter
                 || event.key === Qt.Key_Space) root.activateChoice();
    }

    Text {
        id: heading
        width: parent.width
        text: qsTr("Revisar colagem no terminal")
        color: Theme.textPrimary
        font.pixelSize: 15
        font.bold: true
    }
    Text {
        id: warning
        anchors.top: heading.bottom
        anchors.topMargin: Theme.spacingSmall
        width: parent.width
        text: qsTr("O texto contém quebras de linha ou controles que podem executar comandos. Confira antes de colar. ESC será exibido como ␛.")
        textFormat: Text.PlainText
        wrapMode: Text.WordWrap
        color: Theme.textSecondary
        font.pixelSize: 12
    }
    Flickable {
        anchors.top: warning.bottom
        anchors.topMargin: Theme.spacingMedium
        anchors.bottom: buttons.top
        anchors.bottomMargin: Theme.spacingMedium
        width: parent.width
        clip: true
        contentWidth: width
        contentHeight: preview.height
        boundsBehavior: Flickable.StopAtBounds
        Text {
            id: preview
            width: parent.width
            text: root.text.slice(0, 3000).replace(/\r\n?/g, "\n")
                .replace(/[\x00-\x08\x0b\x0c\x0e-\x1f\x7f]/g, function(ch) {
                    return "<0x" + ch.charCodeAt(0).toString(16) + ">";
                }) + (root.text.length > 3000 ? "\n…" : "")
            textFormat: Text.PlainText
            wrapMode: Text.WrapAnywhere
            font.family: Theme.monoFont
            font.pixelSize: 12
            color: Theme.textPrimary
        }
    }
    Flow {
        id: buttons
        anchors.bottom: parent.bottom
        width: parent.width
        spacing: Theme.spacingSmall
        KvButton {
            text: qsTr("Cancelar")
            selected: root.choice === 0
            onClicked: root.cancelRequested()
        }
        KvButton {
            text: qsTr("Colar em uma linha")
            selected: root.choice === 1
            onClicked: root.pasteRequested(true)
        }
        KvButton {
            text: qsTr("Colar")
            selected: root.choice === 2
            onClicked: root.pasteRequested(false)
        }
    }
}
