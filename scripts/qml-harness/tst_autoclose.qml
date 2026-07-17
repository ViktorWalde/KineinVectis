import QtQuick
// Carrega o EditorAutoClosePairs.qml REAL (arquivo do projeto, sem copia).
import "../../ui/qml/editor"

// O QUE ESTA SUITE MEDE. Nao "o que o nosso codigo faz hoje" — isso so
// congelaria o comportamento atual, erros inclusos. Ela codifica o que Zed e
// VS Code CONCORDAM que deve acontecer ao digitar um par, porque sao os dois
// que ja pagaram para descobrir as bordas. Onde batemos, teste verde; onde
// divergimos, a divergencia esta marcada e DATADA abaixo, nao escondida.
//
// Referencia de COMPORTAMENTO, nunca de codigo: Zed e' GPL-3.0 e este projeto
// e' MIT/Apache-2.0 (mesma regra que o ContextoIA ja aplica ao Serial Studio).
Item {
    id: root
    width: 100; height: 100

    property int braceRequests: 0

    // TextEdit REAL: o auto-close mexe em documento de verdade (insert/remove/
    // select/cursor), e um falso so provaria que os fakes concordam entre si.
    TextEdit {
        id: editor
        visible: false
    }

    // Fake do AutoCloseRegions (C++): o rastreio REAL — QTextCursor
    // acompanhando edicoes — e' coberto por ui/tests/tst_auto_close_regions.cpp;
    // aqui interessa a POLITICA (quando o pares consulta/anota/consome), e um
    // fake com a mesma superficie basta. Padrao do tst_completion.
    property QtObject fakeRegions: QtObject {
        property var marks: ({})
        function notePairInserted(p) { marks[p] = true; }
        function isAutoClosedAt(p) { return marks[p] === true; }
        function consumeAt(p) { delete marks[p]; }
    }

    EditorAutoClosePairs {
        id: pares
        target: editor
        regions: root.fakeRegions
        onCloserBraceRequested: root.braceRequests += 1
    }

    // Um evento de tecla e' lido por .text e .modifiers; o objeto simples basta.
    function tecla(texto, modificadores) {
        return { text: texto, modifiers: modificadores === undefined ? 0 : modificadores };
    }

    function estado(texto, posicao) {
        editor.text = texto;
        editor.cursorPosition = posicao;
        editor.select(posicao, posicao);
        fakeRegions.marks = ({});
    }

    Component.onCompleted: {
        let falhas = 0;

        // 1) Auto-close basico: o abridor traz o fechador e o cursor fica DENTRO.
        // E o ciclo completo (E6): o fechador foi ANOTADO, entao digitar `)` em
        // seguida faz type-over — pula, nao duplica — e CONSOME a anotacao.
        estado("", 0);
        if (!pares.handleTypingKey(tecla("("))) falhas += 1;
        if (editor.text !== "()") falhas += 2;
        if (editor.cursorPosition !== 1) falhas += 4;
        if (!pares.handleTypingKey(tecla(")"))) falhas += 67108864;
        if (editor.text !== "()" || editor.cursorPosition !== 2) falhas += 134217728;
        if (root.fakeRegions.isAutoClosedAt(1)) falhas += 268435456; // consumido

        // 2) Surround: com selecao ativa, o abridor ENVOLVE em vez de
        // substituir, e a selecao continua sobre o mesmo texto (autoSurround
        // no VS Code; mesmo comportamento no Zed).
        editor.text = "alvo";
        editor.select(0, 4);
        if (!pares.handleTypingKey(tecla("("))) falhas += 8;
        if (editor.text !== "(alvo)") falhas += 16;
        if (editor.selectionStart !== 1 || editor.selectionEnd !== 5) falhas += 32;

        // 3) Backspace no par VAZIO apaga os dois lados. Fora do par vazio, nao
        // se mete: apagar so o abridor deixaria o fechador orfao.
        estado("()", 1);
        if (!pares.handlePairBackspace()) falhas += 64;
        if (editor.text !== "") falhas += 128;
        estado("(x)", 2);
        if (pares.handlePairBackspace()) falhas += 256;

        // 4) Aspas coladas em palavra NAO duplicam: "don't" nao pode virar
        // "don''t". E' o caso que mais irrita e os dois editores o tratam.
        estado("don", 3);
        if (pares.handleTypingKey(tecla("'"))) falhas += 512;
        estado("dont", 3);   // cursor antes do "t": proximo char e' palavra
        if (pares.handleTypingKey(tecla("'"))) falhas += 1024;

        // 5) Abridor colado ANTES de palavra nao auto-fecha: quem escreve "(" na
        // frente de "foo" quer chamar/agrupar, nao criar "()foo".
        estado("foo", 0);
        if (pares.handleTypingKey(tecla("("))) falhas += 2048;

        // 6) Ctrl e' atalho, nao digitacao. Mas Ctrl+Alt e' AltGr em teclado
        // europeu e produz caractere legitimo — tem que passar.
        estado("", 0);
        if (pares.handleTypingKey(tecla("(", Qt.ControlModifier))) falhas += 4096;
        estado("", 0);
        if (!pares.handleTypingKey(tecla("(", Qt.ControlModifier | Qt.AltModifier))) falhas += 8192;

        // 7) Desligado e' desligado (setting autoClosePairs).
        pares.enabled = false;
        estado("", 0);
        if (pares.handleTypingKey(tecla("("))) falhas += 16384;
        if (pares.handlePairBackspace()) falhas += 32768;
        pares.enabled = true;

        // 8) "<" so fecha em contexto seguro. Generico e' comparacao, template
        // ou shift — auto-fechar ali seria pior que nao fazer nada.
        estado("#include ", 9);
        if (!pares.handleTypingKey(tecla("<"))) falhas += 65536;
        if (editor.text !== "#include <>") falhas += 131072;
        estado("a ", 2);
        if (pares.handleTypingKey(tecla("<"))) falhas += 262144;

        // 9) "}" delega: a insercao com dedent vive no EditorTextController.
        root.braceRequests = 0;
        estado("", 0);
        if (!pares.handleTypingKey(tecla("}"))) falhas += 524288;
        if (root.braceRequests !== 1) falhas += 1048576;

        // 10) DIVERGENCIA RESOLVIDA EM 2026-07-17 (E6). Fechador escrito a MAO
        // nao e' nosso: o handler NAO consome, e o TextEdit insere o `)` que o
        // usuario digitou — em vez de engoli-lo, como fazia o type-over cego.
        // (VS Code autoClosingOvertype "auto": so pula o auto-inserido.)
        estado("foo(bar)", 7);       // cursor antes do ")" escrito a mao
        if (pares.handleTypingKey(tecla(")"))) falhas += 2097152;
        if (editor.text !== "foo(bar)") falhas += 4194304;
        if (editor.cursorPosition !== 7) falhas += 8388608;

        // 10b) Sem rastreador ligado (regions null), type-over NENHUM: duplicar
        // e' visivel e corrigivel, engolir tecla e' silencioso.
        pares.regions = null;
        estado("()", 1);
        if (pares.handleTypingKey(tecla(")"))) falhas += 536870912;
        pares.regions = root.fakeRegions;

        // 10c) Aspa antes de aspa escrita a mao nao duplica nem pula: insercao
        // simples (o type-over cego mascarava este caso; Code OSS nao
        // auto-fecha aspa antes de aspa).
        estado("\"", 0);
        if (pares.handleTypingKey(tecla("\""))) falhas += 1073741824;

        // 10d) O `>` do `#include <>` tambem e' rastreado: digitar `>` sobre o
        // fechador auto-inserido pula em vez de duplicar.
        estado("#include ", 9);
        pares.handleTypingKey(tecla("<"));           // vira "#include <>", anota 10
        if (!pares.handleTypingKey(tecla(">"))) falhas += 16384 * 131072; // 2^31
        if (editor.text !== "#include <>" || editor.cursorPosition !== 11) {
            falhas += 16384 * 262144; // 2^32
        }

        // 11) Barra invertida escapa a aspa: NAO faz type-over MESMO quando a
        // aspa no cursor foi auto-inserida (o fake a marca de proposito). Em
        // `x = "a\` + cursor + `"`, digitar `"` insere uma aspa escapada —
        // invariante do Code OSS, que poe a regra da barra ACIMA da origem.
        estado('x = "a\\"', 7);      // anterior = \  proximo = "
        root.fakeRegions.marks[7] = true;
        if (pares.handleTypingKey(tecla('"'))) falhas += 16777216;
        if (editor.cursorPosition !== 7) falhas += 33554432;   // nao pode ter pulado

        // O codigo de saida de um processo tem 8 BITS: Qt.exit(256) sai como 0.
        // O mask vai para a SAIDA (onde nao trunca) e o exit so diz passou/falhou.
        if (falhas !== 0) console.error("FALHAS bitmask=" + falhas);
        Qt.exit(falhas === 0 ? 0 : 1);
    }
}
