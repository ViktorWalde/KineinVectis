import QtQuick

// R1 (DocsPublic/roadmaps/26): fonte UNICA de metricas de celula do terminal.
//
// Por que existe. Ate 0.60 o texto e o cursor eram posicionados por sistemas
// independentes: o texto por layout (Column/Row, posicoes reais) e o cursor por
// aritmetica explicita (`Math.floor(col * charWidth)`). Com a celula
// fracionaria — medido em 2026-07-16: charWidth 7.796875, lineHeight 17.6875,
// DPR 1 — o `floor` descartava ate 0,95px, e o erro VARIAVA com a coluna
// (0,36 a 0,95). O cursor barra tem 2px, entao isso era metade dele. Ver a
// tabela em DocsPublic/roadmaps/26 §4.6.
//
// A correcao (R1.3): arredondar a grade em pixels FISICOS primeiro e derivar os
// logicos. Com a celula em pixel fisico inteiro, `col * cellWidth` e exato dos
// dois lados e nao ha o que arredondar depois. Um offset constante nao
// resolveria: o erro nao era constante.
//
// Sem regra de negocio (R1.1): isto so mede. Nao conhece IPC, sessao, modo VT
// nem o contrato — pixels nao entram no protocolo (R1.5).
Item {
    id: root

    // --- entradas ---
    property string fontFamily: "monospace"
    property int fontPixelSize: 13
    // Injetavel de proposito: o harness exercita DPR 1, 1.25, 1.5 e 2 sem tela.
    property real devicePixelRatio: 1.0

    visible: false

    TextMetrics {
        id: advanceProbe

        font.family: root.fontFamily
        font.pixelSize: root.fontPixelSize
        font.preferShaping: false
        // "M" e a referencia classica de avanco monoespacado. Medir sem peso:
        // Noto Sans Mono e variavel, e a medicao confirmou que Normal, Medium e
        // Italic tem o MESMO avanco (7.796875) — peso nao entra na grade.
        text: "M"
    }

    FontMetrics {
        id: face

        font.family: root.fontFamily
        font.pixelSize: root.fontPixelSize
    }

    // --- medida do GLIFO: o que a fonte diz, sem arredondar ---
    readonly property real glyphAdvance: advanceProbe.advanceWidth
    readonly property real ascent: face.ascent
    readonly property real descent: face.descent
    readonly property real leading: face.leading
    readonly property real glyphHeight: face.height

    // --- medida da CELULA: a grade que a gente desenha ---
    readonly property real dpr: devicePixelRatio > 0 ? devicePixelRatio : 1.0

    // Arredondamento ao pixel fisico MAIS PROXIMO, nao para baixo.
    // `floor` comprimiria a celula: 7.796875 → 7 perde 0,8px por coluna, ~64px
    // ao longo de 80 colunas. `round` → 8 expande 0,2px por coluna, ~16px. O
    // valor exato importa menos que ser deterministico e unico; o que nao pode
    // e cada consumidor arredondar do seu jeito.
    readonly property int cellWidthPx: Math.max(1, Math.round(glyphAdvance * dpr))
    readonly property int cellHeightPx: Math.max(1, Math.round(glyphHeight * dpr))
    readonly property real cellWidth: cellWidthPx / dpr
    readonly property real cellHeight: cellHeightPx / dpr

    // --- posicao do glifo DENTRO da celula ---
    // A celula e >= o glifo (arredondamos para cima na pratica), entao sobra
    // folga. Centralizar a folga e o que um terminal faz; R2 troca o
    // AlignVCenter implicito do Text por esta baseline explicita.
    readonly property real glyphLeft: (cellWidth - glyphAdvance) / 2
    readonly property int cellBaselinePx: Math.round(
            (cellHeightPx - (ascent + descent) * dpr) / 2 + ascent * dpr)
    readonly property real cellBaseline: cellBaselinePx / dpr
    readonly property real glyphTop: cellBaseline - ascent

    // --- conversoes de grade: ninguem mais faz essa conta (R1.4) ---
    // Exatas por construcao: cellWidthPx e inteiro, entao col * cellWidthPx cai
    // sempre no mesmo pixel fisico. O texto acumula `cells * cellWidth` e chega
    // no mesmo lugar — que e o ponto da fatia.
    function xForColumn(col) {
        return (col * cellWidthPx) / dpr;
    }

    function yForRow(row) {
        return (row * cellHeightPx) / dpr;
    }

    function widthForCells(cells) {
        return (cells * cellWidthPx) / dpr;
    }

    function columnAt(x) {
        return Math.max(0, Math.floor(x / cellWidth));
    }

    function rowAt(y) {
        return Math.max(0, Math.floor(y / cellHeight));
    }

    // Quantas celulas inteiras cabem — usado pelo resize para pedir o grid ao
    // core. Piso: uma celula parcial nao e uma coluna.
    function columnsIn(width) {
        return Math.max(0, Math.floor(width / cellWidth));
    }

    function rowsIn(height) {
        return Math.max(0, Math.floor(height / cellHeight));
    }
}
