import QtQuick

// AS OPERACOES SOBRE LINHAS INTEIRAS: duplicar, mover, comentar e apagar.
//
// Saiu do `EditorTextController` em 2026-09-26, quando a correcao estrutural da
// indentacao (E1) o levou a 419 linhas contra o limite de 400. E' uma
// responsabilidade inteira, e nao um pedaco cortado no tamanho: as quatro
// operam sobre a LINHA (ou o bloco selecionado) como unidade, todas preservam
// selecao e cursor, e nenhuma delas olha para o caractere sob o cursor.
//
// O que ficou no dono do texto e' o que age no PONTO: indentar selecao, nova
// linha, `}`, Home inteligente.
Item {
    id: root

    property var surfaceBridge: null
    property var geometry: null
    property string editorIndent: "    "

    visible: false

    function ready() {
        return surfaceBridge !== null && surfaceBridge.ready() && geometry !== null;
    }

    function editorText() {
        return ready() ? surfaceBridge.text() : "";
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
        const start = geometry.lineStartAt(cursor);
        const end = geometry.lineEndAt(cursor);
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
        const blockStart = geometry.lineStartAt(hadSelection ? selectionStart : cursor);
        const blockEnd = geometry.lineEndAt(anchor);
        const block = text.substring(blockStart, blockEnd);

        let shift = 0;
        if (delta < 0) {
            if (blockStart === 0) {
                return;
            }
            const previousStart = geometry.lineStartAt(blockStart - 1);
            const previousLine = text.substring(previousStart, blockStart - 1);
            surface.remove(previousStart, blockEnd);
            surface.insert(previousStart, block + "\n" + previousLine);
            shift = -(previousLine.length + 1);
        } else {
            if (blockEnd >= text.length) {
                return;
            }
            const nextStart = blockEnd + 1;
            const nextEnd = geometry.lineEndAt(nextStart);
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
        const starts = geometry.selectedLineStarts();
        if (starts.length === 0) {
            return;
        }

        const text = surface.text;
        let allCommented = true;
        let hasContent = false;
        for (let i = 0; i < starts.length; i++) {
            const line = text.substring(starts[i], geometry.lineEndAt(starts[i]));
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
            const line = text.substring(start, geometry.lineEndAt(start));
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
        const start = geometry.lineStartAt(cursor);
        const end = geometry.lineEndAt(cursor);
        const text = surface.text;
        if (end < text.length) {
            surface.remove(start, end + 1);
        } else if (start > 0) {
            surface.remove(start - 1, end);
        } else if (end > start) {
            surface.remove(start, end);
        }
    }
}
