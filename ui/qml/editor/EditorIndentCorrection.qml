import QtQuick

// AS TRES TRAVAS DA CORRECAO ESTRUTURAL DE INDENTACAO (E1, 2026-09-26).
//
// O fallback local ja' esta' na tela quando esta conversa comeca — a tecla
// nunca espera. O que acontece aqui e' guardar o que foi perguntado e, quando a
// gramatica responder, decidir se a resposta ainda vale.
//
// As travas, e a razao de cada uma:
//
//   1. MESMO DOCUMENTO — a resposta pode chegar depois de o autor trocar de
//      aba, e indentar o arquivo errado e' pior que nao indentar;
//   2. MESMA VERSAO — o core responde a partir da arvore que TEM, que pode ser
//      anterior ao que o autor ja' digitou. A versao que volta e' a da ARVORE,
//      nao a que foi pedida, e comparar as duas e' o motivo de o
//      `ParsedDocument` guardar versao;
//   3. MESMO TEXTO — conferida por quem aplica: o que esta' na linha ainda tem
//      de ser exatamente o que o fallback pos.
//
// A terceira existe porque as duas primeiras dependem de o core estar correto.
// A UI nao confia, confere.
QtObject {
    id: root

    property int lineStart: -1
    property string appliedIndent: ""
    property string path: ""
    property int version: 0

    // A resposta passou nas travas: quem tem o texto aplica.
    signal approved(int lineStart, string appliedIndent, int level)

    readonly property bool pending: root.lineStart >= 0

    // Guarda o que o fallback aplicou, antes de o pedido sair.
    function remember(lineStart, appliedIndent) {
        root.lineStart = lineStart;
        root.appliedIndent = appliedIndent;
    }

    // O carimbo de quem sabe o caminho e a versao.
    function stamp(path, version) {
        root.path = path;
        root.version = version;
    }

    function forget() {
        root.lineStart = -1;
        root.appliedIndent = "";
        root.path = "";
        root.version = 0;
    }

    // Devolve `true` quando a resposta passou nas duas primeiras travas e foi
    // encaminhada. `false` e' resultado normal, nao erro.
    function accept(path, version, level) {
        const askedLineStart = root.lineStart;
        const askedIndent = root.appliedIndent;
        const askedPath = root.path;
        const askedVersion = root.version;
        root.forget();
        if (askedLineStart < 0 || level < 0 || path === "") {
            return false;
        }
        if (path !== askedPath || version !== askedVersion) {
            return false;
        }
        root.approved(askedLineStart, askedIndent, level);
        return true;
    }
}
