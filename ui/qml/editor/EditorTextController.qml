import QtQuick

Item {
    id: root

    property var surfaceBridge: null
    readonly property string editorIndent: "    "

    visible: false

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

    function insertNewline() {
        if (!ready()) {
            return;
        }
        const surface = surfaceBridge.editorSurface;
        const cursor = surface.cursorPosition;
        const lineStart = geo.lineStartAt(cursor);
        const beforeCursor = surface.text.substring(lineStart, cursor);
        const baseIndent = geo.lineIndentAt(lineStart);
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
        const start = geo.lineStartAt(cursor);
        const end = geo.lineEndAt(cursor);
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
        const blockStart = geo.lineStartAt(hadSelection ? selectionStart : cursor);
        const blockEnd = geo.lineEndAt(anchor);
        const block = text.substring(blockStart, blockEnd);

        let shift = 0;
        if (delta < 0) {
            if (blockStart === 0) {
                return;
            }
            const previousStart = geo.lineStartAt(blockStart - 1);
            const previousLine = text.substring(previousStart, blockStart - 1);
            surface.remove(previousStart, blockEnd);
            surface.insert(previousStart, block + "\n" + previousLine);
            shift = -(previousLine.length + 1);
        } else {
            if (blockEnd >= text.length) {
                return;
            }
            const nextStart = blockEnd + 1;
            const nextEnd = geo.lineEndAt(nextStart);
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
        const starts = geo.selectedLineStarts();
        if (starts.length === 0) {
            return;
        }

        const text = surface.text;
        let allCommented = true;
        let hasContent = false;
        for (let i = 0; i < starts.length; i++) {
            const line = text.substring(starts[i], geo.lineEndAt(starts[i]));
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
            const line = text.substring(start, geo.lineEndAt(start));
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
        const start = geo.lineStartAt(cursor);
        const end = geo.lineEndAt(cursor);
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
        const end = geo.lineEndAt(target);
        surface.cursorPosition = Math.min(target + Math.max(1, column) - 1, end);
    }
}
