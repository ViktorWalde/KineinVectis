import QtQuick
import "../../ui/qml/editor"

// O EditorSelectionLadder e o EditorTextGeometry REAIS.
//
// POR QUE ESTE TESTE EXISTE (2026-09-04). A escada de selecao e' o unico
// pedaco do editor com MEMORIA, e a memoria e' fragil de proposito: o
// historico so' vale enquanto a selecao atual for a ultima expansao registrada
// sobre um texto do MESMO TAMANHO. Ate' 2026-09-03 ela morava dentro do
// EditorTextController, e essa invariante nao tinha teste nenhum — quebra-la
// nao produzia erro, produzia um Ctrl+Shift+W que encolhe para o lugar errado.
Item {
    id: root
    width: 100; height: 100

    // "    let x = foo(bar);"  — indices uteis:
    //   0..3 indentacao   4 'l'   15 '('   16 'b'   19 ')'   20 ';'  len 21
    readonly property string amostra: "    let x = foo(bar);"

    QtObject {
        id: fakeSurface
        property string text: ""
        property int cursorPosition: 0
        property int selectionStart: 0
        property int selectionEnd: 0
        function select(a, b) { selectionStart = a; selectionEnd = b; cursorPosition = b; }
    }

    QtObject {
        id: fakeBridge
        property var editorSurface: fakeSurface
        function ready() { return true; }
        function text() { return fakeSurface.text; }
    }

    Item {
        visible: false

        EditorTextGeometry {
            id: geo
            surfaceBridge: fakeBridge
        }

        EditorSelectionLadder {
            id: ladder
            surfaceBridge: fakeBridge
            geometry: geo
        }
    }

    function put(cursor) {
        fakeSurface.text = root.amostra;
        fakeSurface.cursorPosition = cursor;
        fakeSurface.selectionStart = cursor;
        fakeSurface.selectionEnd = cursor;
        ladder.reset();
    }

    function selecao() {
        return fakeSurface.selectionStart + ".." + fakeSurface.selectionEnd;
    }

    Component.onCompleted: {
        let f = 0;

        // A escada, subindo do cursor dentro de "bar":
        //   palavra -> par COM delimitadores -> linha sem indentacao -> documento
        put(17);
        ladder.expand();
        if (selecao() !== "16..19") f += 1;          // "bar"
        ladder.expand();
        if (selecao() !== "15..20") f += 2;          // "(bar)"
        ladder.expand();
        if (selecao() !== "4..21") f += 4;           // linha sem indentacao
        ladder.expand();
        if (selecao() !== "0..21") f += 8;           // documento

        // No topo, expandir de novo nao mexe em nada.
        ladder.expand();
        if (selecao() !== "0..21") f += 16;

        // E descendo, degrau por degrau, na ordem inversa exata.
        ladder.shrink();
        if (selecao() !== "4..21") f += 32;
        ladder.shrink();
        if (selecao() !== "15..20") f += 64;
        ladder.shrink();
        if (selecao() !== "16..19") f += 128;

        // O ultimo degrau volta ao CURSOR de onde a escada partiu — a
        // primeira expansao tambem empilhou, entao Ctrl+Shift+W desfaz tudo.
        ladder.shrink();
        if (selecao() !== "17..17") f += 256;
        // Sem historico, encolher nao adivinha: fica onde esta'.
        ladder.shrink();
        ladder.shrink();
        if (selecao() !== "17..17") f += 512;

        // A GUARDA DO HISTORICO. Depois de duas expansoes, uma selecao feita
        // POR FORA (mouse) invalida a pilha: o shrink seguinte nao pode voltar
        // para um degrau que nao descreve mais o que esta na tela.
        put(17);
        ladder.expand();
        ladder.expand();
        fakeSurface.select(0, 3);
        ladder.shrink();
        if (selecao() !== "0..3") f += 1024;

        // E o TAMANHO do texto tambem invalida: editar entre expandir e
        // encolher desloca todos os offsets guardados.
        put(17);
        ladder.expand();
        ladder.expand();
        fakeSurface.text = root.amostra + "\n";
        ladder.shrink();
        if (selecao() !== "15..20") f += 2048;

        // Cursor DENTRO da indentacao pula a linha-sem-indentacao: aquele
        // candidato COMECA depois do cursor, entao nao o contem. O primeiro
        // degrau vira a linha inteira. Nao e' defeito — e' a consequencia
        // direta de "o menor candidato que CONTEM a selecao".
        put(2);
        ladder.expand();
        if (selecao() !== "0..21") f += 4096;

        // Texto desbalanceado nao trava nem inventa par: um fechador sem
        // abridor e' ignorado, e a escada segue pela linha.
        fakeSurface.text = "a) b";
        fakeSurface.cursorPosition = 3;
        fakeSurface.selectionStart = 3;
        fakeSurface.selectionEnd = 3;
        ladder.reset();
        ladder.expand();
        if (selecao() !== "3..4") f += 8192;         // a palavra "b"
        ladder.expand();
        if (selecao() !== "0..4") f += 16384;        // a linha inteira

        // O codigo de saida de um processo tem 8 BITS: o mask vai para a SAIDA
        // (onde nao trunca) e o exit so diz passou/falhou.
        if (f !== 0) console.error("FALHAS bitmask=" + f);
        Qt.exit(f === 0 ? 0 : 1);
    }
}
