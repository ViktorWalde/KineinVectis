import QtQuick
// Carrega o SymbolMergeRules REAL (arquivo do projeto, sem copia).
import "../../ui/qml/editor"

// AS DUAS FONTES DE SIMBOLO, MESCLADAS (L1, 2026-09-26).
//
// §11.1 do roadmap 48: deduplicar por arquivo/posicao e manter a fonte visivel
// quando ela explicar divergencia. As duas fontes nao se substituem — o indice
// responde sem arquivo aberto e desde o primeiro segundo; o LSP responde depois
// e sabe mais.
Item {
    id: root

    property int failures: 0

    SymbolMergeRules {
        id: rules
    }

    function check(ok, bit, message) {
        if (!ok) {
            root.failures += bit;
            console.error(message);
        }
    }

    function sym(nome, caminho, linha) {
        return { "name": nome, "path": caminho, "line": linha, "kind": "function" };
    }

    Component.onCompleted: {
        // SO' O INDICE: e' o estado normal antes de o servidor subir, e a aba
        // nao pode ficar vazia esperando.
        const soIndice = rules.merge([sym("a", "/p/x.rs", 1)], []);
        check(soIndice.length === 1, 1, "o indice aparece sozinho");
        check(soIndice[0].source === "índice", 2, "e diz que e' do indice");
        check(rules.sourceWorthShowing(soIndice) === false, 4,
              "com uma fonte so', dizer a fonte nao explica nada");

        // SO' O LSP.
        const soLsp = rules.merge([], [sym("a", "/p/x.rs", 1)]);
        check(soLsp.length === 1 && soLsp[0].source === "lsp", 8, "o LSP sozinho");

        // MESMA POSICAO: um item so', e o do LSP vence — ele diz mais sobre o
        // mesmo simbolo.
        const mesmo = rules.merge([sym("a", "/p/x.rs", 10)], [sym("a", "/p/x.rs", 10)]);
        check(mesmo.length === 1, 16, "mesma posicao deduplica: " + mesmo.length);
        check(mesmo[0].source === "lsp", 32, "e quem fica e' o LSP");

        // MESMO NOME em arquivos diferentes NAO e' duplicata.
        const homonimos = rules.merge([sym("a", "/p/x.rs", 1)], [sym("a", "/p/y.rs", 1)]);
        check(homonimos.length === 2, 64, "mesmo nome, arquivos diferentes: dois itens");
        check(rules.sourceWorthShowing(homonimos) === true, 128,
              "com duas fontes na lista, vale dizer de onde veio");

        // MESMO ARQUIVO, linhas diferentes: dois itens.
        const duasLinhas = rules.merge([sym("a", "/p/x.rs", 1)], [sym("b", "/p/x.rs", 9)]);
        check(duasLinhas.length === 2, 256, "linhas diferentes nao deduplicam");

        // A ORDEM: o indice primeiro, porque e' o que o autor ja' estava vendo
        // quando o LSP respondeu. Reordenar a lista sob o cursor dele seria
        // pior que mostrar menos.
        const ordem = rules.merge([sym("doIndice", "/p/x.rs", 1)],
                                  [sym("doLsp", "/p/y.rs", 2)]);
        check(ordem[0].name === "doIndice" && ordem[1].name === "doLsp", 512,
              "o indice vem primeiro: " + ordem.map(s => s.name).join(","));

        // O ORIGINAL NAO E' MUTADO: marcar a fonte nao pode escrever na lista
        // que o chamador ainda tem.
        const original = [sym("a", "/p/x.rs", 1)];
        rules.merge(original, []);
        check(original[0].source === undefined, 1024,
              "a lista de entrada foi alterada");

        // Entradas vazias ou nulas nao estouram.
        check(rules.merge(null, undefined).length === 0, 2048, "nulo e' lista vazia");
        check(rules.keyFor(null) === "", 4096, "chave de nulo e' vazia");

        Qt.exit(root.failures === 0 ? 0 : 1);
    }
}
