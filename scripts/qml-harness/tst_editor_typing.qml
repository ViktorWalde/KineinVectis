import QtQuick
import "../../ui/qml/editor"

// O EditorTypingController REAL (arquivo do projeto, sem copia).
//
// POR QUE ESTE TESTE EXISTE (2026-09-04). O auto-close de pares e o roteamento
// de tecla viveram ate' 2026-09-03 dentro do EditorTextSurface.qml, um
// componente VISUAL que nenhum harness instancia — a regra so' rodava na IDE
// de verdade, e um erro nela chegava ao autor antes de chegar ao gate. Quando
// a regra ganhou dono proprio, ela ficou exercitavel: o controller fala com a
// superficie pela API publica dela, entao cinco funcoes falsas bastam.
Item {
    id: root
    width: 100; height: 100

    // Superficie FALSA que muta o texto DE VERDADE — sem isso, "inseriu o
    // fechador" nao se distingue de "nao fez nada".
    QtObject {
        id: fakeSurface
        property string text: ""
        property int cursorPosition: 0
        property int selectionStart: 0
        property int selectionEnd: 0
        function remove(a, b) { text = text.substring(0, a) + text.substring(b); }
        function insert(p, t) { text = text.substring(0, p) + t + text.substring(p); }
        function select(a, b) { selectionStart = a; selectionEnd = b; cursorPosition = b; }
    }

    property int indents: 0
    property int closerBraces: 0
    property int completionDismissals: 0
    property int actionsAccepts: 0
    property int newlines: 0

    Item {
        visible: false
        EditorTypingController {
            id: typing
            surface: fakeSurface
            onIndentRequested: root.indents += 1
            onCloserBraceRequested: root.closerBraces += 1
            onCompletionDismissRequested: root.completionDismissals += 1
            onActionsAcceptRequested: root.actionsAccepts += 1
            onNewlineRequested: root.newlines += 1
        }
    }

    function put(content, cursor) {
        fakeSurface.text = content;
        fakeSurface.cursorPosition = cursor;
        fakeSurface.selectionStart = cursor;
        fakeSurface.selectionEnd = cursor;
    }

    function tecla(texto) {
        return { text: texto, modifiers: Qt.NoModifier, key: 0 };
    }

    Component.onCompleted: {
        let f = 0;

        // 1) abridor fecha o par e deixa o cursor DENTRO
        put("foo ", 4);
        if (!typing.handleTypingKey(tecla("("))) f += 1;
        if (fakeSurface.text !== "foo ()" || fakeSurface.cursorPosition !== 5) f += 2;

        // 2) type-over: o fechador ja presente e' pulado, nao duplicado
        put("()", 1);
        if (!typing.handleTypingKey(tecla(")"))) f += 4;
        if (fakeSurface.text !== "()" || fakeSurface.cursorPosition !== 2) f += 8;

        // 3) abridor com selecao ENVOLVE em vez de substituir
        fakeSurface.text = "abc";
        fakeSurface.selectionStart = 0;
        fakeSurface.selectionEnd = 3;
        fakeSurface.cursorPosition = 3;
        if (!typing.handleTypingKey(tecla("("))) f += 16;
        if (fakeSurface.text !== "(abc)") f += 32;
        if (fakeSurface.selectionStart !== 1 || fakeSurface.selectionEnd !== 4) f += 64;

        // 4) aspa colada em palavra NAO duplica (don't -> don''t)
        put("don", 3);
        if (typing.handleTypingKey(tecla("'"))) f += 128;
        if (fakeSurface.text !== "don") f += 256;

        // 5) CR1: "<" so' fecha logo depois de `#include `
        put("#include ", 9);
        if (!typing.handleTypingKey(tecla("<"))) f += 512;
        if (fakeSurface.text !== "#include <>") f += 1024;
        put("a < ", 2);
        if (typing.handleTypingKey(tecla("<"))) f += 2048;

        // 6) backspace entre par VAZIO apaga os dois
        put("()", 1);
        if (!typing.handleBackspace()) f += 4096;
        if (fakeSurface.text !== "") f += 8192;
        // com conteudo dentro, NAO apaga o par
        put("(a)", 1);
        if (typing.handleBackspace()) f += 16384;

        // 7) setting desligado neutraliza as duas entradas
        typing.autoCloseEnabled = false;
        put("foo ", 4);
        if (typing.handleTypingKey(tecla("("))) f += 32768;
        put("()", 1);
        if (typing.handleBackspace()) f += 65536;
        typing.autoCloseEnabled = true;

        // 8) roteamento: Tab pede indentacao
        put("x", 1);
        if (!typing.route({ text: "", modifiers: Qt.NoModifier, key: Qt.Key_Tab })) f += 131072;
        if (root.indents !== 1) f += 262144;

        // 9) "}" digitado pede o dedent ao dono do texto, nao insere aqui
        put("    ", 4);
        if (!typing.handleTypingKey(tecla("}"))) f += 524288;
        if (root.closerBraces !== 1 || fakeSurface.text !== "    ") f += 1048576;

        // 10) o que esta ABERTO muda o significado da tecla: com a lista de
        // acoes visivel, Enter ACEITA em vez de quebrar linha.
        typing.actionsVisible = true;
        if (!typing.route({ text: "", modifiers: Qt.NoModifier, key: Qt.Key_Return })) f += 2097152;
        if (root.actionsAccepts !== 1 || root.newlines !== 0) f += 4194304;
        typing.actionsVisible = false;

        // 11) Esc com completacao aberta fecha a completacao
        typing.completionVisible = true;
        if (!typing.route({ text: "", modifiers: Qt.NoModifier, key: Qt.Key_Escape })) f += 8388608;
        if (root.completionDismissals !== 1) f += 16777216;
        typing.completionVisible = false;

        // 12) sem nada aberto, Enter quebra linha
        if (!typing.route({ text: "", modifiers: Qt.NoModifier, key: Qt.Key_Return })) f += 33554432;
        if (root.newlines !== 1) f += 67108864;

        // O codigo de saida de um processo tem 8 BITS: o mask vai para a SAIDA
        // (onde nao trunca) e o exit so diz passou/falhou.
        if (f !== 0) console.error("FALHAS bitmask=" + f);
        Qt.exit(f === 0 ? 0 : 1);
    }
}
