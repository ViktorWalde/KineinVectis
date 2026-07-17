import QtQuick
import "../../ui/qml/editor"

Item {
    id: root
    width: 100; height: 100

    // Superficie FALSA que muta o texto de verdade (remove/insert reais),
    // pra que replace/replaceAll sejam exercitados de fato.
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

    QtObject {
        id: fakeBridge
        property var editorSurface: fakeSurface
        function ready() { return true; }
        function text() { return fakeSurface.text; }
        function focusEditor() {}
    }

    QtObject {
        id: fakeText
        function currentWord() { return ""; }
    }

    // Controller REAL sob pai INVISIVEL (mesma hierarquia da aplicacao —
    // regressao do D1 de brinde).
    Item {
        visible: false
        EditorFindController {
            id: find
            surfaceBridge: fakeBridge
            textController: fakeText
        }
    }

    function reset(t) {
        fakeSurface.text = t;
        fakeSurface.select(0, 0);
        find.query = ""; find.replacement = "";
        find.caseSensitive = false; find.wholeWord = false; find.useRegex = false;
        find.barVisible = true;
    }

    Component.onCompleted: {
        let f = 0;

        // 1) Busca literal, case-insensitive por padrao. "foo Foo foobar"
        reset("foo Foo foobar");
        find.setQuery("foo");
        if (find.matchCount !== 3) f += 1;

        // 2) barVisible tem que LER true (regressao do bug D1 no novo controller)
        if (find.barVisible !== true) f += 2;

        // 3) Case-sensitive: so "foo" e "foobar" -> 2
        find.toggleCaseSensitive();
        if (find.matchCount !== 2) f += 4;

        // 4) Palavra inteira (+case) : so o "foo" solto -> 1
        find.toggleWholeWord();
        if (find.matchCount !== 1) f += 8;

        // 5) Navegacao CIRCULAR: 3 matches, next 3x volta ao 1o
        reset("a a a");
        find.setQuery("a");
        const first = find.current;
        find.findNext(); find.findNext(); find.findNext();
        if (find.current !== first) f += 16;
        // prev do primeiro vai pro ultimo (wrap pra tras)
        find.findPrevious();
        if (find.current !== 2) f += 32;

        // 6) REGEX DE LARGURA ZERO — o teste que mata: "a*" nao pode
        //    travar em laco infinito. Se travar, o processo nunca sai.
        reset("abc");
        find.useRegex = true;
        find.setQuery("a*");
        if (find.matchCount < 1) f += 64;   // so precisa ter TERMINADO

        // 7) Regex invalida -> estado de erro, 0 matches, sem crash
        find.setQuery("(unclosed");
        if (find.invalidRegex !== true || find.matchCount !== 0) f += 128;

        // 8) Substituir TUDO (de tras pra frente, offsets nao invalidam).
        //    Substituto MAIOR que o buscado e o caso que quebra offsets.
        reset("x x x");
        find.setQuery("x");
        find.setReplacement("LONGO");
        find.replaceAll();
        if (fakeSurface.text !== "LONGO LONGO LONGO") f += 256;

        // 9) Substituir UM: troca o atual e avanca
        reset("aa aa");
        find.setQuery("aa");
        find.setReplacement("b");
        find.replaceCurrent();
        if (fakeSurface.text !== "b aa") f += 512;

        // 10) Regex com grupo: $1 resolvido no trecho casado
        reset("foo(1) foo(2)");
        find.useRegex = true;
        find.setQuery("foo\\((\\d)\\)");
        find.setReplacement("bar[$1]");
        find.replaceAll();
        if (fakeSurface.text !== "bar[1] bar[2]") f += 1024;

        // 11) close() zera e some
        find.close();
        if (find.barVisible !== false || find.matchCount !== 0) f += 2048;
        // O codigo de saida de um processo tem 8 BITS: Qt.exit(256) sai como 0.
        // Enquanto o bitmask ia direto para o exit, todo check com bit >= 256
        // era letra morta: passava verde mesmo quebrado, que e exatamente a
        // doenca que esta suite existe para impedir. O mask agora vai para a
        // SAIDA (onde nao trunca) e o exit so diz passou/falhou.
        if (f !== 0) console.error("FALHAS bitmask=" + f);
        Qt.exit(f === 0 ? 0 : 1);
    }
}
