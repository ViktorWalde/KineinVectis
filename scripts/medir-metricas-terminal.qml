import QtQuick

// R0 item 6 (docs/roadmaps/26): tabela numerica das metricas de celula.
// Nao desenha terminal, nao le sessao, nao loga texto do usuario. So mede a
// fonte que o Theme pede ("monospace", 13) do jeito que o TerminalViewport a usa.
Item {
    width: 10
    height: 10

    // Igual ao TerminalPanel: mede "M" sem peso explicito.
    TextMetrics {
        id: base
        font.family: "monospace"
        font.pixelSize: 13
        font.preferShaping: false
        text: "M"
    }

    // Igual ao span quando o ANSI pede bold.
    TextMetrics {
        id: medium
        font.family: "monospace"
        font.pixelSize: 13
        font.weight: Font.Medium
        font.preferShaping: false
        text: "M"
    }

    TextMetrics {
        id: italic
        font.family: "monospace"
        font.pixelSize: 13
        font.italic: true
        font.preferShaping: false
        text: "M"
    }

    FontMetrics {
        id: fm
        font.family: "monospace"
        font.pixelSize: 13
    }

    FontMetrics {
        id: fmMedium
        font.family: "monospace"
        font.pixelSize: 13
        font.weight: Font.Medium
    }

    // Um Text real, montado como o TerminalViewport monta o span.
    Text {
        id: spanLike
        text: "M"
        font.family: "monospace"
        font.pixelSize: 13
        font.preferShaping: false
        verticalAlignment: Text.AlignVCenter
        height: base.height
    }

    Text {
        id: spanLikeBold
        text: "M"
        font.family: "monospace"
        font.pixelSize: 13
        font.weight: Font.Medium
        font.preferShaping: false
        verticalAlignment: Text.AlignVCenter
        height: base.height
    }

    function row(label, value) {
        console.log("  " + label.padEnd(34) + String(value));
    }

    Component.onCompleted: {
        console.log("=== R0 — metricas de celula (Theme.monoFont=monospace, 13px) ===");
        console.log("");
        console.log("[celula que POSICIONA o cursor: TextMetrics sem peso]");
        row("charWidth (advanceWidth 'M')", base.advanceWidth);
        row("lineHeight (height)", base.height);
        row("tightBoundingRect.height", base.tightBoundingRect.height);
        row("boundingRect.height", base.boundingRect.height);
        console.log("");
        console.log("[metricas da fonte]");
        row("ascent", fm.ascent);
        row("descent", fm.descent);
        row("leading", fm.leading);
        row("height (ascent+descent+leading)", fm.height);
        row("ascent+descent", fm.ascent + fm.descent);
        console.log("");
        console.log("[avanco por peso — o span usa Medium quando ANSI bold]");
        row("advanceWidth Normal", base.advanceWidth);
        row("advanceWidth Medium", medium.advanceWidth);
        row("DIVERGE?", base.advanceWidth !== medium.advanceWidth ? "SIM <<<<" : "nao");
        row("height Normal", base.height);
        row("height Medium", medium.height);
        row("height DIVERGE?", base.height !== medium.height ? "SIM <<<<" : "nao");
        row("ascent Normal / Medium",
            fm.ascent + " / " + fmMedium.ascent);
        console.log("");
        row("advanceWidth Italic", italic.advanceWidth);
        row("italic DIVERGE?",
            base.advanceWidth !== italic.advanceWidth ? "SIM <<<<" : "nao");
        console.log("");
        console.log("[Text real vs caixa da celula — AlignVCenter]");
        row("Text.contentHeight", spanLike.contentHeight);
        row("Text.contentWidth", spanLike.contentWidth);
        row("caixa (lineHeight)", base.height);
        row("folga vertical (caixa-conteudo)", base.height - spanLike.contentHeight);
        row("deslocamento do AlignVCenter",
            (base.height - spanLike.contentHeight) / 2);
        row("bold contentWidth", spanLikeBold.contentWidth);
        console.log("");
        console.log("[arredondamento — cursor usa Math.floor, span usa real]");
        for (var col of [1, 7, 13, 40, 79]) {
            var real = col * base.advanceWidth;
            var floored = Math.floor(real);
            row("col " + col + ": real / floor / erro",
                real.toFixed(3) + " / " + floored + " / "
                + (real - floored).toFixed(3));
        }
        console.log("");
        console.log("[DPR]");
        row("Screen.devicePixelRatio", Screen.devicePixelRatio);
        row("Screen.pixelDensity", Screen.pixelDensity);
        row("charWidth em px fisicos",
            base.advanceWidth * Screen.devicePixelRatio);
        row("lineHeight em px fisicos",
            base.height * Screen.devicePixelRatio);
        row("charWidth fisico e inteiro?",
            Number.isInteger(base.advanceWidth * Screen.devicePixelRatio)
                ? "sim" : "NAO <<<<");
        row("lineHeight fisico e inteiro?",
            Number.isInteger(base.height * Screen.devicePixelRatio)
                ? "sim" : "NAO <<<<");

        Qt.exit(0);
    }
}
