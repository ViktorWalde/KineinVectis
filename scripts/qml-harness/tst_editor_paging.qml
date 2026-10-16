import QtQuick
import KineinVectis

// PageUp/PageDown no editor (16.2, roadmaps/59 §7.1.2): o EditorTypingController
// e o EditorViewport REAIS, com dubles do Flickable e do TextEdit. Linhas de
// 20 px; o duble do `positionAt` devolve linha * 1000 + x, para a conta ser
// legivel no teste.
Item {
    id: root

    property int failures: 0
    property var pages: []
    property int dismissed: 0

    function check(condition, label) {
        if (!condition) { root.failures++; console.error("FALHOU: " + label); }
    }

    QtObject {
        id: fakeSurface
        property string text: ""
        property int cursorPosition: 0
        property int selectionStart: 0
        property int selectionEnd: 0
    }
    EditorTypingController {
        id: typing
        surface: fakeSurface
        onPageRequested: (direction, extend) => root.pages = root.pages.concat([[direction, extend]])
        onCompletionDismissRequested: root.dismissed++
    }

    QtObject {
        id: fakeFlick
        property real contentX: 0
        property real contentY: 0
        property real width: 400
        property real height: 200
        property real contentHeight: 6020
    }
    QtObject {
        id: fakeText
        property var cursorRectangle: ({ x: 30, y: 40, width: 1, height: 20 })
        property real contentHeight: 6000
        property int cursorPosition: 0
        property var selectedTo: null
        function positionAt(x, y) { return Math.floor(y / 20) * 1000 + x; }
        function moveCursorSelection(position, mode) { fakeText.selectedTo = position; fakeText.cursorPosition = position; }
    }
    EditorViewport {
        id: viewport
        flick: fakeFlick
        textEdit: fakeText
        lineHeight: 20
    }

    function key(code, modifiers) { return { key: code, modifiers: modifiers, text: "" }; }

    Component.onCompleted: {
        // A tecla: sem modificador e com Shift, e' pagina; com Ctrl, nao.
        root.check(typing.route(key(Qt.Key_PageDown, Qt.NoModifier)), "PageDown consumido");
        root.check(typing.route(key(Qt.Key_PageUp, Qt.ShiftModifier)), "Shift+PageUp consumido");
        root.check(!typing.route(key(Qt.Key_PageDown, Qt.ControlModifier)), "Ctrl+PageDown segue adiante");
        root.check(JSON.stringify(root.pages) === "[[1,false],[-1,true]]", "pedidos " + JSON.stringify(root.pages));
        typing.completionVisible = true;
        typing.route(key(Qt.Key_PageDown, Qt.NoModifier));
        root.check(root.dismissed === 1 && root.pages.length === 3, "com a completacao aberta, fecha e pagina");
        typing.completionVisible = false;

        // A pagina: 10 linhas visiveis; tela e cursor andam 200 px.
        viewport.page(1, false);
        root.check(fakeFlick.contentY === 200, "a tela anda uma pagina: " + fakeFlick.contentY);
        root.check(fakeText.cursorPosition === 12030, "o cursor anda uma pagina, mesma coluna: " + fakeText.cursorPosition);

        // No fim: a tela para no ultimo quadro; o cursor vai a' ultima linha.
        fakeFlick.contentY = 5800;
        fakeText.cursorRectangle = { x: 30, y: 5900, width: 1, height: 20 };
        viewport.page(1, false);
        root.check(fakeFlick.contentY === 5820, "a tela para no fim: " + fakeFlick.contentY);
        root.check(fakeText.cursorPosition === 299030, "o cursor na ultima linha: " + fakeText.cursorPosition);

        // Com Shift, no comeco: a selecao vai ate' a primeira linha.
        fakeFlick.contentY = 100;
        fakeText.cursorRectangle = { x: 30, y: 120, width: 1, height: 20 };
        viewport.page(-1, true);
        root.check(fakeFlick.contentY === 0 && fakeText.selectedTo === 30, "Shift+PageUp seleciona ate' o topo: " + fakeText.selectedTo);

        // Arquivo menor que a tela: nada rola, o cursor vai ao fim.
        fakeFlick.contentHeight = 120; fakeText.contentHeight = 100; fakeFlick.contentY = 0;
        fakeText.cursorRectangle = { x: 30, y: 0, width: 1, height: 20 };
        viewport.page(1, false);
        root.check(fakeFlick.contentY === 0 && fakeText.cursorPosition === 4030, "arquivo curto: " + fakeText.cursorPosition);

        // O ensureVisible que veio da superficie: o cursor abaixo da tela rola o minimo.
        fakeFlick.contentHeight = 6020; fakeFlick.contentY = 0;
        viewport.ensureVisible({ x: 30, y: 400, width: 1, height: 20 });
        root.check(fakeFlick.contentY === 220, "segue o cursor: " + fakeFlick.contentY);
        Qt.exit(root.failures === 0 ? 0 : 1);
    }
}
