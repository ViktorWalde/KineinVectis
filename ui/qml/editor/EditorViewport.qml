import QtQuick

// O QUE FICA VISIVEL no editor: seguir o cursor e a pagina (16.2,
// roadmaps/59 §7.1.2). Geometria de rolagem, sem pintura e sem texto; a
// superficie (EditorTextSurface) so' liga o `Flickable` e o `TextEdit`.
//
// POR QUE A PAGINA MORA AQUI. O `TextEdit` do Qt Quick nao tem PageUp/PageDown:
// o `QQuickTextControl` ignora a tecla, porque nao conhece a area de rolagem
// (o `QWidgetTextControl` a calcula pelo `QAbstractScrollArea`). Quem conhece
// e' quem rola. O `ensureVisible` veio junto, da superficie: e' a mesma
// pergunta ("o que a tela mostra"), e um harness consegue exercitar os dois
// com dubles de cinco campos.
QtObject {
    id: root

    property var flick: null       // o Flickable (contentX/Y, width, height, contentHeight)
    property var textEdit: null    // o TextEdit (cursorRectangle, positionAt, ...)
    property real lineHeight: 1

    // Rola o minimo para `rect` (o retangulo do cursor) caber na tela.
    function ensureVisible(rect) {
        if (root.flick === null) return;
        if (root.flick.contentX >= rect.x) {
            root.flick.contentX = rect.x;
        } else if (root.flick.contentX + root.flick.width <= rect.x + rect.width) {
            root.flick.contentX = rect.x + rect.width - root.flick.width;
        }
        if (root.flick.contentY >= rect.y) {
            root.flick.contentY = rect.y;
        } else if (root.flick.contentY + root.flick.height <= rect.y + rect.height) {
            root.flick.contentY = rect.y + rect.height - root.flick.height;
        }
    }

    // A conta da pagina, sem estado. A pagina e' o numero de linhas inteiras
    // visiveis (no minimo uma). A tela anda uma pagina, presa entre o topo e o
    // fim; o cursor anda a mesma pagina, presa na primeira e na ultima linha:
    // no fim do arquivo a tela para e o cursor ainda vai a' ultima linha, como
    // no JetBrains. `y` e' o meio da linha de destino, para o `positionAt`.
    function pageTarget(direction, cursorY, contentY, viewportHeight, textHeight, flickHeight, line) {
        const step = Math.max(1, line);
        const page = Math.max(1, Math.floor(viewportHeight / step)) * step;
        const lastScroll = Math.max(0, flickHeight - viewportHeight);
        const lastLineTop = Math.max(0, textHeight - step);
        return {
            contentY: Math.max(0, Math.min(lastScroll, contentY + direction * page)),
            y: Math.max(0, Math.min(lastLineTop, cursorY + direction * page)) + step / 2
        };
    }

    // PageUp (-1) ou PageDown (+1); com `extendSelection`, a selecao cresce
    // ate' o destino (Shift+Page).
    function page(direction, extendSelection) {
        if (root.flick === null || root.textEdit === null) return;
        const cursor = root.textEdit.cursorRectangle;
        const goal = root.pageTarget(direction, cursor.y, root.flick.contentY, root.flick.height,
                                     root.textEdit.contentHeight, root.flick.contentHeight,
                                     root.lineHeight);
        root.flick.contentY = goal.contentY;
        const position = root.textEdit.positionAt(cursor.x, goal.y);
        if (extendSelection) root.textEdit.moveCursorSelection(position, TextEdit.SelectCharacters);
        else root.textEdit.cursorPosition = position;
    }
}
