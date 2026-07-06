import QtQuick

Item {
    id: root

    property var surfaceBridge: null
    readonly property string editorIndent: "    "

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
        const before = Math.max(0, position - 1);
        const previousBreak = editorText().lastIndexOf("\n", before);
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
        let indent = lineIndentAt(lineStart);
        const trimmed = beforeCursor.replace(/[ \t]+$/, "");
        if (trimmed.endsWith("{") || trimmed.endsWith("(")
                || trimmed.endsWith("[") || trimmed.endsWith(":")) {
            indent += editorIndent;
        }
        surface.insert(cursor, "\n" + indent);
        surface.cursorPosition = cursor + 1 + indent.length;
    }
}
