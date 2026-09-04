import QtQuick

// PERGUNTAS sobre o texto — nenhuma delas o modifica.
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-03). O EditorTextController tinha 573
// linhas e misturava duas coisas de natureza oposta: PERGUNTAR onde comeca a
// linha, onde termina a palavra, qual a indentacao — e MODIFICAR o texto
// (indentar, duplicar, comentar, mover). O corte separa consulta de comando; e'
// por responsabilidade, nao por tamanho (ARCHITECTURE.md §4 regra 9).
//
// Ter as consultas num dono so' importa porque elas sao usadas por TRES lados —
// o proprio controller de texto, a escada de selecao (EditorSelectionLadder) e
// a completacao — e uma copia divergente aqui produz cursor no lugar errado,
// que e' o tipo de bug que ninguem reporta e todo mundo sente.
//
// TUDO E' OFFSET EM CARACTERES (UTF-16, como o TextEdit conta), menos
// `cursorLineColumn`, que devolve linha/coluna 1-BASED porque e' o que a UI
// mostra ao autor.
Item {
    id: root

    visible: false

    property var surfaceBridge: null

    // Regras puras de texto — uma unica DEFINICAO, ver TextRules.qml.
    readonly property TextRules rules: TextRules {}

    function ready() {
        return surfaceBridge !== null && surfaceBridge.ready();
    }

    function text() {
        return ready() ? surfaceBridge.text() : "";
    }

    function cursorLineColumn() {
        const cursor = ready() ? surfaceBridge.editorSurface.cursorPosition : 0;
        const content = text();
        let line = 1;
        let lineStart = 0;
        let offset = 0;
        while (offset < cursor) {
            const next = content.indexOf("\n", offset);
            if (next < 0 || next >= cursor) {
                break;
            }
            line++;
            lineStart = next + 1;
            offset = next + 1;
        }
        return {
            line: line,
            column: cursor - lineStart + 1
        };
    }

    function wordStartAt(position) {
        const content = text();
        let start = position;
        while (start > 0 && root.rules.isWordChar(content.charAt(start - 1))) {
            start--;
        }
        return start;
    }

    function wordEndAt(position) {
        const content = text();
        let end = position;
        while (end < content.length && root.rules.isWordChar(content.charAt(end))) {
            end++;
        }
        return end;
    }

    function currentWord() {
        if (!ready()) {
            return "";
        }
        const cursor = surfaceBridge.editorSurface.cursorPosition;
        return text().substring(wordStartAt(cursor), wordEndAt(cursor));
    }

    function lineStartAt(position) {
        // lastIndexOf com fromIndex 0 ainda olha o índice 0; sem a
        // guarda, posição 0 com "\n" inicial devolvia linha errada.
        const previousBreak = position <= 0
                ? -1 : text().lastIndexOf("\n", position - 1);
        return previousBreak < 0 ? 0 : previousBreak + 1;
    }

    function lineEndAt(position) {
        const content = text();
        const nextBreak = content.indexOf("\n", position);
        return nextBreak < 0 ? content.length : nextBreak;
    }

    function lineIndentAt(lineStart) {
        const content = text();
        let end = lineStart;
        while (end < content.length) {
            const ch = content.charAt(end);
            if (ch !== " " && ch !== "\t") {
                break;
            }
            end++;
        }
        return content.substring(lineStart, end);
    }

    // Inicio de cada linha TOCADA pela selecao. Com selecao vazia e' so' a
    // linha do cursor. O `- 1` no fim e' deliberado: uma selecao que termina
    // na coluna 0 da linha seguinte NAO inclui essa linha, senao selecionar
    // tres linhas com o mouse indentaria quatro.
    function selectedLineStarts() {
        if (!ready()) {
            return [];
        }
        const surface = surfaceBridge.editorSurface;
        const selectionStart = Math.min(surface.selectionStart,
                                        surface.selectionEnd);
        const selectionEnd = Math.max(surface.selectionStart,
                                      surface.selectionEnd);
        const effectiveEnd = selectionEnd > selectionStart
                ? Math.max(selectionStart, selectionEnd - 1)
                : surface.cursorPosition;
        const starts = [];
        let lineStart = lineStartAt(selectionStart);
        const lastLineStart = lineStartAt(effectiveEnd);
        while (lineStart <= lastLineStart) {
            starts.push(lineStart);
            const lineEnd = lineEndAt(lineStart);
            if (lineEnd >= text().length) {
                break;
            }
            lineStart = lineEnd + 1;
        }
        return starts;
    }
}
