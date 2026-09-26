pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A COLUNA DE SIMBOLOS E O SPLITTER DELA (extraida do EditorPane, 2026-09-25).
//
// Saiu de la' porque o painel chegou ao limite de 300 linhas da catraca ao
// ganhar o lado a lado do Markdown. Os dois formam UMA coisa: o splitter nao
// existe sem a coluna, mede exatamente a largura dela e some junto. Separar
// aqui foi o que a catraca existe para forcar.
//
// A ALCA DE REABRIR VEM JUNTO, e por isso este Item nunca fica invisivel: ele
// encolhe para largura ZERO. Assim a borda esquerda dele E' a borda direita do
// painel quando fechado, e quem se ancora nela nao precisa de condicional
// nenhuma — o editor e a previa simplesmente se ancoram a `outlineSide.left`.
Item {
    id: root

    property var items: []
    property var symbols: null
    property real panelWidth: 220
    property bool expanded: false
    property bool available: false

    signal collapseRequested()
    signal openRequested(int line, int column)
    signal resizeRequested(real delta)
    signal resetRequested()

    function focusSearch(query) {
        outlinePanel.focusSearch(query);
    }

    // A largura inclui o vao do splitter: quem se ancora a' esquerda desta
    // coluna nao precisa saber que ele existe.
    width: root.expanded ? root.panelWidth + Theme.panelGap : 0

    PanelSplitter {
        visible: root.expanded
        anchors.left: parent.left
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        width: Theme.panelGap
        onDragged: function(delta) {
            root.resizeRequested(-delta);
        }
        onResetRequested: root.resetRequested()
    }

    EditorOutlinePanel {
        id: outlinePanel

        visible: root.expanded
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.rightMargin: Theme.spacingSmall
        anchors.bottomMargin: Theme.spacingSmall
        width: root.panelWidth
        items: root.items
        symbols: root.symbols
        onCollapseRequested: root.collapseRequested()
        onOpenRequested: function(line, column) {
            root.openRequested(line, column);
        }
    }

    EditorOutlineHandle {
        visible: root.available && !root.expanded
        anchors.right: parent.right
        anchors.rightMargin: Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        z: 18
        onExpandRequested: root.collapseRequested()
    }
}
