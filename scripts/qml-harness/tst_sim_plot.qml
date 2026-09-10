// O GRAFICO NAO PODE QUEBRAR NOS CASOS DEGENERADOS.
//
// Por que existe: a escala de um grafico e' `(v - min) / (max - min)`, e ela
// tem tres entradas que aparecem DE VERDADE e a quebram:
//
//   trilha vazia      a corrida foi recusada, e a tela desenha antes de saber
//   um ponto so'      a integracao divergiu no primeiro passo
//   curva CONSTANTE   oscilador com y(0)=0 e v(0)=0 nao sai do lugar, e
//                     `max - min` vale ZERO
//
// O terceiro e' o que dói: sem tratamento, a divisao por zero produz `NaN`, o
// `Canvas` nao desenha nada, e a tela fica em branco SEM erro — a mesma forma
// de falha da previa de 10 pixels. Nenhum outro gate ve isso: o qmllint acha o
// QML impecavel, porque ele E' impecavel.
//
// MUTACAO QUE PROVA O GATE: tire o ramo `maior - menor < 1e-15` do `extent` e a
// assercao da curva constante cai com min===max.
import QtQuick
import KineinVectis

Item {
    id: root

    width: 800
    height: 600

    SimPlot2d {
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
        if (grafico.extent("t") !== null) root.falhas += 1;
        if (grafico.extent("y") !== null) root.falhas += 2;
        // O mapeamento com limites nulos cai no meio, em vez de dar NaN.
        if (!isFinite(grafico.mapY(0, null, 100))) root.falhas += 4;
        if (!isFinite(grafico.mapX(0, null, 100))) root.falhas += 8;

        // 2. CURVA CONSTANTE: `max - min` e' zero, e a escala tem de abrir uma
        //    janela em vez de dividir por zero.
        grafico.trail = [
            { t: 0, y: 0, dy: 0 },
            { t: 1, y: 0, dy: 0 },
            { t: 2, y: 0, dy: 0 }
        ];
        const constante = grafico.extent("y");
        if (constante === null) {
            root.falhas += 16;
        } else {
            if (constante.max <= constante.min) root.falhas += 32;
            const meio = grafico.mapY(0, constante, 200);
            if (!isFinite(meio)) root.falhas += 64;
            // O valor constante fica no MEIO da janela, e nao colado na borda.
            if (!root.quase(meio, 100, 1)) root.falhas += 128;
        }

        // 3. CONSTANTE NAO-ZERO: a janela abre em volta do valor, e nao em
        //    volta do zero.
        grafico.trail = [{ t: 0, y: 5, dy: 0 }, { t: 1, y: 5, dy: 0 }];
        const cinco = grafico.extent("y");
        if (cinco === null || cinco.min >= 5 || cinco.max <= 5) root.falhas += 256;
        if (!isFinite(grafico.mapY(5, cinco, 200))) root.falhas += 512;

        // 4. UM PONTO SO': o tempo tambem colapsa.
        grafico.trail = [{ t: 0, y: 1, dy: 0 }];
        const umT = grafico.extent("t");
        if (umT === null || umT.max <= umT.min) root.falhas += 1024;
        if (!isFinite(grafico.mapX(0, umT, 300))) root.falhas += 2048;

        // 5. VALOR NAO-FINITO no meio da trilha: ele e' PULADO, e o resto ainda
        //    escala. Uma trilha com um `inf` nao pode apagar o grafico inteiro.
        grafico.trail = [
            { t: 0, y: 1, dy: 0 },
            { t: 1, y: Infinity, dy: 0 },
            { t: 2, y: 3, dy: 0 }
        ];
        const comInf = grafico.extent("y");
        if (comInf === null) {
            root.falhas += 4096;
        } else if (!isFinite(comInf.min) || !isFinite(comInf.max)) {
            root.falhas += 8192;
        } else if (comInf.max > 100) {
            // Se o `Infinity` tivesse entrado, o maximo seria ele.
            root.falhas += 16384;
        }

        // 6. TRILHA SO' DE NAO-FINITOS: nao ha' escala possivel, e a resposta e'
        //    `null` — nao um intervalo inventado.
        grafico.trail = [{ t: 0, y: NaN, dy: 0 }, { t: 1, y: Infinity, dy: 0 }];
        if (grafico.extent("y") !== null) root.falhas += 32768;

        // 7. CASO NORMAL: a escala mapeia extremos nas bordas, com a folga.
        grafico.trail = [
            { t: 0, y: 0, dy: 0 },
            { t: 1, y: 10, dy: 0 },
            { t: 2, y: -10, dy: 0 }
        ];
        const normal = grafico.extent("y");
        if (normal === null) {
            root.falhas += 65536;
        } else {
            // Com folga de 8%, o intervalo cobre alem dos extremos.
            if (normal.min > -10 || normal.max < 10) root.falhas += 131072;
            // Y CRESCE PARA BAIXO na tela: o maior valor tem o menor pixel.
            const alto = grafico.mapY(10, normal, 200);
            const baixo = grafico.mapY(-10, normal, 200);
            if (!(alto < baixo)) root.falhas += 262144;
            if (alto < 0 || baixo > 200) root.falhas += 524288;
        }

        // 8. A DERIVADA e' detectada pela trilha, e nao suposta.
        if (!grafico.temDerivada) root.falhas += 1048576;
        grafico.trail = [{ t: 0, y: 1 }, { t: 1, y: 2 }];
        if (grafico.temDerivada) root.falhas += 2097152;

        if (root.falhas !== 0) console.error("FALHAS bitmask=" + root.falhas);
        Qt.exit(root.falhas === 0 ? 0 : 1);
    }
}
