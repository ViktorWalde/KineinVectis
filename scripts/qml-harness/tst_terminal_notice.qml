import QtQuick
import KineinVectis

Item {
    id: root
    width: 360
    height: 240
    property int failures: 0
    function check(ok, message) { if (!ok) { failures++; console.error(message); } }
    function find(item, name) {
        if (item.objectName === name) return item;
        for (const child of item.children) {
            const found = find(child, name);
            if (found) return found;
        }
        return null;
    }
    TerminalPanel {
        id: panel
        anchors.fill: parent
        render: ({id: "t1", cols: 80, rows: 24, lines: [[{text:"program-output", cells:14}]],
                  notice: {code:"INTERNAL_ERROR", message:"Estado do processo indisponível; tente fechar novamente."}})
    }
    Component.onCompleted: Qt.callLater(root.verifyNotice)
    function verifyNotice() {
        const error = find(panel, "terminalCommandError");
        const viewport = find(panel, "terminalViewport");
        check(error.visible && error.heading === "Falha no terminal", "Aviso do core aparece na faixa existente");
        check(error.message === panel.render.notice.message, "Mensagem vem do core");
        check(viewport.y >= error.height && viewport.height > 0, "Faixa reserva espaço sem cobrir o terminal");
        check(panel.lines[0][0].text === "program-output", "Aviso não altera saída do programa");
        panel.render = {id:"t1", cols:80, rows:24, lines:[[{text:"program-output", cells:14}]]};
        Qt.callLater(root.verifyRecovery);
    }
    function verifyRecovery() {
        check(!find(panel, "terminalCommandError").visible && panel.processNotice === "", "Recuperação remove o aviso");
        check(find(panel, "terminalViewport").y === 0, "Grid recupera o espaço");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
