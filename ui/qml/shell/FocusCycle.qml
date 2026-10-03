import QtQuick

// O CICLO DE FOCO entre as areas (0.3.9 F4, roadmap 53 §5.8): Ctrl+F6 leva o
// teclado para a PROXIMA area visivel, Ctrl+Shift+F6 para a anterior; o fim
// volta ao comeco. Ordem: o slot da esquerda (Projeto, Git ou Banco) -> o
// editor -> o slot da direita (2026-10-03: a janela cujo icone esta' no
// trilho da direita) -> o painel de baixo. Area escondida fica de fora.
//
//   [esquerda] -> [editor] -> [direita] -> [painel de baixo] --+
//        ^                                                     |
//        +-------------------- Ctrl+F6 ------------------------+
//
// Cada area expoe `focusArea()`. A area ATUAL e' a que contem o item com o
// foco ativo da janela; sem nenhuma, o ciclo comeca no editor.
QtObject {
    id: root

    property var areas: []
    property Item editorArea: null

    function visibleAreas() {
        const list = [];
        for (let i = 0; i < root.areas.length; i++) {
            if (root.areas[i] !== null && root.areas[i].visible) list.push(root.areas[i]);
        }
        return list;
    }

    // `item` esta' dentro de `area`? (sobe pela cadeia de pais)
    function contains(area, item) {
        for (let node = item; node !== null && node !== undefined; node = node.parent) {
            if (node === area) return true;
        }
        return false;
    }

    function cycle(focusedItem, direction) {
        const list = root.visibleAreas();
        if (list.length === 0) return;
        let current = -1;
        for (let i = 0; i < list.length; i++) {
            if (root.contains(list[i], focusedItem)) current = i;
        }
        let next;
        if (current < 0) {
            next = Math.max(0, list.indexOf(root.editorArea));
        } else {
            next = (current + direction + list.length) % list.length;
        }
        list[next].focusArea();
    }
}
