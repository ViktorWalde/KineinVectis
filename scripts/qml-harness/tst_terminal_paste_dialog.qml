import QtQuick
import KineinVectis

Item {
    id: root
    width: 640
    height: 360
    property int failures: 0
    property int previewsChecked: 0
    property string sent: ""
    TerminalInputController {
        id: input
        terminalActive: true
        sessionId: "dialog-test"
        onDataRequested: function(data) { root.sent = data; }
    }
    TerminalPasteDialog {
        id: dialog
        anchors.fill: parent
        visible: input.pastePending
        text: input.pendingPaste
        onCancelRequested: input.cancelPaste()
        onPasteRequested: function(singleLine) { input.confirmPaste(singleLine); }
    }
    function check(ok, reason) {
        if (!ok) { failures++; console.error(reason); }
    }
    function checkGeometry(item) {
        check(item.width >= 0 && item.height >= 0, "geometria negativa no dialogo");
        if (item.textFormat !== undefined && item.text !== undefined
                && String(item.text).indexOf("<b>literal") === 0) {
            previewsChecked++;
            check(item.textFormat === Text.PlainText, "preview nao interpreta markup");
        }
        for (let index = 0; index < item.children.length; index++)
            checkGeometry(item.children[index]);
    }
    Component.onCompleted: input.requestPaste("<b>literal</b>\ncomando\n")
    Timer {
        interval: 150
        running: true
        onTriggered: {
            check(dialog.visible && dialog.activeFocus, "confirmacao recebe foco");
            check(root.sent === "", "abrir dialogo nao envia clipboard");
            check(dialog.choice === 0, "cancelar e a escolha inicial");
            check(dialog.frameWidth <= root.width && dialog.frameHeight <= root.height,
                  "moldura cabe na janela");
            root.checkGeometry(dialog);
            check(root.previewsChecked > 0, "preview real foi inspecionado");
            dialog.activateChoice();
            check(!dialog.visible && root.sent === "", "cancelar nao envia bytes");
            input.requestPaste("primeiro\nsegundo\n");
            dialog.choice = 1;
            dialog.activateChoice();
            check(root.sent === "primeiro segundo", "acao explicita de uma linha");
            input.requestPaste("ultima\n");
            dialog.choice = 2;
            dialog.activateChoice();
            check(root.sent === "ultima\r", "confirmar preserva Enter final");
            root.width = 360;
            root.height = 260;
            input.requestPaste("<b>literal</b>\n" + "texto longo ".repeat(400));
            smallWindow.start();
        }
    }
    Timer {
        id: smallWindow
        interval: 100
        onTriggered: {
            root.checkGeometry(dialog);
            check(dialog.frameWidth <= root.width && dialog.frameHeight <= root.height,
                  "moldura cabe apos resize");
            check(dialog.choice === 0, "reabertura volta para Cancelar");
            input.sessionId = "other";
            check(!dialog.visible && root.sent === "ultima\r", "troca de sessao cancela");
            Qt.exit(root.failures === 0 ? 0 : 1);
        }
    }
}
