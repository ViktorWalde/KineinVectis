// O GRAFICO DA FORMA VETORIAL NAO PODE QUEBRAR — e ele tem um caso a mais que
// o escalar.
//
// Por que existe: o `SimPlot2d` ja' tem gate para os tres casos degenerados da
// escala (trilha vazia, um ponto so', curva constante). A forma vetorial
// acrescenta um QUARTO, que so' ela pode ter:
//
//   AMOSTRA CURTA   uma corrida recusada no meio, ou um estado inicial com
//                   menos valores que componentes, deixa `values` mais curto
//                   que o indice pedido. `values[3]` de um array de 2 e'
//                   `undefined`, que vira NaN na conta e SOME sem erro
//
// E um requisito que nao e' degenerado, e' de LEITURA:
//
//   TRAJETORIA QUADRADA   escalar x e y por limites diferentes deforma a
//                         figura. Uma orbita circular viraria elipse na tela, e
//                         a pergunta que a trajetoria existe para responder —
//                         "ela fechou?" — deixaria de ter resposta visivel
//
// Nenhum outro gate ve isso: o qmllint acha o QML impecavel, porque ele E'.
//
// MUTACOES QUE PROVAM O GATE:
//   1. tire o `if (!isFinite(menor) || !isFinite(maior)) return null;` do
//      `extentOf` e a assercao do indice inexistente cai (bitmask 256): o
//      extent devolve `{min: Infinity, max: -Infinity}` em vez de `null`
//
// E DUAS mutacoes que NAO mataram, registradas porque ensinam mais que as que
// mataram (medido em 2026-09-06):
//
//   teto de comprimento     `indice < amostra.values.length` era REDUNDANTE: o
//                           acesso fora da faixa ja' chega como `undefined`.
//                           Saiu do codigo — linha que nenhuma mutacao mata nao
//                           esta' defendendo nada
//   guarda de `undefined`   tambem nao mata, porque em JS `undefined < x` e
//                           `undefined > x` sao os dois falsos. FICOU, e o
//                           motivo esta' escrito no SimPlotSystem.qml: a
//                           alternativa e' depender de comportamento implicito
//   2. troque o `Math.max` do `squareExtent` por `Math.min` e a assercao do
//      lado igual cai
//   3. tire o ramo `maior - menor < 1e-15` e a assercao da curva constante cai
import QtQuick
import KineinVectis

Item {
    id: root

    width: 800
    height: 600

    SimPlotSystem {
        id: grafico

        anchors.fill: parent
        trail: []
    }

    property int falhas: 0

    function quase(a, b, tol) {
        return Math.abs(a - b) <= tol;
    }

    Component.onCompleted: {
        // 1. TRILHA VAZIA: sem limites, sem desenho, sem quebrar.
        if (grafico.extentOf(0) !== null) root.falhas += 1;
        if (grafico.extentOf(-1) !== null) root.falhas += 2;
        if (!isFinite(grafico.mapX(0, null, 100))) root.falhas += 4;
        if (!isFinite(grafico.mapY(0, null, 100))) root.falhas += 8;

        // 2. AMOSTRA CURTA: o componente 3 nao existe em todas as amostras.
        //    O extent tem de sair dos que EXISTEM, sem NaN.
        grafico.trail = [
            { t: 0, values: [1, 2, 3, 4] },
            { t: 1, values: [2, 3] },
            { t: 2, values: [5, 6, 7, 8] }
        ];
        const curto = grafico.extentOf(3);
        if (curto === null) {
            root.falhas += 16;
        } else {
            if (!isFinite(curto.min) || !isFinite(curto.max)) root.falhas += 32;
            // So' as amostras 0 e 2 tem o indice 3: 4 e 8.
            if (!root.quase(curto.min, 4, 1e-9)) root.falhas += 64;
            if (!root.quase(curto.max, 8, 1e-9)) root.falhas += 128;
        }
        // E um indice que NAO existe em amostra nenhuma da null, em vez de NaN.
        if (grafico.extentOf(9) !== null) root.falhas += 256;

        // 3. CURVA CONSTANTE: `max - min` e' zero e a janela tem de abrir.
        grafico.trail = [
            { t: 0, values: [0, 5] },
            { t: 1, values: [0, 5] },
            { t: 2, values: [0, 5] }
        ];
        const constante = grafico.extentOf(0);
        if (constante === null) {
            root.falhas += 512;
        } else if (constante.max <= constante.min) {
            root.falhas += 1024;
        } else {
            const meio = grafico.mapY(0, constante, 200);
            if (!isFinite(meio)) root.falhas += 2048;
        }

        // 4. TRAJETORIA QUADRADA: um circulo de raio 1 tem de sair circulo.
        //    x varia de -1 a 1 e y de -1 a 1: os lados ja' sao iguais.
        //    O caso que importa e' o ASSIMETRICO — uma orbita que espiralou:
        //    x de -1 a 1 (lado 2) e y de -0,2 a 0,2 (lado 0,4). O lado comum
        //    tem de ser o MAIOR dos dois, ou a figura estica.
        const a = { min: -1, max: 1 };
        const b = { min: -0.2, max: 0.2 };
        const quadrado = grafico.squareExtent(a, b);
        if (quadrado === null) {
            root.falhas += 4096;
        } else {
            const ladoX = quadrado.x.max - quadrado.x.min;
            const ladoY = quadrado.y.max - quadrado.y.min;
            if (!root.quase(ladoX, ladoY, 1e-12)) root.falhas += 8192;
            if (!root.quase(ladoX, 2, 1e-12)) root.falhas += 16384;
            // E o centro de cada eixo tem de continuar sendo o centro do dado:
            // alargar so' de um lado deslocaria a figura.
            if (!root.quase((quadrado.y.min + quadrado.y.max) / 2, 0, 1e-12)) {
                root.falhas += 32768;
            }
        }
        // Com um limite nulo, sem quebrar.
        if (grafico.squareExtent(null, b) !== null) root.falhas += 65536;

        // 5. A COR de cada componente e' estavel e da a volta.
        if (grafico.componentColor(0) !== grafico.componentColor(6)) {
            root.falhas += 131072;
        }

        if (root.falhas !== 0) console.error("FALHAS bitmask=" + root.falhas);
        Qt.exit(root.falhas === 0 ? 0 : 1);
    }
}
