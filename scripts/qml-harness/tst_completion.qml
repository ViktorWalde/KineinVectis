import QtQuick
// Carrega o EditorCompletionController.qml REAL (arquivo do projeto, sem copia).
import "../../ui/qml/editor"

Item {
    id: root
    width: 100; height: 100
    property int completionRequests: 0

    // Fakes minimos com a mesma superficie que o controller usa.
    QtObject {
        id: fakeSurface
        property string text: "let x = St"
        property int cursorPosition: 10
        property bool editorActiveFocus: true
        function remove(a, b) {}
        function insert(p, t) {}
    }

    QtObject {
        id: fakeBridge
        property var editorSurface: fakeSurface
        function ready() { return true; }
        function text() { return fakeSurface.text; }
        function focusEditor() {}
    }

    QtObject {
        id: fakeDocs
        property int currentTab: 0
        function currentFilePath() { return "/tmp/x.rs"; }
    }

    QtObject {
        id: fakeText
        function wordStartAt(pos) { return 8; }              // inicio de "St"
        function cursorLineColumn() { return { line: 1, column: 11 }; }
    }

    // O controller REAL, dentro de um pai INVISIVEL — exatamente como
    // ele vive dentro do EditorController na aplicacao.
    Item {
        id: invisibleParent
        visible: false

        EditorCompletionController {
            id: completion
            surfaceBridge: fakeBridge
            documentController: fakeDocs
            textController: fakeText
            localItems: [
                { kind: "definition", name: "Status" },
                { kind: "reference", name: "String" },
                { kind: "scope", name: "ignorado" }
            ]
            onCompletionRequested: root.completionRequests += 1
        }
    }

    Component.onCompleted: {
        let falhas = 0;

        // 1) Servidor responde (como a sonda ja provava). Prefixo digitado = "St".
        completion.prefixStart = 8;
        completion.handleResolved([
            { label: "String", insertText: "String", detail: "struct", kind: "struct" },
            { label: "Struct", insertText: "Struct", detail: "kw", kind: "kw" },
            { label: "into_iter", insertText: "into_iter", detail: "fn", kind: "fn" }
        ], false);

        // O popup TEM que estar visivel e com os matches fuzzy de "St".
        if (completion.popupVisible !== true) falhas += 1;      // o bug D1
        if (completion.completionModel.count !== 2) falhas += 2; // String + Struct

        // 2) dismiss() fecha.
        completion.dismiss();
        if (completion.popupVisible !== false) falhas += 4;

        // 3) Sem itens -> nao abre.
        completion.prefixStart = 8;
        completion.handleResolved([], false);
        if (completion.popupVisible !== false) falhas += 8;

        // 4) Antes do LSP responder, o índice Tree-sitter já abre uma lista
        // local. Um segundo pedido enquanto o primeiro está pendente não pode
        // lotar o core com completion obsoleto.
        completion.clear();
        completion.requestCompletion();
        if (completion.popupVisible !== true) falhas += 16;
        if (completion.completionModel.count !== 2) falhas += 32;
        if (root.completionRequests !== 1) falhas += 64;
        completion.requestCompletion();
        if (root.completionRequests !== 1 || !completion.refreshQueued) falhas += 128;
        completion.handleFailed();
        if (completion.popupVisible !== true) falhas += 256;

        // 5) A escolha do usuario NAO pode ser desfeita por resposta do
        // servidor. Relato do autor: "a sugestao fica presa no primeiro item".
        // A janela e enorme — a A3.2 mediu a primeira completion do
        // rust-analyzer em 2520 ms — e nesse intervalo o usuario ja desceu na
        // lista que o fallback local abriu.
        const itens = [
            { label: "String", insertText: "String", detail: "struct", kind: "struct" },
            { label: "Struct", insertText: "Struct", detail: "kw", kind: "kw" }
        ];
        completion.clear();
        completion.prefixStart = 8;
        completion.handleResolved(itens, false);
        completion.move(1);                                  // desceu para "Struct"
        if (completion.index !== 1) falhas += 512;
        completion.handleResolved(itens, false);             // resposta atrasada chega
        if (completion.index !== 1) falhas += 1024;          // nao pode voltar para "String"

        // 6) Se o item escolhido SAIU da lista nova, cai para o primeiro: e o
        // unico indice que com certeza existe. Selecao fantasma seria pior que
        // reset — aceitaria um item que o usuario nao esta vendo.
        completion.clear();
        completion.prefixStart = 8;
        completion.handleResolved(itens, false);
        completion.move(1);                                  // "Struct"
        completion.handleResolved([itens[0]], false);        // "Struct" sumiu
        if (completion.index !== 0) falhas += 2048;

        // 7) Digitar e diferente de refresh: ao digitar, o ranking do servidor
        // muda e o topo volta a ser a melhor aposta (como VS Code).
        completion.clear();
        completion.prefixStart = 8;
        completion.handleResolved(itens, false);
        completion.move(1);
        completion.handleTextEdited();
        if (completion.index !== 0) falhas += 4096;
        // O codigo de saida de um processo tem 8 BITS: Qt.exit(256) sai como 0.
        // Enquanto o bitmask ia direto para o exit, todo check com bit >= 256
        // era letra morta: passava verde mesmo quebrado, que e exatamente a
        // doenca que esta suite existe para impedir. O mask agora vai para a
        // SAIDA (onde nao trunca) e o exit so diz passou/falhou.
        if (falhas !== 0) console.error("FALHAS bitmask=" + falhas);
        Qt.exit(falhas === 0 ? 0 : 1);
    }
}
