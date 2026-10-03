import QtQuick

// O MODO FOCO (0.3.9 F4, roadmap 53 §5.8): recolhe a area da esquerda, os
// Simbolos e o painel de baixo, e guarda como estavam; sair restaura
// EXATAMENTE o anterior. Abrir um painel a' mao durante o Foco encerra o modo
// SEM restaurar — quem mexeu foi o usuario, e o layout dele vale.
//
//   normal --toggle--> foco (guarda {explorador, baixo, simbolos}; recolhe)
//   foco   --toggle--> normal (restaura o guardado)
//   foco   --painel aberto a' mao--> normal (sem restaurar)
QtObject {
    id: root

    property var shell: null
    property bool active: false
    property var saved: null
    property bool applying: false

    function toggle() {
        root.applying = true;
        if (root.active) {
            const before = root.saved;
            root.active = false;
            root.saved = null;
            root.shell.showExplorer = before.showExplorer;
            root.shell.showBottomPanel = before.showBottomPanel;
            root.shell.outlineCollapsed = before.outlineCollapsed;
        } else {
            root.saved = { showExplorer: root.shell.showExplorer,
                           showBottomPanel: root.shell.showBottomPanel,
                           outlineCollapsed: root.shell.outlineCollapsed };
            root.active = true;
            root.shell.showExplorer = false;
            root.shell.showBottomPanel = false;
            root.shell.outlineCollapsed = true;
        }
        root.applying = false;
        root.shell.persistLayoutSoon();
    }

    // Um painel ABRIU: se nao foi o proprio modo, o Foco acaba sem restaurar.
    function panelOpened(opened) {
        if (root.active && opened && !root.applying) {
            root.active = false;
            root.saved = null;
        }
    }
}
