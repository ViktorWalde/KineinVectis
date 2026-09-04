import QtQuick

// IR ATE' UMA POSICAO — abrindo o arquivo antes, se preciso.
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-03). Diagnostico do compilador, "ir
// para a definicao", resultado de busca e clique na pilha do debugger pedem
// todos a MESMA coisa: "leve o cursor para linha L, coluna C do arquivo F".
// Isso e' navegacao, nao gestao de abas, e vivia dentro do
// EditorDocumentController (548 linhas, quatro vocabularios).
//
// O PEDIDO PODE FICAR PENDENTE, e e' por isso que existe estado aqui: se o
// arquivo ainda nao esta aberto, o destino fica guardado e so' e' aplicado
// quando o conteudo chega do core (`hasPendingFor` + `applyPending`). Sem essa
// espera, o salto acontecia num buffer vazio e o cursor parava na posicao 0.
//
// LINHA E COLUNA SAO 1-BASED aqui (e' o que compilador e LSP-para-usuario
// falam); o offset em caracteres e' derivado na hora de aplicar.
//
// COMO CONVERSA. Recebe o `filesModel` e o `surfaceBridge`; PEDE por sinal
// (`tabSelectionRequested`, `readFileRequested`). Nao chama de volta para o
// EditorDocumentController — nao ha ciclo entre os dois.
Item {
    id: root

    visible: false

    property var filesModel: null
    property var surfaceBridge: null
    property string workspaceRoot: ""

    // Destino guardado enquanto o arquivo nao esta pronto. "" = nada pendente.
    property string pendingPath: ""
    property int pendingLine: 0
    property int pendingColumn: 0

    signal tabSelectionRequested(int index)
    signal readFileRequested(string path)

    function surfaceReady() {
        return surfaceBridge !== null && surfaceBridge.ready();
    }

    function reset() {
        pendingPath = "";
        pendingLine = 0;
        pendingColumn = 0;
    }

    function hasPendingFor(path) {
        return pendingPath === path;
    }

    // `file` pode vir relativo (o compilador emite assim); a raiz do workspace
    // completa o caminho.
    function request(file, line, column) {
        if (file === "") {
            return;
        }
        const path = file.startsWith("/") ? file : workspaceRoot + "/" + file;
        pendingPath = path;
        pendingLine = line;
        pendingColumn = column;
        for (let i = 0; i < filesModel.count; i++) {
            if (filesModel.get(i).path === path) {
                tabSelectionRequested(i);
                applyPending();
                return;
            }
        }
        readFileRequested(path);
    }

    // Converte linha/coluna em offset contando os "\n". Linha alem do fim do
    // arquivo para na ultima quebra encontrada, em vez de estourar.
    function applyPending() {
        if (!surfaceReady() || pendingPath === "") {
            return;
        }
        const text = surfaceBridge.text();
        let offset = 0;
        for (let currentLine = 1; currentLine < pendingLine; currentLine++) {
            const next = text.indexOf("\n", offset);
            if (next < 0) {
                break;
            }
            offset = next + 1;
        }
        if (pendingColumn > 0) {
            offset += pendingColumn - 1;
        }
        surfaceBridge.editorSurface.cursorPosition = Math.min(offset,
                                                              text.length);
        surfaceBridge.focusEditor();
        pendingPath = "";
    }
}
