import QtQuick

Item {
    id: root

    property var surfaceBridge: null
    readonly property string editorIndent: "    "
    // E3: memória do expand/shrink selection. O histórico só vale
    // enquanto a seleção atual for a última expansão sobre o mesmo
    // texto (guarda barata; ver docs-privada/diario/18, fatia E3).
    property var expandHistory: []
    property var lastExpansion: null

    visible: false

    function ready() {
        return surfaceBridge !== null && surfaceBridge.ready();
    }

    function editorText() {
        return ready() ? surfaceBridge.text() : "";
    }

    function cursorLineColumn() {
        const cursor = ready() ? surfaceBridge.editorSurface.cursorPosition : 0;
        const text = editorText();
        let line = 1;
        let lineStart = 0;
        let offset = 0;
        while (offset < cursor) {
            const next = text.indexOf("\n", offset);
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

    function isWordChar(ch) {
        return (ch >= "a" && ch <= "z") || (ch >= "A" && ch <= "Z")
                || (ch >= "0" && ch <= "9") || ch === "_";
    }

    function wordStartAt(position) {
        const text = editorText();
        let start = position;
        while (start > 0 && isWordChar(text.charAt(start - 1))) {
            start--;
        }
        return start;
    }

    function currentWord() {
        if (!ready()) {
            return "";
        }
        const text = surfaceBridge.text();
        const start = wordStartAt(surfaceBridge.editorSurface.cursorPosition);
        let end = surfaceBridge.editorSurface.cursorPosition;
        while (end < text.length && isWordChar(text.charAt(end))) {
            end++;
        }
        return text.substring(start, end);
    }

    function lineStartAt(position) {
        // lastIndexOf com fromIndex 0 ainda olha o índice 0; sem a
        // guarda, posição 0 com "\n" inicial devolvia linha errada.
        const previousBreak = position <= 0
                ? -1 : editorText().lastIndexOf("\n", position - 1);
        return previousBreak < 0 ? 0 : previousBreak + 1;
    }

    function lineEndAt(position) {
        const text = editorText();
        const nextBreak = text.indexOf("\n", position);
        return nextBreak < 0 ? text.length : nextBreak;
    }

    function lineIndentAt(lineStart) {
        const text = editorText();
        let end = lineStart;
        while (end < text.length) {
            const ch = text.charAt(end);
            if (ch !== " " && ch !== "\t") {
                break;
            }
            end++;
        }
        return text.substring(lineStart, end);
    }

    function selectedLineStarts() {
        if (!ready()) {
            return [];
        }
        const surface = surfaceBridge.editorSurface;
        const selectionStart = Math.min(surface.selectionStart, surface.selectionEnd);
        const selectionEnd = Math.max(surface.selectionStart, surface.selectionEnd);
        const effectiveEnd = selectionEnd > selectionStart
                ? Math.max(selectionStart, selectionEnd - 1)
                : surface.cursorPosition;
        const starts = [];
        let lineStart = lineStartAt(selectionStart);
        const lastLineStart = lineStartAt(effectiveEnd);
        while (lineStart <= lastLineStart) {
            starts.push(lineStart);
            const lineEnd = lineEndAt(lineStart);
            if (lineEnd >= editorText().length) {
                break;
            }
            lineStart = lineEnd + 1;
        }
        return starts;
    }

    function indentSelection() {
        if (!ready()) {
            return;
        }
        const surface = surfaceBridge.editorSurface;
        const starts = selectedLineStarts();
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
        const starts = selectedLineStarts();
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
            surface.cursorPosition = Math.max(lineStartAt(cursor),
                                              cursor - removedBeforeCursor);
        }
    }

    function insertNewline() {
        if (!ready()) {
            return;
        }
        const surface = surfaceBridge.editorSurface;
        const cursor = surface.cursorPosition;
        const lineStart = lineStartAt(cursor);
        const beforeCursor = surface.text.substring(lineStart, cursor);
        const baseIndent = lineIndentAt(lineStart);
        const trimmed = beforeCursor.replace(/[ \t]+$/, "");

        const lineContent = beforeCursor.substring(baseIndent.length);
        if (lineContent.indexOf("//") === 0) {
            const continuation = baseIndent + "// ";
            surface.insert(cursor, "\n" + continuation);
            surface.cursorPosition = cursor + 1 + continuation.length;
            return;
        }

        const searchFrom = Math.max(0, cursor - 1);
        const blockStart = surface.text.lastIndexOf("/*", searchFrom);
        const blockEnd = surface.text.lastIndexOf("*/", searchFrom);
        if (blockStart >= 0 && blockStart > blockEnd) {
            const blockContent = lineContent.replace(/^[ \t]+/, "");
            const continuation = baseIndent
                    + (blockContent.indexOf("*") === 0 ? "* " : " * ");
            surface.insert(cursor, "\n" + continuation);
            surface.cursorPosition = cursor + 1 + continuation.length;
            return;
        }

        if (trimmed.endsWith("{") && surface.text.charAt(cursor) === "}") {
            const innerIndent = baseIndent + editorIndent;
            surface.insert(cursor, "\n" + innerIndent + "\n" + baseIndent);
            surface.cursorPosition = cursor + 1 + innerIndent.length;
            return;
        }

        let indent = baseIndent;
        if (trimmed.endsWith("{") || trimmed.endsWith("(")
                || trimmed.endsWith("[") || trimmed.endsWith(":")) {
            indent += editorIndent;
        }
        surface.insert(cursor, "\n" + indent);
        surface.cursorPosition = cursor + 1 + indent.length;
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
        const lineStart = lineStartAt(cursor);
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
                const openIndent = lineIndentAt(lineStartAt(opener));
                if (openIndent !== beforeCursor) {
                    surface.remove(lineStart, cursor);
                    surface.insert(lineStart, openIndent + "}");
                    surface.cursorPosition = lineStart + openIndent.length + 1;
                    return;
                }
            }
        }
        surface.insert(cursor, "}");
        surface.cursorPosition = cursor + 1;
    }

    // E3: Home alterna primeiro caractere de texto ↔ coluna 0;
    // com extendSelection a âncora oposta ao cursor é preservada.
    function smartHome(extendSelection) {
        if (!ready()) {
            return;
        }
        const surface = surfaceBridge.editorSurface;
        const cursor = surface.cursorPosition;
        const lineStart = lineStartAt(cursor);
        const firstText = lineStart + lineIndentAt(lineStart).length;
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

    // E3: pares ()/[]/{} que envolvem a seleção, numa varredura única
    // com pilha (fechador sem par no topo é ignorado — tolerante a
    // texto desbalanceado). Devolve conteúdo e par-com-delimitadores.
    function enclosingPairRanges(selectionStart, selectionEnd, text) {
        const closerToOpener = { ")": "(", "]": "[", "}": "{" };
        const stack = [];
        const ranges = [];
        for (let i = 0; i < text.length; i++) {
            const character = text.charAt(i);
            if (character === "(" || character === "[" || character === "{") {
                stack.push({ character: character, index: i });
            } else if (closerToOpener[character] !== undefined) {
                if (stack.length > 0 && stack[stack.length - 1].character
                        === closerToOpener[character]) {
                    const opener = stack.pop();
                    if (opener.index + 1 <= selectionStart && i >= selectionEnd) {
                        ranges.push({ start: opener.index + 1, end: i });
                        ranges.push({ start: opener.index, end: i + 1 });
                    }
                }
            }
        }
        return ranges;
    }

    // E3: expande para o MENOR candidato que contém estritamente a
    // seleção (palavra, linha sem indentação, linha, pares
    // envolventes, documento) — a escada JetBrains emerge sem máquina
    // de estados. Design e limitações: docs-privada/diario/18, fatia E3.
    function expandSelection() {
        if (!ready()) {
            return;
        }
        const surface = surfaceBridge.editorSurface;
        const selectionStart = Math.min(surface.selectionStart,
                                        surface.selectionEnd);
        const selectionEnd = Math.max(surface.selectionStart,
                                      surface.selectionEnd);
        const text = editorText();
        if (lastExpansion === null || lastExpansion.start !== selectionStart
                || lastExpansion.end !== selectionEnd
                || lastExpansion.length !== text.length) {
            expandHistory = [];
        }
        const candidates = [];
        if (selectionStart === selectionEnd) {
            const wordStart = wordStartAt(selectionStart);
            let wordEnd = selectionEnd;
            while (wordEnd < text.length && isWordChar(text.charAt(wordEnd))) {
                wordEnd++;
            }
            if (wordStart < wordEnd) {
                candidates.push({ start: wordStart, end: wordEnd });
            }
        }
        const lineStart = lineStartAt(selectionStart);
        const lineEnd = lineEndAt(selectionEnd);
        candidates.push({ start: lineStart + lineIndentAt(lineStart).length,
                          end: lineEnd });
        candidates.push({ start: lineStart, end: lineEnd });
        const pairs = enclosingPairRanges(selectionStart, selectionEnd, text);
        for (let i = 0; i < pairs.length; i++) {
            candidates.push(pairs[i]);
        }
        candidates.push({ start: 0, end: text.length });
        let best = null;
        for (let i = 0; i < candidates.length; i++) {
            const candidate = candidates[i];
            if (candidate.start > selectionStart
                    || candidate.end < selectionEnd
                    || (candidate.start === selectionStart
                        && candidate.end === selectionEnd)) {
                continue;
            }
            if (best === null
                    || candidate.end - candidate.start < best.end - best.start) {
                best = candidate;
            }
        }
        if (best === null) {
            return;
        }
        surface.select(best.start, best.end);
        // O TextEdit pode clampar (ex.: não seleciona o "\n" final do
        // documento); registrar a seleção REAL mantém o histórico do
        // shrink válido. Sem mudança efetiva, não vira degrau.
        const appliedStart = Math.min(surface.selectionStart,
                                      surface.selectionEnd);
        const appliedEnd = Math.max(surface.selectionStart,
                                    surface.selectionEnd);
        if (appliedStart === selectionStart && appliedEnd === selectionEnd) {
            return;
        }
        expandHistory.push({ start: selectionStart, end: selectionEnd });
        lastExpansion = { start: appliedStart, end: appliedEnd,
                          length: text.length };
    }

    // E3: volta um degrau da escada de expansão.
    function shrinkSelection() {
        if (!ready()) {
            return;
        }
        const surface = surfaceBridge.editorSurface;
        const selectionStart = Math.min(surface.selectionStart,
                                        surface.selectionEnd);
        const selectionEnd = Math.max(surface.selectionStart,
                                      surface.selectionEnd);
        const text = editorText();
        if (lastExpansion === null || lastExpansion.start !== selectionStart
                || lastExpansion.end !== selectionEnd
                || lastExpansion.length !== text.length
                || expandHistory.length === 0) {
            return;
        }
        const previous = expandHistory.pop();
        surface.select(previous.start, previous.end);
        lastExpansion = { start: previous.start, end: previous.end,
                          length: text.length };
    }

    function duplicateLineOrSelection() {
        if (!ready()) {
            return;
        }
        const surface = surfaceBridge.editorSurface;
        const selectionStart = Math.min(surface.selectionStart, surface.selectionEnd);
        const selectionEnd = Math.max(surface.selectionStart, surface.selectionEnd);
        if (selectionEnd > selectionStart) {
            const selected = surface.text.substring(selectionStart, selectionEnd);
            surface.insert(selectionEnd, selected);
            surface.select(selectionEnd, selectionEnd + selected.length);
            return;
        }
        const cursor = surface.cursorPosition;
        const start = lineStartAt(cursor);
        const end = lineEndAt(cursor);
        const line = surface.text.substring(start, end);
        surface.insert(end, "\n" + line);
        surface.cursorPosition = cursor + line.length + 1;
    }

    function moveLines(delta) {
        if (!ready()) {
            return;
        }
        const surface = surfaceBridge.editorSurface;
        const text = surface.text;
        const selectionStart = Math.min(surface.selectionStart, surface.selectionEnd);
        const selectionEnd = Math.max(surface.selectionStart, surface.selectionEnd);
        const hadSelection = selectionEnd > selectionStart;
        const cursor = surface.cursorPosition;
        const anchor = hadSelection ? Math.max(selectionStart, selectionEnd - 1) : cursor;
        const blockStart = lineStartAt(hadSelection ? selectionStart : cursor);
        const blockEnd = lineEndAt(anchor);
        const block = text.substring(blockStart, blockEnd);

        let shift = 0;
        if (delta < 0) {
            if (blockStart === 0) {
                return;
            }
            const previousStart = lineStartAt(blockStart - 1);
            const previousLine = text.substring(previousStart, blockStart - 1);
            surface.remove(previousStart, blockEnd);
            surface.insert(previousStart, block + "\n" + previousLine);
            shift = -(previousLine.length + 1);
        } else {
            if (blockEnd >= text.length) {
                return;
            }
            const nextStart = blockEnd + 1;
            const nextEnd = lineEndAt(nextStart);
            const nextLine = text.substring(nextStart, nextEnd);
            surface.remove(blockStart, nextEnd);
            surface.insert(blockStart, nextLine + "\n" + block);
            shift = nextLine.length + 1;
        }
        if (hadSelection) {
            surface.select(selectionStart + shift, selectionEnd + shift);
        } else {
            surface.cursorPosition = cursor + shift;
        }
    }

    function toggleLineComment(token) {
        if (!ready() || token === "") {
            return;
        }
        const surface = surfaceBridge.editorSurface;
        const starts = selectedLineStarts();
        if (starts.length === 0) {
            return;
        }

        const text = surface.text;
        let allCommented = true;
        let hasContent = false;
        for (let i = 0; i < starts.length; i++) {
            const line = text.substring(starts[i], lineEndAt(starts[i]));
            const content = line.replace(/^[ \t]+/, "");
            if (content === "") {
                continue;
            }
            hasContent = true;
            if (content.indexOf(token) !== 0) {
                allCommented = false;
                break;
            }
        }
        if (!hasContent) {
            return;
        }

        for (let i = starts.length - 1; i >= 0; i--) {
            const start = starts[i];
            const line = text.substring(start, lineEndAt(start));
            const indentLength = line.length - line.replace(/^[ \t]+/, "").length;
            const contentStart = start + indentLength;
            const content = line.substring(indentLength);
            if (content === "") {
                continue;
            }
            if (allCommented) {
                let removeLength = token.length;
                if (content.charAt(token.length) === " ") {
                    removeLength++;
                }
                surface.remove(contentStart, contentStart + removeLength);
            } else {
                surface.insert(contentStart, token + " ");
            }
        }
    }

    function deleteCurrentLine() {
        if (!ready()) {
            return;
        }
        const surface = surfaceBridge.editorSurface;
        const cursor = surface.cursorPosition;
        const start = lineStartAt(cursor);
        const end = lineEndAt(cursor);
        const text = surface.text;
        if (end < text.length) {
            surface.remove(start, end + 1);
        } else if (start > 0) {
            surface.remove(start - 1, end);
        } else if (end > start) {
            surface.remove(start, end);
        }
    }

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
        const end = lineEndAt(target);
        surface.cursorPosition = Math.min(target + Math.max(1, column) - 1, end);
    }
}
