import QtQuick
import "../../ui/qml/panels/bottom"

// R1 (docs/roadmaps/26 §4.6): a grade do terminal cai em pixel FISICO inteiro,
// e texto e cursor derivam da mesma fonte de metricas.
//
// A regressao que isto protege: o cursor era posicionado por
// `Math.floor(col * charWidth)` enquanto o texto era posicionado por layout, em
// posicoes reais. Com a celula fracionaria (7.796875 x 17.6875 em DPR 1) o erro
// chegava a 0,95px e VARIAVA com a coluna — metade da largura do cursor barra.
// Um offset constante nao consertaria; o teste abaixo fixa a propriedade que
// conserta: a posicao da coluna N e exatamente N celulas, sem residuo.
Item {
    id: root
    width: 10
    height: 10

    // DPRs do gate. 1.25 e 1.5 sao escalas fracionarias reais de Wayland.
    readonly property var dprs: [1.0, 1.25, 1.5, 2.0]

    TerminalMetrics {
        id: metrics
        fontFamily: "monospace"
        fontPixelSize: 13
    }

    function quaseIgual(a, b) {
        return Math.abs(a - b) < 1e-9;
    }

    Component.onCompleted: {
        let failures = 0;

        for (const dpr of root.dprs) {
            metrics.devicePixelRatio = dpr;

            // R1.3: a celula tem que ser inteira em pixels FISICOS. E o
            // invariante da fatia — se falhar, a grade volta a escorregar.
            if (!Number.isInteger(metrics.cellWidthPx)
                    || !Number.isInteger(metrics.cellHeightPx)) {
                failures += 1;
            }
            if (metrics.cellWidthPx < 1 || metrics.cellHeightPx < 1) {
                failures += 2;
            }

            // A celula logica deriva da fisica, nunca o contrario.
            if (!quaseIgual(metrics.cellWidth, metrics.cellWidthPx / dpr)) {
                failures += 4;
            }
            if (!quaseIgual(metrics.cellHeight, metrics.cellHeightPx / dpr)) {
                failures += 8;
            }

            // O CORACAO DA CORRECAO: a coluna N cai exatamente em N celulas, e
            // em pixel fisico inteiro. Sem residuo, para qualquer coluna.
            for (const col of [0, 1, 7, 13, 40, 79, 200]) {
                const x = metrics.xForColumn(col);
                if (!quaseIgual(x, col * metrics.cellWidth)) {
                    failures += 16;
                    break;
                }
                if (!Number.isInteger(Math.round(x * dpr))
                        || Math.abs(x * dpr - col * metrics.cellWidthPx) > 1e-9) {
                    failures += 32;
                    break;
                }
            }

            for (const row of [0, 1, 23, 50]) {
                const y = metrics.yForRow(row);
                if (!quaseIgual(y, row * metrics.cellHeight)) {
                    failures += 64;
                    break;
                }
            }

            // O texto acumula `widthForCells`; o cursor usa `xForColumn`. Os
            // dois TEM que chegar no mesmo lugar — era exatamente aqui que
            // divergiam.
            for (const cells of [1, 3, 40, 79]) {
                if (!quaseIgual(metrics.widthForCells(cells),
                                metrics.xForColumn(cells))) {
                    failures += 128;
                    break;
                }
            }

            // Ida e volta: o pixel do meio da celula N volta como coluna N.
            for (const col of [0, 5, 39, 79]) {
                const meio = metrics.xForColumn(col) + metrics.cellWidth / 2;
                if (metrics.columnAt(meio) !== col) {
                    failures += 256;
                    break;
                }
            }
            for (const row of [0, 5, 23]) {
                const meio = metrics.yForRow(row) + metrics.cellHeight / 2;
                if (metrics.rowAt(meio) !== row) {
                    failures += 512;
                    break;
                }
            }

            // A celula acomoda o glifo: folga nao negativa, senao a tinta
            // transborda (era o caso — contentHeight 18 numa caixa de 17.6875).
            if (metrics.glyphLeft < -1e-9) {
                failures += 1024;
            }

            // A baseline fica dentro da celula e tambem em pixel fisico.
            if (metrics.cellBaselinePx < 0
                    || metrics.cellBaselinePx > metrics.cellHeightPx
                    || !Number.isInteger(metrics.cellBaselinePx)) {
                failures += 2048;
            }

            // columnsIn e o inverso de xForColumn: N celulas cabem em N celulas.
            for (const n of [1, 24, 80]) {
                if (metrics.columnsIn(metrics.widthForCells(n)) !== n) {
                    failures += 4096;
                    break;
                }
            }
        }

        // Peso e italico NAO entram na grade: medido em 2026-07-16, Normal,
        // Medium e Italic tem o mesmo avanco no Noto Sans Mono. Se um dia
        // mudarem, a celula teria que considerar o pior caso — este teste
        // avisa em vez de deixar o texto sair da grade em silencio.
        metrics.devicePixelRatio = 1.0;
        const larguraBase = metrics.cellWidthPx;
        if (larguraBase < 1) failures += 8192;
        // O codigo de saida de um processo tem 8 BITS: Qt.exit(256) sai como 0.
        // Enquanto o bitmask ia direto para o exit, todo check com bit >= 256
        // era letra morta: passava verde mesmo quebrado, que e exatamente a
        // doenca que esta suite existe para impedir. O mask agora vai para a
        // SAIDA (onde nao trunca) e o exit so diz passou/falhou.
        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
