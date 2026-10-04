import QtQuick

// PEGAR O FOCO AO ABRIR E DEVOLVER AO FECHAR, para popover e dialogo
// (2026-10-04). Sem foco, o Esc nao chega (o menu de toolchain e o do Python
// so' fechavam com clique fora); com foco e sem devolver, o popover INVISIVEL
// fica com ele, e o que se digita vai para lugar nenhum — o defeito achado no
// AppMenuPopup no mesmo dia. Uso:
//
//   KvFocusKeeper { id: keeper; target: root }
//   onVisibleChanged: visible ? keeper.take() : keeper.giveBack()
//   Keys.onEscapePressed: root.dismissRequested()
QtObject {
    id: root

    property Item target: null
    property Item previous: null

    function take() {
        if (root.target === null) return;
        const window = root.target.Window.window;
        if (window !== null && !root.target.activeFocus) root.previous = window.activeFocusItem;
        root.target.forceActiveFocus();
    }

    function giveBack() {
        if (root.target === null || !root.target.activeFocus) return;
        if (root.previous !== null && root.previous.visible && root.previous.enabled) root.previous.forceActiveFocus();
        if (root.target.activeFocus) root.target.focus = false;
        root.previous = null;
    }
}
