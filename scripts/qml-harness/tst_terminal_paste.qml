import QtQuick
import "../../ui/qml/panels/bottom"

Item {
    id: root
    property string sent: ""
    property int failures: 0
    TerminalInputController {
        id: input
        terminalActive: true
        sessionId: "a"
        onDataRequested: function(data) { root.sent = data; }
    }
    function check(ok, reason) {
        if (!ok) { failures++; console.error(reason); }
    }
    Component.onCompleted: {
        input.requestPaste("simple");
        check(sent === "simple" && !input.pastePending, "paste simples");
        input.requestPaste("printf primeiro\nprintf segundo\n");
        check(sent === "simple" && input.pastePending, "multilinha requer decisao");
        input.requestPaste("novo paste por clique do meio");
        check(sent === "simple" && input.pendingPaste === "printf primeiro\nprintf segundo\n",
              "novo pedido nao contorna nem substitui confirmacao aberta");
        input.cancelPaste();
        check(!input.pastePending && sent === "simple", "cancelar nao envia bytes");
        input.requestPaste("comando\r");
        check(input.pastePending, "CR isolado pode executar comando");
        input.confirmPaste(true);
        check(sent === "comando", "uma linha exige Enter posterior");
        input.requestPaste("primeiro\r\nsegundo\n");
        input.confirmPaste(false);
        check(sent === "primeiro\rsegundo\r", "CRLF nao envia dois Enter");
        input.bracketedPaste = true;
        input.requestPaste("um\ndois");
        check(!input.pastePending && sent === "\x1b[200~um\rdois\x1b[201~", "modo VT respeitado");
        input.requestPaste("\x1b[201~intruso\n");
        check(input.pastePending, "ESC requer revisao");
        input.confirmPaste(false);
        check(sent === "\x1b[200~\u241b[201~intruso\r\x1b[201~", "payload nao pode fechar envelope");
        input.bracketedPaste = false;
        input.requestPaste("segredo\n");
        const before = sent;
        input.sessionId = "b";
        input.confirmPaste(false);
        check(sent === before && !input.pastePending, "confirmacao nao migra para outra sessao");
        input.requestPaste("pendente\n");
        input.terminalActive = false;
        input.confirmPaste(false);
        check(sent === before && !input.pastePending, "sessao encerrada cancela colagem");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
