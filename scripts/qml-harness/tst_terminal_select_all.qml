import QtQuick
import "../../ui/qml/panels/bottom"

Item {
    id: root
    property int failures: 0
    property string copied: "clipboard anterior"
    property string requestId: ""
    property string requestToken: ""
    property int copyRequests: 0

    QtObject {
        id: runtime
        signal terminalSelectAllRequested(string id, string selectionId)
        signal terminalCopySelectionRequested(string id, string selectionId)
        signal terminalSelectionCopied(string id, string selectionId, string text, bool valid)
        signal terminalSelectionFailed()
        onTerminalSelectAllRequested: function(id, token) { root.requestId = id; root.requestToken = token; }
        onTerminalCopySelectionRequested: root.copyRequests += 1
    }

    TerminalSelectionController {
        id: selection
        sessionId: "t1"
        runtimeController: runtime
        lines: [[{ text: "visivel", cells: 7 }]]
        onTextReady: function(text) { root.copied = text; }
    }

    function check(value, message) {
        if (!value) { failures++; console.error(message); }
    }

    Component.onCompleted: {
        selection.selectAll();
        const first = requestToken;
        check(requestId === "t1" && first !== "" && selection.hasSelection, "pedido da sessao ativa");
        check(copied === "clipboard anterior" && selection.selectedText() === "", "selecionar nao copia nem guarda historico QML");
        selection.handleCoreSelection(""); // frame anterior a selecao
        check(selection.hasSelection, "frame anterior nao cancela pedido pendente");
        selection.handleCoreSelection(first);
        selection.lines = [[{ text: "historico", cells: 9 }]];
        check(selection.hasSelection, "scroll preserva selecao completa");
        selection.copySelection();
        selection.copySelection();
        check(copyRequests === 1, "pedido de copia em curso nao duplica");
        runtime.terminalSelectionCopied("t2", first, "errado", true);
        check(copied === "clipboard anterior", "outra aba nao copia");
        runtime.terminalSelectionCopied("t1", first, "primeiro\nvisivel", true);
        check(copied === "primeiro\nvisivel" && selection.hasSelection, "copiar tudo preserva a selecao depois da resposta");

        selection.selectAll();
        const second = requestToken;
        selection.handleCoreSelection(second);
        selection.copySelection();
        runtime.terminalSelectionCopied("t1", second, "texto completo", true);
        check(selection.hasSelection && copied === "texto completo", "alias/menu preserva selecao");
        selection.copySelection();
        selection.handleCoreSelection(""); // output ou limpeza mudou o buffer
        runtime.terminalSelectionCopied("t1", second, "obsoleto", true);
        check(copied === "texto completo" && !selection.hasSelection, "output invalida copia atrasada");

        selection.selectAll();
        selection.copySelection();
        const closing = requestToken;
        selection.sessionId = "t3";
        runtime.terminalSelectionCopied("t1", closing, "sessao encerrada", true);
        check(copied === "texto completo" && !selection.hasSelection, "sessao nova nao herda selecao");
        selection.selectAll();
        selection.copySelection();
        runtime.terminalSelectionCopied("t3", requestToken, "", false);
        check(copied === "texto completo" && !selection.hasSelection, "core recusou snapshot obsoleto");

        selection.selectAll();
        selection.copySelection();
        selection.available = false;
        runtime.terminalSelectionCopied("t3", requestToken, "oculto", true);
        check(copied === "texto completo", "painel oculto cancela copia");
        selection.available = true;
        selection.selectVisible();
        selection.copySelection();
        check(copied === "historico" && selection.hasSelection, "copia local preserva selecao");
        selection.selectAll();
        runtime.terminalSelectionFailed();
        check(!selection.hasSelection, "erro libera selecao/teclas");

        selection.selectAll();
        const cancelled = requestToken;
        selection.handleCoreSelection(cancelled);
        selection.copySelection();
        selection.selectAll();
        const replacement = requestToken;
        selection.handleCoreSelection(replacement);
        selection.copySelection();
        runtime.terminalSelectionCopied("t3", cancelled, "gesto cancelado", true);
        check(selection.copyPending && copied === "historico", "resposta de gesto anterior nao consome o novo");
        runtime.terminalSelectionCopied("t3", replacement, "gesto atual", true);
        check(copied === "gesto atual" && selection.hasSelection, "novo gesto continua copiavel");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
