import QtQuick

Item {
    id: root

    property var surfaceBridge: null
    readonly property string editorIndent: "    "

    // E1: o fallback local ja' esta' na tela quando isto dispara; o sinal
    // PERGUNTA a' gramatica se ela discorda. Nada sincrono no caminho da tecla.
    signal indentRequested(int line, int column, string trigger)


    visible: false

    EditorLineOperations {
        id: lineOps

        surfaceBridge: root.surfaceBridge
        geometry: geo
        editorIndent: root.editorIndent
    }

    // As TRES TRAVAS da resposta da gramatica; ver EditorIndentCorrection.qml.
    EditorIndentCorrection {
        id: correction

        onApproved: function(lineStart, appliedIndent, level) {
            root.applyIndentCorrection(lineStart, appliedIndent, level);
        }
    }

    // A REGRA da indentacao local; ver EditorIndentRules.qml. Ela e' o caminho
    // NORMAL, e nao o plano B: roda sempre, na hora da tecla.
    EditorIndentRules {
        id: indentRules

        unit: root.editorIndent
    }

    // As PERGUNTAS sobre o texto; ver EditorTextGeometry.qml.
    EditorTextGeometry {
        id: geo

        surfaceBridge: root.surfaceBridge
    }

    // A escada expandir/encolher selecao, com memoria propria;
    // ver EditorSelectionLadder.qml.
    EditorSelectionLadder {
        id: ladder

        surfaceBridge: root.surfaceBridge
        geometry: geo
    }

    function ready() {
        return surfaceBridge !== null && surfaceBridge.ready();
    }

    function editorText() {
        return geo.text();
    }

    function cursorLineColumn() {
        return geo.cursorLineColumn();
    }

    function wordStartAt(position) {
        return geo.wordStartAt(position);
    }

    function currentWord() {
        return geo.currentWord();
    }

    function indentSelection() {
        if (!ready()) {
            return;
        }
        const surface = surfaceBridge.editorSurface;
        const starts = geo.selectedLineStarts();
        const hadSelection = surface.selectionStart !== surface.selectionEnd;
        const selectionStart = Math.min(surface.selectionStart, surface.selectionEnd);
        const selectionEnd = Math.max(surface.selectionStart, surface.selectionEnd);
        const cursor = surface.cursorPosition;
        for (let i = starts.length - 1; i >= 0; i--) {
            surface.insert(starts[i], editorIndent);
        }
        if (hadSelection) {
            surface.select(selectionStart + editorIndent.length,
                           selectionEnd + starts.length * editorIndent.length);
        } else {
            surface.cursorPosition = cursor + editorIndent.length;
        }
    }

    function unindentSelection() {
        if (!ready()) {
            return;
        }
        const surface = surfaceBridge.editorSurface;
        const starts = geo.selectedLineStarts();
        const hadSelection = surface.selectionStart !== surface.selectionEnd;
        const selectionStart = Math.min(surface.selectionStart, surface.selectionEnd);
        const selectionEnd = Math.max(surface.selectionStart, surface.selectionEnd);
        const cursor = surface.cursorPosition;
        let removedBeforeStart = 0;
        let removedBeforeEnd = 0;
        let removedBeforeCursor = 0;
        for (let i = starts.length - 1; i >= 0; i--) {
            const start = starts[i];
            let removeCount = 0;
            if (surface.text.charAt(start) === "\t") {
                removeCount = 1;
            } else {
                while (removeCount < editorIndent.length
                       && surface.text.charAt(start + removeCount) === " ") {
                    removeCount++;
                }
            }
            if (removeCount === 0) {
                continue;
            }
            surface.remove(start, start + removeCount);
            if (start < selectionStart) {
                removedBeforeStart += removeCount;
            }
            if (start < selectionEnd) {
                removedBeforeEnd += removeCount;
            }
            if (start < cursor) {
                removedBeforeCursor += removeCount;
            }
        }
        if (hadSelection) {
            surface.select(Math.max(0, selectionStart - removedBeforeStart),
                           Math.max(0, selectionEnd - removedBeforeEnd));
        } else {
            surface.cursorPosition = Math.max(geo.lineStartAt(cursor),
                                              cursor - removedBeforeCursor);
        }
    }

    // A REGRA saiu para o EditorIndentRules (2026-09-26); aqui ficou o GESTO.
    // A correcao por gramatica vem depois, pelo `syntaxTree.indent`.
    function insertNewline() {
        if (!ready()) {
            return;
        }
        const surface = surfaceBridge.editorSurface;
        const cursor = surface.cursorPosition;
        const plan = indentRules.forNewline(surface.text, cursor);
        const closer = plan.closerIndent === "" ? "" : "\n" + plan.closerIndent;
        surface.insert(cursor, "\n" + plan.insert + closer);
        surface.cursorPosition = cursor + 1 + plan.insert.length;
        root.askGrammarForLine(cursor + 1, plan.insert, "newline");
    }

    // E3: "}" digitado com só whitespace antes do cursor desce para a
    // indentação da linha do "{" casado (varredura reversa por
    // profundidade; strings/comentários fora, limitação registrada).
    // Sem par casado ou indentação já certa, insere cru.
    function insertCloserBrace() {
        if (!ready()) {
            return;
        }
        const surface = surfaceBridge.editorSurface;
        const cursor = surface.cursorPosition;
        const text = editorText();
        const lineStart = geo.lineStartAt(cursor);
        const beforeCursor = text.substring(lineStart, cursor);
        if (/^[ \t]*$/.test(beforeCursor)) {
            let depth = 1;
            let opener = -1;
            for (let i = cursor - 1; i >= 0; i--) {
                const character = text.charAt(i);
                if (character === "}") {
                    depth++;
                } else if (character === "{") {
                    depth--;
                    if (depth === 0) {
                        opener = i;
                        break;
                    }
                }
            }
            if (opener >= 0) {
                const openIndent = geo.lineIndentAt(geo.lineStartAt(opener));
                if (openIndent !== beforeCursor) {
                    surface.remove(lineStart, cursor);
                    surface.insert(lineStart, openIndent + "}");
                    surface.cursorPosition = lineStart + openIndent.length + 1;
                    root.askGrammarForLine(lineStart, openIndent, "closeDelimiter");
                    return;
                }
            }
            root.askGrammarForLine(lineStart, beforeCursor, "closeDelimiter");
        }
        surface.insert(cursor, "}");
        surface.cursorPosition = cursor + 1;
    }

    // O PEDIDO a' gramatica, guardando o que o fallback aplicou. Quem sabe o
    // caminho e a versao e' o EditorController, que carimba com
    // `noteIndentRequest` no mesmo gesto.
    function askGrammarForLine(lineStart, appliedIndent, trigger) {
        correction.remember(lineStart, appliedIndent);
        // A coluna do protocolo e' ZERO-based em unidades UTF-16; a geometria
        // devolve 1-based.
        const position = geo.cursorLineColumn();
        root.indentRequested(position.line, position.column - 1, trigger);
    }

    function noteIndentRequest(path, version) {
        correction.stamp(path, version);
    }

    function handleIndentAnswer(path, version, level) {
        return correction.accept(path, version, level);
    }

    // A TERCEIRA TRAVA: o texto entre o inicio da linha e o cursor ainda tem de
    // ser EXATAMENTE o que o fallback pos. Se o autor digitou no meio, a
    // resposta chegou tarde e nao vale mais.
    function applyIndentCorrection(lineStart, appliedIndent, level) {
        if (!ready() || level < 0) {
            return false;
        }
        const surface = surfaceBridge.editorSurface;
        const wanted = indentRules.indentFor(level);
        if (wanted === appliedIndent) {
            return false;
        }
        const atual = surface.text.substring(lineStart, lineStart + appliedIndent.length);
        if (atual !== appliedIndent) {
            return false;
        }
        const cursor = surface.cursorPosition;
        surface.remove(lineStart, lineStart + appliedIndent.length);
        surface.insert(lineStart, wanted);
        surface.cursorPosition = cursor - appliedIndent.length + wanted.length;
        return true;
    }

    // E3: Home alterna primeiro caractere de texto ↔ coluna 0;
    // com extendSelection a âncora oposta ao cursor é preservada.
    function smartHome(extendSelection) {
        if (!ready()) {
            return;
        }
        const surface = surfaceBridge.editorSurface;
        const cursor = surface.cursorPosition;
        const lineStart = geo.lineStartAt(cursor);
        const firstText = lineStart + geo.lineIndentAt(lineStart).length;
        const target = cursor === firstText ? lineStart : firstText;
        if (extendSelection) {
            const anchor = surface.selectionStart === surface.selectionEnd
                    ? cursor
                    : (cursor === surface.selectionEnd
                       ? surface.selectionStart : surface.selectionEnd);
            surface.select(anchor, target);
        } else {
            surface.cursorPosition = target;
        }
    }

    function expandSelection() {
        ladder.expand();
    }

    function shrinkSelection() {
        ladder.shrink();
    }

    // As operacoes sobre LINHAS INTEIRAS tem dono proprio; ver
    // EditorLineOperations.qml.
    readonly property alias lines: lineOps

    function goToLine(line, column) {
        if (!ready()) {
            return;
        }
        const surface = surfaceBridge.editorSurface;
        const text = surface.text;
        let target = 0;
        let current = 1;
        while (current < line) {
            const nextBreak = text.indexOf("\n", target);
            if (nextBreak < 0) {
                break;
            }
            target = nextBreak + 1;
            current++;
        }
        const end = geo.lineEndAt(target);
        surface.cursorPosition = Math.min(target + Math.max(1, column) - 1, end);
    }
}
