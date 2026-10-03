import QtQuick
import KineinVectis

// O SLOT DA DIREITA (2026-10-03): onde a janela acoplada abre quando o icone
// dela esta' no trilho da direita (ShellDocks). Vazio por si — a janela e' a
// mesma do slot da esquerda, so' muda de pai —, com a alca de largura no vao
// a esquerda dele.
Item {
    id: root

    property var shellController: null
    property bool workspaceOpen: false
    // Quem tem as janelas (ShellLeftWindowHost): o Ctrl+F6 entra por ele.
    property var windowHost: null

    function focusArea() {
        root.windowHost.focusSlot("right");
    }

    width: visible ? root.shellController.rightWidth : 0
    // Os Simbolos abertos escondem o slot por um momento (ShellDocks.showing).
    visible: root.workspaceOpen && root.shellController.rightWindow !== ""
             && root.shellController.effectiveOutlineCollapsed

    PanelSplitter {
        x: -width
        width: Theme.panelGap
        height: parent.height
        onDragged: delta => root.shellController.resizeRightDock(-delta)
    }
}
